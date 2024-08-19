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
