//! This module contains the macro used to implement the [`HyperLogLog`] trait for a given counter.

#[macro_export]
/// Implements the [`HyperLogLog`] trait for a given counter.
macro_rules! hll_impl {
    ($counter:ty) => {
        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> PartialEq for $counter {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.counter == other.counter
            }
        }

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Eq for $counter {}

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> BitOrAssign
            for $counter
        {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.counter |= rhs.counter;
            }
        }

        #[cfg(feature = "serde")]
        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> serde::Serialize
            for $counter
        {
            #[inline]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(Some(1 << P::EXPONENT))?;
                for register in self.registers().iter_registers() {
                    seq.serialize_element(&register)?;
                }
                seq.end()
            }
        }

        #[cfg(feature = "serde")]
        impl<'de, P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
            serde::Deserialize<'de> for $counter
        {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let mut registers: R = R::default();
                let visitor = $crate::serde::RegisterVisitor::<u8>::new(1 << P::EXPONENT);
                let mut iter = deserializer.deserialize_seq(visitor)?.into_iter();
                registers.apply_to_registers(|_| iter.next().unwrap());
                debug_assert_eq!(iter.next(), None);
                Ok(Self::from_registers(registers))
            }
        }

        impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> BitOr for $counter {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    counter: self.counter | rhs.counter,
                }
            }
        }

        impl<
                P: Precision,
                B: Bits,
                Hasher: HasherType,
                R: Registers<P, B>,
            > MutableSet for $counter
        {
            #[inline]
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
            > ExtendableApproximatedSet<T> for $counter
        {
            #[inline]
            fn insert(&mut self, element: &T) -> bool {
                self.counter.insert(element)
            }
        }

        impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B> + VariableWords<CH>, CH: CompositeHash<P, B>>
            Hybridazable<CH> for $counter
        {
            type IterSortedHashes<'words> = <R as VariableWords<CH>>::Words<'words> where Self: 'words, CH: 'words;

            #[inline]
            fn dehybridize(&mut self) {
                self.counter.dehybridize()
            }

            #[inline]
            fn new_hybrid() -> Self {
                Self {
                    counter: Hybridazable::new_hybrid(),
                }
            }

            #[inline]
            fn is_hybrid(&self) -> bool {
                self.counter.is_hybrid()
            }

            #[inline]
            fn capacity(&self) -> usize {
                self.counter.capacity()
            }

            #[inline]
            fn number_of_hashes(&self) -> usize {
                self.counter.number_of_hashes()
            }

            #[inline]
            fn contains<T: core::hash::Hash>(&self, element: &T) -> bool {
                Hybridazable::contains(&self.counter, element)
            }

            #[inline]
            fn clear_words(&mut self) {
                self.counter.clear_words()
            }

            #[inline]
            fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_> {
                self.counter.iter_sorted_hashes()
            }

            #[inline]
            fn hybrid_insert<T: core::hash::Hash>(&mut self, element: &T) -> bool {
                self.counter.hybrid_insert(element)
            }
        }
    };
}
