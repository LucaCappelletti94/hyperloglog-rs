use hyperloglog_rs::prelude::*;
use macro_test_utils::*;

#[derive(Default)]
struct TestNamedA;

impl Named for TestNamedA {
    fn name(&self) -> String {
        "A".to_string()
    }
}

impl ExtendableApproximatedSet<u64> for TestNamedA {
    fn insert(&mut self, _element: &u64) -> bool {
        true
    }
}

impl Estimator<f64> for TestNamedA {
    fn estimate_cardinality(&self) -> f64 {
        0.0
    }

    fn estimate_union_cardinality_with_cardinalities(
        &self,
        _other: &Self,
        _cardinality: f64,
        _other_cardinality: f64,
    ) -> f64 {
        0.0
    }

    fn estimate_union_cardinality(&self, _other: &Self) -> f64 {
        0.0
    }
}

#[derive(Default)]
struct TestNamedB;

impl Named for TestNamedB {
    fn name(&self) -> String {
        "B".to_string()
    }
}

impl ExtendableApproximatedSet<u64> for TestNamedB {
    fn insert(&mut self, _element: &u64) -> bool {
        true
    }
}

impl Estimator<f64> for TestNamedB {
    fn estimate_cardinality(&self) -> f64 {
        0.0
    }

    fn estimate_union_cardinality_with_cardinalities(
        &self,
        _other: &Self,
        _cardinality: f64,
        _other_cardinality: f64,
    ) -> f64 {
        0.0
    }

    fn estimate_union_cardinality(&self, _other: &Self) -> f64 {
        0.0
    }
}

#[test]
fn test_named_derive() {
    #[derive(Named, ExtendableApproximatedSet, Estimator)]
    enum MyEnum {
        A(TestNamedA),
        B(TestNamedB),
    }

    let a = MyEnum::A(TestNamedA);
    let b = MyEnum::B(TestNamedB);

    assert_eq!(a.name(), "A");
    assert_eq!(b.name(), "B");
}

#[test]
fn test_named_derive_with_generics() {
    #[derive(Named, ExtendableApproximatedSet, Estimator)]
    enum MyEnum<const C: usize> {
        A(TestNamedA),
        B(TestNamedB),
    }

    let a: MyEnum<2> = MyEnum::A(TestNamedA);
    let b: MyEnum<5> = MyEnum::B(TestNamedB);

    assert_eq!(a.name(), "A");
    assert_eq!(b.name(), "B");
}
