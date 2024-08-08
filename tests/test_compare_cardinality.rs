//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
mod utils;
use hyperloglog_rs::prelude::*;

#[cfg(feature = "std")]
mod test_comparison {
    use super::*;
    use core::f64;
    use core::hash::BuildHasher;
    use hyperloglog_rs::sip::Sip64Scalar;
    use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
    use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
    use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::hash::RandomState;
    use streaming_algorithms::HyperLogLog as SAHyperLogLog;
    use utils::populate;
    use utils::MutableSet;
    use utils::{splitmix64, statistical_report};

    pub(super) trait Cardinality {
        fn cardinality(&self) -> f64;
    }

    impl<I: Hash + Eq + PartialEq> Cardinality for HashSet<I> {
        fn cardinality(&self) -> f64 {
            self.len() as f64
        }
    }

    impl<
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            Hasher: core::hash::Hasher + Default,
        > Cardinality for HyperLogLog<P, B, R, Hasher>
    where
        Self: HyperLogLogTrait<P, B, Hasher>,
    {
        fn cardinality(&self) -> f64 {
            self.estimate_cardinality()
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

    fn evaluate<P: mem_dbg::MemSize + Precision, S: Cardinality + MutableSet<usize>>(
        random_state: u64,
    ) -> (f64, usize) {
        let (set, set_size) = populate::<P, S>(random_state);
        (set.cardinality(), set_size)
    }

    fn evaluate_hll<
        P: Precision + PrecisionConstants<f64> + ArrayRegister<B> + mem_dbg::MemSize,
        B: Bits + mem_dbg::MemSize,
    >(
        random_state: u64,
        siphasher13_mrs: &mut Vec<usize>,
        siphasher24_mrs: &mut Vec<usize>,
        xxhasher_mrs: &mut Vec<usize>,
        exact: f64,
        siphasher13_absolute_errors: &mut Vec<f64>,
        siphasher24_absolute_errors: &mut Vec<f64>,
        xxhasher_absolute_errors: &mut Vec<f64>,
    ) where
        <P as ArrayRegister<B>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        let (hll_siphasher13, hll_siphasher13_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, Sip64Scalar<1, 3>>,
        >(random_state);
        siphasher13_mrs.push(hll_siphasher13_mr);
        siphasher13_absolute_errors.push((exact - hll_siphasher13).abs() / exact);

        let (hll_siphasher24, hll_siphasher24_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, Sip64Scalar<2, 4>>,
        >(random_state);
        siphasher24_mrs.push(hll_siphasher24_mr);
        siphasher24_absolute_errors.push((exact - hll_siphasher24).abs() / exact);

        let (hll_xxhash, hll_xxhash_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, twox_hash::XxHash64>,
        >(random_state);
        xxhasher_mrs.push(hll_xxhash_mr);
        xxhasher_absolute_errors.push((exact - hll_xxhash).abs() / exact);
    }

    pub(super) fn test_cardinality_comparatively<
        P: mem_dbg::MemSize
            + Precision
            + ArrayRegister<Bits6>
            + PrecisionConstants<f64>
            + ArrayRegister<Bits5>
            + ArrayRegister<Bits4>,
    >()
    where
        <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfZeros: mem_dbg::MemSize,
    {
        let number_of_vectors = 5_000;
        let mut random_state = splitmix64(6516781878233_u64);

        let mut exact_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_plus_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_mrs = Vec::with_capacity(number_of_vectors);
        let mut sa_mrs = Vec::with_capacity(number_of_vectors);

        let mut hll4_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll4_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll4_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll5_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_xxhasher_mrs = Vec::with_capacity(number_of_vectors);

        let exact_absolute_errors = vec![0.0; number_of_vectors];
        let mut tabac_plus_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut sa_absolute_errors = Vec::with_capacity(number_of_vectors);

        let mut hll4_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll4_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll4_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll5_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);

        (0..number_of_vectors).for_each(|_| {
            random_state = splitmix64(splitmix64(random_state));

            let (exact, exact_mr) = evaluate::<P, HashSet<usize>>(random_state);
            exact_mrs.push(exact_mr);

            let (tabac_plus, tabac_plus_mr) =
                evaluate::<P, TabacHyperLogLogPlus<usize, RandomState>>(random_state);
            tabac_plus_mrs.push(tabac_plus_mr);

            let (tabac_pf, tabac_pf_mr) =
                evaluate::<P, TabacHyperLogLogPF<usize, RandomState>>(random_state);
            tabac_pf_mrs.push(tabac_pf_mr);

            let (sa, sa_mr) = evaluate::<P, SAHyperLogLog<usize>>(random_state);
            sa_mrs.push(sa_mr);

            evaluate_hll::<P, Bits4>(
                random_state,
                &mut hll4_siphasher13_mrs,
                &mut hll4_siphasher24_mrs,
                &mut hll4_xxhasher_mrs,
                exact,
                &mut hll4_siphasher13_absolute_errors,
                &mut hll4_siphasher24_absolute_errors,
                &mut hll4_xxhasher_absolute_errors,
            );

            evaluate_hll::<P, Bits5>(
                random_state,
                &mut hll5_siphasher13_mrs,
                &mut hll5_siphasher24_mrs,
                &mut hll5_xxhasher_mrs,
                exact,
                &mut hll5_siphasher13_absolute_errors,
                &mut hll5_siphasher24_absolute_errors,
                &mut hll5_xxhasher_absolute_errors,
            );

            evaluate_hll::<P, Bits6>(
                random_state,
                &mut hll6_siphasher13_mrs,
                &mut hll6_siphasher24_mrs,
                &mut hll6_xxhasher_mrs,
                exact,
                &mut hll6_siphasher13_absolute_errors,
                &mut hll6_siphasher24_absolute_errors,
                &mut hll6_xxhasher_absolute_errors,
            );

            tabac_plus_absolute_errors.push((exact - tabac_plus).abs() / exact);
            tabac_pf_absolute_errors.push((exact - tabac_pf).abs() / exact);
            sa_absolute_errors.push((exact - sa).abs() / exact);
        });

        statistical_report::<13, P>(
            &[
                "HashSet",
                "Tabac's HLL++",
                "Tabac's HLL",
                "Streaming Algorithms",
                "HLL6 + Siphasher13",
                "HLL6 + Siphasher24",
                "HLL6 + Xxhasher",
                "HLL5 + Siphasher13",
                "HLL5 + Siphasher24",
                "HLL5 + Xxhasher",
                "HLL4 + Siphasher13",
                "HLL4 + Siphasher24",
                "HLL4 + Xxhasher",
            ],
            &[
                &exact_absolute_errors,
                &tabac_plus_absolute_errors,
                &tabac_pf_absolute_errors,
                &sa_absolute_errors,
                &hll6_siphasher13_absolute_errors,
                &hll6_siphasher24_absolute_errors,
                &hll6_xxhasher_absolute_errors,
                &hll5_siphasher13_absolute_errors,
                &hll5_siphasher24_absolute_errors,
                &hll5_xxhasher_absolute_errors,
                &hll4_siphasher13_absolute_errors,
                &hll4_siphasher24_absolute_errors,
                &hll4_xxhasher_absolute_errors,
            ],
            &[
                &exact_mrs,
                &tabac_plus_mrs,
                &tabac_pf_mrs,
                &sa_mrs,
                &hll6_siphasher13_mrs,
                &hll6_siphasher24_mrs,
                &hll6_xxhasher_mrs,
                &hll5_siphasher13_mrs,
                &hll5_siphasher24_mrs,
                &hll5_xxhasher_mrs,
                &hll4_siphasher13_mrs,
                &hll4_siphasher24_mrs,
                &hll4_xxhasher_mrs,
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
                    test_comparison::test_cardinality_comparatively::<$precision>();
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
