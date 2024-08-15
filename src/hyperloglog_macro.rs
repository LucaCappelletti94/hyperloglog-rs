#[macro_export]
/// Implements the HyperLogLog trait for a given counter.
macro_rules! hll_impl {
    ($counter:ty) => {
        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Default for $counter {
            fn default() -> Self {
                Self {
                    counter: Default::default(),
                }
            }
        }

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> PartialEq for $counter {
            fn eq(&self, other: &Self) -> bool {
                self.counter == other.counter
            }
        }

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Eq for $counter {}

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> BitOrAssign
            for $counter
        {
            fn bitor_assign(&mut self, rhs: Self) {
                self.counter |= rhs.counter;
            }
        }

        #[cfg(feature = "serde")]
        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> serde::Serialize
            for $counter
        {
            #[inline(always)]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(P::NUMBER_OF_REGISTERS))?;
                for register in self.registers().iter_registers() {
                    seq.serialize_element(&(register as u8))?;
                }
                seq.end()
            }
        }

        #[cfg(feature = "serde")]
        impl<'de, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
            serde::Deserialize<'de> for $counter
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let mut registers: R = R::zeroed();
                let visitor = crate::serde::RegisterVisitor::<u8>::new(P::NUMBER_OF_REGISTERS);
                let mut iter = deserializer.deserialize_seq(visitor)?.into_iter();
                registers.apply(|_| iter.next().unwrap() as u32);
                debug_assert_eq!(iter.next(), None);
                Ok(Self::from_registers(registers))
            }
        }

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> BitOr for $counter {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    counter: self.counter | rhs.counter,
                }
            }
        }

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
            HyperLogLog<P, B, Hasher> for $counter
        {
            type Registers = R;

            fn registers(&self) -> &Self::Registers {
                self.counter.registers()
            }

            fn get_number_of_zero_registers(&self) -> <P as Precision>::NumberOfZeros {
                self.counter.get_number_of_zero_registers()
            }

            fn get_register(&self, index: usize) -> u32 {
                self.counter.get_register(index)
            }

            fn harmonic_sum<F: FloatNumber>(&self) -> F
            where
                P: PrecisionConstants<F>,
            {
                self.counter.harmonic_sum()
            }

            fn from_registers(registers: R) -> Self {
                Self {
                    counter: HyperLogLog::from_registers(registers),
                }
            }
        }



        impl<
                P: Precision,
                B: Bits,
                Hasher: HasherType,
                R: Registers<P, B>,
            > SetProperties for $counter
        {
            #[inline(always)]
            fn is_empty(&self) -> bool {
                self.counter.is_empty()
            }

            #[inline(always)]
            fn is_full(&self) -> bool {
                self.counter.is_full()
            }
        }

        impl<
                P: Precision,
                B: Bits,
                Hasher: HasherType,
                R: Registers<P, B>,
            > MutableSet for $counter
        {
            #[inline(always)]
            fn clear(&mut self) {
                self.counter.clear()
            }
        }

        impl<
                P: Precision,
                B: Bits,
                Hasher: HasherType,
                R: Registers<P, B>,
                T: core::hash::Hash,
            > ApproximatedSet<T> for $counter
        {
            #[inline(always)]
            fn may_contain(&self, element: &T) -> bool {
                self.counter.may_contain(element)
            }
        }

        impl<
                P: Precision,
                B: Bits,
                Hasher: HasherType,
                R: Registers<P, B>,
                T: core::hash::Hash,
            > ExtendableApproximatedSet<T> for $counter
        {
            #[inline(always)]
            fn insert(&mut self, element: &T) -> bool {
                self.counter.insert(element)
            }
        }

        impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B> + Words<Word = u64>>
            Hybridazable for $counter
        {
            type IterSortedHashes<'a> = core::iter::Take<R::WordIter<'a>> where Self: 'a;

            fn is_hybrid(&self) -> bool {
                self.counter.is_hybrid()
            }

            fn dehybridize(&mut self) {
                self.counter.dehybridize()
            }

            fn number_of_hashes(&self) -> usize {
                self.counter.number_of_hashes()
            }

            fn capacity(&self) -> usize {
                self.counter.capacity()
            }

            fn contains<T: core::hash::Hash>(&self, element: &T) -> bool {
                Hybridazable::contains(&self.counter, element)
            }

            fn clear_words(&mut self) {
                self.counter.clear_words()
            }

            fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_> {
                self.counter.iter_sorted_hashes()
            }

            fn hybrid_insert<T: core::hash::Hash>(&mut self, element: &T) -> bool {
                self.counter.hybrid_insert(element)
            }
        }
    };
}
