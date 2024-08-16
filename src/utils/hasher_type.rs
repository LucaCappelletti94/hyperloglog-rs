//! Trait for Hashers that can be deaulted and shared among threads.
use core::hash::Hasher;

/// Trait for Hashers that can be deaulted and shared among threads.
pub trait HasherType: Default + Hasher + Send + Sync + Clone {}

impl<T> HasherType for T where T: Default + Hasher + Send + Sync + Clone {}
