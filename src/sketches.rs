//! Exact sketching algorithms.
//!
//! This submodule contains the implementation of the exact sketching algorithms
//! as part of a trait.
//!
//! A sketch is a representation of the similarity between two list of sets.
//!
//! It is used in cases such as in graphs for representing the similarity between
//! two nodes, de facto providing features that characterize a candidate edge
//! between two nodes.
//!
//! While in the [`HyperLogLog`] case we provide the approximated version of this algorithm,
//! sometimes it is necessary, such as in test cases, to have the exact version of the
//! algorithm. The approximated version is faster and uses less memory, but it is not,
//! of course, guaranteed to be exact.
use crate::prelude::{
    Bits, CardinalityEstimator, FloatOps, HasherType, HyperLogLog, Number, Precision, Registers,
    Zero,
};

/// The disjoint-cell cardinalities of a hypersphere sketch over `M` nested left sets and `N` nested
/// right sets: the `M*N` exclusive overlap grid plus the `M` left and `N` right margins. Build one
/// with [`JointSketch::estimate`] (or the trait method [`HyperSpheresSketch::joint_sketch`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointSketch<const M: usize, const N: usize> {
    /// `overlap[i][j] = |L_i intersect R_j|`, the exclusive overlap grid.
    pub overlap: [[f64; N]; M],
    /// `left_diff[i] = |L_i \ B_{N-1}|`, the left margins.
    pub left_diff: [f64; M],
    /// `right_diff[j] = |R_j \ A_{M-1}|`, the right margins.
    pub right_diff: [f64; N],
}

/// The per-cell theoretical standard error of a [`JointSketch`], same shape as the sketch it
/// accompanies: a standard error for every overlap cell and every margin. Produced by
/// [`HyperLogLog::joint_sketch_error`]. Each entry is one standard deviation of that cell's estimate
/// in absolute (cardinality) units, so the relative error of a cell is its standard error divided by
/// its estimated value. Small overlap cells between large sets carry a large relative error, which is
/// exactly what this surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointSketchError<const M: usize, const N: usize> {
    /// Standard error of each `overlap[i][j]` cell.
    pub overlap_se: [[f64; N]; M],
    /// Standard error of each `left_diff[i]` margin.
    pub left_diff_se: [f64; M],
    /// Standard error of each `right_diff[j]` margin.
    pub right_diff_se: [f64; N],
}

impl<const M: usize, const N: usize> JointSketch<M, N> {
    /// Estimates the joint sketch of `M` nested left counters and `N` nested right counters, in a
    /// single call. The estimator follows the *mode* of the operands, exactly like the scalar
    /// estimators: pass plain counters for the pairwise inclusion-exclusion estimate, or pass
    /// [`mle`](HyperLogLog::mle) views to run the joint maximum-likelihood optimization (which jointly
    /// fits every disjoint cell and is generally more accurate, at a higher cost). `M` and `N` are
    /// inferred from the arrays.
    ///
    /// To run the generalized joint maximum-likelihood optimization that fits every disjoint cell at
    /// once (rather than inclusion-exclusion over the 2-set MLE), pass [`jmle`](HyperLogLog::jmle)
    /// views instead.
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
    /// // Default pairwise inclusion-exclusion:
    /// let sketch = JointSketch::estimate(&[a], &[b]);
    /// assert!((sketch.union() - 6_000.0).abs() / 6_000.0 < 0.2);
    /// ```
    #[inline]
    pub fn estimate<E: HyperSpheresSketch>(lefts: &[E; M], rights: &[E; N]) -> Self {
        E::joint_sketch(lefts, rights)
    }

    /// The total union cardinality: the sum of every disjoint cell.
    #[inline]
    pub fn union(&self) -> f64 {
        self.overlap.iter().flatten().copied().sum::<f64>()
            + self.left_diff.iter().sum::<f64>()
            + self.right_diff.iter().sum::<f64>()
    }

