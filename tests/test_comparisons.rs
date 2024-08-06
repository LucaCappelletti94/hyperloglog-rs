//! Comparative test making sure that the quality of the estimation
//! obtain by the hyperloglog-rs library is equal or better than the
//! competitor libraries.
//! Evaluation of set-like properties across different data structures.
mod utils;
use hyperloglog_rs::prelude::*;
use utils::{splitmix64, xorshift64};

#[cfg(feature = "std")]
mod test_comparison {
    use super::*;
    use core::hash::BuildHasher;
    use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
    use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
    use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
    use stattest::test::WilcoxonWTest;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::hash::RandomState;
    use streaming_algorithms::HyperLogLog as SAHyperLogLog;

    pub(super) trait SetLike<I> {
        fn insert(&mut self, x: I);
        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize>;
    }

    impl<I: Hash + Eq + PartialEq> SetLike<I> for HashSet<I> {
        fn insert(&mut self, x: I) {
            self.insert(x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let left_cardinality = self.len();
            let right_cardinality = other.len();
            let union_cardinality = self.union(other).count();
            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    impl<I: Hash, P: Precision + PrecisionConstants<f64>, B: Bits, R: Registers<P, B>> SetLike<I>
        for HyperLogLog<P, B, R>
    where
        Self: HyperLogLogTrait<P, B>,
    {
        fn insert(&mut self, x: I) {
            <Self as HyperLogLogTrait<P, B>>::insert(self, x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let euc = self.estimate_union_and_sets_cardinality(other);
            let left_cardinality = euc.get_left_cardinality() as usize;
            let right_cardinality = euc.get_right_cardinality() as usize;
            let mut union_cardinality = euc.get_union_cardinality() as usize;

            // We need to correct the union for rounding errors.
            if left_cardinality + right_cardinality < union_cardinality {
                union_cardinality = left_cardinality + right_cardinality;
            }

            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    impl<
            I: Hash,
            P: Precision + PrecisionConstants<f64>,
            M: Multiplicities<P, B>,
            B: Bits,
            R: Registers<P, B>,
        > SetLike<I> for HLLMultiplicities<P, B, R, M>
    where
        Self: HyperLogLogTrait<P, B>,
    {
        fn insert(&mut self, x: I) {
            <Self as HyperLogLogTrait<P, B>>::insert(self, x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let euc = self.estimate_union_and_sets_cardinality(other);
            let left_cardinality = euc.get_left_cardinality() as usize;
            let right_cardinality = euc.get_right_cardinality() as usize;
            let mut union_cardinality = euc.get_union_cardinality() as usize;

            // We need to correct the union for rounding errors.
            if left_cardinality + right_cardinality < union_cardinality {
                union_cardinality = left_cardinality + right_cardinality;
            }

            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    impl<
            const ERROR: i32,
            I: Hash,
            P: Precision + PrecisionConstants<f64>,
            B: Bits,
            R: Registers<P, B>,
            M: Multiplicities<P, B>,
        > SetLike<I> for MLE<ERROR, HLLMultiplicities<P, B, R, M>>
    where
        Self: HyperLogLogTrait<P, B>,
    {
        fn insert(&mut self, x: I) {
            <Self as HyperLogLogTrait<P, B>>::insert(self, x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let euc = self.estimate_union_and_sets_cardinality(other);
            let left_cardinality = self.estimate_cardinality() as usize;
            let right_cardinality = other.estimate_cardinality() as usize;
            let mut union_cardinality = euc.get_union_cardinality() as usize;

            // We need to correct the union for rounding errors.
            if left_cardinality + right_cardinality < union_cardinality {
                union_cardinality = left_cardinality + right_cardinality;
            }

            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    impl<I: Hash + Eq + PartialEq, B: BuildHasher> SetLike<I> for TabacHyperLogLogPF<I, B>
    where
        Self: Clone,
    {
        fn insert(&mut self, x: I) {
            TabacHyperLogLog::insert(self, &x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let mut self_copy = self.clone();
            let mut other_copy = other.clone();
            let left_cardinality = self_copy.count() as usize;
            let right_cardinality = other_copy.count() as usize;
            self_copy.merge(&other_copy).unwrap();
            let mut union_cardinality = self_copy.count() as usize;

            if left_cardinality + right_cardinality < union_cardinality {
                union_cardinality = left_cardinality + right_cardinality;
            }

            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    impl<I: Hash + Eq + PartialEq, B: BuildHasher> SetLike<I> for TabacHyperLogLogPlus<I, B>
    where
        Self: Clone,
    {
        fn insert(&mut self, x: I) {
            TabacHyperLogLog::insert(self, &x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let mut self_copy = self.clone();
            let mut other_copy = other.clone();
            let left_cardinality = self_copy.count() as usize;
            let right_cardinality = other_copy.count() as usize;
            self_copy.merge(&other_copy).unwrap();
            let mut union_cardinality = self_copy.count() as usize;

            if left_cardinality + right_cardinality < union_cardinality {
                union_cardinality = left_cardinality + right_cardinality;
            }

            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    impl<I: Hash + Eq + PartialEq> SetLike<I> for SAHyperLogLog<I> {
        fn insert(&mut self, x: I) {
            self.push(&x);
        }

        fn cardinalities(&self, other: &Self) -> EstimatedUnionCardinalities<usize> {
            let left_cardinality = self.len() as usize;
            let right_cardinality = other.len() as usize;
            let mut copy = self.clone();
            copy.union(other);
            let mut union_cardinality = copy.len() as usize;
            if left_cardinality + right_cardinality < union_cardinality {
                union_cardinality = left_cardinality + right_cardinality;
            }

            EstimatedUnionCardinalities::from((
                left_cardinality,
                right_cardinality,
                union_cardinality,
            ))
        }
    }

    struct Cardinalities {
        left_cardinality: usize,
        right_cardinality: usize,
        intersection_cardinality: usize,
        union_cardinality: usize,
    }

    #[derive(Debug, Default, Copy, Clone)]
    struct AbsoluteError {
        left_cardinality: f64,
        right_cardinality: f64,
        intersection_cardinality: f64,
        union_cardinality: f64,
    }

    impl AbsoluteError {
        fn from_cardinalities(exact: &Cardinalities, estimated: &Cardinalities) -> Self {
            Self {
                left_cardinality: (exact.left_cardinality as f64
                    - estimated.left_cardinality as f64)
                    .abs()
                    / exact.left_cardinality.max(1) as f64,
                right_cardinality: (exact.right_cardinality as f64
                    - estimated.right_cardinality as f64)
                    .abs()
                    / exact.right_cardinality.max(1) as f64,
                intersection_cardinality: (exact.intersection_cardinality as f64
                    - estimated.intersection_cardinality as f64)
                    .abs()
                    / exact.intersection_cardinality.max(1) as f64,
                union_cardinality: (exact.union_cardinality as f64
                    - estimated.union_cardinality as f64)
                    .abs()
                    / exact.union_cardinality.max(1) as f64,
            }
        }
    }

    impl core::ops::Sub for AbsoluteError {
        type Output = Self;

        fn sub(self, other: Self) -> Self {
            Self {
                left_cardinality: self.left_cardinality - other.left_cardinality,
                right_cardinality: self.right_cardinality - other.right_cardinality,
                intersection_cardinality: self.intersection_cardinality
                    - other.intersection_cardinality,
                union_cardinality: self.union_cardinality - other.union_cardinality,
            }
        }
    }

    impl core::ops::Add for AbsoluteError {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            Self {
                left_cardinality: self.left_cardinality + other.left_cardinality,
                right_cardinality: self.right_cardinality + other.right_cardinality,
                intersection_cardinality: self.intersection_cardinality
                    + other.intersection_cardinality,
                union_cardinality: self.union_cardinality + other.union_cardinality,
            }
        }
    }

    impl core::ops::Div<f64> for AbsoluteError {
        type Output = Self;

        fn div(self, divisor: f64) -> Self {
            Self {
                left_cardinality: self.left_cardinality / divisor,
                right_cardinality: self.right_cardinality / divisor,
                intersection_cardinality: self.intersection_cardinality / divisor,
                union_cardinality: self.union_cardinality / divisor,
            }
        }
    }

    fn evaluate<S: SetLike<usize>>(
        mut left_set_like: S,
        mut right_set_like: S,
        left: &Vec<usize>,
        right: &Vec<usize>,
    ) -> Cardinalities {
        for l in left.iter() {
            left_set_like.insert(l.clone());
        }

        for r in right.iter() {
            right_set_like.insert(r.clone());
        }

        let euc = left_set_like.cardinalities(&right_set_like);

        Cardinalities {
            left_cardinality: euc.get_left_cardinality(),
            right_cardinality: euc.get_right_cardinality(),
            intersection_cardinality: euc.get_intersection_cardinality(),
            union_cardinality: euc.get_union_cardinality(),
        }
    }

    fn get_random_vector(maximal_size: usize, random_state: &mut u64) -> Vec<usize> {
        (0..maximal_size)
            .map(|_| {
                *random_state = splitmix64(xorshift64(*random_state));
                *random_state as usize
            })
            .collect()
    }

    pub(super) fn test_quality_comparatively<
        P: Precision
            + ArrayRegister<Bits6>
            + ArrayMultiplicities<Bits6>
            + PrecisionConstants<f64>
            + ArrayRegister<Bits5>
            + ArrayMultiplicities<Bits5>,
    >() {
        let number_of_vectors = 100;
        let maximal_size = 1_000_000;
        let mut random_state = 6516781878233_u64;

        let hll5 = HyperLogLog::<P, Bits5, <P as ArrayRegister<Bits5>>::ArrayRegister>::default();
        let hll_multi5 = HLLMultiplicities::<
            P,
            Bits5,
            <P as ArrayRegister<Bits5>>::ArrayRegister,
            <P as ArrayMultiplicities<Bits5>>::ArrayMultiplicities,
        >::default();
        let mle5 = MLE::<
            3,
            HLLMultiplicities<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::ArrayRegister,
                <P as ArrayMultiplicities<Bits5>>::ArrayMultiplicities,
            >,
        >::default();

        let hll6 = HyperLogLog::<P, Bits6, <P as ArrayRegister<Bits6>>::ArrayRegister>::default();
        let hll_multi6 = HLLMultiplicities::<
            P,
            Bits6,
            <P as ArrayRegister<Bits6>>::ArrayRegister,
            <P as ArrayMultiplicities<Bits6>>::ArrayMultiplicities,
        >::default();
        let mle6 = MLE::<
            3,
            HLLMultiplicities<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::ArrayRegister,
                <P as ArrayMultiplicities<Bits6>>::ArrayMultiplicities,
            >,
        >::default();

        let errors = (0..number_of_vectors)
            .map(|_| {
                let left = get_random_vector(maximal_size, &mut random_state);
                let right = get_random_vector(maximal_size, &mut random_state);

                let exact = evaluate(HashSet::new(), HashSet::new(), &left, &right);

                let tabac_plus = evaluate(
                    TabacHyperLogLogPlus::new(P::EXPONENT as u8, RandomState::new()).unwrap(),
                    TabacHyperLogLogPlus::new(P::EXPONENT as u8, RandomState::new()).unwrap(),
                    &left,
                    &right,
                );

                let tabac_pf = evaluate(
                    TabacHyperLogLogPF::new(P::EXPONENT as u8, RandomState::new()).unwrap(),
                    TabacHyperLogLogPF::new(P::EXPONENT as u8, RandomState::new()).unwrap(),
                    &left,
                    &right,
                );

                let sa = evaluate(
                    SAHyperLogLog::new(P::error_rate()),
                    SAHyperLogLog::new(P::error_rate()),
                    &left,
                    &right,
                );

                let ours_hll5 = evaluate(hll5.clone(), hll5.clone(), &left, &right);
                let ours_hll_multi5 =
                    evaluate(hll_multi5.clone(), hll_multi5.clone(), &left, &right);
                let ours_mle5 = evaluate(mle5.clone(), mle5.clone(), &left, &right);
                let ours_hll6 = evaluate(hll6.clone(), hll6.clone(), &left, &right);
                let ours_hll_multi6 =
                    evaluate(hll_multi6.clone(), hll_multi6.clone(), &left, &right);
                let ours_mle6 = evaluate(mle6.clone(), mle6.clone(), &left, &right);

                vec![
                    AbsoluteError::from_cardinalities(&exact, &tabac_pf),
                    AbsoluteError::from_cardinalities(&exact, &tabac_plus),
                    AbsoluteError::from_cardinalities(&exact, &sa),
                    AbsoluteError::from_cardinalities(&exact, &ours_hll5),
                    AbsoluteError::from_cardinalities(&exact, &ours_hll_multi5),
                    AbsoluteError::from_cardinalities(&exact, &ours_mle5),
                    AbsoluteError::from_cardinalities(&exact, &ours_hll6),
                    AbsoluteError::from_cardinalities(&exact, &ours_hll_multi6),
                    AbsoluteError::from_cardinalities(&exact, &ours_mle6),
                ]
            })
            .collect::<Vec<_>>();

        for (library_name, index) in [("TabacPF", 0), ("TabacPlus", 1), ("StreamingAlgorithms", 2)]
        {
            let their_cardinalities = errors
                .iter()
                .map(|e| e[index])
                .map(|mae| (mae.left_cardinality + mae.right_cardinality) / 2.0)
                .collect::<Vec<_>>();
            let their_unions = errors
                .iter()
                .map(|e| e[index])
                .map(|mae| mae.union_cardinality)
                .collect::<Vec<_>>();
            let their_intersections = errors
                .iter()
                .map(|e| e[index])
                .map(|mae| mae.intersection_cardinality)
                .collect::<Vec<_>>();
            for (approach_name, inner_index) in [
                ("HLL5", 3),
                ("HLLMulti5", 4),
                ("MLE5", 5),
                ("HLL6", 6),
                ("HLLMulti6", 7),
                ("MLE6", 8),
            ] {
                let our_cardinalities = errors
                    .iter()
                    .map(|e| e[inner_index])
                    .map(|mae| (mae.left_cardinality + mae.right_cardinality) / 2.0)
                    .collect::<Vec<_>>();
                let our_unions = errors
                    .iter()
                    .map(|e| e[inner_index])
                    .map(|mae| mae.union_cardinality)
                    .collect::<Vec<_>>();
                let our_intersections = errors
                    .iter()
                    .map(|e| e[inner_index])
                    .map(|mae| mae.intersection_cardinality)
                    .collect::<Vec<_>>();

                for (feature_name, ours, theirs) in [
                    ("cardinalities", our_cardinalities, &their_cardinalities),
                    ("unions", our_unions, &their_unions),
                    ("intersections", our_intersections, &their_intersections),
                ] {
                    let w_test = WilcoxonWTest::paired(&ours, &theirs).unwrap();
                    // We check with a Wilcoxon signed-rank test if the difference between the
                    // two approaches is significant. If it is, we compare the mean absolute error
                    // of the two approaches. If the mean absolute error of the competitor library
                    // is lower, we fail the test.
                    if w_test.p_value() < 0.05 {
                        let our_mean = ours.iter().sum::<f64>() / ours.len() as f64;
                        let their_mean = theirs.iter().sum::<f64>() / theirs.len() as f64;
                        assert!(
                            our_mean <= their_mean,
                            "The MAE ({}) of the {} approach > than MAE ({}) of the {} approach for the '{}' feature (p-value {}).",
                            our_mean,
                            approach_name,
                            library_name,
                            their_mean,
                            feature_name,
                            w_test.p_value()
                        );
                    }
                }
            }
        }
    }
}

macro_rules! test_quality_comparatively_at_precision {
    ($( $precision: ty ),*) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_quality_comparatively_at_ $precision:lower>] () {
                    test_comparison::test_quality_comparatively::<$precision>();
                }
            }
        )*
    };
}

test_quality_comparatively_at_precision!(
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
