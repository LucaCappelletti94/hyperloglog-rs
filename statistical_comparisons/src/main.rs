use hyperloglog_rs::prelude::*;

/// Macro to generate the list of calls to cardinality comparison for a given precision type.
macro_rules! cardinality_comparisons {
    ($($precision:ty),*) => {
        $(
            cardinality::cardinality_comparatively::<{<$precision as Precision>::EXPONENT}, $precision>();
        )*
    }
}

/// Macro to generate the list of calls to union comparison for a given precision type.
macro_rules! union_comparisons {
    ($($precision:ty),*) => {
        $(
            union::union_comparatively::<{<$precision as Precision>::EXPONENT}, $precision>();
        )*
    }
}

fn main() {
    cardinality_comparisons!(
        Precision4,
        Precision5,
        Precision6,
        Precision7,
        Precision8,
        Precision9,
        Precision10,
        Precision11,
        Precision12,
        Precision13,
        Precision14,
        Precision15,
        Precision16,
        Precision17,
        Precision18
    );
    union_comparisons!(
        Precision4,
        Precision5,
        Precision6,
        Precision7,
        Precision8,
        Precision9,
        Precision10,
        Precision11,
        Precision12,
        Precision13,
        Precision14,
        Precision15,
        Precision16,
        Precision17,
        Precision18
    );
}
