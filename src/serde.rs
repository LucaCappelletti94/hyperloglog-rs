/// Struct to deserialize a vector of T
pub(crate) struct RegisterVisitor<T> {
    expected_length: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> RegisterVisitor<T> {
    pub(crate) fn new(expected_length: usize) -> Self {
        Self {
            expected_length,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'de, T: Default + Copy + serde::Deserialize<'de>> serde::de::Visitor<'de>
    for RegisterVisitor<T>
{
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
