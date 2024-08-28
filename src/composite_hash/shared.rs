//! Utilities used across the composite hash submodule.
use super::{CompositeHash, CompositeHashError};
use core::slice::Iter;

/// Iterator variants.
#[allow(non_camel_case_types)]
pub(super) enum IterVariants<'a> {
    /// Variant for 8-bit hashes.
    u8(Iter<'a, u8>),
    /// Variant for 16-bit hashes.
    u16(Iter<'a, u16>),
    /// Variant for 24-bit hashes.
    u24(Iter<'a, [u8; 3]>),
    /// Variant for 32-bit hashes.
    u32(Iter<'a, u32>),
}

impl<'a> From<Iter<'a, u8>> for IterVariants<'a> {
    fn from(iter: Iter<'a, u8>) -> Self {
        IterVariants::u8(iter)
    }
}

impl<'a> From<Iter<'a, u16>> for IterVariants<'a> {
    fn from(iter: Iter<'a, u16>) -> Self {
        IterVariants::u16(iter)
    }
}

impl<'a> From<Iter<'a, [u8; 3]>> for IterVariants<'a> {
    fn from(iter: Iter<'a, [u8; 3]>) -> Self {
        IterVariants::u24(iter)
    }
}

impl<'a> From<Iter<'a, u32>> for IterVariants<'a> {
    fn from(iter: Iter<'a, u32>) -> Self {
        IterVariants::u32(iter)
    }
}

impl<'a> IterVariants<'a> {
    const fn hash_bits(&self) -> u8 {
        match self {
            IterVariants::u8(_) => 8,
            IterVariants::u16(_) => 16,
            IterVariants::u24(_) => 24,
            IterVariants::u32(_) => 32,
        }
    }
}

impl Iterator for IterVariants<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterVariants::u8(iter) => iter.next().copied().map(u64::from),
            IterVariants::u16(iter) => iter.next().copied().map(u64::from),
            IterVariants::u24(iter) => iter.next().map(|hash| {
                let mut bytes = [0; 8];
                bytes[..3].copy_from_slice(hash);
                u64::from_le_bytes(bytes)
            }),
            IterVariants::u32(iter) => iter.next().copied().map(u64::from),
        }
    }
}

/// An iterator over the decoded hashes.
pub struct DecodedIter<'a, CH: CompositeHash> {
    /// The iterator variant.
    variant: IterVariants<'a>,
    /// The composite hash type.
    _phantom: core::marker::PhantomData<CH>,
}

impl<'a, CH: CompositeHash> Iterator for DecodedIter<'a, CH> {
    type Item = (u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.variant.next().map(|hash| {
            let (register, index) = CH::decode(hash, self.variant.hash_bits());
            (register, index)
        })
    }
}

impl<'a, CH: CompositeHash> From<IterVariants<'a>> for DecodedIter<'a, CH> {
    fn from(variant: IterVariants<'a>) -> Self {
        DecodedIter {
            variant,
            _phantom: core::marker::PhantomData,
        }
    }
}
/// An iterator over the decoded hashes.
pub struct DowngradedIter<'a, CH> {
    /// The iterator variant.
    variant: IterVariants<'a>,
    /// The number of bits to shift.
    shift: u8,
    /// The composite hash type.
    _phantom: core::marker::PhantomData<CH>,
}

impl<'a, CH: CompositeHash> Iterator for DowngradedIter<'a, CH> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.variant
            .next()
            .map(|hash| CH::downgrade(hash, self.variant.hash_bits(), self.shift))
    }
}

impl<'a, CH: CompositeHash> DowngradedIter<'a, CH> {
    /// Create a new iterator from the provided iterator and the number of bits to shift.
    pub(super) fn new(variant: IterVariants<'a>, shift: u8) -> Self {
        DowngradedIter {
            variant,
            shift,
            _phantom: core::marker::PhantomData,
        }
    }
}

