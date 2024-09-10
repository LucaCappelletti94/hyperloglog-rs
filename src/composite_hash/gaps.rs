//! Gap-based composite hash implementation.
use core::marker::PhantomData;
use core::u64;
mod bitreader;
mod bitwriter;
mod optimal_codes;
use super::gap_birthday_paradox::{
    GAP_HASH_BIRTHDAY_PARADOX_CARDINALITIES, GAP_HASH_BIRTHDAY_PARADOX_ERRORS,
};
use super::{CompositeHash, CompositeHashError, Debug, LastBufferedBit, Precision, SwitchHash};
use crate::bits::Bits;
use bitreader::{len_rice, BitReader};
use bitwriter::BitWriter;
use optimal_codes::OPTIMAL_RICE_COEFFICIENTS;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Gap-based composite hash.
pub struct GapHash<P: Precision, B: Bits> {
    switch: PhantomData<SwitchHash<P, B>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Struct representing the portions to be encoded in the gap encoding.
pub struct GapFragment {
    /// The bits expected to have uniform distribution.
    pub uniform_delta: u64,
    /// The bits expected to have geometric distribution.
    pub geometric_minus_one: u64,
}

trait SkipSliceAhead {
    fn skip(self, position: usize) -> Self;
}

impl SkipSliceAhead for &[u8] {
    #[inline]
    fn skip(self, position: usize) -> Self {
        &self[position..]
    }
}

impl SkipSliceAhead for &mut [u8] {
    #[inline]
    fn skip(self, position: usize) -> Self {
        &mut self[position..]
    }
}

impl<P: Precision, B: Bits> GapHash<P, B> {
    #[inline]
    /// Returns the gap encoding for the given SwitchHash.
    pub fn into_gap_fragment(
        previous_hash: u64,
        hash_to_encode: u64,
        hash_bits: u8,
    ) -> GapFragment {
        debug_assert!(previous_hash > hash_to_encode);

        let previous_fragment = SwitchHash::<P, B>::scompose_hash(previous_hash, hash_bits);
        let fragment_to_encode = SwitchHash::<P, B>::scompose_hash(hash_to_encode, hash_bits);

        debug_assert!(
            previous_fragment.index > fragment_to_encode.index
                || previous_fragment.register > fragment_to_encode.register
                || previous_fragment.hash_remainder > fragment_to_encode.hash_remainder,
            "The previous register ({}) must be greater or equal to the second register ({})",
            previous_fragment.register,
            fragment_to_encode.register
        );

        // When P::EXPONENT + B::NUMBER_OF_BITS == hash_bits, there is absolutely
        // no hash remainder to include in the uniform portion of the hash, as that
        // part of the hash is solely composed of the index.
        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            GapFragment { uniform_delta: previous_fragment.index - fragment_to_encode.index, geometric_minus_one: fragment_to_encode.register - 1 }
        } else {
            // The uniform portion of the hash is composed by the index and the hash remainder.
            let previous_uniform = previous_fragment.uniform(hash_bits);
            let to_encode_uniform = fragment_to_encode.uniform(hash_bits);

            let uniform_delta = if previous_uniform > to_encode_uniform {
                ((previous_uniform - to_encode_uniform) << 1) - 1
            } else {
                (to_encode_uniform - previous_uniform) << 1
            };
            GapFragment { uniform_delta, geometric_minus_one: fragment_to_encode.register - 1 }
        }
    }

