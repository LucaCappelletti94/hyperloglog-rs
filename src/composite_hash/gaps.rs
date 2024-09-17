//! Gap-based composite hash implementation.
use core::marker::PhantomData;
use core::u64;
mod bitreader;
mod bitwriter;
mod optimal_codes;
use super::{
    switch::{DecodedIter, DowngradedIter},
    Debug, LastBufferedBit, Precision, SaturationError, SwitchHash,
};
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
    pub uniform_delta: u32,
    /// The bits expected to have geometric distribution.
    pub geometric_minus_one: u8,
}

struct EncodedHashEnvironment {
    previous_to_current_gap: Option<GapFragment>,
    current_to_next_gap: Option<GapFragment>,
    number_of_inserted_bits: u32,
    previous_to_current_size: u32,
    current_to_next_size: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct InsertMetadata {
    pub(crate) hash_bits: u8,
    pub(crate) duplicates: u32,
    pub(crate) bit_index: u32,
}

trait SkipSliceAhead {
    fn reserve(self, position: usize) -> Self;
    fn len(&self) -> usize;
}

impl SkipSliceAhead for &[u8] {
    #[inline]
    fn reserve(self, position: usize) -> Self {
        &self[..self.len() - position]
    }

    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl SkipSliceAhead for &mut [u8] {
    #[inline]
    fn reserve(self, position: usize) -> Self {
        let number_of_entries = <[u8]>::len(self);
        &mut self[..number_of_entries - position]
    }

    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl<P: Precision, B: Bits> GapHash<P, B> {
    #[inline]
    /// Returns the gap encoding for the given SwitchHash.
    pub fn into_gap_fragment(
        previous_hash: u32,
        hash_to_encode: u32,
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
            GapFragment {
                uniform_delta: previous_fragment.index - fragment_to_encode.index,
                geometric_minus_one: fragment_to_encode.register - 1,
            }
        } else {
            // The uniform portion of the hash is composed by the index and the hash remainder.
            let previous_uniform = previous_fragment.uniform(hash_bits);
            let to_encode_uniform = fragment_to_encode.uniform(hash_bits);

            let uniform_delta = if previous_uniform > to_encode_uniform {
                ((previous_uniform - to_encode_uniform) << 1) - 1
            } else {
                (to_encode_uniform - previous_uniform) << 1
            };
            GapFragment {
                uniform_delta,
                geometric_minus_one: fragment_to_encode.register - 1,
            }
        }
    }

    #[inline]
    /// Applies a gap to a replacement hash.
    ///
    /// # Arguments
    /// * `current_hash` - The current hash.
    /// * `current_hash_gap_fragment` - The gap fragment between a previous (undecoded) hash and the current hash.
    /// * `replacement_hash` - The hash to apply the gap to.
    /// * `hash_bits` - The number of bits used to decode the hashes.
    /// * `target_hash_bits` - The number of bits used to encode the target hash.
    ///
    /// # Implementative details
    /// The apply gap method is meant to apply the gap between an unknown previous hash that would be too expensive to decode
    /// and the current hash to a replacement hash.
    fn apply_gap(
        current_hash: u32,
        current_hash_gap_fragment: GapFragment,
        replacement_hash: u32,
        hash_bits: u8,
    ) -> GapFragment {
        // The replacement hash must be strictly greater than the current hash.
        debug_assert!(current_hash < replacement_hash);

        let current_hash_fragment = SwitchHash::<P, B>::scompose_hash(current_hash, hash_bits);
        let replacement_hash_fragment =
            SwitchHash::<P, B>::scompose_hash(replacement_hash, hash_bits);

        if P::EXPONENT + B::NUMBER_OF_BITS == hash_bits {
            let previous_hash_index =
                current_hash_gap_fragment.uniform_delta + current_hash_fragment.index;
            let uniform_delta = previous_hash_index - replacement_hash_fragment.index;

            GapFragment {
                uniform_delta,
                geometric_minus_one: replacement_hash_fragment.register - 1,
            }
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

            GapFragment {
                uniform_delta,
                geometric_minus_one: replacement_hash_fragment.register - 1,
            }
        }
    }

    #[inline]
    /// Whether the hashes are currently employing an index to speed up rank operations.
    ///
    /// # Implementative details
    /// Since an index uses up some of the bits that we could use to encode the hash,
    /// we only employ it for cases where we store a large number of hashes.
    pub(super) const fn has_rank_index() -> bool {
        (1_u64 << P::EXPONENT) * B::NUMBER_OF_BITS as u64 / Self::SMALLEST_VIABLE_HASH_BITS as u64
            > 500
    }

    #[inline]
    /// Returns the number of bits necessary to encode the rank index.
    const fn rank_index_bits() -> u8 {
        P::EXPONENT
            + match B::NUMBER_OF_BITS {
                4 => 2,
                5 | 6 => 3,
                _ => unreachable!(),
            }
    }

    #[inline]
    /// Returns the capacity of the rank index.
    const fn rank_index_exponent() -> u8 {
        P::EXPONENT - 9
    }

    #[inline]
    /// Returns the capacity of the rank index.
    const fn rank_index_capacity() -> u32 {
        1 << Self::rank_index_exponent()
    }

    #[inline]
    #[must_use]
    /// Returns the total expected size of the rank index.
    pub const fn rank_index_total_size(hash_bits: u8) -> u32 {
        if Self::has_rank_index() {
            (Self::rank_index_capacity() - 1) * (Self::rank_index_bits() + hash_bits) as u32
        } else {
            0
        }
    }

    #[inline]
    #[must_use]
    /// Returns the total padded size of the rank index.
    pub const fn rank_index_padded_size(hash_bits: u8) -> u32 {
        Self::rank_index_total_size(hash_bits).div_ceil(8) * 8
    }

    #[inline]
    /// Returns the index of the rank index associated to the given hash.
    const fn rank_index_hash_bucket(hash_bits: u8, hash: u32) -> u32 {
        Self::rank_index_capacity() - hash / (1u32 << (hash_bits - Self::rank_index_exponent())) - 1
    }

    #[inline]
    /// Returns the index mask for the non-initialized index.
    const fn rank_index_mask() -> u32 {
        (1 << Self::rank_index_bits()) - 1
    }

    #[inline]
    /// Returns the index offset given the hashes.
    const fn rank_index_offset(hashes: &[u8], hash_bits: u8) -> u32 {
        hashes.len() as u32 * 8 - Self::rank_index_total_size(hash_bits)
    }

    #[allow(unsafe_code)]
    #[inline]
    /// Initializes the rank index with the given hashes.
    fn initialize_rank_index(hashes: &mut [u8], hash_bits: u8) {
        debug_assert!(Self::has_rank_index());

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr().cast::<u64>(),
                hashes.len() / size_of::<u64>(),
            )
        };