    /// Decomposes into the raw `(overlap, left_diff, right_diff)` arrays.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> ([[f64; N]; M], [f64; M], [f64; N]) {
        (self.overlap, self.left_diff, self.right_diff)
    }

    /// The per-cell shell maxima: the largest value each differential cell could take given the
    /// reconstructed marginals. These are exactly the denominators [`normalize`](Self::normalize)
    /// divides by: for an overlap cell the maximal differential overlap (the increment of
    /// `|A \ B| + |B|` across the shell), for a margin the differential shell size
    /// (`|A_i| - |A_{i-1}|`). Returned as a `JointSketch` of the same shape. Reconstructed purely from
    /// the differential cells, so it works on any decomposition (estimated or exact).
    #[must_use]
    pub fn shell_maxima(&self) -> Self {
        // Cumulative left/right cardinalities and the intersection prefix sums, all reconstructed
        // from the differential cells: A_i holds every overlap cell in rows <= i plus the left
        // margins <= i; B_j symmetrically; |A_i intersect B_j| is the 2D prefix sum of the overlaps.
        let mut card_left = [f64::ZERO; M];
        for i in 0..M {
            let shell = self.overlap[i].iter().copied().sum::<f64>() + self.left_diff[i];
            card_left[i] = shell + if i > 0 { card_left[i - 1] } else { f64::ZERO };
        }
        let mut card_right = [f64::ZERO; N];
        for j in 0..N {
            let shell = (0..M).map(|i| self.overlap[i][j]).sum::<f64>() + self.right_diff[j];
            card_right[j] = shell + if j > 0 { card_right[j - 1] } else { f64::ZERO };
        }
        let mut inter = [[f64::ZERO; N]; M];
        for i in 0..M {
            for j in 0..N {
                let up = if i > 0 { inter[i - 1][j] } else { f64::ZERO };
                let left = if j > 0 { inter[i][j - 1] } else { f64::ZERO };
                let diag = if i > 0 && j > 0 {
                    inter[i - 1][j - 1]
                } else {
                    f64::ZERO
                };
                inter[i][j] = self.overlap[i][j] + up + left - diag;
            }
        }

        let cl = |i: isize| {
            if i < 0 {
                f64::ZERO
            } else {
                card_left[i as usize]
            }
        };
        let cr = |j: isize| {
            if j < 0 {
                f64::ZERO
            } else {
                card_right[j as usize]
            }
        };
        let it = |i: isize, j: isize| {
            if i < 0 || j < 0 {
                f64::ZERO
            } else {
                inter[i as usize][j as usize]
            }
        };

        let mut overlap = [[f64::ZERO; N]; M];
        for i in 0..M {
            for j in 0..N {
                // |A_i \ B_j| = |A_i| - |A_i intersect B_j|; the maximal differential overlap is the
                // increment of (|A \ B| + |B|) across this shell.
                let a_minus_b = cl(i as isize).saturating_zero_sub(it(i as isize, j as isize));
                let prev_a_minus_b =
                    cl(i as isize - 1).saturating_zero_sub(it(i as isize - 1, j as isize));
                overlap[i][j] = (a_minus_b + cr(j as isize))
                    .saturating_zero_sub(prev_a_minus_b + cr(j as isize - 1));
            }
        }
        let mut left_diff = [f64::ZERO; M];
        for i in 0..M {
            left_diff[i] = cl(i as isize).saturating_zero_sub(cl(i as isize - 1));
        }
        let mut right_diff = [f64::ZERO; N];
        for j in 0..N {
            right_diff[j] = cr(j as isize).saturating_zero_sub(cr(j as isize - 1));
        }
        Self {
            overlap,
            left_diff,
            right_diff,
        }
    }

    /// Converts this absolute decomposition into normalized shell fractions in `[0, 1]`, matching
    /// [`HyperSpheresSketch::normalized_joint_sketch`]: each cell divided by its
    /// [`shell_maxima`](Self::shell_maxima). Unlike `normalized_joint_sketch` (which reads the
    /// operands), this works on any [`JointSketch`] -- the joint-MLE output or an exact ground-truth
    /// decomposition. On the output of [`HyperSpheresSketch::joint_sketch`] it reproduces
    /// `normalized_joint_sketch` exactly.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let smax = self.shell_maxima();
        let mut overlap = [[f64::ZERO; N]; M];
        for i in 0..M {
            for j in 0..N {
                overlap[i][j] = self.overlap[i][j]
                    .max(f64::ZERO)
                    .saturating_one_div(smax.overlap[i][j]);
            }
        }
        let mut left_diff = [f64::ZERO; M];
        for i in 0..M {
            left_diff[i] = self.left_diff[i]
                .max(f64::ZERO)
                .saturating_one_div(smax.left_diff[i]);
        }
        let mut right_diff = [f64::ZERO; N];
        for j in 0..N {
            right_diff[j] = self.right_diff[j]
                .max(f64::ZERO)
                .saturating_one_div(smax.right_diff[j]);
        }
        Self {
            overlap,
            left_diff,
            right_diff,
        }
    }
}