    #[inline]
    /// Applies a gap to a replacement hash.
    /// 
    /// # Arguments
    /// * `current_hash` - The current hash.
    /// * `current_hash_gap_fragment` - The gap fragment between a previous (undecoded) hash and the current hash.
    /// * `replacement_hash` - The hash to apply the gap to.
    /// * `hash_bits` - The number of bits used to encode the hashes.
    /// 
    /// # Implementative details
    /// The apply gap method is meant to apply the gap between an unknown previous hash that would be too expensive to decode
    /// and the current hash to a replacement hash.
    fn apply_gap(
        current_hash: u64,
        current_hash_gap_fragment: GapFragment,
        replacement_hash: u64,
        hash_bits: u8,
    ) -> GapFragment {
        // The replacement hash must be strictly greater than the current hash.
        debug_assert!(current_hash < replacement_hash);

        let current_hash_fragment = SwitchHash::<P, B>::scompose_hash(current_hash, hash_bits);
        let replacement_hash_fragment = SwitchHash::<P, B>::scompose_hash(replacement_hash, hash_bits);

        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            let previous_hash_index = current_hash_gap_fragment.uniform_delta + current_hash_fragment.index;
            let uniform_delta = previous_hash_index - replacement_hash_fragment.index;

            GapFragment { uniform_delta, geometric_minus_one: replacement_hash_fragment.register - 1 }
        } else {
            let current_hash_uniform = current_hash_fragment.uniform(hash_bits);
            let replacement_hash_uniform = replacement_hash_fragment.uniform(hash_bits);

            let previous_uniform = if current_hash_gap_fragment.uniform_delta & 1 == 1 {
                ((current_hash_gap_fragment.uniform_delta + 1) >> 1) + current_hash_uniform
            } else {
                current_hash_uniform - (current_hash_gap_fragment.uniform_delta >> 1)
            };

            let uniform_delta = if previous_uniform > replacement_hash_uniform {
                ((previous_uniform - replacement_hash_uniform) << 1) - 1
            } else {
                (replacement_hash_uniform - previous_uniform) << 1
            };

            GapFragment { uniform_delta, geometric_minus_one: replacement_hash_fragment.register - 1 }
        }
    }

    #[inline]
    /// Whether the hashes are currently employing an index to speed up rank operations.
    ///
    /// # Implementative details
    /// Since an index uses up some of the bits that we could use to encode the hash,
    /// we only employ it for cases where we store a large number of hashes.
    const fn has_rank_index() -> bool {
        (1_u64 << P::EXPONENT) * B::NUMBER_OF_BITS as u64 / Self::SMALLEST_VIABLE_HASH_BITS as u64
            > 500
    }

    #[inline]
    /// Returns the number of bits necessary to encode the rank index.
    const fn rank_index_bits() -> u8 {
        P::EXPONENT
            + match B::NUMBER_OF_BITS {
                4 => 2,
                5 => 3,
                6 => 3,
                _ => unreachable!(),
            }
    }

    #[cfg(feature = "std")]
    #[allow(unsafe_code)]
    /// Prints the rank index for debugging purposes.
    fn debug_index(hashes: &[u8], hash_bits: u8) {
        debug_assert!(Self::has_rank_index());

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr() as *const u32,
                hashes.len() / core::mem::size_of::<u32>(),
            )
        };

        // We retrieve the 0-th implicit bucket.
        let mut reader = BitReader::skip(
            hashes32,
            GapHash::<P, B>::rank_index_total_size(usize::from(hash_bits)),
        );
        let first_hash = reader.read_bits(usize::from(hash_bits));
        println!(
            "Bucket: 0 - Bit index: 0 - Hash: {first_hash}"
        );

        let mut reader = BitReader::new(hashes32);

        for bucket in 1..Self::rank_index_capacity() {
            let bit_index = reader.read_bits(usize::from(Self::rank_index_bits()));
            let hash = reader.read_bits(usize::from(hash_bits));

            debug_assert!(
                bit_index == Self::rank_index_mask() || usize::try_from(bit_index).unwrap() <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
                "The bit index ({bit_index}) must be less than the number of bits in the hashes ({})",
                hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits))
            );

            println!(
                "Bucket: {} - Bit index: {}{} - Hash: {}",
                bucket,
                bit_index,
                if bit_index == Self::rank_index_mask() {
                    " (NONE)"
                } else {
                    ""
                },
                hash
            );
        }
    }

    #[inline]
    /// Returns the capacity of the rank index.
    const fn rank_index_capacity() -> usize {
        P::EXPONENT as usize * 2
    }

    #[inline]
    /// Returns the total expected size of the rank index.
    pub const fn rank_index_total_size(hash_bits: usize) -> usize {
        if Self::has_rank_index() {
            (Self::rank_index_capacity() - 1) * (Self::rank_index_bits() as usize + hash_bits)
        } else {
            0
        }
    }

    #[inline]
    /// Returns the total padded size of the rank index.
    pub const fn rank_index_padded_size(hash_bits: usize) -> usize {
        Self::rank_index_total_size(hash_bits).div_ceil(8) * 8
    }

    #[inline]
    /// Returns the index of the rank index associated to the given hash.
    fn rank_index_hash_bucket(hash_bits: u8, hash: u64) -> usize {
        Self::rank_index_capacity() - hash as usize / (1usize << hash_bits).div_ceil(Self::rank_index_capacity()) - 1
    }

    #[inline]
    /// Returns the index mask for the non-initialized index.
    const fn rank_index_mask() -> u64 {
        (1 << Self::rank_index_bits()) - 1
    }

    #[allow(unsafe_code)]
    #[inline]
    /// Initializes the rank index with the given hashes.
    fn initialize_rank_index(hashes: &mut [u8], hash_bits: u8) {
        debug_assert!(Self::has_rank_index());

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        let mut writer = BitWriter::new(hashes64);

        for _ in 1..Self::rank_index_capacity() {
            writer.write_bits(Self::rank_index_mask(), Self::rank_index_bits());
            writer.write_bits(0, hash_bits);

            debug_assert!(
                writer.tell() <= Self::rank_index_total_size(usize::from(hash_bits)),
            );
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    /// Increases the bit index of all entries in the index smaller than the given hash.
    ///
    /// # Arguments
    /// * `hashes` - The slice of hashes to update.
    /// * `hash` - The hash to use as a threshold.
    /// * `bit_index_after_hash` - The bit index of the position just after the inserted hash.
    /// * `successor_hash` - The hash associated to the bucket that follows the inserted hash.
    /// * `hash_bits` - The number of bits used to encode the hashes.
    /// * `shift` - The number of bits to shift the index.
    ///
    /// # Implementative details
    /// This method is used to update all bit indices in the rank index upon an insertion
    /// of a new hash. The 'bit_index_after_hash' is relevant for the cases where the 'hash'
    /// has been inserted just before a bucket - in such cases, the shift does not apply to
    /// the bucket that immediately follows the inserted hash, as part of the expansion is
    /// relative to the hash associated to the bucket itself. Here is a schema illustrating
    /// this particular case, split into two parts: the hashes part and the associated rank index.
    /// 
    /// Suppose we have a predecessor hash x, a hash y that we  want to insert and a successor hash 'successor_hash',
    /// which also happens to be the hash associated to the bucket that follows the inserted hash. We
    /// start from the following state:
    /// 
    /// ```text
    /// Indices: [ ... | bit index i, hash successor_hash | bit index (i + 1), hash q| ... ]
    /// Hash gaps: [ ... | (pred - x) | (x - successor_hash) | ... ]
    ///                               ↑
    ///                               This position is equal to bit index i
    /// ```
    /// 
    /// When we insert y, it will cause the hash list to expand, and as we are not inserting the hash
    /// itself but the gap between the hash and the predecessor and successor, the following state will
    /// be reached for the Hash gaps:
    /// 
    /// ```text
    /// Hash gaps: [ ... | (pred - x) | (x - y) | (y - successor_hash) | ... ]
    ///                               ↑
    ///                               This position is equal to 'bit_index_after_hash'
    /// ```
    /// 
    /// Such change needs to be reflected in the bit indices in the rank index. The bit index i + 1
    /// needs to become equal to 'bit_index_after_hash', i.e. the bit index of the position right after
    /// the newly inserted hash. Indices associated to subsequent buckets, instead, need to be fully
    /// shifted by the increased size of the hash list.
    /// 
    /// ```text
    /// Indices: [ ... | bit index i + bit_index_after_hash, hash successor_hash | bit index (i + 1) + shift, hash q| ... ]
    /// Hash gaps: [ ... | (pred - x) | (x - y) | (y - successor_hash) | ... ]
    /// ```
    /// 
    fn shift_index(hashes: &mut [u8], hash: u64, bit_index_after_hash: usize, successor_hash: u64, hash_bits: u8, shift: u64) {
        let hash_bucket = Self::rank_index_hash_bucket(hash_bits, hash);

        // If the hash bucket is the last one, we do not have any subsequent buckets to update.
        if hash_bucket == Self::rank_index_capacity() - 1 {
            return;
        }

        let bucket_size = usize::from(hash_bits + Self::rank_index_bits());
        let hash_bucket_position = hash_bucket * bucket_size;

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr() as *const u32,
                hashes.len() / core::mem::size_of::<u32>(),
            )
        };

        let mut reader = BitReader::skip(hashes32, hash_bucket_position);
        let mut writer = BitWriter::new(hashes64);
        let mut current_bit_index = 0;

        for bucket in hash_bucket..Self::rank_index_capacity() - 1{
            writer.seek(bucket * bucket_size);

            // We check that the writer tell is within bounds.
            debug_assert!(
                writer.tell() < Self::rank_index_total_size(usize::from(hash_bits)),
            );

            current_bit_index = reader.read_bits(usize::from(Self::rank_index_bits())) as u64;
            // We move the reader ahead by hash_bits bits so it is positioned at the next entry.
            let current_bucket_hash = reader.read_bits(usize::from(hash_bits));

            // We check that the reader tell is within bounds.
            debug_assert!(
                reader.last_read_bit_position() <= Self::rank_index_total_size(usize::from(hash_bits)),
                "The reader tell ({}) must be less than the total size of the rank index ({}). We are at bucket {bucket} and started at hash bucket {hash_bucket}. The bucket bitsize is {bucket_size}.",
                reader.last_read_bit_position(),
                Self::rank_index_total_size(usize::from(hash_bits))
            );

            // We compute the new current bit index. First, we check
            // whether the current bit index is initialized. If it is not,
            // we skip the entry.
            if current_bit_index == Self::rank_index_mask() {
                debug_assert!(
                    current_bucket_hash == 0,
                    "The current bit index is not initialized, but the current bucket hash is not 0."
                );

                continue;
            }
            debug_assert!(
                usize::try_from(current_bit_index).unwrap() + Self::rank_index_total_size(usize::from(hash_bits)) < hashes.len() * 8,
                "The rank-index-correct bit index ({current_bit_index} + {}) must be less than the number of bits in the hashes ({}).",
                Self::rank_index_total_size(usize::from(hash_bits)),
                hashes.len() * 8
            );

            // If the hash we have just inserted is shifting to the left the current bucket index,
            // it must be larger that the hash associated to the current bucket.
            debug_assert!(
                hash > current_bucket_hash,
                "The hash ({hash}) must be larger than the current bucket hash ({current_bucket_hash})."
            );

            // If the curreny bit index is smaller than 'bit_index_after_hash', it means that it strictly
            // follows the inserted hash, and as such it must be updated to 'bit_index_after_hash'. In such
            // cases, we expect that the hash associated to the bucket to be equal to the 'successor_hash' and
            // vice-versa, i.e. when the hash associated to the bucket is equal to the 'successor_hash', the
            // bit index must be less or equal to 'bit_index_after_hash'.

            let shifted_index = if current_bucket_hash == successor_hash {
                bit_index_after_hash as u64
            } else {
                debug_assert!((bit_index_after_hash as u64) < current_bit_index + shift);
                current_bit_index + shift
            };

            // We check that the shifted index is always less than the mask
            // and that the bit index is within bounds.
            debug_assert!(
                shifted_index < Self::rank_index_mask(),
                "The shifted index must not be equal to the mask."
            );

            debug_assert!(
                usize::try_from(shifted_index).unwrap() + Self::rank_index_total_size(usize::from(hash_bits)) < hashes.len() * 8,
                "The shifted rank-index-correct bit index ({shifted_index} + {}) must be less than the number of bits in the hashes ({}).",
                Self::rank_index_total_size(usize::from(hash_bits)),
                hashes.len() * 8
            );
            // We update the bit index.
            writer.write_bits(shifted_index, Self::rank_index_bits());

            debug_assert!(
                writer.tell() <= Self::rank_index_total_size(usize::from(hash_bits)),
            );
        }

        // When we are done updating, the reader must be at the end of the rank index
        // and the writer must be at the end of the rank index minus hash_bits bits.
        debug_assert_eq!(
            reader.last_read_bit_position(), Self::rank_index_total_size(usize::from(hash_bits)),
        );

        if current_bit_index != Self::rank_index_mask() {
            debug_assert_eq!(
                writer.tell(), Self::rank_index_total_size(usize::from(hash_bits)) - usize::from(hash_bits),
            );
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    /// Updates the rank index with the given hash.
    ///
    /// # Arguments
    /// * `hashes` - The slice of hashes to update.
    /// * `hash_bits` - The number of bits used to encode the hashes.
    /// * `bit_index` - The bit index where the hash is stored.
    /// * `hash` - The hash to update the rank index with.
    fn update_rank_index(hashes: &mut [u8], hash_bits: u8, bit_index: usize, hash: u64) {
        debug_assert!(
            usize::try_from(bit_index).unwrap()
                <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({bit_index}) must be less than the number of bits in the hashes."
        );
        let hash_bucket = Self::rank_index_hash_bucket(hash_bits, hash);

        if hash_bucket == 0 {
            return;
        }

        let hash_bucket_position =
            (hash_bucket - 1) * (usize::from(Self::rank_index_bits()) + usize::from(hash_bits));

        debug_assert!(
            hash_bucket_position < hashes.len(),
            "The hash ({hash}) has hash bucket {hash_bucket}, but a hash bucket has size {} and the hashes only have {} bits, while the position in bits of the bucket would be {hash_bucket_position}.",
            usize::from(Self::rank_index_bits()) + usize::from(hash_bits),
            hashes.len()*8,
        );

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr() as *const u32,
                hashes.len() / core::mem::size_of::<u32>(),
            )
        };

        let mut reader = BitReader::skip(hashes32, hash_bucket_position);
        let bucket_bit_index = reader.read_bits(usize::from(Self::rank_index_bits()));
        let bucket_hash = reader.read_bits(usize::from(hash_bits));

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        // If the hash we received is smaller than the hash in the bucket, we update the bucket.
        if bucket_hash < hash || bucket_bit_index == Self::rank_index_mask() {
            let mut writer = BitWriter::new(hashes64);
            writer.seek(hash_bucket_position);
            writer.write_bits(bit_index as u64, Self::rank_index_bits());
            writer.write_bits(hash, hash_bits);

            debug_assert!(
                writer.tell() <= Self::rank_index_total_size(usize::from(hash_bits)),
            );
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    /// Returns the number of the bucket containing the provided bit index.
    /// 
    /// # Arguments
    /// * `hashes` - The slice of hashes to search in.
    /// * `hash_bits` - The number of bits used to encode the hashes.
    /// * `bit_index` - The bit index to search for.
    /// 
    fn rank_index_bucket_for_bit_index(hashes: &[u8], hash_bits: u8, bit_index: usize) -> usize {
        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr() as *const u32,
                hashes.len() / core::mem::size_of::<u32>(),
            )
        };

        let mut reader = BitReader::skip(hashes32, 0);
        let mut number_of_skipped_buckets = 0;

        for bucket in 0..Self::rank_index_capacity() - 1 {
            let bucket_bit_index = reader.read_bits(usize::from(Self::rank_index_bits()));
            reader.read_bits(usize::from(hash_bits));

            if bucket_bit_index == Self::rank_index_mask() {
                number_of_skipped_buckets += 1;
                continue;
            }

            if bit_index < usize::try_from(bucket_bit_index).unwrap() {
                return bucket;
            }

            debug_assert!(
                reader.last_read_bit_position() <= Self::rank_index_total_size(usize::from(hash_bits)),
            );
        }

        Self::rank_index_capacity() - 1
    }

    #[allow(unsafe_code)]
    #[inline]
    /// Returns the bit index and associated hash that is the best search starting point for the given hash.
    ///
    /// # Arguments
    /// * `hashes` - The slice of hashes to search in.
    /// * `number_of_hashes` - The number of hashes in the slice.
    /// * `hash_bits` - The number of bits used to encode the hashes.
    /// * `hash` - The hash to search for.
    ///
    /// # Implementative details
    /// The index is composed by k entries, which include the hash and the bit index where the hash is stored.
    /// We store both values explicitly to avoid having to decode the hash to find the bit index. The hash
    /// requires `hash_bits` bits, while the bit index has to represent any value between 0 and `hashes.len() * 8`,
    /// which is equal to `ceil(2^{EXPONENT} * B::NUMBER_OF_BITS, 64) * 64` bits, since the underlying storage is
    /// what we will use upon saturation to store the HyperLogLog registers inplace of the current HashList. An upper
    /// bound for the number of bits required to store the bit index is `log2(1 + (2^{EXPONENT} * B::NUMBER_OF_BITS))`
    /// We can remove the `1` from the logarithm, as it is negligible since the other term is an exponential term.
    /// We obtain therefore `EXPONENT + log2(B::NUMBER_OF_BITS)` bits to store the bit index.
    ///
    /// We employ the index solely when the number of hashes is quite large, and as such we can reasonably assume
    /// the hashes to have a uniform distribution. Keeping in mind that we know the largest possible hash (i.e.
    /// `1 << hash_bits`), we bucket the hashes in `rank_index_capacity` buckets, and in each i-th bucket we store
    /// the largest hash that is smaller than `(rank_index_capacity - i) * (1 << hash_bits) / rank_index_capacity`.
    /// The entry associated to the largest bucket is not stored as part of the index, as recalling that the hash list
    /// is sorted in descending order, it always has necessarily bit index 0 and is already stored explicitly in the
    /// hash list.
    ///
    /// Each bucket, except for the first implicit one, takes the following form:
    ///
    /// ```text
    /// [ index stored in `rank_index_bits` bits | hash stored in `hash_bits` bits ]
    /// ```
    ///
    /// The overall expected structure stored in the provided hashes takes the following form:
    ///
    /// ```text
    /// [ bucket 1 | bucket 2 | ... | bucket rank_index_capacity - 2 ] [first hash] [ prefix-coded hashes ]
    /// ```
    ///
    /// When no hashes are yet stored in the index, we return None.
    /// When the value at the bucket associated with a given hash is not yet initialized, we search the previous
    /// bucket (i.e. the one associated with the larger hash).
    fn best_search_start(hashes: &[u8], hash_bits: u8, hash: u64) -> (usize, u64) {
        let hash_bucket = Self::rank_index_hash_bucket(hash_bits, hash);

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr() as *const u32,
                hashes.len() / core::mem::size_of::<u32>(),
            )
        };

        for bucket in (0..=hash_bucket).rev() {
            if bucket == 0 {
                // We read the first hash (the largest one) explicitly, as it is not part of the index.
                let end_of_index = Self::rank_index_total_size(usize::from(hash_bits));

                let mut reader = BitReader::skip(hashes32, end_of_index);
                let first_hash = reader.read_bits(usize::from(hash_bits));

                return (0, first_hash);
            }

            let hash_bucket_position =
                (bucket - 1) * (usize::from(Self::rank_index_bits()) + usize::from(hash_bits));

            let mut reader = BitReader::skip(hashes32, hash_bucket_position);

            let bit_index = reader.read_bits(usize::from(Self::rank_index_bits()));
            let hash = reader.read_bits(usize::from(hash_bits));

            debug_assert!(
                reader.last_read_bit_position() <= Self::rank_index_total_size(usize::from(hash_bits)),
            );

            if bit_index != Self::rank_index_mask() {
                // We check that the index is within the expected bounds.
                debug_assert!(
                    usize::try_from(bit_index).unwrap() <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
                    "The bit index ({bit_index}) must be less than the number of bits in the hashes."
                );

                return (usize::try_from(bit_index).unwrap(), hash);
            }
        }
        unreachable!("The first bucket must always be initialized.");
    }

    #[inline]
    /// Returns the hashes skipping ahead by the rank index size.
    ///
    /// # Implementative details
    /// In some parametrization of Precision and Bits, we employ the rank index to speed up rank operations.
    /// The rank index is solely actively used when we are using Prefix-Free codes, as in such cases we cannot
    /// execute a binary search to find the hash we are looking for. That being said, if we were to not preserve
    /// a certain portion of memory for the rank index, we would need to somehow free it when we switch from the
    /// simple hash list to the Prefix-Free encoded hash list, but we execute the switch only when we have completely
    /// saturated the hash list, and therefore a situation where we do not have any wiggle room to store the rank index.
    fn skip_rank_index<S: SkipSliceAhead>(hashes: S, hash_bits: u8) -> S {
        debug_assert_eq!(
            hash_bits % 8,
            0,
            "The hash bits must be a multiple of 8 when requiring to skip the rank index."
        );
        hashes.skip(Self::rank_index_padded_size(usize::from(hash_bits)))
    }
}

