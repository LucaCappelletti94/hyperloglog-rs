
use crate::prelude::*;

#[inline]
pub fn count_128(registers: &[u32; 26]) -> (usize, f32) {
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
	let word_13 = registers[13];
	let word_14 = registers[14];
	let word_15 = registers[15];
	let word_16 = registers[16];
	let word_17 = registers[17];
	let word_18 = registers[18];
	let word_19 = registers[19];
	let word_20 = registers[20];
	let word_21 = registers[21];
	let word_22 = registers[22];
	let word_23 = registers[23];
	let word_24 = registers[24];
	let word_25 = registers[25];

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
	let register_64 = (word_12 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_65 = word_13 & LOWER_REGISTER_MASK;
	let register_66 = (word_13 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_67 = (word_13 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_68 = (word_13 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_69 = (word_13 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_70 = word_14 & LOWER_REGISTER_MASK;
	let register_71 = (word_14 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_72 = (word_14 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_73 = (word_14 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_74 = (word_14 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_75 = word_15 & LOWER_REGISTER_MASK;
	let register_76 = (word_15 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_77 = (word_15 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_78 = (word_15 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_79 = (word_15 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_80 = word_16 & LOWER_REGISTER_MASK;
	let register_81 = (word_16 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_82 = (word_16 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_83 = (word_16 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_84 = (word_16 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_85 = word_17 & LOWER_REGISTER_MASK;
	let register_86 = (word_17 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_87 = (word_17 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_88 = (word_17 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_89 = (word_17 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_90 = word_18 & LOWER_REGISTER_MASK;
	let register_91 = (word_18 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_92 = (word_18 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_93 = (word_18 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_94 = (word_18 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_95 = word_19 & LOWER_REGISTER_MASK;
	let register_96 = (word_19 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_97 = (word_19 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_98 = (word_19 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_99 = (word_19 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_100 = word_20 & LOWER_REGISTER_MASK;
	let register_101 = (word_20 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_102 = (word_20 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_103 = (word_20 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_104 = (word_20 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_105 = word_21 & LOWER_REGISTER_MASK;
	let register_106 = (word_21 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_107 = (word_21 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_108 = (word_21 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_109 = (word_21 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_110 = word_22 & LOWER_REGISTER_MASK;
	let register_111 = (word_22 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_112 = (word_22 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_113 = (word_22 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_114 = (word_22 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_115 = word_23 & LOWER_REGISTER_MASK;
	let register_116 = (word_23 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_117 = (word_23 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_118 = (word_23 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_119 = (word_23 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_120 = word_24 & LOWER_REGISTER_MASK;
	let register_121 = (word_24 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_122 = (word_24 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_123 = (word_24 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_124 = (word_24 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_125 = word_25 & LOWER_REGISTER_MASK;
	let register_126 = (word_25 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_127 = (word_25 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;

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
		(register_63 == 0) as usize +
		(register_64 == 0) as usize +
		(register_65 == 0) as usize +
		(register_66 == 0) as usize +
		(register_67 == 0) as usize +
		(register_68 == 0) as usize +
		(register_69 == 0) as usize +
		(register_70 == 0) as usize +
		(register_71 == 0) as usize +
		(register_72 == 0) as usize +
		(register_73 == 0) as usize +
		(register_74 == 0) as usize +
		(register_75 == 0) as usize +
		(register_76 == 0) as usize +
		(register_77 == 0) as usize +
		(register_78 == 0) as usize +
		(register_79 == 0) as usize +
		(register_80 == 0) as usize +
		(register_81 == 0) as usize +
		(register_82 == 0) as usize +
		(register_83 == 0) as usize +
		(register_84 == 0) as usize +
		(register_85 == 0) as usize +
		(register_86 == 0) as usize +
		(register_87 == 0) as usize +
		(register_88 == 0) as usize +
		(register_89 == 0) as usize +
		(register_90 == 0) as usize +
		(register_91 == 0) as usize +
		(register_92 == 0) as usize +
		(register_93 == 0) as usize +
		(register_94 == 0) as usize +
		(register_95 == 0) as usize +
		(register_96 == 0) as usize +
		(register_97 == 0) as usize +
		(register_98 == 0) as usize +
		(register_99 == 0) as usize +
		(register_100 == 0) as usize +
		(register_101 == 0) as usize +
		(register_102 == 0) as usize +
		(register_103 == 0) as usize +
		(register_104 == 0) as usize +
		(register_105 == 0) as usize +
		(register_106 == 0) as usize +
		(register_107 == 0) as usize +
		(register_108 == 0) as usize +
		(register_109 == 0) as usize +
		(register_110 == 0) as usize +
		(register_111 == 0) as usize +
		(register_112 == 0) as usize +
		(register_113 == 0) as usize +
		(register_114 == 0) as usize +
		(register_115 == 0) as usize +
		(register_116 == 0) as usize +
		(register_117 == 0) as usize +
		(register_118 == 0) as usize +
		(register_119 == 0) as usize +
		(register_120 == 0) as usize +
		(register_121 == 0) as usize +
		(register_122 == 0) as usize +
		(register_123 == 0) as usize +
		(register_124 == 0) as usize +
		(register_125 == 0) as usize +
		(register_126 == 0) as usize +
		(register_127 == 0) as usize,
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
		1.0_f32 / (1u64 << register_63) as f32 +
		1.0_f32 / (1u64 << register_64) as f32 +
		1.0_f32 / (1u64 << register_65) as f32 +
		1.0_f32 / (1u64 << register_66) as f32 +
		1.0_f32 / (1u64 << register_67) as f32 +
		1.0_f32 / (1u64 << register_68) as f32 +
		1.0_f32 / (1u64 << register_69) as f32 +
		1.0_f32 / (1u64 << register_70) as f32 +
		1.0_f32 / (1u64 << register_71) as f32 +
		1.0_f32 / (1u64 << register_72) as f32 +
		1.0_f32 / (1u64 << register_73) as f32 +
		1.0_f32 / (1u64 << register_74) as f32 +
		1.0_f32 / (1u64 << register_75) as f32 +
		1.0_f32 / (1u64 << register_76) as f32 +
		1.0_f32 / (1u64 << register_77) as f32 +
		1.0_f32 / (1u64 << register_78) as f32 +
		1.0_f32 / (1u64 << register_79) as f32 +
		1.0_f32 / (1u64 << register_80) as f32 +
		1.0_f32 / (1u64 << register_81) as f32 +
		1.0_f32 / (1u64 << register_82) as f32 +
		1.0_f32 / (1u64 << register_83) as f32 +
		1.0_f32 / (1u64 << register_84) as f32 +
		1.0_f32 / (1u64 << register_85) as f32 +
		1.0_f32 / (1u64 << register_86) as f32 +
		1.0_f32 / (1u64 << register_87) as f32 +
		1.0_f32 / (1u64 << register_88) as f32 +
		1.0_f32 / (1u64 << register_89) as f32 +
		1.0_f32 / (1u64 << register_90) as f32 +
		1.0_f32 / (1u64 << register_91) as f32 +
		1.0_f32 / (1u64 << register_92) as f32 +
		1.0_f32 / (1u64 << register_93) as f32 +
		1.0_f32 / (1u64 << register_94) as f32 +
		1.0_f32 / (1u64 << register_95) as f32 +
		1.0_f32 / (1u64 << register_96) as f32 +
		1.0_f32 / (1u64 << register_97) as f32 +
		1.0_f32 / (1u64 << register_98) as f32 +
		1.0_f32 / (1u64 << register_99) as f32 +
		1.0_f32 / (1u64 << register_100) as f32 +
		1.0_f32 / (1u64 << register_101) as f32 +
		1.0_f32 / (1u64 << register_102) as f32 +
		1.0_f32 / (1u64 << register_103) as f32 +
		1.0_f32 / (1u64 << register_104) as f32 +
		1.0_f32 / (1u64 << register_105) as f32 +
		1.0_f32 / (1u64 << register_106) as f32 +
		1.0_f32 / (1u64 << register_107) as f32 +
		1.0_f32 / (1u64 << register_108) as f32 +
		1.0_f32 / (1u64 << register_109) as f32 +
		1.0_f32 / (1u64 << register_110) as f32 +
		1.0_f32 / (1u64 << register_111) as f32 +
		1.0_f32 / (1u64 << register_112) as f32 +
		1.0_f32 / (1u64 << register_113) as f32 +
		1.0_f32 / (1u64 << register_114) as f32 +
		1.0_f32 / (1u64 << register_115) as f32 +
		1.0_f32 / (1u64 << register_116) as f32 +
		1.0_f32 / (1u64 << register_117) as f32 +
		1.0_f32 / (1u64 << register_118) as f32 +
		1.0_f32 / (1u64 << register_119) as f32 +
		1.0_f32 / (1u64 << register_120) as f32 +
		1.0_f32 / (1u64 << register_121) as f32 +
		1.0_f32 / (1u64 << register_122) as f32 +
		1.0_f32 / (1u64 << register_123) as f32 +
		1.0_f32 / (1u64 << register_124) as f32 +
		1.0_f32 / (1u64 << register_125) as f32 +
		1.0_f32 / (1u64 << register_126) as f32 +
		1.0_f32 / (1u64 << register_127) as f32
    )
}
