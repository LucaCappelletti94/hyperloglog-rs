use std::ops::{Index, IndexMut};

use crate::prelude::*;
use serde::de::Visitor;
use serde::ser::SerializeSeq;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HyperLogLogArray<const PRECISION: usize, const BITS: usize, const N: usize>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    counters: [HyperLogLog<PRECISION, BITS>; N],
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> Default
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Returns
    /// A new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::default();
    /// ```
    fn default() -> Self {
        Self {
            counters: [HyperLogLog::new(); N],
        }
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Creates a new HyperLogLogArray with the given precision and number of bits.
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let hll_array = HyperLogLogArray::<12, 6, 3>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            counters: [HyperLogLog::new(); N],
        }
    }

    #[inline(always)]
    /// Returns the estimated overlap cardinality matrices with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap cardinality matrices with.
    ///
    /// # Returns
    /// The estimated overlap cardinality matrices with the provided HyperLogLogArray.
    ///
    pub fn estimate_overlap_cardinalities(&self, other: &Self) -> [[f32; N]; N] {
        HyperLogLog::estimated_overlap_cardinality_matrix(self.as_ref(), other.as_ref())
    }

    #[inline(always)]
    /// Returns the estimated difference cardinality matrices with the provided HyperLogLog.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLog to estimate the difference cardinality matrices with.
    ///
    /// # Returns
    /// The estimated difference cardinality matrices with the provided HyperLogLogArray.
    ///
    pub fn estimated_difference_cardinality_vector(
        &self,
        other: &HyperLogLog<PRECISION, BITS>,
    ) -> [f32; N] {
        HyperLogLog::estimated_difference_cardinality_vector(self.as_ref(), other)
    }

    #[inline(always)]
    /// Returns the estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    ///
    /// # Arguments
    /// * `other`: The HyperLogLogArray to estimate the overlap and difference cardinality matrices and vectors with.
    ///
    /// # Returns
    /// The estimated overlap and difference cardinality matrices and vectors with the provided HyperLogLogArray.
    pub fn estimated_overlap_and_differences_cardinality_matrices(
        &self,
        other: &Self,
    ) -> ([[f32; N]; N], [f32; N], [f32; N]) {
        HyperLogLog::estimated_overlap_and_differences_cardinality_matrices(
            self.as_ref(),
            other.as_ref(),
        )
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    AsRef<[HyperLogLog<PRECISION, BITS>; N]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Returns a reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A reference to the underlying array of HyperLogLog counters.
    fn as_ref(&self) -> &[HyperLogLog<PRECISION, BITS>; N] {
        &self.counters
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize>
    AsMut<[HyperLogLog<PRECISION, BITS>; N]> for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Returns a mutable reference to the underlying array of HyperLogLog counters.
    ///
    /// # Returns
    /// A mutable reference to the underlying array of HyperLogLog counters.
    fn as_mut(&mut self) -> &mut [HyperLogLog<PRECISION, BITS>; N] {
        &mut self.counters
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> Index<usize>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    type Output = HyperLogLog<PRECISION, BITS>;

    #[inline(always)]
    /// Returns a reference to the HyperLogLog counter at the given index.
    ///
    /// # Arguments
    /// * `index`: The index of the HyperLogLog counter to return.
    ///
    /// # Returns
    /// A reference to the HyperLogLog counter at the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<12, 6, 4>::new();
    /// hll_array[0].insert(&1);
    /// hll_array[1].insert(&2);
    /// hll_array[2].insert(&3);
    ///
    /// assert!(hll_array[0].estimate_cardinality() > 0.9
    ///     && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[1].estimate_cardinality() > 0.9
    ///    && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[2].estimate_cardinality() > 0.9
    ///   && hll_array[2].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[3].estimate_cardinality() > -0.1
    ///  && hll_array[3].estimate_cardinality() < 0.1
    /// );
    ///
    /// ```
    fn index(&self, index: usize) -> &Self::Output {
        &self.counters[index]
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> IndexMut<usize>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    #[inline(always)]
    /// Returns a mutable reference to the HyperLogLog counter at the given index.
    ///
    /// # Arguments
    /// * `index`: The index of the HyperLogLog counter to return.
    ///
    /// # Returns
    /// A mutable reference to the HyperLogLog counter at the given index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// #![feature(generic_const_exprs)]
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<12, 6, 4>::new();
    /// hll_array[0].insert(&1);
    /// hll_array[1].insert(&2);
    /// hll_array[2].insert(&3);
    ///
    /// assert!(hll_array[0].estimate_cardinality() > 0.9
    ///    && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[1].estimate_cardinality() > 0.9
    ///  && hll_array[1].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[2].estimate_cardinality() > 0.9
    /// && hll_array[2].estimate_cardinality() < 1.1
    /// );
    /// assert!(hll_array[3].estimate_cardinality() > -0.1
    /// && hll_array[3].estimate_cardinality() < 0.1
    /// );
    ///
    /// ```
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.counters[index]
    }
}

impl<const PRECISION: usize, const BITS: usize, const N: usize> Serialize
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
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
    /// #![feature(generic_const_exprs)]
    /// use serde::Serialize;
    /// use serde_json::Serializer;
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll_array = HyperLogLogArray::<12, 6, 3>::new();
    /// hll_array[0].insert(&1);
    /// hll_array[1].insert(&2);
    /// hll_array[2].insert(&3);
    /// let mut serializer = Serializer::new(Vec::new());
    /// let result = hll_array.serialize(&mut serializer);
    /// assert!(result.is_ok(), "Serialization failed, error: {:?}", result.err());
    /// let hll_array_str = String::from_utf8(serializer.into_inner()).unwrap();
    /// let hll_array_deserialized = serde_json::from_str(&hll_array_str);
    /// assert!(hll_array_deserialized.is_ok(), "Deserialization failed, error: {:?}", hll_array_deserialized.err());
    /// let hll_array_deserialized = hll_array_deserialized.unwrap();
    /// assert_eq!(hll_array, hll_array_deserialized, "Deserialized array does not match original array");
    /// ```
    ///
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.counters.len()))?;
        for counter in self.counters {
            seq.serialize_element(&counter)?;
        }
        seq.end()
    }
}

