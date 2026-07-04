//! Slab-view invariants for the `slice_view` module.
//!
//! Together they lock the promise the module makes: the counter is a whole number of `u64`
//! words with no padding or field-order surprises, its bytes round-trip through `as_words` and
//! `from_words`, and reconstruction refuses slices whose length is not exactly `word_len()`.

use core::mem::{align_of, size_of};

use hyperloglog_rs::prelude::*;
use hyperloglog_rs::slice_view::FixedLayoutRegisters;

/// The declared `word_len()` MUST equal the counter's actual byte footprint divided by 8, for
/// every `(P, B)` combination in the sparse sweep. Any struct-layout drift (a reintroduced
/// `#[repr(Rust)]`, an added field, tail padding) fires here loudly.
#[test]
fn word_len_matches_size_of() {
    fn check<P, B>()
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
        <P as PackedRegister<B>>::Array: FixedLayoutRegisters<P, B>,
    {
        type Counter<P, B> = HyperLogLog<P, B, <P as PackedRegister<B>>::Array>;
        let bytes = size_of::<Counter<P, B>>();
        let align = align_of::<Counter<P, B>>();
        let words = Counter::<P, B>::word_len();
        assert_eq!(
            bytes,
            words * 8,
            "word_len() must equal size_of() / 8 for P{}B{}",
            P::EXPONENT,
            B::NUMBER_OF_BITS,
        );
        assert_eq!(
            align,
            8,
            "align_of() must equal u64's alignment for P{}B{}",
            P::EXPONENT,
            B::NUMBER_OF_BITS,
        );
    }

    // The (P, B) matrix mirrors the default CI matrix in `hyperloglog-derive`, plus the extreme
    // Precision4 case that stresses the smallest possible register array.
    check::<Precision4, Bits4>();
    check::<Precision4, Bits6>();
    check::<Precision8, Bits4>();
    check::<Precision8, Bits6>();
    check::<Precision10, Bits6>();
    check::<Precision12, Bits4>();
    check::<Precision12, Bits6>();
    check::<Precision16, Bits4>();
    check::<Precision16, Bits6>();
}

/// After inserting a stream into the counter, viewing it through `as_words` and reconstructing a
/// counter over that slice via `from_words` MUST yield an equivalent state: further inserts on
/// both copies produce byte-identical slabs. This proves the whole `HyperLogLog` state, sparse
/// mode included, lives inside the visible `[u64]` slab with nothing hidden in padding or
/// elsewhere.
#[test]
fn roundtrip_inserts() {
    type Counter = HyperLogLog<Precision10, Bits6>;
    let mut original = Counter::default();
    // A stream that exercises the sparse -> hash-list transition without saturating the buffer,
    // then the dense-mode path once we push past saturation, so both codec-encoded and register-
    // encoded state have to survive the round-trip.
    for value in 0u64..300 {
        original.insert_value(value);
    }
    let words_snapshot: Vec<u64> = original.as_words().to_vec();
    assert_eq!(words_snapshot.len(), Counter::word_len());

    let reconstructed = Counter::from_words(&words_snapshot)
        .expect("length matches word_len(), alignment holds via &[u64]");
    // Both copies must agree on estimator output at this snapshot.
    assert_eq!(
        original.estimate_cardinality(),
        reconstructed.estimate_cardinality(),
        "reconstructed counter must estimate identically to the original",
    );
    assert_eq!(
        original.is_sorted_value_list(),
        reconstructed.is_sorted_value_list(),
    );
    assert_eq!(
        original.is_sorted_hash_list(),
        reconstructed.is_sorted_hash_list(),
    );

    // Now push both copies further with the same stream and check byte parity, so we know the
    // reconstructed counter's inner state was truly bitwise-identical (not just estimator-
    // equivalent by coincidence). We rebuild the mutable copy by copying the slab words into a
    // fresh backing array, taking `from_words_mut`, and threading the extra insert stream.
    let mut mut_words = words_snapshot.clone();
    {
        let counter = Counter::from_words_mut(&mut mut_words)
            .expect("length matches word_len(), alignment holds via &mut [u64]");
        for value in 300u64..1_000 {
            counter.insert_value(value);
        }
    }
    let mut baseline = original;
    for value in 300u64..1_000 {
        baseline.insert_value(value);
    }
    assert_eq!(
        baseline.as_words(),
        mut_words.as_slice(),
        "byte-identical continuation from a round-tripped slab",
    );
}

/// `from_words` MUST reject a slice whose length is not exactly `word_len()`. The alignment
/// guard inside `from_words` is redundant when the caller passes a valid `&[u64]` (Rust
/// guarantees `&[u64]` is `u64`-aligned by construction) and the release-mode optimiser folds
/// it away as unreachable, so we cannot meaningfully test the misaligned branch here without
/// invoking undefined behaviour that the compiler is allowed to elide. The guard is retained
/// in the impl as a defensive belt against future API extensions that widen the parameter
/// type; the test surface is what a safe caller can actually observe.
#[test]
fn length_rejected() {
    type Counter = HyperLogLog<Precision10, Bits6>;

    // Length below `word_len()`.
    let short = vec![0u64; Counter::word_len() - 1];
    assert!(
        Counter::from_words(&short).is_none(),
        "a slice shorter than word_len() must yield None",
    );

    // Length above `word_len()`.
    let long = vec![0u64; Counter::word_len() + 1];
    assert!(
        Counter::from_words(&long).is_none(),
        "a slice longer than word_len() must yield None",
    );

    // Length of exactly `word_len()` succeeds (positive control, so a bug that makes the length
    // check reject every input would fail here loudly).
    let ok = vec![0u64; Counter::word_len()];
    assert!(
        Counter::from_words(&ok).is_some(),
        "a slice of exactly word_len() must yield Some",
    );

    // Same three cases for the mutable variant.
    let mut short_mut = vec![0u64; Counter::word_len() - 1];
    assert!(Counter::from_words_mut(&mut short_mut).is_none());
    let mut long_mut = vec![0u64; Counter::word_len() + 1];
    assert!(Counter::from_words_mut(&mut long_mut).is_none());
    let mut ok_mut = vec![0u64; Counter::word_len()];
    assert!(Counter::from_words_mut(&mut ok_mut).is_some());
}