impl<P: Precision, B: Bits> GapHash<P, B> {
    #[inline]
    #[must_use]
    /// Returns whether the hashes are currently to be considered prefix-free-encoded.
    pub fn is_prefix_free_encoded(
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
    ) -> bool {
        hash_bits < Self::LARGEST_VIABLE_HASH_BITS
            || number_of_hashes * usize::from(hash_bits) > bit_index
    }

    #[inline]
    fn b(hash_bits: u8) -> (u8, u8, u8) {
        let data =
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];

        for (target_hash_bits, uniform, geometric) in data {
            if *target_hash_bits == hash_bits {
                return (*target_hash_bits, *uniform, *geometric);
            }
        }

        unreachable!("The hash bits ({hash_bits}) must be one of the optimal hash bits.",);
    }

    #[inline]
    fn uniform_coefficient(hash_bits: u8) -> u8 {
        Self::b(hash_bits).1
    }

    #[inline]
    fn geometric_coefficient(hash_bits: u8) -> u8 {
        Self::b(hash_bits).2
    }

    #[inline]
    fn has_rice_coefficients() -> bool {
        !OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4]
            .is_empty()
    }

    #[inline]
    fn next_hash_bits(candidate_hash_bits: u8) -> u8 {
        let data =
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];

        for (target_hash_bits, _, _) in data.iter().rev() {
            if candidate_hash_bits >= *target_hash_bits {
                return *target_hash_bits;
            }
        }

        unreachable!("The hash bits ({candidate_hash_bits}) must be one of the optimal hash bits.",);
    }
}

impl<P: Precision, B: Bits> CompositeHash for GapHash<P, B> {
    type Precision = P;
    type Bits = B;

    type Decoded<'a> = DispatchedDecodedIter<'a, P, B>;
    type Downgraded<'a> = DispatchedDowngradedIter<'a, P, B>;