impl<'de, const PRECISION: usize, const BITS: usize, const N: usize> Deserialize<'de>
    for HyperLogLogArray<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
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
    ///
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self {
            counters: deserializer.deserialize_seq(HLLArrayVisitor)?,
        })
    }
}

#[derive(Default)]
/// Struct to deserialize a vector of u32
pub struct HLLArrayVisitor<const PRECISION: usize, const BITS: usize, const N: usize>;

/// A visitor implementation used for deserializing an array of HLL into a fixed-size array.
///
/// This visitor is used internally by the `serde` deserialization process for the HyperLogLog struct.
/// It converts the deserialized sequence of HLL values into a fixed-size array.
///
/// # Generic Parameters
///
/// * `'de`: Lifetime specifier for the deserialization process.
/// * `PRECISION`: The precision parameter of the HyperLogLog counter.
/// * `BITS`: The number of bits used for each register in the HyperLogLog counter.
///
/// # Constraints
/// The visitor requires the following constraints:
///
/// * The precision and bits parameters must satisfy the condition `[(); ceil(1 << PRECISION, 32 / BITS)]:`.
///
/// # Associated Types
///
/// * `Value`: The type of the resulting fixed-size array.
///
/// # Methods
///
/// ## expecting
///
/// Sets the error message for the expectation of an array of HLL.
///
/// ### Arguments
///
/// * `formatter`: A mutable reference to the formatter used to format the error message.
///
/// ### Returns
/// A `std::fmt::Result` indicating the success or failure of the formatting operation.
///
/// ## visit_seq
/// Processes the deserialized sequence and converts it into a fixed-size array of HLL values.
///
/// ### Arguments
/// * `seq`: The sequence access object used to iterate over the deserialized elements.
///
/// ### Returns
/// The resulting fixed-size array of u32 values, or an error if the deserialization failed.
impl<'de, const PRECISION: usize, const BITS: usize, const N: usize> Visitor<'de>
    for HLLArrayVisitor<PRECISION, BITS, N>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    type Value = [HyperLogLog<PRECISION, BITS>; N];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of HLL")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut hll_array = [HyperLogLog::new(); N];
        let mut hll_array_iter = hll_array.iter_mut();
        while let Some(value) = seq.next_element()? {
            if let Some(target) = hll_array_iter.next() {
                *target = value;
            } else {
                return Err(serde::de::Error::invalid_length(hll_array.len(), &self));
            }
        }
        Ok(hll_array)
    }
}

