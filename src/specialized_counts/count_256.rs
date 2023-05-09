
use crate::prelude::*;

#[inline]
pub fn count_256(registers: &[u32; 52]) -> (usize, f32) {
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
	let word_26 = registers[26];
	let word_27 = registers[27];
	let word_28 = registers[28];
	let word_29 = registers[29];
	let word_30 = registers[30];
	let word_31 = registers[31];
	let word_32 = registers[32];
	let word_33 = registers[33];
	let word_34 = registers[34];
	let word_35 = registers[35];
	let word_36 = registers[36];
	let word_37 = registers[37];
	let word_38 = registers[38];
	let word_39 = registers[39];
	let word_40 = registers[40];
	let word_41 = registers[41];
	let word_42 = registers[42];
	let word_43 = registers[43];
	let word_44 = registers[44];
	let word_45 = registers[45];
	let word_46 = registers[46];
	let word_47 = registers[47];
	let word_48 = registers[48];
	let word_49 = registers[49];
	let word_50 = registers[50];
	let word_51 = registers[51];

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
	let register_128 = (word_25 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_129 = (word_25 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_130 = word_26 & LOWER_REGISTER_MASK;
	let register_131 = (word_26 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_132 = (word_26 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_133 = (word_26 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_134 = (word_26 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_135 = word_27 & LOWER_REGISTER_MASK;
	let register_136 = (word_27 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_137 = (word_27 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_138 = (word_27 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_139 = (word_27 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_140 = word_28 & LOWER_REGISTER_MASK;
	let register_141 = (word_28 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_142 = (word_28 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_143 = (word_28 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_144 = (word_28 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_145 = word_29 & LOWER_REGISTER_MASK;
	let register_146 = (word_29 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_147 = (word_29 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_148 = (word_29 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_149 = (word_29 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_150 = word_30 & LOWER_REGISTER_MASK;
	let register_151 = (word_30 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_152 = (word_30 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_153 = (word_30 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_154 = (word_30 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_155 = word_31 & LOWER_REGISTER_MASK;
	let register_156 = (word_31 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_157 = (word_31 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_158 = (word_31 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_159 = (word_31 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_160 = word_32 & LOWER_REGISTER_MASK;
	let register_161 = (word_32 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_162 = (word_32 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_163 = (word_32 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_164 = (word_32 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_165 = word_33 & LOWER_REGISTER_MASK;
	let register_166 = (word_33 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_167 = (word_33 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_168 = (word_33 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_169 = (word_33 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_170 = word_34 & LOWER_REGISTER_MASK;
	let register_171 = (word_34 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_172 = (word_34 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_173 = (word_34 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_174 = (word_34 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_175 = word_35 & LOWER_REGISTER_MASK;
	let register_176 = (word_35 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_177 = (word_35 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_178 = (word_35 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_179 = (word_35 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_180 = word_36 & LOWER_REGISTER_MASK;
	let register_181 = (word_36 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_182 = (word_36 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_183 = (word_36 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_184 = (word_36 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_185 = word_37 & LOWER_REGISTER_MASK;
	let register_186 = (word_37 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_187 = (word_37 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_188 = (word_37 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_189 = (word_37 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_190 = word_38 & LOWER_REGISTER_MASK;
	let register_191 = (word_38 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_192 = (word_38 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_193 = (word_38 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_194 = (word_38 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_195 = word_39 & LOWER_REGISTER_MASK;
	let register_196 = (word_39 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_197 = (word_39 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_198 = (word_39 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_199 = (word_39 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_200 = word_40 & LOWER_REGISTER_MASK;
	let register_201 = (word_40 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_202 = (word_40 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_203 = (word_40 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_204 = (word_40 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_205 = word_41 & LOWER_REGISTER_MASK;
	let register_206 = (word_41 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_207 = (word_41 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_208 = (word_41 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_209 = (word_41 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_210 = word_42 & LOWER_REGISTER_MASK;
	let register_211 = (word_42 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_212 = (word_42 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_213 = (word_42 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_214 = (word_42 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_215 = word_43 & LOWER_REGISTER_MASK;
	let register_216 = (word_43 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_217 = (word_43 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_218 = (word_43 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_219 = (word_43 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_220 = word_44 & LOWER_REGISTER_MASK;
	let register_221 = (word_44 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_222 = (word_44 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_223 = (word_44 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_224 = (word_44 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_225 = word_45 & LOWER_REGISTER_MASK;
	let register_226 = (word_45 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_227 = (word_45 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_228 = (word_45 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_229 = (word_45 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_230 = word_46 & LOWER_REGISTER_MASK;
	let register_231 = (word_46 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_232 = (word_46 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_233 = (word_46 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_234 = (word_46 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_235 = word_47 & LOWER_REGISTER_MASK;
	let register_236 = (word_47 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_237 = (word_47 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_238 = (word_47 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_239 = (word_47 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_240 = word_48 & LOWER_REGISTER_MASK;
	let register_241 = (word_48 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_242 = (word_48 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_243 = (word_48 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_244 = (word_48 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_245 = word_49 & LOWER_REGISTER_MASK;
	let register_246 = (word_49 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_247 = (word_49 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_248 = (word_49 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_249 = (word_49 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_250 = word_50 & LOWER_REGISTER_MASK;
	let register_251 = (word_50 >> NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_252 = (word_50 >> 2 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_253 = (word_50 >> 3 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_254 = (word_50 >> 4 * NUMBER_OF_BITS_PER_REGISTER) & LOWER_REGISTER_MASK;
	let register_255 = word_51 & LOWER_REGISTER_MASK;

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
		(register_127 == 0) as usize +
		(register_128 == 0) as usize +
		(register_129 == 0) as usize +
		(register_130 == 0) as usize +
		(register_131 == 0) as usize +
		(register_132 == 0) as usize +
		(register_133 == 0) as usize +
		(register_134 == 0) as usize +
		(register_135 == 0) as usize +
		(register_136 == 0) as usize +
		(register_137 == 0) as usize +
		(register_138 == 0) as usize +
		(register_139 == 0) as usize +
		(register_140 == 0) as usize +
		(register_141 == 0) as usize +
		(register_142 == 0) as usize +
		(register_143 == 0) as usize +
		(register_144 == 0) as usize +
		(register_145 == 0) as usize +
		(register_146 == 0) as usize +
		(register_147 == 0) as usize +
		(register_148 == 0) as usize +
		(register_149 == 0) as usize +
		(register_150 == 0) as usize +
		(register_151 == 0) as usize +
		(register_152 == 0) as usize +
		(register_153 == 0) as usize +
		(register_154 == 0) as usize +
		(register_155 == 0) as usize +
		(register_156 == 0) as usize +
		(register_157 == 0) as usize +
		(register_158 == 0) as usize +
		(register_159 == 0) as usize +
		(register_160 == 0) as usize +
		(register_161 == 0) as usize +
		(register_162 == 0) as usize +
		(register_163 == 0) as usize +
		(register_164 == 0) as usize +
		(register_165 == 0) as usize +
		(register_166 == 0) as usize +
		(register_167 == 0) as usize +
		(register_168 == 0) as usize +
		(register_169 == 0) as usize +
		(register_170 == 0) as usize +
		(register_171 == 0) as usize +
		(register_172 == 0) as usize +
		(register_173 == 0) as usize +
		(register_174 == 0) as usize +
		(register_175 == 0) as usize +
		(register_176 == 0) as usize +
		(register_177 == 0) as usize +
		(register_178 == 0) as usize +
		(register_179 == 0) as usize +
		(register_180 == 0) as usize +
		(register_181 == 0) as usize +
		(register_182 == 0) as usize +
		(register_183 == 0) as usize +
		(register_184 == 0) as usize +
		(register_185 == 0) as usize +
		(register_186 == 0) as usize +
		(register_187 == 0) as usize +
		(register_188 == 0) as usize +
		(register_189 == 0) as usize +
		(register_190 == 0) as usize +
		(register_191 == 0) as usize +
		(register_192 == 0) as usize +
		(register_193 == 0) as usize +
		(register_194 == 0) as usize +
		(register_195 == 0) as usize +
		(register_196 == 0) as usize +
		(register_197 == 0) as usize +
		(register_198 == 0) as usize +
		(register_199 == 0) as usize +
		(register_200 == 0) as usize +
		(register_201 == 0) as usize +
		(register_202 == 0) as usize +
		(register_203 == 0) as usize +
		(register_204 == 0) as usize +
		(register_205 == 0) as usize +
		(register_206 == 0) as usize +
		(register_207 == 0) as usize +
		(register_208 == 0) as usize +
		(register_209 == 0) as usize +
		(register_210 == 0) as usize +
		(register_211 == 0) as usize +
		(register_212 == 0) as usize +
		(register_213 == 0) as usize +
		(register_214 == 0) as usize +
		(register_215 == 0) as usize +
		(register_216 == 0) as usize +
		(register_217 == 0) as usize +
		(register_218 == 0) as usize +
		(register_219 == 0) as usize +
		(register_220 == 0) as usize +
		(register_221 == 0) as usize +
		(register_222 == 0) as usize +
		(register_223 == 0) as usize +
		(register_224 == 0) as usize +
		(register_225 == 0) as usize +
		(register_226 == 0) as usize +
		(register_227 == 0) as usize +
		(register_228 == 0) as usize +
		(register_229 == 0) as usize +
		(register_230 == 0) as usize +
		(register_231 == 0) as usize +
		(register_232 == 0) as usize +
		(register_233 == 0) as usize +
		(register_234 == 0) as usize +
		(register_235 == 0) as usize +
		(register_236 == 0) as usize +
		(register_237 == 0) as usize +
		(register_238 == 0) as usize +
		(register_239 == 0) as usize +
		(register_240 == 0) as usize +
		(register_241 == 0) as usize +
		(register_242 == 0) as usize +
		(register_243 == 0) as usize +
		(register_244 == 0) as usize +
		(register_245 == 0) as usize +
		(register_246 == 0) as usize +
		(register_247 == 0) as usize +
		(register_248 == 0) as usize +
		(register_249 == 0) as usize +
		(register_250 == 0) as usize +
		(register_251 == 0) as usize +
		(register_252 == 0) as usize +
		(register_253 == 0) as usize +
		(register_254 == 0) as usize +
		(register_255 == 0) as usize,
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
		1.0_f32 / (1u64 << register_127) as f32 +
		1.0_f32 / (1u64 << register_128) as f32 +
		1.0_f32 / (1u64 << register_129) as f32 +
		1.0_f32 / (1u64 << register_130) as f32 +
		1.0_f32 / (1u64 << register_131) as f32 +
		1.0_f32 / (1u64 << register_132) as f32 +
		1.0_f32 / (1u64 << register_133) as f32 +
		1.0_f32 / (1u64 << register_134) as f32 +
		1.0_f32 / (1u64 << register_135) as f32 +
		1.0_f32 / (1u64 << register_136) as f32 +
		1.0_f32 / (1u64 << register_137) as f32 +
		1.0_f32 / (1u64 << register_138) as f32 +
		1.0_f32 / (1u64 << register_139) as f32 +
		1.0_f32 / (1u64 << register_140) as f32 +
		1.0_f32 / (1u64 << register_141) as f32 +
		1.0_f32 / (1u64 << register_142) as f32 +
		1.0_f32 / (1u64 << register_143) as f32 +
		1.0_f32 / (1u64 << register_144) as f32 +
		1.0_f32 / (1u64 << register_145) as f32 +
		1.0_f32 / (1u64 << register_146) as f32 +
		1.0_f32 / (1u64 << register_147) as f32 +
		1.0_f32 / (1u64 << register_148) as f32 +
		1.0_f32 / (1u64 << register_149) as f32 +
		1.0_f32 / (1u64 << register_150) as f32 +
		1.0_f32 / (1u64 << register_151) as f32 +
		1.0_f32 / (1u64 << register_152) as f32 +
		1.0_f32 / (1u64 << register_153) as f32 +
		1.0_f32 / (1u64 << register_154) as f32 +
		1.0_f32 / (1u64 << register_155) as f32 +
		1.0_f32 / (1u64 << register_156) as f32 +
		1.0_f32 / (1u64 << register_157) as f32 +
		1.0_f32 / (1u64 << register_158) as f32 +
		1.0_f32 / (1u64 << register_159) as f32 +
		1.0_f32 / (1u64 << register_160) as f32 +
		1.0_f32 / (1u64 << register_161) as f32 +
		1.0_f32 / (1u64 << register_162) as f32 +
		1.0_f32 / (1u64 << register_163) as f32 +
		1.0_f32 / (1u64 << register_164) as f32 +
		1.0_f32 / (1u64 << register_165) as f32 +
		1.0_f32 / (1u64 << register_166) as f32 +
		1.0_f32 / (1u64 << register_167) as f32 +
		1.0_f32 / (1u64 << register_168) as f32 +
		1.0_f32 / (1u64 << register_169) as f32 +
		1.0_f32 / (1u64 << register_170) as f32 +
		1.0_f32 / (1u64 << register_171) as f32 +
		1.0_f32 / (1u64 << register_172) as f32 +
		1.0_f32 / (1u64 << register_173) as f32 +
		1.0_f32 / (1u64 << register_174) as f32 +
		1.0_f32 / (1u64 << register_175) as f32 +
		1.0_f32 / (1u64 << register_176) as f32 +
		1.0_f32 / (1u64 << register_177) as f32 +
		1.0_f32 / (1u64 << register_178) as f32 +
		1.0_f32 / (1u64 << register_179) as f32 +
		1.0_f32 / (1u64 << register_180) as f32 +
		1.0_f32 / (1u64 << register_181) as f32 +
		1.0_f32 / (1u64 << register_182) as f32 +
		1.0_f32 / (1u64 << register_183) as f32 +
		1.0_f32 / (1u64 << register_184) as f32 +
		1.0_f32 / (1u64 << register_185) as f32 +
		1.0_f32 / (1u64 << register_186) as f32 +
		1.0_f32 / (1u64 << register_187) as f32 +
		1.0_f32 / (1u64 << register_188) as f32 +
		1.0_f32 / (1u64 << register_189) as f32 +
		1.0_f32 / (1u64 << register_190) as f32 +
		1.0_f32 / (1u64 << register_191) as f32 +
		1.0_f32 / (1u64 << register_192) as f32 +
		1.0_f32 / (1u64 << register_193) as f32 +
		1.0_f32 / (1u64 << register_194) as f32 +
		1.0_f32 / (1u64 << register_195) as f32 +
		1.0_f32 / (1u64 << register_196) as f32 +
		1.0_f32 / (1u64 << register_197) as f32 +
		1.0_f32 / (1u64 << register_198) as f32 +
		1.0_f32 / (1u64 << register_199) as f32 +
		1.0_f32 / (1u64 << register_200) as f32 +
		1.0_f32 / (1u64 << register_201) as f32 +
		1.0_f32 / (1u64 << register_202) as f32 +
		1.0_f32 / (1u64 << register_203) as f32 +
		1.0_f32 / (1u64 << register_204) as f32 +
		1.0_f32 / (1u64 << register_205) as f32 +
		1.0_f32 / (1u64 << register_206) as f32 +
		1.0_f32 / (1u64 << register_207) as f32 +
		1.0_f32 / (1u64 << register_208) as f32 +
		1.0_f32 / (1u64 << register_209) as f32 +
		1.0_f32 / (1u64 << register_210) as f32 +
		1.0_f32 / (1u64 << register_211) as f32 +
		1.0_f32 / (1u64 << register_212) as f32 +
		1.0_f32 / (1u64 << register_213) as f32 +
		1.0_f32 / (1u64 << register_214) as f32 +
		1.0_f32 / (1u64 << register_215) as f32 +
		1.0_f32 / (1u64 << register_216) as f32 +
		1.0_f32 / (1u64 << register_217) as f32 +
		1.0_f32 / (1u64 << register_218) as f32 +
		1.0_f32 / (1u64 << register_219) as f32 +
		1.0_f32 / (1u64 << register_220) as f32 +
		1.0_f32 / (1u64 << register_221) as f32 +
		1.0_f32 / (1u64 << register_222) as f32 +
		1.0_f32 / (1u64 << register_223) as f32 +
		1.0_f32 / (1u64 << register_224) as f32 +
		1.0_f32 / (1u64 << register_225) as f32 +
		1.0_f32 / (1u64 << register_226) as f32 +
		1.0_f32 / (1u64 << register_227) as f32 +
		1.0_f32 / (1u64 << register_228) as f32 +
		1.0_f32 / (1u64 << register_229) as f32 +
		1.0_f32 / (1u64 << register_230) as f32 +
		1.0_f32 / (1u64 << register_231) as f32 +
		1.0_f32 / (1u64 << register_232) as f32 +
		1.0_f32 / (1u64 << register_233) as f32 +
		1.0_f32 / (1u64 << register_234) as f32 +
		1.0_f32 / (1u64 << register_235) as f32 +
		1.0_f32 / (1u64 << register_236) as f32 +
		1.0_f32 / (1u64 << register_237) as f32 +
		1.0_f32 / (1u64 << register_238) as f32 +
		1.0_f32 / (1u64 << register_239) as f32 +
		1.0_f32 / (1u64 << register_240) as f32 +
		1.0_f32 / (1u64 << register_241) as f32 +
		1.0_f32 / (1u64 << register_242) as f32 +
		1.0_f32 / (1u64 << register_243) as f32 +
		1.0_f32 / (1u64 << register_244) as f32 +
		1.0_f32 / (1u64 << register_245) as f32 +
		1.0_f32 / (1u64 << register_246) as f32 +
		1.0_f32 / (1u64 << register_247) as f32 +
		1.0_f32 / (1u64 << register_248) as f32 +
		1.0_f32 / (1u64 << register_249) as f32 +
		1.0_f32 / (1u64 << register_250) as f32 +
		1.0_f32 / (1u64 << register_251) as f32 +
		1.0_f32 / (1u64 << register_252) as f32 +
		1.0_f32 / (1u64 << register_253) as f32 +
		1.0_f32 / (1u64 << register_254) as f32 +
		1.0_f32 / (1u64 << register_255) as f32
    )
}