#[inline]
#[must_use]
#[allow(unsafe_code)]
pub(super) fn into_variant<'a>(
    hashes: &'a [u8],
    number_of_hashes: usize,
    hash_bits: u8,
) -> IterVariants<'a> {
    assert!(hash_bits >= 8);
    assert!(hash_bits == 8 || hash_bits == 16 || hash_bits == 24 || hash_bits == 32);
    assert!(hashes.len() >= number_of_hashes * usize::from(hash_bits / 8));
    let hashes = &hashes[..number_of_hashes * usize::from(hash_bits / 8)];
    match hash_bits {
        8 => IterVariants::u8(hashes.iter()),
        16 => {
            let hashes: &[u16] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr() as *const u16, hashes.len() / 2)
            };
            IterVariants::u16(hashes.iter())
        }
        24 => {
            let hashes: &[[u8; 3]] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr() as *const [u8; 3], hashes.len() / 3)
            };
            IterVariants::u24(hashes.iter())
        }
        32 => {
            let hashes: &[u32] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr() as *const u32, hashes.len() / 4)
            };
            IterVariants::u32(hashes.iter())
        }
        _ => unreachable!(),
    }
}

#[inline]
#[must_use]
#[allow(unsafe_code)]
pub(super) fn find<'a, CH>(
    hashes: &'a [u8],
    number_of_hashes: usize,
    index: usize,
    register: u8,
    original_hash: u64,
    hash_bits: u8,
) -> Result<usize, (usize, u64)>
where
    CH: CompositeHash,
{
    assert!(hash_bits >= 8);
    assert!(hash_bits == 8 || hash_bits == 16 || hash_bits == 24 || hash_bits == 32);
    assert!(hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS);
    assert!(hashes.len() >= number_of_hashes * usize::from(hash_bits / 8));
    let encoded_hash = CH::encode(index, register, original_hash, hash_bits);

    let hashes: &[u8] = &hashes[..number_of_hashes * usize::from(hash_bits / 8)];

    match hash_bits / 8 {
        1 => {
            let hash = u8::try_from(encoded_hash).unwrap();
            hashes
                // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                .binary_search_by(|a| hash.cmp(a))
                .map(|index| index * usize::from(hash_bits))
                .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
        }
        2 => {
            // We transmute the hash to a u16, as we will be comparing it with other u16s.
            let hashes: &[u16] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr() as *const u16, hashes.len() / 2)
            };
            let hash = u16::try_from(encoded_hash).unwrap();
            debug_assert_eq!(hashes.len(), number_of_hashes);
            hashes
                // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                .binary_search_by(|a| hash.cmp(a))
                .map(|index| index * usize::from(hash_bits))
                .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
        }
        3 => {
            // We transmute the hash to a hash of [u8; 3], as we will be comparing it with other [u8; 3]s.
            // Since these values are stored in little-endian, we need to reverse the bytes into a u32 before
            // comparing them.
            let hashes: &[[u8; 3]] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr() as *const [u8; 3], hashes.len() / 3)
            };
            let hash: u32 = u32::try_from(encoded_hash).unwrap();
            debug_assert_eq!(hashes.len(), number_of_hashes);
            let result = hashes
                // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                .binary_search_by(|a| {
                    let mut four_bytes = [0; 4];
                    four_bytes[..3].copy_from_slice(a);
                    let a = u32::from_le_bytes(four_bytes);
                    hash.cmp(&a)
                })
                .map(|index| index * usize::from(hash_bits))
                .map_err(|index| (index * usize::from(hash_bits), encoded_hash));

            result
        }
        4 => {
            // We transmute the hash to a u32, as we will be comparing it with other u32s.
            let hashes: &[u32] = unsafe {
                core::slice::from_raw_parts(hashes.as_ptr() as *const u32, hashes.len() / 4)
            };
            let hash = u32::try_from(encoded_hash).unwrap();
            debug_assert_eq!(hashes.len(), number_of_hashes);
            hashes
                // The hashes are sorted in a descending order, so we need to find the first hash that is less than the encoded hash.
                .binary_search_by(|a| hash.cmp(a))
                .map(|index| index * usize::from(hash_bits))
                .map_err(|index| (index * usize::from(hash_bits), encoded_hash))
        }
        _ => unreachable!(),
    }
}

