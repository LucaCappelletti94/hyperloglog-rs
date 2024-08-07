//! Bench to compare the performance of different hash functions.
#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion};
use std::hash::Hash;
use std::hash::Hasher;

use std::hint::black_box;

const NUMBER_OF_ELEMENTS: usize = 50_000;

macro_rules! bench_sip {
    ($c:expr, $d:expr) => {
        paste::item! {
            fn [<bench_sip_ $c _ $d>] (b: &mut Criterion) {
                b.bench_function(
                    format!("sip_{}_{}", $c, $d).as_str(),
                    |b| {
                        b.iter(||{
                            black_box(for element in 0..NUMBER_OF_ELEMENTS {
                                let mut hasher = hyperloglog_rs::sip::Sip64Scalar::<$c, $d>::default();
                                element.hash(&mut hasher);
                                let _ = hasher.finish();
                            })
                        })
                });
            }
        }
    };
}

macro_rules! bench_sip_by_cs {
    ($($c:expr),*) => {
        $(
            bench_sip!($c, 4);
            bench_sip!($c, 8);
        )*
    };
}
bench_sip_by_cs!(1, 2, 3, 4);

fn bench_xx_hasher(b: &mut Criterion) {
    b.bench_function("xx_hasher", |b| {
        b.iter(|| {
            black_box(for element in 0..NUMBER_OF_ELEMENTS {
                let mut hasher = twox_hash::XxHash64::default();
                element.hash(&mut hasher);
                let _ = hasher.finish();
            })
        })
    });
}

criterion_group! {
    name=sip;
    config = Criterion::default();
    targets=bench_sip_1_4, bench_sip_1_8, bench_sip_2_4, bench_sip_2_8, bench_sip_3_4, bench_sip_3_8, bench_sip_4_4, bench_sip_4_8
}
criterion_group! {
    name=xx_hasher;
    config = Criterion::default().sample_size(500);
    targets=bench_xx_hasher
}

criterion_main!(sip, xx_hasher);
