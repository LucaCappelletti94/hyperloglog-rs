
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_128_registers_and_5_bits(words: &[u32; 22]) -> f32 {
    let mut raw_estimate = 0.0;

	let [register_0, register_1, register_2, register_3, register_4, register_5] = split_registers::<6>(words[0]);
	raw_estimate += 1.0 / (1u64 << register_0) as f32 + 1.0 / (1u64 << register_1) as f32 + 1.0 / (1u64 << register_2) as f32 + 1.0 / (1u64 << register_3) as f32 + 1.0 / (1u64 << register_4) as f32 + 1.0 / (1u64 << register_5) as f32;

	let [register_6, register_7, register_8, register_9, register_10, register_11] = split_registers::<6>(words[1]);
	raw_estimate += 1.0 / (1u64 << register_6) as f32 + 1.0 / (1u64 << register_7) as f32 + 1.0 / (1u64 << register_8) as f32 + 1.0 / (1u64 << register_9) as f32 + 1.0 / (1u64 << register_10) as f32 + 1.0 / (1u64 << register_11) as f32;

	let [register_12, register_13, register_14, register_15, register_16, register_17] = split_registers::<6>(words[2]);
	raw_estimate += 1.0 / (1u64 << register_12) as f32 + 1.0 / (1u64 << register_13) as f32 + 1.0 / (1u64 << register_14) as f32 + 1.0 / (1u64 << register_15) as f32 + 1.0 / (1u64 << register_16) as f32 + 1.0 / (1u64 << register_17) as f32;

	let [register_18, register_19, register_20, register_21, register_22, register_23] = split_registers::<6>(words[3]);
	raw_estimate += 1.0 / (1u64 << register_18) as f32 + 1.0 / (1u64 << register_19) as f32 + 1.0 / (1u64 << register_20) as f32 + 1.0 / (1u64 << register_21) as f32 + 1.0 / (1u64 << register_22) as f32 + 1.0 / (1u64 << register_23) as f32;

	let [register_24, register_25, register_26, register_27, register_28, register_29] = split_registers::<6>(words[4]);
	raw_estimate += 1.0 / (1u64 << register_24) as f32 + 1.0 / (1u64 << register_25) as f32 + 1.0 / (1u64 << register_26) as f32 + 1.0 / (1u64 << register_27) as f32 + 1.0 / (1u64 << register_28) as f32 + 1.0 / (1u64 << register_29) as f32;

	let [register_30, register_31, register_32, register_33, register_34, register_35] = split_registers::<6>(words[5]);
	raw_estimate += 1.0 / (1u64 << register_30) as f32 + 1.0 / (1u64 << register_31) as f32 + 1.0 / (1u64 << register_32) as f32 + 1.0 / (1u64 << register_33) as f32 + 1.0 / (1u64 << register_34) as f32 + 1.0 / (1u64 << register_35) as f32;

	let [register_36, register_37, register_38, register_39, register_40, register_41] = split_registers::<6>(words[6]);
	raw_estimate += 1.0 / (1u64 << register_36) as f32 + 1.0 / (1u64 << register_37) as f32 + 1.0 / (1u64 << register_38) as f32 + 1.0 / (1u64 << register_39) as f32 + 1.0 / (1u64 << register_40) as f32 + 1.0 / (1u64 << register_41) as f32;

	let [register_42, register_43, register_44, register_45, register_46, register_47] = split_registers::<6>(words[7]);
	raw_estimate += 1.0 / (1u64 << register_42) as f32 + 1.0 / (1u64 << register_43) as f32 + 1.0 / (1u64 << register_44) as f32 + 1.0 / (1u64 << register_45) as f32 + 1.0 / (1u64 << register_46) as f32 + 1.0 / (1u64 << register_47) as f32;

	let [register_48, register_49, register_50, register_51, register_52, register_53] = split_registers::<6>(words[8]);
	raw_estimate += 1.0 / (1u64 << register_48) as f32 + 1.0 / (1u64 << register_49) as f32 + 1.0 / (1u64 << register_50) as f32 + 1.0 / (1u64 << register_51) as f32 + 1.0 / (1u64 << register_52) as f32 + 1.0 / (1u64 << register_53) as f32;

	let [register_54, register_55, register_56, register_57, register_58, register_59] = split_registers::<6>(words[9]);
	raw_estimate += 1.0 / (1u64 << register_54) as f32 + 1.0 / (1u64 << register_55) as f32 + 1.0 / (1u64 << register_56) as f32 + 1.0 / (1u64 << register_57) as f32 + 1.0 / (1u64 << register_58) as f32 + 1.0 / (1u64 << register_59) as f32;

	let [register_60, register_61, register_62, register_63, register_64, register_65] = split_registers::<6>(words[10]);
	raw_estimate += 1.0 / (1u64 << register_60) as f32 + 1.0 / (1u64 << register_61) as f32 + 1.0 / (1u64 << register_62) as f32 + 1.0 / (1u64 << register_63) as f32 + 1.0 / (1u64 << register_64) as f32 + 1.0 / (1u64 << register_65) as f32;

	let [register_66, register_67, register_68, register_69, register_70, register_71] = split_registers::<6>(words[11]);
	raw_estimate += 1.0 / (1u64 << register_66) as f32 + 1.0 / (1u64 << register_67) as f32 + 1.0 / (1u64 << register_68) as f32 + 1.0 / (1u64 << register_69) as f32 + 1.0 / (1u64 << register_70) as f32 + 1.0 / (1u64 << register_71) as f32;

	let [register_72, register_73, register_74, register_75, register_76, register_77] = split_registers::<6>(words[12]);
	raw_estimate += 1.0 / (1u64 << register_72) as f32 + 1.0 / (1u64 << register_73) as f32 + 1.0 / (1u64 << register_74) as f32 + 1.0 / (1u64 << register_75) as f32 + 1.0 / (1u64 << register_76) as f32 + 1.0 / (1u64 << register_77) as f32;

	let [register_78, register_79, register_80, register_81, register_82, register_83] = split_registers::<6>(words[13]);
	raw_estimate += 1.0 / (1u64 << register_78) as f32 + 1.0 / (1u64 << register_79) as f32 + 1.0 / (1u64 << register_80) as f32 + 1.0 / (1u64 << register_81) as f32 + 1.0 / (1u64 << register_82) as f32 + 1.0 / (1u64 << register_83) as f32;

	let [register_84, register_85, register_86, register_87, register_88, register_89] = split_registers::<6>(words[14]);
	raw_estimate += 1.0 / (1u64 << register_84) as f32 + 1.0 / (1u64 << register_85) as f32 + 1.0 / (1u64 << register_86) as f32 + 1.0 / (1u64 << register_87) as f32 + 1.0 / (1u64 << register_88) as f32 + 1.0 / (1u64 << register_89) as f32;

	let [register_90, register_91, register_92, register_93, register_94, register_95] = split_registers::<6>(words[15]);
	raw_estimate += 1.0 / (1u64 << register_90) as f32 + 1.0 / (1u64 << register_91) as f32 + 1.0 / (1u64 << register_92) as f32 + 1.0 / (1u64 << register_93) as f32 + 1.0 / (1u64 << register_94) as f32 + 1.0 / (1u64 << register_95) as f32;

	let [register_96, register_97, register_98, register_99, register_100, register_101] = split_registers::<6>(words[16]);
	raw_estimate += 1.0 / (1u64 << register_96) as f32 + 1.0 / (1u64 << register_97) as f32 + 1.0 / (1u64 << register_98) as f32 + 1.0 / (1u64 << register_99) as f32 + 1.0 / (1u64 << register_100) as f32 + 1.0 / (1u64 << register_101) as f32;

	let [register_102, register_103, register_104, register_105, register_106, register_107] = split_registers::<6>(words[17]);
	raw_estimate += 1.0 / (1u64 << register_102) as f32 + 1.0 / (1u64 << register_103) as f32 + 1.0 / (1u64 << register_104) as f32 + 1.0 / (1u64 << register_105) as f32 + 1.0 / (1u64 << register_106) as f32 + 1.0 / (1u64 << register_107) as f32;

	let [register_108, register_109, register_110, register_111, register_112, register_113] = split_registers::<6>(words[18]);
	raw_estimate += 1.0 / (1u64 << register_108) as f32 + 1.0 / (1u64 << register_109) as f32 + 1.0 / (1u64 << register_110) as f32 + 1.0 / (1u64 << register_111) as f32 + 1.0 / (1u64 << register_112) as f32 + 1.0 / (1u64 << register_113) as f32;

	let [register_114, register_115, register_116, register_117, register_118, register_119] = split_registers::<6>(words[19]);
	raw_estimate += 1.0 / (1u64 << register_114) as f32 + 1.0 / (1u64 << register_115) as f32 + 1.0 / (1u64 << register_116) as f32 + 1.0 / (1u64 << register_117) as f32 + 1.0 / (1u64 << register_118) as f32 + 1.0 / (1u64 << register_119) as f32;

	let [register_120, register_121, register_122, register_123, register_124, register_125] = split_registers::<6>(words[20]);
	raw_estimate += 1.0 / (1u64 << register_120) as f32 + 1.0 / (1u64 << register_121) as f32 + 1.0 / (1u64 << register_122) as f32 + 1.0 / (1u64 << register_123) as f32 + 1.0 / (1u64 << register_124) as f32 + 1.0 / (1u64 << register_125) as f32;

	let [register_126, register_127, _, _, _, _] = split_registers::<6>(words[21]);
	raw_estimate += 1.0 / (1u64 << register_126) as f32 + 1.0 / (1u64 << register_127) as f32;


    raw_estimate
}
