//! Broadword (SWAR) register-wise maximum for the dense-dense `HyperLogLog` merge.
//!
//! Port of Sebastiano Vigna's `merge_hyperloglog_bitwise` from `card-est-array`, specialised to
//! `u64` words and stripped of every allocation. The algorithm computes the register-wise max
//! of two densely packed `[u64]` backends in `O(num_words)` word-level operations rather than
//! the naive `O(num_regs)` per-register extract-max-reinsert loop. For `Precision10 Bits5` (80
//! words, 1024 registers) that is roughly 1700 word-level operations per merge instead of the
//! 7000-ish the per-register path runs.
//!
//! The algorithm needs two precomputed masks tied to the packing shape: `msb_mask` has bit
//! `i * R + R - 1` set for `i` in `0..num_regs` (the msb of each register), and `lsb_mask` has
//! bit `i * R` set for the same `i` (the lsb of each register). It also needs two scratch word
//! buffers of the same length as the operand slabs. Both scratch buffers and the masks are
//! passed in by the caller, so the callable itself has no heap dependency and no static state
//! and the caller decides whether the storage lives on the stack, in a helper struct, or in a
//! thread-local.
#![allow(unsafe_code)]

/// Multi-word big-integer subtract, in place: `x -= y`. Iterates through the words
/// **high-index to low-index**, matching sketching-core's MSB-first layout in which
/// `word[0]` holds the most-significant bits and `word[n-1]` holds the least-significant
/// ones. The borrow propagates from the LSB word (`n - 1`) toward the MSB word (`0`).
/// Wrapping arithmetic; the caller guarantees non-negativity (the top word of `x` sits above
/// the top word of `y` in bit weight).
#[inline]
fn subtract(x: &mut [u64], y: &[u64]) {
    debug_assert_eq!(x.len(), y.len());
    let mut borrow = false;
    for (x_word, &y_word) in x.iter_mut().zip(y.iter()).rev() {
        let mut xv = *x_word;
        if !borrow {
            borrow = xv < y_word;
        } else if xv != 0 {
            xv = xv.wrapping_sub(1);
            borrow = xv < y_word;
        } else {
            xv = xv.wrapping_sub(1);
        }
        *x_word = xv.wrapping_sub(y_word);
    }
}

/// Register-wise maximum of `dst` and `src`, storing the result in `dst`.
///
/// Both slabs interpret their bit stream as `num_words * 64 / register_size` packed unsigned
/// integers of `register_size` bits each (`register_size` in `1..64`). The masks `msb_mask`
/// and `lsb_mask` are the "highest bit of each register" and "lowest bit of each register"
/// masks respectively, matching the packing shape (see [`build_msb_mask`] and
/// [`build_lsb_mask`]).
///
/// The scratch buffers `acc` and `mask_scratch` must have the same length as `dst`. Their
/// contents on entry are ignored; their contents on return are unspecified. Callers that
/// merge many pairs SHOULD reuse the same two buffers across calls: fresh per-call storage
/// (heap, stack, or thread-local) is fine, but the algorithm itself allocates nothing.
///
/// # Panics
///
/// Debug-only length mismatch checks on `dst`, `src`, `msb_mask`, `lsb_mask`, `acc`, and
/// `mask_scratch`.
pub fn merge_max_broadword(
    dst: &mut [u64],
    src: &[u64],
    msb_mask: &[u64],
    lsb_mask: &[u64],
    register_size: usize,
    acc: &mut [u64],
    mask_scratch: &mut [u64],
) {
    debug_assert_eq!(dst.len(), src.len());
    debug_assert_eq!(dst.len(), msb_mask.len());
    debug_assert_eq!(dst.len(), lsb_mask.len());
    debug_assert_eq!(dst.len(), acc.len());
    debug_assert_eq!(dst.len(), mask_scratch.len());
    debug_assert!((1..=64).contains(&register_size));

    let n = dst.len();
    if n == 0 {
        return;
    }
    let register_size_minus_1 = register_size - 1;
    let shift_top = u64::BITS as usize - register_size_minus_1;

    // Phase 1: register-wise strict comparison `src > dst` computed via
    // `z = ((((y | H_r) - (x & !H_r)) | (y ^ x)) ^ (y | !x)) & H_r`,
    // with `x = dst`, `y = src`, `H_r = msb_mask`. The msb bit of each register in `z` ends up
    // set iff `y > x` for that register. Sign of the multi-word subtract is guaranteed because
    // `y | H_r` sits strictly above `x & !H_r` in bit weight at every register position.
    for i in 0..n {
        acc[i] = src[i] | msb_mask[i];
    }
    for i in 0..n {
        mask_scratch[i] = dst[i] & !msb_mask[i];
    }
    subtract(acc, mask_scratch);
    for i in 0..n {
        let dv = dst[i];
        let sv = src[i];
        let mv = msb_mask[i];
        acc[i] = ((acc[i] | (sv ^ dv)) ^ (sv | !dv)) & mv;
    }

    // Phase 2: turn the per-register msb bit in `acc` into a full per-register bitmask by
    // shifting each msb bit down (toward the register's lsb) by `register_size - 1` bit-stream
    // places, then broadcasting via
    //   `full_mask = (((acc >> (r - 1)) | H_r) - L_r) | H_r) ^ acc`.
    // In sketching-core's MSB-first storage, "shift bit-stream right by k" translates to
    // `word[k+1] |= (word[k] & low_bits_mask) << (65 - r)` at the word boundary. So
    // `mask_scratch[k]` receives contributions from `acc[k]` (its own bits shifted right by
    // `r - 1`) and from `acc[k - 1]` (its low `r - 1` bits shifted left to the top of
    // `mask_scratch[k]`). Word `0` has no `acc[-1]`, so it is special-cased. The subtract
    // must be a multi-word big-integer subtract so a borrow at a straddling register carries
    // across the word boundary in the correct direction.
    mask_scratch[0] = (acc[0] >> register_size_minus_1) | msb_mask[0];
    for k in 1..n {
        let lo = acc[k] >> register_size_minus_1;
        let hi = acc[k - 1] << shift_top;
        mask_scratch[k] = (lo | hi) | msb_mask[k];
    }
    subtract(mask_scratch, lsb_mask);
    for i in 0..n {
        mask_scratch[i] = (mask_scratch[i] | msb_mask[i]) ^ acc[i];
    }

    // Phase 3: select `src` where `mask_scratch` is 1, `dst` elsewhere, storing back into
    // `dst`. Equivalent to `dst ^ (mask & (dst ^ src))`.
    for i in 0..n {
        dst[i] ^= mask_scratch[i] & (dst[i] ^ src[i]);
    }
}

