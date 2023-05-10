
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_256_registers_and_8_bits(words: &[u32; 64]) -> f32 {
    let mut raw_estimate = 0.0;

	let [register_0, register_1, register_2, register_3] = split_registers::<4>(words[0]);
	raw_estimate += 1.0 / (1u64 << register_0) as f32 + 1.0 / (1u64 << register_1) as f32 + 1.0 / (1u64 << register_2) as f32 + 1.0 / (1u64 << register_3) as f32;

	let [register_4, register_5, register_6, register_7] = split_registers::<4>(words[1]);
	raw_estimate += 1.0 / (1u64 << register_4) as f32 + 1.0 / (1u64 << register_5) as f32 + 1.0 / (1u64 << register_6) as f32 + 1.0 / (1u64 << register_7) as f32;

	let [register_8, register_9, register_10, register_11] = split_registers::<4>(words[2]);
	raw_estimate += 1.0 / (1u64 << register_8) as f32 + 1.0 / (1u64 << register_9) as f32 + 1.0 / (1u64 << register_10) as f32 + 1.0 / (1u64 << register_11) as f32;

	let [register_12, register_13, register_14, register_15] = split_registers::<4>(words[3]);
	raw_estimate += 1.0 / (1u64 << register_12) as f32 + 1.0 / (1u64 << register_13) as f32 + 1.0 / (1u64 << register_14) as f32 + 1.0 / (1u64 << register_15) as f32;

	let [register_16, register_17, register_18, register_19] = split_registers::<4>(words[4]);
	raw_estimate += 1.0 / (1u64 << register_16) as f32 + 1.0 / (1u64 << register_17) as f32 + 1.0 / (1u64 << register_18) as f32 + 1.0 / (1u64 << register_19) as f32;

	let [register_20, register_21, register_22, register_23] = split_registers::<4>(words[5]);
	raw_estimate += 1.0 / (1u64 << register_20) as f32 + 1.0 / (1u64 << register_21) as f32 + 1.0 / (1u64 << register_22) as f32 + 1.0 / (1u64 << register_23) as f32;

	let [register_24, register_25, register_26, register_27] = split_registers::<4>(words[6]);
	raw_estimate += 1.0 / (1u64 << register_24) as f32 + 1.0 / (1u64 << register_25) as f32 + 1.0 / (1u64 << register_26) as f32 + 1.0 / (1u64 << register_27) as f32;

	let [register_28, register_29, register_30, register_31] = split_registers::<4>(words[7]);
	raw_estimate += 1.0 / (1u64 << register_28) as f32 + 1.0 / (1u64 << register_29) as f32 + 1.0 / (1u64 << register_30) as f32 + 1.0 / (1u64 << register_31) as f32;

	let [register_32, register_33, register_34, register_35] = split_registers::<4>(words[8]);
	raw_estimate += 1.0 / (1u64 << register_32) as f32 + 1.0 / (1u64 << register_33) as f32 + 1.0 / (1u64 << register_34) as f32 + 1.0 / (1u64 << register_35) as f32;

	let [register_36, register_37, register_38, register_39] = split_registers::<4>(words[9]);
	raw_estimate += 1.0 / (1u64 << register_36) as f32 + 1.0 / (1u64 << register_37) as f32 + 1.0 / (1u64 << register_38) as f32 + 1.0 / (1u64 << register_39) as f32;

	let [register_40, register_41, register_42, register_43] = split_registers::<4>(words[10]);
	raw_estimate += 1.0 / (1u64 << register_40) as f32 + 1.0 / (1u64 << register_41) as f32 + 1.0 / (1u64 << register_42) as f32 + 1.0 / (1u64 << register_43) as f32;

	let [register_44, register_45, register_46, register_47] = split_registers::<4>(words[11]);
	raw_estimate += 1.0 / (1u64 << register_44) as f32 + 1.0 / (1u64 << register_45) as f32 + 1.0 / (1u64 << register_46) as f32 + 1.0 / (1u64 << register_47) as f32;

	let [register_48, register_49, register_50, register_51] = split_registers::<4>(words[12]);
	raw_estimate += 1.0 / (1u64 << register_48) as f32 + 1.0 / (1u64 << register_49) as f32 + 1.0 / (1u64 << register_50) as f32 + 1.0 / (1u64 << register_51) as f32;

	let [register_52, register_53, register_54, register_55] = split_registers::<4>(words[13]);
	raw_estimate += 1.0 / (1u64 << register_52) as f32 + 1.0 / (1u64 << register_53) as f32 + 1.0 / (1u64 << register_54) as f32 + 1.0 / (1u64 << register_55) as f32;

	let [register_56, register_57, register_58, register_59] = split_registers::<4>(words[14]);
	raw_estimate += 1.0 / (1u64 << register_56) as f32 + 1.0 / (1u64 << register_57) as f32 + 1.0 / (1u64 << register_58) as f32 + 1.0 / (1u64 << register_59) as f32;

	let [register_60, register_61, register_62, register_63] = split_registers::<4>(words[15]);
	raw_estimate += 1.0 / (1u64 << register_60) as f32 + 1.0 / (1u64 << register_61) as f32 + 1.0 / (1u64 << register_62) as f32 + 1.0 / (1u64 << register_63) as f32;

	let [register_64, register_65, register_66, register_67] = split_registers::<4>(words[16]);
	raw_estimate += 1.0 / (1u64 << register_64) as f32 + 1.0 / (1u64 << register_65) as f32 + 1.0 / (1u64 << register_66) as f32 + 1.0 / (1u64 << register_67) as f32;

	let [register_68, register_69, register_70, register_71] = split_registers::<4>(words[17]);
	raw_estimate += 1.0 / (1u64 << register_68) as f32 + 1.0 / (1u64 << register_69) as f32 + 1.0 / (1u64 << register_70) as f32 + 1.0 / (1u64 << register_71) as f32;

	let [register_72, register_73, register_74, register_75] = split_registers::<4>(words[18]);
	raw_estimate += 1.0 / (1u64 << register_72) as f32 + 1.0 / (1u64 << register_73) as f32 + 1.0 / (1u64 << register_74) as f32 + 1.0 / (1u64 << register_75) as f32;

	let [register_76, register_77, register_78, register_79] = split_registers::<4>(words[19]);
	raw_estimate += 1.0 / (1u64 << register_76) as f32 + 1.0 / (1u64 << register_77) as f32 + 1.0 / (1u64 << register_78) as f32 + 1.0 / (1u64 << register_79) as f32;

	let [register_80, register_81, register_82, register_83] = split_registers::<4>(words[20]);
	raw_estimate += 1.0 / (1u64 << register_80) as f32 + 1.0 / (1u64 << register_81) as f32 + 1.0 / (1u64 << register_82) as f32 + 1.0 / (1u64 << register_83) as f32;

	let [register_84, register_85, register_86, register_87] = split_registers::<4>(words[21]);
	raw_estimate += 1.0 / (1u64 << register_84) as f32 + 1.0 / (1u64 << register_85) as f32 + 1.0 / (1u64 << register_86) as f32 + 1.0 / (1u64 << register_87) as f32;

	let [register_88, register_89, register_90, register_91] = split_registers::<4>(words[22]);
	raw_estimate += 1.0 / (1u64 << register_88) as f32 + 1.0 / (1u64 << register_89) as f32 + 1.0 / (1u64 << register_90) as f32 + 1.0 / (1u64 << register_91) as f32;

	let [register_92, register_93, register_94, register_95] = split_registers::<4>(words[23]);
	raw_estimate += 1.0 / (1u64 << register_92) as f32 + 1.0 / (1u64 << register_93) as f32 + 1.0 / (1u64 << register_94) as f32 + 1.0 / (1u64 << register_95) as f32;

	let [register_96, register_97, register_98, register_99] = split_registers::<4>(words[24]);
	raw_estimate += 1.0 / (1u64 << register_96) as f32 + 1.0 / (1u64 << register_97) as f32 + 1.0 / (1u64 << register_98) as f32 + 1.0 / (1u64 << register_99) as f32;

	let [register_100, register_101, register_102, register_103] = split_registers::<4>(words[25]);
	raw_estimate += 1.0 / (1u64 << register_100) as f32 + 1.0 / (1u64 << register_101) as f32 + 1.0 / (1u64 << register_102) as f32 + 1.0 / (1u64 << register_103) as f32;

	let [register_104, register_105, register_106, register_107] = split_registers::<4>(words[26]);
	raw_estimate += 1.0 / (1u64 << register_104) as f32 + 1.0 / (1u64 << register_105) as f32 + 1.0 / (1u64 << register_106) as f32 + 1.0 / (1u64 << register_107) as f32;

	let [register_108, register_109, register_110, register_111] = split_registers::<4>(words[27]);
	raw_estimate += 1.0 / (1u64 << register_108) as f32 + 1.0 / (1u64 << register_109) as f32 + 1.0 / (1u64 << register_110) as f32 + 1.0 / (1u64 << register_111) as f32;

	let [register_112, register_113, register_114, register_115] = split_registers::<4>(words[28]);
	raw_estimate += 1.0 / (1u64 << register_112) as f32 + 1.0 / (1u64 << register_113) as f32 + 1.0 / (1u64 << register_114) as f32 + 1.0 / (1u64 << register_115) as f32;

	let [register_116, register_117, register_118, register_119] = split_registers::<4>(words[29]);
	raw_estimate += 1.0 / (1u64 << register_116) as f32 + 1.0 / (1u64 << register_117) as f32 + 1.0 / (1u64 << register_118) as f32 + 1.0 / (1u64 << register_119) as f32;

	let [register_120, register_121, register_122, register_123] = split_registers::<4>(words[30]);
	raw_estimate += 1.0 / (1u64 << register_120) as f32 + 1.0 / (1u64 << register_121) as f32 + 1.0 / (1u64 << register_122) as f32 + 1.0 / (1u64 << register_123) as f32;

	let [register_124, register_125, register_126, register_127] = split_registers::<4>(words[31]);
	raw_estimate += 1.0 / (1u64 << register_124) as f32 + 1.0 / (1u64 << register_125) as f32 + 1.0 / (1u64 << register_126) as f32 + 1.0 / (1u64 << register_127) as f32;

	let [register_128, register_129, register_130, register_131] = split_registers::<4>(words[32]);
	raw_estimate += 1.0 / (1u64 << register_128) as f32 + 1.0 / (1u64 << register_129) as f32 + 1.0 / (1u64 << register_130) as f32 + 1.0 / (1u64 << register_131) as f32;

	let [register_132, register_133, register_134, register_135] = split_registers::<4>(words[33]);
	raw_estimate += 1.0 / (1u64 << register_132) as f32 + 1.0 / (1u64 << register_133) as f32 + 1.0 / (1u64 << register_134) as f32 + 1.0 / (1u64 << register_135) as f32;

	let [register_136, register_137, register_138, register_139] = split_registers::<4>(words[34]);
	raw_estimate += 1.0 / (1u64 << register_136) as f32 + 1.0 / (1u64 << register_137) as f32 + 1.0 / (1u64 << register_138) as f32 + 1.0 / (1u64 << register_139) as f32;

	let [register_140, register_141, register_142, register_143] = split_registers::<4>(words[35]);
	raw_estimate += 1.0 / (1u64 << register_140) as f32 + 1.0 / (1u64 << register_141) as f32 + 1.0 / (1u64 << register_142) as f32 + 1.0 / (1u64 << register_143) as f32;

	let [register_144, register_145, register_146, register_147] = split_registers::<4>(words[36]);
	raw_estimate += 1.0 / (1u64 << register_144) as f32 + 1.0 / (1u64 << register_145) as f32 + 1.0 / (1u64 << register_146) as f32 + 1.0 / (1u64 << register_147) as f32;

	let [register_148, register_149, register_150, register_151] = split_registers::<4>(words[37]);
	raw_estimate += 1.0 / (1u64 << register_148) as f32 + 1.0 / (1u64 << register_149) as f32 + 1.0 / (1u64 << register_150) as f32 + 1.0 / (1u64 << register_151) as f32;

	let [register_152, register_153, register_154, register_155] = split_registers::<4>(words[38]);
	raw_estimate += 1.0 / (1u64 << register_152) as f32 + 1.0 / (1u64 << register_153) as f32 + 1.0 / (1u64 << register_154) as f32 + 1.0 / (1u64 << register_155) as f32;

	let [register_156, register_157, register_158, register_159] = split_registers::<4>(words[39]);
	raw_estimate += 1.0 / (1u64 << register_156) as f32 + 1.0 / (1u64 << register_157) as f32 + 1.0 / (1u64 << register_158) as f32 + 1.0 / (1u64 << register_159) as f32;

	let [register_160, register_161, register_162, register_163] = split_registers::<4>(words[40]);
	raw_estimate += 1.0 / (1u64 << register_160) as f32 + 1.0 / (1u64 << register_161) as f32 + 1.0 / (1u64 << register_162) as f32 + 1.0 / (1u64 << register_163) as f32;

	let [register_164, register_165, register_166, register_167] = split_registers::<4>(words[41]);
	raw_estimate += 1.0 / (1u64 << register_164) as f32 + 1.0 / (1u64 << register_165) as f32 + 1.0 / (1u64 << register_166) as f32 + 1.0 / (1u64 << register_167) as f32;

	let [register_168, register_169, register_170, register_171] = split_registers::<4>(words[42]);
	raw_estimate += 1.0 / (1u64 << register_168) as f32 + 1.0 / (1u64 << register_169) as f32 + 1.0 / (1u64 << register_170) as f32 + 1.0 / (1u64 << register_171) as f32;

	let [register_172, register_173, register_174, register_175] = split_registers::<4>(words[43]);
	raw_estimate += 1.0 / (1u64 << register_172) as f32 + 1.0 / (1u64 << register_173) as f32 + 1.0 / (1u64 << register_174) as f32 + 1.0 / (1u64 << register_175) as f32;

	let [register_176, register_177, register_178, register_179] = split_registers::<4>(words[44]);
	raw_estimate += 1.0 / (1u64 << register_176) as f32 + 1.0 / (1u64 << register_177) as f32 + 1.0 / (1u64 << register_178) as f32 + 1.0 / (1u64 << register_179) as f32;

	let [register_180, register_181, register_182, register_183] = split_registers::<4>(words[45]);
	raw_estimate += 1.0 / (1u64 << register_180) as f32 + 1.0 / (1u64 << register_181) as f32 + 1.0 / (1u64 << register_182) as f32 + 1.0 / (1u64 << register_183) as f32;

	let [register_184, register_185, register_186, register_187] = split_registers::<4>(words[46]);
	raw_estimate += 1.0 / (1u64 << register_184) as f32 + 1.0 / (1u64 << register_185) as f32 + 1.0 / (1u64 << register_186) as f32 + 1.0 / (1u64 << register_187) as f32;

	let [register_188, register_189, register_190, register_191] = split_registers::<4>(words[47]);
	raw_estimate += 1.0 / (1u64 << register_188) as f32 + 1.0 / (1u64 << register_189) as f32 + 1.0 / (1u64 << register_190) as f32 + 1.0 / (1u64 << register_191) as f32;

	let [register_192, register_193, register_194, register_195] = split_registers::<4>(words[48]);
	raw_estimate += 1.0 / (1u64 << register_192) as f32 + 1.0 / (1u64 << register_193) as f32 + 1.0 / (1u64 << register_194) as f32 + 1.0 / (1u64 << register_195) as f32;

	let [register_196, register_197, register_198, register_199] = split_registers::<4>(words[49]);
	raw_estimate += 1.0 / (1u64 << register_196) as f32 + 1.0 / (1u64 << register_197) as f32 + 1.0 / (1u64 << register_198) as f32 + 1.0 / (1u64 << register_199) as f32;

	let [register_200, register_201, register_202, register_203] = split_registers::<4>(words[50]);
	raw_estimate += 1.0 / (1u64 << register_200) as f32 + 1.0 / (1u64 << register_201) as f32 + 1.0 / (1u64 << register_202) as f32 + 1.0 / (1u64 << register_203) as f32;

	let [register_204, register_205, register_206, register_207] = split_registers::<4>(words[51]);
	raw_estimate += 1.0 / (1u64 << register_204) as f32 + 1.0 / (1u64 << register_205) as f32 + 1.0 / (1u64 << register_206) as f32 + 1.0 / (1u64 << register_207) as f32;

	let [register_208, register_209, register_210, register_211] = split_registers::<4>(words[52]);
	raw_estimate += 1.0 / (1u64 << register_208) as f32 + 1.0 / (1u64 << register_209) as f32 + 1.0 / (1u64 << register_210) as f32 + 1.0 / (1u64 << register_211) as f32;

	let [register_212, register_213, register_214, register_215] = split_registers::<4>(words[53]);
	raw_estimate += 1.0 / (1u64 << register_212) as f32 + 1.0 / (1u64 << register_213) as f32 + 1.0 / (1u64 << register_214) as f32 + 1.0 / (1u64 << register_215) as f32;

	let [register_216, register_217, register_218, register_219] = split_registers::<4>(words[54]);
	raw_estimate += 1.0 / (1u64 << register_216) as f32 + 1.0 / (1u64 << register_217) as f32 + 1.0 / (1u64 << register_218) as f32 + 1.0 / (1u64 << register_219) as f32;

	let [register_220, register_221, register_222, register_223] = split_registers::<4>(words[55]);
	raw_estimate += 1.0 / (1u64 << register_220) as f32 + 1.0 / (1u64 << register_221) as f32 + 1.0 / (1u64 << register_222) as f32 + 1.0 / (1u64 << register_223) as f32;

	let [register_224, register_225, register_226, register_227] = split_registers::<4>(words[56]);
	raw_estimate += 1.0 / (1u64 << register_224) as f32 + 1.0 / (1u64 << register_225) as f32 + 1.0 / (1u64 << register_226) as f32 + 1.0 / (1u64 << register_227) as f32;

	let [register_228, register_229, register_230, register_231] = split_registers::<4>(words[57]);
	raw_estimate += 1.0 / (1u64 << register_228) as f32 + 1.0 / (1u64 << register_229) as f32 + 1.0 / (1u64 << register_230) as f32 + 1.0 / (1u64 << register_231) as f32;

	let [register_232, register_233, register_234, register_235] = split_registers::<4>(words[58]);
	raw_estimate += 1.0 / (1u64 << register_232) as f32 + 1.0 / (1u64 << register_233) as f32 + 1.0 / (1u64 << register_234) as f32 + 1.0 / (1u64 << register_235) as f32;

	let [register_236, register_237, register_238, register_239] = split_registers::<4>(words[59]);
	raw_estimate += 1.0 / (1u64 << register_236) as f32 + 1.0 / (1u64 << register_237) as f32 + 1.0 / (1u64 << register_238) as f32 + 1.0 / (1u64 << register_239) as f32;

	let [register_240, register_241, register_242, register_243] = split_registers::<4>(words[60]);
	raw_estimate += 1.0 / (1u64 << register_240) as f32 + 1.0 / (1u64 << register_241) as f32 + 1.0 / (1u64 << register_242) as f32 + 1.0 / (1u64 << register_243) as f32;

	let [register_244, register_245, register_246, register_247] = split_registers::<4>(words[61]);
	raw_estimate += 1.0 / (1u64 << register_244) as f32 + 1.0 / (1u64 << register_245) as f32 + 1.0 / (1u64 << register_246) as f32 + 1.0 / (1u64 << register_247) as f32;

	let [register_248, register_249, register_250, register_251] = split_registers::<4>(words[62]);
	raw_estimate += 1.0 / (1u64 << register_248) as f32 + 1.0 / (1u64 << register_249) as f32 + 1.0 / (1u64 << register_250) as f32 + 1.0 / (1u64 << register_251) as f32;

	let [register_252, register_253, register_254, register_255] = split_registers::<4>(words[63]);
	raw_estimate += 1.0 / (1u64 << register_252) as f32 + 1.0 / (1u64 << register_253) as f32 + 1.0 / (1u64 << register_254) as f32 + 1.0 / (1u64 << register_255) as f32;


    raw_estimate
}
