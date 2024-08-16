//! Module to handle serialization and deserialization of the registers
use core::marker::PhantomData;
use serde::de::SeqAccess;
use core::any::type_name;
use core::fmt::Formatter;
use serde::de::Visitor;

/// Struct to deserialize a vector of T
pub(crate) struct RegisterVisitor<T> {
    /// The expected length of the vector
    expected_length: usize,
    /// Phantom data to keep the type
    _phantom: PhantomData<T>,
}

impl<T> RegisterVisitor<T> {
    /// Creates a new [`RegisterVisitor`]
    pub(crate) fn new(expected_length: usize) -> Self {
        Self {
            expected_length,
            _phantom: PhantomData,
        }
    }
}

impl<'de, T: Default + Copy + serde::Deserialize<'de>> Visitor<'de>
    for RegisterVisitor<T>
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str(&format!(
            "an array of {} {} elements",
            self.expected_length,
            type_name::<T>()
        ))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
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
