use crate::prelude::*;
use serde::de::{DeserializeOwned, Visitor};
use serde::ser::SerializeSeq;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub trait SerdeHyperLogLogTrait<P: Precision, B: Bits, Hasher: core::hash::Hasher + Default>:
    HyperLogLogTrait<P, B, Hasher> + Serialize + DeserializeOwned
{
}

impl<
        P: Precision,
        B: Bits,
        Hasher: core::hash::Hasher + Default,
        H: HyperLogLogTrait<P, B, Hasher> + Serialize + DeserializeOwned,
    > SerdeHyperLogLogTrait<P, B, Hasher> for H
{
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default> Serialize
    for HyperLogLog<P, B, R, Hasher>
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(P::NUMBER_OF_REGISTERS))?;
        for register in self.registers().iter_registers() {
            seq.serialize_element(&(register as u8))?;
        }
        seq.end()
    }
}

impl<'de, P: Precision, B: Bits, R: Registers<P, B>, Hasher: core::hash::Hasher + Default> Deserialize<'de>
    for HyperLogLog<P, B, R, Hasher>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut registers: R = R::zeroed();
        let visitor = RegisterVisitor::<u8>::new(P::NUMBER_OF_REGISTERS);
        let mut iter = deserializer.deserialize_seq(visitor)?.into_iter();
        registers.apply(|_| iter.next().unwrap() as u32);
        debug_assert_eq!(iter.next(), None);
        Ok(Self::from_registers(registers))
    }
}

impl<
        P: Precision,
        B: Bits,
        H: SerdeHyperLogLogTrait<P, B, Hasher>,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > Serialize for HyperLogLogArray<P, B, H, Hasher, N>
{
    #[inline(always)]
    /// Serializes the HyperLogLog counter using the given serializer.
    ///
    /// This method is part of the `Serialize` trait implementation for the HyperLogLog struct,
    /// allowing the counter to be serialized into a format supported by the serializer.
    ///
    /// # Arguments
    /// * `serializer`: The serializer used to serialize the HyperLogLog counter.
    ///
    /// # Returns
    /// The serialization result, indicating success or failure.
    ///
    /// # Example
    /// In this example, we serialize an array of HyperLogLog counters into a JSON string.
    /// The resulting string is then deserialized back into an array of HyperLogLog counters.
    ///
    /// Since we cannot implement these traits for array, we need to wrap the array in a struct,
    /// which in this case is `HyperLogLogArray`.
    ///
    /// ```rust
    /// use hyperloglog_rs::prelude::*;
    /// use serde::Serialize;
    /// use serde_json::Serializer;
    ///
    /// let mut hll_array = HyperLogLogArray::<
    ///     Precision4,
    ///     Bits6,
    ///     HyperLogLog<Precision4, Bits6, <Precision4 as ArrayRegister<Bits6>>::ArrayRegister>,
    ///     3,
    /// >::default();
    /// hll_array.insert(0, &1);
    /// hll_array.insert(1, &2);
    /// hll_array.insert(2, &3);
    /// let mut serializer = Serializer::new(Vec::new());
    /// let result = hll_array.serialize(&mut serializer);
    /// assert!(
    ///     result.is_ok(),
    ///     "1) Serialization failed, error: {:?}",
    ///     result.err()
    /// );
    /// let hll_array_str = String::from_utf8(serializer.into_inner()).unwrap();
    /// let hll_array_deserialized = serde_json::from_str(&hll_array_str);
    /// assert!(
    ///     hll_array_deserialized.is_ok(),
    ///     "2) Deserialization failed, error: {:?}",
    ///     hll_array_deserialized.err()
    /// );
    /// let hll_array_deserialized = hll_array_deserialized.unwrap();
    /// assert_eq!(
    ///     hll_array, hll_array_deserialized,
    ///     "3) Deserialized array does not match original array"
    /// );
    /// ```
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(N))?;
        let counters: &[H; N] = self.as_ref();
        for counter in counters {
            seq.serialize_element(&counter)?;
        }
        seq.end()
    }
}

impl<
        'de,
        P: Precision,
        B: Bits,
        H: SerdeHyperLogLogTrait<P, B, Hasher> + Copy,
        Hasher: core::hash::Hasher + Default + Default,
        const N: usize,
    > Deserialize<'de> for HyperLogLogArray<P, B, H, Hasher, N>
{
    #[inline(always)]
    /// Deserializes the HyperLogLog counter using the given deserializer.
    ///
    /// This method is part of the `Deserialize` trait implementation for the HyperLogLog struct,
    /// allowing the counter to be deserialized from a format supported by the deserializer.
    ///
    /// # Arguments
    /// * `deserializer`: The deserializer used to deserialize the HyperLogLog counter.
    ///
    /// # Returns
    /// The deserialization result, indicating success or failure.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(
            deserializer.deserialize_seq(HLLArrayVisitor::default())?,
        ))
    }
}

#[derive(Default)]
/// Struct to deserialize a vector of u32
pub struct HLLArrayVisitor<
    P: Precision,
    B: Bits,
    H: SerdeHyperLogLogTrait<P, B, Hasher>,
    Hasher: core::hash::Hasher + Default,
    const N: usize,
> {
    _phantom: core::marker::PhantomData<(P, B, H, Hasher)>,
}

/// A visitor implementation used for deserializing an array.
impl<
        'de,
        P: Precision,
        B: Bits,
        H: SerdeHyperLogLogTrait<P, B, Hasher> + Copy,
        Hasher: core::hash::Hasher + Default,
        const N: usize,
    > Visitor<'de> for HLLArrayVisitor<P, B, H, Hasher, N>
{
    type Value = [H; N];

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!(
            "an array of {} {}",
            N,
            core::any::type_name::<H>()
        ))
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut array = [H::default(); N];
        let mut array_iter: core::slice::IterMut<H> = array.iter_mut();
        while let Some(value) = seq.next_element()? {
            array_iter
                .next()
                .map(|target: &mut H| {
                    *target = value;
                })
                .ok_or(serde::de::Error::invalid_length(N, &self))?;
        }
        Ok(array)
    }
}

/// Struct to deserialize a vector of T
struct RegisterVisitor<T> {
    expected_length: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> RegisterVisitor<T> {
    fn new(expected_length: usize) -> Self {
        Self {
            expected_length,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'de, T: Default + Copy + Deserialize<'de>> Visitor<'de> for RegisterVisitor<T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!(
            "an array of {} {} elements",
            self.expected_length,
            core::any::type_name::<T>()
        ))
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut array = vec![T::default(); self.expected_length];
        {
            let mut array_iter = array.iter_mut();
            while let Some(value) = seq.next_element()? {
                if let Some(target) = array_iter.next() {
                    *target = value;
                } else {
                    return Err(serde::de::Error::invalid_length(array.len(), &self));
                }
            }
        }
        Ok(array)
    }
}
