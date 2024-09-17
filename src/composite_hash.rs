//! Composite hashes.

use crate::prelude::Precision;
use core::fmt::Debug;

pub mod gaps;
pub use gaps::GapHash;
pub mod switch;
pub use switch::SwitchHash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enumeration of errors that can occur when downgrading a composite hash.
pub enum SaturationError {
    /// The underlying vector that contains the hash list is saturated but can be extended.
    ExtendableSaturation,
    /// The Hash List is saturated and cannot be downgraded.
    Saturation(u32),
}

/// A reader buffer with a known last buffered bit.
pub trait LastBufferedBit {
    /// Returns the last buffered bit.
    fn last_buffered_bit(&self) -> u32;

    /// Returns the hash bits.
    fn hash_bits(&self) -> u8;
}

#[cfg(test)]
mod test_composite_hash {
    use core::u64;

    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_precisions_and_bits;

    #[allow(unsafe_code)]
    #[cfg(feature = "std")]
    fn test_composite_hash<P: Precision + PackedRegister<B>, B: Bits>() {
        let mut random_state = 498_123_456_789;
        let number_of_iterations = core::cmp::min(1, 10_000 / (1 << (P::EXPONENT - 4)));

        for _ in 0..number_of_iterations {
            let number_of_bits = (1 << P::EXPONENT) * usize::from(B::NUMBER_OF_BITS);

            let number_of_hashes =
                number_of_bits / usize::from(GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS);

            assert!(number_of_bits >= 64);
            assert!(number_of_hashes > 2);

            // We create an array where we will store the hashes.
            let mut reference_hashes = vec![0; number_of_hashes * 6];
            // Since we do not have an array that keeps track of the number of hashes we have inserted, we will use a counter.
            let mut number_of_inserted_hashes = 0;
            // We allow a relatively small number of bits for the hash, so to force the degradation and saturation events.
            let mut encoded_hashes = vec![u64::MAX; number_of_bits.div_ceil(64)];
            let mut encoded_hashes: &mut [u8] = unsafe {
                core::slice::from_raw_parts_mut(
                    encoded_hashes.as_mut_ptr() as *mut u8,
                    encoded_hashes.len() * 8,
                )
            };
            // We store the maximal position where we have inserted a hash.
            let mut writer_tell = 0;

            // We start from the maximal number of bits for the hash.
            let mut hash_bits = GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS;

            // Flag to check whether saturation was reached.
            let mut saturation_reached = false;

            random_state = splitmix64(random_state);
            for random_value in
                iter_random_values::<u64>(number_of_hashes as u64 * 20, None, Some(random_state))
            {
                // We start each iteration by checking that the hashes are consistent.
                for (reference_hash, hash) in
                    reference_hashes
                        .iter()
                        .copied()
                        .zip(GapHash::<P, B>::downgraded(
                            &encoded_hashes,
                            number_of_inserted_hashes,
                            hash_bits,
                            writer_tell,
                            0,
                        ))
                {
                    assert_eq!(
                        reference_hash, hash,
                        "The reference hash ({reference_hash:064b}) does not match the hash ({hash:064b}). Working with hash bits {hash_bits}.",
                    );
                }

                let (index, register, original_hash) = <HyperLogLog<
                    P,
                    B,
                    <P as PackedRegister<B>>::Array,
                >>::index_and_register_and_hash(
                    &random_value
                );

                let mut reference_encoded_hash =
                    GapHash::<P, B>::encode(index, register, original_hash, hash_bits);

                let result = GapHash::<P, B>::insert_sorted_desc(
                    &mut encoded_hashes,
                    number_of_inserted_hashes,
                    writer_tell,
                    index,
                    register,
                    original_hash,
                    hash_bits,
                );

                if let Err(SaturationError::Saturation(_)) = result {
                    saturation_reached = true;
                    break;
                }

                if let Ok(Some(insert_metadata)) = result {
                    // If the hash was inserted, there must NOT be a reference stored with the same hash.
                    assert!(!reference_hashes[..number_of_inserted_hashes as usize]
                        .contains(&reference_encoded_hash));
                    reference_hashes[number_of_inserted_hashes as usize] = reference_encoded_hash;
                    number_of_inserted_hashes += 1;
                    // We sort by decreasing order so that we can use the binary search.
                    reference_hashes[..number_of_inserted_hashes as usize]
                        .sort_unstable_by(|a, b| b.cmp(a));

                    if insert_metadata.hash_bits != hash_bits {
                        let mut last_reference_hash = u32::MAX;
                        let mut reference_duplicates = 0;
                        for i in 0..number_of_inserted_hashes {
                            let downgraded_hash = GapHash::<P, B>::downgrade(
                                reference_hashes[i as usize],
                                hash_bits,
                                hash_bits - insert_metadata.hash_bits,
                            );
                            if downgraded_hash == last_reference_hash {
                                reference_duplicates += 1;
                                continue;
                            }
                            last_reference_hash = downgraded_hash;
                            reference_hashes[i as usize - reference_duplicates] = downgraded_hash;
                        }

                        number_of_inserted_hashes -= insert_metadata.duplicates;

                        // We set the values after 'number_of_inserted_hashes' to u64::MAX, so that
                        // if we end up comparing with such a value we can notice the error immediately.
                        for i in number_of_inserted_hashes..number_of_hashes as u32 {
                            reference_hashes[i as usize] = u32::MAX;
                        }

                        assert_eq!(
                            insert_metadata.duplicates as usize,
                            reference_duplicates,
                            "The number of duplicates ({}) does not match the number of duplicates in the reference ({reference_duplicates}).",
                            insert_metadata.duplicates
                        );

                        hash_bits = insert_metadata.hash_bits;

                        // When we downgrade during an insertion, the encoded hash is downgraded to the new
                        // hash bits, so we need to downgrade it to the original hash bits.
                        reference_encoded_hash =
                            GapHash::<P, B>::encode(index, register, original_hash, hash_bits);
                    }

                    writer_tell = insert_metadata.bit_index;

                    // We check that the inserted hash appears among the inserted hashes.
                    assert!(
                        GapHash::<P, B>::downgraded(
                            &encoded_hashes,
                            number_of_inserted_hashes,
                            hash_bits,
                            writer_tell,
                            0
                        )
                        .any(|hash| hash == reference_encoded_hash),
                        "Hash {reference_encoded_hash} was not found after insertion. We have inserted {number_of_inserted_hashes} hashes so far."
                    );

                    // If we attempt to insert the same hash again, it should not be inserted.
                    assert_eq!(
                        GapHash::<P, B>::insert_sorted_desc(
                            &mut encoded_hashes,
                            number_of_inserted_hashes,
                            writer_tell,
                            index,
                            register,
                            original_hash,
                            hash_bits,
                        ),
                        Ok(None),
                        "After having inserted the hash ({reference_encoded_hash:064b}), it was inserted again. Working with hash bits {hash_bits}.",
                    );
                }

                // After inserting the hash, we check if the hash is found.
                assert!(
                    GapHash::<P, B>::find(
                        encoded_hashes,
                        number_of_inserted_hashes,
                        index,
                        register,
                        original_hash,
                        hash_bits,
                        writer_tell
                    ),
                    concat!(
                        "Failed to find the hash after inserting it. ",
                        "The register is {} and the index is {}, and the fake hash is {}. ",
                        "The number of bits for the hash is {}, precision is {} and the number of bits is {}. ",
                        "In the reference hashes it is at position {}."
                    ),
                    register,
                    index,
                    original_hash,
                    hash_bits,
                    P::EXPONENT,
                    B::NUMBER_OF_BITS,
                    reference_hashes
                        .iter()
                        .position(|&hash| hash == reference_encoded_hash)
                        .unwrap(),
                );
            }

            assert!(saturation_reached);
        }
    }

