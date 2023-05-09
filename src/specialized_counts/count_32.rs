
use crate::prelude::*;

#[inline]
pub fn count_32(registers: &[u32; 7]) -> (usize, f32) {
	let word_0 = registers[0];
	let word_1 = registers[1];
	let word_2 = registers[2];
	let word_3 = registers[3];
	let word_4 = registers[4];
	let word_5 = registers[5];
	let word_6 = registers[6];

	let register_0 = word_0 & LOWER_REGISTER_MASK;
	let register_1 = (word_0 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_2 = (word_0 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_3 = (word_0 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_4 = (word_0 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_5 = word_1 & LOWER_REGISTER_MASK;
	let register_6 = (word_1 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_7 = (word_1 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_8 = (word_1 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_9 = (word_1 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_10 = word_2 & LOWER_REGISTER_MASK;
	let register_11 = (word_2 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_12 = (word_2 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_13 = (word_2 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_14 = (word_2 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_15 = word_3 & LOWER_REGISTER_MASK;
	let register_16 = (word_3 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_17 = (word_3 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_18 = (word_3 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_19 = (word_3 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_20 = word_4 & LOWER_REGISTER_MASK;
	let register_21 = (word_4 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_22 = (word_4 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_23 = (word_4 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_24 = (word_4 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_25 = word_5 & LOWER_REGISTER_MASK;
	let register_26 = (word_5 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_27 = (word_5 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_28 = (word_5 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_29 = (word_5 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_30 = word_6 & LOWER_REGISTER_MASK;
	let register_31 = (word_6 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;

    (
		(register_0 == 0) as usize +
		(register_1 == 0) as usize +
		(register_2 == 0) as usize +
		(register_3 == 0) as usize +
		(register_4 == 0) as usize +
		(register_5 == 0) as usize +
		(register_6 == 0) as usize +
		(register_7 == 0) as usize +
		(register_8 == 0) as usize +
		(register_9 == 0) as usize +
		(register_10 == 0) as usize +
		(register_11 == 0) as usize +
		(register_12 == 0) as usize +
		(register_13 == 0) as usize +
		(register_14 == 0) as usize +
		(register_15 == 0) as usize +
		(register_16 == 0) as usize +
		(register_17 == 0) as usize +
		(register_18 == 0) as usize +
		(register_19 == 0) as usize +
		(register_20 == 0) as usize +
		(register_21 == 0) as usize +
		(register_22 == 0) as usize +
		(register_23 == 0) as usize +
		(register_24 == 0) as usize +
		(register_25 == 0) as usize +
		(register_26 == 0) as usize +
		(register_27 == 0) as usize +
		(register_28 == 0) as usize +
		(register_29 == 0) as usize +
		(register_30 == 0) as usize +
		(register_31 == 0) as usize,
		1.0_f32 / (1u64 << register_0) as f32 +
		1.0_f32 / (1u64 << register_1) as f32 +
		1.0_f32 / (1u64 << register_2) as f32 +
		1.0_f32 / (1u64 << register_3) as f32 +
		1.0_f32 / (1u64 << register_4) as f32 +
		1.0_f32 / (1u64 << register_5) as f32 +
		1.0_f32 / (1u64 << register_6) as f32 +
		1.0_f32 / (1u64 << register_7) as f32 +
		1.0_f32 / (1u64 << register_8) as f32 +
		1.0_f32 / (1u64 << register_9) as f32 +
		1.0_f32 / (1u64 << register_10) as f32 +
		1.0_f32 / (1u64 << register_11) as f32 +
		1.0_f32 / (1u64 << register_12) as f32 +
		1.0_f32 / (1u64 << register_13) as f32 +
		1.0_f32 / (1u64 << register_14) as f32 +
		1.0_f32 / (1u64 << register_15) as f32 +
		1.0_f32 / (1u64 << register_16) as f32 +
		1.0_f32 / (1u64 << register_17) as f32 +
		1.0_f32 / (1u64 << register_18) as f32 +
		1.0_f32 / (1u64 << register_19) as f32 +
		1.0_f32 / (1u64 << register_20) as f32 +
		1.0_f32 / (1u64 << register_21) as f32 +
		1.0_f32 / (1u64 << register_22) as f32 +
		1.0_f32 / (1u64 << register_23) as f32 +
		1.0_f32 / (1u64 << register_24) as f32 +
		1.0_f32 / (1u64 << register_25) as f32 +
		1.0_f32 / (1u64 << register_26) as f32 +
		1.0_f32 / (1u64 << register_27) as f32 +
		1.0_f32 / (1u64 << register_28) as f32 +
		1.0_f32 / (1u64 << register_29) as f32 +
		1.0_f32 / (1u64 << register_30) as f32 +
		1.0_f32 / (1u64 << register_31) as f32
    )
}
