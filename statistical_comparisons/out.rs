#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod cardinality {
    //! Comparative test making sure that the quality of the estimation
    //! obtain by the hyperloglog-rs library is equal or better than the
    //! competitor libraries.
    //! Evaluation of set-like properties across different data structures.
    use crate::utils::*;
    use core::f64;
    use hyperloglog_rs::prelude::*;
    use indicatif::ParallelProgressIterator;
    use indicatif::ProgressBar;
    use indicatif::ProgressStyle;
    use mem_dbg::MemSize;
    use mem_dbg::SizeFlags;
    use rayon::prelude::*;
    pub(super) fn cardinality_comparatively<
        const EXPONENT: usize,
        P: mem_dbg::MemSize + Precision + ArrayRegister<Bits8> + ArrayRegister<Bits6>
            + ArrayRegister<Bits5> + ArrayRegister<Bits4>,
    >()
    where
        <P as ArrayRegister<Bits8>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfRegisters: mem_dbg::MemSize,
    {
        let path = ::alloc::__export::must_use({
            let res = ::alloc::fmt::format(
                format_args!(
                    "./statistical_tests_reports/cardinality_{0}.csv",
                    P::EXPONENT,
                ),
            );
            res
        });
        let path = std::path::Path::new(&path);
        if path.exists() {
            {
                ::std::io::_print(
                    format_args!("Skipping evaluation cardinality {0}\n", P::EXPONENT),
                );
            };
            return;
        }
        {
            ::std::io::_print(
                format_args!("Running evaluation cardinality {0}\n", P::EXPONENT),
            );
        };
        match (&P::EXPONENT, &EXPONENT) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        let number_of_vectors = 2000;
        let minimum_sample_interval = 5;
        let maximum_sample_interval = 1000;
        let random_state = splitmix64(9516748163234878233_u64);
        let progress_bar = ProgressBar::new(number_of_vectors as u64);
        progress_bar
            .set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .unwrap()
                    .progress_chars("##-"),
            );
        let (
            exact_cardinalities,
            mean_errors,
            memory_requirements,
        ): (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<usize>>) = (0..number_of_vectors)
            .into_par_iter()
            .progress_with(progress_bar)
            .flat_map(|thread_number| {
                let random_state = splitmix64(
                    splitmix64(random_state.wrapping_mul(thread_number + 1)),
                );
                let mut exact_cardinalities = Vec::new();
                let mut all_memory_requirements = Vec::new();
                let mut all_mean_errors = Vec::new();
                let mut all_objects = SetLikeObjects::<EXPONENT, P>::all_cardinalities();
                let mut current_sample_rate = minimum_sample_interval;
                for (i, l) in iter_random_values(2_000_000, None, random_state)
                    .enumerate()
                {
                    all_objects
                        .iter_mut()
                        .for_each(|object| {
                            <SetLikeObjects<
                                EXPONENT,
                                P,
                            > as TestSetLike<u64>>::insert(object, &l)
                        });
                    if i % current_sample_rate == 0 {
                        if current_sample_rate < maximum_sample_interval {
                            current_sample_rate *= 2;
                        }
                        let (
                            cardinalities,
                            memory_requirements,
                        ): (Vec<f64>, Vec<usize>) = all_objects
                            .iter()
                            .map(|object| {
                                let cardinality = object.cardinality();
                                let memory_requirement = object
                                    .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                                (cardinality, memory_requirement)
                            })
                            .unzip();
                        let mean_errors = cardinalities
                            .iter()
                            .map(|v| (v - cardinalities[0]).abs() / cardinalities[0])
                            .collect();
                        exact_cardinalities.push(cardinalities[0]);
                        all_memory_requirements.push(memory_requirements);
                        all_mean_errors.push(mean_errors);
                    }
                }
                (exact_cardinalities, all_mean_errors, all_memory_requirements)
            })
            .fold(
                || (Vec::new(), Vec::new(), Vec::new()),
                |
                    (mut cardinalities, mut mean_errors, mut memory_requirements),
                    (c, me, mr)|
                {
                    cardinalities.push(c);
                    mean_errors.push(me);
                    memory_requirements.push(mr);
                    (cardinalities, mean_errors, memory_requirements)
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new()),
                |
                    (mut cardinalities, mut mean_errors, mut memory_requirements),
                    (c, me, mr)|
                {
                    cardinalities.extend(c);
                    mean_errors.extend(me);
                    memory_requirements.extend(mr);
                    (cardinalities, mean_errors, memory_requirements)
                },
            );
        let names = SetLikeObjects::<EXPONENT, P>::all_cardinalities()
            .iter()
            .map(|object| object.name())
            .collect::<Vec<_>>();
        statistical_report::<
            P,
        >(&names, exact_cardinalities, mean_errors, memory_requirements, "cardinality");
    }
}
pub mod union {
    //! Comparative test making sure that the quality of the estimation
    //! obtain by the hyperloglog-rs library is equal or better than the
    //! competitor libraries.
    //! Evaluation of set-like properties across different data structures.
    use crate::utils::*;
    use hyperloglog_rs::prelude::*;
    use core::f64;
    use indicatif::ParallelProgressIterator;
    use indicatif::ProgressBar;
    use indicatif::ProgressStyle;
    use mem_dbg::MemSize;
    use mem_dbg::SizeFlags;
    use rayon::prelude::*;
    pub(super) fn union_comparatively<
        const EXPONENT: usize,
        P: mem_dbg::MemSize + Precision + ArrayRegister<Bits8> + ArrayRegister<Bits6>
            + ArrayRegister<Bits5> + ArrayRegister<Bits4> + ArrayRegister<Bits5>,
    >()
    where
        <P as ArrayRegister<Bits8>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits6>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits5>>::ArrayRegister: mem_dbg::MemSize,
        <P as ArrayRegister<Bits4>>::ArrayRegister: mem_dbg::MemSize,
        <P as hyperloglog_rs::prelude::Precision>::NumberOfRegisters: mem_dbg::MemSize,
    {
        let path = ::alloc::__export::must_use({
            let res = ::alloc::fmt::format(
                format_args!("./statistical_tests_reports/union_{0}.csv", P::EXPONENT),
            );
            res
        });
        let path = std::path::Path::new(&path);
        if path.exists() {
            {
                ::std::io::_print(
                    format_args!("Skipping evaluation union {0}\n", P::EXPONENT),
                );
            };
            return;
        }
        {
            ::std::io::_print(
                format_args!("Running evaluation union {0}\n", P::EXPONENT),
            );
        };
        match (&P::EXPONENT, &EXPONENT) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        let number_of_vectors = 3_00;
        let minimum_number_of_samples = 5;
        let maximum_number_of_samples = 2000;
        let left_random_state = splitmix64(6516781878233_u64);
        let right_random_state = splitmix64(497635734233_u64);
        let progress_bar = ProgressBar::new(number_of_vectors as u64);
        progress_bar
            .set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                    .unwrap()
                    .progress_chars("##-"),
            );
        let (
            unions,
            mean_errors,
            memory_requirements,
        ): (Vec<f64>, Vec<Vec<f64>>, Vec<Vec<usize>>) = (0..number_of_vectors)
            .into_par_iter()
            .progress_with(progress_bar)
            .flat_map(|thread_number| {
                let left_random_state = splitmix64(
                    splitmix64(left_random_state.wrapping_mul(thread_number + 1)),
                );
                let right_random_state = splitmix64(
                    splitmix64(right_random_state.wrapping_mul(thread_number + 1)),
                );
                let mut exact_unions: Vec<f64> = Vec::new();
                let mut all_memory_requirements: Vec<Vec<usize>> = Vec::new();
                let mut all_mean_errors: Vec<Vec<f64>> = Vec::new();
                let mut left_objects = SetLikeObjects::<EXPONENT, P>::all_union();
                let mut right_objects = SetLikeObjects::<EXPONENT, P>::all_union();
                let mut left_iter = iter_random_values(
                    2_000_000,
                    Some(1_000_000),
                    left_random_state,
                );
                let mut right_iter = iter_random_values(
                    2_000_000,
                    Some(1_000_000),
                    right_random_state,
                );
                let mut current_sample_rate = minimum_number_of_samples;
                let mut i = 0;
                loop {
                    let mut new_object = false;
                    if let Some(left) = left_iter.next() {
                        left_objects
                            .iter_mut()
                            .for_each(|object| {
                                <SetLikeObjects<
                                    EXPONENT,
                                    P,
                                > as TestSetLike<u64>>::insert(object, &left)
                            });
                        new_object = true;
                    }
                    if let Some(right) = right_iter.next() {
                        right_objects
                            .iter_mut()
                            .for_each(|object| {
                                <SetLikeObjects<
                                    EXPONENT,
                                    P,
                                > as TestSetLike<u64>>::insert(object, &right)
                            });
                        new_object = true;
                    }
                    if !new_object {
                        break;
                    }
                    if i % current_sample_rate == 0 {
                        if current_sample_rate < maximum_number_of_samples {
                            current_sample_rate *= 2;
                        }
                        let (
                            cardinalities,
                            memory_requirements,
                        ): (Vec<f64>, Vec<usize>) = left_objects
                            .iter()
                            .zip(right_objects.iter())
                            .map(|(left, right)| {
                                let cardinality = left.union(right);
                                let memory_requirement = left
                                    .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS)
                                    + right
                                        .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                                (cardinality, memory_requirement)
                            })
                            .unzip();
                        let mean_errors = cardinalities
                            .iter()
                            .map(|v| (v - cardinalities[0]).abs() / cardinalities[0])
                            .collect();
                        exact_unions.push(cardinalities[0]);
                        all_memory_requirements.push(memory_requirements);
                        all_mean_errors.push(mean_errors);
                    }
                    i += 1;
                }
                (exact_unions, all_mean_errors, all_memory_requirements)
            })
            .fold(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(mut unions, mut mean_errors, mut memory_requirements), (u, me, mr)| {
                    unions.push(u);
                    mean_errors.push(me);
                    memory_requirements.push(mr);
                    (unions, mean_errors, memory_requirements)
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(mut unions, mut mean_errors, mut memory_requirements), (u, me, mr)| {
                    unions.extend(u);
                    mean_errors.extend(me);
                    memory_requirements.extend(mr);
                    (unions, mean_errors, memory_requirements)
                },
            );
        let names = SetLikeObjects::<EXPONENT, P>::all_union()
            .iter()
            .map(|object| object.name())
            .collect::<Vec<_>>();
        statistical_report::<
            P,
        >(&names, unions, mean_errors, memory_requirements, "union");
    }
}
pub mod utils {
    use std::fmt::Display;
    use std::hash::RandomState;
    use hyperloglog_rs::prelude::*;
    use indicatif::ProgressIterator;
    use mem_dbg::{MemSize, MemDbg};
    use stattest::test::StatisticalTest;
    use stattest::test::WilcoxonWTest;
    use cardinality_estimator::CardinalityEstimator;
    use hyperloglog_rs::prelude::{
        Estimator, ExtendableApproximatedSet, HasherType, Precision,
    };
    use hyperloglogplus::HyperLogLog as TabacHyperLogLog;
    use hyperloglogplus::HyperLogLogPF as TabacHyperLogLogPF;
    use hyperloglogplus::HyperLogLogPlus as TabacHyperLogLogPlus;
    use hypertwobits::h2b::HyperTwoBits as H2B;
    use hypertwobits::h2b::{
        M1024 as M1024H2B, M128 as M128H2B, M2048 as M2048H2B, M256 as M256H2B,
        M4096 as M4096H2B, M512 as M512H2B, M64 as M64H2B,
    };
    use hypertwobits::h3b::HyperThreeBits as H3B;
    use hypertwobits::h3b::{
        M1024 as M1024H3B, M128 as M128H3B, M2048 as M2048H3B, M256 as M256H3B,
        M4096 as M4096H3B, M512 as M512H3B, M64 as M64H3B,
    };
    use rust_hyperloglog::HyperLogLog as RustHyperLogLog;
    use simple_hll::HyperLogLog as SimpleHyperLogLog;
    use sourmash::signature::SigsTrait;
    use sourmash::sketch::hyperloglog::HyperLogLog as SourMashHyperLogLog;
    use std::marker::PhantomData;
    use std::usize;
    use streaming_algorithms::HyperLogLog as SAHyperLogLog;
    use macro_test_utils::*;
    struct SimpleHLL<const P: usize> {
        estimator: SimpleHyperLogLog<P>,
    }
    #[automatically_derived]
    impl<const P: usize> ::core::fmt::Debug for SimpleHLL<P> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "SimpleHLL",
                "estimator",
                &&self.estimator,
            )
        }
    }
    #[automatically_derived]
    impl<const P: usize> ::core::clone::Clone for SimpleHLL<P> {
        #[inline]
        fn clone(&self) -> SimpleHLL<P> {
            SimpleHLL {
                estimator: ::core::clone::Clone::clone(&self.estimator),
            }
        }
    }
    #[automatically_derived]
    impl<const P: usize> ::core::default::Default for SimpleHLL<P> {
        #[inline]
        fn default() -> SimpleHLL<P> {
            SimpleHLL {
                estimator: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl<const P: usize> mem_dbg::MemDbgImpl for SimpleHLL<P>
    where
        SimpleHyperLogLog<P>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes.push((0usize, { builtin # offset_of(SimpleHLL < P >, estimator) }));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <SimpleHyperLogLog<
                            P,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<const P: usize> mem_dbg::CopyType for SimpleHLL<P>
    where
        SimpleHyperLogLog<P>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<const P: usize> mem_dbg::MemSize for SimpleHLL<P>
    where
        SimpleHyperLogLog<P>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <SimpleHyperLogLog<
                    P,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<SimpleHyperLogLog<P>>();
            bytes
        }
    }
    struct CloudFlareHLL<const P: usize, const B: usize, H: HasherType> {
        estimator: CardinalityEstimator<u64, H, P, B>,
    }
    #[automatically_derived]
    impl<
        const P: usize,
        const B: usize,
        H: ::core::fmt::Debug + HasherType,
    > ::core::fmt::Debug for CloudFlareHLL<P, B, H> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "CloudFlareHLL",
                "estimator",
                &&self.estimator,
            )
        }
    }
    #[automatically_derived]
    impl<
        const P: usize,
        const B: usize,
        H: ::core::clone::Clone + HasherType,
    > ::core::clone::Clone for CloudFlareHLL<P, B, H> {
        #[inline]
        fn clone(&self) -> CloudFlareHLL<P, B, H> {
            CloudFlareHLL {
                estimator: ::core::clone::Clone::clone(&self.estimator),
            }
        }
    }
    #[automatically_derived]
    impl<
        const P: usize,
        const B: usize,
        H: ::core::default::Default + HasherType,
    > ::core::default::Default for CloudFlareHLL<P, B, H> {
        #[inline]
        fn default() -> CloudFlareHLL<P, B, H> {
            CloudFlareHLL {
                estimator: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl<const P: usize, const B: usize, H: HasherType> mem_dbg::MemDbgImpl
    for CloudFlareHLL<P, B, H>
    where
        CardinalityEstimator<u64, H, P, B>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes
                .push((
                    0usize,
                    { builtin # offset_of(CloudFlareHLL < P, B, H >, estimator) },
                ));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <CardinalityEstimator<
                            u64,
                            H,
                            P,
                            B,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<const P: usize, const B: usize, H: HasherType> mem_dbg::CopyType
    for CloudFlareHLL<P, B, H>
    where
        CardinalityEstimator<u64, H, P, B>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<const P: usize, const B: usize, H: HasherType> mem_dbg::MemSize
    for CloudFlareHLL<P, B, H>
    where
        CardinalityEstimator<u64, H, P, B>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <CardinalityEstimator<
                    u64,
                    H,
                    P,
                    B,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<CardinalityEstimator<u64, H, P, B>>();
            bytes
        }
    }
    struct HyperTwoBits<S: hypertwobits::h2b::Sketch> {
        estimator: H2B<S>,
    }
    #[automatically_derived]
    impl<S: ::core::fmt::Debug + hypertwobits::h2b::Sketch> ::core::fmt::Debug
    for HyperTwoBits<S> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "HyperTwoBits",
                "estimator",
                &&self.estimator,
            )
        }
    }
    #[automatically_derived]
    impl<S: ::core::clone::Clone + hypertwobits::h2b::Sketch> ::core::clone::Clone
    for HyperTwoBits<S> {
        #[inline]
        fn clone(&self) -> HyperTwoBits<S> {
            HyperTwoBits {
                estimator: ::core::clone::Clone::clone(&self.estimator),
            }
        }
    }
    #[automatically_derived]
    impl<
        S: ::core::default::Default + hypertwobits::h2b::Sketch,
    > ::core::default::Default for HyperTwoBits<S> {
        #[inline]
        fn default() -> HyperTwoBits<S> {
            HyperTwoBits {
                estimator: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl<S: hypertwobits::h2b::Sketch> mem_dbg::MemDbgImpl for HyperTwoBits<S>
    where
        H2B<S>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes
                .push((0usize, { builtin # offset_of(HyperTwoBits < S >, estimator) }));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <H2B<
                            S,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<S: hypertwobits::h2b::Sketch> mem_dbg::CopyType for HyperTwoBits<S>
    where
        H2B<S>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<S: hypertwobits::h2b::Sketch> mem_dbg::MemSize for HyperTwoBits<S>
    where
        H2B<S>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <H2B<
                    S,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<H2B<S>>();
            bytes
        }
    }
    struct HyperThreeBits<S: hypertwobits::h3b::Sketch> {
        estimator: H3B<S>,
    }
    #[automatically_derived]
    impl<S: ::core::fmt::Debug + hypertwobits::h3b::Sketch> ::core::fmt::Debug
    for HyperThreeBits<S> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "HyperThreeBits",
                "estimator",
                &&self.estimator,
            )
        }
    }
    #[automatically_derived]
    impl<S: ::core::clone::Clone + hypertwobits::h3b::Sketch> ::core::clone::Clone
    for HyperThreeBits<S> {
        #[inline]
        fn clone(&self) -> HyperThreeBits<S> {
            HyperThreeBits {
                estimator: ::core::clone::Clone::clone(&self.estimator),
            }
        }
    }
    #[automatically_derived]
    impl<
        S: ::core::default::Default + hypertwobits::h3b::Sketch,
    > ::core::default::Default for HyperThreeBits<S> {
        #[inline]
        fn default() -> HyperThreeBits<S> {
            HyperThreeBits {
                estimator: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl<S: hypertwobits::h3b::Sketch> mem_dbg::MemDbgImpl for HyperThreeBits<S>
    where
        H3B<S>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes
                .push((
                    0usize,
                    { builtin # offset_of(HyperThreeBits < S >, estimator) },
                ));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <H3B<
                            S,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<S: hypertwobits::h3b::Sketch> mem_dbg::CopyType for HyperThreeBits<S>
    where
        H3B<S>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<S: hypertwobits::h3b::Sketch> mem_dbg::MemSize for HyperThreeBits<S>
    where
        H3B<S>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <H3B<
                    S,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<H3B<S>>();
            bytes
        }
    }
    struct SourMash<P: Precision> {
        estimator: SourMashHyperLogLog,
        _precision: PhantomData<P>,
    }
    #[automatically_derived]
    impl<P: ::core::fmt::Debug + Precision> ::core::fmt::Debug for SourMash<P> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "SourMash",
                "estimator",
                &self.estimator,
                "_precision",
                &&self._precision,
            )
        }
    }
    #[automatically_derived]
    impl<P: ::core::clone::Clone + Precision> ::core::clone::Clone for SourMash<P> {
        #[inline]
        fn clone(&self) -> SourMash<P> {
            SourMash {
                estimator: ::core::clone::Clone::clone(&self.estimator),
                _precision: ::core::clone::Clone::clone(&self._precision),
            }
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemDbgImpl for SourMash<P>
    where
        SourMashHyperLogLog: mem_dbg::MemDbgImpl,
        PhantomData<P>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes.push((0usize, { builtin # offset_of(SourMash < P >, estimator) }));
            id_sizes.push((1usize, { builtin # offset_of(SourMash < P >, _precision) }));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <SourMashHyperLogLog as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    1usize => {
                        <PhantomData<
                            P,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self._precision,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("_precision"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::CopyType for SourMash<P>
    where
        SourMashHyperLogLog: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemSize for SourMash<P>
    where
        SourMashHyperLogLog: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <SourMashHyperLogLog as mem_dbg::MemSize>::mem_size(
                    &self.estimator,
                    _memsize_flags,
                ) - core::mem::size_of::<SourMashHyperLogLog>();
            bytes
                += <PhantomData<
                    P,
                > as mem_dbg::MemSize>::mem_size(&self._precision, _memsize_flags)
                    - core::mem::size_of::<PhantomData<P>>();
            bytes
        }
    }
    impl<P: Precision> Default for SourMash<P> {
        fn default() -> Self {
            Self {
                estimator: SourMashHyperLogLog::new(P::EXPONENT as usize, usize::MAX)
                    .unwrap(),
                _precision: PhantomData,
            }
        }
    }
    struct RustHLL<P: Precision> {
        estimator: RustHyperLogLog,
        _precision: PhantomData<P>,
    }
    #[automatically_derived]
    impl<P: ::core::fmt::Debug + Precision> ::core::fmt::Debug for RustHLL<P> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "RustHLL",
                "estimator",
                &self.estimator,
                "_precision",
                &&self._precision,
            )
        }
    }
    #[automatically_derived]
    impl<P: ::core::clone::Clone + Precision> ::core::clone::Clone for RustHLL<P> {
        #[inline]
        fn clone(&self) -> RustHLL<P> {
            RustHLL {
                estimator: ::core::clone::Clone::clone(&self.estimator),
                _precision: ::core::clone::Clone::clone(&self._precision),
            }
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemDbgImpl for RustHLL<P>
    where
        RustHyperLogLog: mem_dbg::MemDbgImpl,
        PhantomData<P>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes.push((0usize, { builtin # offset_of(RustHLL < P >, estimator) }));
            id_sizes.push((1usize, { builtin # offset_of(RustHLL < P >, _precision) }));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <RustHyperLogLog as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    1usize => {
                        <PhantomData<
                            P,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self._precision,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("_precision"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::CopyType for RustHLL<P>
    where
        RustHyperLogLog: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemSize for RustHLL<P>
    where
        RustHyperLogLog: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <RustHyperLogLog as mem_dbg::MemSize>::mem_size(
                    &self.estimator,
                    _memsize_flags,
                ) - core::mem::size_of::<RustHyperLogLog>();
            bytes
                += <PhantomData<
                    P,
                > as mem_dbg::MemSize>::mem_size(&self._precision, _memsize_flags)
                    - core::mem::size_of::<PhantomData<P>>();
            bytes
        }
    }
    impl<P: Precision> Default for RustHLL<P> {
        fn default() -> Self {
            Self {
                estimator: RustHyperLogLog::new_deterministic(
                    P::error_rate(),
                    6755343421867645123_u128,
                ),
                _precision: PhantomData,
            }
        }
    }
    struct TabacHLLPlusPlus<P: Precision> {
        estimator: TabacHyperLogLogPlus<u64, RandomState>,
        _precision: PhantomData<P>,
    }
    #[automatically_derived]
    impl<P: ::core::clone::Clone + Precision> ::core::clone::Clone
    for TabacHLLPlusPlus<P> {
        #[inline]
        fn clone(&self) -> TabacHLLPlusPlus<P> {
            TabacHLLPlusPlus {
                estimator: ::core::clone::Clone::clone(&self.estimator),
                _precision: ::core::clone::Clone::clone(&self._precision),
            }
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemDbgImpl for TabacHLLPlusPlus<P>
    where
        TabacHyperLogLogPlus<u64, RandomState>: mem_dbg::MemDbgImpl,
        PhantomData<P>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes
                .push((
                    0usize,
                    { builtin # offset_of(TabacHLLPlusPlus < P >, estimator) },
                ));
            id_sizes
                .push((
                    1usize,
                    { builtin # offset_of(TabacHLLPlusPlus < P >, _precision) },
                ));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <TabacHyperLogLogPlus<
                            u64,
                            RandomState,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    1usize => {
                        <PhantomData<
                            P,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self._precision,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("_precision"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::CopyType for TabacHLLPlusPlus<P>
    where
        TabacHyperLogLogPlus<u64, RandomState>: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemSize for TabacHLLPlusPlus<P>
    where
        TabacHyperLogLogPlus<u64, RandomState>: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <TabacHyperLogLogPlus<
                    u64,
                    RandomState,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<TabacHyperLogLogPlus<u64, RandomState>>();
            bytes
                += <PhantomData<
                    P,
                > as mem_dbg::MemSize>::mem_size(&self._precision, _memsize_flags)
                    - core::mem::size_of::<PhantomData<P>>();
            bytes
        }
    }
    impl<P: Precision> Default for TabacHLLPlusPlus<P> {
        fn default() -> Self {
            Self {
                estimator: TabacHyperLogLogPlus::new(
                        P::EXPONENT as u8,
                        RandomState::default(),
                    )
                    .unwrap(),
                _precision: PhantomData,
            }
        }
    }
    struct TabacHLL<P: Precision> {
        estimator: TabacHyperLogLogPF<u64, RandomState>,
        _precision: PhantomData<P>,
    }
    #[automatically_derived]
    impl<P: ::core::clone::Clone + Precision> ::core::clone::Clone for TabacHLL<P> {
        #[inline]
        fn clone(&self) -> TabacHLL<P> {
            TabacHLL {
                estimator: ::core::clone::Clone::clone(&self.estimator),
                _precision: ::core::clone::Clone::clone(&self._precision),
            }
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemDbgImpl for TabacHLL<P>
    where
        TabacHyperLogLogPF<u64, RandomState>: mem_dbg::MemDbgImpl,
        PhantomData<P>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes.push((0usize, { builtin # offset_of(TabacHLL < P >, estimator) }));
            id_sizes.push((1usize, { builtin # offset_of(TabacHLL < P >, _precision) }));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <TabacHyperLogLogPF<
                            u64,
                            RandomState,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    1usize => {
                        <PhantomData<
                            P,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self._precision,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("_precision"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::CopyType for TabacHLL<P>
    where
        TabacHyperLogLogPF<u64, RandomState>: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemSize for TabacHLL<P>
    where
        TabacHyperLogLogPF<u64, RandomState>: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <TabacHyperLogLogPF<
                    u64,
                    RandomState,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<TabacHyperLogLogPF<u64, RandomState>>();
            bytes
                += <PhantomData<
                    P,
                > as mem_dbg::MemSize>::mem_size(&self._precision, _memsize_flags)
                    - core::mem::size_of::<PhantomData<P>>();
            bytes
        }
    }
    impl<P: Precision> Default for TabacHLL<P> {
        fn default() -> Self {
            Self {
                estimator: TabacHyperLogLogPF::new(
                        P::EXPONENT as u8,
                        RandomState::default(),
                    )
                    .unwrap(),
                _precision: PhantomData,
            }
        }
    }
    struct SAHLL<P: Precision> {
        estimator: SAHyperLogLog<u64>,
        _precision: PhantomData<P>,
    }
    #[automatically_derived]
    impl<P: ::core::fmt::Debug + Precision> ::core::fmt::Debug for SAHLL<P> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "SAHLL",
                "estimator",
                &self.estimator,
                "_precision",
                &&self._precision,
            )
        }
    }
    #[automatically_derived]
    impl<P: ::core::clone::Clone + Precision> ::core::clone::Clone for SAHLL<P> {
        #[inline]
        fn clone(&self) -> SAHLL<P> {
            SAHLL {
                estimator: ::core::clone::Clone::clone(&self.estimator),
                _precision: ::core::clone::Clone::clone(&self._precision),
            }
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemDbgImpl for SAHLL<P>
    where
        SAHyperLogLog<u64>: mem_dbg::MemDbgImpl,
        PhantomData<P>: mem_dbg::MemDbgImpl,
    {
        #[inline(always)]
        fn _mem_dbg_rec_on(
            &self,
            _memdbg_writer: &mut impl core::fmt::Write,
            _memdbg_total_size: usize,
            _memdbg_max_depth: usize,
            _memdbg_prefix: &mut String,
            _memdbg_is_last: bool,
            _memdbg_flags: mem_dbg::DbgFlags,
        ) -> core::fmt::Result {
            let mut id_sizes: Vec<(usize, usize)> = ::alloc::vec::Vec::new();
            id_sizes.push((0usize, { builtin # offset_of(SAHLL < P >, estimator) }));
            id_sizes.push((1usize, { builtin # offset_of(SAHLL < P >, _precision) }));
            let n = id_sizes.len();
            id_sizes.push((n, core::mem::size_of::<Self>()));
            id_sizes.sort_by_key(|x| x.1);
            for i in 0..n {
                id_sizes[i].1 = id_sizes[i + 1].1 - id_sizes[i].1;
            }
            if !_memdbg_flags.contains(mem_dbg::DbgFlags::RUST_LAYOUT) {
                id_sizes.sort_by_key(|x| x.0);
            }
            for (i, (field_idx, padded_size)) in id_sizes.into_iter().enumerate().take(n)
            {
                match field_idx {
                    0usize => {
                        <SAHyperLogLog<
                            u64,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self.estimator,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("estimator"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    1usize => {
                        <PhantomData<
                            P,
                        > as mem_dbg::MemDbgImpl>::_mem_dbg_depth_on(
                            &self._precision,
                            _memdbg_writer,
                            _memdbg_total_size,
                            _memdbg_max_depth,
                            _memdbg_prefix,
                            Some("_precision"),
                            i == n - 1,
                            padded_size,
                            _memdbg_flags,
                        )?
                    }
                    _ => {
                        ::core::panicking::panic(
                            "internal error: entered unreachable code",
                        )
                    }
                }
            }
            Ok(())
        }
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::CopyType for SAHLL<P>
    where
        SAHyperLogLog<u64>: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        type Copy = mem_dbg::False;
    }
    #[automatically_derived]
    impl<P: Precision> mem_dbg::MemSize for SAHLL<P>
    where
        SAHyperLogLog<u64>: mem_dbg::MemSize,
        PhantomData<P>: mem_dbg::MemSize,
    {
        fn mem_size(&self, _memsize_flags: mem_dbg::SizeFlags) -> usize {
            let mut bytes = core::mem::size_of::<Self>();
            bytes
                += <SAHyperLogLog<
                    u64,
                > as mem_dbg::MemSize>::mem_size(&self.estimator, _memsize_flags)
                    - core::mem::size_of::<SAHyperLogLog<u64>>();
            bytes
                += <PhantomData<
                    P,
                > as mem_dbg::MemSize>::mem_size(&self._precision, _memsize_flags)
                    - core::mem::size_of::<PhantomData<P>>();
            bytes
        }
    }
    impl<P: Precision> Default for SAHLL<P> {
        fn default() -> Self {
            Self {
                estimator: SAHyperLogLog::new(P::error_rate()),
                _precision: PhantomData,
            }
        }
    }
    #[cfg(feature = "std")]
    impl<const P: usize> hyperloglog_rs::prelude::Named for SimpleHLL<P> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(format_args!("SHLL<P{0}, B8, Vec>", P));
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<S: hypertwobits::h2b::Sketch> hyperloglog_rs::prelude::Named
    for HyperTwoBits<S> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "H2B<{0}>",
                        std::any::type_name::<S>().split("::").last().unwrap(),
                    ),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<S: hypertwobits::h3b::Sketch> hyperloglog_rs::prelude::Named
    for HyperThreeBits<S> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "H3B<{0}>",
                        std::any::type_name::<S>().split("::").last().unwrap(),
                    ),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<P: Precision> hyperloglog_rs::prelude::Named for SourMash<P> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("SM<P{0}, B8, Vec>", P::EXPONENT),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<H: HasherType, const P: usize, const B: usize> hyperloglog_rs::prelude::Named
    for CloudFlareHLL<P, B, H> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "CF<P{0}, B{1}, Mix> + {2}",
                        P,
                        B,
                        std::any::type_name::<H>().split("::").last().unwrap(),
                    ),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<P: Precision> hyperloglog_rs::prelude::Named for RustHLL<P> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("FrankPP<P{0}, B8, Vec> + SipHasher13", P::EXPONENT),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<P: Precision> hyperloglog_rs::prelude::Named for TabacHLLPlusPlus<P> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("TabacPP<P{0}, B6, Vec> + XxHash64", P::EXPONENT),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<P: Precision> hyperloglog_rs::prelude::Named for TabacHLL<P> {
        fn name(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("Tabac<P{0}, B6, Vec> + XxHash64", P::EXPONENT),
                );
                res
            })
        }
    }
    #[cfg(feature = "std")]
    impl<P: Precision> hyperloglog_rs::prelude::Named for SAHLL<P> {
        fn name(&self) -> String {
            match (&(P::EXPONENT as u8), &self.estimator.precision()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "SA<P{0}, B6, Vec> + XxHash64",
                        self.estimator.precision(),
                    ),
                );
                res
            })
        }
    }
    impl<const P: usize> ExtendableApproximatedSet<u64> for SimpleHLL<P> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.add_object(item);
            true
        }
    }
    impl<S: hypertwobits::h2b::Sketch> ExtendableApproximatedSet<u64>
    for HyperTwoBits<S> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.insert(item);
            true
        }
    }
    impl<S: hypertwobits::h3b::Sketch> ExtendableApproximatedSet<u64>
    for HyperThreeBits<S> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.insert(item);
            true
        }
    }
    impl<P: Precision> ExtendableApproximatedSet<u64> for SourMash<P> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.add_sequence(item.to_le_bytes().as_ref(), false).unwrap();
            true
        }
    }
    impl<H: HasherType, const P: usize, const B: usize> ExtendableApproximatedSet<u64>
    for CloudFlareHLL<P, B, H> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.insert(item);
            true
        }
    }
    impl<P: Precision> ExtendableApproximatedSet<u64> for RustHLL<P> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.insert(item);
            true
        }
    }
    impl<P: Precision> ExtendableApproximatedSet<u64> for TabacHLLPlusPlus<P> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.insert(item);
            true
        }
    }
    impl<P: Precision> ExtendableApproximatedSet<u64> for TabacHLL<P> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.insert(item);
            true
        }
    }
    impl<P: Precision> ExtendableApproximatedSet<u64> for SAHLL<P> {
        fn insert(&mut self, item: &u64) -> bool {
            self.estimator.push(item);
            true
        }
    }
    impl<const P: usize> Estimator<f64> for SimpleHLL<P> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.count() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(&other.estimator);
            copy.estimator.count() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<H: HasherType, const P: usize, const B: usize> Estimator<f64>
    for CloudFlareHLL<P, B, H> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.estimate() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(&other.estimator);
            copy.estimator.estimate() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            true
        }
    }
    impl<S: Clone + hypertwobits::h2b::Sketch + Send + Sync> Estimator<f64>
    for HyperTwoBits<S> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.count() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(other.estimator.clone());
            copy.estimator.count() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<S: Clone + hypertwobits::h3b::Sketch + Send + Sync> Estimator<f64>
    for HyperThreeBits<S> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.count() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(other.estimator.clone());
            copy.estimator.count() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<P: Precision> Estimator<f64> for SourMash<P> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.cardinality() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            self.estimator.union(&other.estimator) as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<P: Precision> Estimator<f64> for RustHLL<P> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.len() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(&other.estimator);
            copy.estimator.len() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<P: Precision> Estimator<f64> for TabacHLLPlusPlus<P> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.clone().count() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(&other.estimator).unwrap();
            copy.estimator.count() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<P: Precision> Estimator<f64> for TabacHLL<P> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.clone().count() as f64
        }
        fn estimate_union_cardinality(&self, _other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.merge(&self.estimator).unwrap();
            copy.estimator.count() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    impl<P: Precision> Estimator<f64> for SAHLL<P> {
        fn estimate_cardinality(&self) -> f64 {
            self.estimator.len() as f64
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            let mut copy = self.clone();
            copy.estimator.union(&other.estimator);
            copy.estimator.len() as f64
        }
        fn is_union_estimate_non_deterministic(&self, _other: &Self) -> bool {
            false
        }
    }
    /// A variant on the MemSize trait where the size of the
    /// enum is skipped from the calculation. It always follow
    /// the references.
    pub trait TransparentMemSize {
        /// Returns the size of the object in bytes.
        fn transparent_mem_size(&self) -> usize;
    }
    /// Enumerations will all HyperTwo variants we
    /// take into consideration for the benchmarks.
    pub enum HyperTwoVariants {
        H2BM64(HyperTwoBits<M64H2B>),
        H2BM128(HyperTwoBits<M128H2B>),
        H2BM256(HyperTwoBits<M256H2B>),
        H2BM512(HyperTwoBits<M512H2B>),
        H2BM1024(HyperTwoBits<M1024H2B>),
        H2BM2048(HyperTwoBits<M2048H2B>),
        H2BM4096(HyperTwoBits<M4096H2B>),
        H3BM64(HyperThreeBits<M64H3B>),
        H3BM128(HyperThreeBits<M128H3B>),
        H3BM256(HyperThreeBits<M256H3B>),
        H3BM512(HyperThreeBits<M512H3B>),
        H3BM1024(HyperThreeBits<M1024H3B>),
        H3BM2048(HyperThreeBits<M2048H3B>),
        H3BM4096(HyperThreeBits<M4096H3B>),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for HyperTwoVariants {
        #[inline]
        fn clone(&self) -> HyperTwoVariants {
            match self {
                HyperTwoVariants::H2BM64(__self_0) => {
                    HyperTwoVariants::H2BM64(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H2BM128(__self_0) => {
                    HyperTwoVariants::H2BM128(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H2BM256(__self_0) => {
                    HyperTwoVariants::H2BM256(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H2BM512(__self_0) => {
                    HyperTwoVariants::H2BM512(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H2BM1024(__self_0) => {
                    HyperTwoVariants::H2BM1024(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H2BM2048(__self_0) => {
                    HyperTwoVariants::H2BM2048(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H2BM4096(__self_0) => {
                    HyperTwoVariants::H2BM4096(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM64(__self_0) => {
                    HyperTwoVariants::H3BM64(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM128(__self_0) => {
                    HyperTwoVariants::H3BM128(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM256(__self_0) => {
                    HyperTwoVariants::H3BM256(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM512(__self_0) => {
                    HyperTwoVariants::H3BM512(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM1024(__self_0) => {
                    HyperTwoVariants::H3BM1024(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM2048(__self_0) => {
                    HyperTwoVariants::H3BM2048(::core::clone::Clone::clone(__self_0))
                }
                HyperTwoVariants::H3BM4096(__self_0) => {
                    HyperTwoVariants::H3BM4096(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    impl Named for HyperTwoVariants {
        fn name(&self) -> String {
            match self {
                HyperTwoVariants::H2BM64(inner) => inner.name(),
                HyperTwoVariants::H2BM128(inner) => inner.name(),
                HyperTwoVariants::H2BM256(inner) => inner.name(),
                HyperTwoVariants::H2BM512(inner) => inner.name(),
                HyperTwoVariants::H2BM1024(inner) => inner.name(),
                HyperTwoVariants::H2BM2048(inner) => inner.name(),
                HyperTwoVariants::H2BM4096(inner) => inner.name(),
                HyperTwoVariants::H3BM64(inner) => inner.name(),
                HyperTwoVariants::H3BM128(inner) => inner.name(),
                HyperTwoVariants::H3BM256(inner) => inner.name(),
                HyperTwoVariants::H3BM512(inner) => inner.name(),
                HyperTwoVariants::H3BM1024(inner) => inner.name(),
                HyperTwoVariants::H3BM2048(inner) => inner.name(),
                HyperTwoVariants::H3BM4096(inner) => inner.name(),
            }
        }
    }
    impl ExtendableApproximatedSet<u64> for HyperTwoVariants {
        fn insert(&mut self, element: &u64) -> bool {
            match self {
                HyperTwoVariants::H2BM64(inner) => inner.insert(element),
                HyperTwoVariants::H2BM128(inner) => inner.insert(element),
                HyperTwoVariants::H2BM256(inner) => inner.insert(element),
                HyperTwoVariants::H2BM512(inner) => inner.insert(element),
                HyperTwoVariants::H2BM1024(inner) => inner.insert(element),
                HyperTwoVariants::H2BM2048(inner) => inner.insert(element),
                HyperTwoVariants::H2BM4096(inner) => inner.insert(element),
                HyperTwoVariants::H3BM64(inner) => inner.insert(element),
                HyperTwoVariants::H3BM128(inner) => inner.insert(element),
                HyperTwoVariants::H3BM256(inner) => inner.insert(element),
                HyperTwoVariants::H3BM512(inner) => inner.insert(element),
                HyperTwoVariants::H3BM1024(inner) => inner.insert(element),
                HyperTwoVariants::H3BM2048(inner) => inner.insert(element),
                HyperTwoVariants::H3BM4096(inner) => inner.insert(element),
            }
        }
    }
    impl Estimator<f64> for HyperTwoVariants {
        fn estimate_cardinality(&self) -> f64 {
            match self {
                HyperTwoVariants::H2BM64(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H2BM128(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H2BM256(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H2BM512(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H2BM1024(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H2BM2048(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H2BM4096(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM64(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM128(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM256(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM512(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM1024(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM2048(inner) => inner.estimate_cardinality(),
                HyperTwoVariants::H3BM4096(inner) => inner.estimate_cardinality(),
            }
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            match (self, other) {
                (HyperTwoVariants::H2BM64(inner), HyperTwoVariants::H2BM64(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HyperTwoVariants::H2BM128(inner), HyperTwoVariants::H2BM128(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HyperTwoVariants::H2BM256(inner), HyperTwoVariants::H2BM256(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HyperTwoVariants::H2BM512(inner), HyperTwoVariants::H2BM512(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (
                    HyperTwoVariants::H2BM1024(inner),
                    HyperTwoVariants::H2BM1024(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HyperTwoVariants::H2BM2048(inner),
                    HyperTwoVariants::H2BM2048(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HyperTwoVariants::H2BM4096(inner),
                    HyperTwoVariants::H2BM4096(other),
                ) => inner.estimate_union_cardinality(other),
                (HyperTwoVariants::H3BM64(inner), HyperTwoVariants::H3BM64(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HyperTwoVariants::H3BM128(inner), HyperTwoVariants::H3BM128(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HyperTwoVariants::H3BM256(inner), HyperTwoVariants::H3BM256(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HyperTwoVariants::H3BM512(inner), HyperTwoVariants::H3BM512(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (
                    HyperTwoVariants::H3BM1024(inner),
                    HyperTwoVariants::H3BM1024(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HyperTwoVariants::H3BM2048(inner),
                    HyperTwoVariants::H3BM2048(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HyperTwoVariants::H3BM4096(inner),
                    HyperTwoVariants::H3BM4096(other),
                ) => inner.estimate_union_cardinality(other),
                _ => {
                    ::core::panicking::panic_fmt(
                        format_args!("Union cardinality not defined for these variants."),
                    );
                }
            }
        }
        fn is_union_estimate_non_deterministic(&self, other: &Self) -> bool {
            match (self, other) {
                (HyperTwoVariants::H2BM64(inner), HyperTwoVariants::H2BM64(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HyperTwoVariants::H2BM128(inner), HyperTwoVariants::H2BM128(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HyperTwoVariants::H2BM256(inner), HyperTwoVariants::H2BM256(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HyperTwoVariants::H2BM512(inner), HyperTwoVariants::H2BM512(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (
                    HyperTwoVariants::H2BM1024(inner),
                    HyperTwoVariants::H2BM1024(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HyperTwoVariants::H2BM2048(inner),
                    HyperTwoVariants::H2BM2048(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HyperTwoVariants::H2BM4096(inner),
                    HyperTwoVariants::H2BM4096(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (HyperTwoVariants::H3BM64(inner), HyperTwoVariants::H3BM64(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HyperTwoVariants::H3BM128(inner), HyperTwoVariants::H3BM128(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HyperTwoVariants::H3BM256(inner), HyperTwoVariants::H3BM256(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HyperTwoVariants::H3BM512(inner), HyperTwoVariants::H3BM512(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (
                    HyperTwoVariants::H3BM1024(inner),
                    HyperTwoVariants::H3BM1024(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HyperTwoVariants::H3BM2048(inner),
                    HyperTwoVariants::H3BM2048(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HyperTwoVariants::H3BM4096(inner),
                    HyperTwoVariants::H3BM4096(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                _ => {
                    ::core::panicking::panic_fmt(
                        format_args!("Union cardinality not defined for these variants."),
                    );
                }
            }
        }
    }
    impl TransparentMemSize for HyperTwoVariants {
        fn transparent_mem_size(&self) -> usize {
            match self {
                HyperTwoVariants::H2BM64(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H2BM128(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H2BM256(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H2BM512(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H2BM1024(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H2BM2048(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H2BM4096(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM64(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM128(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM256(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM512(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM1024(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM2048(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HyperTwoVariants::H3BM4096(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
            }
        }
    }
    /// Enumerations will all HyperLogLog variants we
    /// take into consideration for the benchmarks.
    pub enum HLLVariants<const EXPONENT: usize, P: Precision>
    where
        P: ArrayRegisters + PackedArrayRegisters + Named + MemSize + MemDbg,
    {
        TabacHyperLogLogPlus(TabacHLLPlusPlus<P>),
        TabacHyperLogLogPF(TabacHLL<P>),
        SAHyperLogLog(SAHLL<P>),
        RustHyperLogLog(RustHLL<P>),
        CE4(CloudFlareHLL<EXPONENT, 4, wyhash::WyHash>),
        CE5(CloudFlareHLL<EXPONENT, 5, wyhash::WyHash>),
        CE6(CloudFlareHLL<EXPONENT, 6, wyhash::WyHash>),
        SimpleHLL(SimpleHLL<EXPONENT>),
        PP4ArrayXxhasher(
            PlusPlus<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        PP4ArrayWyHash(
            PlusPlus<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::ArrayRegister,
                wyhash::WyHash,
            >,
        ),
        PP5ArrayXxhasher(
            PlusPlus<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        PP5ArrayWyHash(
            PlusPlus<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::ArrayRegister,
                wyhash::WyHash,
            >,
        ),
        PP6ArrayXxhasher(
            PlusPlus<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        PP6ArrayWyHash(
            PlusPlus<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::ArrayRegister,
                wyhash::WyHash,
            >,
        ),
        PP4PackedArrayXxhasher(
            PlusPlus<
                P,
                Bits4,
                <P as PackedArrayRegister<Bits4>>::PackedArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        PP4PackedArrayWyHash(
            PlusPlus<
                P,
                Bits4,
                <P as PackedArrayRegister<Bits4>>::PackedArrayRegister,
                wyhash::WyHash,
            >,
        ),
        PP5PackedArrayXxhasher(
            PlusPlus<
                P,
                Bits5,
                <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        PP5PackedArrayWyHash(
            PlusPlus<
                P,
                Bits5,
                <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                wyhash::WyHash,
            >,
        ),
        PP6PackedArrayXxhasher(
            PlusPlus<
                P,
                Bits6,
                <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        PP6PackedArrayWyHash(
            PlusPlus<
                P,
                Bits6,
                <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                wyhash::WyHash,
            >,
        ),
        LLB4ArrayXxhasher(
            LogLogBeta<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        LLB4ArrayWyHash(
            LogLogBeta<
                P,
                Bits4,
                <P as ArrayRegister<Bits4>>::ArrayRegister,
                wyhash::WyHash,
            >,
        ),
        LLB5ArrayXxhasher(
            LogLogBeta<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        LLB5ArrayWyHash(
            LogLogBeta<
                P,
                Bits5,
                <P as ArrayRegister<Bits5>>::ArrayRegister,
                wyhash::WyHash,
            >,
        ),
        LLB6ArrayXxhasher(
            LogLogBeta<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::ArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        LLB6ArrayWyHash(
            LogLogBeta<
                P,
                Bits6,
                <P as ArrayRegister<Bits6>>::ArrayRegister,
                wyhash::WyHash,
            >,
        ),
        LLB4PackedArrayWyHash(
            LogLogBeta<
                P,
                Bits4,
                <P as PackedArrayRegister<Bits4>>::PackedArrayRegister,
                wyhash::WyHash,
            >,
        ),
        LLB5PackedArrayXxhasher(
            LogLogBeta<
                P,
                Bits5,
                <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        LLB5PackedArrayWyHash(
            LogLogBeta<
                P,
                Bits5,
                <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                wyhash::WyHash,
            >,
        ),
        LLB6PackedArrayXxhasher(
            LogLogBeta<
                P,
                Bits6,
                <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                twox_hash::XxHash64,
            >,
        ),
        LLB6PackedArrayWyHash(
            LogLogBeta<
                P,
                Bits6,
                <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                wyhash::WyHash,
            >,
        ),
        MLEPP4WyHash(
            MLE<
                PlusPlus<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        MLEPP5WyHash(
            MLE<
                PlusPlus<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        MLEPP6WyHash(
            MLE<
                PlusPlus<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        MLEPP4Xxhasher(
            MLE<
                PlusPlus<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        MLEPP5Xxhasher(
            MLE<
                PlusPlus<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        MLEPP6Xxhasher(
            MLE<
                PlusPlus<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        MLELLB4WyHash(
            MLE<
                LogLogBeta<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        MLELLB5WyHash(
            MLE<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        MLELLB6WyHash(
            MLE<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        MLELLB4Xxhasher(
            MLE<
                LogLogBeta<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        MLELLB5Xxhasher(
            MLE<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        MLELLB6Xxhasher(
            MLE<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP4ArrayXxhasher(
            Hybrid<
                PlusPlus<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP4ArrayWyHash(
            Hybrid<
                PlusPlus<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridPP5ArrayXxhasher(
            Hybrid<
                PlusPlus<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP5ArrayWyHash(
            Hybrid<
                PlusPlus<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridPP6ArrayXxhasher(
            Hybrid<
                PlusPlus<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP6ArrayWyHash(
            Hybrid<
                PlusPlus<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridPP4PackedArrayXxhasher(
            Hybrid<
                PlusPlus<
                    P,
                    Bits4,
                    <P as PackedArrayRegister<Bits4>>::PackedArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP4PackedArrayWyHash(
            Hybrid<
                PlusPlus<
                    P,
                    Bits4,
                    <P as PackedArrayRegister<Bits4>>::PackedArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridPP5PackedArrayXxhasher(
            Hybrid<
                PlusPlus<
                    P,
                    Bits5,
                    <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP5PackedArrayWyHash(
            Hybrid<
                PlusPlus<
                    P,
                    Bits5,
                    <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridPP6PackedArrayXxhasher(
            Hybrid<
                PlusPlus<
                    P,
                    Bits6,
                    <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridPP6PackedArrayWyHash(
            Hybrid<
                PlusPlus<
                    P,
                    Bits6,
                    <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridLLB4ArrayXxhasher(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridLLB4ArrayWyHash(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits4,
                    <P as ArrayRegister<Bits4>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridLLB5ArrayXxhasher(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridLLB5ArrayWyHash(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as ArrayRegister<Bits5>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridLLB6ArrayXxhasher(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridLLB6ArrayWyHash(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as ArrayRegister<Bits6>>::ArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridLLB4PackedArrayWyHash(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits4,
                    <P as PackedArrayRegister<Bits4>>::PackedArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridLLB5PackedArrayXxhasher(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridLLB5PackedArrayWyHash(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits5,
                    <P as PackedArrayRegister<Bits5>>::PackedArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridLLB6PackedArrayXxhasher(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                    twox_hash::XxHash64,
                >,
            >,
        ),
        HybridLLB6PackedArrayWyHash(
            Hybrid<
                LogLogBeta<
                    P,
                    Bits6,
                    <P as PackedArrayRegister<Bits6>>::PackedArrayRegister,
                    wyhash::WyHash,
                >,
            >,
        ),
        HybridMLEPP4WyHash(
            Hybrid<
                MLE<
                    PlusPlus<
                        P,
                        Bits4,
                        <P as ArrayRegister<Bits4>>::ArrayRegister,
                        wyhash::WyHash,
                    >,
                >,
            >,
        ),
        HybridMLEPP5WyHash(
            Hybrid<
                MLE<
                    PlusPlus<
                        P,
                        Bits5,
                        <P as ArrayRegister<Bits5>>::ArrayRegister,
                        wyhash::WyHash,
                    >,
                >,
            >,
        ),
        HybridMLEPP6WyHash(
            Hybrid<
                MLE<
                    PlusPlus<
                        P,
                        Bits6,
                        <P as ArrayRegister<Bits6>>::ArrayRegister,
                        wyhash::WyHash,
                    >,
                >,
            >,
        ),
        HybridMLEPP4Xxhasher(
            Hybrid<
                MLE<
                    PlusPlus<
                        P,
                        Bits4,
                        <P as ArrayRegister<Bits4>>::ArrayRegister,
                        twox_hash::XxHash64,
                    >,
                >,
            >,
        ),
        HybridMLEPP5Xxhasher(
            Hybrid<
                MLE<
                    PlusPlus<
                        P,
                        Bits5,
                        <P as ArrayRegister<Bits5>>::ArrayRegister,
                        twox_hash::XxHash64,
                    >,
                >,
            >,
        ),
        HybridMLEPP6Xxhasher(
            Hybrid<
                MLE<
                    PlusPlus<
                        P,
                        Bits6,
                        <P as ArrayRegister<Bits6>>::ArrayRegister,
                        twox_hash::XxHash64,
                    >,
                >,
            >,
        ),
        HybridMLELLB4WyHash(
            Hybrid<
                MLE<
                    LogLogBeta<
                        P,
                        Bits4,
                        <P as ArrayRegister<Bits4>>::ArrayRegister,
                        wyhash::WyHash,
                    >,
                >,
            >,
        ),
        HybridMLELLB5WyHash(
            Hybrid<
                MLE<
                    LogLogBeta<
                        P,
                        Bits5,
                        <P as ArrayRegister<Bits5>>::ArrayRegister,
                        wyhash::WyHash,
                    >,
                >,
            >,
        ),
        HybridMLELLB6WyHash(
            Hybrid<
                MLE<
                    LogLogBeta<
                        P,
                        Bits6,
                        <P as ArrayRegister<Bits6>>::ArrayRegister,
                        wyhash::WyHash,
                    >,
                >,
            >,
        ),
        HybridMLELLB4Xxhasher(
            Hybrid<
                MLE<
                    LogLogBeta<
                        P,
                        Bits4,
                        <P as ArrayRegister<Bits4>>::ArrayRegister,
                        twox_hash::XxHash64,
                    >,
                >,
            >,
        ),
        HybridMLELLB5Xxhasher(
            Hybrid<
                MLE<
                    LogLogBeta<
                        P,
                        Bits5,
                        <P as ArrayRegister<Bits5>>::ArrayRegister,
                        twox_hash::XxHash64,
                    >,
                >,
            >,
        ),
        HybridMLELLB6Xxhasher(
            Hybrid<
                MLE<
                    LogLogBeta<
                        P,
                        Bits6,
                        <P as ArrayRegister<Bits6>>::ArrayRegister,
                        twox_hash::XxHash64,
                    >,
                >,
            >,
        ),
    }
    #[automatically_derived]
    impl<const EXPONENT: usize, P: ::core::clone::Clone + Precision> ::core::clone::Clone
    for HLLVariants<EXPONENT, P>
    where
        P: ArrayRegisters + PackedArrayRegisters + Named + MemSize + MemDbg,
    {
        #[inline]
        fn clone(&self) -> HLLVariants<EXPONENT, P> {
            match self {
                HLLVariants::TabacHyperLogLogPlus(__self_0) => {
                    HLLVariants::TabacHyperLogLogPlus(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::TabacHyperLogLogPF(__self_0) => {
                    HLLVariants::TabacHyperLogLogPF(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::SAHyperLogLog(__self_0) => {
                    HLLVariants::SAHyperLogLog(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::RustHyperLogLog(__self_0) => {
                    HLLVariants::RustHyperLogLog(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::CE4(__self_0) => {
                    HLLVariants::CE4(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::CE5(__self_0) => {
                    HLLVariants::CE5(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::CE6(__self_0) => {
                    HLLVariants::CE6(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::SimpleHLL(__self_0) => {
                    HLLVariants::SimpleHLL(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP4ArrayXxhasher(__self_0) => {
                    HLLVariants::PP4ArrayXxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP4ArrayWyHash(__self_0) => {
                    HLLVariants::PP4ArrayWyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP5ArrayXxhasher(__self_0) => {
                    HLLVariants::PP5ArrayXxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP5ArrayWyHash(__self_0) => {
                    HLLVariants::PP5ArrayWyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP6ArrayXxhasher(__self_0) => {
                    HLLVariants::PP6ArrayXxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP6ArrayWyHash(__self_0) => {
                    HLLVariants::PP6ArrayWyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::PP4PackedArrayXxhasher(__self_0) => {
                    HLLVariants::PP4PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::PP4PackedArrayWyHash(__self_0) => {
                    HLLVariants::PP4PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::PP5PackedArrayXxhasher(__self_0) => {
                    HLLVariants::PP5PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::PP5PackedArrayWyHash(__self_0) => {
                    HLLVariants::PP5PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::PP6PackedArrayXxhasher(__self_0) => {
                    HLLVariants::PP6PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::PP6PackedArrayWyHash(__self_0) => {
                    HLLVariants::PP6PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::LLB4ArrayXxhasher(__self_0) => {
                    HLLVariants::LLB4ArrayXxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::LLB4ArrayWyHash(__self_0) => {
                    HLLVariants::LLB4ArrayWyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::LLB5ArrayXxhasher(__self_0) => {
                    HLLVariants::LLB5ArrayXxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::LLB5ArrayWyHash(__self_0) => {
                    HLLVariants::LLB5ArrayWyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::LLB6ArrayXxhasher(__self_0) => {
                    HLLVariants::LLB6ArrayXxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::LLB6ArrayWyHash(__self_0) => {
                    HLLVariants::LLB6ArrayWyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::LLB4PackedArrayWyHash(__self_0) => {
                    HLLVariants::LLB4PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::LLB5PackedArrayXxhasher(__self_0) => {
                    HLLVariants::LLB5PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::LLB5PackedArrayWyHash(__self_0) => {
                    HLLVariants::LLB5PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::LLB6PackedArrayXxhasher(__self_0) => {
                    HLLVariants::LLB6PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::LLB6PackedArrayWyHash(__self_0) => {
                    HLLVariants::LLB6PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::MLEPP4WyHash(__self_0) => {
                    HLLVariants::MLEPP4WyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLEPP5WyHash(__self_0) => {
                    HLLVariants::MLEPP5WyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLEPP6WyHash(__self_0) => {
                    HLLVariants::MLEPP6WyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLEPP4Xxhasher(__self_0) => {
                    HLLVariants::MLEPP4Xxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLEPP5Xxhasher(__self_0) => {
                    HLLVariants::MLEPP5Xxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLEPP6Xxhasher(__self_0) => {
                    HLLVariants::MLEPP6Xxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLELLB4WyHash(__self_0) => {
                    HLLVariants::MLELLB4WyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLELLB5WyHash(__self_0) => {
                    HLLVariants::MLELLB5WyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLELLB6WyHash(__self_0) => {
                    HLLVariants::MLELLB6WyHash(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLELLB4Xxhasher(__self_0) => {
                    HLLVariants::MLELLB4Xxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLELLB5Xxhasher(__self_0) => {
                    HLLVariants::MLELLB5Xxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::MLELLB6Xxhasher(__self_0) => {
                    HLLVariants::MLELLB6Xxhasher(::core::clone::Clone::clone(__self_0))
                }
                HLLVariants::HybridPP4ArrayXxhasher(__self_0) => {
                    HLLVariants::HybridPP4ArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP4ArrayWyHash(__self_0) => {
                    HLLVariants::HybridPP4ArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP5ArrayXxhasher(__self_0) => {
                    HLLVariants::HybridPP5ArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP5ArrayWyHash(__self_0) => {
                    HLLVariants::HybridPP5ArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP6ArrayXxhasher(__self_0) => {
                    HLLVariants::HybridPP6ArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP6ArrayWyHash(__self_0) => {
                    HLLVariants::HybridPP6ArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP4PackedArrayXxhasher(__self_0) => {
                    HLLVariants::HybridPP4PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP4PackedArrayWyHash(__self_0) => {
                    HLLVariants::HybridPP4PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP5PackedArrayXxhasher(__self_0) => {
                    HLLVariants::HybridPP5PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP5PackedArrayWyHash(__self_0) => {
                    HLLVariants::HybridPP5PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP6PackedArrayXxhasher(__self_0) => {
                    HLLVariants::HybridPP6PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridPP6PackedArrayWyHash(__self_0) => {
                    HLLVariants::HybridPP6PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB4ArrayXxhasher(__self_0) => {
                    HLLVariants::HybridLLB4ArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB4ArrayWyHash(__self_0) => {
                    HLLVariants::HybridLLB4ArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB5ArrayXxhasher(__self_0) => {
                    HLLVariants::HybridLLB5ArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB5ArrayWyHash(__self_0) => {
                    HLLVariants::HybridLLB5ArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB6ArrayXxhasher(__self_0) => {
                    HLLVariants::HybridLLB6ArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB6ArrayWyHash(__self_0) => {
                    HLLVariants::HybridLLB6ArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB4PackedArrayWyHash(__self_0) => {
                    HLLVariants::HybridLLB4PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB5PackedArrayXxhasher(__self_0) => {
                    HLLVariants::HybridLLB5PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB5PackedArrayWyHash(__self_0) => {
                    HLLVariants::HybridLLB5PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB6PackedArrayXxhasher(__self_0) => {
                    HLLVariants::HybridLLB6PackedArrayXxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridLLB6PackedArrayWyHash(__self_0) => {
                    HLLVariants::HybridLLB6PackedArrayWyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLEPP4WyHash(__self_0) => {
                    HLLVariants::HybridMLEPP4WyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLEPP5WyHash(__self_0) => {
                    HLLVariants::HybridMLEPP5WyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLEPP6WyHash(__self_0) => {
                    HLLVariants::HybridMLEPP6WyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLEPP4Xxhasher(__self_0) => {
                    HLLVariants::HybridMLEPP4Xxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLEPP5Xxhasher(__self_0) => {
                    HLLVariants::HybridMLEPP5Xxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLEPP6Xxhasher(__self_0) => {
                    HLLVariants::HybridMLEPP6Xxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLELLB4WyHash(__self_0) => {
                    HLLVariants::HybridMLELLB4WyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLELLB5WyHash(__self_0) => {
                    HLLVariants::HybridMLELLB5WyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLELLB6WyHash(__self_0) => {
                    HLLVariants::HybridMLELLB6WyHash(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLELLB4Xxhasher(__self_0) => {
                    HLLVariants::HybridMLELLB4Xxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLELLB5Xxhasher(__self_0) => {
                    HLLVariants::HybridMLELLB5Xxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
                HLLVariants::HybridMLELLB6Xxhasher(__self_0) => {
                    HLLVariants::HybridMLELLB6Xxhasher(
                        ::core::clone::Clone::clone(__self_0),
                    )
                }
            }
        }
    }
    impl<const EXPONENT: usize, P: Precision> Named for HLLVariants<EXPONENT, P>
    where
        P: ArrayRegisters + PackedArrayRegisters + Named + MemSize + MemDbg,
    {
        fn name(&self) -> String {
            match self {
                HLLVariants::TabacHyperLogLogPlus(inner) => inner.name(),
                HLLVariants::TabacHyperLogLogPF(inner) => inner.name(),
                HLLVariants::SAHyperLogLog(inner) => inner.name(),
                HLLVariants::RustHyperLogLog(inner) => inner.name(),
                HLLVariants::CE4(inner) => inner.name(),
                HLLVariants::CE5(inner) => inner.name(),
                HLLVariants::CE6(inner) => inner.name(),
                HLLVariants::SimpleHLL(inner) => inner.name(),
                HLLVariants::PP4ArrayXxhasher(inner) => inner.name(),
                HLLVariants::PP4ArrayWyHash(inner) => inner.name(),
                HLLVariants::PP5ArrayXxhasher(inner) => inner.name(),
                HLLVariants::PP5ArrayWyHash(inner) => inner.name(),
                HLLVariants::PP6ArrayXxhasher(inner) => inner.name(),
                HLLVariants::PP6ArrayWyHash(inner) => inner.name(),
                HLLVariants::PP4PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::PP4PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::PP5PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::PP5PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::PP6PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::PP6PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::LLB4ArrayXxhasher(inner) => inner.name(),
                HLLVariants::LLB4ArrayWyHash(inner) => inner.name(),
                HLLVariants::LLB5ArrayXxhasher(inner) => inner.name(),
                HLLVariants::LLB5ArrayWyHash(inner) => inner.name(),
                HLLVariants::LLB6ArrayXxhasher(inner) => inner.name(),
                HLLVariants::LLB6ArrayWyHash(inner) => inner.name(),
                HLLVariants::LLB4PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::LLB5PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::LLB5PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::LLB6PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::LLB6PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::MLEPP4WyHash(inner) => inner.name(),
                HLLVariants::MLEPP5WyHash(inner) => inner.name(),
                HLLVariants::MLEPP6WyHash(inner) => inner.name(),
                HLLVariants::MLEPP4Xxhasher(inner) => inner.name(),
                HLLVariants::MLEPP5Xxhasher(inner) => inner.name(),
                HLLVariants::MLEPP6Xxhasher(inner) => inner.name(),
                HLLVariants::MLELLB4WyHash(inner) => inner.name(),
                HLLVariants::MLELLB5WyHash(inner) => inner.name(),
                HLLVariants::MLELLB6WyHash(inner) => inner.name(),
                HLLVariants::MLELLB4Xxhasher(inner) => inner.name(),
                HLLVariants::MLELLB5Xxhasher(inner) => inner.name(),
                HLLVariants::MLELLB6Xxhasher(inner) => inner.name(),
                HLLVariants::HybridPP4ArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridPP4ArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridPP5ArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridPP5ArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridPP6ArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridPP6ArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridPP4PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridPP4PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridPP5PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridPP5PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridPP6PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridPP6PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridLLB4ArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridLLB4ArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridLLB5ArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridLLB5ArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridLLB6ArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridLLB6ArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridLLB4PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridLLB5PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridLLB5PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridLLB6PackedArrayXxhasher(inner) => inner.name(),
                HLLVariants::HybridLLB6PackedArrayWyHash(inner) => inner.name(),
                HLLVariants::HybridMLEPP4WyHash(inner) => inner.name(),
                HLLVariants::HybridMLEPP5WyHash(inner) => inner.name(),
                HLLVariants::HybridMLEPP6WyHash(inner) => inner.name(),
                HLLVariants::HybridMLEPP4Xxhasher(inner) => inner.name(),
                HLLVariants::HybridMLEPP5Xxhasher(inner) => inner.name(),
                HLLVariants::HybridMLEPP6Xxhasher(inner) => inner.name(),
                HLLVariants::HybridMLELLB4WyHash(inner) => inner.name(),
                HLLVariants::HybridMLELLB5WyHash(inner) => inner.name(),
                HLLVariants::HybridMLELLB6WyHash(inner) => inner.name(),
                HLLVariants::HybridMLELLB4Xxhasher(inner) => inner.name(),
                HLLVariants::HybridMLELLB5Xxhasher(inner) => inner.name(),
                HLLVariants::HybridMLELLB6Xxhasher(inner) => inner.name(),
            }
        }
    }
    impl<const EXPONENT: usize, P: Precision> ExtendableApproximatedSet<u64>
    for HLLVariants<EXPONENT, P>
    where
        P: ArrayRegisters + PackedArrayRegisters + Named + MemSize + MemDbg,
    {
        fn insert(&mut self, element: &u64) -> bool {
            match self {
                HLLVariants::TabacHyperLogLogPlus(inner) => inner.insert(element),
                HLLVariants::TabacHyperLogLogPF(inner) => inner.insert(element),
                HLLVariants::SAHyperLogLog(inner) => inner.insert(element),
                HLLVariants::RustHyperLogLog(inner) => inner.insert(element),
                HLLVariants::CE4(inner) => inner.insert(element),
                HLLVariants::CE5(inner) => inner.insert(element),
                HLLVariants::CE6(inner) => inner.insert(element),
                HLLVariants::SimpleHLL(inner) => inner.insert(element),
                HLLVariants::PP4ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::PP4ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::PP5ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::PP5ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::PP6ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::PP6ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::PP4PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::PP4PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::PP5PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::PP5PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::PP6PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::PP6PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::LLB4ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::LLB4ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::LLB5ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::LLB5ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::LLB6ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::LLB6ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::LLB4PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::LLB5PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::LLB5PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::LLB6PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::LLB6PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::MLEPP4WyHash(inner) => inner.insert(element),
                HLLVariants::MLEPP5WyHash(inner) => inner.insert(element),
                HLLVariants::MLEPP6WyHash(inner) => inner.insert(element),
                HLLVariants::MLEPP4Xxhasher(inner) => inner.insert(element),
                HLLVariants::MLEPP5Xxhasher(inner) => inner.insert(element),
                HLLVariants::MLEPP6Xxhasher(inner) => inner.insert(element),
                HLLVariants::MLELLB4WyHash(inner) => inner.insert(element),
                HLLVariants::MLELLB5WyHash(inner) => inner.insert(element),
                HLLVariants::MLELLB6WyHash(inner) => inner.insert(element),
                HLLVariants::MLELLB4Xxhasher(inner) => inner.insert(element),
                HLLVariants::MLELLB5Xxhasher(inner) => inner.insert(element),
                HLLVariants::MLELLB6Xxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP4ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP4ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridPP5ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP5ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridPP6ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP6ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridPP4PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP4PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridPP5PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP5PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridPP6PackedArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridPP6PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridLLB4ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridLLB4ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridLLB5ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridLLB5ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridLLB6ArrayXxhasher(inner) => inner.insert(element),
                HLLVariants::HybridLLB6ArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridLLB4PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridLLB5PackedArrayXxhasher(inner) => {
                    inner.insert(element)
                }
                HLLVariants::HybridLLB5PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridLLB6PackedArrayXxhasher(inner) => {
                    inner.insert(element)
                }
                HLLVariants::HybridLLB6PackedArrayWyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLEPP4WyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLEPP5WyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLEPP6WyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLEPP4Xxhasher(inner) => inner.insert(element),
                HLLVariants::HybridMLEPP5Xxhasher(inner) => inner.insert(element),
                HLLVariants::HybridMLEPP6Xxhasher(inner) => inner.insert(element),
                HLLVariants::HybridMLELLB4WyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLELLB5WyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLELLB6WyHash(inner) => inner.insert(element),
                HLLVariants::HybridMLELLB4Xxhasher(inner) => inner.insert(element),
                HLLVariants::HybridMLELLB5Xxhasher(inner) => inner.insert(element),
                HLLVariants::HybridMLELLB6Xxhasher(inner) => inner.insert(element),
            }
        }
    }
    impl<const EXPONENT: usize, P: Precision> Estimator<f64> for HLLVariants<EXPONENT, P>
    where
        P: ArrayRegisters + PackedArrayRegisters + Named + MemSize + MemDbg,
    {
        fn estimate_cardinality(&self) -> f64 {
            match self {
                HLLVariants::TabacHyperLogLogPlus(inner) => inner.estimate_cardinality(),
                HLLVariants::TabacHyperLogLogPF(inner) => inner.estimate_cardinality(),
                HLLVariants::SAHyperLogLog(inner) => inner.estimate_cardinality(),
                HLLVariants::RustHyperLogLog(inner) => inner.estimate_cardinality(),
                HLLVariants::CE4(inner) => inner.estimate_cardinality(),
                HLLVariants::CE5(inner) => inner.estimate_cardinality(),
                HLLVariants::CE6(inner) => inner.estimate_cardinality(),
                HLLVariants::SimpleHLL(inner) => inner.estimate_cardinality(),
                HLLVariants::PP4ArrayXxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::PP4ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::PP5ArrayXxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::PP5ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::PP6ArrayXxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::PP6ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::PP4PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::PP4PackedArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::PP5PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::PP5PackedArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::PP6PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::PP6PackedArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB4ArrayXxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB4ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB5ArrayXxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB5ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB6ArrayXxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB6ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB4PackedArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB5PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::LLB5PackedArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::LLB6PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::LLB6PackedArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLEPP4WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLEPP5WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLEPP6WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLEPP4Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::MLEPP5Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::MLEPP6Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::MLELLB4WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLELLB5WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLELLB6WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::MLELLB4Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::MLELLB5Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::MLELLB6Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridPP4ArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP4ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridPP5ArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP5ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridPP6ArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP6ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridPP4PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP4PackedArrayWyHash(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP5PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP5PackedArrayWyHash(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP6PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridPP6PackedArrayWyHash(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB4ArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB4ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridLLB5ArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB5ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridLLB6ArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB6ArrayWyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridLLB4PackedArrayWyHash(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB5PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB5PackedArrayWyHash(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB6PackedArrayXxhasher(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridLLB6PackedArrayWyHash(inner) => {
                    inner.estimate_cardinality()
                }
                HLLVariants::HybridMLEPP4WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLEPP5WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLEPP6WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLEPP4Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLEPP5Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLEPP6Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLELLB4WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLELLB5WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLELLB6WyHash(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLELLB4Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLELLB5Xxhasher(inner) => inner.estimate_cardinality(),
                HLLVariants::HybridMLELLB6Xxhasher(inner) => inner.estimate_cardinality(),
            }
        }
        fn estimate_union_cardinality(&self, other: &Self) -> f64 {
            match (self, other) {
                (
                    HLLVariants::TabacHyperLogLogPlus(inner),
                    HLLVariants::TabacHyperLogLogPlus(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::TabacHyperLogLogPF(inner),
                    HLLVariants::TabacHyperLogLogPF(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::SAHyperLogLog(inner),
                    HLLVariants::SAHyperLogLog(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::RustHyperLogLog(inner),
                    HLLVariants::RustHyperLogLog(other),
                ) => inner.estimate_union_cardinality(other),
                (HLLVariants::CE4(inner), HLLVariants::CE4(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HLLVariants::CE5(inner), HLLVariants::CE5(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HLLVariants::CE6(inner), HLLVariants::CE6(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HLLVariants::SimpleHLL(inner), HLLVariants::SimpleHLL(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (
                    HLLVariants::PP4ArrayXxhasher(inner),
                    HLLVariants::PP4ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP4ArrayWyHash(inner),
                    HLLVariants::PP4ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP5ArrayXxhasher(inner),
                    HLLVariants::PP5ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP5ArrayWyHash(inner),
                    HLLVariants::PP5ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP6ArrayXxhasher(inner),
                    HLLVariants::PP6ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP6ArrayWyHash(inner),
                    HLLVariants::PP6ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP4PackedArrayXxhasher(inner),
                    HLLVariants::PP4PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP4PackedArrayWyHash(inner),
                    HLLVariants::PP4PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP5PackedArrayXxhasher(inner),
                    HLLVariants::PP5PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP5PackedArrayWyHash(inner),
                    HLLVariants::PP5PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP6PackedArrayXxhasher(inner),
                    HLLVariants::PP6PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::PP6PackedArrayWyHash(inner),
                    HLLVariants::PP6PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB4ArrayXxhasher(inner),
                    HLLVariants::LLB4ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB4ArrayWyHash(inner),
                    HLLVariants::LLB4ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB5ArrayXxhasher(inner),
                    HLLVariants::LLB5ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB5ArrayWyHash(inner),
                    HLLVariants::LLB5ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB6ArrayXxhasher(inner),
                    HLLVariants::LLB6ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB6ArrayWyHash(inner),
                    HLLVariants::LLB6ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB4PackedArrayWyHash(inner),
                    HLLVariants::LLB4PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB5PackedArrayXxhasher(inner),
                    HLLVariants::LLB5PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB5PackedArrayWyHash(inner),
                    HLLVariants::LLB5PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB6PackedArrayXxhasher(inner),
                    HLLVariants::LLB6PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::LLB6PackedArrayWyHash(inner),
                    HLLVariants::LLB6PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (HLLVariants::MLEPP4WyHash(inner), HLLVariants::MLEPP4WyHash(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HLLVariants::MLEPP5WyHash(inner), HLLVariants::MLEPP5WyHash(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (HLLVariants::MLEPP6WyHash(inner), HLLVariants::MLEPP6WyHash(other)) => {
                    inner.estimate_union_cardinality(other)
                }
                (
                    HLLVariants::MLEPP4Xxhasher(inner),
                    HLLVariants::MLEPP4Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLEPP5Xxhasher(inner),
                    HLLVariants::MLEPP5Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLEPP6Xxhasher(inner),
                    HLLVariants::MLEPP6Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLELLB4WyHash(inner),
                    HLLVariants::MLELLB4WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLELLB5WyHash(inner),
                    HLLVariants::MLELLB5WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLELLB6WyHash(inner),
                    HLLVariants::MLELLB6WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLELLB4Xxhasher(inner),
                    HLLVariants::MLELLB4Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLELLB5Xxhasher(inner),
                    HLLVariants::MLELLB5Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::MLELLB6Xxhasher(inner),
                    HLLVariants::MLELLB6Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP4ArrayXxhasher(inner),
                    HLLVariants::HybridPP4ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP4ArrayWyHash(inner),
                    HLLVariants::HybridPP4ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP5ArrayXxhasher(inner),
                    HLLVariants::HybridPP5ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP5ArrayWyHash(inner),
                    HLLVariants::HybridPP5ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP6ArrayXxhasher(inner),
                    HLLVariants::HybridPP6ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP6ArrayWyHash(inner),
                    HLLVariants::HybridPP6ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP4PackedArrayXxhasher(inner),
                    HLLVariants::HybridPP4PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP4PackedArrayWyHash(inner),
                    HLLVariants::HybridPP4PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP5PackedArrayXxhasher(inner),
                    HLLVariants::HybridPP5PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP5PackedArrayWyHash(inner),
                    HLLVariants::HybridPP5PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP6PackedArrayXxhasher(inner),
                    HLLVariants::HybridPP6PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridPP6PackedArrayWyHash(inner),
                    HLLVariants::HybridPP6PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB4ArrayXxhasher(inner),
                    HLLVariants::HybridLLB4ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB4ArrayWyHash(inner),
                    HLLVariants::HybridLLB4ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB5ArrayXxhasher(inner),
                    HLLVariants::HybridLLB5ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB5ArrayWyHash(inner),
                    HLLVariants::HybridLLB5ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB6ArrayXxhasher(inner),
                    HLLVariants::HybridLLB6ArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB6ArrayWyHash(inner),
                    HLLVariants::HybridLLB6ArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB4PackedArrayWyHash(inner),
                    HLLVariants::HybridLLB4PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB5PackedArrayXxhasher(inner),
                    HLLVariants::HybridLLB5PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB5PackedArrayWyHash(inner),
                    HLLVariants::HybridLLB5PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB6PackedArrayXxhasher(inner),
                    HLLVariants::HybridLLB6PackedArrayXxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridLLB6PackedArrayWyHash(inner),
                    HLLVariants::HybridLLB6PackedArrayWyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLEPP4WyHash(inner),
                    HLLVariants::HybridMLEPP4WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLEPP5WyHash(inner),
                    HLLVariants::HybridMLEPP5WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLEPP6WyHash(inner),
                    HLLVariants::HybridMLEPP6WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLEPP4Xxhasher(inner),
                    HLLVariants::HybridMLEPP4Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLEPP5Xxhasher(inner),
                    HLLVariants::HybridMLEPP5Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLEPP6Xxhasher(inner),
                    HLLVariants::HybridMLEPP6Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLELLB4WyHash(inner),
                    HLLVariants::HybridMLELLB4WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLELLB5WyHash(inner),
                    HLLVariants::HybridMLELLB5WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLELLB6WyHash(inner),
                    HLLVariants::HybridMLELLB6WyHash(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLELLB4Xxhasher(inner),
                    HLLVariants::HybridMLELLB4Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLELLB5Xxhasher(inner),
                    HLLVariants::HybridMLELLB5Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                (
                    HLLVariants::HybridMLELLB6Xxhasher(inner),
                    HLLVariants::HybridMLELLB6Xxhasher(other),
                ) => inner.estimate_union_cardinality(other),
                _ => {
                    ::core::panicking::panic_fmt(
                        format_args!("Union cardinality not defined for these variants."),
                    );
                }
            }
        }
        fn is_union_estimate_non_deterministic(&self, other: &Self) -> bool {
            match (self, other) {
                (
                    HLLVariants::TabacHyperLogLogPlus(inner),
                    HLLVariants::TabacHyperLogLogPlus(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::TabacHyperLogLogPF(inner),
                    HLLVariants::TabacHyperLogLogPF(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::SAHyperLogLog(inner),
                    HLLVariants::SAHyperLogLog(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::RustHyperLogLog(inner),
                    HLLVariants::RustHyperLogLog(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (HLLVariants::CE4(inner), HLLVariants::CE4(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HLLVariants::CE5(inner), HLLVariants::CE5(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HLLVariants::CE6(inner), HLLVariants::CE6(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HLLVariants::SimpleHLL(inner), HLLVariants::SimpleHLL(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (
                    HLLVariants::PP4ArrayXxhasher(inner),
                    HLLVariants::PP4ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP4ArrayWyHash(inner),
                    HLLVariants::PP4ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP5ArrayXxhasher(inner),
                    HLLVariants::PP5ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP5ArrayWyHash(inner),
                    HLLVariants::PP5ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP6ArrayXxhasher(inner),
                    HLLVariants::PP6ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP6ArrayWyHash(inner),
                    HLLVariants::PP6ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP4PackedArrayXxhasher(inner),
                    HLLVariants::PP4PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP4PackedArrayWyHash(inner),
                    HLLVariants::PP4PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP5PackedArrayXxhasher(inner),
                    HLLVariants::PP5PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP5PackedArrayWyHash(inner),
                    HLLVariants::PP5PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP6PackedArrayXxhasher(inner),
                    HLLVariants::PP6PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::PP6PackedArrayWyHash(inner),
                    HLLVariants::PP6PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB4ArrayXxhasher(inner),
                    HLLVariants::LLB4ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB4ArrayWyHash(inner),
                    HLLVariants::LLB4ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB5ArrayXxhasher(inner),
                    HLLVariants::LLB5ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB5ArrayWyHash(inner),
                    HLLVariants::LLB5ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB6ArrayXxhasher(inner),
                    HLLVariants::LLB6ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB6ArrayWyHash(inner),
                    HLLVariants::LLB6ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB4PackedArrayWyHash(inner),
                    HLLVariants::LLB4PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB5PackedArrayXxhasher(inner),
                    HLLVariants::LLB5PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB5PackedArrayWyHash(inner),
                    HLLVariants::LLB5PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB6PackedArrayXxhasher(inner),
                    HLLVariants::LLB6PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::LLB6PackedArrayWyHash(inner),
                    HLLVariants::LLB6PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (HLLVariants::MLEPP4WyHash(inner), HLLVariants::MLEPP4WyHash(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HLLVariants::MLEPP5WyHash(inner), HLLVariants::MLEPP5WyHash(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (HLLVariants::MLEPP6WyHash(inner), HLLVariants::MLEPP6WyHash(other)) => {
                    inner.is_union_estimate_non_deterministic(other)
                }
                (
                    HLLVariants::MLEPP4Xxhasher(inner),
                    HLLVariants::MLEPP4Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLEPP5Xxhasher(inner),
                    HLLVariants::MLEPP5Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLEPP6Xxhasher(inner),
                    HLLVariants::MLEPP6Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLELLB4WyHash(inner),
                    HLLVariants::MLELLB4WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLELLB5WyHash(inner),
                    HLLVariants::MLELLB5WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLELLB6WyHash(inner),
                    HLLVariants::MLELLB6WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLELLB4Xxhasher(inner),
                    HLLVariants::MLELLB4Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLELLB5Xxhasher(inner),
                    HLLVariants::MLELLB5Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::MLELLB6Xxhasher(inner),
                    HLLVariants::MLELLB6Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP4ArrayXxhasher(inner),
                    HLLVariants::HybridPP4ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP4ArrayWyHash(inner),
                    HLLVariants::HybridPP4ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP5ArrayXxhasher(inner),
                    HLLVariants::HybridPP5ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP5ArrayWyHash(inner),
                    HLLVariants::HybridPP5ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP6ArrayXxhasher(inner),
                    HLLVariants::HybridPP6ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP6ArrayWyHash(inner),
                    HLLVariants::HybridPP6ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP4PackedArrayXxhasher(inner),
                    HLLVariants::HybridPP4PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP4PackedArrayWyHash(inner),
                    HLLVariants::HybridPP4PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP5PackedArrayXxhasher(inner),
                    HLLVariants::HybridPP5PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP5PackedArrayWyHash(inner),
                    HLLVariants::HybridPP5PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP6PackedArrayXxhasher(inner),
                    HLLVariants::HybridPP6PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridPP6PackedArrayWyHash(inner),
                    HLLVariants::HybridPP6PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB4ArrayXxhasher(inner),
                    HLLVariants::HybridLLB4ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB4ArrayWyHash(inner),
                    HLLVariants::HybridLLB4ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB5ArrayXxhasher(inner),
                    HLLVariants::HybridLLB5ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB5ArrayWyHash(inner),
                    HLLVariants::HybridLLB5ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB6ArrayXxhasher(inner),
                    HLLVariants::HybridLLB6ArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB6ArrayWyHash(inner),
                    HLLVariants::HybridLLB6ArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB4PackedArrayWyHash(inner),
                    HLLVariants::HybridLLB4PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB5PackedArrayXxhasher(inner),
                    HLLVariants::HybridLLB5PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB5PackedArrayWyHash(inner),
                    HLLVariants::HybridLLB5PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB6PackedArrayXxhasher(inner),
                    HLLVariants::HybridLLB6PackedArrayXxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridLLB6PackedArrayWyHash(inner),
                    HLLVariants::HybridLLB6PackedArrayWyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLEPP4WyHash(inner),
                    HLLVariants::HybridMLEPP4WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLEPP5WyHash(inner),
                    HLLVariants::HybridMLEPP5WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLEPP6WyHash(inner),
                    HLLVariants::HybridMLEPP6WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLEPP4Xxhasher(inner),
                    HLLVariants::HybridMLEPP4Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLEPP5Xxhasher(inner),
                    HLLVariants::HybridMLEPP5Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLEPP6Xxhasher(inner),
                    HLLVariants::HybridMLEPP6Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLELLB4WyHash(inner),
                    HLLVariants::HybridMLELLB4WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLELLB5WyHash(inner),
                    HLLVariants::HybridMLELLB5WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLELLB6WyHash(inner),
                    HLLVariants::HybridMLELLB6WyHash(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLELLB4Xxhasher(inner),
                    HLLVariants::HybridMLELLB4Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLELLB5Xxhasher(inner),
                    HLLVariants::HybridMLELLB5Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                (
                    HLLVariants::HybridMLELLB6Xxhasher(inner),
                    HLLVariants::HybridMLELLB6Xxhasher(other),
                ) => inner.is_union_estimate_non_deterministic(other),
                _ => {
                    ::core::panicking::panic_fmt(
                        format_args!("Union cardinality not defined for these variants."),
                    );
                }
            }
        }
    }
    impl<const EXPONENT: usize, P: Precision> TransparentMemSize
    for HLLVariants<EXPONENT, P>
    where
        P: ArrayRegisters + PackedArrayRegisters + Named + MemSize + MemDbg,
    {
        fn transparent_mem_size(&self) -> usize {
            match self {
                HLLVariants::TabacHyperLogLogPlus(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::TabacHyperLogLogPF(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::SAHyperLogLog(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::RustHyperLogLog(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::CE4(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::CE5(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::CE6(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::SimpleHLL(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP4ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP4ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP5ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP5ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP6ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP6ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP4PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP4PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP5PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP5PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP6PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::PP6PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB4ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB4ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB5ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB5ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB6ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB6ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB4PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB5PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB5PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB6PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::LLB6PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLEPP4WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLEPP5WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLEPP6WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLEPP4Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLEPP5Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLEPP6Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLELLB4WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLELLB5WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLELLB6WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLELLB4Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLELLB5Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::MLELLB6Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP4ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP4ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP5ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP5ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP6ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP6ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP4PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP4PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP5PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP5PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP6PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridPP6PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB4ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB4ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB5ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB5ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB6ArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB6ArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB4PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB5PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB5PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB6PackedArrayXxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridLLB6PackedArrayWyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLEPP4WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLEPP5WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLEPP6WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLEPP4Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLEPP5Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLEPP6Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLELLB4WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLELLB5WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLELLB6WyHash(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLELLB4Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLELLB5Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
                HLLVariants::HybridMLELLB6Xxhasher(inner) => {
                    inner
                        .mem_size(
                            mem_dbg::SizeFlags::default()
                                | mem_dbg::SizeFlags::FOLLOW_REFS,
                        )
                }
            }
        }
    }
    fn standard_deviation(values: &[f64], mean: f64) -> f64 {
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
            / values.len() as f64;
        variance.sqrt()
    }
    fn mean(values: &[f64]) -> f64 {
        values.iter().sum::<f64>() / values.len() as f64
    }
    fn mean_usize(values: &[usize]) -> f64 {
        values.iter().sum::<usize>() as f64 / values.len() as f64
    }
    /// Transposes a provided vector of vectors.
    pub fn transpose<T: Copy + Default>(vec: &Vec<Vec<T>>) -> Vec<Vec<T>> {
        let mut transposed_vector = ::alloc::vec::from_elem(
            ::alloc::vec::from_elem(T::default(), vec.len()),
            vec[0].len(),
        );
        let progress_bar = indicatif::ProgressBar::new(vec.len() as u64);
        progress_bar
            .set_style(
                indicatif::ProgressStyle::default_bar()
                    .template(
                        "Transposing: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
                    )
                    .unwrap()
                    .progress_chars("##-"),
            );
        for i in (0..vec.len()).progress_with(progress_bar) {
            for j in 0..vec[i].len() {
                transposed_vector[j][i] = vec[i][j];
            }
        }
        transposed_vector
    }
    fn write_csv<P: Precision, T: Display + Default + Copy>(
        transposed_data: &Vec<Vec<T>>,
        feature_name: &str,
        data_name: &str,
        approach_names: &[&str],
    ) {
        let mut writer = csv::Writer::from_path(
                &::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "./statistical_tests_reports/{0}_{1}_{2}.csv",
                            feature_name,
                            data_name,
                            P::EXPONENT,
                        ),
                    );
                    res
                }),
            )
            .unwrap();
        writer.write_record(approach_names.iter().copied()).unwrap();
        let progress_bar = indicatif::ProgressBar::new(transposed_data.len() as u64);
        progress_bar
            .set_style(
                indicatif::ProgressStyle::default_bar()
                    .template(
                        "Writing CSV: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
                    )
                    .unwrap()
                    .progress_chars("##-"),
            );
        for row in transposed_data.iter().progress_with(progress_bar) {
            match (&row.len(), &approach_names.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::None,
                        );
                    }
                }
            };
            let row: Vec<String> = row.iter().map(|v| v.to_string()).collect();
            writer.write_record(row).unwrap();
        }
        writer.flush().unwrap();
    }
    pub fn statistical_report<P: Precision>(
        approach_names: &[&str],
        features: Vec<f64>,
        transposed_absolute_errors: Vec<Vec<f64>>,
        transposed_memory_requirements: Vec<Vec<usize>>,
        feature_name: &str,
    ) {
        match (&transposed_absolute_errors[0].len(), &approach_names.len()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        match (&transposed_memory_requirements[0].len(), &approach_names.len()) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        };
        let transposed_features = transpose(
            &<[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([features])),
        );
        write_csv::<
            P,
            f64,
        >(&transposed_features, feature_name, "features", &["HashSet"]);
        write_csv::<
            P,
            f64,
        >(&transposed_absolute_errors, feature_name, "absolute_errors", approach_names);
        let absolute_errors = transpose(&transposed_absolute_errors);
        let memory_requirements = transpose(&transposed_memory_requirements);
        let means: Vec<f64> = absolute_errors
            .iter()
            .map(|errors| mean(errors))
            .collect();
        let stds: Vec<f64> = absolute_errors
            .iter()
            .zip(means.iter())
            .map(|(errors, mean)| standard_deviation(errors, *mean))
            .collect();
        let mut writer = csv::Writer::from_path(
                &::alloc::__export::must_use({
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "./statistical_tests_reports/{1}_{0}.csv",
                            P::EXPONENT,
                            feature_name,
                        ),
                    );
                    res
                }),
            )
            .unwrap();
        writer
            .write_record(
                &[
                    "feature",
                    "first_approach",
                    "second_approach",
                    "p-value",
                    "winner",
                    "first_memsize",
                    "first_mean",
                    "first_std",
                    "second_memsize",
                    "second_mean",
                    "second_std",
                    "precision",
                    "theoretical_error",
                ],
            )
            .unwrap();
        let progress_bar = indicatif::ProgressBar::new(approach_names.len() as u64);
        progress_bar
            .set_style(
                indicatif::ProgressStyle::default_bar()
                    .template(
                        "Running tests: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
                    )
                    .unwrap()
                    .progress_chars("##-"),
            );
        for (
            i,
            (
                (first_approach_name, first_memsize),
                (first_absolute_errors, (first_mean, first_std)),
            ),
        ) in approach_names
            .iter()
            .zip(memory_requirements.iter())
            .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
            .enumerate()
            .progress_with(progress_bar)
        {
            for (
                j,
                (
                    (second_approach_name, second_memsize),
                    (second_absolute_errors, (second_mean, second_std)),
                ),
            ) in approach_names
                .iter()
                .zip(memory_requirements.iter())
                .zip(absolute_errors.iter().zip(means.iter().zip(stds.iter())))
                .enumerate()
            {
                if i >= j {
                    continue;
                }
                let w_test = WilcoxonWTest::paired(
                    first_absolute_errors,
                    second_absolute_errors,
                );
                writer
                    .write_record(
                        &[
                            feature_name,
                            first_approach_name,
                            second_approach_name,
                            w_test
                                .as_ref()
                                .map(|w_test| ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0:.5}", w_test.p_value()),
                                    );
                                    res
                                }))
                                .unwrap_or("Unknown".to_owned())
                                .as_str(),
                            if let Ok(w_test) = w_test.as_ref() {
                                if w_test.p_value() < 0.05 {
                                    if first_mean < second_mean { "First" } else { "Second" }
                                } else {
                                    "None"
                                }
                            } else {
                                "Unknown"
                            },
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", mean_usize(first_memsize)),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", first_mean),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", first_std),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", mean_usize(second_memsize)),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", second_mean),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", second_std),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", P::EXPONENT),
                                    );
                                    res
                                })
                                .as_str(),
                            ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!("{0}", P::error_rate()),
                                    );
                                    res
                                })
                                .as_str(),
                        ],
                    )
                    .unwrap();
            }
        }
        writer.flush().unwrap();
    }
}
