//! Exact-values list: a sorted (descending) delta-coded list of literal `u64` values stored in the
//! same byte buffer as the hash list, for the exact-values representation mode.
//!
//! The codec is deliberately simple and fully allocation-free: each value is written as an
//! Elias-gamma-style code (a unary length prefix followed by its significant bits), the first
//! (largest) value absolutely and each subsequent value as the positive gap to its predecessor.
//! Reading is a lazy iterator ([`ValueIter`]); insertion splices the new value into the bitstream in
//! place (shifting the tail), so neither path allocates. The composite-hash index/register/remainder
//! decomposition does not apply to arbitrary user values, so this does not reuse the hash-list
//! `GapHash` codec, only the same big-endian, MSB-first bit layout.

/// Number of significant bits of `value` (`0` for the value `0`).
#[inline]
fn significant_bits(value: u64) -> u32 {
    64 - value.leading_zeros()
}

/// Encoded bit length of one value: a unary length prefix (`nbits + 1` bits) and `nbits` value bits.
#[inline]
fn code_len(value: u64) -> u32 {
    2 * significant_bits(value) + 1
}

/// Reads the bit at index `bit` from the buffer, interpreted as big-endian `u64` words with the most
/// significant bit first (the layout used by the hash-list bitstream).
#[inline]
fn get_bit(buffer: &[u8], bit: u32) -> u64 {
    let word = (bit / 64) as usize * 8;
    let offset = bit % 64;
    let value = u64::from_be_bytes(buffer[word..word + 8].try_into().unwrap());
    (value >> (63 - offset)) & 1
}

/// Sets the bit at index `bit` in the buffer to `bit_value` (the low bit of `bit_value`).
#[inline]
fn set_bit(buffer: &mut [u8], bit: u32, bit_value: u64) {
    let word = (bit / 64) as usize * 8;
    let offset = bit % 64;
    let mut value = u64::from_be_bytes(buffer[word..word + 8].try_into().unwrap());
    let mask = 1u64 << (63 - offset);
    if bit_value & 1 == 1 {
        value |= mask;
    } else {
        value &= !mask;
    }
    buffer[word..word + 8].copy_from_slice(&value.to_be_bytes());
}

/// Writes one value's code starting at bit position `pos`, returning the position past it.
#[inline]
fn write_value_at(buffer: &mut [u8], mut pos: u32, value: u64) -> u32 {
    let nbits = significant_bits(value);
    for _ in 0..nbits {
        set_bit(buffer, pos, 0);
        pos += 1;
    }
    set_bit(buffer, pos, 1);
    pos += 1;
    for shift in (0..nbits).rev() {
        set_bit(buffer, pos, value >> shift);
        pos += 1;
    }
    pos
}

/// Reads one value's code starting at bit position `pos`, returning the decoded value (an absolute
/// value or a gap, depending on position) and the position past it.
#[inline]
fn read_value_at(buffer: &[u8], mut pos: u32) -> (u64, u32) {
    let mut nbits = 0u32;
    while get_bit(buffer, pos) == 0 {
        nbits += 1;
        pos += 1;
    }
    pos += 1;
    let mut value = 0u64;
    for _ in 0..nbits {
        value = (value << 1) | get_bit(buffer, pos);
        pos += 1;
    }
    (value, pos)
}

/// Lazy iterator over the stored values, yielded in descending order, without allocating.
pub(crate) struct ValueIter<'a> {
    buffer: &'a [u8],
    pos: u32,
    remaining: u32,
    previous: u64,
    first: bool,
}

impl<'a> ValueIter<'a> {
    #[inline]
    pub(crate) fn new(buffer: &'a [u8], count: u32) -> Self {
        Self {
            buffer,
            pos: 0,
            remaining: count,
            previous: 0,
            first: true,
        }
    }
}

impl Iterator for ValueIter<'_> {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<u64> {
        if self.remaining == 0 {
            return None;
        }
        let (code, next_pos) = read_value_at(self.buffer, self.pos);
        self.pos = next_pos;
        // The first code is the absolute largest value; the rest are positive gaps.
        self.previous = if self.first {
            self.first = false;
            code
        } else {
            self.previous - code
        };
        self.remaining -= 1;
        Some(self.previous)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining as usize, Some(self.remaining as usize))
    }
}

impl ExactSizeIterator for ValueIter<'_> {}

