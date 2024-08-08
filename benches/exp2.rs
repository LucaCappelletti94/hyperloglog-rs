//! Criterion benchmarks comparing different approaches to compute 2^-x.

use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use hyperloglog_rs::prelude::FloatNumber;

const REPETITIONS: i64 = 20_000;
const MAXIMAL_NUMBER_OF_REGISTERS: u64 = 64;

fn bench_powi_f64(b: &mut Criterion) {
    b.bench_function("exp2_f64", |b| {
        b.iter(|| {
            let mut total: f64 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(2.0.powi(black_box(register_value) as i32));
                }
            }
            total
        })
    });
}

fn bench_powi_f32(b: &mut Criterion) {
    b.bench_function("exp2_f32", |b| {
        b.iter(|| {
            let mut total: f32 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(2.0.powi(black_box(register_value) as i32));
                }
            }
            total
        })
    });
}

fn bench_exp2_f64(b: &mut Criterion) {
    b.bench_function("exp2_f64", |b| {
        b.iter(|| {
            let mut total = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box((black_box(register_value) as f64).exp2());
                }
            }
            total
        })
    });
}

fn bench_exp2_f32(b: &mut Criterion) {
    b.bench_function("exp2_f32", |b| {
        b.iter(|| {
            let mut total = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box((black_box(register_value) as f32).exp2());
                }
            }
            total
        })
    });
}

fn bench_current_f32(b: &mut Criterion) {
    b.bench_function("current_f32", |b| {
        b.iter(|| {
            let mut total: f32 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(f32::inverse_register(black_box(register_value) as i32));
                }
            }
            total
        })
    });
}

fn bench_current_f64(b: &mut Criterion) {
    b.bench_function("current_f64", |b| {
        b.iter(|| {
            let mut total: f64 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(f64::inverse_register(black_box(register_value) as i32));
                }
            }
            total
        })
    });
}

fn bench_alec_f64(b: &mut Criterion) {
    b.bench_function("alec_f64", |b| {
        b.iter(|| {
            let mut total: f64 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(f64::from_bits(
                        u64::max_value().wrapping_sub(u64::from(black_box(register_value))) << 54
                            >> 2,
                    ))
                }
            }
            total
        })
    });
}

static LOOKUP_F64_EXP2: [f64; 64] = [
    1.0,
    0.5,
    0.25,
    0.125,
    0.0625,
    0.03125,
    0.015625,
    0.0078125,
    0.00390625,
    0.001953125,
    0.0009765625,
    0.00048828125,
    0.000244140625,
    0.00012207031,
    0.000061035156,
    0.000030517578,
    0.000015258789,
    0.0000076293945,
    0.0000038146973,
    0.0000019073486,
    0.00000095367432,
    0.00000047683716,
    0.00000023841858,
    0.00000011920929,
    0.000000059604645,
    0.000000029802322,
    0.000000014901161,
    0.0000000074505806,
    0.0000000037252903,
    0.0000000018626451,
    0.00000000093132257,
    0.00000000046566128,
    0.00000000023283064,
    0.00000000011641532,
    0.000000000058207661,
    0.000000000029103831,
    0.000000000014551915,
    0.0000000000072759575,
    0.0000000000036379787,
    0.0000000000018189893,
    0.00000000000090949467,
    0.00000000000045474734,
    0.00000000000022737367,
    0.00000000000011368684,
    0.00000000000005684342,
    0.00000000000002842171,
    0.000000000000014210854,
    0.000000000000007105427,
    0.0000000000000035527135,
    0.0000000000000017763568,
    0.0000000000000008881784,
    0.0000000000000004440892,
    0.0000000000000002220446,
    0.0000000000000001110223,
    0.00000000000000005551122,
    0.00000000000000002775561,
    0.000000000000000013877805,
    0.0000000000000000069389023,
    0.0000000000000000034694512,
    0.0000000000000000017347256,
    0.0000000000000000008673628,
    0.0000000000000000004336814,
    0.0000000000000000002168407,
    0.00000000000000000010842035,
];

static LOOKUP_F32_EXP2: [f32; 64] = [
    1.0,
    0.5,
    0.25,
    0.125,
    0.0625,
    0.03125,
    0.015625,
    0.0078125,
    0.00390625,
    0.001953125,
    0.0009765625,
    0.00048828125,
    0.000244140625,
    0.00012207031,
    0.000061035156,
    0.000030517578,
    0.000015258789,
    0.0000076293945,
    0.0000038146973,
    0.0000019073486,
    0.00000095367432,
    0.00000047683716,
    0.00000023841858,
    0.00000011920929,
    0.000000059604645,
    0.000000029802322,
    0.000000014901161,
    0.0000000074505806,
    0.0000000037252903,
    0.0000000018626451,
    0.00000000093132257,
    0.00000000046566128,
    0.00000000023283064,
    0.00000000011641532,
    0.000000000058207661,
    0.000000000029103831,
    0.000000000014551915,
    0.0000000000072759575,
    0.0000000000036379787,
    0.0000000000018189893,
    0.00000000000090949467,
    0.00000000000045474734,
    0.00000000000022737367,
    0.00000000000011368684,
    0.00000000000005684342,
    0.00000000000002842171,
    0.000000000000014210854,
    0.000000000000007105427,
    0.0000000000000035527135,
    0.0000000000000017763568,
    0.0000000000000008881784,
    0.0000000000000004440892,
    0.0000000000000002220446,
    0.0000000000000001110223,
    0.00000000000000005551122,
    0.00000000000000002775561,
    0.000000000000000013877805,
    0.0000000000000000069389023,
    0.0000000000000000034694512,
    0.0000000000000000017347256,
    0.0000000000000000008673628,
    0.0000000000000000004336814,
    0.0000000000000000002168407,
    0.00000000000000000010842035,
];

fn bench_lookup_f64(b: &mut Criterion) {
    b.bench_function("lookup_f64", |b| {
        b.iter(|| {
            let mut total: f64 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(LOOKUP_F64_EXP2[black_box(register_value) as usize]);
                }
            }
            total
        })
    });
}

fn bench_lookup_f32(b: &mut Criterion) {
    b.bench_function("lookup_f32", |b| {
        b.iter(|| {
            let mut total: f32 = 0.0;
            for _ in 0..REPETITIONS {
                for register_value in 0..MAXIMAL_NUMBER_OF_REGISTERS {
                    total += black_box(LOOKUP_F32_EXP2[black_box(register_value) as usize]);
                }
            }
            total
        })
    });
}

criterion_group! {
    name=exp2;
    config = Criterion::default();
    targets=bench_exp2_f64, bench_exp2_f32, bench_current_f32, bench_current_f64, bench_alec_f64, bench_powi_f64, bench_powi_f32, bench_lookup_f64, bench_lookup_f32
}

criterion_main!(exp2);
