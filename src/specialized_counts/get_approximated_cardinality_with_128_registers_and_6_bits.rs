
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_128_registers_and_6_bits(words: &[u32; 26]) -> (usize, f32) {
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
	let [register_30, register_31, register_32, register_33, register_34] = split_registers::<5>(words[6]);
	raw_estimate += 1.0 / (1u64 << register_30) as f32 + 1.0 / (1u64 << register_31) as f32 + 1.0 / (1u64 << register_32) as f32 + 1.0 / (1u64 << register_33) as f32 + 1.0 / (1u64 << register_34) as f32;
	number_of_zero_registers += (register_30 == 0) as usize + (register_31 == 0) as usize + (register_32 == 0) as usize + (register_33 == 0) as usize + (register_34 == 0) as usize;
	let [register_35, register_36, register_37, register_38, register_39] = split_registers::<5>(words[7]);
	raw_estimate += 1.0 / (1u64 << register_35) as f32 + 1.0 / (1u64 << register_36) as f32 + 1.0 / (1u64 << register_37) as f32 + 1.0 / (1u64 << register_38) as f32 + 1.0 / (1u64 << register_39) as f32;
	number_of_zero_registers += (register_35 == 0) as usize + (register_36 == 0) as usize + (register_37 == 0) as usize + (register_38 == 0) as usize + (register_39 == 0) as usize;
	let [register_40, register_41, register_42, register_43, register_44] = split_registers::<5>(words[8]);
	raw_estimate += 1.0 / (1u64 << register_40) as f32 + 1.0 / (1u64 << register_41) as f32 + 1.0 / (1u64 << register_42) as f32 + 1.0 / (1u64 << register_43) as f32 + 1.0 / (1u64 << register_44) as f32;
	number_of_zero_registers += (register_40 == 0) as usize + (register_41 == 0) as usize + (register_42 == 0) as usize + (register_43 == 0) as usize + (register_44 == 0) as usize;
	let [register_45, register_46, register_47, register_48, register_49] = split_registers::<5>(words[9]);
	raw_estimate += 1.0 / (1u64 << register_45) as f32 + 1.0 / (1u64 << register_46) as f32 + 1.0 / (1u64 << register_47) as f32 + 1.0 / (1u64 << register_48) as f32 + 1.0 / (1u64 << register_49) as f32;
	number_of_zero_registers += (register_45 == 0) as usize + (register_46 == 0) as usize + (register_47 == 0) as usize + (register_48 == 0) as usize + (register_49 == 0) as usize;
	let [register_50, register_51, register_52, register_53, register_54] = split_registers::<5>(words[10]);
	raw_estimate += 1.0 / (1u64 << register_50) as f32 + 1.0 / (1u64 << register_51) as f32 + 1.0 / (1u64 << register_52) as f32 + 1.0 / (1u64 << register_53) as f32 + 1.0 / (1u64 << register_54) as f32;
	number_of_zero_registers += (register_50 == 0) as usize + (register_51 == 0) as usize + (register_52 == 0) as usize + (register_53 == 0) as usize + (register_54 == 0) as usize;
	let [register_55, register_56, register_57, register_58, register_59] = split_registers::<5>(words[11]);
	raw_estimate += 1.0 / (1u64 << register_55) as f32 + 1.0 / (1u64 << register_56) as f32 + 1.0 / (1u64 << register_57) as f32 + 1.0 / (1u64 << register_58) as f32 + 1.0 / (1u64 << register_59) as f32;
	number_of_zero_registers += (register_55 == 0) as usize + (register_56 == 0) as usize + (register_57 == 0) as usize + (register_58 == 0) as usize + (register_59 == 0) as usize;
	let [register_60, register_61, register_62, register_63, register_64] = split_registers::<5>(words[12]);
	raw_estimate += 1.0 / (1u64 << register_60) as f32 + 1.0 / (1u64 << register_61) as f32 + 1.0 / (1u64 << register_62) as f32 + 1.0 / (1u64 << register_63) as f32 + 1.0 / (1u64 << register_64) as f32;
	number_of_zero_registers += (register_60 == 0) as usize + (register_61 == 0) as usize + (register_62 == 0) as usize + (register_63 == 0) as usize + (register_64 == 0) as usize;
	let [register_65, register_66, register_67, register_68, register_69] = split_registers::<5>(words[13]);
	raw_estimate += 1.0 / (1u64 << register_65) as f32 + 1.0 / (1u64 << register_66) as f32 + 1.0 / (1u64 << register_67) as f32 + 1.0 / (1u64 << register_68) as f32 + 1.0 / (1u64 << register_69) as f32;
	number_of_zero_registers += (register_65 == 0) as usize + (register_66 == 0) as usize + (register_67 == 0) as usize + (register_68 == 0) as usize + (register_69 == 0) as usize;
	let [register_70, register_71, register_72, register_73, register_74] = split_registers::<5>(words[14]);
	raw_estimate += 1.0 / (1u64 << register_70) as f32 + 1.0 / (1u64 << register_71) as f32 + 1.0 / (1u64 << register_72) as f32 + 1.0 / (1u64 << register_73) as f32 + 1.0 / (1u64 << register_74) as f32;
	number_of_zero_registers += (register_70 == 0) as usize + (register_71 == 0) as usize + (register_72 == 0) as usize + (register_73 == 0) as usize + (register_74 == 0) as usize;
	let [register_75, register_76, register_77, register_78, register_79] = split_registers::<5>(words[15]);
	raw_estimate += 1.0 / (1u64 << register_75) as f32 + 1.0 / (1u64 << register_76) as f32 + 1.0 / (1u64 << register_77) as f32 + 1.0 / (1u64 << register_78) as f32 + 1.0 / (1u64 << register_79) as f32;
	number_of_zero_registers += (register_75 == 0) as usize + (register_76 == 0) as usize + (register_77 == 0) as usize + (register_78 == 0) as usize + (register_79 == 0) as usize;
	let [register_80, register_81, register_82, register_83, register_84] = split_registers::<5>(words[16]);
	raw_estimate += 1.0 / (1u64 << register_80) as f32 + 1.0 / (1u64 << register_81) as f32 + 1.0 / (1u64 << register_82) as f32 + 1.0 / (1u64 << register_83) as f32 + 1.0 / (1u64 << register_84) as f32;
	number_of_zero_registers += (register_80 == 0) as usize + (register_81 == 0) as usize + (register_82 == 0) as usize + (register_83 == 0) as usize + (register_84 == 0) as usize;
	let [register_85, register_86, register_87, register_88, register_89] = split_registers::<5>(words[17]);
	raw_estimate += 1.0 / (1u64 << register_85) as f32 + 1.0 / (1u64 << register_86) as f32 + 1.0 / (1u64 << register_87) as f32 + 1.0 / (1u64 << register_88) as f32 + 1.0 / (1u64 << register_89) as f32;
	number_of_zero_registers += (register_85 == 0) as usize + (register_86 == 0) as usize + (register_87 == 0) as usize + (register_88 == 0) as usize + (register_89 == 0) as usize;
	let [register_90, register_91, register_92, register_93, register_94] = split_registers::<5>(words[18]);
	raw_estimate += 1.0 / (1u64 << register_90) as f32 + 1.0 / (1u64 << register_91) as f32 + 1.0 / (1u64 << register_92) as f32 + 1.0 / (1u64 << register_93) as f32 + 1.0 / (1u64 << register_94) as f32;
	number_of_zero_registers += (register_90 == 0) as usize + (register_91 == 0) as usize + (register_92 == 0) as usize + (register_93 == 0) as usize + (register_94 == 0) as usize;
	let [register_95, register_96, register_97, register_98, register_99] = split_registers::<5>(words[19]);
	raw_estimate += 1.0 / (1u64 << register_95) as f32 + 1.0 / (1u64 << register_96) as f32 + 1.0 / (1u64 << register_97) as f32 + 1.0 / (1u64 << register_98) as f32 + 1.0 / (1u64 << register_99) as f32;
	number_of_zero_registers += (register_95 == 0) as usize + (register_96 == 0) as usize + (register_97 == 0) as usize + (register_98 == 0) as usize + (register_99 == 0) as usize;
	let [register_100, register_101, register_102, register_103, register_104] = split_registers::<5>(words[20]);
	raw_estimate += 1.0 / (1u64 << register_100) as f32 + 1.0 / (1u64 << register_101) as f32 + 1.0 / (1u64 << register_102) as f32 + 1.0 / (1u64 << register_103) as f32 + 1.0 / (1u64 << register_104) as f32;
	number_of_zero_registers += (register_100 == 0) as usize + (register_101 == 0) as usize + (register_102 == 0) as usize + (register_103 == 0) as usize + (register_104 == 0) as usize;
	let [register_105, register_106, register_107, register_108, register_109] = split_registers::<5>(words[21]);
	raw_estimate += 1.0 / (1u64 << register_105) as f32 + 1.0 / (1u64 << register_106) as f32 + 1.0 / (1u64 << register_107) as f32 + 1.0 / (1u64 << register_108) as f32 + 1.0 / (1u64 << register_109) as f32;
	number_of_zero_registers += (register_105 == 0) as usize + (register_106 == 0) as usize + (register_107 == 0) as usize + (register_108 == 0) as usize + (register_109 == 0) as usize;
	let [register_110, register_111, register_112, register_113, register_114] = split_registers::<5>(words[22]);
	raw_estimate += 1.0 / (1u64 << register_110) as f32 + 1.0 / (1u64 << register_111) as f32 + 1.0 / (1u64 << register_112) as f32 + 1.0 / (1u64 << register_113) as f32 + 1.0 / (1u64 << register_114) as f32;
	number_of_zero_registers += (register_110 == 0) as usize + (register_111 == 0) as usize + (register_112 == 0) as usize + (register_113 == 0) as usize + (register_114 == 0) as usize;
	let [register_115, register_116, register_117, register_118, register_119] = split_registers::<5>(words[23]);
	raw_estimate += 1.0 / (1u64 << register_115) as f32 + 1.0 / (1u64 << register_116) as f32 + 1.0 / (1u64 << register_117) as f32 + 1.0 / (1u64 << register_118) as f32 + 1.0 / (1u64 << register_119) as f32;
	number_of_zero_registers += (register_115 == 0) as usize + (register_116 == 0) as usize + (register_117 == 0) as usize + (register_118 == 0) as usize + (register_119 == 0) as usize;
	let [register_120, register_121, register_122, register_123, register_124] = split_registers::<5>(words[24]);
	raw_estimate += 1.0 / (1u64 << register_120) as f32 + 1.0 / (1u64 << register_121) as f32 + 1.0 / (1u64 << register_122) as f32 + 1.0 / (1u64 << register_123) as f32 + 1.0 / (1u64 << register_124) as f32;
	number_of_zero_registers += (register_120 == 0) as usize + (register_121 == 0) as usize + (register_122 == 0) as usize + (register_123 == 0) as usize + (register_124 == 0) as usize;
	let [register_125, register_126, register_127, _, _] = split_registers::<5>(words[25]);
	raw_estimate += 1.0 / (1u64 << register_125) as f32 + 1.0 / (1u64 << register_126) as f32 + 1.0 / (1u64 << register_127) as f32;
	number_of_zero_registers += (register_125 == 0) as usize + (register_126 == 0) as usize + (register_127 == 0) as usize;

    (
number_of_zero_registers,
raw_estimate
    )
}
