//! Submodule providing the 'NaiveIntegerHash' struct, which implements the
//! Hasher and BuildHasher traits for hashing integers in a naive way.

use core::hash::{BuildHasher, Hasher};

#[derive(Default, Clone, Copy)]
/// Struct for hashing integers in a naive way.
pub struct NaiveIntegerHash {
    state: u64,
}

impl BuildHasher for NaiveIntegerHash {
    type Hasher = NaiveIntegerHash;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        NaiveIntegerHash { state: 0 }
    }
}

impl Hasher for NaiveIntegerHash {
    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.state = i;
    }

    #[inline]
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }
}