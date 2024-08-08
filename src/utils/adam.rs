//! Implementation of the adam optimizer.

use crate::utils::FloatNumber;

pub(crate) struct Adam<F, const N: usize> {
    first_moments: [F; N],
    second_moments: [F; N],
    time: i32,
    learning_rate: F,
    first_order_decay_factor: F,
    second_order_decay_factor: F,
}

impl<F: FloatNumber, const N: usize> Default for Adam<F, N> {
    fn default() -> Self {
        Adam {
            first_moments: [F::ZERO; N],
            second_moments: [F::ZERO; N],
            time: 0,
            learning_rate: F::ONE / F::ONE_THOUSAND,
            first_order_decay_factor: F::from_usize(9) / F::TEN,
            second_order_decay_factor: F::from_usize(999) / F::ONE_THOUSAND,
        }
    }
}

impl<F: FloatNumber, const N: usize> Adam<F, N> {
    #[inline(always)]
    pub(crate) fn apply(&mut self, gradients: &mut [F; N], phis: &mut [F; N]) {
        self.time += 1;
        self.first_moments
            .iter_mut()
            .zip(self.second_moments.iter_mut())
            .zip(gradients.iter_mut().zip(phis.iter_mut()))
            .for_each(|((first_moment, second_moment), (gradient, phi))| {
                *first_moment = self.first_order_decay_factor * *first_moment
                    + (F::ONE - self.first_order_decay_factor) * *gradient;
                *second_moment = self.second_order_decay_factor * *second_moment
                    + (F::ONE - self.second_order_decay_factor) * (*gradient).powi(2);
                let adaptative_learning_rate = self.learning_rate
                    * (F::ONE - self.second_order_decay_factor.powi(self.time)).sqrt()
                    / (F::ONE - self.first_order_decay_factor.powi(self.time));
                let second_moment_root = (*second_moment).sqrt();
                *gradient = adaptative_learning_rate * (*first_moment)
                    / if second_moment_root > F::EPSILON {
                        second_moment_root
                    } else {
                        F::EPSILON
                    };
                *phi += *gradient;
            });
    }
}