/// The default pairwise inclusion-exclusion joint sketch: estimates each cumulative intersection
/// `|A_i intersect B_j|` from the marginal and union cardinalities, then differences them into the
/// disjoint cells. Shared by the [`HyperSpheresSketch`] default and the `HyperLogLog` override (which
/// first unifies the operand representation).
pub(crate) fn inclusion_exclusion_joint_sketch<
    E: CardinalityEstimator,
    const L: usize,
    const R: usize,
>(
    lefts: &[E; L],
    rights: &[E; R],
) -> JointSketch<L, R> {
    // Initialize overlap and differences cardinality matrices/vectors.
    let mut last_row = [f64::ZERO; R];
    let mut differential_overlap_cardinality_matrix = [[f64::ZERO; R]; L];
    let mut left_difference_cardinality_vector = [f64::ZERO; L];
    let mut right_cardinalities = [f64::ZERO; R];

    rights
        .iter()
        .zip(right_cardinalities.iter_mut())
        .for_each(|(right, right_cardinality)| {
            *right_cardinality = right.estimate_cardinality();
        });

    let mut right_difference_cardinality_vector = [f64::ZERO; R];
    let mut euc: EstimatedUnionCardinalities<f64> = EstimatedUnionCardinalities {
        left: f64::ZERO,
        right: f64::ZERO,
        union: f64::ZERO,
    };
    let mut last_left_difference = f64::ZERO;

    // Populate the overlap cardinality matrix.
    for (i, left) in lefts.iter().enumerate() {
        let mut last_right_difference = f64::ZERO;
        let left_cardinality = left.estimate_cardinality();
        let mut cumulative_row = f64::ZERO;
        for (j, (right, right_cardinality)) in rights.iter().zip(right_cardinalities).enumerate() {
            let union_cardinality = left.estimate_union_cardinality(right);
            euc = EstimatedUnionCardinalities {
                left: left_cardinality,
                right: right_cardinality,
                union: union_cardinality,
            };
            let delta = last_row[j] + cumulative_row;
            differential_overlap_cardinality_matrix[i][j] = euc
                .get_intersection_cardinality()
                .saturating_zero_sub(delta);
            last_row[j] = if euc.get_intersection_cardinality() > delta {
                euc.get_intersection_cardinality()
            } else {
                delta
            };

            cumulative_row += differential_overlap_cardinality_matrix[i][j];
            debug_assert!(cumulative_row >= f64::ZERO, "Expected cumulative_row to be larger than zero, but it is not. Got: cumulative_row: {cumulative_row:?}, delta: {delta:?}");

            // We always set the value of the right difference so that the last time we write this
            // will necessarily be with the last and largest left set.
            right_difference_cardinality_vector[j] = euc
                .get_right_difference_cardinality()
                .saturating_zero_sub(last_right_difference);

            last_right_difference = euc.get_right_difference_cardinality();
        }
        left_difference_cardinality_vector[i] = euc
            .get_left_difference_cardinality()
            .saturating_zero_sub(last_left_difference);
        last_left_difference = euc.get_left_difference_cardinality();
    }

    JointSketch {
        overlap: differential_overlap_cardinality_matrix,
        left_diff: left_difference_cardinality_vector,
        right_diff: right_difference_cardinality_vector,
    }
}