/// Returns whether `value` is stored, without allocating, stopping early once the descending scan
/// passes `value`.
#[must_use]
pub(crate) fn contains_value(buffer: &[u8], count: u32, value: u64) -> bool {
    let mut pos = 0u32;
    let mut previous = 0u64;
    for index in 0..count {
        let (code, next_pos) = read_value_at(buffer, pos);
        pos = next_pos;
        previous = if index == 0 { code } else { previous - code };
        if previous == value {
            return true;
        }
        if previous < value {
            return false;
        }
    }
    false
}

/// Returns the exact number of distinct values in the union of two exact lists, by a two-pointer
/// merge over the descending streams. Allocation-free.
#[must_use]
pub(crate) fn union_count(buffer_a: &[u8], count_a: u32, buffer_b: &[u8], count_b: u32) -> u32 {
    let mut a = ValueIter::new(buffer_a, count_a).peekable();
    let mut b = ValueIter::new(buffer_b, count_b).peekable();
    let mut count = 0u32;
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (Some(x), Some(y)) => {
                count += 1;
                // Descending streams: advance whichever is larger, both on a tie.
                if x == y {
                    a.next();
                    b.next();
                } else if x > y {
                    a.next();
                } else {
                    b.next();
                }
            }
            (Some(_), None) => {
                a.next();
                count += 1;
            }
            (None, Some(_)) => {
                b.next();
                count += 1;
            }
            (None, None) => return count,
        }
    }
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

/// Shifts the bit range `[from, end)` to higher indices by `delta` bits, in place, copying backward
/// so the moved bits never overwrite bits not yet read. Splicing a value only ever grows the stream
/// (one gamma code never splits into two shorter ones), so the shift is always towards higher
/// indices.
#[inline]
fn shift_bits_right(buffer: &mut [u8], from: u32, end: u32, delta: u32) {
    for bit in (from..end).rev() {
        let value = get_bit(buffer, bit);
        set_bit(buffer, bit + delta, value);
    }
}