    #[inline]
    #[must_use]
    fn downgraded(
        hashes: &[u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
        shift: u8,
    ) -> Self::Downgraded<'_> {
        // We check that the provided bit index is within bounds.
        assert!(
            bit_index <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({bit_index}) must be less or equal to the number of bits in the hashes."
        );

        // If we are employing prefix-free codes, we use the DowngradedIter
        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            DispatchedDowngradedIter::PrefixCodeDowngradedIter(PrefixCodeDowngradedIter::new(
                hashes, bit_index, hash_bits, shift,
            ))
        } else {
            DispatchedDowngradedIter::InnerDowngradedIter(SwitchHash::<P, B>::downgraded(
                Self::skip_rank_index(hashes, hash_bits),
                number_of_hashes,
                hash_bits,
                bit_index,
                shift,
            ))
        }
    }

    #[inline]
    #[must_use]
    fn decoded(
        hashes: &[u8],
        number_of_hashes: usize,
        hash_bits: u8,
        bit_index: usize,
    ) -> Self::Decoded<'_> {
        // We check that the provided bit index is within bounds.
        assert!(
            bit_index <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({bit_index}) must be less or equal to the number of bits in the hashes."
        );
        assert!(
            hash_bits >= Self::SMALLEST_VIABLE_HASH_BITS,
            "The hash bits ({hash_bits}) must be greater or equal to the smallest viable hash bits ({})",
            Self::SMALLEST_VIABLE_HASH_BITS,
        );
        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            DispatchedDecodedIter::PrefixCodeDecodedIter(PrefixCodeDecodedIter::new(
                hashes, bit_index, hash_bits,
            ))
        } else {
            DispatchedDecodedIter::InnerDecodedIter(SwitchHash::<P, B>::decoded(
                Self::skip_rank_index(hashes, hash_bits),
                number_of_hashes,
                hash_bits,
                bit_index,
            ))
        }
    }

    #[inline]
    #[must_use]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u64 {
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << Self::Precision::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            Self::Precision::EXPONENT,
        );
        SwitchHash::<P, B>::encode(index, register, original_hash, hash_bits)
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    fn decode(hash: u64, hash_bits: u8) -> (u8, usize) {
        SwitchHash::<P, B>::decode(hash, hash_bits)
    }

    #[inline]
    #[must_use]
    /// Downgrade the hash into a smaller hash.
    fn downgrade(hash: u64, hash_bits: u8, shift: u8) -> u64 {
        SwitchHash::<P, B>::downgrade(hash, hash_bits, shift)
    }

    #[inline]
    #[allow(unsafe_code)]
    fn find(
        hashes: &[u8],
        number_of_hashes: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
        bit_index: usize,
    ) -> bool {
        // We check that the provided bit index is within bounds.
        assert!(
            bit_index <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({bit_index}) must be less or equal to the number of bits in the hashes."
        );
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << Self::Precision::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            Self::Precision::EXPONENT,
        );

        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            let encoded_hash = Self::encode(index, register, original_hash, hash_bits);

            let iter: PrefixCodeIter<'_, P, B> = if Self::has_rank_index() {
                PrefixCodeIter::new_with_rank_index(hashes, bit_index, hash_bits, encoded_hash)
            } else {
                PrefixCodeIter::new(hashes, bit_index, hash_bits)
            };

            let mut found = false;
            for value in iter {
                // The values are sorted in descending order, so we can stop when we find a value
                // that is less than or equal to the value we want to insert
                if value <= encoded_hash {
                    found = value == encoded_hash;
                    break;
                }
            }
            found
        } else {
            SwitchHash::<P, B>::find(
                Self::skip_rank_index(hashes, hash_bits),
                number_of_hashes,
                index,
                register,
                original_hash,
                hash_bits,
                bit_index,
            )
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn insert_sorted_desc(
        hashes: &mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<usize>, CompositeHashError> {
        // We check that the provided bit index is within bounds.
        debug_assert!(
            bit_index <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({bit_index}) must be less or equal to the number of bits in the hashes."
        );

        if !Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            return match SwitchHash::<P, B>::insert_sorted_desc(
                Self::skip_rank_index(hashes, hash_bits),
                number_of_hashes,
                bit_index,
                index,
                register,
                original_hash,
                hash_bits,
            ) {
                Err(_) => {
                    // If we do not have rice coefficients for this particular combination of
                    // Precision and Bits, we cannot proceed to using prefix-free codes, and
                    // we must declare complete Saturation, which implies switching from the
                    // HashList to the HyperLogLog registers.
                    if !Self::has_rice_coefficients() {
                        return Err(CompositeHashError::Saturation);
                    }

                    // We check whether we can dowgrade to the current hash bits, or if we need to
                    // downgrade to a smaller hash bits.
                    let next_hash_bits = Self::next_hash_bits(hash_bits);
                    if hash_bits > next_hash_bits {
                        return Err(CompositeHashError::DowngradableSaturation);
                    };

                    let (duplicates, new_bit_index) =
                        Self::downgrade_inplace(hashes, number_of_hashes, bit_index, hash_bits, 0);

                    debug_assert_eq!(
                        duplicates, 0,
                        "There should be no duplicates when first prefix-coding a new value."
                    );

                    // And we try to insert the hash again.
                    Self::insert_sorted_desc(
                        hashes,
                        number_of_hashes,
                        new_bit_index,
                        index,
                        register,
                        original_hash,
                        hash_bits,
                    )
                }
                Ok(value) => Ok(value),
            };
        }

        debug_assert!(
            Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index),
            "The hashes must be prefix-free encoded to be able to use prefix-free codes."
        );

        // We check that all hashes are still ordered in descending order
        debug_assert!(
            Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
                .is_sorted_by(|a, b| b < a),
            "Illegal hashes state: attempting to insert a value with hash bits {hash_bits}, number of hashes {number_of_hashes} and bit index {bit_index} at index {index} and register {register} with original hash {original_hash}.",
        );

        let hashes_ref: &[u8] =
            unsafe { core::slice::from_raw_parts(hashes.as_ptr() as *const u8, hashes.len()) };

        let encoded_hash = Self::encode(index, register, original_hash, hash_bits);

        // Before editing anything, we check that the hash list as provided is sorted in descending order.
        debug_assert!(
            PrefixCodeIter::<P, B>::new(hashes_ref, bit_index, hash_bits)
                .is_sorted_by(|a, b| b < a),
            "The hashes must be sorted in descending order.",
        );


        // iter until we find where we should insert
        let mut iter: PrefixCodeIter<'_, P, B> = if Self::has_rank_index() {
            // The list still has to be fully sorted even when we start to iterate from a point in the middle.
            debug_assert!(
                PrefixCodeIter::<P, B>::new_with_rank_index(hashes_ref, bit_index, hash_bits, encoded_hash)
                    .is_sorted_by(|a, b| b < a),
                "The hashes must be sorted in descending order.",
            );

            PrefixCodeIter::new_with_rank_index(hashes_ref, bit_index, hash_bits, encoded_hash)
        } else {
            PrefixCodeIter::new(hashes_ref, bit_index, hash_bits)
        };

        let mut previous_hash = None;
        let mut next_hash = None;
        let mut last_read_bit_position = iter.last_read_bit_position();
        let mut last_read_bit_position_variation = 0;

        if !Self::has_rank_index() {
            debug_assert_eq!(
                last_read_bit_position, 0,
                "The last read bit position must be 0 when not using a rank index.",
            );
        }

        debug_assert!(
            last_read_bit_position >= Self::rank_index_total_size(usize::from(hash_bits)),
            "Last read bit position ({last_read_bit_position}) must be greater or equal to the rank index size ({}), otherwise instead of reading hashes it would be reading the rank index.",
            Self::rank_index_total_size(usize::from(hash_bits)),
        );

        while let Some(value) = iter.next() {
            // if Self::has_rank_index() {
            //     if Self::rank_index_bucket_for_bit_index(hashes, hash_bits, last_read_bit_position - Self::rank_index_total_size(usize::from(hash_bits))) != 
            //     Self::rank_index_hash_bucket(hash_bits, value) {
            //         Self::debug_index(hashes, hash_bits);
            //     }

            //     debug_assert_eq!(
            //         Self::rank_index_bucket_for_bit_index(hashes, hash_bits, last_read_bit_position - Self::rank_index_total_size(usize::from(hash_bits))),
            //         Self::rank_index_hash_bucket(hash_bits, value),
            //         "Hash {value} is at bit index {} and is in the wrong bucket. Precision: {}, number of bits: {}, hash bits: {hash_bits}.",
            //         last_read_bit_position - Self::rank_index_total_size(usize::from(hash_bits)),
            //         P::EXPONENT,
            //         B::NUMBER_OF_BITS,
            //     );
            // }

            // The values are sorted in descending order, so we can stop when we find a value
            // that is less than or equal to the value we want to insert
            if encoded_hash >= value {
                // if the value is equal to the encoded_hash value, we don't need to insert it
                if value == encoded_hash {
                    return Ok(None);
                }
                last_read_bit_position_variation = iter.last_read_bit_position() - last_read_bit_position;
                next_hash = Some(value);
                break;
            }

            last_read_bit_position = iter.last_read_bit_position();
            previous_hash = Some(value);
        }

        // If we are changing the first hash in the hast list, we expect that the 'last_read_bit_position_variation'
        // is equal to the hash bits.
        debug_assert!(
            last_read_bit_position > Self::rank_index_total_size(usize::from(hash_bits))
                || last_read_bit_position_variation == usize::from(hash_bits),
            "When we are changing the first hash in the hash list, the last read bit position variation ({last_read_bit_position_variation}) must be equal to the hash bits ({hash_bits}).",
        );

        let prev_to_current_gap = match (previous_hash, Self::has_rank_index()) {
            (None, true) => {
                // When we use a rank index and we have immediately found the value we position where
                // we would like to insert the value, it means we are replacing the first hash in the
                // bucket. While we have stored this hash in the index, and we do have the gap informations
                // between this hash and its successor, we do not have the true previous hash, which is the
                // gap between the previous hash and the previous previous hash. A trivial approach like
                // decoding the previous previous hash would require us to decode the entirety of the previous
                // bucket and would therefore be inefficient.
                next_hash = Some(iter.previous);
                if last_read_bit_position == Self::rank_index_total_size(usize::from(hash_bits)) {
                    None
                } else {
                    Some(Self::apply_gap(next_hash.unwrap(), iter.previous_gap_fragment.unwrap(), encoded_hash, hash_bits))
                }
            }
            (Some(_), true) | (_, false) => previous_hash
                .map(|previous_hash| {
                    debug_assert!(previous_hash > encoded_hash);
                    Self::into_gap_fragment(previous_hash, encoded_hash, hash_bits)
                })
        };

        let current_to_next_gap = next_hash
            .map(|next_hash| {
                debug_assert!(next_hash < encoded_hash);
                Self::into_gap_fragment(encoded_hash, next_hash, hash_bits)
            });

        // We check that we would be actually able to insert the new value, given the current
        // bit index and the size the new value would require.
            
        // When the currently encoded value has a predecessor, we will be adding the gap between
        // the previous hash and the current hash. If it does not have a predecessor, we will be
        // adding the current hash itself, so it will require the full hash bits.

        debug_assert!(
            prev_to_current_gap.is_some() || last_read_bit_position == Self::rank_index_total_size(usize::from(hash_bits)),
            "The previous to current gap ({prev_to_current_gap:?}) must be None when the last read bit position ({last_read_bit_position}) is the size of the rank index ({}).",
            Self::rank_index_total_size(usize::from(hash_bits))
        );

        let previous_to_current_size = prev_to_current_gap.map_or(
            usize::from(hash_bits),
            |prev_to_current_gap| {
                len_rice(
                    prev_to_current_gap.uniform_delta,
                    Self::uniform_coefficient(hash_bits),
                    prev_to_current_gap.geometric_minus_one,
                    Self::geometric_coefficient(hash_bits),
                )
            },
        );

        // When the currently encoded value has a successor, we will be adding the gap between
        // the current hash and the next hash. If it does not have a successor, then there is
        // nothing to write. If there is no predecessor but there is a successor, it means that
        // we are replacing the first hash in the hash list, which has size 'hash_bits'.
        // Such delta is already accounted for in 'last_read_bit_position_variation'.
        let current_to_next_size: usize = current_to_next_gap.map_or(0, |current_to_next_gap| {
            len_rice(
                current_to_next_gap.uniform_delta,
                Self::uniform_coefficient(hash_bits),
                current_to_next_gap.geometric_minus_one,
                Self::geometric_coefficient(hash_bits),
            )
        });

        let number_of_inserted_bits = previous_to_current_size + current_to_next_size - last_read_bit_position_variation;

        let new_bit_index = bit_index + number_of_inserted_bits;

        if new_bit_index + Self::rank_index_total_size(usize::from(hash_bits))
            > hashes_ref.len() * 8
        {
            if hash_bits == Self::SMALLEST_VIABLE_HASH_BITS {
                return Err(CompositeHashError::Saturation);
            }
            return Err(CompositeHashError::DowngradableSaturation);
        }

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        let mut writer = BitWriter::new(hashes64);
        writer.seek(last_read_bit_position);

        debug_assert!(
            writer.tell() >= Self::rank_index_total_size(usize::from(hash_bits)),
            "The writer tell ({}) must be greater or equal to the rank index size ({}), otherwise instead of writing hashes it would be writing on the rank index.",
            writer.tell(),
            Self::rank_index_total_size(usize::from(hash_bits)),
        );

        // Now that we have determined where to insert the new value, the subsequent values
        // will be solely read from the bitstream and written to the writer.
        let mut bypass: BypassIter<'_> = iter.into_bypass();
        // In order to bring the reader a bit more ahead and make more unlikely to get
        // read-write conflicts, we read the next value.
        let mut next = bypass.next();

        // If there is no previos value, we would need to write the encoded value itself but
        // writing such a high value in prefix-free encoding would be inefficient. Therefore,
        // we write the first hash explicitly.
        if let Some(prev_to_current_gap) = prev_to_current_gap {
            let total_wrote = writer.write_rice(
                prev_to_current_gap.uniform_delta,
                prev_to_current_gap.geometric_minus_one,
                Self::uniform_coefficient(hash_bits),
                Self::geometric_coefficient(hash_bits),
            );

            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() >= writer.tell(),
                "Reader tell ({}) must be greater than writer tell ({}, just wrote {total_wrote}) in insert at hash size {hash_bits}.",
                bypass.last_buffered_bit(),
                writer.tell(),
            );
        } else {
            debug_assert_eq!(
                writer.tell(),
                Self::rank_index_total_size(usize::from(hash_bits)),
                "The writer tell must be 0 or rank index size if there is no previous value"
            );

            writer.write_bits(encoded_hash, hash_bits);
        }

        // We check that practice matches theory:
        debug_assert_eq!(
            writer.tell() - last_read_bit_position,
            previous_to_current_size,
        );

        if let Some(current_to_next_gap) = current_to_next_gap {
            let total_wrote = writer.write_rice(
                current_to_next_gap.uniform_delta,
                current_to_next_gap.geometric_minus_one,
                Self::uniform_coefficient(hash_bits),
                Self::geometric_coefficient(hash_bits),
            );

            debug_assert!(
                bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
                "Reader tell ({}) must be greater than writer tell ({}, just wrote {total_wrote}) in insert at hash size {hash_bits}.",
                bypass.last_buffered_bit(),
                writer.tell(),
            );
            
            // We check that practice matches theory:
            debug_assert_eq!(
                writer.tell() - last_read_bit_position,
                previous_to_current_size + current_to_next_size,
            );

            if number_of_inserted_bits > 0 {
                // If our hashes are not vbyted, we have to write all of the remaining hashes one-by-one
                // as we could not just shift the hashes to the right.
                while let Some((value, n_bits)) = next {
                    next = bypass.next();
                    writer.write_bits(value, n_bits);
                    debug_assert!(
                        bypass.len() == 0 || bypass.last_buffered_bit() > writer.tell(),
                        "Reader tell ({}) must be greater than writer tell ({}) in insert at hash size {hash_bits}.",
                        bypass.last_buffered_bit(),
                        writer.tell(),
                    );
                }

                let writer_tell = writer.tell();

                // We check that practice matches theory:
                debug_assert_eq!(
                    writer_tell - Self::rank_index_total_size(usize::from(hash_bits)),
                    new_bit_index,
                    "Expected writer tell (started at {last_read_bit_position} [{}], prev_hash: {previous_hash:?} next_hash {next_hash:?}) to match bit index {bit_index} + value variation {number_of_inserted_bits} = ({new_bit_index}). Rank index size: {}",
                    last_read_bit_position - Self::rank_index_total_size(usize::from(hash_bits)),
                    Self::rank_index_total_size(usize::from(hash_bits)),
                );
            }
        }

        drop(writer);

        if Self::has_rank_index() {
            // If we are using a rank index, we may need to update the values in the index.
            Self::update_rank_index(
                hashes,
                hash_bits, 
                last_read_bit_position - Self::rank_index_total_size(usize::from(hash_bits)),
                encoded_hash
            );

            if let Some(next_hash) = next_hash {
                // Furthermore, we need to shift the bit index associated to values that are smaller
                // than the current hash and therefore will be shifted to the right.
                Self::shift_index(
                    hashes,
                    encoded_hash,
                    last_read_bit_position + previous_to_current_size - Self::rank_index_total_size(usize::from(hash_bits)),
                    next_hash,
                    hash_bits,
                    number_of_inserted_bits as u64,
                );
            }
        }

        debug_assert!(
            PrefixCodeIter::<P, B>::new(hashes, new_bit_index, hash_bits)
                .is_sorted_by(|a, b| b < a)
        );

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!(
            "Inserted at hash size {hash_bits} with bit index {}/{bit_index} and index {index} and register {register} with encoded hash {encoded_hash} adding {number_of_inserted_bits} bits.",
            last_read_bit_position - Self::rank_index_total_size(usize::from(hash_bits))
        );

        Ok(Some(new_bit_index))
    }

    #[inline]
    #[allow(unsafe_code)]
    /// Downgrade the hash into a smaller hash in place.
    fn downgrade_inplace(
        hashes: &mut [u8],
        number_of_hashes: usize,
        bit_index: usize,
        hash_bits: u8,
        shift: u8,
    ) -> (u32, usize) {
        // We check that the provided bit index is within bounds.
        assert!(
            bit_index <= hashes.len() * 8 - Self::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({bit_index}) must be less or equal to the number of bits in the hashes."
        );

        let target_hash_bits = hash_bits - shift;

        // safe because the slice is originally allocated as u64s
        debug_assert!(hashes.len() % core::mem::size_of::<u64>() == 0);
        let hashes_64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr() as *mut u64,
                hashes.len() / core::mem::size_of::<u64>(),
            )
        };

        // Copy of the mutable reference of the hashes which we employ
        // to write the new hashes.
        let hashes_8: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(hashes.as_mut_ptr() as *mut u8, hashes.len())
        };

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!("Downgrading at hash size {hash_bits} with shift {shift}.");

        // We initialize the index before we start writing the hashes.
        if Self::has_rank_index() {
            Self::initialize_rank_index(hashes, target_hash_bits);
        }

        let mut writer = BitWriter::new(hashes_64);
        writer.seek(Self::rank_index_total_size(usize::from(target_hash_bits)));

        debug_assert!(
            Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, 0)
                .is_sorted_by(|a, b| b < a)
        );

        let mut iter = Self::downgraded(hashes, number_of_hashes, hash_bits, bit_index, shift);

        // We write the first hash explicitly, as otherwise it would be
        // written in a very inefficient way.
        let mut previous_hash = iter.next().unwrap();
        writer.write_bits(previous_hash, target_hash_bits);

        if Self::has_rank_index() {
            Self::update_rank_index(hashes_8, target_hash_bits, 0, previous_hash);
        }

        let mut duplicates = 0;

        let mut maybe_next = iter.next();
        let mut maybe_double_next = iter.next();

        let uniform_coefficient = Self::uniform_coefficient(target_hash_bits);
        let geometric_coefficient = Self::geometric_coefficient(target_hash_bits);

        while let Some(next) = maybe_next {
            maybe_next = maybe_double_next;
            maybe_double_next = iter.next();

            if next == previous_hash {
                duplicates += 1;
                continue;
            }

            debug_assert!(
                previous_hash > next,
                "The hashes must be sorted in descending order. Found {previous_hash} <= {next}."
            );

            let fragment = Self::into_gap_fragment(previous_hash, next, target_hash_bits);
            let writer_tell_before_write = writer.tell();

            debug_assert!(
                (fragment.uniform_delta >> uniform_coefficient) < 64,
                "Uniform delta: {}, uniform_coefficient: {}, shifted: {}, target_hash_bits: {target_hash_bits}, precision: {}, bits: {}.",
                fragment.uniform_delta,
                uniform_coefficient,
                fragment.uniform_delta >> uniform_coefficient,
                P::EXPONENT,
                B::NUMBER_OF_BITS
            );
            debug_assert!(
                (fragment.geometric_minus_one >> geometric_coefficient) < 64,
                "Geometric delta: {}, geometric_coefficient: {}, shifted: {}, target_hash_bits: {target_hash_bits}, precision: {}, bits: {}.",
                fragment.geometric_minus_one,
                geometric_coefficient,
                fragment.geometric_minus_one >> geometric_coefficient,
                P::EXPONENT,
                B::NUMBER_OF_BITS
            );

            let total_wrote = writer.write_rice(
                fragment.uniform_delta,
                fragment.geometric_minus_one,
                uniform_coefficient,
                geometric_coefficient,
            );

            // If we are using the index, we need to insert the position and the hash
            // that represents the largest entry of the current bucket. First, we determine
            // whether the current hash is the largest hash in the bucket. We can easily
            // determine this by comparing the current hash with the previous hash. If the
            // bucket demarking line is between the two hashes, since the hashes are sorted
            // in descending order, the current hash is necessarily the largest hash in the
            // bucket. We then write the position and the hash in the rank index.
            if Self::has_rank_index() {
                let previous_hash_bucket =
                    Self::rank_index_hash_bucket(target_hash_bits, previous_hash);
                let current_hash_bucket = Self::rank_index_hash_bucket(target_hash_bits, next);

                if current_hash_bucket > previous_hash_bucket {
                    Self::update_rank_index(
                        hashes_8,
                        target_hash_bits, 
                        writer_tell_before_write - Self::rank_index_total_size(usize::from(target_hash_bits)),
                        next
                    );
                }
            }

            debug_assert!(
                iter.last_buffered_bit() >= writer.tell(),
                "Reader tell ({}) must be greater than writer tell ({}, just wrote {total_wrote}, {previous_hash} - {next}) in downgrade at hash size {hash_bits} with shift {shift}. Precision: {}, Bits: {}.",
                iter.last_buffered_bit(),
                writer.tell(),
                P::EXPONENT,
                B::NUMBER_OF_BITS
            );
            previous_hash = next;
        }

        let writer_tell = writer.tell();

        // If we are using the rank index, we need to remove the index size from the writer tell to convert it into the updated bit index.
        let updated_bit_index = writer_tell - Self::rank_index_total_size(usize::from(target_hash_bits));

        drop(writer);

        debug_assert!(
            updated_bit_index <= bit_index,
            "PFC-ing at bit size {hash_bits} with shift {shift} should decrease the bit index ({bit_index}), but got writer tell {updated_bit_index}. Precision: {}, Bits: {}.",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        );

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!(
            "Writer tell at {updated_bit_index}, started from {bit_index}, with {duplicates} duplicates.",
        );

        debug_assert_eq!(
            PrefixCodeIter::<P, B>::new(
                hashes,
                updated_bit_index,
                target_hash_bits,
            )
            .count(),
            number_of_hashes - duplicates,
            "The number of hashes (minus duplicates) must be the same after downgrading. Current writer tell: {updated_bit_index}, started from {bit_index}."
        );

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!("Downgrading completed with {} duplicates.", duplicates);

        (u32::try_from(duplicates).unwrap(), updated_bit_index)
    }

    #[inline]
    fn target_downgraded_hash_bits(
        _number_of_hashes: usize,
        _bit_index: usize,
        hash_bits: u8,
    ) -> u8 {
        Self::next_hash_bits(hash_bits - 1)
    }

    const SMALLEST_VIABLE_HASH_BITS: u8 = Self::Precision::EXPONENT + Self::Bits::NUMBER_OF_BITS;
    const LARGEST_VIABLE_HASH_BITS: u8 = SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
    const BIRTHDAY_CARDINALITIES: &[u32] = GAP_HASH_BIRTHDAY_PARADOX_CARDINALITIES
        [P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
    const BIRTHDAY_RELATIVE_ERRORS: &[f64] =
        GAP_HASH_BIRTHDAY_PARADOX_ERRORS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub enum DispatchedDowngradedIter<'a, P: Precision, B: Bits> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDowngradedIter(PrefixCodeDowngradedIter<'a, P, B>),
    /// Variants for when the prefix-free codes are not used.
    InnerDowngradedIter(<SwitchHash<P, B> as CompositeHash>::Downgraded<'a>),
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for DispatchedDowngradedIter<'a, P, B> {
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDowngradedIter(iter) => {
                iter.last_buffered_bit()
                    + GapHash::<P, B>::rank_index_total_size(usize::from(iter.hash_bits()))
            }
        }
    }

    fn hash_bits(&self) -> u8 {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.iter.hash_bits,
            Self::InnerDowngradedIter(iter) => iter.hash_bits(),
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for DispatchedDowngradedIter<'a, P, B> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.next(),
            Self::InnerDowngradedIter(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
/// Bypass iterator which instead of executing any operation on the [`BitReader`] stream,
/// just reads u64 words up until the end of the stream.
struct BypassIter<'a> {
    /// The bitstream to read from.
    bitstream: BitReader<'a>,
    /// The expected end of the current bit-stream.
    bit_index: usize,
}

impl Iterator for BypassIter<'_> {
    type Item = (u64, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitstream.last_read_bit_position() >= self.bit_index {
            return None;
        }
        let n_bits = core::cmp::min(64, self.bit_index - self.bitstream.last_read_bit_position());
        Some((self.bitstream.read_bits(n_bits), n_bits as u8))
    }
}

