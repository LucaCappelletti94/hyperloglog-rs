use crate::prelude::HyperLogLogTrait;
use crate::prelude::*;
use crate::utils::*;
use core::hash::Hash;

#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A probabilistic algorithm for estimating the number of distinct elements in a set.
///
/// HyperLogLog is a probabilistic algorithm designed to estimate the number
/// of distinct elements in a set. It does so by taking advantage of the fact
/// that the representation of an element can be transformed into a uniform
/// distribution in a space with a fixed range.
///
/// HyperLogLog works by maintaining a fixed-sized register array,
/// where each register holds a counter. The algorithm splits the input set into subsets,
/// applies a hash function to each element in the subset, and then updates
/// the corresponding counter in the register array.
///
/// HyperLogLog uses a trick called "probabilistic counting" to estimate
/// the number of distinct elements in the set. Each register counter is converted
/// to a binary string, and the algorithm counts the number of leading zeros in
/// each binary string. The maximum number of leading zeros over all counters
/// is used to estimate the number of distinct elements in the set.
///
/// HyperLogLog has a tunable parameter called precision that determines
/// the accuracy of the algorithm. Higher precision leads to better accuracy,
/// but requires more memory. The error rate of the algorithm is guaranteed
/// to be within a certain bound, depending on the chosen precision.
///
/// # Examples
///
/// ```
/// use hyperloglog_rs::prelude::*;
///
/// let mut hll = HyperLogLog::<
///     Precision12,
///     Bits6,
///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
/// >::default();
/// hll.insert(&"apple");
/// hll.insert(&"banana");
/// hll.insert(&"cherry");
///
/// let estimated_cardinality: f32 = hll.estimate_cardinality();
/// assert!(estimated_cardinality >= 3.0_f32 * 0.9 && estimated_cardinality <= 3.0_f32 * 1.1);
/// ```
///
/// # Citations
///
/// This implementation is based on the following papers:
///
/// * Flajolet, Philippe, et al. "HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm." DMTCS Proceedings 1 (2007): 127-146.
/// * Heule, Stefan, Marc Nunkesser, and Alexander Hall. "HyperLogLog in practice: algorithmic engineering of a state of the art cardinality estimation algorithm." Proceedings of the 16th International Conference on Extending Database Technology. 2013.
pub struct HyperLogLog<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    Hasher: core::hash::Hasher + Default = twox_hash::XxHash64,
> {
    registers: R,
    number_of_zero_registers: P::NumberOfZeros,
    harmonic_sum: f64,
    _phantom: core::marker::PhantomData<(P, B, Hasher)>,
}

/// Implementation of partial equality for HyperLogLog so as to compare two HyperLogLog instances
/// ignoring the harmonic sum.
impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default> PartialEq
    for HyperLogLog<P, B, R, Hasher>
{
    fn eq(&self, other: &Self) -> bool {
        self.registers == other.registers
    }
}