/// Inserts `value` into the sorted (descending) exact list of `count` values stored in `buffer`,
/// splicing it in place. Allocation-free.
#[must_use]
pub(crate) fn insert_value(buffer: &mut [u8], count: u32, value: u64) -> ValueInsertion {
    let capacity_bits = (buffer.len() * 8) as u32;

    if count == 0 {
        if code_len(value) > capacity_bits {
            return ValueInsertion::DoesNotFit;
        }
        write_value_at(buffer, 0, value);
        return ValueInsertion::Inserted;
    }

    // Single descending pass: locate the insertion point, detect duplicates, and find the total
    // length. `before` is the predecessor (the smallest stored value still greater than `value`).
    let mut pos = 0u32;
    let mut previous = 0u64;
    let mut before: Option<u64> = None;
    let mut next: Option<(u64, u32, u32)> = None; // (value, code_start, code_end)
    for index in 0..count {
        let start = pos;
        let (code, end) = read_value_at(buffer, pos);
        let current = if index == 0 { code } else { previous - code };
        pos = end;
        previous = current;
        if next.is_none() {
            if current == value {
                return ValueInsertion::Duplicate;
            }
            if current < value {
                next = Some((current, start, end));
            } else {
                before = Some(current);
            }
        }
    }
    let total_bits = pos;

    match next {
        // `value` is the new minimum: append its gap to the smallest stored value at the end.
        None => {
            let gap = before.expect("a non-empty list has a predecessor") - value;
            if total_bits + code_len(gap) > capacity_bits {
                return ValueInsertion::DoesNotFit;
            }
            write_value_at(buffer, total_bits, gap);
            ValueInsertion::Inserted
        }
        // `value` is spliced before `next`: write its code, then rewrite `next` relative to `value`.
        Some((next_value, split_bit, next_end)) => {
            let code_a = match before {
                None => value,              // new maximum: stored absolutely
                Some(prev) => prev - value, // gap from the predecessor
            };
            let new_next_gap = value - next_value;
            let old_next_len = next_end - split_bit;
            let new_len = code_len(code_a) + code_len(new_next_gap);
            // Replacing `next`'s code with `code_a` followed by `next`'s shorter gap always grows the
            // stream, so `new_len > old_next_len`.
            debug_assert!(new_len > old_next_len);
            let delta = new_len - old_next_len;
            if total_bits + delta > capacity_bits {
                return ValueInsertion::DoesNotFit;
            }
            shift_bits_right(buffer, next_end, total_bits, delta);
            let after_a = write_value_at(buffer, split_bit, code_a);
            write_value_at(buffer, after_a, new_next_gap);
            ValueInsertion::Inserted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::iter_random_values;

    /// Inserts an ascending value set and checks lazy recovery (descending), membership, and the
    /// reported insert outcomes.
    fn check_inserts(values: &[u64], words: usize) {
        let mut buffer = vec![0u8; words * 8];
        let mut count = 0u32;
        let mut expected: Vec<u64> = Vec::new();
        for &value in values {
            let outcome = insert_value(&mut buffer, count, value);
            if expected.contains(&value) {
                assert_eq!(outcome, ValueInsertion::Duplicate, "value {value}");
            } else {
                assert_eq!(outcome, ValueInsertion::Inserted, "value {value}");
                expected.push(value);
                count += 1;
            }
        }
        expected.sort_unstable();

        let mut recovered: Vec<u64> = ValueIter::new(&buffer, count).collect();
        assert_eq!(recovered.len(), count as usize);
        // ValueIter yields descending.
        let mut descending = expected.clone();
        descending.reverse();
        assert_eq!(recovered, descending, "recovery mismatch");

        recovered.sort_unstable();
        assert_eq!(recovered, expected);

        for &value in &expected {
            assert!(contains_value(&buffer, count, value), "missing {value}");
        }
    }

    #[test]
    fn test_insert_edge_cases() {
        check_inserts(&[], 1);
        check_inserts(&[0], 1);
        check_inserts(&[u64::MAX], 4);
        check_inserts(&[0, u64::MAX], 8);
        check_inserts(&[u64::MAX, 0], 8);
        check_inserts(&[3, 2, 1, 0], 2);
        check_inserts(&[0, 1, 2, 3], 2);
        check_inserts(&[5, 1, 9, 1, 3], 4);
        check_inserts(&[1_000_000, 12, 11, 10], 4);
    }

    #[test]
    fn test_insert_random_order() {
        for seed in 0..32u64 {
            let values: Vec<u64> =
                iter_random_values::<u64>(150, Some(1 << 40), Some(seed)).collect();
            check_inserts(&values, 512);
        }
    }

    #[test]
    fn test_contains_absent() {
        let mut buffer = vec![0u8; 8 * 8];
        let mut count = 0u32;
        for value in [10u64, 20, 30, 40] {
            if insert_value(&mut buffer, count, value) == ValueInsertion::Inserted {
                count += 1;
            }
        }
        assert!(!contains_value(&buffer, count, 25));
        assert!(!contains_value(&buffer, count, 5));
        assert!(!contains_value(&buffer, count, 50));
    }

    #[test]
    fn test_saturation_reported_not_panic() {
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

    #[test]
    fn test_append_saturation_reported() {
        // Strictly descending inserts make every value a new minimum, appended at the end, so this
        // exercises the append branch's saturation path.
        let mut buffer = vec![0u8; 8];
        let mut count = 0u32;
        let mut saturated = false;
        for value in (0..64u64).rev().map(|k| k * 1000) {
            match insert_value(&mut buffer, count, value) {
                ValueInsertion::Inserted => count += 1,
                ValueInsertion::Duplicate => {}
                ValueInsertion::DoesNotFit => {
                    saturated = true;
                    break;
                }
            }
        }
        assert!(saturated, "appending past the tiny buffer must saturate");
    }

    #[test]
    fn test_union_count_symmetric() {
        let mut a = vec![0u8; 8 * 8];
        let mut b = vec![0u8; 8 * 8];
        let mut count_a = 0u32;
        let mut count_b = 0u32;
        for value in [10u64, 20, 30, 40] {
            if insert_value(&mut a, count_a, value) == ValueInsertion::Inserted {
                count_a += 1;
            }
        }
        for value in [25u64, 35, 40, 50, 60] {
            if insert_value(&mut b, count_b, value) == ValueInsertion::Inserted {
                count_b += 1;
            }
        }
        // {10,20,30,40} union {25,35,40,50,60} = {10,20,25,30,35,40,50,60}, 8 distinct.
        assert_eq!(union_count(&a, count_a, &b, count_b), 8);
        // The swapped order exercises the mirrored merge branches.
        assert_eq!(union_count(&b, count_b, &a, count_a), 8);
    }
}
