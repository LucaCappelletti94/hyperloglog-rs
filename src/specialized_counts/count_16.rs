
use crate::prelude::*;

#[inline]
pub fn count_16(registers: &[u32; 4]) -> (usize, f32) {
	let word_0 = registers[0];
	let word_1 = registers[1];
	let word_2 = registers[2];
	let word_3 = registers[3];

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
		(register_15 == 0) as usize,
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
		1.0_f32 / (1u64 << register_15) as f32
    )
}
