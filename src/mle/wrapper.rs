//! The [`Mle`] mode wrapper: a view over a [`HyperLogLog`] whose set estimates use the
//! maximum-likelihood estimators instead of the default HyperLogLog++ ones.
//!
//! Obtain it with [`HyperLogLog::mle`] and return to the default estimators with
//! [`Mle::into_inner`]. Because it implements [`CardinalityEstimator`] (and
//! [`HyperSpheresSketch`]), the derived intersection / Jaccard / difference estimates and the
//! overlap matrices come for free, all computed from the MLE primitives, and `Mle` can be passed to
//! any code generic over those traits.

use super::JointOptimizer;
use crate::estimator::CardinalityEstimator;
use crate::prelude::{Bits, HasherType, HyperLogLog, Precision, Registers};
use crate::sketches::{HyperSpheresSketch, JointSketch};

/// A maximum-likelihood-estimation view over a [`HyperLogLog`] (here a borrowed one, produced by
/// [`HyperLogLog::mle`]). See the module documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mle<H>(pub H);

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
    /// Returns a maximum-likelihood-estimation view over this counter. The view borrows the counter
    /// (no copy) and routes its estimates through the MLE estimators. Use [`Mle::into_inner`] to go
    /// back to the default estimators.
    #[inline]
    pub fn mle(&self) -> Mle<&Self> {
        Mle(self)
    }
}

impl<H> Mle<H> {
    /// Returns the wrapped counter (or reference), switching back from MLE to the default
    /// (HyperLogLog++) estimators, which the bare [`HyperLogLog`] provides.
    #[inline]
    pub fn into_inner(self) -> H {
        self.0
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> CardinalityEstimator
    for Mle<&HyperLogLog<P, B, R, H>>
{
    /// Estimates the cardinality via the single-counter maximum-likelihood estimator.
    ///
    /// # Warning
    /// This estimator is dominated by the default [`HyperLogLog::estimate_cardinality`]: it is both
    /// slower and less accurate. It exists for completeness and comparison. Prefer the default
    /// (i.e. `self.into_inner().estimate_cardinality()`) unless you specifically want the MLE value.
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        self.0.estimate_cardinality_mle()
    }

    #[inline]
    /// Estimates the union cardinality via the joint maximum-likelihood estimator.
    fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.0.estimate_union_cardinality_mle(other.0)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for Mle<&HyperLogLog<P, B, R, H>>
{
    #[inline]
    /// Overridden to use the joint MLE optimization (rather than the pairwise inclusion-exclusion
    /// default), unwrapping the views and delegating to the joint MLE sketch.
    fn joint_sketch<const L: usize, const N: usize>(
        lefts: &[Self; L],
        rights: &[Self; N],
    ) -> JointSketch<L, N> {
        let left_counters: [HyperLogLog<P, B, R, H>; L] =
            core::array::from_fn(|i| lefts[i].0.clone());
        let right_counters: [HyperLogLog<P, B, R, H>; N] =
            core::array::from_fn(|j| rights[j].0.clone());
        HyperLogLog::<P, B, R, H>::joint_sketch_mle::<L, N>(&left_counters, &right_counters)
    }
}

/// Sealed helper that lets [`JointSketch::estimate_with`] name only the optimizer while inferring the
/// underlying counter type. Implemented for [`Mle`] views over [`HyperLogLog`]; it is not meant to be
/// named or implemented downstream (pass `_` for it at the call site).
#[doc(hidden)]
pub trait MleJointSketch: Sized {
    /// Runs the optimizer-selected joint MLE over arrays of these views.
    fn joint_sketch_mle_with<O: JointOptimizer, const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> JointSketch<M, N>;
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> MleJointSketch
    for Mle<&HyperLogLog<P, B, R, H>>
{
    #[inline]
    fn joint_sketch_mle_with<O: JointOptimizer, const M: usize, const N: usize>(
        lefts: &[Self; M],
        rights: &[Self; N],
    ) -> JointSketch<M, N> {
        let left_counters: [HyperLogLog<P, B, R, H>; M] =
            core::array::from_fn(|i| lefts[i].0.clone());
        let right_counters: [HyperLogLog<P, B, R, H>; N] =
            core::array::from_fn(|j| rights[j].0.clone());
        HyperLogLog::<P, B, R, H>::joint_sketch_mle_with::<O, M, N>(&left_counters, &right_counters)
    }
}

impl<const M: usize, const N: usize> JointSketch<M, N> {
    /// Estimates the joint MLE sketch from [`mle`](HyperLogLog::mle) views with a caller-chosen
    /// optimizer type `O`, selected at compile time by turbofish. This is the power-user counterpart
    /// of [`JointSketch::estimate`], which uses the default `Chain<Adam, Lbfgs>`. Compose optimizers
    /// with [`Chain`](crate::prelude::Chain), or implement [`JointOptimizer`] for a custom strategy;
    /// [`Lbfgs`](crate::prelude::Lbfgs) alone is fastest where the objective is unimodal.
    ///
    /// The counter type is inferred from the operands, so only the optimizer is named (followed by an
    /// inference placeholder): `JointSketch::estimate_with::<Lbfgs, _>(..)`.
    ///
    /// # Examples
    /// ```
    /// use hyperloglog_rs::prelude::*;
    /// type Hll = HyperLogLog<Precision12, Bits6>;
    ///
    /// let mut a = Hll::default();
    /// let mut b = Hll::default();
    /// for x in 0u64..4_000 {
    ///     a.insert(&x);
    /// }
    /// for x in 2_000u64..6_000 {
    ///     b.insert(&x);
    /// }
    ///
    /// // Plain L-BFGS (the counter type and M, N are inferred).
    /// let sketch = JointSketch::estimate_with::<Lbfgs, _>(&[a.mle()], &[b.mle()]);
    /// assert!((sketch.union() - 6_000.0).abs() / 6_000.0 < 0.2);
    /// ```
    #[inline]
    pub fn estimate_with<O: JointOptimizer, V: MleJointSketch>(
        lefts: &[V; M],
        rights: &[V; N],
    ) -> Self {
        V::joint_sketch_mle_with::<O, M, N>(lefts, rights)
    }
}
