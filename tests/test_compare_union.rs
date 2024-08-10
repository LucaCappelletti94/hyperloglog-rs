//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
mod utils;
use hyperloglog_rs::prelude::*;
use utils::{populate, statistical_report};

#[cfg(feature = "std")]
mod test_comparison {
    use super::*;
    use cardinality_estimator::CardinalityEstimator;
    use core::f64;
    use core::hash::BuildHasher;
    use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
    use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
    use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
    use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::hash::RandomState;
    use streaming_algorithms::HyperLogLog as SAHyperLogLog;
    use utils::MutableSet;

    pub(super) trait Union {
        fn union(&self, other: &Self) -> f64;
    }

    impl<I: Hash + Eq + PartialEq> Union for HashSet<I> {
        fn union(&self, other: &Self) -> f64 {
            self.union(other).count() as f64
        }
    }

    impl Union for RustHyperLogLog {
        fn union(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.merge(&other);
            copy.len() as f64
        }
    }

    impl<const P: usize, I: Hash, H: HasherType, const W: usize> Union
        for CardinalityEstimator<I, H, P, W>
    {
        fn union(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.merge(&other);
            copy.estimate() as f64
        }
    }

    impl<
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            Hasher: HasherType,
        > Union for HyperLogLog<P, B, R, Hasher>
    where
        Self: HyperLogLogTrait<P, B, Hasher>,
    {
        fn union(&self, other: &Self) -> f64 {
            self.estimate_union_cardinality(other)
        }
    }

    impl<
            const ERROR: i32,
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            Hasher: HasherType,
        > Union for MLE<HyperLogLog<P, B, R, Hasher>, ERROR>
    where
        Self: HyperLogLogTrait<P, B, Hasher>,
    {
        fn union(&self, other: &Self) -> f64 {
            self.estimate_union_cardinality(other)
        }
    }

    impl<I: Hash + Eq + PartialEq, B: BuildHasher> Union for TabacHyperLogLogPF<I, B>
    where
        Self: Clone,
    {
        fn union(&self, other: &Self) -> f64 {
            let mut self_copy = self.clone();
            self_copy.merge(&other).unwrap();
            self_copy.count()
        }
    }

    impl<I: Hash + Eq + PartialEq, B: BuildHasher> Union for TabacHyperLogLogPlus<I, B>
    where
        Self: Clone,
    {
        fn union(&self, other: &Self) -> f64 {
            let mut self_copy = self.clone();
            self_copy.merge(&other).unwrap();
            self_copy.count()
        }
    }

    impl<I: Hash + Eq + PartialEq> Union for SAHyperLogLog<I> {
        fn union(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            <SAHyperLogLog<I>>::union(&mut copy, &other);
            copy.len()
        }
    }

    fn evaluate<P: Precision, S: Union + MutableSet<u64>>(
        left_random_state: u64,
        right_random_state: u64,
    ) -> (f64, usize) {
        let (left_set, mem_size_left) = populate::<P, S>(left_random_state, Some(10_000_000));
        let (right_set, mem_size_right) = populate::<P, S>(right_random_state, Some(10_000_000));
        (
            left_set.union(&right_set),
            (mem_size_left + mem_size_right) / 2,
        )
    }

    fn evaluate_hll<
        P: Precision + PrecisionConstants<f64> + ArrayRegister<B> + mem_dbg::MemSize,
        B: Bits + mem_dbg::MemSize,
    >(
        left_random_state: u64,
        right_random_state: u64,
        xxhasher_mrs: &mut Vec<usize>,
        exact: f64,
        xxhasher_absolute_errors: &mut Vec<f64>,
    ) where
        <P as ArrayRegister<B>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        // let (hll_wyhash, hll_wyhash_mr) = evaluate::<
        //     P,
        //     HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, WyHash>,
        // >(left_random_state, right_random_state);
        // wyhash_mrs.push(hll_wyhash_mr);
        // wyhash_absolute_errors.push((exact - hll_wyhash).abs() / exact);

        // let (hll_default, hll_default_mr) = evaluate::<
        //     P,
        //     HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, DefaultHasher>,
        // >(left_random_state, right_random_state);
        // default_mrs.push(hll_default_mr);
        // default_absolute_errors.push((exact - hll_default).abs() / exact);

        let (hll_xxhash, hll_xxhash_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, twox_hash::XxHash64>,
        >(left_random_state, right_random_state);
        xxhasher_mrs.push(hll_xxhash_mr);
        xxhasher_absolute_errors.push((exact - hll_xxhash).abs() / exact);
    }

