//! Optimizers for training neural networks or MLE estimators.

#[cfg(not(feature = "std"))]
use micromath::F32Ext;

pub struct Adam<const N: usize> {
    first_moments: [f32; N],
    second_moments: [f32; N],
    time: i32,
    learning_rate: f32,
    first_order_decay_factor: f32,
    second_order_decay_factor: f32,
}

impl<const N: usize> Default for Adam<N> {
    fn default() -> Self {
        Adam {
            first_moments: [0.0; N],
            second_moments: [0.0; N],
            time: 0,
            learning_rate: 0.001,
            first_order_decay_factor: 0.9,
            second_order_decay_factor: 0.999,
        }
    }
}

impl<const N: usize> Adam<N> {
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

pub struct SGD<const N: usize> {
    learning_rate: f32,
}

impl<const N: usize> SGD<N> {
    pub fn from_learning_rate_factor(learning_rate_factor: f32) -> Self {
        SGD {
            learning_rate: 0.01 * learning_rate_factor,
        }
    }

    #[inline(always)]
    pub fn apply(&self, gradients: &mut [f32; N], phis: &mut [f32; N]) {
        gradients
            .iter_mut()
            .zip(phis.iter_mut())
            .for_each(|(gradient, phi)| {
                *gradient *= self.learning_rate;
                *phi += *gradient;
            });
    }
}
