//! Exact-values list: a sorted (descending) delta-coded list of literal `u64` values stored in the
//! same byte buffer as the hash list, for the exact-values representation mode.
//!
//! This reuses the generic bit-level primitives ([`BitWriter`], [`BitReader`]) and only adds a thin
//! Elias-gamma-style code over full `u64` gaps. The composite-hash index/register/remainder
//! decomposition does not apply to arbitrary user values, so the codec here is deliberately simple:
//! the first (largest) value is stored absolutely and each subsequent value as the positive gap to
//! its predecessor. Because the exact mode is the smallest-cardinality regime, each insert decodes
//! the list, splices the new value in sorted order, and re-encodes; this avoids in-place bitstream
//! surgery at the cost of an O(n) rewrite per insert (n is small here).

use super::bitreader::BitReader;
use super::bitwriter::BitWriter;
use core::mem::size_of;

/// Number of bits one value occupies under [`write_value`]: a unary length prefix (`nbits + 1`
/// bits) followed by the value's `nbits` significant bits.
#[inline]
fn value_bit_len(value: u64) -> u32 {
    let nbits = 64 - value.leading_zeros();
    2 * nbits + 1
}

/// Total encoded bit length of a sorted-ascending value list (encoded descending: first value
/// absolute, then positive gaps).
#[inline]
fn total_bit_len(values_asc: &[u64]) -> u32 {
    let mut iter = values_asc.iter().rev();
    let Some(&first) = iter.next() else {
        return 0;
    };
    let mut bits = value_bit_len(first);
    let mut prev = first;
    for &value in iter {
        bits += value_bit_len(prev - value);
        prev = value;
    }
    bits
}

/// Writes one value as `unary(nbits)` followed by its `nbits` significant bits, where `nbits` is the
/// number of significant bits (`0` for the value `0`). `nbits <= 64`, so the unary run is bounded.
#[inline]
fn write_value(writer: &mut BitWriter, value: u64) {
    let nbits = (64 - value.leading_zeros()) as u8;
    writer.write_unary(nbits);
    writer.write_bits(value, nbits);
}

/// Inverse of [`write_value`].
#[inline]
fn read_value(reader: &mut BitReader) -> u64 {
    let nbits = reader.read_unary();
    reader.read_bits(nbits)
}

/// Reinterprets the byte buffer as `u64` words for the writer (matches the hash-list convention).
#[allow(unsafe_code)]
#[inline]
fn as_words_mut(buffer: &mut [u8]) -> &mut [u64] {
    unsafe {
        core::slice::from_raw_parts_mut(
            buffer.as_mut_ptr().cast::<u64>(),
            buffer.len() / size_of::<u64>(),
        )
    }
}

/// Reinterprets the byte buffer as `u32` words for the reader (matches the hash-list convention).
#[allow(unsafe_code)]
#[inline]
fn as_half_words(buffer: &[u8]) -> &[u32] {
    unsafe {
        core::slice::from_raw_parts(
            buffer.as_ptr().cast::<u32>(),
            buffer.len() / size_of::<u32>(),
        )
    }
}

/// Encodes a sorted-ascending value list into the buffer (stored descending). Returns the bit
/// length written, or `None` if the list does not fit the buffer.
#[must_use]
fn encode(buffer: &mut [u8], values_asc: &[u64]) -> Option<u32> {
    let total_bits = total_bit_len(values_asc);
    if total_bits as usize > buffer.len() * 8 {
        return None;
    }
    let words = as_words_mut(buffer);
    let mut iter = values_asc.iter().rev();
    let mut writer = BitWriter::new(words);
    if let Some(&first) = iter.next() {
        write_value(&mut writer, first);
        let mut prev = first;
        for &value in iter {
            write_value(&mut writer, prev - value);
            prev = value;
        }
    }
    let tell = writer.tell();
    drop(writer);
    Some(tell)
}

/// Decodes the stored values into a sorted-ascending vector.
#[must_use]
pub(crate) fn decode_values(buffer: &[u8], count: u32) -> Vec<u64> {
    let mut values = Vec::with_capacity(count as usize);
    if count == 0 {
        return values;
    }
    let mut reader = BitReader::new(as_half_words(buffer));
    let mut prev = read_value(&mut reader);
    values.push(prev);
    for _ in 1..count {
        prev -= read_value(&mut reader);
        values.push(prev);
    }
    // Stored descending; reverse to ascending.
    values.reverse();
    values
}