impl ExactSizeIterator for BypassIter<'_> {
    fn len(&self) -> usize {
        self.bit_index
            .saturating_sub(self.bitstream.last_read_bit_position())
            .div_ceil(64)
    }
}

impl<'a> LastBufferedBit for BypassIter<'a> {
    fn last_buffered_bit(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }

    fn hash_bits(&self) -> u8 {
        unreachable!("The BypassIter does not have a hash bits associated with it.")
    }
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub struct PrefixCodeDowngradedIter<'a, P: Precision, B: Bits> {
    iter: PrefixCodeIter<'a, P, B>,
    shift: u8,
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for PrefixCodeDowngradedIter<'a, P, B> {
    fn last_buffered_bit(&self) -> usize {
        self.iter.last_buffered_bit()
    }

    fn hash_bits(&self) -> u8 {
        self.iter.hash_bits
    }
}

impl<'a, P: Precision, B: Bits> PrefixCodeDowngradedIter<'a, P, B> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], maximal_bit_index: usize, hash_bits: u8, shift: u8) -> Self {
        Self {
            iter: PrefixCodeIter::new(hashes, maximal_bit_index, hash_bits),
            shift,
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for PrefixCodeDowngradedIter<'a, P, B> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(GapHash::<P, B>::downgrade(
            self.iter.next()?,
            self.iter.hash_bits,
            self.shift,
        ))
    }
}