        let mut writer = BitWriter::new(hashes64);
        writer.seek(Self::rank_index_offset(hashes, hash_bits));

        for _ in 1..Self::rank_index_capacity() {
            writer.write_bits(Self::rank_index_mask(), Self::rank_index_bits());
            writer.write_bits(0u64, hash_bits);
        }

        debug_assert_eq!(
            writer.tell(),
            hashes.len() as u32 * 8,
            "The rank index must be fully initialized."
        );
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
    fn shift_index(
        hashes: &mut [u8],
        hash: u32,
        bit_index_after_hash: u32,
        successor_hash: u32,
        hash_bits: u8,
        shift: u32,
    ) {
        let hash_bucket = Self::rank_index_hash_bucket(hash_bits, hash);

        // If the hash bucket is the last one, we do not have any subsequent buckets to update.
        if hash_bucket == Self::rank_index_capacity() - 1 {
            return;
        }

        let bucket_size = u32::from(hash_bits + Self::rank_index_bits());
        let hash_bucket_position = hash_bucket * bucket_size;

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr().cast::<u64>(),
                hashes.len() / size_of::<u64>(),
            )
        };

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr().cast::<u32>(),
                hashes.len() / size_of::<u32>(),
            )
        };

        let mut reader = BitReader::skip(
            hashes32,
            Self::rank_index_offset(hashes, hash_bits) + hash_bucket_position,
        );
        let mut writer = BitWriter::new(hashes64);
        let mut current_bit_index;

        for bucket in hash_bucket..Self::rank_index_capacity() - 1 {
            writer.seek(Self::rank_index_offset(hashes, hash_bits) + bucket * bucket_size);

            current_bit_index = reader.read_bits(Self::rank_index_bits()) as u32;
            // We move the reader ahead by hash_bits bits so it is positioned at the next entry.
            let current_bucket_hash = reader.read_bits(hash_bits) as u32;

            // We check that the reader tell is within bounds.
            debug_assert!(
                reader.last_read_bit_position() >= Self::rank_index_offset(hashes, hash_bits),
                "The reader tell ({}) must be less than the total size of the rank index ({}). We are at bucket {bucket} and started at hash bucket {hash_bucket}. The bucket bitsize is {bucket_size}.",
                reader.last_read_bit_position(),
                Self::rank_index_total_size(hash_bits)
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
                bit_index_after_hash
            } else {
                current_bit_index + shift
            };

            // We check that the shifted index is always less than the mask
            // and that the bit index is within bounds.
            debug_assert!(
                shifted_index < Self::rank_index_mask(),
                "The shifted index must not be equal to the mask."
            );

            // We update the bit index.
            writer.write_bits(shifted_index as u64, Self::rank_index_bits());
        }

        // When we are done updating, the reader must be at the end of the rank index
        // and the writer must be at the end of the rank index minus hash_bits bits.
        debug_assert_eq!(reader.last_read_bit_position(), hashes.len() as u32 * 8,);
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
    fn update_rank_index(hashes: &mut [u8], hash_bits: u8, bit_index: u32, hash: u32) {
        let hash_bucket = Self::rank_index_hash_bucket(hash_bits, hash);

        if hash_bucket == 0 {
            return;
        }

        let hash_bucket_position =
            (hash_bucket - 1) * u32::from(Self::rank_index_bits() + hash_bits);

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr().cast::<u32>(),
                hashes.len() / size_of::<u32>(),
            )
        };

        let mut reader = BitReader::skip(
            hashes32,
            Self::rank_index_offset(hashes, hash_bits) + hash_bucket_position,
        );
        let bucket_bit_index = reader.read_bits(Self::rank_index_bits()) as u32;
        let bucket_hash = reader.read_bits(hash_bits) as u32;

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr().cast::<u64>(),
                hashes.len() / size_of::<u64>(),
            )
        };

        // If the hash we received is smaller than the hash in the bucket, we update the bucket.
        if bucket_hash < hash || bucket_bit_index == Self::rank_index_mask() {
            let mut writer = BitWriter::new(hashes64);
            writer.seek(Self::rank_index_offset(hashes, hash_bits) + hash_bucket_position);
            writer.write_bits(bit_index, Self::rank_index_bits());
            writer.write_bits(hash, hash_bits);
        }
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
    fn best_search_start(hashes: &[u8], hash_bits: u8, hash: u32) -> (u32, u32) {
        let hash_bucket = Self::rank_index_hash_bucket(hash_bits, hash);

        let hashes32 = unsafe {
            core::slice::from_raw_parts(
                hashes.as_ptr().cast::<u32>(),
                hashes.len() / size_of::<u32>(),
            )
        };

        for bucket in (0..=hash_bucket).rev() {
            if bucket == 0 {
                let mut reader = BitReader::new(hashes32);
                let first_hash = reader.read_bits(hash_bits) as u32;

                return (0, first_hash);
            }

            let hash_bucket_position =
                (bucket - 1) * u32::from(Self::rank_index_bits() + hash_bits);

            let mut reader = BitReader::skip(
                hashes32,
                Self::rank_index_offset(hashes, hash_bits) + hash_bucket_position,
            );

            let bit_index = reader.read_bits(Self::rank_index_bits()) as u32;
            let hash = reader.read_bits(hash_bits) as u32;

            debug_assert!(
                reader.last_read_bit_position() >= Self::rank_index_offset(hashes, hash_bits),
            );

            if bit_index != Self::rank_index_mask() {
                return (bit_index, hash);
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

        // If the hashes are more than half of the expected maximal capacity, we recerve the last
        // `rank_index_padded_size` bytes for the rank index.

        let maximal_size = ((1 << P::EXPONENT) * usize::from(B::NUMBER_OF_BITS)).div_ceil(64) * 8;

        if hashes.len() > maximal_size / 2 {
            hashes.reserve(Self::rank_index_padded_size(hash_bits) as usize)
        } else {
            hashes
        }
    }
}

impl<P: Precision, B: Bits> GapHash<P, B> {
    #[inline]
    #[must_use]
    /// Returns whether the hashes are currently to be considered prefix-free-encoded.
    pub fn is_prefix_free_encoded(number_of_hashes: u32, hash_bits: u8, bit_index: u32) -> bool {
        hash_bits < Self::LARGEST_VIABLE_HASH_BITS
            || number_of_hashes * u32::from(hash_bits) > bit_index
    }

    #[inline]
    fn b(hash_bits: u8) -> (u8, u8) {
        let data =
            OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];

        for (target_hash_bits, uniform) in data {
            if *target_hash_bits == hash_bits {
                return (*target_hash_bits, *uniform);
            }
        }

        unreachable!("The hash bits ({hash_bits}) must be one of the optimal hash bits.",);
    }

    #[inline]
    fn uniform_coefficient(hash_bits: u8) -> u8 {
        Self::b(hash_bits).1
    }
}

