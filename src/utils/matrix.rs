//! Implements the matrix trait for arrays.

/// Trait for matrices.
pub trait Matrix<T, const ROWS: usize> {
    /// Returns the column of the matrix.
    fn column(&self, column: usize) -> [T; ROWS];
}

impl<const ROWS: usize, T: Copy + Default, R> Matrix<T, ROWS>
    for [R; ROWS]
where
    R: AsRef<[T]>,
{
    #[inline]
    #[allow(unsafe_code)]
    /// Returns the column of the matrix.
    ///
    /// # Safety
    /// We are guaranteed that the length of the row is equal to the number of columns,
    /// hence we can safely use `get_unchecked`.
    fn column(&self, column: usize) -> [T; ROWS] {
        debug_assert!(column < self.as_ref().len());
        let mut result = [T::default(); ROWS];
        for (i, row) in self.iter().enumerate() {
            debug_assert_eq!(row.as_ref().len(), self.as_ref().len());
            result[i] = unsafe { *row.as_ref().get_unchecked(column) };
        }
        result
    }
}