/// Implementation of equality for HyperLogLog so as to compare two HyperLogLog instances
/// ignoring the harmonic sum.
impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default> Eq
    for HyperLogLog<P, B, R, Hasher>
{
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default>
    HyperLogLog<P, B, R, Hasher>
{
    /// Create a new HyperLogLog counter.
    fn new() -> Self {
        Self {
            registers: R::zeroed(),
            number_of_zero_registers: unsafe {
                P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked()
            },
            harmonic_sum: P::NUMBER_OF_REGISTERS as f64,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a new HyperLogLog counter from an array of registers.
    ///
    /// # Arguments
    ///
    /// * `registers` - An array of u32 registers to use for the HyperLogLog counter.
    ///
    /// # Returns
    ///
    /// A new HyperLogLog counter initialized with the given registers.
    pub fn from_registers(registers: R) -> Self {
        let (number_of_zero_registers, harmonic_sum) = registers
            .iter_registers()
            .map(|register| {
                (
                    P::NumberOfZeros::from_bool(register.is_zero()),
                    f64::inverse_register(register as i32),
                )
            })
            .fold(
                (P::NumberOfZeros::ZERO, 0.0),
                |(acc_zeros, acc_sum), (zeros, sum)| (acc_zeros + zeros, acc_sum + sum),
            );

        Self {
            registers,
            number_of_zero_registers,
            harmonic_sum,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default> Default
    for HyperLogLog<P, B, R, Hasher>
{
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default>
    HyperLogLogTrait<P, B, Hasher> for HyperLogLog<P, B, R, Hasher>
{
    type Registers = R;

    #[inline(always)]
    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = HyperLogLog::<
    ///     Precision14,
    ///     Bits5,
    ///     <Precision14 as ArrayRegister<Bits5>>::ArrayRegister,
    /// >::default();
    ///
    /// // Add some elements to the counter.
    /// hll.insert(&1);
    /// hll.insert(&2);
    /// hll.insert(&3);
    ///
    /// // Get the number of zero registers.
    /// let number_of_zero_registers = hll.get_number_of_zero_registers();
    ///
    /// assert_eq!(number_of_zero_registers, 16381);
    /// ```
    fn get_number_of_zero_registers(&self) -> P::NumberOfZeros {
        self.number_of_zero_registers
    }

    #[inline(always)]
    /// Returns a reference to the registers of the HyperLogLog counter.
    fn registers(&self) -> &Self::Registers {
        &self.registers
    }

    #[inline(always)]
    /// Returns the harmonic sum of the registers.
    fn harmonic_sum<F: FloatNumber>(&self) -> F {
        F::from_f64(self.harmonic_sum)
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        // The harmonic sum is defined as Sum(2^(-register_value)) for all registers.
        // When all registers are maximally filled, i.e. equal to the maximal multiplicity value,
        // the harmonic sum is equal to (2^(-max_multiplicity)) * number_of_registers.
        // Since number_of_registers is a power of 2, specifically 2^exponent, the harmonic sum
        // is equal to 2^(exponent - max_multiplicity).
        self.harmonic_sum <= miminal_harmonic_sum::<P, B>()
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter, and returns whether the counter has changed.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HyperLogLog::<
    ///     Precision10,
    ///     Bits6,
    ///     <Precision10 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    ///
    /// hll.insert("Hello");
    /// hll.insert("World");
    ///
    /// assert!(hll.estimate_cardinality::<f32>() >= 2.0);
    /// ```
    ///
    /// # Performance
    ///
    /// The performance of this function depends on the size of the HyperLogLog counter (`N`), the number
    /// of distinct elements in the input, and the hash function used to hash elements. For a given value of `N`,
    /// the function has an average time complexity of O(1) and a worst-case time complexity of O(log N).
    /// However, the actual time complexity may vary depending on the distribution of the hashed elements.
    ///
    /// # Errors
    ///
    /// This function does not return any errors.
    fn insert<T: Hash>(&mut self, rhs: T) -> bool {
        let (hash, index) = Self::hash_and_index::<T>(&rhs);

        // Count leading zeros.
        let new_register_value: u32 = 1 + hash.leading_zeros();
        debug_assert!(new_register_value < 1 << B::NUMBER_OF_BITS);

        let (old_register_value, larger_register_value) =
            unsafe { self.registers.set_greater(index, new_register_value) };

        self.number_of_zero_registers -= P::NumberOfZeros::from_bool(old_register_value == 0);

        self.harmonic_sum += f64::inverse_register(larger_register_value as i32)
            - f64::inverse_register(old_register_value as i32);

        old_register_value != new_register_value
    }

    fn get_register(&self, index: usize) -> u32 {
        self.registers.get_register(index)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, A: Hash, Hasher: core::hash::Hasher + Default>
    core::iter::FromIterator<A> for HyperLogLog<P, B, R, Hasher>
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut hll = Self::new();
        hll.extend(iter);
        hll
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        Hasher: core::hash::Hasher + Default,
        Rhs: HyperLogLogTrait<P, B, Hasher>,
    > core::ops::BitOrAssign<Rhs> for HyperLogLog<P, B, R, Hasher>
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Rhs) {
        let mut rhs_registers = rhs.registers().iter_registers();

        self.registers.apply(|old_register| {
            let rhs_register: u32 = rhs_registers.next().unwrap();

            if rhs_register > old_register {
                self.harmonic_sum += f64::inverse_register(rhs_register as i32)
                    - f64::inverse_register(old_register as i32);
                if old_register == 0 {
                    self.number_of_zero_registers -= P::NumberOfZeros::ONE;
                }
                rhs_register
            } else {
                old_register
            }
        });
    }
}

impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        Rhs: HyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
    > core::ops::BitOr<Rhs> for HyperLogLog<P, B, R, Hasher>
{
    type Output = Self;

    #[inline(always)]
    fn bitor(mut self, rhs: Rhs) -> Self {
        self.bitor_assign(rhs);
        self
    }
}
