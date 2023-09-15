#![feature(test)]
extern crate test;

use fasthash::MetroHasher;
use highway::HighwayHasher;
use hyperloglog_rs::prelude::*;

use siphasher::sip::{SipHasher13, SipHasher24};
use test::{black_box, Bencher};

const BITS: usize = 5;
const NUMBER_OF_ELEMENTS: usize = 10_000;

#[bench]
fn bench_estimation_siphasher13_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher13_multiplicity_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, SipHasher13> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

// BEGIN TESTING ON SIPHASHER24

#[bench]
fn bench_estimation_siphasher24_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_siphasher24_multiplicity_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, SipHasher24> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

// BEGIN TESTING ON HIGHWAYHASH

#[bench]
fn bench_estimation_highwayhasher_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_highwayhasher_multiplicity_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, HighwayHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_16(b: &mut Bencher) {
    // Optionally include some setup
    let mut hll: HyperLogLog<Precision4, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_32(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision5, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_64(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision6, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_128(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision7, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_256(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision8, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_512(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision9, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_1024(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision10, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_2048(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision11, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_4096(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision12, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_8192(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision13, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_16389(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision14, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_32768(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision15, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}

#[bench]
fn bench_estimation_metro_multiplicity_65536(b: &mut Bencher) {
    let mut hll: HyperLogLog<Precision16, BITS, MetroHasher> = HyperLogLog::new();

    b.iter(|| {
        black_box(for i in 0..NUMBER_OF_ELEMENTS {
            hll.insert(i);
            hll.estimate_cardinality_with_multiplicities();
        });
    });
}
