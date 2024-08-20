//! Submodule providing the trait AsBytes

/// Trait for an object that can be represented as bytes.
pub trait AsBytes {
    /// The bytes representation of the object.
    type Bytes;

    /// Returns the bytes representation of the object.
    fn as_bytes(self) -> Self::Bytes;
}

impl AsBytes for u8 {
    type Bytes = [u8; 1];

    #[inline]
    fn as_bytes(self) -> Self::Bytes {
        [self]
    }
}

impl AsBytes for u16 {
    type Bytes = [u8; 2];

    #[inline]
    fn as_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
}

impl AsBytes for u32 {
    type Bytes = [u8; 4];

    #[inline]
    fn as_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
}

impl AsBytes for u64 {
    type Bytes = [u8; 8];

    #[inline]
    fn as_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use super::*;

    #[test]
    fn test_as_bytes() {
        let u8_value = 42u8;
        let u16_value = 42u16;
        let u24_value = u24::from(42_u32);
        let u32_value = 42u32;
        let u40_value = u40::from(42_u16);
        let u48_value = u48::from(42_u16);
        let u56_value = u56::from(42_u16);
        let u64_value = 42u64;

        assert_eq!(u8_value.as_bytes(), [42]);
        assert_eq!(u16_value.as_bytes(), [0, 42]);
        assert_eq!(u24_value.as_bytes(), [0, 0, 42]);
        assert_eq!(u32_value.as_bytes(), [0, 0, 0, 42]);
        assert_eq!(u40_value.as_bytes(), [0, 0, 0, 0, 42]);
        assert_eq!(u48_value.as_bytes(), [0, 0, 0, 0, 0, 42]);
        assert_eq!(u56_value.as_bytes(), [0, 0, 0, 0, 0, 0, 42]);
        assert_eq!(u64_value.as_bytes(), [0, 0, 0, 0, 0, 0, 0, 42]);
    }
}