#[derive(Debug, Clone)]
/// Iterator over downgraded hashes.
pub struct PrefixCodeIter<'a, P: Precision, B: Bits> {
    bitstream: BitReader<'a>,
    previous: u64,
    first: bool,
    hash_bits: u8,
    maximal_bit_index: usize,
    previous_gap_fragment: Option<GapFragment>,
    previous_index: u64,
    previous_hash_remainder: u64,
    previous_uniform: u64,
    uniform_coefficient: u8,
    geometric_coefficient: u8,
    #[cfg(test)]
    iteration: usize,
    _phantom: PhantomData<GapHash<P, B>>,
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for PrefixCodeIter<'a, P, B> {
    fn last_buffered_bit(&self) -> usize {
        self.bitstream.last_buffered_bit_position()
    }

    fn hash_bits(&self) -> u8 {
        self.hash_bits
    }
}

impl<'a, P: Precision, B: Bits> PrefixCodeIter<'a, P, B> {
    fn last_read_bit_position(&self) -> usize {
        self.bitstream.last_read_bit_position()
    }

    fn into_bypass(self) -> BypassIter<'a> {
        BypassIter {
            bitstream: self.bitstream,
            bit_index: self.maximal_bit_index + GapHash::<P, B>::rank_index_total_size(usize::from(self.hash_bits)),
        }
    }

    #[allow(unsafe_code)]
    fn new_with_rank_index(
        hashes: &'a [u8],
        maximal_bit_index: usize,
        hash_bits: u8,
        current_hash: u64,
    ) -> Self {
        debug_assert!(GapHash::<P, B>::has_rank_index());

        // We find the best starting point for the iterator that is closest to the current hash.
        let (bit_index, bucket_hash) =
            GapHash::<P, B>::best_search_start(hashes, hash_bits, current_hash);

        debug_assert!(
            maximal_bit_index + GapHash::<P, B>::rank_index_total_size(usize::from(hash_bits)) <= hashes.len() * 8,
            "The bit index ({maximal_bit_index}) must be less or equal to the number of bits in the hashes."
        );

        debug_assert!(
            bit_index 
                <= maximal_bit_index,
            "The bit index ({bit_index}) must be less or equal to the maximal bit index ({maximal_bit_index})."
        );

        Self {
            previous: bucket_hash,
            first: true,
            bitstream: BitReader::skip(
                unsafe {
                    core::slice::from_raw_parts_mut(
                        hashes.as_ptr() as *mut u32,
                        hashes.len() / core::mem::size_of::<u32>(),
                    )
                },
                bit_index + GapHash::<P, B>::rank_index_total_size(usize::from(hash_bits)),
            ),
            hash_bits,
            maximal_bit_index,
            previous_gap_fragment: None,
            previous_uniform: u64::MAX,
            previous_index: u64::MAX,
            previous_hash_remainder: u64::MAX,
            uniform_coefficient: GapHash::<P, B>::uniform_coefficient(hash_bits),
            geometric_coefficient: GapHash::<P, B>::geometric_coefficient(hash_bits),
            #[cfg(test)]
            iteration: 0,
            _phantom: PhantomData,
        }
    }

    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], maximal_bit_index: usize, hash_bits: u8) -> Self {
        debug_assert!(
            maximal_bit_index <= hashes.len() * 8 - GapHash::<P, B>::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({maximal_bit_index}) must be less or equal to the number of bits in the hashes."
        );

        Self {
            previous: u64::MAX,
            first: true,
            bitstream: BitReader::skip(
                unsafe {
                    core::slice::from_raw_parts_mut(
                        hashes.as_ptr() as *mut u32,
                        hashes.len() / core::mem::size_of::<u32>(),
                    )
                },
                GapHash::<P, B>::rank_index_total_size(usize::from(hash_bits)),
            ),
            hash_bits,
            maximal_bit_index,
            previous_gap_fragment: None,
            previous_uniform: u64::MAX,
            previous_index: u64::MAX,
            previous_hash_remainder: u64::MAX,
            uniform_coefficient: GapHash::<P, B>::uniform_coefficient(hash_bits),
            geometric_coefficient: GapHash::<P, B>::geometric_coefficient(hash_bits),
            #[cfg(test)]
            iteration: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for PrefixCodeIter<'a, P, B> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;

            if self.bitstream.last_read_bit_position() == GapHash::<P, B>::rank_index_total_size(usize::from(self.hash_bits)) {
                self.previous = self.bitstream.read_bits(usize::from(self.hash_bits));
            } else {
                let uniform_delta = self.bitstream.read_rice(self.uniform_coefficient);
                let geometric_minus_one = self.bitstream.read_rice(self.geometric_coefficient);
                self.previous_gap_fragment = Some(GapFragment {
                    uniform_delta,
                    geometric_minus_one,
                });
            }

            let hash_fragment = SwitchHash::<P, B>::scompose_hash(
                self.previous,
                self.hash_bits,
            );

            // The geometric minus one should be always equal to the geometric portion of the hash fragment,
            // i.e. the register value.
            if let Some(previous_gap_fragment) = self.previous_gap_fragment {
                debug_assert_eq!(
                    previous_gap_fragment.geometric_minus_one + 1, hash_fragment.register,
                    "Failed to read the previous hash fragment. Hash bits: {}, reader tell: {}, hash: {}",
                    self.hash_bits,
                    self.bitstream.last_read_bit_position(),
                    self.previous
                );
            }

            self.previous_uniform = hash_fragment.uniform(self.hash_bits);
            self.previous_index = hash_fragment.index;
            self.previous_hash_remainder = hash_fragment.hash_remainder;

            return Some(self.previous);
        }
        if self.maximal_bit_index
            + GapHash::<P, B>::rank_index_total_size(usize::from(self.hash_bits))
            <= self.bitstream.last_read_bit_position()
        {
            return None;
        }
        #[cfg(test)]
        {
            self.iteration += 1;
        }

        let uniform_delta = self.bitstream.read_rice(self.uniform_coefficient);
        let geometric_minus_one = self.bitstream.read_rice(self.geometric_coefficient);
        self.previous_gap_fragment = Some(GapFragment {
            uniform_delta,
            geometric_minus_one,
        });

        // When P::EXPONENT + B::NUMBER_OF_BITS == hash_bits, there is absolutely
        // no hash remainder to include in the uniform portion of the hash, as that
        // part of the hash is solely composed of the index.
        if P::EXPONENT + B::NUMBER_OF_BITS == self.hash_bits {
            self.previous_index -= uniform_delta;

            let current_hash = SwitchHash::<P, B>::compose_hash(
                self.previous_index,
                geometric_minus_one + 1,
                0,
                self.hash_bits,
            );

            #[cfg(test)]
            debug_assert!(
                current_hash < self.previous,
                "{}) The current hash ({}) must be less than the previous hash ({}). The reader tell is {} and the maximal bit index is {}. Previous hash is {}. The rank index size is {}.",
                self.iteration,
                current_hash,
                self.previous,
                self.bitstream.last_read_bit_position(),
                self.maximal_bit_index,
                self.previous,
                GapHash::<P, B>::rank_index_total_size(usize::from(self.hash_bits))
            );

            self.previous = current_hash;

            return Some(self.previous);
        }

        self.previous_uniform = if uniform_delta & 1 == 0 {
            self.previous_uniform + (uniform_delta >> 1)
        } else {
            debug_assert!(
                self.previous_uniform >= (uniform_delta >> 1) + 1,
                "The previous uniform ({}) must be greater than or equal to the uniform ({uniform_delta}) >> 1 + 1 = {}. The reader tell is {} and the maximal bit index is {}. Previous hash is {}. The rank index size is {}.",
                self.previous_uniform,
                (uniform_delta >> 1) + 1,
                self.bitstream.last_read_bit_position(),
                self.maximal_bit_index,
                self.previous,
                GapHash::<P, B>::rank_index_total_size(usize::from(self.hash_bits))
            );

            self.previous_uniform - ((uniform_delta >> 1) + 1)
        };

        let remainder_size = self.hash_bits - 1 - P::EXPONENT;
        let to_decode_index = self.previous_uniform >> remainder_size;
        let to_decode_hash_remainder = self.previous_uniform & ((1 << remainder_size) - 1);

        self.previous_index = to_decode_index;
        self.previous_hash_remainder = to_decode_hash_remainder;

        let new_hash = SwitchHash::<P, B>::compose_hash(
            self.previous_index,
            geometric_minus_one + 1,
            self.previous_hash_remainder,
            self.hash_bits,
        );

        #[cfg(test)]
        debug_assert!(
            new_hash < self.previous,
            "{}) The current hash ({}) must be less than the previous hash ({}). The reader tell is {} and the maximal bit index is {}. Previous hash is {}. Hash bits: {}. The rank index size is {}.",
            self.iteration,
            new_hash,
            self.previous,
            self.bitstream.last_read_bit_position(),
            self.maximal_bit_index,
            self.previous,
            self.hash_bits,
            GapHash::<P, B>::rank_index_total_size(usize::from(self.hash_bits))
        );

        self.previous = new_hash;

        Some(self.previous)
    }
}

