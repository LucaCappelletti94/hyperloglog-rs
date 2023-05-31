//! Module providing trait for comparing two numbers and returning the maximum and minimum.

pub trait MaxMin: PartialOrd {
    /// Returns the maximum of two numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let max = 2.0_f32.max(3.0_f32);
    ///
    /// assert_eq!(max, 3.0_f32);
    ///
    /// ```
    fn get_max(self, other: Self) -> Self;

    /// Returns the minimum of two numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let min = 2.0_f32.min(3.0_f32);
    ///
    /// assert_eq!(min, 2.0_f32);
    ///
    /// ```
    fn get_min(self, other: Self) -> Self;
}

impl MaxMin for f32 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for f64 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for u8 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for u16 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for u32 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for u64 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for u128 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for usize {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for i8 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for i16 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for i32 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for i64 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}

impl MaxMin for i128 {
    #[inline(always)]
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
}