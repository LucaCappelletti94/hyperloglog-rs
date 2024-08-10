//! Trait for Hashers that can be deaulted and shared among threads.

pub trait HasherType: Default + core::hash::Hasher + Send + Sync + Clone{}

impl<T> HasherType for T where T: Default + core::hash::Hasher + Send + Sync + Clone{}