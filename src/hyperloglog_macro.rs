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
    };
}
