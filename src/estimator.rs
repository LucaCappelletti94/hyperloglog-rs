//! The [`CardinalityEstimator`] trait: the HyperLogLog-specific estimator interface.
//!
//! It extends the shared [`sketching_core::CardinalityEstimator`] trait (which provides
//! `estimate_cardinality`, `estimate_union_cardinality`, and the derived intersection, Jaccard, and
//! difference estimates) with HyperLogLog-specific error and bias analysis methods.
//!
//! It is implemented by [`HyperLogLog`] (using its default, `HyperLogLog`++ corrected estimators) and
//! by the [`Mle`](crate::mle::Mle) mode wrapper (using the maximum-likelihood estimators).

use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use sketching_core::sparse_value_list::SparseValueCodec;

/// Estimates set cardinalities with HyperLogLog-specific error and bias analysis.
///
/// Extends [`sketching_core::CardinalityEstimator`] to add methods for predicting the theoretical
/// relative standard error and systematic bias of the register estimator. The base trait provides
/// `estimate_cardinality`, `estimate_union_cardinality`, and the derived intersection, Jaccard, and
/// difference estimates.
pub trait HllCardinalityEstimator: sketching_core::CardinalityEstimator {
    /// The theoretical relative standard error (the sampling error, one standard deviation as a
    /// fraction of the cardinality) of [`estimate_cardinality`](sketching_core::CardinalityEstimator::estimate_cardinality) at this
    /// counter's current estimate. This is the variance term only; the systematic bias is reported
    /// separately by [`predicted_bias`](Self::predicted_bias).
    fn predicted_relative_standard_error(&self) -> f64;

    /// The theoretical systematic relative bias of the register estimator, evaluated at this counter's
    /// current estimate. It models the raw `alpha * m^2 / harmonic_sum` estimator, so it is meaningful
    /// in the saturation regime (above the correction bound `7.5 * 2^P`, where
    /// [`estimate_cardinality`](sketching_core::CardinalityEstimator::estimate_cardinality) returns the raw value), and overstates the
    /// true bias below that bound, where the empirical `HyperLogLog`++ correction has already removed
    /// most of it. It is zero for the exact value-list and the unbiased hash-list representations.
    /// Because bias is a function of the true (unknown) cardinality, this evaluates the model at the
    /// current estimate; prefer [`bias_at`](Self::bias_at) with a known cardinality for planning.
    fn predicted_bias(&self) -> f64;

    /// The theoretical relative standard error at an arbitrary `cardinality`, using the dense-register
    /// model (the regime where error analysis is meaningful). Takes `&self` only to select the
    /// estimation method and the precision and bits of the type.
    fn relative_standard_error_at(&self, cardinality: f64) -> f64;