/// Wires `HyperLogLog` to the approximate sketching algorithms. The required cardinality and union
/// estimators come from its [`CardinalityEstimator`] implementation.
impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperSpheresSketch
    for HyperLogLog<P, B, R, H>
{
    #[inline]
    fn joint_sketch<const L: usize, const N: usize>(
        lefts: &[Self; L],
        rights: &[Self; N],
    ) -> JointSketch<L, N> {
        // If the nested operands span different representations (some still hash lists, some already
        // registers, as happens when a chain straddles the saturation transition), the differential
        // decomposition mixes near-exact and probabilistic cell estimates and amplifies the mismatch
        // into a spike. Materialize everything to registers first so all cells are consistent. This
        // is a no-op once every operand is already dense, and is skipped entirely while they are all
        // still hash lists (where the set algebra is near-exact).
        let any_dense = lefts
            .iter()
            .chain(rights.iter())
            .any(|counter| counter.is_hyperloglog());
        if any_dense {
            let lefts: [Self; L] = core::array::from_fn(|i| lefts[i].clone().into_hll());
            let rights: [Self; N] = core::array::from_fn(|j| rights[j].clone().into_hll());
            inclusion_exclusion_joint_sketch(&lefts, &rights)
        } else {
            inclusion_exclusion_joint_sketch(lefts, rights)
        }
    }
}

/// Trait for sketching algorithms that provide the overlap and differences cardinality matrices.
/// The required cardinality and union estimators are inherited from [`CardinalityEstimator`].
pub trait HyperSpheresSketch: CardinalityEstimator + Sized {
    #[inline]
    /// Returns the overlap and differences cardinality matrices of two lists of sets.
    ///
    /// # Arguments
    /// * `left` - The first list of sets.
    /// * `right` - The second list of sets.
    ///
    /// # Returns
    /// * `overlap_cardinality_matrix` - Matrix of estimated overlapping cardinalities between the elements of the left and right arrays.
    /// * `left_difference_cardinality_vector` - Vector of estimated difference cardinalities between the elements of the left array and the last element of the right array.
    /// * `right_difference_cardinality_vector` - Vector of estimated difference cardinalities between the elements of the right array and the last element of the left array.
    ///
    /// # Implementative details
    /// We expect the elements of the left and right arrays to be increasingly contained in the next one.
    ///
    /// # Examples
    /// In the following illustration, we show that for two vectors left and right of three elements,
    /// we expect to compute the exclusively overlap matrix $A_{ij}$ and the exclusively differences vectors $`B_i`$.    
    ///
    /// ![Illustration of overlaps](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/triple_overlap.png?raw=true)
    ///
    /// Very similarly, for the case of vectors of two elements:
    ///
    /// ![Illustration of overlaps](https://github.com/LucaCappelletti94/hyperloglog-rs/blob/main/tuple_overlap.png?raw=true)
    fn joint_sketch<const L: usize, const R: usize>(
        lefts: &[Self; L],
        rights: &[Self; R],
    ) -> JointSketch<L, R> {
        inclusion_exclusion_joint_sketch(lefts, rights)
    }

