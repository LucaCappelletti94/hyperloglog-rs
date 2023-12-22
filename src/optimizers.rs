//! Optimizers for training neural networks or MLE estimators.
use crate::prelude::Primitive;

pub struct Adam<F, const N: usize> {
    first_moments: [F; N],
    second_moments: [F; N],
    time: i32,
    learning_rate: F,
    first_order_decay_factor: F,
    second_order_decay_factor: F,
}

impl<F: Default + Primitive<f32>, const N: usize> Default for Adam<F, N> {
    fn default() -> Self {
        Adam {
            first_moments: [F::default(); N],
            second_moments: [F::default(); N],
            time: 0,
            learning_rate: F::reverse(0.001),
            first_order_decay_factor: F::reverse(0.9),
            second_order_decay_factor: F::reverse(0.999),
        }
    }
}

impl<const N: usize> Adam<f32, N> {
    #[inline(always)]
    pub fn apply(&mut self, gradients: &mut [f32; N], phis: &mut [f32; N]) {
        self.time += 1;
        self.first_moments
            .iter_mut()
            .zip(self.second_moments.iter_mut())
            .zip(gradients.iter_mut().zip(phis.iter_mut()))
            .for_each(|((first_moment, second_moment), (gradient, phi))| {
                *first_moment = self.first_order_decay_factor * *first_moment
                    + (1.0 - self.first_order_decay_factor) * *gradient;
                *second_moment = self.second_order_decay_factor * *second_moment
                    + (1.0 - self.second_order_decay_factor) * (*gradient).powi(2);
                let adaptative_learning_rate = self.learning_rate
                    * (1.0 - self.second_order_decay_factor.powi(self.time)).sqrt()
                    / (1.0 - self.first_order_decay_factor.powi(self.time));
                *gradient = adaptative_learning_rate * (*first_moment)
                    / (*second_moment).sqrt().max(f32::EPSILON);
                *phi += *gradient;
            });
    }
}