/// Writes the msb mask for `num_regs` registers of `register_size` bits, packed high-to-low
/// in a `num_words * 64`-bit stream, into `out`. Under sketching-core's MSB-first storage,
/// register `i`'s MSB is at LSB-indexed bit `63 - (i * register_size) % 64` of word
/// `(i * register_size) / 64`. `out.len()` must equal `num_words`.
pub fn build_msb_mask(out: &mut [u64], register_size: usize, num_regs: usize) {
    build_register_mask(out, register_size, num_regs, 0);
}

/// Writes the lsb mask for `num_regs` registers of `register_size` bits, packed high-to-low
/// in a `num_words * 64`-bit stream, into `out`. Under sketching-core's MSB-first storage,
/// register `i`'s LSB is at LSB-indexed bit `63 - (i * register_size + register_size - 1) % 64`
/// of word `(i * register_size + register_size - 1) / 64`. `out.len()` must equal `num_words`.
pub fn build_lsb_mask(out: &mut [u64], register_size: usize, num_regs: usize) {
    build_register_mask(out, register_size, num_regs, register_size - 1);
}

/// Location of the bit inside the word slab that corresponds to the given per-register offset
/// in the bit stream. `(word_index, bit_within_word)` uses LSB-indexed bit numbering (bit 0 =
/// least significant bit of the word). Under sketching-core's MSB-first storage, register
/// `i`'s j-th bit (counting from the register's MSB) lives at stream position
/// `i * register_size + j`, and stream position `p` maps to LSB-indexed bit `63 - (p % 64)`
/// of word `p / 64`. Shared between the runtime and compile-time mask builders so they
/// cannot drift.
#[inline]
#[must_use]
const fn register_bit_position(
    i: usize,
    register_size: usize,
    offset_from_reg_msb: usize,
) -> (usize, u32) {
    let stream_pos = i * register_size + offset_from_reg_msb;
    (stream_pos / 64, 63 - (stream_pos as u32 % 64))
}

