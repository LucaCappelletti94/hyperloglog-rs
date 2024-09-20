//! Implements the matrix trait for arrays.

use core::fmt::Debug;

/// Trait for matrices.
pub trait Matrix<T, const ROWS: usize> {
    /// Returns the column of the matrix.
    fn column(&self, column: usize) -> [T; ROWS];
}

impl<const ROWS: usize, T: Copy + Default + Debug, R> Matrix<T, ROWS> for [R; ROWS]
where
    R: AsRef<[T]> + Debug,
{
    #[inline]
    #[allow(unsafe_code)]
    /// Returns the column of the matrix.
    ///
    /// # Safety
    /// We are guaranteed that the length of the row is equal to the number of columns,
    /// hence we can safely use `get_unchecked`.
    fn column(&self, column: usize) -> [T; ROWS] {
        let mut result = [T::default(); ROWS];
        for (i, row) in self.iter().enumerate() {
            result[i] = unsafe { *row.as_ref().get_unchecked(column) };
        }
        result
    }
}
