
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_32_registers_and_6_bits(words: &[u32; 7]) -> (usize, f32) {
    let mut raw_estimate = 0.0;
    let mut number_of_zero_registers = 0;

	let [register_0, register_1, register_2, register_3, register_4] = split_registers::<5>(words[0]);
	raw_estimate += 1.0 / (1u64 << register_0) as f32 + 1.0 / (1u64 << register_1) as f32 + 1.0 / (1u64 << register_2) as f32 + 1.0 / (1u64 << register_3) as f32 + 1.0 / (1u64 << register_4) as f32;
	number_of_zero_registers += (register_0 == 0) as usize + (register_1 == 0) as usize + (register_2 == 0) as usize + (register_3 == 0) as usize + (register_4 == 0) as usize;
	let [register_5, register_6, register_7, register_8, register_9] = split_registers::<5>(words[1]);
	raw_estimate += 1.0 / (1u64 << register_5) as f32 + 1.0 / (1u64 << register_6) as f32 + 1.0 / (1u64 << register_7) as f32 + 1.0 / (1u64 << register_8) as f32 + 1.0 / (1u64 << register_9) as f32;
	number_of_zero_registers += (register_5 == 0) as usize + (register_6 == 0) as usize + (register_7 == 0) as usize + (register_8 == 0) as usize + (register_9 == 0) as usize;
	let [register_10, register_11, register_12, register_13, register_14] = split_registers::<5>(words[2]);
	raw_estimate += 1.0 / (1u64 << register_10) as f32 + 1.0 / (1u64 << register_11) as f32 + 1.0 / (1u64 << register_12) as f32 + 1.0 / (1u64 << register_13) as f32 + 1.0 / (1u64 << register_14) as f32;
	number_of_zero_registers += (register_10 == 0) as usize + (register_11 == 0) as usize + (register_12 == 0) as usize + (register_13 == 0) as usize + (register_14 == 0) as usize;
	let [register_15, register_16, register_17, register_18, register_19] = split_registers::<5>(words[3]);
	raw_estimate += 1.0 / (1u64 << register_15) as f32 + 1.0 / (1u64 << register_16) as f32 + 1.0 / (1u64 << register_17) as f32 + 1.0 / (1u64 << register_18) as f32 + 1.0 / (1u64 << register_19) as f32;
	number_of_zero_registers += (register_15 == 0) as usize + (register_16 == 0) as usize + (register_17 == 0) as usize + (register_18 == 0) as usize + (register_19 == 0) as usize;
	let [register_20, register_21, register_22, register_23, register_24] = split_registers::<5>(words[4]);
	raw_estimate += 1.0 / (1u64 << register_20) as f32 + 1.0 / (1u64 << register_21) as f32 + 1.0 / (1u64 << register_22) as f32 + 1.0 / (1u64 << register_23) as f32 + 1.0 / (1u64 << register_24) as f32;
	number_of_zero_registers += (register_20 == 0) as usize + (register_21 == 0) as usize + (register_22 == 0) as usize + (register_23 == 0) as usize + (register_24 == 0) as usize;
	let [register_25, register_26, register_27, register_28, register_29] = split_registers::<5>(words[5]);
	raw_estimate += 1.0 / (1u64 << register_25) as f32 + 1.0 / (1u64 << register_26) as f32 + 1.0 / (1u64 << register_27) as f32 + 1.0 / (1u64 << register_28) as f32 + 1.0 / (1u64 << register_29) as f32;
	number_of_zero_registers += (register_25 == 0) as usize + (register_26 == 0) as usize + (register_27 == 0) as usize + (register_28 == 0) as usize + (register_29 == 0) as usize;
	let [register_30, register_31, _, _, _] = split_registers::<5>(words[6]);
	raw_estimate += 1.0 / (1u64 << register_30) as f32 + 1.0 / (1u64 << register_31) as f32;
	number_of_zero_registers += (register_30 == 0) as usize + (register_31 == 0) as usize;

    (
number_of_zero_registers,
raw_estimate
    )
}