    fn evaluate_mle<
        const ERROR: i32,
        P: Precision + PrecisionConstants<f64> + ArrayRegister<B> + mem_dbg::MemSize,
        B: Bits + mem_dbg::MemSize,
    >(
        left_random_state: u64,
        right_random_state: u64,
        xxhasher_mrs: &mut Vec<usize>,
        exact: f64,
        xxhasher_absolute_errors: &mut Vec<f64>,
    ) where
        <P as ArrayRegister<B>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        let (hll_xxhash, hll_xxhash_mr) = evaluate::<
            P,
            MLE<
                HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, twox_hash::XxHash64>,
                ERROR,
            >,
        >(left_random_state, right_random_state);
        xxhasher_mrs.push(hll_xxhash_mr);
        xxhasher_absolute_errors.push((exact - hll_xxhash).abs() / exact);
    }

    pub(super) fn test_union_comparatively<
        const EXPONENT: usize,
        P: mem_dbg::MemSize
            + Precision
            + ArrayRegister<Bits6>
            + ArrayRegister<Bits5>
            + ArrayRegister<Bits4>
            + PrecisionConstants<f64>
            + ArrayRegister<Bits5>,
    >()
    where
        <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        assert_eq!(P::EXPONENT, EXPONENT);
        let number_of_vectors = 5_000;
        let mut random_state = splitmix64(6516781878233_u64);

        let exact_absolute_errors = vec![0.0; number_of_vectors];
        // let mut hll4_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll4_default_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll4_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll5_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll5_default_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll5_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll6_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut hll6_default_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut mle2_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut mle2_default_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle2_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut mle3_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut mle3_default_absolute_errors = Vec::with_capacity(number_of_vectors);
        // let mut mle3_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut tabac_plus_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut sa_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut cardinality_estimator_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut rust_hll_absolute_errors = Vec::with_capacity(number_of_vectors);

        let mut exact_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll4_wyhash_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll4_default_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll4_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll5_wyhash_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll5_default_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll5_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll6_wyhash_mrs = Vec::with_capacity(number_of_vectors);
        // let mut hll6_default_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        // let mut mle2_wyhash_mrs = Vec::with_capacity(number_of_vectors);
        // let mut mle2_default_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle2_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        // let mut mle3_wyhash_mrs = Vec::with_capacity(number_of_vectors);
        // let mut mle3_default_mrs = Vec::with_capacity(number_of_vectors);
        // let mut mle3_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_plus_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_mrs = Vec::with_capacity(number_of_vectors);
        let mut sa_mrs = Vec::with_capacity(number_of_vectors);
        let mut cardinality_estimator_mrs = Vec::with_capacity(number_of_vectors);
        let mut rust_hll_mrs = Vec::with_capacity(number_of_vectors);

        (0..number_of_vectors).for_each(|_| {
            let left_random_state = splitmix64(splitmix64(random_state));
            let right_random_state = splitmix64(splitmix64(left_random_state));
            random_state = splitmix64(right_random_state);

            let (exact, exact_mr) =
                evaluate::<P, HashSet<u64>>(left_random_state, right_random_state);
            exact_mrs.push(exact_mr);

            let (tabac_plus, tabac_plus_mr) = evaluate::<P, TabacHyperLogLogPlus<u64, RandomState>>(
                left_random_state,
                right_random_state,
            );
            tabac_plus_mrs.push(tabac_plus_mr);

            let (tabac_pf, tabac_pf_mr) = evaluate::<P, TabacHyperLogLogPF<u64, RandomState>>(
                left_random_state,
                right_random_state,
            );
            tabac_pf_mrs.push(tabac_pf_mr);

            let (sa, sa_mr) =
                evaluate::<P, SAHyperLogLog<u64>>(left_random_state, right_random_state);
            sa_mrs.push(sa_mr);

            let (rust_hll, rust_hll_mr) =
                evaluate::<P, RustHyperLogLog>(left_random_state, right_random_state);
            rust_hll_mrs.push(rust_hll_mr);

            let (cardinality_estimator, cardinality_estimator_mr) =
                evaluate::<P, CardinalityEstimator<u64, wyhash::WyHash, EXPONENT, 6>>(
                    left_random_state,
                    right_random_state,
                );
            cardinality_estimator_mrs.push(cardinality_estimator_mr);

            // evaluate_hll::<P, Bits4>(left_random_state, right_random_state, &mut hll4_wyhash_mrs, &mut hll4_default_mrs, &mut hll4_xxhasher_mrs, exact, &mut hll4_wyhash_absolute_errors, &mut hll4_default_absolute_errors, &mut hll4_xxhasher_absolute_errors);
            // evaluate_hll::<P, Bits5>(left_random_state, right_random_state, &mut hll5_wyhash_mrs, &mut hll5_default_mrs, &mut hll5_xxhasher_mrs, exact, &mut hll5_wyhash_absolute_errors, &mut hll5_default_absolute_errors, &mut hll5_xxhasher_absolute_errors);
            evaluate_hll::<P, Bits6>(
                left_random_state,
                right_random_state,
                &mut hll6_xxhasher_mrs,
                exact,
                &mut hll6_xxhasher_absolute_errors,
            );
            evaluate_mle::<2, P, Bits6>(
                left_random_state,
                right_random_state,
                &mut mle2_xxhasher_mrs,
                exact,
                &mut mle2_xxhasher_absolute_errors,
            );
            // evaluate_mle::<3, P, Bits6>(left_random_state, right_random_state, &mut mle3_wyhash_mrs, &mut mle3_default_mrs, &mut mle3_xxhasher_mrs, exact, &mut mle3_wyhash_absolute_errors, &mut mle3_default_absolute_errors, &mut mle3_xxhasher_absolute_errors);

            tabac_plus_absolute_errors.push((exact - tabac_plus).abs() / exact);
            tabac_pf_absolute_errors.push((exact - tabac_pf).abs() / exact);
            sa_absolute_errors.push((exact - sa).abs() / exact);
            cardinality_estimator_absolute_errors
                .push((exact - cardinality_estimator).abs() / exact);
            rust_hll_absolute_errors.push((exact - rust_hll).abs() / exact);
        });

        statistical_report::<8, P>(
            &[
                "HashSet",
                "Tabac's HLL++",
                "Tabac's HLL",
                "Streaming Algorithms",
                // "HLL4 + WyHash",
                // "HLL4 + DefaultHasher",
                // "HLL4 + Xxhasher",
                // "HLL5 + WyHash",
                // "HLL5 + DefaultHasher",
                // "HLL5 + Xxhasher",
                // "HLL6 + WyHash",
                // "HLL6 + DefaultHasher",
                "HLL6 + Xxhasher",
                // "MLE2 + WyHash",
                // "MLE2 + DefaultHasher",
                "MLE2 + Xxhasher",
                // "MLE3 + WyHash",
                // "MLE3 + DefaultHasher",
                // "MLE3 + Xxhasher",
                "Rust-HLL",
                "Cardinality Estimator",
            ],
            &[
                &exact_absolute_errors,
                &tabac_plus_absolute_errors,
                &tabac_pf_absolute_errors,
                &sa_absolute_errors,
                // &hll4_wyhash_absolute_errors,
                // &hll4_default_absolute_errors,
                // &hll4_xxhasher_absolute_errors,
                // &hll5_wyhash_absolute_errors,
                // &hll5_default_absolute_errors,
                // &hll5_xxhasher_absolute_errors,
                // &hll6_wyhash_absolute_errors,
                // &hll6_default_absolute_errors,
                &hll6_xxhasher_absolute_errors,
                // &mle2_wyhash_absolute_errors,
                // &mle2_default_absolute_errors,
                &mle2_xxhasher_absolute_errors,
                // &mle3_wyhash_absolute_errors,
                // &mle3_default_absolute_errors,
                // &mle3_xxhasher_absolute_errors,
                &rust_hll_absolute_errors,
                &cardinality_estimator_absolute_errors,
            ],
            &[
                &exact_mrs,
                &tabac_plus_mrs,
                &tabac_pf_mrs,
                &sa_mrs,
                // &hll4_wyhash_mrs,
                // &hll4_default_mrs,
                // &hll4_xxhasher_mrs,
                // &hll5_wyhash_mrs,
                // &hll5_default_mrs,
                // &hll5_xxhasher_mrs,
                // &hll6_wyhash_mrs,
                // &hll6_default_mrs,
                &hll6_xxhasher_mrs,
                // &mle2_wyhash_mrs,
                // &mle2_default_mrs,
                &mle2_xxhasher_mrs,
                // &mle3_wyhash_mrs,
                // &mle3_default_mrs,
                // &mle3_xxhasher_mrs,
                &rust_hll_mrs,
                &cardinality_estimator_mrs,
            ],
            "union",
        );
    }
}

macro_rules! test_union_comparatively_at_precision {
    ($( $precision: ty ),*) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_union_comparatively_at_ $precision:lower>] () {
                    test_comparison::test_union_comparatively::<{$precision::EXPONENT}, $precision>();
                }
            }
        )*
    };
}

test_union_comparatively_at_precision!(
    Precision4,
    Precision5,
    Precision6,
    Precision7,
    Precision8,
    Precision9,
    Precision10,
    Precision11,
    Precision12,
    Precision13,
    Precision14,
    Precision15,
    Precision16
);
