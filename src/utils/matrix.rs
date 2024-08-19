//! Implements the matrix trait for arrays.

/// Trait for matrices.
pub trait Matrix<T, const ROWS: usize, const COLUMNS: usize> {
    /// Returns the column of the matrix.
    fn column(&self, column: usize) -> [T; ROWS];
}

impl<const COLUMNS: usize, const ROWS: usize, T: Copy + Default, R> Matrix<T, ROWS, COLUMNS> for [R; ROWS]
where
    R: AsRef<[T; COLUMNS]>,
{
    #[inline]
    fn column(&self, column: usize) -> [T; ROWS] {
        debug_assert!(column < COLUMNS);
        let mut result = [T::default(); ROWS];
        for (i, row) in self.iter().enumerate() {
            debug_assert_eq!(row.as_ref().len(), COLUMNS);
            result[i] = row.as_ref()[column];
        }
        result
    }
}