#[inline]
fn build_register_mask(
    out: &mut [u64],
    register_size: usize,
    num_regs: usize,
    offset_from_reg_msb: usize,
) {
    debug_assert!(offset_from_reg_msb < register_size);
    let num_words = out.len();
    for word in out.iter_mut() {
        *word = 0;
    }
    for i in 0..num_regs {
        let (word, bit_in_word) = register_bit_position(i, register_size, offset_from_reg_msb);
        if word < num_words {
            out[word] |= 1u64 << bit_in_word;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    /// Bit-for-bit reference max: extract each register from both slabs, take the max,
    /// reinsert. Used to cross-check the broadword output against a slow but obviously-correct
    /// baseline. Layout matches sketching-core's `Packed<Words<N>, V>`: register `i` occupies
    /// bit-stream positions `[i * B, (i + 1) * B)` where stream position 0 is the MSB of
    /// `word[0]`, and straddled registers put their high bits in the LSB end of the earlier
    /// word and their low bits in the MSB end of the later word.
    fn reference_max(dst: &mut [u64], src: &[u64], register_size: usize, num_regs: usize) {
        let mask = (1u64 << register_size) - 1;
        let r = register_size as u32;
        for i in 0..num_regs {
            let stream_pos = i * register_size;
            let word = stream_pos / 64;
            let off = (stream_pos % 64) as u32; // offset from the MSB of the word
            let d_val = extract_msb_first(dst, word, off, r, mask);
            let s_val = extract_msb_first(src, word, off, r, mask);
            let m_val = d_val.max(s_val);
            insert_msb_first(dst, word, off, r, m_val, mask);
        }
    }

    /// Straightforward port of sketching-core's `extract_value_from_word` +
    /// `extract_bridge_value_from_word`, kept inline here to avoid pulling the whole
    /// `VariableWord` machinery into the test.
    #[inline]
    fn extract_msb_first(slab: &[u64], word: usize, off: u32, r: u32, mask: u64) -> u64 {
        if off + r <= 64 {
            (slab[word] >> (64 - r - off)) & mask
        } else {
            let low_bits_in_upper = off + r - 64;
            let higher_bits_mask = mask >> low_bits_in_upper;
            let higher_bits = (slab[word] & higher_bits_mask) << low_bits_in_upper;
            let lower_bits = slab[word + 1] >> (64 - low_bits_in_upper);
            higher_bits | lower_bits
        }
    }

    /// Straightforward port of sketching-core's `insert_value_into_word` +
    /// `insert_bridge_value_into_word`. Clears the register bits before writing.
    #[inline]
    fn insert_msb_first(slab: &mut [u64], word: usize, off: u32, r: u32, value: u64, mask: u64) {
        if off + r <= 64 {
            let flipped_offset = 64 - r - off;
            slab[word] = (slab[word] & !(mask << flipped_offset)) | (value << flipped_offset);
        } else {
            let n_lower = off + r - 64;
            let lower_bits_mask = (1u64 << n_lower) - 1;
            let higher_bits_mask = mask >> n_lower;
            let lower_bits = value & lower_bits_mask;
            let higher_bits = value >> n_lower;
            slab[word] = (slab[word] & !higher_bits_mask) | higher_bits;
            let shift = 64 - n_lower;
            slab[word + 1] = (slab[word + 1] & !(lower_bits_mask << shift)) | (lower_bits << shift);
        }
    }

    fn seeded_slab<const N: usize>(register_size: usize, num_regs: usize, seed: u64) -> [u64; N] {
        let register_mask = (1u64 << register_size) - 1;
        let r = register_size as u32;
        let mut slab = [0u64; N];
        let mut state = seed;
        for i in 0..num_regs {
            state = splitmix64(state);
            let v = state & register_mask;
            let stream_pos = i * register_size;
            let word = stream_pos / 64;
            let off = (stream_pos % 64) as u32;
            insert_msb_first(&mut slab, word, off, r, v, register_mask);
        }
        slab
    }

    /// Direct behavioural tests for [`subtract`], the multi-word big-integer subtract used
    /// by both phases of the algorithm. Two equality-boundary cases (`xv == y_word` at the
    /// LSB word, with and without an incoming borrow) are called out explicitly because they
    /// expose the `borrow = xv < y_word` predicate directly. Replacing that `<` with `<=` (a
    /// realistic off-by-one mistake) would over-borrow when a word position happens to hold
    /// identical values, propagating an extra `-1` into the more-significant word. The
    /// through-algorithm tests do not routinely construct that pattern at a subtract
    /// boundary, so a dedicated unit test is the only way to close that gap.
    #[test]
    fn subtract_matches_bigint_semantics() {
        // Layout is MSB-first: `word[0]` is the most-significant word, `word[n-1]` the
        // least-significant. Borrow propagates from the LSB word toward the MSB word.

        // Case 1: simple subtract with no borrow.
        //   value = 5, subtract 3 -> 2. Word[0] untouched.
        let mut x = [0u64, 5u64];
        subtract(&mut x, &[0u64, 3u64]);
        assert_eq!(x, [0, 2], "5 - 3 without borrow");

        // Case 2: equality at the LSB word, no incoming borrow. `xv == y_word` MUST NOT
        // trigger a borrow; the result is exactly zero at that position and word[0] stays
        // unchanged. The `<` mutant misfires here.
        let mut x = [7u64, 5u64];
        subtract(&mut x, &[3u64, 5u64]);
        assert_eq!(
            x,
            [4, 0],
            "equality at LSB word must NOT borrow (word[0]: 7 - 3 = 4)"
        );

        // Case 3: borrow across the word boundary because the LSB value is smaller than
        // subtrahend. word[1] = 3 - 5 wraps to `u64::MAX - 1`; word[0] takes the borrow, 7 - 3 - 1 = 3.
        let mut x = [7u64, 3u64];
        subtract(&mut x, &[3u64, 5u64]);
        assert_eq!(
            x,
            [3, 3u64.wrapping_sub(5)],
            "borrow must propagate from LSB to MSB word (word[0] pays 1)",
        );

        // Case 4: three-word borrow chain crossing two boundaries. Value = (0, 0, 0) - (0, 0, 1)
        // = wraps to (u64::MAX, u64::MAX, u64::MAX) with the borrow chained all the way.
        let mut x = [0u64, 0u64, 0u64];
        subtract(&mut x, &[0u64, 0u64, 1u64]);
        assert_eq!(
            x,
            [u64::MAX, u64::MAX, u64::MAX],
            "three-word borrow chain must propagate to the MSB word",
        );

        // Case 5: equality at LSB word WITH incoming borrow from an even-lower word. Three
        // words `[hi, mid, lo]`. `lo == y_lo` (5 == 5) does NOT itself require a borrow, but
        // if a borrow had been carried into `mid` earlier via some other path, the algorithm
        // MUST still track it correctly. Here we set up `mid < y_mid` so a genuine borrow is
        // needed at `mid`, then confirm the middle word decrements correctly and the top
        // word absorbs the borrow. This exercises the `borrow = xv < y_word` predicate
        // inside the "already borrowing" branch (line 35), which the second `<= mutant`
        // targets.
        let mut x = [10u64, 2u64, 5u64];
        subtract(&mut x, &[3u64, 4u64, 5u64]);
        // Bit math: (10 * 2^128 + 2 * 2^64 + 5) - (3 * 2^128 + 4 * 2^64 + 5) =
        //   (7 * 2^128 - 2 * 2^64) = (6 * 2^128 + (2^64 - 2) * 2^64). Word[2] = 0,
        //   word[1] = -2 as u64 = u64::MAX - 1, word[0] = 10 - 3 - 1 = 6.
        assert_eq!(
            x,
            [6, 2u64.wrapping_sub(4), 0],
            "borrow at mid, no additional borrow at LSB (equal), and MSB pays exactly one",
        );

        // Case 6: incoming borrow AND equality at LSB. word[2] = 5 == y[2] = 5 (equal at
        // LSB, no self-borrow); word[1] = 0 < y[1] = 4 (borrow needed); word[0] = 10 - 3 - 1.
        // Fully isolated from case 5 to hit the specific ordering that the `<= mutant on
        // line 35 flips.
        let mut x = [10u64, 0u64, 5u64];
        subtract(&mut x, &[3u64, 4u64, 5u64]);
        assert_eq!(
            x,
            [6, 0u64.wrapping_sub(4), 0],
            "equality at LSB word with mid-word borrow: MSB pays exactly one",
        );
    }

    /// Random-input parity: broadword output must equal the reference max for every register.
    #[test]
    fn broadword_matches_reference_p10b5() {
        const REGISTER_SIZE: usize = 5;
        const NUM_REGS: usize = 1024;
        const NUM_WORDS: usize = NUM_REGS * REGISTER_SIZE / 64;

        let mut dst: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, 0x00A1_1CE0);
        let src: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, 0x0000_B0B0);
        let mut dst_reference = dst;

        let mut msb_mask = [0u64; NUM_WORDS];
        let mut lsb_mask = [0u64; NUM_WORDS];
        build_msb_mask(&mut msb_mask, REGISTER_SIZE, NUM_REGS);
        build_lsb_mask(&mut lsb_mask, REGISTER_SIZE, NUM_REGS);
        let mut acc = [0u64; NUM_WORDS];
        let mut mask_scratch = [0u64; NUM_WORDS];
        merge_max_broadword(
            &mut dst,
            &src,
            &msb_mask,
            &lsb_mask,
            REGISTER_SIZE,
            &mut acc,
            &mut mask_scratch,
        );
        reference_max(&mut dst_reference, &src, REGISTER_SIZE, NUM_REGS);
        assert_eq!(dst, dst_reference, "broadword max must match reference");
    }

    /// Parity check at `Bits4`, where every register fits inside a single word (`64 / 4 = 16`
    /// registers per word, no straddling). Exercises the branchless "same-word" path in
    /// `reference_max`, the trivial `hi = 0` corner of Phase 2's shift, and confirms the
    /// broadword algorithm degrades cleanly to the no-straddle case.
    #[test]
    fn broadword_matches_reference_p10b4() {
        const REGISTER_SIZE: usize = 4;
        const NUM_REGS: usize = 1024;
        const NUM_WORDS: usize = NUM_REGS * REGISTER_SIZE / 64; // 64

        let mut dst: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, 0x00A1_1CE0);
        let src: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, 0x0000_B0B0);
        let mut dst_reference = dst;

        let mut msb_mask = [0u64; NUM_WORDS];
        let mut lsb_mask = [0u64; NUM_WORDS];
        build_msb_mask(&mut msb_mask, REGISTER_SIZE, NUM_REGS);
        build_lsb_mask(&mut lsb_mask, REGISTER_SIZE, NUM_REGS);
        let mut acc = [0u64; NUM_WORDS];
        let mut mask_scratch = [0u64; NUM_WORDS];
        merge_max_broadword(
            &mut dst,
            &src,
            &msb_mask,
            &lsb_mask,
            REGISTER_SIZE,
            &mut acc,
            &mut mask_scratch,
        );
        reference_max(&mut dst_reference, &src, REGISTER_SIZE, NUM_REGS);
        assert_eq!(
            dst, dst_reference,
            "broadword max must match reference at Bits4"
        );
    }

    /// Same parity check at `Bits6`, which straddles differently.
    #[test]
    fn broadword_matches_reference_p10b6() {
        const REGISTER_SIZE: usize = 6;
        const NUM_REGS: usize = 1024;
        const NUM_WORDS: usize = NUM_REGS * REGISTER_SIZE / 64;

        let mut dst: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, 0x00A1_1CE0);
        let src: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, 0x0000_B0B0);
        let mut dst_reference = dst;

        let mut msb_mask = [0u64; NUM_WORDS];
        let mut lsb_mask = [0u64; NUM_WORDS];
        build_msb_mask(&mut msb_mask, REGISTER_SIZE, NUM_REGS);
        build_lsb_mask(&mut lsb_mask, REGISTER_SIZE, NUM_REGS);
        let mut acc = [0u64; NUM_WORDS];
        let mut mask_scratch = [0u64; NUM_WORDS];
        merge_max_broadword(
            &mut dst,
            &src,
            &msb_mask,
            &lsb_mask,
            REGISTER_SIZE,
            &mut acc,
            &mut mask_scratch,
        );
        reference_max(&mut dst_reference, &src, REGISTER_SIZE, NUM_REGS);
        assert_eq!(dst, dst_reference);
    }

    /// A broadly random property check with many seeds, so a single pathological register
    /// value cannot hide behind lucky bytes.
    #[test]
    fn broadword_matches_reference_random_seeds() {
        const REGISTER_SIZE: usize = 5;
        const NUM_REGS: usize = 1024;
        const NUM_WORDS: usize = NUM_REGS * REGISTER_SIZE / 64;

        let mut msb_mask = [0u64; NUM_WORDS];
        let mut lsb_mask = [0u64; NUM_WORDS];
        build_msb_mask(&mut msb_mask, REGISTER_SIZE, NUM_REGS);
        build_lsb_mask(&mut lsb_mask, REGISTER_SIZE, NUM_REGS);

        for seed_index in 0..32u64 {
            let mut seed_state = seed_index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let dst_seed = iter_random_values::<u64>(1, None, Some(seed_state))
                .next()
                .unwrap();
            seed_state = seed_state.wrapping_add(0x1234_5678);
            let src_seed = iter_random_values::<u64>(1, None, Some(seed_state))
                .next()
                .unwrap();

            let mut dst: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, dst_seed);
            let src: [u64; NUM_WORDS] = seeded_slab(REGISTER_SIZE, NUM_REGS, src_seed);
            let mut dst_reference = dst;

            let mut acc = [0u64; NUM_WORDS];
            let mut mask_scratch = [0u64; NUM_WORDS];
            merge_max_broadword(
                &mut dst,
                &src,
                &msb_mask,
                &lsb_mask,
                REGISTER_SIZE,
                &mut acc,
                &mut mask_scratch,
            );
            reference_max(&mut dst_reference, &src, REGISTER_SIZE, NUM_REGS);
            assert_eq!(
                dst, dst_reference,
                "broadword max diverged at seed_index={seed_index}",
            );
        }
    }

    // Proptest-driven property check on top of the deterministic 32-seed sweep. Draws
    // arbitrary `[u64; 80]` operand pairs for the `Precision10 Bits5` shape and asserts the
    // broadword register-wise max matches the reference max. Proptest picks 256 cases by
    // default and shrinks any failing input to a minimal counterexample, so a regression
    // that only fires on a specific bit pattern surfaces immediately rather than lurking
    // behind the fixed seed set.
    proptest::proptest! {
        #[test]
        fn broadword_matches_reference_proptest_p10b5(
            dst_in in proptest::prelude::any::<[u64; 80]>(),
            src_in in proptest::prelude::any::<[u64; 80]>(),
        ) {
            const REGISTER_SIZE: usize = 5;
            const NUM_REGS: usize = 1024;
            let mut msb_mask = [0u64; 80];
            let mut lsb_mask = [0u64; 80];
            build_msb_mask(&mut msb_mask, REGISTER_SIZE, NUM_REGS);
            build_lsb_mask(&mut lsb_mask, REGISTER_SIZE, NUM_REGS);
            let mut dst = dst_in;
            let mut dst_reference = dst_in;
            let mut acc = [0u64; 80];
            let mut mask_scratch = [0u64; 80];
            merge_max_broadword(
                &mut dst,
                &src_in,
                &msb_mask,
                &lsb_mask,
                REGISTER_SIZE,
                &mut acc,
                &mut mask_scratch,
            );
            reference_max(&mut dst_reference, &src_in, REGISTER_SIZE, NUM_REGS);
            proptest::prop_assert_eq!(dst, dst_reference);
        }
    }

    /// The runtime mask builders must place each bit exactly at the register position implied
    /// by sketching-core's [`sketching_core::split_packed_index`], for every `(P, B)` pair the
    /// crate supports. Any future change to the packing math trips this test loudly.
    /// `#[test_precisions_and_bits]` expands into one `#[test]` per `(P, B)` on the default
    /// matrix (`{P4, P8, P12, P16} x {B4, B6}`) and one per pair on the full grid when
    /// compiled with `--features exhaustive-tests`.
    #[hyperloglog_derive::test_precisions_and_bits]
    fn masks_match_sketching_core_layout<P, B>()
    where
        P: crate::prelude::Precision + crate::prelude::PackedRegister<B>,
        B: crate::prelude::Bits,
    {
        let register_size = B::NUMBER_OF_BITS_USIZE;
        let num_regs = 1_usize << P::EXPONENT;
        let num_words = (num_regs * register_size).div_ceil(64);

        let mut msb_mask = alloc::vec![0u64; num_words];
        let mut lsb_mask = alloc::vec![0u64; num_words];
        build_msb_mask(&mut msb_mask, register_size, num_regs);
        build_lsb_mask(&mut lsb_mask, register_size, num_regs);

        for i in 0..num_regs {
            let (word_msb, offset_msb) = sketching_core::split_packed_index::<B>(i);
            let msb_bit_in_word = 63 - u32::from(offset_msb);
            assert_ne!(
                msb_mask[word_msb] & (1u64 << msb_bit_in_word),
                0,
                "MSB mask missing bit at register {i} of (P = {}, B = {})",
                P::EXPONENT,
                B::NUMBER_OF_BITS,
            );

            let lsb_stream_pos = i * register_size + register_size - 1;
            let word_lsb = lsb_stream_pos / 64;
            let lsb_bit_in_word = 63 - (lsb_stream_pos % 64) as u32;
            assert_ne!(
                lsb_mask[word_lsb] & (1u64 << lsb_bit_in_word),
                0,
                "LSB mask missing bit at register {i} of (P = {}, B = {})",
                P::EXPONENT,
                B::NUMBER_OF_BITS,
            );
        }
    }
}