impl<P: Precision, B: Bits> GapHash<P, B> {
    #[inline]
    #[must_use]
    pub(crate) fn downgraded(
        hashes: &[u8],
        number_of_hashes: u32,
        hash_bits: u8,
        bit_index: u32,
        shift: u8,
    ) -> DispatchedDowngradedIter<'_, P, B> {
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
    pub(crate) fn decoded(
        hashes: &[u8],
        number_of_hashes: u32,
        hash_bits: u8,
        bit_index: u32,
    ) -> DispatchedDecodedIter<'_, P, B> {
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
    pub fn encode(index: usize, register: u8, original_hash: u64, hash_bits: u8) -> u32 {
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << P::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            P::EXPONENT,
        );
        SwitchHash::<P, B>::encode(index, register, original_hash, hash_bits)
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    pub(super) fn decode(hash: u32, hash_bits: u8) -> (u8, usize) {
        SwitchHash::<P, B>::decode(hash, hash_bits)
    }

    #[inline]
    #[must_use]
    /// Downgrade the hash into a smaller hash.
    pub fn downgrade(hash: u32, hash_bits: u8, shift: u8) -> u32 {
        SwitchHash::<P, B>::downgrade(hash, hash_bits, shift)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(crate) fn find(
        hashes: &[u8],
        number_of_hashes: u32,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
        bit_index: u32,
    ) -> bool {
        debug_assert!(register > 0);
        debug_assert!(
            index < 1 << P::EXPONENT,
            "The index ({index}) must be less than 2^({})",
            P::EXPONENT,
        );
        let encoded_hash = Self::encode(index, register, original_hash, hash_bits);

        if Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
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
                encoded_hash,
                hash_bits,
                bit_index,
            )
            .is_ok()
        }
    }

    #[inline]
    fn encoded_hash_environment(
        previous_hash: Option<u32>,
        current_hash: u32,
        mut next_hash: Option<u32>,
        hash_bits: u8,
        last_read_bit_position: u32,
        last_read_bit_position_variation: u32,
        previous_gap_fragment: Option<GapFragment>,
        backup_next_hash: Option<u32>,
    ) -> EncodedHashEnvironment {
        let previous_to_current_gap = match (previous_hash, Self::has_rank_index()) {
            (None, true) => {
                // When we use a rank index and we have immediately found the value we position where
                // we would like to insert the value, it means we are replacing the first hash in the
                // bucket. While we have stored this hash in the index, and we do have the gap informations
                // between this hash and its successor, we do not have the true previous hash, which is the
                // gap between the previous hash and the previous previous hash. A trivial approach like
                // decoding the previous previous hash would require us to decode the entirety of the previous
                // bucket and would therefore be inefficient.
                next_hash = backup_next_hash;
                // If the reader is at the first hash, and was not positioned at some later bit index,
                // we truly do not have a previous hash to speak of.
                if last_read_bit_position == 0 {
                    None
                } else {
                    // If we do have a previous hash, but it is unknown because we jumped the reader to the
                    // current position and we have not decoded all previous hashes, we necessarily must have
                    // a current hash (i.e. the first hash in the bucket) and we can compute the gap between
                    // the current hash and the previous (unknown) hash.
                    Some(Self::apply_gap(
                        next_hash.unwrap(),
                        previous_gap_fragment.unwrap(),
                        current_hash,
                        hash_bits,
                    ))
                }
            }
            (Some(_), true) | (_, false) => previous_hash.map(|previous_hash| {
                debug_assert!(previous_hash > current_hash);
                Self::into_gap_fragment(previous_hash, current_hash, hash_bits)
            }),
        };

        let current_to_next_gap = next_hash.map(|next_hash| {
            debug_assert!(next_hash < current_hash);
            Self::into_gap_fragment(current_hash, next_hash, hash_bits)
        });

        let previous_to_current_size =
            previous_to_current_gap.map_or(u32::from(hash_bits), |previous_to_current_gap| {
                len_rice(
                    previous_to_current_gap.uniform_delta,
                    Self::uniform_coefficient(hash_bits),
                    previous_to_current_gap.geometric_minus_one,
                )
            });

        // When the currently encoded value has a successor, we will be adding the gap between
        // the current hash and the next hash. If it does not have a successor, then there is
        // nothing to write. If there is no predecessor but there is a successor, it means that
        // we are replacing the first hash in the hash list, which has size 'hash_bits'.
        // Such delta is already accounted for in 'last_read_bit_position_variation'.
        let current_to_next_size: u32 = current_to_next_gap.map_or(0, |current_to_next_gap| {
            len_rice(
                current_to_next_gap.uniform_delta,
                Self::uniform_coefficient(hash_bits),
                current_to_next_gap.geometric_minus_one,
            )
        });

        debug_assert!(
            previous_to_current_size + current_to_next_size >= last_read_bit_position_variation,
            "The total size of the encoded hashes ({previous_to_current_size} + {current_to_next_size}) must be greater or equal to the last read bit position variation ({last_read_bit_position_variation}).",
        );

        let number_of_inserted_bits =
            previous_to_current_size + current_to_next_size - last_read_bit_position_variation;

        EncodedHashEnvironment {
            previous_to_current_gap,
            current_to_next_gap,
            number_of_inserted_bits,
            previous_to_current_size,
            current_to_next_size,
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(crate) fn insert_sorted_desc(
        hashes: &mut [u8],
        number_of_hashes: u32,
        bit_index: u32,
        index: usize,
        register: u8,
        original_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<InsertMetadata>, SaturationError> {
        if !Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index) {
            return match SwitchHash::<P, B>::insert_sorted_desc(
                Self::skip_rank_index(hashes, hash_bits),
                number_of_hashes,
                bit_index,
                Self::encode(index, register, original_hash, hash_bits),
                hash_bits,
            ) {
                Err(SaturationError::Saturation(_)) => {
                    // If the non-Prefix-Free-Encoded hash list is saturated, we must switch to the
                    // PFC variant, which generally requires a smaller number of bits to encode the hashes.
                    // This procedure will potentially reduce the number of hash bits employed, introduce
                    // duplicates (which are removed and we receive their number) and return the new bit index.
                    Self::insert_downgrading(
                        hashes,
                        number_of_hashes,
                        index,
                        register,
                        original_hash,
                        bit_index,
                        hash_bits,
                    )
                    .map(|metadata| Some(metadata))
                }
                other => other,
            };
        }

        debug_assert!(
            Self::is_prefix_free_encoded(number_of_hashes, hash_bits, bit_index),
            "The hashes must be prefix-free encoded to be able to use prefix-free codes."
        );

        let hashes_ref: &[u8] =
            unsafe { core::slice::from_raw_parts(hashes.as_ptr().cast::<u8>(), hashes.len()) };

        let encoded_hash = Self::encode(index, register, original_hash, hash_bits);

        // iter until we find where we should insert
        let mut iter: PrefixCodeIter<'_, P, B> = if Self::has_rank_index() {
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
            last_read_bit_position <= Self::rank_index_offset(hashes, hash_bits),
            "Last read bit position ({last_read_bit_position}) must be greater or equal to the rank index size ({}), otherwise instead of reading hashes it would be reading the rank index.",
            Self::rank_index_total_size(hash_bits),
        );

        while let Some(value) = iter.next() {
            // The values are sorted in descending order, so we can stop when we find a value
            // that is less than or equal to the value we want to insert
            if encoded_hash >= value {
                // if the value is equal to the encoded_hash value, we don't need to insert it
                if value == encoded_hash {
                    return Ok(None);
                }
                last_read_bit_position_variation =
                    iter.last_read_bit_position() - last_read_bit_position;
                next_hash = Some(value);
                break;
            }

            last_read_bit_position = iter.last_read_bit_position();
            previous_hash = Some(value);
        }

        let EncodedHashEnvironment {
            previous_to_current_gap,
            current_to_next_gap,
            number_of_inserted_bits,
            previous_to_current_size,
            current_to_next_size,
        } = Self::encoded_hash_environment(
            previous_hash,
            encoded_hash,
            next_hash,
            hash_bits,
            last_read_bit_position,
            last_read_bit_position_variation,
            iter.previous_gap_fragment,
            Some(iter.previous),
        );

        if bit_index + number_of_inserted_bits > Self::rank_index_offset(hashes, hash_bits) {
            // If inserting the newly encoded hash would cause the hash list to overflow, we must
            // downgrade it. This will change the number of hash bits, bit index, and potentially
            // introduce duplicates which will therefore reduce the number of hashes.
            return Self::insert_downgrading(
                hashes,
                number_of_hashes,
                index,
                register,
                original_hash,
                bit_index,
                hash_bits,
            )
            .map(|metadata| Some(metadata));
        }

        let hashes64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr().cast::<u64>(),
                hashes.len() / size_of::<u64>(),
            )
        };

        let mut writer = BitWriter::new(hashes64);
        writer.seek(last_read_bit_position);

        // Now that we have determined where to insert the new value, the subsequent values
        // will be solely read from the bitstream and written to the writer.
        let mut bypass: BypassIter<'_> = iter.into_bypass();
        // In order to bring the reader a bit more ahead and make more unlikely to get
        // read-write conflicts, we read the next value.
        let mut next = bypass.next();

        // If there is no previos value, we would need to write the encoded value itself but
        // writing such a high value in prefix-free encoding would be inefficient. Therefore,
        // we write the first hash explicitly.
        if let Some(previous_to_current_gap) = previous_to_current_gap {
            let total_wrote = writer.write_rice(
                previous_to_current_gap.uniform_delta,
                previous_to_current_gap.geometric_minus_one,
                Self::uniform_coefficient(hash_bits),
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
                0,
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
                        bypass.len() == 0 || bypass.last_buffered_bit()  > writer.tell(),
                        "Reader tell ({}) must be greater than writer tell ({}) in insert at hash size {hash_bits}.",
                        bypass.last_buffered_bit(),
                        writer.tell(),
                    );
                }

                let writer_tell = writer.tell();

                // We check that practice matches theory:
                debug_assert_eq!(
                    writer_tell,
                    bit_index + number_of_inserted_bits,
                    "Expected writer tell (started at {last_read_bit_position} [{}], prev_hash: {previous_hash:?} next_hash {next_hash:?}) to match bit index {bit_index} + value variation {number_of_inserted_bits} = ({}).",
                    last_read_bit_position,
                    bit_index + number_of_inserted_bits
                );
            }
        }

        drop(writer);

        if Self::has_rank_index() {
            // If we are using a rank index, we may need to update the values in the index.
            Self::update_rank_index(hashes, hash_bits, last_read_bit_position, encoded_hash);

            if let Some(next_hash) = next_hash {
                // Furthermore, we need to shift the bit index associated to values that are smaller
                // than the current hash and therefore will be shifted to the right.
                Self::shift_index(
                    hashes,
                    encoded_hash,
                    last_read_bit_position + previous_to_current_size,
                    next_hash,
                    hash_bits,
                    number_of_inserted_bits,
                );
            }
        }

        Ok(Some(InsertMetadata {
            hash_bits,
            duplicates: 0,
            bit_index: bit_index + number_of_inserted_bits,
        }))
    }

    /// The smallest viable hash bits that can be employed.
    pub const SMALLEST_VIABLE_HASH_BITS: u8 = P::EXPONENT + B::NUMBER_OF_BITS;
    /// The largest viable hash bits that can be employed.
    pub const LARGEST_VIABLE_HASH_BITS: u8 = SwitchHash::<P, B>::LARGEST_VIABLE_HASH_BITS;

    #[inline]
    #[allow(unsafe_code)]
    /// Downgrade the hash into a smaller hash in place.
    fn insert_downgrading(
        hashes: &mut [u8],
        number_of_hashes: u32,
        index: usize,
        register: u8,
        original_hash: u64,
        bit_index: u32,
        mut hash_bits: u8,
    ) -> Result<InsertMetadata, SaturationError> {
        // First, we need to identify the target hash bits which will allow us to insert
        // a new hash.
        let original_hash_bits = hash_bits;
        let expected_maximal_bit_index;
        let mut uniform_coefficient: u8 = u8::MAX;
        let mut encoded_hash: u32;

        if hash_bits < Self::LARGEST_VIABLE_HASH_BITS {
            hash_bits -= 1;
        }

        loop {
            // If the hash is already prefix-free encoded, we need to search the next smaller
            // hash that we can use to encode the hashes.
            let data =
                OPTIMAL_RICE_COEFFICIENTS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];

            let mut found_smaller = false;

            for (target_hash_bits, target_uniform_coefficient) in data.iter().rev() {
                if hash_bits >= *target_hash_bits {
                    hash_bits = *target_hash_bits;
                    uniform_coefficient = *target_uniform_coefficient;
                    found_smaller = true;
                    break;
                }
            }

            if !found_smaller {
                // If we cannot find a smaller hash bits, we need to return a Saturation error.
                return Err(SaturationError::Saturation(bit_index));
            }

            // Then we verify whether this hash bits are viable, i.e. whether the size of the gaps encoded
            // with this particular hash bits are ALWAYS smaller than the current ones. If this is the case,
            // we can safely use this hash bits, otherwise we need to iterate to the next smaller hash bits.

            let mut iter = GapHash::<P, B>::downgraded(
                hashes,
                number_of_hashes,
                original_hash_bits,
                bit_index,
                original_hash_bits - hash_bits,
            );

            encoded_hash = Self::encode(index, register, original_hash, hash_bits);

            let mut reader_buffer_overflow = false;

            let mut previous_hash: Option<u32> = None;
            let mut writer_tell = 0;
            let mut maybe_next = iter.next();
            let mut maybe_double_next = iter.next();
            let mut encoded_hash_accounted_for = false;

            while let Some(next) = maybe_next {
                maybe_next = maybe_double_next;
                maybe_double_next = iter.next();

                if Some(next) == previous_hash {
                    continue;
                }

                // If is possible that by downgrading the hash bits, the encoded hash
                // will become equal to some other hash. In such cases, we will simply
                // mark it as a duplicate and skip it.
                if encoded_hash == next {
                    encoded_hash_accounted_for = true;
                }

                // We have already checked that encoded hash is not present
                // in the hash list, so we just need to check whether it is
                // larger than the current hash.
                writer_tell += if encoded_hash > next && !encoded_hash_accounted_for {
                    encoded_hash_accounted_for = true;

                    let previous_gap_size =
                        previous_hash.map_or(u32::from(hash_bits), |previous_hash| {
                            let gap =
                                Self::into_gap_fragment(previous_hash, encoded_hash, hash_bits);
                            len_rice(
                                gap.uniform_delta,
                                uniform_coefficient,
                                gap.geometric_minus_one,
                            )
                        });

                    let successor_gap_size = Self::into_gap_fragment(encoded_hash, next, hash_bits);
                    previous_gap_size
                        + len_rice(
                            successor_gap_size.uniform_delta,
                            uniform_coefficient,
                            successor_gap_size.geometric_minus_one,
                        )
                } else if let Some(previous_hash) = previous_hash {
                    let fragment =
                        GapHash::<P, B>::into_gap_fragment(previous_hash, next, hash_bits);
                    len_rice(
                        fragment.uniform_delta,
                        uniform_coefficient,
                        fragment.geometric_minus_one,
                    )
                } else {
                    // If we do not have a previous hash and we are not inserting the encoded hash,
                    // we need to write the next hash explicitly using hash bits.
                    u32::from(hash_bits)
                };

                previous_hash = Some(next);

                // If we have not accounted for the encoded hash as it is smaller than all the
                // hashes in the list, we need to add it as the last hash.
                if maybe_next.is_none() && !encoded_hash_accounted_for {
                    encoded_hash_accounted_for = true;
                    let gap = Self::into_gap_fragment(next, encoded_hash, hash_bits);
                    writer_tell += len_rice(
                        gap.uniform_delta,
                        uniform_coefficient,
                        gap.geometric_minus_one,
                    );
                }

                // If the writer tell with the current hash bits would end up overlapping
                // with the reader buffer, we need to try with a smaller hash bits.
                if writer_tell > iter.last_buffered_bit() || writer_tell > bit_index {
                    reader_buffer_overflow = true;
                    break;
                }
            }

            debug_assert!(encoded_hash_accounted_for);

            // If we encountered a reader overflow, or the new bit index is not small enough
            // to accomodate a new hash, we need to try with a smaller hash bits.
            if reader_buffer_overflow {
                hash_bits -= 1;
                continue;
            }

            // Otherwise we have identified the next hash bits to employ!
            expected_maximal_bit_index = writer_tell;
            break;
        }

        // safe because the slice is originally allocated as u64s
        debug_assert!(hashes.len() % size_of::<u64>() == 0);
        let hashes_64 = unsafe {
            core::slice::from_raw_parts_mut(
                hashes.as_mut_ptr().cast::<u64>(),
                hashes.len() / size_of::<u64>(),
            )
        };

        // Copy of the mutable reference of the hashes which we employ
        // to write the new hashes.
        let hashes_8: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(hashes.as_mut_ptr().cast::<u8>(), hashes.len())
        };

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!(
            "Downgrading from hash size {original_hash_bits} to {hash_bits}, from bit index {} to {}, total hashes bits {}, of which {} for rank index.",
            bit_index,
            expected_maximal_bit_index,
            hashes.len() * 8,
            Self::rank_index_total_size(hash_bits),
        );

        // We initialize the index before we start writing the hashes.
        if GapHash::<P, B>::has_rank_index() {
            GapHash::<P, B>::initialize_rank_index(hashes, hash_bits);
        }

        let mut writer = BitWriter::new(hashes_64);

        let mut iter = GapHash::<P, B>::downgraded(
            hashes,
            number_of_hashes,
            original_hash_bits,
            bit_index,
            original_hash_bits - hash_bits,
        );

        // We write the first hash explicitly, as otherwise it would be
        // written in a very inefficient way.
        let mut previous_hash: Option<u32> = None;
        let mut duplicates = 0;

        let mut maybe_next = iter.next();
        let mut maybe_double_next = iter.next();

        let mut encoded_hash_accounted_for = false;

        while let Some(next) = maybe_next {
            maybe_next = maybe_double_next;
            maybe_double_next = iter.next();

            if Some(next) == previous_hash {
                duplicates += 1;
                continue;
            }

            // If is possible that by downgrading the hash bits, the encoded hash
            // will become equal to some other hash. In such cases, we will simply
            // mark it as a duplicate and skip it.
            if encoded_hash == next {
                encoded_hash_accounted_for = true;
                duplicates += 1;
            }

            let total_wrote: usize = if encoded_hash > next && !encoded_hash_accounted_for {
                encoded_hash_accounted_for = true;

                (if let Some(previous_hash) = previous_hash {
                    let previous_to_current_gap =
                        Self::into_gap_fragment(previous_hash, encoded_hash, hash_bits);
                    if GapHash::<P, B>::has_rank_index() {
                        let previous_hash_bucket =
                            GapHash::<P, B>::rank_index_hash_bucket(hash_bits, previous_hash);
                        let current_hash_bucket =
                            GapHash::<P, B>::rank_index_hash_bucket(hash_bits, encoded_hash);

                        if current_hash_bucket > previous_hash_bucket {
                            GapHash::<P, B>::update_rank_index(
                                hashes_8,
                                hash_bits,
                                writer.tell(),
                                encoded_hash,
                            );
                        }
                    }
                    // If there is a previous gap, we write it.
                    writer.write_rice(
                        previous_to_current_gap.uniform_delta,
                        previous_to_current_gap.geometric_minus_one,
                        uniform_coefficient,
                    )
                } else {
                    // Otherwise this must be the first hash in the list.
                    debug_assert_eq!(writer.tell(), 0);
                    usize::from(writer.write_bits(encoded_hash, hash_bits))
                }) + {
                    let current_to_next_gap =
                        Self::into_gap_fragment(encoded_hash, next, hash_bits);
                    if GapHash::<P, B>::has_rank_index() {
                        let previous_hash_bucket =
                            GapHash::<P, B>::rank_index_hash_bucket(hash_bits, encoded_hash);
                        let current_hash_bucket =
                            GapHash::<P, B>::rank_index_hash_bucket(hash_bits, next);

                        if current_hash_bucket > previous_hash_bucket {
                            GapHash::<P, B>::update_rank_index(
                                hashes_8,
                                hash_bits,
                                writer.tell(),
                                next,
                            );
                        }
                    }
                    writer.write_rice(
                        current_to_next_gap.uniform_delta,
                        current_to_next_gap.geometric_minus_one,
                        uniform_coefficient,
                    )
                }
            } else if let Some(previous_hash) = previous_hash {
                let fragment = GapHash::<P, B>::into_gap_fragment(previous_hash, next, hash_bits);
                if GapHash::<P, B>::has_rank_index() {
                    let previous_hash_bucket =
                        GapHash::<P, B>::rank_index_hash_bucket(hash_bits, previous_hash);
                    let current_hash_bucket =
                        GapHash::<P, B>::rank_index_hash_bucket(hash_bits, next);

                    if current_hash_bucket > previous_hash_bucket {
                        GapHash::<P, B>::update_rank_index(
                            hashes_8,
                            hash_bits,
                            writer.tell(),
                            next,
                        );
                    }
                }
                writer.write_rice(
                    fragment.uniform_delta,
                    fragment.geometric_minus_one,
                    uniform_coefficient,
                )
            } else {
                usize::from(writer.write_bits(next, hash_bits))
            };

            previous_hash = Some(next);

            // If we are at the last hash, and we have not accounted for the encoded hash, we need to write it.
            if maybe_next.is_none() && !encoded_hash_accounted_for {
                encoded_hash_accounted_for = true;
                let previous_to_current_gap =
                    Self::into_gap_fragment(next, encoded_hash, hash_bits);

                writer.write_rice(
                    previous_to_current_gap.uniform_delta,
                    previous_to_current_gap.geometric_minus_one,
                    uniform_coefficient,
                );
            }

            debug_assert!(
                iter.last_buffered_bit() >= writer.tell(),
                "Reader tell ({}) must be greater than writer tell ({}, just wrote {total_wrote}, {previous_hash:?} - {next}) in downgrade from hash size {original_hash_bits} to {hash_bits}. Precision: {}, Bits: {}.",
                iter.last_buffered_bit(),
                writer.tell(),
                P::EXPONENT,
                B::NUMBER_OF_BITS
            );
        }

        debug_assert!(encoded_hash_accounted_for);

        let writer_tell = writer.tell();

        drop(writer);

        debug_assert!(
            writer_tell <= bit_index,
            "PFC-ing from hash bits {original_hash_bits} to {hash_bits} must decrease the bit index ({bit_index}), but got writer tell {writer_tell} (maximal expected {expected_maximal_bit_index}). Precision: {}, Bits: {}.",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        );

        debug_assert_eq!(
            writer_tell, expected_maximal_bit_index,
            "The writer tell ({writer_tell}) must match expected new bit index ({expected_maximal_bit_index}).",
        );

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!(
            "Writer tell at {writer_tell} (maximal expected {expected_maximal_bit_index}), started from {bit_index}, with {duplicates} duplicates.",
        );

        debug_assert_eq!(
            PrefixCodeIter::<P, B>::new(
                hashes,
                writer_tell,
                hash_bits,
            )
            .count() as u32,
            number_of_hashes + 1 - duplicates,
            "The number of hashes (minus duplicates) must be the same after downgrading. Current writer tell: {writer_tell}, started from {bit_index}."
        );

        #[cfg(test)]
        #[cfg(feature = "std")]
        println!("Downgrading completed with {} duplicates.", duplicates);

        Ok(InsertMetadata {
            hash_bits,
            duplicates,
            bit_index: writer_tell,
        })
    }
}