#[derive(Debug)]
/// Iterator over decoded hashes.
pub enum DispatchedDecodedIter<'a, P: Precision, B: Bits> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDecodedIter(PrefixCodeDecodedIter<'a, P, B>),
    /// Variants for when the prefix-free codes are not used.
    InnerDecodedIter(<SwitchHash<P, B> as CompositeHash>::Decoded<'a>),
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for DispatchedDecodedIter<'a, P, B> {
    fn last_buffered_bit(&self) -> usize {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDecodedIter(iter) => iter.last_buffered_bit(),
        }
    }

    fn hash_bits(&self) -> u8 {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.hash_bits(),
            Self::InnerDecodedIter(iter) => iter.hash_bits(),
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for DispatchedDecodedIter<'a, P, B> {
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.next(),
            Self::InnerDecodedIter(iter) => iter.next(),
        }
    }
}

#[derive(Debug)]
/// Iterator over decoded hashes.
pub struct PrefixCodeDecodedIter<'a, P: Precision, B: Bits> {
    iter: PrefixCodeIter<'a, P, B>,
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for PrefixCodeDecodedIter<'a, P, B> {
    fn last_buffered_bit(&self) -> usize {
        self.iter.last_buffered_bit()
    }

    fn hash_bits(&self) -> u8 {
        self.iter.hash_bits()
    }
}