    #[allow(unsafe_code)]
    fn test_composite_hash_stateless_operations<P: Precision + PackedRegister<B>, B: Bits>() {
        const NUMBER_OF_HASHES: usize = 1_000;
        // We start from the maximal number of bits for the hash.
        for hash_bits in
            GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS..=GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS
        {
            for random_value in iter_random_values::<u64>(NUMBER_OF_HASHES as u64, None, None) {
                let (index, register, original_hash) = <HyperLogLog<
                    P,
                    B,
                    <P as PackedRegister<B>>::Array,
                >>::index_and_register_and_hash(
                    &random_value
                );

                let encoded_hash =
                    GapHash::<P, B>::encode(index, register, original_hash, hash_bits);

                // We check that the encoded hash indeed fits within the provided number of bits.
                assert_eq!(
                    u64::from(encoded_hash) >> hash_bits,
                    0,
                    "The encoded hash is too large for the provided number of bits."
                );

                let (decoded_register, decoded_index) =
                    GapHash::<P, B>::decode(encoded_hash, hash_bits);
                assert_eq!(register, decoded_register, "Failed to recover the register at hash bits {hash_bits}. The hash is {encoded_hash:b}. The fake hash is {original_hash:064b}.");
                assert_eq!(index, decoded_index, "Failed to recover the index at hash bits {hash_bits}. The hash is {encoded_hash:b}. The fake hash is {original_hash:064b}.");

                let downgraded_hash = GapHash::<P, B>::downgrade(encoded_hash, hash_bits, 0);
                assert_eq!(
                    encoded_hash, downgraded_hash,
                    "Failed to downgrade by 0 bits. The hash is {encoded_hash:064b}."
                );

                for shift in 1..32 {
                    if hash_bits.saturating_sub(shift) < GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS
                    {
                        break;
                    }
                    let target_hash_bits = hash_bits - shift;
                    let downgraded_hash =
                        GapHash::<P, B>::downgrade(encoded_hash, hash_bits, shift);
                    let (downgraded_register, downgraded_index) =
                        GapHash::<P, B>::decode(downgraded_hash, target_hash_bits);

                    assert_eq!(register, downgraded_register, "Failed to downgrade the register from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}.");
                    assert_eq!(index, downgraded_index, "Failed to downgrade the index from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}.");

                    let encoded_target_hash =
                        GapHash::<P, B>::encode(index, register, original_hash, target_hash_bits);

                    assert_eq!(downgraded_hash, encoded_target_hash, "Downgraded from hash bits {hash_bits} ({encoded_hash:b}) to {target_hash_bits} hash directly encoded ({encoded_target_hash:b}) and downgraded hash do not match ({downgraded_hash:b}). The original hash is {original_hash:064b}.");
                }
            }
        }
    }

    #[test_precisions_and_bits]
    fn test_gap_switch_hash<P: Precision, B: Bits>()
    where
        P: PackedRegister<B>,
    {
        test_composite_hash_stateless_operations::<P, B>();

        #[cfg(feature = "std")]
        test_composite_hash::<P, B>();
    }
}