#[derive(Debug)]
/// Iterator over downgraded hashes.
pub enum DispatchedDowngradedIter<'a, P: Precision, B: Bits> {
    /// Variants for when the prefix-free codes are used.
    PrefixCodeDowngradedIter(PrefixCodeDowngradedIter<'a, P, B>),
    /// Variants for when the prefix-free codes are not used.
    InnerDowngradedIter(DowngradedIter<'a, P, B>),
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for DispatchedDowngradedIter<'a, P, B> {
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDowngradedIter(iter) => iter.last_buffered_bit(),
        }
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        match self {
            Self::PrefixCodeDowngradedIter(iter) => iter.iter.hash_bits,
            Self::InnerDowngradedIter(iter) => iter.hash_bits(),
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for DispatchedDowngradedIter<'a, P, B> {
    type Item = u32;

    #[inline]
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
    bit_index: u32,
}

impl Iterator for BypassIter<'_> {
    type Item = (u64, u8);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bitstream.last_read_bit_position() >= self.bit_index {
            return None;
        }
        let n_bits =
            core::cmp::min(64, self.bit_index - self.bitstream.last_read_bit_position()) as u8;
        Some((self.bitstream.read_bits(n_bits), n_bits))
    }
}

impl ExactSizeIterator for BypassIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.bit_index
            .saturating_sub(self.bitstream.last_read_bit_position())
            .div_ceil(64) as usize
    }
}