impl<'a, P: Precision, B: Bits> PrefixCodeDecodedIter<'a, P, B> {
    #[allow(unsafe_code)]
    fn new(hashes: &'a [u8], maximal_bit_index: usize, hash_bits: u8) -> Self {
        debug_assert!(
            maximal_bit_index <= hashes.len() * 8 - GapHash::<P, B>::rank_index_total_size(usize::from(hash_bits)),
            "The bit index ({maximal_bit_index}) must be less or equal to the number of bits in the hashes."
        );

        Self {
            iter: PrefixCodeIter::new(hashes, maximal_bit_index, hash_bits),
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for PrefixCodeDecodedIter<'a, P, B> {
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|hash| GapHash::<P, B>::decode(hash, self.iter.hash_bits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperloglog_derive::test_precisions_and_bits;
    use crate::prelude::*;

    #[test]
    #[allow(unsafe_code)]
    fn test_rank_index_initialization() {
        for hash_bits in 8..32 {
            let mut hashes = [0u8; 64];
            let rank_index_bitindex_size =
                GapHash::<Precision11, Bits4>::rank_index_bits() as usize;
            assert_eq!(GapHash::<Precision11, Bits4>::rank_index_capacity(), 11);
            GapHash::<Precision11, Bits4>::initialize_rank_index(&mut hashes, hash_bits);

            // We expect the just initialized rank index to be a series of 'rank_index_bitindex_size' ones
            // each one followed by 'hash_bits' zeros.
            let hashes = unsafe {
                core::slice::from_raw_parts(
                    hashes.as_ptr() as *const u32,
                    hashes.len() / core::mem::size_of::<u32>(),
                )
            };
            let mut reader = BitReader::new(&hashes);
            for _ in 0..(GapHash::<Precision11, Bits4>::rank_index_capacity() - 1) {
                assert_eq!(
                    reader.read_bits(rank_index_bitindex_size),
                    (1 << rank_index_bitindex_size) - 1
                );
                assert_eq!(reader.read_bits(usize::from(hash_bits)), 0);
            }
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_update_rank_index() {
        let mut random_state = 4_575_763_274_578_236u64;
        let mut hashes = [0u8; 64];
        for hash_bits in 8..32 {
            random_state = splitmix64(random_state);

            let expected_bucket_size = (1usize << hash_bits)
                .div_ceil(GapHash::<Precision11, Bits4>::rank_index_capacity());

            for mut fake_hash in iter_random_values::<u64>(1_000, None, None) {
                // We adjust the fake hash to the current hash bits.
                fake_hash &= (1 << hash_bits) - 1;

                GapHash::<Precision11, Bits4>::initialize_rank_index(&mut hashes, hash_bits);

                // We now determine the expected bucket.
                debug_assert!(
                    fake_hash as usize / expected_bucket_size < 11,
                    "Hash: {fake_hash}, Expected bucket size: {expected_bucket_size}."
                );

                let expected_bucket = GapHash::<Precision11, Bits4>::rank_index_capacity()
                    - fake_hash as usize / expected_bucket_size;

                // We try to get the best starting point for the iterator from the rank index
                // Since the index is empty, it should return as position 'hash_bits', as if
                // returning the position associated to the first hash in the hash list.
                // While this cannot happen in practice since the index is created upon preliminary
                // saturation of the hash list, in our case we do not have any first hash in the
                // hash list and as such we expect the rank index to return zero.
                let (bit_index, bucket_hash) =
                    GapHash::<Precision11, Bits4>::best_search_start(&hashes, hash_bits, fake_hash);

                assert_eq!(
                    bit_index,
                    0,
                    "Found unexpected bit index for hash {fake_hash}."
                );
                assert_eq!(bucket_hash, 0, "Found unexpected bucket hash.");

                let determined_bucket =
                    GapHash::<Precision11, Bits4>::rank_index_hash_bucket(hash_bits, fake_hash);

                assert_eq!(determined_bucket, expected_bucket);

                for bit_index in
                    GapHash::<Precision11, Bits4>::rank_index_total_size(usize::from(hash_bits))..64
                {
                    // We reset the hashes to the initial state.
                    GapHash::<Precision11, Bits4>::initialize_rank_index(&mut hashes, hash_bits);

                    // We update the rank index with the fake hash.
                    GapHash::<Precision11, Bits4>::update_rank_index(
                        &mut hashes,
                        hash_bits,
                        bit_index,
                        fake_hash,
                    );

                    // We check that the rank index has been updated correctly, as now the best starting
                    // point for the iterator should be the position of the fake hash.
                    let (best_bit_index, best_bucket_hash) =
                        GapHash::<Precision11, Bits4>::best_search_start(
                            &hashes, hash_bits, fake_hash,
                        );

                    assert_eq!(best_bit_index, bit_index);
                    assert_eq!(best_bucket_hash, fake_hash);
                }
            }
        }
    }

    #[test_precisions_and_bits]
    /// Test that the apply_gap function works as expected.
    fn test_apply_gap<P: Precision, B: Bits>() where P: ArrayRegister<B> {
        const NUMBER_OF_HASHES: usize = 1_000;
        let mut first_random_state = 4_575_763_274_578_236u64;
        let mut second_random_state = 564_655_678_565_685_654u64;
        let mut third_random_state = 324_587_447_578_236u64;

        // We start from the maximal number of bits for the hash.
        for hash_bits in GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS..=GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS {

            first_random_state = splitmix64(first_random_state);
            second_random_state = splitmix64(second_random_state);
            third_random_state = splitmix64(third_random_state);
            for ((first, second), third)in iter_random_values::<u64>(NUMBER_OF_HASHES as u64, None, Some(first_random_state)).zip(iter_random_values::<u64>(NUMBER_OF_HASHES as u64, None, Some(second_random_state))).zip(iter_random_values::<u64>(NUMBER_OF_HASHES as u64, None, Some(third_random_state))) {
                let (first_index, first_register, first_original_hash) =
                    <PlusPlus<
                        P,
                        B,
                        <P as ArrayRegister<B>>::Packed,
                    >>::index_and_register_and_hash(&first);

                let first_encoded_hash = GapHash::<P, B>::encode(first_index, first_register, first_original_hash, hash_bits);

                let (second_index, second_register, second_original_hash) =
                    <PlusPlus<
                        P,
                        B,
                        <P as ArrayRegister<B>>::Packed,
                    >>::index_and_register_and_hash(&second);

                let second_encoded_hash = GapHash::<P, B>::encode(second_index, second_register, second_original_hash, hash_bits);

                let (third_index, third_register, third_original_hash) =
                    <PlusPlus<
                        P,
                        B,
                        <P as ArrayRegister<B>>::Packed,
                    >>::index_and_register_and_hash(&third);

                let third_encoded_hash = GapHash::<P, B>::encode(third_index, third_register, third_original_hash, hash_bits);

                let largest = core::cmp::max(first_encoded_hash, core::cmp::max(second_encoded_hash, third_encoded_hash));
                let smallest = core::cmp::min(first_encoded_hash, core::cmp::min(second_encoded_hash, third_encoded_hash));
                let medium = first_encoded_hash + second_encoded_hash + third_encoded_hash - largest - smallest;

                if largest == smallest || largest == medium || medium == smallest {
                    continue;
                }

                // We now compute the gap between the largest and the smallest encoded hashes.
                let largest_to_smallest_gap = GapHash::<P, B>::into_gap_fragment(largest, smallest, hash_bits);

                // Next, we try to apply the same gap to the medium hash, with the expectation
                // that it should be equal to computing directly the gap with the largest hash.
                let largest_to_medium_deduced_gap = GapHash::<P, B>::apply_gap(smallest, largest_to_smallest_gap,  medium, hash_bits);
                let largest_to_medium_direct_gap = GapHash::<P, B>::into_gap_fragment(largest, medium, hash_bits);

                assert_eq!(largest_to_medium_deduced_gap, largest_to_medium_direct_gap, "Failed to apply gap at hash size {hash_bits}.");
            }
        }
    }

}
