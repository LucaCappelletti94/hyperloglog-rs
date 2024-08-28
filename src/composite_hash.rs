//! Composite hashes.

use crate::{bits::Bits, prelude::Precision};

pub mod current;
pub mod gaps;
mod shared;
pub mod switch;
pub use current::CurrentHash;
pub use gaps::GapHash;
pub use switch::SwitchHash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enumeration of errors that can occur when downgrading a composite hash.
pub enum CompositeHashError {
    /// The Hash List is saturated but can be downgraded.
    DowngradableSaturation,
    /// The Hash List is saturated and cannot be downgraded.
    Saturation,
}

/// A composite hash is a 64-bit hash that encodes a register, index and original hash.
pub trait CompositeHash: Send + Sync {
    /// The precision of the hyperloglog.
    type Precision: Precision;
    /// The number of bits for the hash.
    type Bits: Bits;

    /// Iterator on the decoded indices and registers.
    type Decoded<'a>: Iterator<Item = (u8, usize)>;

    /// Iterator on the downgraded composite hashes.
    type Downgraded<'a>: Iterator<Item = u64>;

    /// Returns an iterator on the decoded indices and registers.
    fn decoded<'a>(hashes: &'a [u8], number_of_hashes: usize, hash_bits: u8) -> Self::Decoded<'a>;

    /// Returns an iterator on the downgraded composite hashes.
    ///
    /// # Arguments
    /// * `hashes` - The slice of composite hashes to downgrade.
    /// * `number_of_hashes` - The number of hashes stored in the slice.
    /// * `hash_bits` - The number of bits for the hash to downgrade.
    /// * `shift` - The number of bits to shift the hash to the right.
    fn downgraded<'a>(
        hashes: &'a [u8],
        number_of_hashes: usize,
        hash_bits: u8,
        shift: u8,
    ) -> Self::Downgraded<'a>;

    /// Encodes the provided register, index and original hash into a single 64-bit hash.
    ///
    /// # Arguments
    /// * `index` - The index of the register.
    /// * `register` - The register from the hyperloglog, which is the leading number of bits + 1.
    /// * `original_hash` - The original hash that was used to generate the register.
    /// * `hash_bits` - The number of bits for the hash to encode.
    fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u64;

    /// Returns a result with the index, in bits, where the provided index, register and original hash are stored or
    /// in the case of a failure, the index, in bits, where the hash should be inserted and the encoded hash.
    ///
    /// # Arguments
    /// * `hashes` - The slice of composite hashes to search for the hash.
    /// * `number_of_hashes` - The number of hashes stored in the slice.
    /// * `index` - The index of the register.
    /// * `register` - The register from the hyperloglog, which is the leading number of bits + 1.
    /// * `original_hash` - The original hash that was used to generate the register.
    /// * `hash_bits` - The number of bits for the hash to encode.
    ///
    /// # Returns
    /// An option with the index, in bits, where the provided index, register and original hash are stored.
    fn find<'a>(
        hashes: &'a [u8],
        number_of_hashes: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> Result<usize, (usize, u64)>;

    /// Inserts the provided index, register and original hash into the provided slice of composite hashes,
    /// keeping the hashes sorted by index and register, and returns the bit index where (if) the hash was inserted.
    ///
    /// # Arguments
    /// * `hashes` - The slice of composite hashes to insert the hash into.
    /// * `number_of_hashes` - The number of hashes stored in the slice.
    /// * `bit_index` - The index, in bits, where the writer left off.
    /// * `index` - The index of the register.
    /// * `register` - The register from the hyperloglog, which is the leading number of bits + 1.
    /// * `original_hash` - The original hash that was used to generate the register.
    /// * `hash_bits` - The number of bits for the hash to encode.
    fn insert_sorted_desc<'a>(
        hashes: &'a mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<usize>, CompositeHashError>;

    /// Decodes the provided composite hash into the register and index.
    ///
    /// # Arguments
    /// * `hash` - The composite hash to decode.
    /// * `hash_bits` - The number of bits for the hash to decode.
    fn decode(hash: u64, hash_bits: u8) -> (u8, usize);

    /// Downgrades the provided composite hash into a smaller composite hash.
    ///
    /// # Arguments
    /// * `hash` - The composite hash to downgrade.
    /// * `hash_bits` - The number of bits for the hash to downgrade.
    /// * `shift` - The number of bits to shift the hash to the right.
    fn downgrade(hash: u64, hash_bits: u8, shift: u8) -> u64;

    #[must_use]
    /// Downgrades inplace the entire slice of composite hashes into a smaller composite hash.
    ///
    /// # Arguments
    /// * `hashes` - The slice of composite hashes to downgrade.
    /// * `number_of_hashes` - The number of hashes stored in the slice.
    /// * `bit_index` - The index, in bits, where the writer left off.
    /// * `hash_bits` - The number of bits for the hash to downgrade.
    /// * `shift` - The number of bits to shift the hash to the right.
    ///
    /// # Implementative details
    /// When downgrading an hash, it is possible that the resulting hashes may result duplicated.
    /// Such cases are to be removed, and the number of removed hashes is returned.
    ///
    /// # Returns
    /// The number of newly created duplicated hashes that were removed and the new writer tell.
    fn downgrade_inplace<'a>(
        hashes: &'a mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        hash_bits: u8,
        shift: u8,
    ) -> (u32, usize);

    /// Returns the smallest viable hash for the current precision and number of bits.
    const SMALLEST_VIABLE_HASH_BITS: u8;
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test_composite_hash {
    use core::u64;

    use super::*;
    use crate::prelude::*;
    use gaps::PrefixFreeCode;
    use hyperloglog_derive::test_precisions_and_bits;

    #[allow(unsafe_code)]
    fn test_composite_hash<CH: CompositeHash>()
    where
        CH::Precision: ArrayRegister<CH::Bits>,
    {
        let number_of_hashes = 1_000;

        let mut all_hash_bytes = (CH::SMALLEST_VIABLE_HASH_BITS / 8..=4).collect::<Vec<u8>>();
        all_hash_bytes.sort_unstable();

        for hash_bytes in all_hash_bytes {
            let mut reference_hashes: Vec<u64> = Vec::with_capacity(number_of_hashes);
            let mut encoded_hashes =
                vec![0_u64; ceil(number_of_hashes * usize::from(hash_bytes), 8)];
            let mut encoded_hashes: &mut [u8] = unsafe {
                core::slice::from_raw_parts_mut(
                    encoded_hashes.as_mut_ptr() as *mut u8,
                    encoded_hashes.len() * 8,
                )
            };
            let mut writer_tell = 0;

            for random_value in iter_random_values::<u64>(number_of_hashes as u64, None, None) {
                let (index, register, original_hash) =
                    <PlusPlus<
                        CH::Precision,
                        CH::Bits,
                        <CH::Precision as ArrayRegister<CH::Bits>>::Packed,
                    >>::hash_and_register_and_index(&random_value);

                let hash_bits = hash_bytes * 8;
                let encoded_hash = CH::encode(index, register, original_hash, hash_bits);

                // We check that the encoded hash indeed fits within the provided number of bits.
                assert_eq!(encoded_hash >> hash_bits, 0);

                let (decoded_register, decoded_index) = CH::decode(encoded_hash, hash_bits);
                assert_eq!(register, decoded_register, "Failed to recover the register at hash bits {hash_bits}. The hash is {encoded_hash:b}. The fake hash is {original_hash:064b}.");
                assert_eq!(index, decoded_index, "Failed to recover the index at hash bits {hash_bits}. The hash is {encoded_hash:b}. The fake hash is {original_hash:064b}.");

                match CH::find(
                    encoded_hashes,
                    reference_hashes.len(),
                    index,
                    register,
                    original_hash,
                    hash_bits,
                ) {
                    Ok(_) => {
                        // If the hash is found, while the position itself may vary depending on the implementation,
                        // it certainly needs to exist in the reference hashes.
                        assert_eq!(reference_hashes.contains(&encoded_hash), true);
                    }
                    Err((_, encoded_hash_we_would_insert)) => {
                        assert_eq!(encoded_hash, encoded_hash_we_would_insert);
                        // If the hash is not found, while the position itself may vary depending on the implementation,
                        // it certainly needs to NOT exist in the reference hashes.
                        assert!(
                                    !reference_hashes.contains(&encoded_hash),
                                    concat!(
                                        "The hash was found in the reference hashes, but not in the encoded hashes. ",
                                        "The register is {} and the index is {}, and the fake hash is {}. ",
                                        "The number of bits for the hash is {}, precision is {} and the number of bits is {}."
                                    ),
                                    register,
                                    index,
                                    original_hash,
                                    hash_bits,
                                    CH::Precision::EXPONENT,
                                    CH::Bits::NUMBER_OF_BITS,
                                );
                    }
                }

                let result = CH::insert_sorted_desc(
                    &mut encoded_hashes,
                    reference_hashes.len(),
                    writer_tell,
                    index,
                    register,
                    original_hash,
                    hash_bits,
                );

                if result.is_err() {
                    // Saturation was reached.
                    break;
                }

                if let Ok(Some(bit_index)) = result {
                    writer_tell = bit_index;
                    // If the hash was inserted, there must NOT be a reference stored with the same hash.
                    assert_eq!(reference_hashes.contains(&encoded_hash), false);
                    // We store the hash for future reference.
                    reference_hashes.push(encoded_hash);
                    // We sort by decreasing order so that we can use the binary search.
                    reference_hashes.sort_unstable_by(|a, b| b.cmp(a));
                    // We check that the inserted hash appears among the inserted hashes.
                    assert!(
                        CH::downgraded(&encoded_hashes, reference_hashes.len(), hash_bits, 0).any(|hash| hash == encoded_hash),
                        "Hash {encoded_hash} was not found after insertion. Inserted hashes: {:?}",
                        CH::downgraded(&encoded_hashes, reference_hashes.len(), hash_bits, 0).collect::<Vec<u64>>(),
                    );

                    // If we attempt to insert the same hash again, it should not be inserted.
                    assert_eq!(
                        CH::insert_sorted_desc(
                            &mut encoded_hashes,
                            reference_hashes.len(),
                            writer_tell,
                            index,
                            register,
                            original_hash,
                            hash_bits,
                        ),
                        Ok(None),
                        "After having inserted the hash ({encoded_hash}), it was inserted again. Working with hash bits {hash_bits}.",
                    );
                } else {
                    // If the hash was not inserted, there must be a reference stored with the same hash.
                    assert_eq!(
                                reference_hashes.contains(&encoded_hash), true,
                                concat!(
                                    "The hash was not found in the reference hashes, but it is in the encoded hashes. ",
                                    "The register is {} and the index is {}, and the fake hash is {}. ",
                                    "The number of bits for the hash is {}, precision is {} and the number of bits is {}."
                                ),
                                register,
                                index,
                                original_hash,
                                hash_bits,
                                CH::Precision::EXPONENT,
                                CH::Bits::NUMBER_OF_BITS,
                            );
                }

                // After inserting the hash, we check if the hash is found.
                assert!(
                            CH::find(
                                encoded_hashes,
                                reference_hashes.len(),
                                index,
                                register,
                                original_hash,
                                hash_bits,
                            )
                            .is_ok(),
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
                            CH::Precision::EXPONENT,
                            CH::Bits::NUMBER_OF_BITS,
                            reference_hashes
                                .iter()
                                .position(|&hash| hash == encoded_hash)
                                .unwrap(),
                        );

                let hashes: Vec<u64> =
                    CH::downgraded(encoded_hashes, reference_hashes.len(), hash_bits, 0).collect();

                assert_eq!(hashes.len(), reference_hashes.len());

                for (i, (downgraded_hash, reference_hash)) in
                    hashes.iter().zip(reference_hashes.iter()).enumerate()
                {
                    assert_eq!(
                        reference_hash,
                        downgraded_hash,
                        "Failed to retrieve the hash {i}/{} at hash bits {hash_bits}.",
                        reference_hashes.len()
                    );
                }

                let decoded: Vec<(u8, usize)> =
                    CH::decoded(encoded_hashes, reference_hashes.len(), hash_bits).collect();

                let reference_decoded: Vec<(u8, usize)> = reference_hashes
                    .iter()
                    .map(|&hash| CH::decode(hash, hash_bits))
                    .collect();

                assert_eq!(decoded.len(), reference_decoded.len());
                for (
                    i,
                    ((decoded_register, decoded_index), (reference_register, reference_index)),
                ) in decoded.iter().zip(reference_decoded.iter()).enumerate()
                {
                    assert_eq!(
                        reference_register,
                        decoded_register,
                        "Failed to decode register {i}/{} at hash bits {hash_bits}.",
                        reference_decoded.len()
                    );
                    assert_eq!(
                        reference_index,
                        decoded_index,
                        "Failed to decode index {i}/{} at hash bits {hash_bits}.",
                        reference_decoded.len()
                    );
                }

                // We include the case where the target hash bytes == hash bytes so to test
                // when the degrade is called with shift == 0.
                for target_hash_bytes in CH::SMALLEST_VIABLE_HASH_BITS / 8..=hash_bytes {
                    let target_hash_bits = target_hash_bytes * 8;
                    let shift = hash_bits - target_hash_bits;
                    let downgraded_hash = CH::downgrade(encoded_hash, hash_bits, shift);
                    let (downgraded_register, downgraded_index) =
                        CH::decode(downgraded_hash, target_hash_bits);

                    assert_eq!(register, downgraded_register, "Failed to downgrade the register from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}.");
                    assert_eq!(index, downgraded_index, "Failed to downgrade the index from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}.");

                    let encoded_target_hash =
                        CH::encode(index, register, original_hash, target_hash_bits);

                    assert_eq!(downgraded_hash, encoded_target_hash, "Downgraded from hash bits {hash_bits} ({encoded_hash:b}) to {target_hash_bits} hash directly encoded ({encoded_target_hash:b}) and downgraded hash do not match ({downgraded_hash:b}). The original hash is {original_hash:064b}.");

                    let downgraded_with_iterator =
                        CH::downgraded(encoded_hashes, reference_hashes.len(), hash_bits, shift)
                            .map(|downgraded_hash| CH::decode(downgraded_hash, target_hash_bits))
                            .collect::<Vec<(u8, usize)>>();

                    assert_eq!(hashes.len(), downgraded_with_iterator.len());

                    for (
                        (decoded_register, decoded_index),
                        (downgraded_register, downgraded_index),
                    ) in decoded.iter().zip(downgraded_with_iterator.iter())
                    {
                        assert_eq!(
                                    decoded_register,
                                    downgraded_register,
                                    "Failed to decode register after downgrading with iterator from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}."
                                );
                        assert_eq!(
                                    decoded_index,
                                    downgraded_index,
                                    "Failed to decode index after downgrading with iterator from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}."
                                );
                    }

                    let mut downgraded_encoded_hashes = encoded_hashes.to_vec();
                    let (number_of_duplicates, _new_writer_tell) = CH::downgrade_inplace(
                        &mut downgraded_encoded_hashes,
                        reference_hashes.len(),
                        writer_tell,
                        hash_bits,
                        shift,
                    );

                    let mut downgraded_with_iterator =
                        CH::downgraded(encoded_hashes, reference_hashes.len(), hash_bits, shift)
                            .collect::<Vec<u64>>();

                    let size_before_dedup = downgraded_with_iterator.len();

                    downgraded_with_iterator.dedup();

                    assert_eq!(
                    usize::try_from(number_of_duplicates).unwrap(),
                    size_before_dedup - downgraded_with_iterator.len(),
                    "The number of duplicates is not the same as the number of duplicates found after deduplication."
                );
                }
            }
        }
    }

    #[test_precisions_and_bits]
    fn test_current_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
    {
        test_composite_hash::<current::CurrentHash<P, B>>();
    }

    #[test_precisions_and_bits]
    fn test_switch_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
    {
        test_composite_hash::<switch::SwitchHash<P, B>>();
    }

    #[test_precisions_and_bits]
    fn test_gap_current_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
        current::CurrentHash<P, B>:
            PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
    {
        test_composite_hash::<gaps::GapHash<current::CurrentHash<P, B>>>();
    }

    #[test_precisions_and_bits]
    fn test_gap_switch_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
        switch::SwitchHash<P, B>:
            PrefixFreeCode<8> + PrefixFreeCode<16> + PrefixFreeCode<24> + PrefixFreeCode<32>,
    {
        test_composite_hash::<gaps::GapHash<switch::SwitchHash<P, B>>>();
    }
}