impl<'a> LastBufferedBit for BypassIter<'a> {
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        self.bitstream.last_buffered_bit_position()
    }

    #[inline]
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
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        self.iter.last_buffered_bit()
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        self.iter.hash_bits
    }
}

impl<'a, P: Precision, B: Bits> PrefixCodeDowngradedIter<'a, P, B> {
    #[inline]
    fn new(hashes: &'a [u8], maximal_bit_index: u32, hash_bits: u8, shift: u8) -> Self {
        Self {
            iter: PrefixCodeIter::new(hashes, maximal_bit_index, hash_bits),
            shift,
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for PrefixCodeDowngradedIter<'a, P, B> {
    type Item = u32;

    #[inline]
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
    previous: u32,
    first: bool,
    hash_bits: u8,
    maximal_bit_index: u32,
    previous_gap_fragment: Option<GapFragment>,
    previous_index: u32,
    previous_hash_remainder: u32,
    previous_uniform: u32,
    uniform_coefficient: u8,
    #[cfg(test)]
    iteration: usize,
    _phantom: PhantomData<GapHash<P, B>>,
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for PrefixCodeIter<'a, P, B> {
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        self.bitstream.last_buffered_bit_position()
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        self.hash_bits
    }
}

impl<'a, P: Precision, B: Bits> PrefixCodeIter<'a, P, B> {
    #[inline]
    fn last_read_bit_position(&self) -> u32 {
        self.bitstream.last_read_bit_position()
    }

