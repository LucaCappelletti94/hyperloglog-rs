
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_32_registers_and_8_bits(words: &[u32; 8]) -> (usize, f32) {
    let mut raw_estimate = 0.0;
    let mut number_of_zero_registers = 0;

	let [register_0, register_1, register_2, register_3] = split_registers::<4>(words[0]);
	raw_estimate += 1.0 / (1u64 << register_0) as f32 + 1.0 / (1u64 << register_1) as f32 + 1.0 / (1u64 << register_2) as f32 + 1.0 / (1u64 << register_3) as f32;
	number_of_zero_registers += (register_0 == 0) as usize + (register_1 == 0) as usize + (register_2 == 0) as usize + (register_3 == 0) as usize;
	let [register_4, register_5, register_6, register_7] = split_registers::<4>(words[1]);
	raw_estimate += 1.0 / (1u64 << register_4) as f32 + 1.0 / (1u64 << register_5) as f32 + 1.0 / (1u64 << register_6) as f32 + 1.0 / (1u64 << register_7) as f32;
	number_of_zero_registers += (register_4 == 0) as usize + (register_5 == 0) as usize + (register_6 == 0) as usize + (register_7 == 0) as usize;
	let [register_8, register_9, register_10, register_11] = split_registers::<4>(words[2]);
	raw_estimate += 1.0 / (1u64 << register_8) as f32 + 1.0 / (1u64 << register_9) as f32 + 1.0 / (1u64 << register_10) as f32 + 1.0 / (1u64 << register_11) as f32;
	number_of_zero_registers += (register_8 == 0) as usize + (register_9 == 0) as usize + (register_10 == 0) as usize + (register_11 == 0) as usize;
	let [register_12, register_13, register_14, register_15] = split_registers::<4>(words[3]);
	raw_estimate += 1.0 / (1u64 << register_12) as f32 + 1.0 / (1u64 << register_13) as f32 + 1.0 / (1u64 << register_14) as f32 + 1.0 / (1u64 << register_15) as f32;
	number_of_zero_registers += (register_12 == 0) as usize + (register_13 == 0) as usize + (register_14 == 0) as usize + (register_15 == 0) as usize;
	let [register_16, register_17, register_18, register_19] = split_registers::<4>(words[4]);
	raw_estimate += 1.0 / (1u64 << register_16) as f32 + 1.0 / (1u64 << register_17) as f32 + 1.0 / (1u64 << register_18) as f32 + 1.0 / (1u64 << register_19) as f32;
	number_of_zero_registers += (register_16 == 0) as usize + (register_17 == 0) as usize + (register_18 == 0) as usize + (register_19 == 0) as usize;
	let [register_20, register_21, register_22, register_23] = split_registers::<4>(words[5]);
	raw_estimate += 1.0 / (1u64 << register_20) as f32 + 1.0 / (1u64 << register_21) as f32 + 1.0 / (1u64 << register_22) as f32 + 1.0 / (1u64 << register_23) as f32;
	number_of_zero_registers += (register_20 == 0) as usize + (register_21 == 0) as usize + (register_22 == 0) as usize + (register_23 == 0) as usize;
	let [register_24, register_25, register_26, register_27] = split_registers::<4>(words[6]);
	raw_estimate += 1.0 / (1u64 << register_24) as f32 + 1.0 / (1u64 << register_25) as f32 + 1.0 / (1u64 << register_26) as f32 + 1.0 / (1u64 << register_27) as f32;
	number_of_zero_registers += (register_24 == 0) as usize + (register_25 == 0) as usize + (register_26 == 0) as usize + (register_27 == 0) as usize;
	let [register_28, register_29, register_30, register_31] = split_registers::<4>(words[7]);
	raw_estimate += 1.0 / (1u64 << register_28) as f32 + 1.0 / (1u64 << register_29) as f32 + 1.0 / (1u64 << register_30) as f32 + 1.0 / (1u64 << register_31) as f32;
	number_of_zero_registers += (register_28 == 0) as usize + (register_29 == 0) as usize + (register_30 == 0) as usize + (register_31 == 0) as usize;

    (
number_of_zero_registers,
raw_estimate
    )
}
