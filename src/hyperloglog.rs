use crate::prelude::*;
use crate::utils::*;
use crate::{prelude::HyperLogLogTrait, sip::hash_and_index};
use core::hash::Hash;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
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
pub struct HyperLogLog<P: Precision, B: Bits, R: Registers<P, B>> {
    registers: R,
    number_of_zero_registers: P::NumberOfZeros,
    _precision: core::marker::PhantomData<P>,
    _bits: core::marker::PhantomData<B>,
}

impl<P: Precision, B: Bits, R: Registers<P, B>> HyperLogLog<P, B, R> {
    /// Create a new HyperLogLog counter.
    fn new() -> Self {
        Self {
            registers: R::zeroed(),
            number_of_zero_registers: unsafe {
                P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked()
            },
            _precision: core::marker::PhantomData,
            _bits: core::marker::PhantomData,
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
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let registers = [0_u32; 4];
    /// let hll = HyperLogLog::<Precision4, Bits6, <Precision4 as ArrayRegister<Bits6>>::ArrayRegister>::from_registers(registers);
    /// let collected_registers = hll.iter_registers().collect::<Vec<u32>>();
    /// assert_eq!(collected_registers, vec![0_u32; 1 << 4]);
    /// ```
    pub fn from_registers(registers: R) -> Self {
        let number_of_zero_registers = registers
            .iter_registers()
            .map(|register| {
                if register.is_zero() {
                    P::NumberOfZeros::ONE
                } else {
                    P::NumberOfZeros::ZERO
                }
            })
            .sum();

        Self {
            registers,
            number_of_zero_registers,
            _precision: core::marker::PhantomData,
            _bits: core::marker::PhantomData,
        }
    }
}

/// Implements the Default trait for HyperLogLog.
///
/// HyperLogLog is a probabilistic cardinality estimator that uses a fixed
/// amount of memory to estimate the number of distinct elements in a set.
///
/// # Examples
///
/// ```rust
/// # use hyperloglog_rs::prelude::*;
///
/// let hll: HyperLogLog<Precision10, Bits6, <Precision10 as ArrayRegister<Bits6>>::ArrayRegister> =
///     Default::default();
/// let collected_registers = hll.iter_registers().collect::<Vec<u32>>();
/// assert_eq!(collected_registers, vec![0_u32; 1 << 10]);
/// ```
impl<P: Precision, B: Bits, R: Registers<P, B>> Default for HyperLogLog<P, B, R> {
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>> HyperLogLogTrait<P, B> for HyperLogLog<P, B, R> {
    type IterRegisters<'a> = R::Iter<'a> where Self: 'a;

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
    /// Returns an iterator over the registers of the HyperLogLog counter.
    fn iter_registers(&self) -> Self::IterRegisters<'_> {
        self.registers.iter_registers()
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
        let (hash, index) = hash_and_index::<T, P, B>(&rhs);

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();
        debug_assert!(number_of_zeros < 1 << B::NUMBER_OF_BITS);

        unsafe {
            if let Some(old_register) = self.registers.set_greater(index, number_of_zeros) {
                if old_register == 0 {
                    self.number_of_zero_registers -= P::NumberOfZeros::ONE;
                }
                true
            } else {
                false
            }
        }
    }

    fn get_register(&self, index: usize) -> u32 {
        self.registers.get_register(index)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, A: Hash> core::iter::FromIterator<A>
    for HyperLogLog<P, B, R>
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut hll = Self::new();
        hll.extend(iter);
        hll
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl<P: Precision, B: Bits, R: Registers<P, B>, Rhs: HyperLogLogTrait<P, B>>
    core::ops::BitOrAssign<Rhs> for HyperLogLog<P, B, R>
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Rhs) {
        let mut rhs_registers = rhs.iter_registers();
        let mut number_of_zeros = P::NumberOfZeros::ZERO;

        self.registers.apply(|register| {
            let rhs_register: u32 = rhs_registers.next().unwrap();
            let new_register: u32 = register.max(rhs_register);

            if new_register == 0 {
                number_of_zeros += P::NumberOfZeros::ONE;
            }

            new_register
        });

        self.number_of_zero_registers = number_of_zeros;
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Rhs: HyperLogLogTrait<P, B>> core::ops::BitOr<Rhs>
    for HyperLogLog<P, B, R>
{
    type Output = Self;

    #[inline(always)]
    fn bitor(mut self, rhs: Rhs) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>> From<HyperLogLog<P, B, R>>
    for HLLMultiplicities<P, B, R, M>
{
    fn from(hll: HyperLogLog<P, B, R>) -> Self {
        Self::from_registers(hll.registers)
    }
}
