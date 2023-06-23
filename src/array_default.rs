//! This module contains the `ArrayDefault` trait, which is used to set the default value of an array.
//! This trait is necessary as the standard library only provides a `Default` implementation for arrays
//! of limited length, while we need this for objects of several differenty lengths.

pub trait ArrayDefault<T> {
    fn default_array() -> Self;
}

pub trait ArrayIter<T> {
    type Iter<'a>: Iterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;
    type IterMut<'a>: Iterator<Item = &'a mut T>
    where
        Self: 'a,
        T: 'a;
    type IntoIter: Iterator<Item = T>;

    fn into_iter_elements(self) -> Self::IntoIter;
    fn iter_elements(&self) -> Self::Iter<'_>;
    fn iter_elements_mut(&mut self) -> Self::IterMut<'_>;
    fn len(&self) -> usize;
}

impl<T: Default + Copy, const N: usize> ArrayDefault<T> for [T; N] {
    #[inline(always)]
    fn default_array() -> Self {
        [T::default(); N]
    }
}

impl<T: Default, const N: usize> ArrayIter<T> for [T; N] {
    type Iter<'a> = core::slice::Iter<'a, T> where Self: 'a;
    type IterMut<'a> = core::slice::IterMut<'a, T> where Self: 'a;
    type IntoIter = core::array::IntoIter<T, N>;

    #[inline(always)]
    fn into_iter_elements(self) -> Self::IntoIter {
        self.into_iter()
    }

    #[inline(always)]
    fn iter_elements(&self) -> Self::Iter<'_> {
        self.iter()
    }

    #[inline(always)]
    fn iter_elements_mut(&mut self) -> Self::IterMut<'_> {
        self.iter_mut()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        N
    }
}
