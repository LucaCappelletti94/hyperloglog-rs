//! This module contains the `ArrayDefault` trait, which is used to set the default value of an array.
//! This trait is necessary as the standard library only provides a `Default` implementation for arrays
//! of limited length, while we need this for objects of several differenty lengths.

pub trait ArrayDefault<T> {
    fn default_array() -> Self;
}

pub trait ArrayIter<T: Default + PartialEq> {
    type Iter<'a>: Iterator<Item = &'a T> + DoubleEndedIterator + ExactSizeIterator
    where
        Self: 'a,
        T: 'a;
    type IterMut<'a>: Iterator<Item = &'a mut T> + DoubleEndedIterator + ExactSizeIterator
    where
        Self: 'a,
        T: 'a;
    type IntoIter: Iterator<Item = T> + DoubleEndedIterator + ExactSizeIterator;

    fn into_iter_elements(self) -> Self::IntoIter;
    fn iter_elements(&self) -> Self::Iter<'_>;
    fn iter_elements_mut(&mut self) -> Self::IterMut<'_>;
    fn len(&self) -> usize;
    fn last(&self) -> Option<&T>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the first element that is not equal to the default value.
    fn first_non_zero_index(&self) -> Option<usize> {
        self.iter_elements()
            .position(|a| *a != T::default())
    }
    /// Returns the last element that is not equal to the default value.
    fn last_non_zero_index(&self) -> Option<usize> {
        self.iter_elements()
            .rposition(|a| *a != T::default())
    }
}

pub trait ArrayIterArgmin<T>: ArrayIter<T> where T: PartialOrd + PartialEq + Default{
    /// Returns the index of the element with the smallest value.
    fn argmin(&self) -> Option<usize> {
        self.iter_elements()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
    }
    
    /// Returns the index of the element with the smallest value that is not equal to the default value.
    fn non_zero_argmin(&self) -> Option<usize> {
        self.iter_elements()
            .enumerate()
            .filter(|(_, a)| **a != T::default())
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
    }    
}

pub trait ArrayIterArgmax<T>: ArrayIter<T> where T: PartialOrd + PartialEq + Default{
    /// Returns the index of the element with the largest value.
    fn argmax(&self) -> Option<usize> {
        self.iter_elements()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
    }
    
    /// Returns the index of the element with the largest value that is not equal to the default value.
    fn non_zero_argmax(&self) -> Option<usize> {
        self.iter_elements()
            .enumerate()
            .filter(|(_, a)| **a != T::default())
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
    }
}

impl<T: Default + Copy, const N: usize> ArrayDefault<T> for [T; N] {
    #[inline(always)]
    fn default_array() -> Self {
        [T::default(); N]
    }
}

impl<T: PartialOrd + Default, const N: usize> ArrayIterArgmin<T> for [T; N] {}
impl<T: PartialOrd + Default, const N: usize> ArrayIterArgmax<T> for [T; N] {}

impl<T: Default + PartialEq, const N: usize> ArrayIter<T> for [T; N] {
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

    #[inline(always)]
    fn last(&self) -> Option<&T> {
        <[T]>::last(self)
    }
}
