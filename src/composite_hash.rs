//! Composite hashes.

use crate::{bits::Bits, prelude::Precision};
use core::fmt::Debug;
pub mod current;

#[cfg(feature = "prefix_free_codes")]
pub mod gaps;
#[cfg(feature = "prefix_free_codes")]
pub use gaps::GapHash;

mod shared;
pub mod switch;
pub use current::CurrentHash;
pub use switch::SwitchHash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Enumeration of errors that can occur when downgrading a composite hash.
pub enum CompositeHashError {
    /// The Hash List is saturated but can be downgraded.
    DowngradableSaturation,
    /// The Hash List is saturated and cannot be downgraded.
    Saturation,
}

/// A reader buffer with a known last buffered bit.
pub trait LastBufferedBit {
    /// Returns the last buffered bit.
    fn last_buffered_bit(&self) -> usize;
}

/// A composite hash is a 64-bit hash that encodes a register, index and original hash.
pub trait CompositeHash: Send + Sync + Debug {
    /// The precision of the hyperloglog.
    type Precision: Precision;
    /// The number of bits for the hash.
    type Bits: Bits;

    /// Iterator on the decoded indices and registers.
    type Decoded<'a>: Iterator<Item = (u8, usize)> + Debug + LastBufferedBit + ExactSizeIterator;

    /// Iterator on the downgraded composite hashes.
    type Downgraded<'a>: Iterator<Item = u64> + Debug + LastBufferedBit + ExactSizeIterator;

