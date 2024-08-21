//! Submodule providing the trait [`ToBytes`]

/// Trait for an object that can be represented as bytes.
pub trait ToBytes {
    /// The bytes representation of the object.
    type Bytes;

    /// Returns the bytes representation of the object.
    fn to_bytes(self) -> Self::Bytes;
}

impl ToBytes for u8 {
    type Bytes = [u8; 1];

    #[inline]
    fn to_bytes(self) -> Self::Bytes {
        [self]
    }
}

impl ToBytes for u16 {
    type Bytes = [u8; 2];

    #[inline]
    fn to_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
}

impl ToBytes for u32 {
    type Bytes = [u8; 4];

    #[inline]
    fn to_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
}

impl ToBytes for u64 {
    type Bytes = [u8; 8];

    #[inline]
    fn to_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_to_bytes() {
        let u8_value = 42u8;
        let u16_value = 42u16;
        let u24_value = u24::from(42_u32);
        let u32_value = 42u32;
        let u40_value = u40::from(42_u16);
        let u48_value = u48::from(42_u16);
        let u56_value = u56::from(42_u16);
        let u64_value = 42u64;

        assert_eq!(u8_value.to_bytes(), [42]);
        assert_eq!(u16_value.to_bytes(), [0, 42]);
        assert_eq!(u24_value.to_bytes(), [0, 0, 42]);
        assert_eq!(u32_value.to_bytes(), [0, 0, 0, 42]);
        assert_eq!(u40_value.to_bytes(), [0, 0, 0, 0, 42]);
        assert_eq!(u48_value.to_bytes(), [0, 0, 0, 0, 0, 42]);
        assert_eq!(u56_value.to_bytes(), [0, 0, 0, 0, 0, 0, 42]);
        assert_eq!(u64_value.to_bytes(), [0, 0, 0, 0, 0, 0, 0, 42]);
    }
}
