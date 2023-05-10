
use crate::prelude::*;

#[inline]
pub fn get_generic_approximated_cardinality<const N: usize, const PRECISION: usize, const NUMBER_OF_REGISTERS_IN_WORD: usize>(
    words: &[u32; N],
) -> (usize, f32) {
    let number_of_registers: usize = 1 << PRECISION;
    let number_of_bits_per_register: usize = 32 / NUMBER_OF_REGISTERS_IN_WORD;
    let mask = (1 << number_of_bits_per_register) - 1;
    
    words
        .iter()
        .copied()
        .flat_map(|six_registers| {
            (0..NUMBER_OF_REGISTERS_IN_WORD).map(move |i| {
                six_registers >> i * number_of_bits_per_register & mask
            })
        })
        .take(number_of_registers)
        .fold((0, 0.0), |(number_of_zero_registers, raw_estimate), register|{
            (
                number_of_zero_registers + (register == 0) as usize,
                raw_estimate + 1.0 / (1u64 << register) as f32,
            )
        })
}

#[inline]
pub fn dispatch_specialized_count<
    const N: usize,
    const PRECISION: usize,
    const NUMBER_OF_REGISTERS_IN_WORD: usize
>(
    words: &[u32; N],
) -> (usize, f32) {
    match (N, PRECISION, NUMBER_OF_REGISTERS_IN_WORD) {
		(3, 4, 6) => get_approximated_cardinality_with_16_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(6, 5, 6) => get_approximated_cardinality_with_32_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(11, 6, 6) => get_approximated_cardinality_with_64_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(22, 7, 6) => get_approximated_cardinality_with_128_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(43, 8, 6) => get_approximated_cardinality_with_256_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(86, 9, 6) => get_approximated_cardinality_with_512_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(171, 10, 6) => get_approximated_cardinality_with_1024_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(342, 11, 6) => get_approximated_cardinality_with_2048_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(683, 12, 6) => get_approximated_cardinality_with_4096_registers_and_5_bits(unsafe { core::mem::transmute(words) }),
		(4, 4, 5) => get_approximated_cardinality_with_16_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(7, 5, 5) => get_approximated_cardinality_with_32_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(13, 6, 5) => get_approximated_cardinality_with_64_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(26, 7, 5) => get_approximated_cardinality_with_128_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(52, 8, 5) => get_approximated_cardinality_with_256_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(103, 9, 5) => get_approximated_cardinality_with_512_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(205, 10, 5) => get_approximated_cardinality_with_1024_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(410, 11, 5) => get_approximated_cardinality_with_2048_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(820, 12, 5) => get_approximated_cardinality_with_4096_registers_and_6_bits(unsafe { core::mem::transmute(words) }),
		(4, 4, 4) => get_approximated_cardinality_with_16_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(8, 5, 4) => get_approximated_cardinality_with_32_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(16, 6, 4) => get_approximated_cardinality_with_64_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(32, 7, 4) => get_approximated_cardinality_with_128_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(64, 8, 4) => get_approximated_cardinality_with_256_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(128, 9, 4) => get_approximated_cardinality_with_512_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(256, 10, 4) => get_approximated_cardinality_with_1024_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(512, 11, 4) => get_approximated_cardinality_with_2048_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
		(1024, 12, 4) => get_approximated_cardinality_with_4096_registers_and_8_bits(unsafe { core::mem::transmute(words) }),
        _ => get_generic_approximated_cardinality::<N, PRECISION, NUMBER_OF_REGISTERS_IN_WORD>(words)
    }
}

