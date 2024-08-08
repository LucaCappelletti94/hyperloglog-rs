//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
mod utils;
use hyperloglog_rs::prelude::*;
use utils::{populate, splitmix64, statistical_report};

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
    use utils::MutableSet;

    pub(super) trait Union {
        fn union(&self, other: &Self) -> f64;
    }

    impl<I: Hash + Eq + PartialEq> Union for HashSet<I> {
        fn union(&self, other: &Self) -> f64 {
            self.union(other).count() as f64
        }
    }

    impl<
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            Hasher: core::hash::Hasher + Default,
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
            Hasher: core::hash::Hasher + Default,
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

    fn evaluate<P: Precision, S: Union + MutableSet<usize>>(
        left_random_state: u64,
        right_random_state: u64,
    ) -> (f64, usize) {
        let (left_set, mem_size_left) = populate::<P, S>(left_random_state);
        let (right_set, mem_size_right) = populate::<P, S>(right_random_state);
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
        >(left_random_state, right_random_state);
        siphasher13_mrs.push(hll_siphasher13_mr);
        siphasher13_absolute_errors.push((exact - hll_siphasher13).abs() / exact);

        let (hll_siphasher24, hll_siphasher24_mr) = evaluate::<
            P,
            HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, Sip64Scalar<2, 4>>,
        >(left_random_state, right_random_state);
        siphasher24_mrs.push(hll_siphasher24_mr);
        siphasher24_absolute_errors.push((exact - hll_siphasher24).abs() / exact);

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
            MLE<HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, Sip64Scalar<1, 3>>, ERROR>,
        >(left_random_state, right_random_state);
        siphasher13_mrs.push(hll_siphasher13_mr);
        siphasher13_absolute_errors.push((exact - hll_siphasher13).abs() / exact);

        let (hll_siphasher24, hll_siphasher24_mr) = evaluate::<
            P,
            MLE<HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, Sip64Scalar<2, 4>>, ERROR>,
        >(left_random_state, right_random_state);
        siphasher24_mrs.push(hll_siphasher24_mr);
        siphasher24_absolute_errors.push((exact - hll_siphasher24).abs() / exact);

        let (hll_xxhash, hll_xxhash_mr) = evaluate::<
            P,
            MLE<HyperLogLog<P, B, <P as ArrayRegister<B>>::ArrayRegister, twox_hash::XxHash64>, ERROR>,
        >(left_random_state, right_random_state);
        xxhasher_mrs.push(hll_xxhash_mr);
        xxhasher_absolute_errors.push((exact - hll_xxhash).abs() / exact);
    }

    pub(super) fn test_union_comparatively<
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
        let number_of_vectors = 5_000;
        let mut random_state = splitmix64(6516781878233_u64);

        let exact_absolute_errors = vec![0.0; number_of_vectors];
        let mut hll4_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll4_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll4_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll5_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut hll6_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle2_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle2_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle2_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle3_siphasher13_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle3_siphasher24_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut mle3_xxhasher_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut tabac_plus_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_absolute_errors = Vec::with_capacity(number_of_vectors);
        let mut sa_absolute_errors = Vec::with_capacity(number_of_vectors);

        let mut exact_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll4_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll4_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll4_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll5_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll5_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut hll6_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle2_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle2_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle2_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle3_siphasher13_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle3_siphasher24_mrs = Vec::with_capacity(number_of_vectors);
        let mut mle3_xxhasher_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_plus_mrs = Vec::with_capacity(number_of_vectors);
        let mut tabac_pf_mrs = Vec::with_capacity(number_of_vectors);
        let mut sa_mrs = Vec::with_capacity(number_of_vectors);

        (0..number_of_vectors).for_each(|_| {
            let left_random_state = splitmix64(splitmix64(random_state));
            let right_random_state = splitmix64(splitmix64(left_random_state));
            random_state = splitmix64(right_random_state);

            let (exact, exact_mr) = evaluate::<P, HashSet<usize>>(left_random_state, right_random_state);
            exact_mrs.push(exact_mr);

            let (tabac_plus, tabac_plus_mr) = evaluate::<P, TabacHyperLogLogPlus<usize, RandomState>>(
                left_random_state,
                right_random_state,
            );
            tabac_plus_mrs.push(tabac_plus_mr);

            let (tabac_pf, tabac_pf_mr) = evaluate::<P, TabacHyperLogLogPF<usize, RandomState>>(
                left_random_state,
                right_random_state,
            );
            tabac_pf_mrs.push(tabac_pf_mr);

            let (sa, sa_mr) = evaluate::<P, SAHyperLogLog<usize>>(left_random_state, right_random_state);
            sa_mrs.push(sa_mr);

            evaluate_hll::<P, Bits4>(left_random_state, right_random_state, &mut hll4_siphasher13_mrs, &mut hll4_siphasher24_mrs, &mut hll4_xxhasher_mrs, exact, &mut hll4_siphasher13_absolute_errors, &mut hll4_siphasher24_absolute_errors, &mut hll4_xxhasher_absolute_errors);
            evaluate_hll::<P, Bits5>(left_random_state, right_random_state, &mut hll5_siphasher13_mrs, &mut hll5_siphasher24_mrs, &mut hll5_xxhasher_mrs, exact, &mut hll5_siphasher13_absolute_errors, &mut hll5_siphasher24_absolute_errors, &mut hll5_xxhasher_absolute_errors);
            evaluate_hll::<P, Bits6>(left_random_state, right_random_state, &mut hll6_siphasher13_mrs, &mut hll6_siphasher24_mrs, &mut hll6_xxhasher_mrs, exact, &mut hll6_siphasher13_absolute_errors, &mut hll6_siphasher24_absolute_errors, &mut hll6_xxhasher_absolute_errors);
            evaluate_mle::<2, P, Bits6>(left_random_state, right_random_state, &mut mle2_siphasher13_mrs, &mut mle2_siphasher24_mrs, &mut mle2_xxhasher_mrs, exact, &mut mle2_siphasher13_absolute_errors, &mut mle2_siphasher24_absolute_errors, &mut mle2_xxhasher_absolute_errors);
            evaluate_mle::<3, P, Bits6>(left_random_state, right_random_state, &mut mle3_siphasher13_mrs, &mut mle3_siphasher24_mrs, &mut mle3_xxhasher_mrs, exact, &mut mle3_siphasher13_absolute_errors, &mut mle3_siphasher24_absolute_errors, &mut mle3_xxhasher_absolute_errors);


            tabac_plus_absolute_errors.push((exact - tabac_plus).abs() / exact);
            tabac_pf_absolute_errors.push((exact - tabac_pf).abs() / exact);
            sa_absolute_errors.push((exact - sa).abs() / exact);
        });

        statistical_report::<19, P>(
            &[
                "HashSet",
                "Tabac's HLL++",
                "Tabac's HLL",
                "Streaming Algorithms",
                "HLL4 + Siphasher13",
                "HLL4 + Siphasher24",
                "HLL4 + Xxhasher",
                "HLL5 + Siphasher13",
                "HLL5 + Siphasher24",
                "HLL5 + Xxhasher",
                "HLL6 + Siphasher13",
                "HLL6 + Siphasher24",
                "HLL6 + Xxhasher",
                "MLE2 + Siphasher13",
                "MLE2 + Siphasher24",
                "MLE2 + Xxhasher",
                "MLE3 + Siphasher13",
                "MLE3 + Siphasher24",
                "MLE3 + Xxhasher",
            ],
            &[
                &exact_absolute_errors,
                &tabac_plus_absolute_errors,
                &tabac_pf_absolute_errors,
                &sa_absolute_errors,
                &hll4_siphasher13_absolute_errors,
                &hll4_siphasher24_absolute_errors,
                &hll4_xxhasher_absolute_errors,
                &hll5_siphasher13_absolute_errors,
                &hll5_siphasher24_absolute_errors,
                &hll5_xxhasher_absolute_errors,
                &hll6_siphasher13_absolute_errors,
                &hll6_siphasher24_absolute_errors,
                &hll6_xxhasher_absolute_errors,
                &mle2_siphasher13_absolute_errors,
                &mle2_siphasher24_absolute_errors,
                &mle2_xxhasher_absolute_errors,
                &mle3_siphasher13_absolute_errors,
                &mle3_siphasher24_absolute_errors,
                &mle3_xxhasher_absolute_errors,
            ],
            &[
                &exact_mrs,
                &tabac_plus_mrs,
                &tabac_pf_mrs,
                &sa_mrs,
                &hll4_siphasher13_mrs,
                &hll4_siphasher24_mrs,
                &hll4_xxhasher_mrs,
                &hll5_siphasher13_mrs,
                &hll5_siphasher24_mrs,
                &hll5_xxhasher_mrs,
                &hll6_siphasher13_mrs,
                &hll6_siphasher24_mrs,
                &hll6_xxhasher_mrs,
                &mle2_siphasher13_mrs,
                &mle2_siphasher24_mrs,
                &mle2_xxhasher_mrs,
                &mle3_siphasher13_mrs,
                &mle3_siphasher24_mrs,
                &mle3_xxhasher_mrs,
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
                    test_comparison::test_union_comparatively::<$precision>();
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