    /// Returns an iterator on the decoded indices and registers.
    fn decoded(
        hashes: &[u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
    ) -> Self::Decoded<'_>;

    /// Returns an iterator on the downgraded composite hashes.
    ///
    /// # Arguments
    /// * `hashes` - The slice of composite hashes to downgrade.
    /// * `number_of_hashes` - The number of hashes stored in the slice.
    /// * `hash_bits` - The number of bits for the hash to downgrade.
    /// * `bit_index` - The index, in bits, where the writer left off.
    /// * `shift` - The number of bits to shift the hash to the right.
    fn downgraded(
        hashes: &[u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
        shift: u8,
    ) -> Self::Downgraded<'_>;

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
    ///
    /// # Errors
    /// * `CompositeHashError::Saturation` - The Hash List is saturated and cannot be downgraded.
    /// * `CompositeHashError::DowngradableSaturation` - The Hash List is saturated but can be downgraded.
    fn find(
        hashes: &[u8],
        number_of_hashes: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
        bit_index: usize,
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
    ///
    /// # Errors
    /// * `CompositeHashError::Saturation` - The Hash List is saturated and cannot be downgraded.
    /// * `CompositeHashError::DowngradableSaturation` - The Hash List is saturated but can be downgraded.
    fn insert_sorted_desc(
        hashes: &mut [u8],
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
    fn downgrade_inplace(
        hashes: &mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        hash_bits: u8,
        shift: u8,
    ) -> (u32, usize);

    /// Returns the smallest viable hash for the current precision and number of bits.
    const SMALLEST_VIABLE_HASH_BITS: u8;
    /// Returns the largest viable hash for the current precision and number of bits.
    const LARGEST_VIABLE_HASH_BITS: u8;
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test_composite_hash {
    use core::u64;

    use super::*;
    use crate::prelude::*;
    #[cfg(feature = "prefix_free_codes")]
    use gaps::PrefixFreeCode;
    use hyperloglog_derive::test_precisions_and_bits;

    #[allow(unsafe_code)]
    fn test_composite_hash<CH: CompositeHash>()
    where
        CH::Precision: ArrayRegister<CH::Bits>,
    {
        let number_of_hashes = (1 << CH::Precision::EXPONENT)
            * usize::from(CH::Bits::NUMBER_OF_BITS)
            / usize::from(CH::SMALLEST_VIABLE_HASH_BITS);

        // We create an array where we will store the hashes.
        let mut reference_hashes = vec![0; number_of_hashes];
        // Since we do not have an array that keeps track of the number of hashes we have inserted, we will use a counter.
        let mut number_of_inserted_hashes = 0;
        // We allow a relatively small number of bits for the hash, so to force the degradation and saturation events.
        let mut encoded_hashes = vec![u64::MAX; ceil(number_of_hashes, 8)];
        let mut encoded_hashes: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(
                encoded_hashes.as_mut_ptr() as *mut u8,
                encoded_hashes.len() * 8,
            )
        };
        // We store the maximal position where we have inserted a hash.
        let mut writer_tell = 0;

        // We start from the maximal number of bits for the hash.
        let mut hash_bits = CH::LARGEST_VIABLE_HASH_BITS;

        for random_value in iter_random_values::<u64>(number_of_hashes as u64, None, None) {
            // We start each iteration by checking that the hashes are consistent.
            for (reference_hash, hash) in reference_hashes.iter().copied().zip(CH::downgraded(
                &encoded_hashes,
                number_of_inserted_hashes,
                hash_bits,
                writer_tell,
                0,
            )) {
                assert_eq!(reference_hash, hash);
            }

            let (index, register, original_hash) =
                <PlusPlus<
                    CH::Precision,
                    CH::Bits,
                    <CH::Precision as ArrayRegister<CH::Bits>>::Packed,
                >>::hash_and_register_and_index(&random_value);

            let reference_encoded_hash = CH::encode(index, register, original_hash, hash_bits);

            let result = CH::insert_sorted_desc(
                &mut encoded_hashes,
                number_of_inserted_hashes,
                writer_tell,
                index,
                register,
                original_hash,
                hash_bits,
            );

            if let Err(CompositeHashError::Saturation) = result {
                break;
            }

            if let Err(CompositeHashError::DowngradableSaturation) = result {
                let target_hash_bits = hash_bits - 8;
                let shift = hash_bits - target_hash_bits;
                println!("Downgrading from {hash_bits} to {target_hash_bits}.",);

                let (number_of_duplicates, new_writer_tell) = CH::downgrade_inplace(
                    &mut encoded_hashes,
                    number_of_inserted_hashes,
                    writer_tell,
                    hash_bits,
                    shift,
                );

                let mut last_reference_hash = u64::MAX;
                let mut reference_duplicates = 0;
                for i in 0..number_of_inserted_hashes {
                    let downgraded_hash = CH::downgrade(reference_hashes[i], hash_bits, shift);
                    if downgraded_hash == last_reference_hash {
                        reference_duplicates += 1;
                        continue;
                    }
                    last_reference_hash = downgraded_hash;
                    reference_hashes[i - reference_duplicates] = downgraded_hash;
                }

                number_of_inserted_hashes -= number_of_duplicates as usize;

                // We set the values after 'number_of_inserted_hashes' to u64::MAX, so that
                // if we end up comparing with such a value we can notice the error immediately.
                for i in number_of_inserted_hashes..number_of_hashes {
                    reference_hashes[i] = u64::MAX;
                }

                assert_eq!(
                    number_of_duplicates as usize,
                    reference_duplicates,
                    "The number of duplicates ({number_of_duplicates}) does not match the number of duplicates in the reference ({reference_duplicates}).",
                );

                hash_bits = target_hash_bits;
                writer_tell = new_writer_tell;
                continue;
            }

            if let Ok(Some(bit_index)) = result {
                writer_tell = bit_index;
                // If the hash was inserted, there must NOT be a reference stored with the same hash.
                assert!(!reference_hashes[..number_of_inserted_hashes]
                    .contains(&reference_encoded_hash));
                reference_hashes[number_of_inserted_hashes] = reference_encoded_hash;
                number_of_inserted_hashes += 1;
                // We sort by decreasing order so that we can use the binary search.
                reference_hashes[..number_of_inserted_hashes].sort_unstable_by(|a, b| b.cmp(a));
                // We check that the inserted hash appears among the inserted hashes.
                assert!(
                    CH::downgraded(
                        &encoded_hashes,
                        number_of_inserted_hashes,
                        hash_bits,
                        writer_tell,
                        0
                    )
                    .any(|hash| hash == reference_encoded_hash),
                    "Hash {reference_encoded_hash} was not found after insertion."
                );

                // If we attempt to insert the same hash again, it should not be inserted.
                assert_eq!(
                    CH::insert_sorted_desc(
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
                CH::find(
                    encoded_hashes,
                    number_of_inserted_hashes,
                    index,
                    register,
                    original_hash,
                    hash_bits,
                    writer_tell
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
                    .position(|&hash| hash == reference_encoded_hash)
                    .unwrap(),
            );
        }
    }

    #[allow(unsafe_code)]
    fn test_composite_hash_stateless_operations<CH: CompositeHash>()
    where
        CH::Precision: ArrayRegister<CH::Bits>,
    {
        const NUMBER_OF_HASHES: usize = 1_000;
        // We start from the maximal number of bits for the hash.
        for hash_bits in CH::SMALLEST_VIABLE_HASH_BITS..=CH::LARGEST_VIABLE_HASH_BITS {
            for random_value in iter_random_values::<u64>(NUMBER_OF_HASHES as u64, None, None) {
                let (index, register, original_hash) =
                    <PlusPlus<
                        CH::Precision,
                        CH::Bits,
                        <CH::Precision as ArrayRegister<CH::Bits>>::Packed,
                    >>::hash_and_register_and_index(&random_value);

                let encoded_hash = CH::encode(index, register, original_hash, hash_bits);

                // We check that the encoded hash indeed fits within the provided number of bits.
                assert_eq!(encoded_hash >> hash_bits, 0);

                let (decoded_register, decoded_index) = CH::decode(encoded_hash, hash_bits);
                assert_eq!(register, decoded_register, "Failed to recover the register at hash bits {hash_bits}. The hash is {encoded_hash:b}. The fake hash is {original_hash:064b}.");
                assert_eq!(index, decoded_index, "Failed to recover the index at hash bits {hash_bits}. The hash is {encoded_hash:b}. The fake hash is {original_hash:064b}.");

                let downgraded_hash = CH::downgrade(encoded_hash, hash_bits, 0);
                assert_eq!(
                    encoded_hash, downgraded_hash,
                    "Failed to downgrade by 0 bits. The hash is {encoded_hash:064b}."
                );

                for shift in 1..4 {
                    if hash_bits.saturating_sub(shift * 8) < CH::SMALLEST_VIABLE_HASH_BITS {
                        break;
                    }
                    let target_hash_bits = hash_bits - shift * 8;
                    let shift_bits = shift * 8;
                    let downgraded_hash = CH::downgrade(encoded_hash, hash_bits, shift_bits);
                    let (downgraded_register, downgraded_index) =
                        CH::decode(downgraded_hash, target_hash_bits);

                    assert_eq!(register, downgraded_register, "Failed to downgrade the register from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}.");
                    assert_eq!(index, downgraded_index, "Failed to downgrade the index from hash bits {hash_bits} to {target_hash_bits}. The downgraded hash is {downgraded_hash:b}.");

                    let encoded_target_hash =
                        CH::encode(index, register, original_hash, target_hash_bits);

                    assert_eq!(downgraded_hash, encoded_target_hash, "Downgraded from hash bits {hash_bits} ({encoded_hash:b}) to {target_hash_bits} hash directly encoded ({encoded_target_hash:b}) and downgraded hash do not match ({downgraded_hash:b}). The original hash is {original_hash:064b}.");
                }
            }
        }
    }

    #[test_precisions_and_bits]
    fn test_current_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
    {
        test_composite_hash_stateless_operations::<current::CurrentHash<P, B>>();
        test_composite_hash::<current::CurrentHash<P, B>>();
    }

    #[test_precisions_and_bits]
    fn test_switch_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
    {
        test_composite_hash_stateless_operations::<switch::SwitchHash<P, B>>();
        test_composite_hash::<switch::SwitchHash<P, B>>();
    }

    #[test_precisions_and_bits([[5, 4], [4, 5], [4, 6]])]
    #[cfg(feature = "prefix_free_codes")]
    fn test_gap_current_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
        current::CurrentHash<P, B>: PrefixFreeCode,
    {
        test_composite_hash_stateless_operations::<gaps::GapHash<current::CurrentHash<P, B>>>();
        test_composite_hash::<gaps::GapHash<current::CurrentHash<P, B>>>();
    }

    #[test_precisions_and_bits([[5, 4], [4, 5], [4, 6]])]
    #[cfg(feature = "prefix_free_codes")]
    fn test_gap_switch_hash<P: Precision, B: Bits>()
    where
        P: ArrayRegister<B>,
        switch::SwitchHash<P, B>: PrefixFreeCode,
    {
        test_composite_hash_stateless_operations::<gaps::GapHash<switch::SwitchHash<P, B>>>();
        test_composite_hash::<gaps::GapHash<switch::SwitchHash<P, B>>>();
    }
}
