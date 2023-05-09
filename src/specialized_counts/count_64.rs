
use crate::prelude::*;

#[inline]
pub fn count_64(registers: &[u32; 13]) -> (usize, f32) {
	let word_0 = registers[0];
	let word_1 = registers[1];
	let word_2 = registers[2];
	let word_3 = registers[3];
	let word_4 = registers[4];
	let word_5 = registers[5];
	let word_6 = registers[6];
	let word_7 = registers[7];
	let word_8 = registers[8];
	let word_9 = registers[9];
	let word_10 = registers[10];
	let word_11 = registers[11];
	let word_12 = registers[12];

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
	let register_32 = (word_6 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_33 = (word_6 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_34 = (word_6 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_35 = word_7 & LOWER_REGISTER_MASK;
	let register_36 = (word_7 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_37 = (word_7 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_38 = (word_7 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_39 = (word_7 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_40 = word_8 & LOWER_REGISTER_MASK;
	let register_41 = (word_8 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_42 = (word_8 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_43 = (word_8 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_44 = (word_8 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_45 = word_9 & LOWER_REGISTER_MASK;
	let register_46 = (word_9 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_47 = (word_9 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_48 = (word_9 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_49 = (word_9 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_50 = word_10 & LOWER_REGISTER_MASK;
	let register_51 = (word_10 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_52 = (word_10 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_53 = (word_10 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_54 = (word_10 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_55 = word_11 & LOWER_REGISTER_MASK;
	let register_56 = (word_11 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_57 = (word_11 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_58 = (word_11 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_59 = (word_11 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_60 = word_12 & LOWER_REGISTER_MASK;
	let register_61 = (word_12 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_62 = (word_12 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_63 = (word_12 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;

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
		(register_31 == 0) as usize +
		(register_32 == 0) as usize +
		(register_33 == 0) as usize +
		(register_34 == 0) as usize +
		(register_35 == 0) as usize +
		(register_36 == 0) as usize +
		(register_37 == 0) as usize +
		(register_38 == 0) as usize +
		(register_39 == 0) as usize +
		(register_40 == 0) as usize +
		(register_41 == 0) as usize +
		(register_42 == 0) as usize +
		(register_43 == 0) as usize +
		(register_44 == 0) as usize +
		(register_45 == 0) as usize +
		(register_46 == 0) as usize +
		(register_47 == 0) as usize +
		(register_48 == 0) as usize +
		(register_49 == 0) as usize +
		(register_50 == 0) as usize +
		(register_51 == 0) as usize +
		(register_52 == 0) as usize +
		(register_53 == 0) as usize +
		(register_54 == 0) as usize +
		(register_55 == 0) as usize +
		(register_56 == 0) as usize +
		(register_57 == 0) as usize +
		(register_58 == 0) as usize +
		(register_59 == 0) as usize +
		(register_60 == 0) as usize +
		(register_61 == 0) as usize +
		(register_62 == 0) as usize +
		(register_63 == 0) as usize,
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
		1.0_f32 / (1u64 << register_31) as f32 +
		1.0_f32 / (1u64 << register_32) as f32 +
		1.0_f32 / (1u64 << register_33) as f32 +
		1.0_f32 / (1u64 << register_34) as f32 +
		1.0_f32 / (1u64 << register_35) as f32 +
		1.0_f32 / (1u64 << register_36) as f32 +
		1.0_f32 / (1u64 << register_37) as f32 +
		1.0_f32 / (1u64 << register_38) as f32 +
		1.0_f32 / (1u64 << register_39) as f32 +
		1.0_f32 / (1u64 << register_40) as f32 +
		1.0_f32 / (1u64 << register_41) as f32 +
		1.0_f32 / (1u64 << register_42) as f32 +
		1.0_f32 / (1u64 << register_43) as f32 +
		1.0_f32 / (1u64 << register_44) as f32 +
		1.0_f32 / (1u64 << register_45) as f32 +
		1.0_f32 / (1u64 << register_46) as f32 +
		1.0_f32 / (1u64 << register_47) as f32 +
		1.0_f32 / (1u64 << register_48) as f32 +
		1.0_f32 / (1u64 << register_49) as f32 +
		1.0_f32 / (1u64 << register_50) as f32 +
		1.0_f32 / (1u64 << register_51) as f32 +
		1.0_f32 / (1u64 << register_52) as f32 +
		1.0_f32 / (1u64 << register_53) as f32 +
		1.0_f32 / (1u64 << register_54) as f32 +
		1.0_f32 / (1u64 << register_55) as f32 +
		1.0_f32 / (1u64 << register_56) as f32 +
		1.0_f32 / (1u64 << register_57) as f32 +
		1.0_f32 / (1u64 << register_58) as f32 +
		1.0_f32 / (1u64 << register_59) as f32 +
		1.0_f32 / (1u64 << register_60) as f32 +
		1.0_f32 / (1u64 << register_61) as f32 +
		1.0_f32 / (1u64 << register_62) as f32 +
		1.0_f32 / (1u64 << register_63) as f32
    )
}