impl<const PRECISION: usize, const BITS: usize> Serialize for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
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
    ///
    /// ```
    /// use serde::Serialize;
    /// use serde_json::Serializer;
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let hll = HyperLogLog::<12, 6>::new();
    /// let mut serializer = Serializer::new(Vec::new());
    /// let result = hll.serialize(&mut serializer);
    /// assert!(result.is_ok(), "Serialization failed, error: {:?}", result.err());
    /// ```
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.words.len()))?;
        for word in self.words {
            seq.serialize_element(&word)?;
        }
        seq.end()
    }
}

impl<'de, const PRECISION: usize, const BITS: usize> Deserialize<'de>
    for HyperLogLog<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
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
    ///
    /// # Example
    ///
    /// ```
    /// #![feature(generic_const_exprs)]
    /// use serde::de::Deserialize;
    /// use serde_json::Deserializer;
    /// use hyperloglog_rs::HyperLogLog;
    ///
    /// let words = [0, 0, 0, 0, 5, 0, 4, 0, 0, 3, 0, 0, 0];
    /// let words_str = "[0, 0, 0, 0, 5, 0, 4, 0, 0, 3, 0, 0, 0]";
    /// let mut deserializer = Deserializer::from_str(words_str);
    /// let result = HyperLogLog::<6, 6>::deserialize(&mut deserializer);
    /// assert!(result.is_ok(), "Deserialization failed, error: {:?}", result.err());
    /// let hll = result.unwrap();
    /// hll.get_words().iter().zip(words.iter()).for_each(|(a, b)| assert_eq!(a, b, "Deserialized words do not match, expected: {}, actual: {}", b, a));
    /// ```
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut hll = Self {
            words: deserializer.deserialize_seq(U32ArrayVisitor)?,
            number_of_zero_register: 0,
        };

        hll.number_of_zero_register = hll.iter().filter(|&register| register == 0).count();

        Ok(hll)
    }
}

#[derive(Default)]
/// Struct to deserialize a vector of u32
pub struct U32ArrayVisitor<const PRECISION: usize, const BITS: usize>;

/// A visitor implementation used for deserializing an array of u32 into a fixed-size array.
///
/// This visitor is used internally by the `serde` deserialization process for the HyperLogLog struct.
/// It converts the deserialized sequence of u32 values into a fixed-size array.
///
/// # Generic Parameters
///
/// * `'de`: Lifetime specifier for the deserialization process.
/// * `PRECISION`: The precision parameter of the HyperLogLog counter.
/// * `BITS`: The number of bits used for each register in the HyperLogLog counter.
///
/// # Constraints
/// The visitor requires the following constraints:
///
/// * The precision and bits parameters must satisfy the condition `[(); ceil(1 << PRECISION, 32 / BITS)]:`.
///
/// # Associated Types
///
/// * `Value`: The type of the resulting fixed-size array.
///
/// # Methods
///
/// ## expecting
///
/// Sets the error message for the expectation of an array of u32.
///
/// ### Arguments
///
/// * `formatter`: A mutable reference to the formatter used to format the error message.
///
/// ### Returns
/// A `std::fmt::Result` indicating the success or failure of the formatting operation.
///
/// ## visit_seq
/// Processes the deserialized sequence and converts it into a fixed-size array of u32 values.
///
/// ### Arguments
/// * `seq`: The sequence access object used to iterate over the deserialized elements.
///
/// ### Returns
/// The resulting fixed-size array of u32 values, or an error if the deserialization failed.
impl<'de, const PRECISION: usize, const BITS: usize> Visitor<'de>
    for U32ArrayVisitor<PRECISION, BITS>
where
    [(); ceil(1 << PRECISION, 32 / BITS)]:,
{
    type Value = [u32; ceil(1 << PRECISION, 32 / BITS)];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of u32")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut words_array = [0; ceil(1 << PRECISION, 32 / BITS)];
        let mut words_array_iter = words_array.iter_mut();
        while let Some(value) = seq.next_element()? {
            if let Some(target) = words_array_iter.next() {
                *target = value;
            } else {
                return Err(serde::de::Error::invalid_length(words_array.len(), &self));
            }
        }
        Ok(words_array)
    }
}