    /// The theoretical systematic relative bias at an arbitrary true `cardinality`, using the
    /// dense-register model. Takes `&self` only to select the estimation method and the precision and
    /// bits of the type.
    fn bias_at(&self, cardinality: f64) -> f64;
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C>
    sketching_core::CardinalityEstimator for HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        // Resolves to the inherent method (inherent methods take precedence over trait methods), so
        // this delegates to the default HyperLogLog++ estimate rather than recursing.
        self.estimate_cardinality()
    }

    #[inline]
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.estimate_union_cardinality(other)
    }

    #[inline]
    fn estimate_intersection_cardinality(&self, other: &Self) -> f64 {
        // Prefer the width-consistent estimate (all terms at the union's width) when both operands are
        // hash lists. It falls back to the default inclusion-exclusion otherwise.
        if let Some((intersection, _union)) = self.width_consistent_intersection_and_union(other) {
            return intersection;
        }
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality(other);
        (self_cardinality + other_cardinality - union_cardinality).max(0.0)
    }

    #[inline]
    fn estimate_jaccard_index(&self, other: &Self) -> f64 {
        // Width-consistent Jaccard: intersection and union both taken at the union's width, so the
        // occupancy correction shares one width regime and the estimate does not spike at the
        // composite-width boundary. Falls back to the default inclusion-exclusion otherwise.
        if let Some((intersection, union)) = self.width_consistent_intersection_and_union(other) {
            if union == 0.0 {
                return 0.0;
            }
            return (intersection / union).clamp(0.0, 1.0);
        }
        let self_cardinality = self.estimate_cardinality();
        let other_cardinality = other.estimate_cardinality();
        let union_cardinality = self.estimate_union_cardinality(other);
        if self_cardinality + other_cardinality < union_cardinality || union_cardinality == 0.0 {
            0.0
        } else {
            ((self_cardinality + other_cardinality - union_cardinality) / union_cardinality)
                .clamp(0.0, 1.0)
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HllCardinalityEstimator
    for HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    #[inline]
    fn predicted_relative_standard_error(&self) -> f64 {
        // A sorted value list is an exact count, so it has no sampling error.
        if self.is_sorted_value_list() {
            return 0.0;
        }
        // A sorted hash list is corrected by the occupancy model; its error follows from it.
        if self.is_sorted_hash_list() {
            return Self::hash_list_relative_standard_error(
                self.get_number_of_hashes().unwrap(),
                self.get_hash_bits().unwrap(),
            );
        }
        crate::error_model::register_default_relative_standard_error::<P>()
    }

    #[inline]
    fn predicted_bias(&self) -> f64 {
        // Only the raw register estimator carries a non-negligible bias (near saturation). The value
        // list is exact and the hash-list occupancy inverse is unbiased in expectation.
        if self.is_hyperloglog() {
            crate::error_model::register_raw_bias::<P, B>(self.estimate_cardinality())
        } else {
            0.0
        }
    }

    #[inline]
    fn relative_standard_error_at(&self, _cardinality: f64) -> f64 {
        // The default estimator's variance is flat across its operating range; its saturation error is
        // the bias, see `bias_at`. The `cardinality` argument is unused for this method.
        crate::error_model::register_default_relative_standard_error::<P>()
    }

    #[inline]
    fn bias_at(&self, cardinality: f64) -> f64 {
        crate::error_model::register_raw_bias::<P, B>(cardinality)
    }
}

#[cfg(test)]
mod width_consistent_set_ops {
    //! Tests for width-consistent union, intersection and Jaccard estimates. The regression target is
    //! the "square" in the Jaccard error at the composite-width boundary: when the operands are at a
    //! finer width than their union, inclusion-exclusion mixes corrections from two width regimes and
    //! the Jaccard error spikes. A width-consistent estimate keeps all three cardinalities at the
    //! union's width and removes the spike.
    use crate::prelude::*;
    use sketching_core::CardinalityEstimator;

    type Hll = HyperLogLog<Precision10, Bits6>;

    /// Build two hash-list counters with `shared` common elements, `only_a` unique to A and `only_b`
    /// unique to B, using disjoint element ranges per seed. So the exact union is
    /// `shared + only_a + only_b`, the intersection is `shared`, and the Jaccard index is
    /// `shared / (shared + only_a + only_b)`.
    fn build_pair(seed: u64, shared: u64, only_a: u64, only_b: u64) -> (Hll, Hll) {
        let mut a = Hll::default();
        let mut b = Hll::default();
        let mut idx = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
        for _ in 0..shared {
            a.insert(&idx);
            b.insert(&idx);
            idx = idx.wrapping_add(1);
        }
        for _ in 0..only_a {
            a.insert(&idx);
            idx = idx.wrapping_add(1);
        }
        for _ in 0..only_b {
            b.insert(&idx);
            idx = idx.wrapping_add(1);
        }
        (a, b)
    }

    /// The corrected Jaccard error must stay near its plateau across the union range that straddles
    /// the minimum composite width, not spike into a "square". At a fixed true Jaccard of `1/3` the
    /// operands hold `2s` elements and the union `3s`, so the union crosses the minimum width well
    /// before the operands do. The width-inconsistent inclusion-exclusion peaks at about `4.2%` there,
    /// while a width-consistent estimate stays near `2.5%`.
    #[test]
    fn jaccard_has_no_width_boundary_square() {
        const SEEDS: u64 = 100;
        let truth = 1.0 / 3.0;
        for s in (100..=460).step_by(30) {
            let union = 3 * s;
            let mut mare = 0.0;
            for seed in 0..SEEDS {
                let (a, b) = build_pair(seed, s, s, s);
                assert!(
                    a.is_sorted_hash_list() && b.is_sorted_hash_list(),
                    "operands must be hash lists (s={s}) to exercise the path under test",
                );
                let j = a.estimate_jaccard_index(&b);
                mare += (j - truth).abs() / truth;
            }
            mare /= SEEDS as f64;
            assert!(
                mare < 0.03,
                "union={union} (operands {}): Jaccard MARE {mare:.4} exceeds the plateau, the width-boundary square is present",
                2 * s,
            );
        }
    }

    /// The corrected intersection error at the same width boundary, for the same reason.
    #[test]
    fn intersection_has_no_width_boundary_square() {
        const SEEDS: u64 = 100;
        for s in (100..=460).step_by(30) {
            let truth = s as f64;
            let mut mare = 0.0;
            for seed in 0..SEEDS {
                let (a, b) = build_pair(seed, s, s, s);
                let i = a.estimate_intersection_cardinality(&b);
                mare += (i - truth).abs() / truth;
            }
            mare /= SEEDS as f64;
            assert!(
                mare < 0.05,
                "s={s}: intersection MARE {mare:.4} exceeds the plateau at the width boundary",
            );
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig { cases: 128, ..Default::default() })]

        /// The Jaccard estimate is always a valid index in `[0, 1]`.
        #[test]
        fn jaccard_in_unit_interval(
            shared in 0u64..800,
            only_a in 0u64..800,
            only_b in 0u64..800,
            seed in 0u64..1_000_000,
        ) {
            let (a, b) = build_pair(seed, shared, only_a, only_b);
            let j = a.estimate_jaccard_index(&b);
            proptest::prop_assert!((0.0..=1.0).contains(&j), "jaccard {j} out of [0, 1]");
        }

        /// A counter's Jaccard index with itself is exactly one (its union with itself is itself).
        #[test]
        fn self_jaccard_is_one(
            only_a in 1u64..900,
            seed in 0u64..1_000_000,
        ) {
            let (a, _) = build_pair(seed, 0, only_a, 0);
            let j = a.estimate_jaccard_index(&a);
            proptest::prop_assert!((j - 1.0).abs() < 1e-9, "self jaccard {j} is not one");
        }

        /// The estimate is symmetric in its operands.
        #[test]
        fn jaccard_is_symmetric(
            shared in 0u64..600,
            only_a in 0u64..600,
            only_b in 0u64..600,
            seed in 0u64..1_000_000,
        ) {
            let (a, b) = build_pair(seed, shared, only_a, only_b);
            let ab = a.estimate_jaccard_index(&b);
            let ba = b.estimate_jaccard_index(&a);
            proptest::prop_assert!((ab - ba).abs() < 1e-9, "asymmetric: {ab} vs {ba}");
        }

        /// The estimated intersection never exceeds the smaller estimated operand and is non-negative.
        #[test]
        fn intersection_within_bounds(
            shared in 0u64..600,
            only_a in 0u64..600,
            only_b in 0u64..600,
            seed in 0u64..1_000_000,
        ) {
            let (a, b) = build_pair(seed, shared, only_a, only_b);
            let i = a.estimate_intersection_cardinality(&b);
            let ca = a.estimate_cardinality();
            let cb = b.estimate_cardinality();
            proptest::prop_assert!(i >= 0.0, "negative intersection {i}");
            proptest::prop_assert!(i <= ca.min(cb) + 1e-6, "intersection {i} exceeds min operand {}", ca.min(cb));
        }
    }
}