#[inline]
#[must_use]
#[allow(unsafe_code)]
pub(super) fn insert_sorted_desc<'a, CH>(
    hashes: &'a mut [u8],
    number_of_hashes: usize,
    bit_index: usize,
    index: usize,
    register: u8,
    original_hash: u64,
    hash_bits: u8,
) -> Result<Option<usize>, CompositeHashError>
where
    CH: CompositeHash,
{
    assert!(hash_bits >= 8);
    assert!(
        hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS,
        "The hash bits ({}) must be greater or equal to the smallest viable hash bits ({})",
        hash_bits,
        CH::SMALLEST_VIABLE_HASH_BITS,
    );
    assert!(hash_bits == 8 || hash_bits == 16 || hash_bits == 24 || hash_bits == 32);
    assert!(bit_index == number_of_hashes * usize::from(hash_bits));
    let hash_bytes = usize::from(hash_bits / 8);

    match CH::find(
        hashes,
        number_of_hashes,
        index,
        register,
        original_hash,
        hash_bits,
    ) {
        Ok(_) => {Ok(None)},
        Err((bits, encoded_hash)) => {

            // We check that there is indeed no hash identical to the provided
            // encoded hash:
            debug_assert!(
                CH::downgraded(hashes, number_of_hashes, hash_bits, 0).all(|hash| hash != encoded_hash),
                "The hash ({encoded_hash}) must not be present in the hashes",
            );

            if bit_index / 8 + hash_bytes > hashes.len() {
                if hash_bits == CH::SMALLEST_VIABLE_HASH_BITS {
                    return Err(CompositeHashError::Saturation);
                }
                return Err(CompositeHashError::DowngradableSaturation);
            }

            let index = bits / usize::from(hash_bits);
            assert!(
                hashes.len() >= (number_of_hashes + 1) * hash_bytes,
                "The slice len ({}) must be greater or equal to the product of the slice size ({hash_bytes}) and the number of elements ({number_of_hashes} + 1)",
                hashes.len(),
            );
            debug_assert!(
                index <= number_of_hashes,
                "The index ({index}) must be less or equal to the number of hashes ({number_of_hashes}) with the hash bits ({hash_bits})",
            );

            match hash_bytes {
                1 => {
                    hashes.copy_within(index..number_of_hashes, index + 1);
                    hashes[index] = u8::try_from(encoded_hash).unwrap();
                }
                2 => {
                    let hashes: &mut [u16] = unsafe {
                        core::slice::from_raw_parts_mut(
                            hashes.as_mut_ptr() as *mut u16,
                            hashes.len() / 2,
                        )
                    };
                    hashes.copy_within(index..number_of_hashes, index + 1);
                    hashes[index] = u16::try_from(encoded_hash).unwrap();
                }
                3 => {
                    let hashes: &mut [[u8; 3]] = unsafe {
                        core::slice::from_raw_parts_mut(
                            hashes.as_mut_ptr() as *mut [u8; 3],
                            hashes.len() / 3,
                        )
                    };
                    hashes.copy_within(index..number_of_hashes, index + 1);
                    let mut three_bytes = [0; 3];
                    three_bytes.copy_from_slice(&encoded_hash.to_le_bytes()[..3]);
                    hashes[index] = three_bytes;
                }
                4 => {
                    let hashes: &mut [u32] = unsafe {
                        core::slice::from_raw_parts_mut(
                            hashes.as_mut_ptr() as *mut u32,
                            hashes.len() / 4,
                        )
                    };
                    hashes.copy_within(index..number_of_hashes, index + 1);
                    hashes[index] = u32::try_from(encoded_hash).unwrap();
                }
                _ => unreachable!(),
            }
            Ok(Some((number_of_hashes + 1) * usize::from(hash_bits)))
        }
    }
}
