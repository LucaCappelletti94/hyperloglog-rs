use crate::prelude::*;
use crate::utils::*;
use core::hash::Hash;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A basic counter data structure for HyperLogLog-like counters.
pub(crate) struct BasicLogLog<
    P: Precision,
    B: Bits,
    R: Registers<P, B>,
    Hasher: HasherType,
    S: FloatNumber = f32,
> {
    registers: R,
    number_of_zero_registers: P::NumberOfZeros,
    harmonic_sum: S,
    _phantom: core::marker::PhantomData<(P, B, Hasher)>,
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, S: FloatNumber> core::fmt::Debug
    for BasicLogLog<P, B, R, Hasher, S>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BasicLogLog")
            .field("registers", &self.registers)
            .field("number_of_zero_registers", &self.number_of_zero_registers)
            .field("harmonic_sum", &self.harmonic_sum)
            .finish()
    }
}

/// Implementation of partial equality for HyperLogLog so as to compare two HyperLogLog instances
/// ignoring the harmonic sum.
impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, S: FloatNumber> PartialEq
    for BasicLogLog<P, B, R, Hasher, S>
{
    fn eq(&self, other: &Self) -> bool {
        self.registers == other.registers
    }
}

/// Implementation of equality for HyperLogLog so as to compare two HyperLogLog instances
/// ignoring the harmonic sum.
impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, S: FloatNumber> Eq
    for BasicLogLog<P, B, R, Hasher, S>
{
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, S: FloatNumber>
    BasicLogLog<P, B, R, Hasher, S>
{
    /// Create a new HyperLogLog counter.
    fn new() -> Self {
        Self {
            registers: R::zeroed(),
            number_of_zero_registers: unsafe {
                P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked()
            },
            harmonic_sum: S::from_usize(P::NUMBER_OF_REGISTERS),
            _phantom: core::marker::PhantomData,
        }
    }

    fn compute_hash<T: Hash>(&self, value: T) -> u64 {
        let mut hasher = Hasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn insert_register_value_and_index(&mut self, new_register_value: u32, index: usize) -> bool {
        // Count leading zeros.
        debug_assert!(new_register_value < 1 << B::NUMBER_OF_BITS);

        let (old_register_value, larger_register_value) =
            unsafe { self.registers.set_greater(index, new_register_value) };

        self.number_of_zero_registers -= P::NumberOfZeros::from_bool(old_register_value == 0);

        self.harmonic_sum += S::inverse_register(larger_register_value as i32)
            - S::inverse_register(old_register_value as i32);

        old_register_value != new_register_value
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, S: FloatNumber> Default
    for BasicLogLog<P, B, R, Hasher, S>
{
    /// Returns a new HyperLogLog instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, S: FloatNumber>
    HyperLogLog<P, B, Hasher> for BasicLogLog<P, B, R, Hasher, S>
{
    type Registers = R;

    #[inline(always)]
    /// Returns the number of registers with zero values.
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
        F::from_f64(self.harmonic_sum.to_f64())
    }

    fn get_register(&self, index: usize) -> u32 {
        self.registers.get_register(index)
    }

    fn from_registers(registers: R) -> Self {
        let mut number_of_zero_registers = P::NumberOfZeros::ZERO;
        let mut harmonic_sum = S::ZERO;

        for register in registers.iter_registers() {
            number_of_zero_registers += P::NumberOfZeros::from_bool(register == 0);
            harmonic_sum += S::inverse_register(register as i32);
        }

        Self {
            registers,
            number_of_zero_registers,
            harmonic_sum,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, A: Hash, Hasher: HasherType, S: FloatNumber>
    core::iter::FromIterator<A> for BasicLogLog<P, B, R, Hasher, S>
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
        Hasher: HasherType,
        Rhs: HyperLogLog<P, B, Hasher>,
        S: FloatNumber,
    > core::ops::BitOrAssign<Rhs> for BasicLogLog<P, B, R, Hasher, S>
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Rhs) {
        let mut rhs_registers = rhs.registers().iter_registers();

        self.registers.apply(|old_register| {
            let rhs_register: u32 = rhs_registers.next().unwrap();

            if rhs_register > old_register {
                self.harmonic_sum += S::inverse_register(rhs_register as i32)
                    - S::inverse_register(old_register as i32);
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
        Rhs: HyperLogLog<P, B, Hasher>,
        Hasher: HasherType,
        S: FloatNumber,
    > core::ops::BitOr<Rhs> for BasicLogLog<P, B, R, Hasher, S>
{
    type Output = Self;

    #[inline(always)]
    fn bitor(mut self, rhs: Rhs) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B> + Words<Word = u64>,
        Hasher: HasherType,
        S: FloatNumber,
    > Hybridazable for BasicLogLog<P, B, R, Hasher, S>
{
    type Words = R;
    type IterSortedHashes<'a> = core::iter::Take<R::WordIter<'a>> where Self: 'a;

    fn is_hybrid(&self) -> bool {
        // We employ the harmonic sum as a flag to represent whether the counter is in hybrid mode.
        self.harmonic_sum < S::ZERO
    }

    fn dehybridize(&mut self) {
        if self.is_hybrid() {
            let number_of_hashes = self.number_of_hashes();
            self.number_of_zero_registers = unsafe {
                P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked()
            };
            self.harmonic_sum = S::from_usize(P::NUMBER_OF_REGISTERS);
            let registers = self.registers.clone();
            self.registers = R::zeroed();
            for hash in registers.words().take(number_of_hashes) {
                let (register_value, index) = Self::split_hash(hash);
                self.insert_register_value_and_index(register_value, index);
            }
        }
    }

    fn number_of_hashes(&self) -> usize {
        unsafe {
            self.get_number_of_zero_registers()
                .try_into()
                .unwrap_unchecked()
        }
    }

    fn capacity(&self) -> usize {
        self.registers.number_of_words()
    }

    fn clear_words(&mut self) {
        self.registers.clear();
        self.number_of_zero_registers = P::NumberOfZeros::ZERO;
        self.harmonic_sum = S::NEG_INFINITY;
    }

    fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_> {
        self.registers.words().take(self.number_of_hashes())
    }

    fn contains<T: core::hash::Hash>(&self, element: &T) -> bool {
        debug_assert!(
            self.number_of_hashes() <= self.registers.number_of_words(),
            "Number of hashes ({}) is greater than the number of words ({}) in the list of hashes.",
            self.number_of_hashes(),
            self.registers.number_of_words()
        );
        unsafe {
            self.registers
                .find_sorted_with_len(self.compute_hash(element), self.number_of_hashes())
        }
    }

    fn hybrid_insert<T: core::hash::Hash>(&mut self, element: &T) -> bool {
        // In hybrid setting, we are using the registers as a list of hashes
        // instead of the actual registers of an HyperLogLog counter, and we
        // use the number of zeros as the number of words in the list.

        if self.is_hybrid() {
            // If the counter in hybrid mode has reached saturation, i.e. has as many
            // hashes stored as it can fit, we switch to the normal HyperLogLog mode.
            if self.capacity() == self.number_of_hashes() {
                debug_assert_eq!(
                    self.number_of_hashes(),
                    self.registers.number_of_words(),
                    "Number of hashes ({}) is not equal to the number of words ({}) in the list of hashes.",
                    self.number_of_hashes(),
                    self.registers.number_of_words()
                );
                self.dehybridize();
                self.insert(element)
            } else {
                let hash = self.compute_hash(element);
                if unsafe {
                    self.registers
                        .sorted_insert_with_len(hash, self.number_of_hashes())
                } {
                    debug_assert!(
                        self.number_of_zero_registers <= unsafe{P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked()},
                        "Number of zero registers ({}) is greater than the number of registers ({})",
                        self.number_of_zero_registers,
                        P::NUMBER_OF_REGISTERS
                    );
                    self.number_of_zero_registers += P::NumberOfZeros::ONE;
                    true
                } else {
                    false
                }
            }
        } else {
            self.insert(element)
        }
    }
}

impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B> + Words<Word = u64>,
        Hasher: HasherType,
        S: FloatNumber,
    > Default for Hybrid<BasicLogLog<P, B, R, Hasher, S>>
{
    fn default() -> Self {
        let mut hll: BasicLogLog<P, B, R, Hasher, S> = Default::default();
        hll.harmonic_sum = S::NEG_INFINITY;
        hll.number_of_zero_registers = P::NumberOfZeros::ZERO;
        Self::from(hll)
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>, S: FloatNumber> SetProperties
    for BasicLogLog<P, B, R, Hasher, S>
{
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.number_of_zero_registers
            == unsafe { P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked() }
    }

    #[inline(always)]
    fn is_full(&self) -> bool {
        // The harmonic sum is defined as Sum(2^(-register_value)) for all registers.
        // When all registers are maximally filled, i.e. equal to the maximal multiplicity value,
        // the harmonic sum is equal to (2^(-max_multiplicity)) * number_of_registers.
        // Since number_of_registers is a power of 2, specifically 2^exponent, the harmonic sum
        // is equal to 2^(exponent - max_multiplicity).
        self.harmonic_sum <= miminal_harmonic_sum::<S, P, B>()
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>, T: Hash, S: FloatNumber>
    ApproximatedSet<T> for BasicLogLog<P, B, R, Hasher, S>
{
    #[inline(always)]
    fn may_contain(&self, element: &T) -> bool {
        self.get_register(Self::hash_and_index::<T>(element).1) > 0
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>, S: FloatNumber>
    MutableSet for BasicLogLog<P, B, R, Hasher, S>
{
    fn clear(&mut self) {
        self.registers.clear();
        self.number_of_zero_registers =
            unsafe { P::NumberOfZeros::try_from(P::NUMBER_OF_REGISTERS).unwrap_unchecked() };
        self.harmonic_sum = S::from_usize(P::NUMBER_OF_REGISTERS);
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>, T: Hash, S: FloatNumber>
    ExtendableApproximatedSet<T> for BasicLogLog<P, B, R, Hasher, S>
{
    fn insert(&mut self, element: &T) -> bool {
        let (new_register_value, index) = Self::hash_and_index::<T>(element);

        self.insert_register_value_and_index(new_register_value, index)
    }
}