/// Returns whether `value` is stored, without allocating. The stream is descending, so the scan
/// stops as soon as it passes `value`.
#[must_use]
pub(crate) fn contains_value(buffer: &[u8], count: u32, value: u64) -> bool {
    if count == 0 {
        return false;
    }
    let mut reader = BitReader::new(as_half_words(buffer));
    let mut prev = read_value(&mut reader);
    if prev == value {
        return true;
    }
    for _ in 1..count {
        prev -= read_value(&mut reader);
        if prev == value {
            return true;
        }
        if prev < value {
            return false;
        }
    }
    false
}

/// Result of inserting a value into the exact list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueInsertion {
    /// The value was inserted; the new count is `count + 1`.
    Inserted,
    /// The value was already present; the list is unchanged.
    Duplicate,
    /// The value does not fit in the current buffer; the caller must grow or transition.
    DoesNotFit,
}

/// Inserts `value` into the sorted exact list stored in `buffer` holding `count` values. The buffer
/// is rewritten in place when the value is inserted.
#[must_use]
pub(crate) fn insert_value(buffer: &mut [u8], count: u32, value: u64) -> ValueInsertion {
    let mut values = decode_values(buffer, count);
    match values.binary_search(&value) {
        Ok(_) => ValueInsertion::Duplicate,
        Err(position) => {
            values.insert(position, value);
            if encode(buffer, &values).is_some() {
                ValueInsertion::Inserted
            } else {
                ValueInsertion::DoesNotFit
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::iter_random_values;

    /// Encodes an ascending value list into a buffer of `words` u64 words and checks the round-trip
    /// and the predicted bit length.
    fn check_roundtrip(values_asc: &[u64], words: usize) {
        let mut buffer = vec![0u8; words * 8];
        let tell = encode(&mut buffer, values_asc).expect("the value list should fit");
        assert_eq!(
            tell,
            total_bit_len(values_asc),
            "predicted bit length mismatch"
        );
        let decoded = decode_values(&buffer, values_asc.len() as u32);
        assert_eq!(decoded, values_asc, "round-trip mismatch");
        for &value in values_asc {
            assert!(contains_value(&buffer, values_asc.len() as u32, value));
        }
        // A value not in the set (one above the maximum) must not be reported as present.
        if let Some(&max) = values_asc.last() {
            if max < u64::MAX {
                assert!(!contains_value(&buffer, values_asc.len() as u32, max + 1));
            }
        }
    }

    #[test]
    fn test_roundtrip_edge_cases() {
        check_roundtrip(&[], 1);
        check_roundtrip(&[0], 1);
        check_roundtrip(&[u64::MAX], 4);
        check_roundtrip(&[0, u64::MAX], 8);
        check_roundtrip(&[0, 1, 2, 3], 2);
        check_roundtrip(&[10, 11, 12, 1_000_000], 4);
    }

    #[test]
    fn test_roundtrip_random() {
        for seed in 0..32u64 {
            let mut values: Vec<u64> =
                iter_random_values::<u64>(200, Some(1 << 40), Some(seed)).collect();
            values.sort_unstable();
            values.dedup();
            check_roundtrip(&values, 256);
        }
    }

    #[test]
    fn test_insert_dedup_and_order() {
        let mut buffer = vec![0u8; 64 * 8];
        let mut count = 0u32;
        let inserts = [50u64, 10, 30, 10, 20, 50, 40];
        let mut expected: Vec<u64> = Vec::new();
        for value in inserts {
            let outcome = insert_value(&mut buffer, count, value);
            if expected.contains(&value) {
                assert_eq!(outcome, ValueInsertion::Duplicate);
            } else {
                assert_eq!(outcome, ValueInsertion::Inserted);
                count += 1;
                expected.push(value);
            }
        }
        expected.sort_unstable();
        assert_eq!(decode_values(&buffer, count), expected);
    }

    #[test]
    fn test_saturation_reported_not_panic() {
        // A one-word buffer cannot hold many wide values; insertion must report DoesNotFit.
        let mut buffer = vec![0u8; 8];
        let mut count = 0u32;
        let mut saturated = false;
        for value in iter_random_values::<u64>(64, None, Some(7)) {
            match insert_value(&mut buffer, count, value) {
                ValueInsertion::Inserted => count += 1,
                ValueInsertion::Duplicate => {}
                ValueInsertion::DoesNotFit => {
                    saturated = true;
                    break;
                }
            }
        }
        assert!(saturated, "the tiny buffer must saturate");
    }
}
