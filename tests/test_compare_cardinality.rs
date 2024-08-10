//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
mod utils;
use hyperloglog_rs::prelude::*;

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
    use utils::populate;
    use utils::statistical_report;
    use utils::MutableSet;
    use utils::BetaHLL;

    pub(super) trait Cardinality {
        fn cardinality(&self) -> f64;
    }

    impl<I: Hash + Eq + PartialEq> Cardinality for HashSet<I> {
        fn cardinality(&self) -> f64 {
            self.len() as f64
        }
    }

    impl Cardinality for RustHyperLogLog {
        fn cardinality(&self) -> f64 {
            self.len()
        }
    }

    impl<const P: usize, I: Hash, H: core::hash::Hasher + Default, const W: usize> Cardinality
        for CardinalityEstimator<I, H, P, W>
    {
        fn cardinality(&self) -> f64 {
            self.estimate() as f64
        }
    }

    impl<
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            Hasher: HasherType,
        > Cardinality for HyperLogLog<P, B, R, Hasher>
    where
        Self: HyperLogLogTrait<P, B, Hasher>,
    {
        fn cardinality(&self) -> f64 {
            self.estimate_cardinality()
        }
    }

    impl<
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            Hasher: HasherType,
        > Cardinality for BetaHLL<P, B, R, Hasher>
    {
        fn cardinality(&self) -> f64 {
            self.beta_cardinality()
        }
    }

    impl<I: Hash + Eq + PartialEq, B: BuildHasher> Cardinality for TabacHyperLogLogPF<I, B>
    where
        Self: Clone,
    {
        fn cardinality(&self) -> f64 {
            self.clone().count()
        }
    }

    impl<I: Hash + Eq + PartialEq, B: BuildHasher> Cardinality for TabacHyperLogLogPlus<I, B>
    where
        Self: Clone,
    {
        fn cardinality(&self) -> f64 {
            self.clone().count()
        }
    }

    impl<I: Hash + Eq + PartialEq> Cardinality for SAHyperLogLog<I> {
        fn cardinality(&self) -> f64 {
            self.len()
        }
    }

    fn evaluate<P: mem_dbg::MemSize + Precision, S: Cardinality + MutableSet<u64>>(
        random_state: u64,
    ) -> (f64, usize) {
        let (set, set_size) = populate::<P, S>(random_state, None);
        (set.cardinality(), set_size)
    }

    fn evaluate_hll<
        P: Precision + PrecisionConstants<f64> + ArrayRegister<B> + mem_dbg::MemSize,
        B: Bits + mem_dbg::MemSize,
    >(
        random_state: u64,
        xxhasher_mrs: &mut Vec<usize>,
        wyhash_mrs: &mut Vec<usize>,
        exact: f64,
        xxhasher_absolute_errors: &mut Vec<f64>,
        wyhash_absolute_errors: &mut Vec<f64>,
    ) where
        <P as ArrayRegister<B>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        let (hll_xxhash, hll_xxhash_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, twox_hash::XxHash64>,
        >(random_state);
        xxhasher_mrs.push(hll_xxhash_mr);
        xxhasher_absolute_errors.push((exact - hll_xxhash).abs() / exact);

        let (hll_wyhash, hll_wyhash_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, wyhash::WyHash>,
        >(random_state);
        wyhash_mrs.push(hll_wyhash_mr);
        wyhash_absolute_errors.push((exact - hll_wyhash).abs() / exact);
    }

    fn evaluate_beta<
        P: Precision + PrecisionConstants<f64> + ArrayRegister<B> + mem_dbg::MemSize,
        B: Bits + mem_dbg::MemSize,
    >(
        random_state: u64,
        xxhasher_mrs: &mut Vec<usize>,
        wyhash_mrs: &mut Vec<usize>,
        exact: f64,
        xxhasher_absolute_errors: &mut Vec<f64>,
        wyhash_absolute_errors: &mut Vec<f64>,
    ) where
        <P as ArrayRegister<B>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        let (beta_xxhash, beta_xxhash_mr) = evaluate::<
            P,
            BetaHLL<P, B, <P as ArrayRegister<B>>::ArrayRegister, twox_hash::XxHash64>,
        >(random_state);
        xxhasher_mrs.push(beta_xxhash_mr);
        xxhasher_absolute_errors.push((exact - beta_xxhash).abs() / exact);

        let (beta_wyhash, beta_wyhash_mr) = evaluate::<
            P,
            BetaHLL<P, B, <P as ArrayRegister<B>>::ArrayRegister, wyhash::WyHash>,
        >(random_state);
        wyhash_mrs.push(beta_wyhash_mr);
        wyhash_absolute_errors.push((exact - beta_wyhash).abs() / exact);
    }

    pub(super) fn test_cardinality_comparatively<
        const EXPONENT: usize,
        P: mem_dbg::MemSize
            + Precision
            + ArrayRegister<Bits8>
            + ArrayRegister<Bits6>
            + PrecisionConstants<f64>
            + ArrayRegister<Bits5>
            + ArrayRegister<Bits4>,
    >()
    where
        <P as ArrayRegister<Bits8>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        assert_eq!(P::EXPONENT, EXPONENT);
        let number_of_vectors = 3_000;
        let mut random_state = splitmix64(6516781878233_u64);

        let mut exact_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_plus_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_mrs = Vec::with_capacity(number_of_vectors);
        let mut sa_mrs = Vec::with_capacity(number_of_vectors);
        let mut rust_hll_mrs = Vec::with_capacity(number_of_vectors);
        let mut cardinality_estimator_mrs = Vec::with_capacity(number_of_vectors);

        let mut hll6_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_wyhash_mrs = Vec::with_capacity(number_of_vectors);
        let mut beta6_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut beta6_wyhash_mrs = Vec::with_capacity(number_of_vectors);

        let exact_absolute_errors = vec![0.0; number_of_vectors];
        let mut tabac_plus_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut sa_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut rust_hll_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut cardinality_estimator_absolute_errors = Vec::with_capacity(number_of_vectors);

        let mut hll6_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut beta6_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut beta6_wyhash_absolute_errors = Vec::with_capacity(number_of_vectors);

        (0..number_of_vectors).for_each(|_| {
            random_state = splitmix64(splitmix64(random_state));

            let (exact, exact_mr) = evaluate::<P, HashSet<u64>>(random_state);
            exact_mrs.push(exact_mr);

            let (tabac_plus, tabac_plus_mr) =
                evaluate::<P, TabacHyperLogLogPlus<u64, RandomState>>(random_state);
            tabac_plus_mrs.push(tabac_plus_mr);

            let (tabac_pf, tabac_pf_mr) =
                evaluate::<P, TabacHyperLogLogPF<u64, RandomState>>(random_state);
            tabac_pf_mrs.push(tabac_pf_mr);

            let (sa, sa_mr) = evaluate::<P, SAHyperLogLog<u64>>(random_state);
            sa_mrs.push(sa_mr);

            let (rust_hll, rust_hll_mr) = evaluate::<P, RustHyperLogLog>(random_state);
            rust_hll_mrs.push(rust_hll_mr);

            let (cardinality_estimator, cardinality_estimator_mr) = evaluate::<
                P,
                CardinalityEstimator<u64, wyhash::WyHash, EXPONENT, 6>,
            >(random_state);
            cardinality_estimator_mrs.push(cardinality_estimator_mr);

            evaluate_hll::<P, Bits6>(
                random_state,
                &mut hll6_xxhasher_mrs,
                &mut hll6_wyhash_mrs,
                exact,
                &mut hll6_xxhasher_absolute_errors,
                &mut hll6_wyhash_absolute_errors,
            );

            evaluate_beta::<P, Bits6>(
                random_state,
                &mut beta6_xxhasher_mrs,
                &mut beta6_wyhash_mrs,
                exact,
                &mut beta6_xxhasher_absolute_errors,
                &mut beta6_wyhash_absolute_errors,
            );

            tabac_plus_absolute_errors.push((exact - tabac_plus).abs() / exact);
            tabac_pf_absolute_errors.push((exact - tabac_pf).abs() / exact);
            sa_absolute_errors.push((exact - sa).abs() / exact);
            rust_hll_absolute_errors.push((exact - rust_hll).abs() / exact);
            cardinality_estimator_absolute_errors
                .push((exact - cardinality_estimator).abs() / exact);
        });

        statistical_report::<10, P>(
            &[
                "HashSet",
                "Tabac's HLL++",
                "Tabac's HLL",
                "Streaming Algorithms",
                "HLL6 + Xxhasher",
                "HLL6 + WyHash",
                "Rust-HLL",
                "Cardinality Estimator",
                "Beta6 + Xxhasher",
                "Beta6 + WyHash",
            ],
            &[
                &exact_absolute_errors,
                &tabac_plus_absolute_errors,
                &tabac_pf_absolute_errors,
                &sa_absolute_errors,
                &hll6_xxhasher_absolute_errors,
                &hll6_wyhash_absolute_errors,
                &rust_hll_absolute_errors,
                &cardinality_estimator_absolute_errors,
                &beta6_xxhasher_absolute_errors,
                &beta6_wyhash_absolute_errors,
            ],
            &[
                &exact_mrs,
                &tabac_plus_mrs,
                &tabac_pf_mrs,
                &sa_mrs,
                &hll6_xxhasher_mrs,
                &hll6_wyhash_mrs,
                &rust_hll_mrs,
                &cardinality_estimator_mrs,
                &beta6_xxhasher_mrs,
                &beta6_wyhash_mrs,
            ],
            "cardinality",
        );
    }
}

macro_rules! test_cardinality_comparatively_at_precision {
    ($( $precision: ty ),*) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_cardinality_comparatively_at_ $precision:lower>] () {
                    test_comparison::test_cardinality_comparatively::<{$precision::EXPONENT}, $precision>();
                }
            }
        )*
    };
}

test_cardinality_comparatively_at_precision!(
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
