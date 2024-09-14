use mem_dbg::MemSize;
use std::collections::HashSet;

/// A variant on the `MemSize` trait where the size of the
/// enum is skipped from the calculation. It always follow
/// the references.
pub trait TransparentMemSize {
    /// Returns the size of the object in bytes.
    fn transparent_mem_size(&self) -> usize;
}

impl TransparentMemSize for HashSet<u64> {
    fn transparent_mem_size(&self) -> usize {
        self.mem_size(mem_dbg::SizeFlags::default() | mem_dbg::SizeFlags::FOLLOW_REFS)
    }
}

/// A trait to represent a set.
pub trait Set {
    /// Inserts an element into the set.
    fn insert_element(&mut self, value: u64);
    /// Returns the cardinality of the set.
    fn cardinality(&self) -> f64;
    /// Returns the union of two sets.
    fn union(&self, other: &Self) -> f64;
    /// Returns the name of the model.
    fn model_name(&self) -> String;
}

impl Set for HashSet<u64> {
    fn insert_element(&mut self, value: u64) {
        self.insert(value);
    }

    fn cardinality(&self) -> f64 {
        self.len() as f64
    }

    fn union(&self, other: &Self) -> f64 {
        self.union(other).count() as f64
    }

    fn model_name(&self) -> String {
        "HashSet".to_string()
    }
}