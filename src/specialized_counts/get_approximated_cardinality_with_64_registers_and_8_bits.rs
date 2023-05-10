
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_64_registers_and_8_bits(words: &[u32; 16]) -> (usize, f32) {
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
	let [register_32, register_33, register_34, register_35] = split_registers::<4>(words[8]);
	raw_estimate += 1.0 / (1u64 << register_32) as f32 + 1.0 / (1u64 << register_33) as f32 + 1.0 / (1u64 << register_34) as f32 + 1.0 / (1u64 << register_35) as f32;
	number_of_zero_registers += (register_32 == 0) as usize + (register_33 == 0) as usize + (register_34 == 0) as usize + (register_35 == 0) as usize;
	let [register_36, register_37, register_38, register_39] = split_registers::<4>(words[9]);
	raw_estimate += 1.0 / (1u64 << register_36) as f32 + 1.0 / (1u64 << register_37) as f32 + 1.0 / (1u64 << register_38) as f32 + 1.0 / (1u64 << register_39) as f32;
	number_of_zero_registers += (register_36 == 0) as usize + (register_37 == 0) as usize + (register_38 == 0) as usize + (register_39 == 0) as usize;
	let [register_40, register_41, register_42, register_43] = split_registers::<4>(words[10]);
	raw_estimate += 1.0 / (1u64 << register_40) as f32 + 1.0 / (1u64 << register_41) as f32 + 1.0 / (1u64 << register_42) as f32 + 1.0 / (1u64 << register_43) as f32;
	number_of_zero_registers += (register_40 == 0) as usize + (register_41 == 0) as usize + (register_42 == 0) as usize + (register_43 == 0) as usize;
	let [register_44, register_45, register_46, register_47] = split_registers::<4>(words[11]);
	raw_estimate += 1.0 / (1u64 << register_44) as f32 + 1.0 / (1u64 << register_45) as f32 + 1.0 / (1u64 << register_46) as f32 + 1.0 / (1u64 << register_47) as f32;
	number_of_zero_registers += (register_44 == 0) as usize + (register_45 == 0) as usize + (register_46 == 0) as usize + (register_47 == 0) as usize;
	let [register_48, register_49, register_50, register_51] = split_registers::<4>(words[12]);
	raw_estimate += 1.0 / (1u64 << register_48) as f32 + 1.0 / (1u64 << register_49) as f32 + 1.0 / (1u64 << register_50) as f32 + 1.0 / (1u64 << register_51) as f32;
	number_of_zero_registers += (register_48 == 0) as usize + (register_49 == 0) as usize + (register_50 == 0) as usize + (register_51 == 0) as usize;
	let [register_52, register_53, register_54, register_55] = split_registers::<4>(words[13]);
	raw_estimate += 1.0 / (1u64 << register_52) as f32 + 1.0 / (1u64 << register_53) as f32 + 1.0 / (1u64 << register_54) as f32 + 1.0 / (1u64 << register_55) as f32;
	number_of_zero_registers += (register_52 == 0) as usize + (register_53 == 0) as usize + (register_54 == 0) as usize + (register_55 == 0) as usize;
	let [register_56, register_57, register_58, register_59] = split_registers::<4>(words[14]);
	raw_estimate += 1.0 / (1u64 << register_56) as f32 + 1.0 / (1u64 << register_57) as f32 + 1.0 / (1u64 << register_58) as f32 + 1.0 / (1u64 << register_59) as f32;
	number_of_zero_registers += (register_56 == 0) as usize + (register_57 == 0) as usize + (register_58 == 0) as usize + (register_59 == 0) as usize;
	let [register_60, register_61, register_62, register_63] = split_registers::<4>(words[15]);
	raw_estimate += 1.0 / (1u64 << register_60) as f32 + 1.0 / (1u64 << register_61) as f32 + 1.0 / (1u64 << register_62) as f32 + 1.0 / (1u64 << register_63) as f32;
	number_of_zero_registers += (register_60 == 0) as usize + (register_61 == 0) as usize + (register_62 == 0) as usize + (register_63 == 0) as usize;

    (
number_of_zero_registers,
raw_estimate
    )
}
