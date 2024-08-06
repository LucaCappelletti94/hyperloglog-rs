//! Submodule providing the trait marker Bits.

pub trait Bits: Default + Copy + PartialEq + Eq {
    const NUMBER_OF_BITS: usize;
}

/// Implementation
macro_rules! impl_bits {
    ($($n: expr),*) => {
        $(
            paste::paste! {
                #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
                pub struct [<Bits $n>];

                impl Bits for [<Bits $n>] {
                    const NUMBER_OF_BITS: usize = $n;
                }
            }
        )*
    };
}

impl_bits!(1, 2, 3, 4, 5, 6);