    #[inline]
    fn into_bypass(self) -> BypassIter<'a> {
        BypassIter {
            bitstream: self.bitstream,
            bit_index: self.maximal_bit_index,
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    fn new_with_rank_index(
        hashes: &'a [u8],
        maximal_bit_index: u32,
        hash_bits: u8,
        current_hash: u32,
    ) -> Self {
        debug_assert!(GapHash::<P, B>::has_rank_index());

        // We find the best starting point for the iterator that is closest to the current hash.
        let (bit_index, bucket_hash) =
            GapHash::<P, B>::best_search_start(hashes, hash_bits, current_hash);

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
                    core::slice::from_raw_parts(
                        hashes.as_ptr().cast::<u32>(),
                        hashes.len() / size_of::<u32>(),
                    )
                },
                bit_index,
            ),
            hash_bits,
            maximal_bit_index,
            previous_gap_fragment: None,
            previous_uniform: u32::MAX,
            previous_index: u32::MAX,
            previous_hash_remainder: u32::MAX,
            uniform_coefficient: GapHash::<P, B>::uniform_coefficient(hash_bits),
            #[cfg(test)]
            iteration: 0,
            _phantom: PhantomData,
        }
    }

    #[allow(unsafe_code)]
    #[inline]
    fn new(hashes: &'a [u8], maximal_bit_index: u32, hash_bits: u8) -> Self {
        Self {
            previous: u32::MAX,
            first: true,
            bitstream: BitReader::new(unsafe {
                core::slice::from_raw_parts(
                    hashes.as_ptr().cast::<u32>(),
                    hashes.len() / size_of::<u32>(),
                )
            }),
            hash_bits,
            maximal_bit_index,
            previous_gap_fragment: None,
            previous_uniform: u32::MAX,
            previous_index: u32::MAX,
            previous_hash_remainder: u32::MAX,
            uniform_coefficient: GapHash::<P, B>::uniform_coefficient(hash_bits),
            #[cfg(test)]
            iteration: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for PrefixCodeIter<'a, P, B> {
    type Item = u32;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;

            if self.bitstream.last_read_bit_position() == 0 {
                self.previous = self.bitstream.read_bits(self.hash_bits) as u32;
            } else {
                let uniform_delta = self.bitstream.read_rice(self.uniform_coefficient);
                let geometric_minus_one = self.bitstream.read_unary();
                self.previous_gap_fragment = Some(GapFragment {
                    uniform_delta,
                    geometric_minus_one,
                });
            }

            let hash_fragment = SwitchHash::<P, B>::scompose_hash(self.previous, self.hash_bits);

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
        if self.maximal_bit_index <= self.bitstream.last_read_bit_position() {
            return None;
        }
        #[cfg(test)]
        {
            self.iteration += 1;
        }

        let uniform_delta = self.bitstream.read_rice(self.uniform_coefficient);
        let geometric_minus_one = self.bitstream.read_unary();
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
                GapHash::<P, B>::rank_index_total_size(self.hash_bits)
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
                GapHash::<P, B>::rank_index_total_size(self.hash_bits)
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
            GapHash::<P, B>::rank_index_total_size(self.hash_bits)
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
    InnerDecodedIter(DecodedIter<'a, P, B>),
}

impl<'a, P: Precision, B: Bits> LastBufferedBit for DispatchedDecodedIter<'a, P, B> {
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.last_buffered_bit(),
            Self::InnerDecodedIter(iter) => iter.last_buffered_bit(),
        }
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        match self {
            Self::PrefixCodeDecodedIter(iter) => iter.hash_bits(),
            Self::InnerDecodedIter(iter) => iter.hash_bits(),
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for DispatchedDecodedIter<'a, P, B> {
    type Item = (u8, usize);

    #[inline]
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
    #[inline]
    fn last_buffered_bit(&self) -> u32 {
        self.iter.last_buffered_bit()
    }

    #[inline]
    fn hash_bits(&self) -> u8 {
        self.iter.hash_bits()
    }
}

impl<'a, P: Precision, B: Bits> PrefixCodeDecodedIter<'a, P, B> {
    #[inline]
    fn new(hashes: &'a [u8], maximal_bit_index: u32, hash_bits: u8) -> Self {
        Self {
            iter: PrefixCodeIter::new(hashes, maximal_bit_index, hash_bits),
        }
    }
}

impl<'a, P: Precision, B: Bits> Iterator for PrefixCodeDecodedIter<'a, P, B> {
    type Item = (u8, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|hash| GapHash::<P, B>::decode(hash, self.iter.hash_bits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use hyperloglog_derive::test_precisions_and_bits;

    #[test]
    #[allow(unsafe_code)]
    fn test_rank_index_initialization() {
        for hash_bits in 8..32 {
            let mut hashes = [0u8; 256];
            let rank_index_bitindex_size = GapHash::<Precision11, Bits4>::rank_index_bits();
            assert_eq!(GapHash::<Precision11, Bits4>::rank_index_capacity(), 4);
            GapHash::<Precision11, Bits4>::initialize_rank_index(&mut hashes, hash_bits);

            // We expect the just initialized rank index to be a series of 'rank_index_bitindex_size' ones
            // each one followed by 'hash_bits' zeros.
            let hashesu32 = unsafe {
                core::slice::from_raw_parts(
                    hashes.as_ptr().cast::<u32>(),
                    hashes.len() / size_of::<u32>(),
                )
            };
            let mut reader = BitReader::skip(
                hashesu32,
                GapHash::<Precision11, Bits4>::rank_index_offset(&hashes, hash_bits),
            );
            for _ in 0..(GapHash::<Precision11, Bits4>::rank_index_capacity() - 1) {
                assert_eq!(
                    reader.read_bits(rank_index_bitindex_size),
                    (1 << rank_index_bitindex_size) - 1
                );
                assert_eq!(reader.read_bits(hash_bits), 0);
            }
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_update_rank_index() {
        let mut random_state = 4_575_763_274_578_236u64;
        let mut hashes = [0u8; 256];
        for hash_bits in 8..32 {
            random_state = splitmix64(random_state);

            let expected_bucket_size =
                1u32 << (hash_bits - GapHash::<Precision11, Bits4>::rank_index_exponent());

            for mut fake_hash in iter_random_values::<u32>(1_000, None, None) {
                // We adjust the fake hash to the current hash bits.
                fake_hash &= (1 << hash_bits) - 1;

                GapHash::<Precision11, Bits4>::initialize_rank_index(&mut hashes, hash_bits);

                let expected_bucket = GapHash::<Precision11, Bits4>::rank_index_capacity()
                    - fake_hash / expected_bucket_size
                    - 1;

                // We try to get the best starting point for the iterator from the rank index
                // Since the index is empty, it should return as position 'hash_bits', as if
                // returning the position associated to the first hash in the hash list.
                // While this cannot happen in practice since the index is created upon preliminary
                // saturation of the hash list, in our case we do not have any first hash in the
                // hash list and as such we expect the rank index to return zero.
                let (bit_index, bucket_hash) =
                    GapHash::<Precision11, Bits4>::best_search_start(&hashes, hash_bits, fake_hash);

                assert_eq!(
                    bit_index, 0,
                    "Found unexpected bit index for hash {fake_hash}."
                );
                assert_eq!(bucket_hash, 0, "Found unexpected bucket hash.");

                let determined_bucket =
                    GapHash::<Precision11, Bits4>::rank_index_hash_bucket(hash_bits, fake_hash);

                assert_eq!(determined_bucket, expected_bucket);

                for bit_index in 0..1024 {
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

                    let hash_bucket =
                        GapHash::<Precision11, Bits4>::rank_index_hash_bucket(hash_bits, fake_hash);

                    if hash_bucket != 0 {
                        assert_eq!(
                            best_bit_index,
                            bit_index,
                            "Found unexpected bit index. Total hashes bit size {}, index bit size {}. The index mask is {}.",
                            hashes.len() * 8,
                            GapHash::<Precision11, Bits4>::rank_index_total_size(hash_bits),
                            GapHash::<Precision11, Bits4>::rank_index_mask()
                        );
                        assert_eq!(
                            best_bucket_hash, fake_hash,
                            "Unable to read the fake hash at index {bit_index}."
                        );
                    }
                }
            }
        }
    }

    #[test_precisions_and_bits]
    /// Test that the apply_gap function works as expected.
    fn test_apply_gap<P: Precision, B: Bits>()
    where
        P: PackedRegister<B>,
    {
        const NUMBER_OF_HASHES: usize = 1_000;
        let mut first_random_state = 4_575_763_274_578_236u64;
        let mut second_random_state = 564_655_678_565_685_654u64;
        let mut third_random_state = 324_587_447_578_236u64;

        // We start from the maximal number of bits for the hash.
        for hash_bits in
            GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS..=GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS
        {
            first_random_state = splitmix64(first_random_state);
            second_random_state = splitmix64(second_random_state);
            third_random_state = splitmix64(third_random_state);
            for ((first, second), third) in
                iter_random_values::<u64>(NUMBER_OF_HASHES as u64, None, Some(first_random_state))
                    .zip(iter_random_values::<u64>(
                        NUMBER_OF_HASHES as u64,
                        None,
                        Some(second_random_state),
                    ))
                    .zip(iter_random_values::<u64>(
                        NUMBER_OF_HASHES as u64,
                        None,
                        Some(third_random_state),
                    ))
            {
                let (first_index, first_register, first_original_hash) =
                    <HyperLogLog<
                        P,
                        B,
                        <P as PackedRegister<B>>::Array,
                    >>::index_and_register_and_hash(&first);

                let first_encoded_hash = GapHash::<P, B>::encode(
                    first_index,
                    first_register,
                    first_original_hash,
                    hash_bits,
                );

                let (second_index, second_register, second_original_hash) =
                    <HyperLogLog<
                        P,
                        B,
                        <P as PackedRegister<B>>::Array,
                    >>::index_and_register_and_hash(&second);

                let second_encoded_hash = GapHash::<P, B>::encode(
                    second_index,
                    second_register,
                    second_original_hash,
                    hash_bits,
                );

                let (third_index, third_register, third_original_hash) =
                    <HyperLogLog<
                        P,
                        B,
                        <P as PackedRegister<B>>::Array,
                    >>::index_and_register_and_hash(&third);

                let third_encoded_hash = GapHash::<P, B>::encode(
                    third_index,
                    third_register,
                    third_original_hash,
                    hash_bits,
                );

                let largest = core::cmp::max(
                    first_encoded_hash,
                    core::cmp::max(second_encoded_hash, third_encoded_hash),
                );
                let smallest = core::cmp::min(
                    first_encoded_hash,
                    core::cmp::min(second_encoded_hash, third_encoded_hash),
                );
                let medium = if first_encoded_hash != largest && first_encoded_hash != smallest {
                    first_encoded_hash
                } else if second_encoded_hash != largest && second_encoded_hash != smallest {
                    second_encoded_hash
                } else {
                    third_encoded_hash
                };

                if largest == smallest || largest == medium || medium == smallest {
                    continue;
                }

                // We now compute the gap between the largest and the smallest encoded hashes.
                let largest_to_smallest_gap =
                    GapHash::<P, B>::into_gap_fragment(largest, smallest, hash_bits);

                // Next, we try to apply the same gap to the medium hash, with the expectation
                // that it should be equal to computing directly the gap with the largest hash.
                let largest_to_medium_deduced_gap = GapHash::<P, B>::apply_gap(
                    smallest,
                    largest_to_smallest_gap,
                    medium,
                    hash_bits,
                );
                let largest_to_medium_direct_gap =
                    GapHash::<P, B>::into_gap_fragment(largest, medium, hash_bits);

                assert_eq!(
                    largest_to_medium_deduced_gap, largest_to_medium_direct_gap,
                    "Failed to apply gap at hash size {hash_bits}."
                );
            }
        }
    }
}