    #[inline]
    /// Returns the normalized overlap and differences cardinality matrices of two lists of sets.
    ///
    /// # Arguments
    /// * `left` - The first list of sets.
    /// * `right` - The second list of sets.
    ///
    /// # Returns
    /// * `overlap_cardinality_matrix` - Matrix of normalized estimated overlapping cardinalities between the elements of the left and right arrays.
    /// * `left_difference_cardinality_vector` - Vector of normalized estimated difference cardinalities between the elements of the left array and the last element of the right array.
    /// * `right_difference_cardinality_vector` - Vector of normalized estimated difference cardinalities between the elements of the right array and the last element of the left array.
    fn normalized_joint_sketch<const L: usize, const R: usize>(
        lefts: &[Self; L],
        rights: &[Self; R],
    ) -> JointSketch<L, R> {
        // Initialize overlap and differences cardinality matrices/vectors.
        let mut last_row = [f64::ZERO; R];
        let mut differential_overlap_cardinality_matrix = [[f64::ZERO; R]; L];
        let mut left_difference_cardinality_vector = [f64::ZERO; L];
        let mut right_cardinalities = [f64::ZERO; R];

        rights
            .iter()
            .zip(right_cardinalities.iter_mut())
            .for_each(|(right, right_cardinality)| {
                *right_cardinality = right.estimate_cardinality();
            });

        // We run a debug assert where we check that each right cardinality is
        // larger than the previous one.
        debug_assert!(
            right_cardinalities
                .iter()
                .zip(right_cardinalities.iter().skip(1))
                .all(|(left, right)| left <= right),
            "The right cardinalities should be sorted in ascending order."
        );

        let mut right_difference_cardinality_vector = [f64::ZERO; R];
        let mut euc: EstimatedUnionCardinalities<f64> = EstimatedUnionCardinalities {
            left: f64::ZERO,
            right: f64::ZERO,
            union: f64::ZERO,
        };
        let mut last_left_difference = f64::ZERO;
        let mut last_inner_left_differences = [f64::ZERO; R];
        let mut last_left_cardinality = f64::ZERO;

        // Populate the overlap cardinality matrix.
        for (i, left) in lefts.iter().enumerate() {
            let mut last_right_difference = f64::ZERO;
            let left_cardinality = left.estimate_cardinality();
            let mut cumulative_row = f64::ZERO;
            let mut last_right_cardinality = f64::ZERO;
            for (j, (right, (right_cardinality, last_inner_left_difference))) in rights
                .iter()
                .zip(
                    right_cardinalities
                        .iter()
                        .copied()
                        .zip(last_inner_left_differences.iter_mut()),
                )
                .enumerate()
            {
                let union_cardinality = left.estimate_union_cardinality(right);
                euc = EstimatedUnionCardinalities {
                    left: left_cardinality,
                    right: right_cardinality,
                    union: union_cardinality,
                };
                let delta = last_row[j] + cumulative_row;
                let differential_intersection = euc
                    .get_intersection_cardinality()
                    .saturating_zero_sub(delta);

                debug_assert!(
                    differential_intersection >= f64::ZERO,
                    concat!(
                        "Expected differential_intersection to be larger than zero, but it is not. ",
                        "Got: differential_intersection: {:?}, delta: {:?}",
                    ),
                    differential_intersection,
                    delta,
                );

                let maximal_differential_intersection_cardinality =
                    (euc.get_left_difference_cardinality() + right_cardinality)
                        .saturating_zero_sub(*last_inner_left_difference + last_right_cardinality);
                *last_inner_left_difference = euc.get_left_difference_cardinality();

                differential_overlap_cardinality_matrix[i][j] = differential_intersection
                    .saturating_one_div(maximal_differential_intersection_cardinality);
                last_row[j] = if euc.get_intersection_cardinality() > delta {
                    euc.get_intersection_cardinality()
                } else {
                    delta
                };
                cumulative_row += differential_intersection;

                // We always set the value of the right difference so that the
                // last time we write this will necessarily be with the last
                // and largest left set.

                let differential_right_difference = euc
                    .get_right_difference_cardinality()
                    .saturating_zero_sub(last_right_difference);

                right_difference_cardinality_vector[j] = differential_right_difference
                    .saturating_one_div(
                        right_cardinality.saturating_zero_sub(last_right_cardinality),
                    );
                last_right_difference = euc.get_right_difference_cardinality();
                last_right_cardinality = right_cardinality;
            }
            left_difference_cardinality_vector[i] = euc
                .get_left_difference_cardinality()
                .saturating_zero_sub(last_left_difference)
                .saturating_one_div(left_cardinality.saturating_zero_sub(last_left_cardinality));
            last_left_cardinality = left_cardinality;
            last_left_difference = euc.get_left_difference_cardinality();
        }

        JointSketch {
            overlap: differential_overlap_cardinality_matrix,
            left_diff: left_difference_cardinality_vector,
            right_diff: right_difference_cardinality_vector,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct for more readable code.
struct EstimatedUnionCardinalities<F> {
    /// The estimated cardinality of the left set.
    left: F,
    /// The estimated cardinality of the right set.
    right: F,
    /// The estimated cardinality of the union of the two sets.
    union: F,
}

#[cfg(test)]
mod normalize_tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_normalize_known_cells() {
        // M=N=2 absolute decomposition with hand-chosen consistent cells.
        let s = JointSketch::<2, 2> {
            overlap: [[40.0, 10.0], [20.0, 30.0]],
            left_diff: [100.0, 50.0],
            right_diff: [80.0, 60.0],
        };
        let n = s.normalize();
        for v in n
            .overlap
            .iter()
            .flatten()
            .chain(&n.left_diff)
            .chain(&n.right_diff)
        {
            assert!((0.0..=1.0).contains(v), "fraction out of range: {v}");
        }
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        // Worked out by hand from the cumulative marginals (see `normalize`).
        assert!(close(n.overlap[0][0], 40.0 / 250.0));
        assert!(close(n.overlap[1][1], 30.0 / 150.0));
        assert!(close(n.left_diff[0], 100.0 / 150.0));
        assert!(close(n.left_diff[1], 50.0 / 100.0));
        assert!(close(n.right_diff[0], 80.0 / 140.0));
        assert!(close(n.right_diff[1], 60.0 / 100.0));
    }

    #[test]
    fn test_normalize_matches_normalized_joint_sketch() {
        type Hll = HyperLogLog<Precision10, Bits6>;
        let build = |ranges: &[core::ops::Range<u64>]| {
            let mut h = Hll::default();
            for r in ranges {
                for v in r.clone() {
                    h.insert(&v);
                }
            }
            h
        };
        // Nested counters (a0 subset a1, b0 subset b1) with genuine overlap, kept small enough to
        // stay non-HyperLogLog so the cardinality/union estimates are exact and the two paths must agree.
        let a0 = build(&[0..30]);
        let a1 = build(&[0..30, 30..55, 100..115]);
        let b0 = build(&[20..50]);
        let b1 = build(&[20..50, 30..70, 200..210]);
        assert!(!a1.is_hyperloglog() && !b1.is_hyperloglog());
        let lefts = [a0, a1];
        let rights = [b0, b1];

        let reconstructed = Hll::joint_sketch(&lefts, &rights).normalize();
        let reference = Hll::normalized_joint_sketch(&lefts, &rights);
        // The two paths are algebraically identical but reach the per-shell denominators differently:
        // `normalize` rebuilds the marginal cardinalities by cumulative summation of the overlap
        // matrix, while `normalized_joint_sketch` computes them inline. They agree to floating point
        // only when the cardinality estimates are exact integers. The occupancy hash-list estimator
        // predicts a tiny collision deficit even at the wide composite width (order n^2 / 2^w, a few
        // parts per million for these small counters), so the inputs are not exact integers and the
        // two reconstructions diverge under cancellation by that same order. This is far below the
        // estimator's own noise floor.
        let close = |a: f64, b: f64| (a - b).abs() <= 1e-5 * a.abs().max(b.abs()).max(1.0);
        for i in 0..2 {
            for j in 0..2 {
                assert!(
                    close(reconstructed.overlap[i][j], reference.overlap[i][j]),
                    "overlap[{i}][{j}]: {} vs {}",
                    reconstructed.overlap[i][j],
                    reference.overlap[i][j]
                );
            }
            assert!(close(reconstructed.left_diff[i], reference.left_diff[i]));
            assert!(close(reconstructed.right_diff[i], reference.right_diff[i]));
        }
    }
}

impl<F: Number> EstimatedUnionCardinalities<F> {
    /// Returns the estimated cardinality of the intersection of the two sets.
    fn get_intersection_cardinality(&self) -> F {
        let intersection = self.left + self.right - self.union;
        debug_assert!(
            intersection >= F::ZERO,
            "Expected intersection to be larger than zero, but it is not. Got: intersection: {intersection:?}."
        );
        intersection
    }

    /// Returns the estimated cardinality of the left set minus the right set.
    fn get_left_difference_cardinality(&self) -> F {
        self.union - self.right
    }

    /// Returns the estimated cardinality of the right set minus the left set.
    fn get_right_difference_cardinality(&self) -> F {
        self.union - self.left
    }
}
