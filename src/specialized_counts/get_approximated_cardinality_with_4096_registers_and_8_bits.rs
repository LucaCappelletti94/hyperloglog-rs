
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_4096_registers_and_8_bits(words: &[u32; 1024]) -> f32 {
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

	let [register_256, register_257, register_258, register_259] = split_registers::<4>(words[64]);
	raw_estimate += 1.0 / (1u64 << register_256) as f32 + 1.0 / (1u64 << register_257) as f32 + 1.0 / (1u64 << register_258) as f32 + 1.0 / (1u64 << register_259) as f32;

	let [register_260, register_261, register_262, register_263] = split_registers::<4>(words[65]);
	raw_estimate += 1.0 / (1u64 << register_260) as f32 + 1.0 / (1u64 << register_261) as f32 + 1.0 / (1u64 << register_262) as f32 + 1.0 / (1u64 << register_263) as f32;

	let [register_264, register_265, register_266, register_267] = split_registers::<4>(words[66]);
	raw_estimate += 1.0 / (1u64 << register_264) as f32 + 1.0 / (1u64 << register_265) as f32 + 1.0 / (1u64 << register_266) as f32 + 1.0 / (1u64 << register_267) as f32;

	let [register_268, register_269, register_270, register_271] = split_registers::<4>(words[67]);
	raw_estimate += 1.0 / (1u64 << register_268) as f32 + 1.0 / (1u64 << register_269) as f32 + 1.0 / (1u64 << register_270) as f32 + 1.0 / (1u64 << register_271) as f32;

	let [register_272, register_273, register_274, register_275] = split_registers::<4>(words[68]);
	raw_estimate += 1.0 / (1u64 << register_272) as f32 + 1.0 / (1u64 << register_273) as f32 + 1.0 / (1u64 << register_274) as f32 + 1.0 / (1u64 << register_275) as f32;

	let [register_276, register_277, register_278, register_279] = split_registers::<4>(words[69]);
	raw_estimate += 1.0 / (1u64 << register_276) as f32 + 1.0 / (1u64 << register_277) as f32 + 1.0 / (1u64 << register_278) as f32 + 1.0 / (1u64 << register_279) as f32;

	let [register_280, register_281, register_282, register_283] = split_registers::<4>(words[70]);
	raw_estimate += 1.0 / (1u64 << register_280) as f32 + 1.0 / (1u64 << register_281) as f32 + 1.0 / (1u64 << register_282) as f32 + 1.0 / (1u64 << register_283) as f32;

	let [register_284, register_285, register_286, register_287] = split_registers::<4>(words[71]);
	raw_estimate += 1.0 / (1u64 << register_284) as f32 + 1.0 / (1u64 << register_285) as f32 + 1.0 / (1u64 << register_286) as f32 + 1.0 / (1u64 << register_287) as f32;

	let [register_288, register_289, register_290, register_291] = split_registers::<4>(words[72]);
	raw_estimate += 1.0 / (1u64 << register_288) as f32 + 1.0 / (1u64 << register_289) as f32 + 1.0 / (1u64 << register_290) as f32 + 1.0 / (1u64 << register_291) as f32;

	let [register_292, register_293, register_294, register_295] = split_registers::<4>(words[73]);
	raw_estimate += 1.0 / (1u64 << register_292) as f32 + 1.0 / (1u64 << register_293) as f32 + 1.0 / (1u64 << register_294) as f32 + 1.0 / (1u64 << register_295) as f32;

	let [register_296, register_297, register_298, register_299] = split_registers::<4>(words[74]);
	raw_estimate += 1.0 / (1u64 << register_296) as f32 + 1.0 / (1u64 << register_297) as f32 + 1.0 / (1u64 << register_298) as f32 + 1.0 / (1u64 << register_299) as f32;

	let [register_300, register_301, register_302, register_303] = split_registers::<4>(words[75]);
	raw_estimate += 1.0 / (1u64 << register_300) as f32 + 1.0 / (1u64 << register_301) as f32 + 1.0 / (1u64 << register_302) as f32 + 1.0 / (1u64 << register_303) as f32;

	let [register_304, register_305, register_306, register_307] = split_registers::<4>(words[76]);
	raw_estimate += 1.0 / (1u64 << register_304) as f32 + 1.0 / (1u64 << register_305) as f32 + 1.0 / (1u64 << register_306) as f32 + 1.0 / (1u64 << register_307) as f32;

	let [register_308, register_309, register_310, register_311] = split_registers::<4>(words[77]);
	raw_estimate += 1.0 / (1u64 << register_308) as f32 + 1.0 / (1u64 << register_309) as f32 + 1.0 / (1u64 << register_310) as f32 + 1.0 / (1u64 << register_311) as f32;

	let [register_312, register_313, register_314, register_315] = split_registers::<4>(words[78]);
	raw_estimate += 1.0 / (1u64 << register_312) as f32 + 1.0 / (1u64 << register_313) as f32 + 1.0 / (1u64 << register_314) as f32 + 1.0 / (1u64 << register_315) as f32;

	let [register_316, register_317, register_318, register_319] = split_registers::<4>(words[79]);
	raw_estimate += 1.0 / (1u64 << register_316) as f32 + 1.0 / (1u64 << register_317) as f32 + 1.0 / (1u64 << register_318) as f32 + 1.0 / (1u64 << register_319) as f32;

	let [register_320, register_321, register_322, register_323] = split_registers::<4>(words[80]);
	raw_estimate += 1.0 / (1u64 << register_320) as f32 + 1.0 / (1u64 << register_321) as f32 + 1.0 / (1u64 << register_322) as f32 + 1.0 / (1u64 << register_323) as f32;

	let [register_324, register_325, register_326, register_327] = split_registers::<4>(words[81]);
	raw_estimate += 1.0 / (1u64 << register_324) as f32 + 1.0 / (1u64 << register_325) as f32 + 1.0 / (1u64 << register_326) as f32 + 1.0 / (1u64 << register_327) as f32;

	let [register_328, register_329, register_330, register_331] = split_registers::<4>(words[82]);
	raw_estimate += 1.0 / (1u64 << register_328) as f32 + 1.0 / (1u64 << register_329) as f32 + 1.0 / (1u64 << register_330) as f32 + 1.0 / (1u64 << register_331) as f32;

	let [register_332, register_333, register_334, register_335] = split_registers::<4>(words[83]);
	raw_estimate += 1.0 / (1u64 << register_332) as f32 + 1.0 / (1u64 << register_333) as f32 + 1.0 / (1u64 << register_334) as f32 + 1.0 / (1u64 << register_335) as f32;

	let [register_336, register_337, register_338, register_339] = split_registers::<4>(words[84]);
	raw_estimate += 1.0 / (1u64 << register_336) as f32 + 1.0 / (1u64 << register_337) as f32 + 1.0 / (1u64 << register_338) as f32 + 1.0 / (1u64 << register_339) as f32;

	let [register_340, register_341, register_342, register_343] = split_registers::<4>(words[85]);
	raw_estimate += 1.0 / (1u64 << register_340) as f32 + 1.0 / (1u64 << register_341) as f32 + 1.0 / (1u64 << register_342) as f32 + 1.0 / (1u64 << register_343) as f32;

	let [register_344, register_345, register_346, register_347] = split_registers::<4>(words[86]);
	raw_estimate += 1.0 / (1u64 << register_344) as f32 + 1.0 / (1u64 << register_345) as f32 + 1.0 / (1u64 << register_346) as f32 + 1.0 / (1u64 << register_347) as f32;

	let [register_348, register_349, register_350, register_351] = split_registers::<4>(words[87]);
	raw_estimate += 1.0 / (1u64 << register_348) as f32 + 1.0 / (1u64 << register_349) as f32 + 1.0 / (1u64 << register_350) as f32 + 1.0 / (1u64 << register_351) as f32;

	let [register_352, register_353, register_354, register_355] = split_registers::<4>(words[88]);
	raw_estimate += 1.0 / (1u64 << register_352) as f32 + 1.0 / (1u64 << register_353) as f32 + 1.0 / (1u64 << register_354) as f32 + 1.0 / (1u64 << register_355) as f32;

	let [register_356, register_357, register_358, register_359] = split_registers::<4>(words[89]);
	raw_estimate += 1.0 / (1u64 << register_356) as f32 + 1.0 / (1u64 << register_357) as f32 + 1.0 / (1u64 << register_358) as f32 + 1.0 / (1u64 << register_359) as f32;

	let [register_360, register_361, register_362, register_363] = split_registers::<4>(words[90]);
	raw_estimate += 1.0 / (1u64 << register_360) as f32 + 1.0 / (1u64 << register_361) as f32 + 1.0 / (1u64 << register_362) as f32 + 1.0 / (1u64 << register_363) as f32;

	let [register_364, register_365, register_366, register_367] = split_registers::<4>(words[91]);
	raw_estimate += 1.0 / (1u64 << register_364) as f32 + 1.0 / (1u64 << register_365) as f32 + 1.0 / (1u64 << register_366) as f32 + 1.0 / (1u64 << register_367) as f32;

	let [register_368, register_369, register_370, register_371] = split_registers::<4>(words[92]);
	raw_estimate += 1.0 / (1u64 << register_368) as f32 + 1.0 / (1u64 << register_369) as f32 + 1.0 / (1u64 << register_370) as f32 + 1.0 / (1u64 << register_371) as f32;

	let [register_372, register_373, register_374, register_375] = split_registers::<4>(words[93]);
	raw_estimate += 1.0 / (1u64 << register_372) as f32 + 1.0 / (1u64 << register_373) as f32 + 1.0 / (1u64 << register_374) as f32 + 1.0 / (1u64 << register_375) as f32;

	let [register_376, register_377, register_378, register_379] = split_registers::<4>(words[94]);
	raw_estimate += 1.0 / (1u64 << register_376) as f32 + 1.0 / (1u64 << register_377) as f32 + 1.0 / (1u64 << register_378) as f32 + 1.0 / (1u64 << register_379) as f32;

	let [register_380, register_381, register_382, register_383] = split_registers::<4>(words[95]);
	raw_estimate += 1.0 / (1u64 << register_380) as f32 + 1.0 / (1u64 << register_381) as f32 + 1.0 / (1u64 << register_382) as f32 + 1.0 / (1u64 << register_383) as f32;

	let [register_384, register_385, register_386, register_387] = split_registers::<4>(words[96]);
	raw_estimate += 1.0 / (1u64 << register_384) as f32 + 1.0 / (1u64 << register_385) as f32 + 1.0 / (1u64 << register_386) as f32 + 1.0 / (1u64 << register_387) as f32;

	let [register_388, register_389, register_390, register_391] = split_registers::<4>(words[97]);
	raw_estimate += 1.0 / (1u64 << register_388) as f32 + 1.0 / (1u64 << register_389) as f32 + 1.0 / (1u64 << register_390) as f32 + 1.0 / (1u64 << register_391) as f32;

	let [register_392, register_393, register_394, register_395] = split_registers::<4>(words[98]);
	raw_estimate += 1.0 / (1u64 << register_392) as f32 + 1.0 / (1u64 << register_393) as f32 + 1.0 / (1u64 << register_394) as f32 + 1.0 / (1u64 << register_395) as f32;

	let [register_396, register_397, register_398, register_399] = split_registers::<4>(words[99]);
	raw_estimate += 1.0 / (1u64 << register_396) as f32 + 1.0 / (1u64 << register_397) as f32 + 1.0 / (1u64 << register_398) as f32 + 1.0 / (1u64 << register_399) as f32;

	let [register_400, register_401, register_402, register_403] = split_registers::<4>(words[100]);
	raw_estimate += 1.0 / (1u64 << register_400) as f32 + 1.0 / (1u64 << register_401) as f32 + 1.0 / (1u64 << register_402) as f32 + 1.0 / (1u64 << register_403) as f32;

	let [register_404, register_405, register_406, register_407] = split_registers::<4>(words[101]);
	raw_estimate += 1.0 / (1u64 << register_404) as f32 + 1.0 / (1u64 << register_405) as f32 + 1.0 / (1u64 << register_406) as f32 + 1.0 / (1u64 << register_407) as f32;

	let [register_408, register_409, register_410, register_411] = split_registers::<4>(words[102]);
	raw_estimate += 1.0 / (1u64 << register_408) as f32 + 1.0 / (1u64 << register_409) as f32 + 1.0 / (1u64 << register_410) as f32 + 1.0 / (1u64 << register_411) as f32;

	let [register_412, register_413, register_414, register_415] = split_registers::<4>(words[103]);
	raw_estimate += 1.0 / (1u64 << register_412) as f32 + 1.0 / (1u64 << register_413) as f32 + 1.0 / (1u64 << register_414) as f32 + 1.0 / (1u64 << register_415) as f32;

	let [register_416, register_417, register_418, register_419] = split_registers::<4>(words[104]);
	raw_estimate += 1.0 / (1u64 << register_416) as f32 + 1.0 / (1u64 << register_417) as f32 + 1.0 / (1u64 << register_418) as f32 + 1.0 / (1u64 << register_419) as f32;

	let [register_420, register_421, register_422, register_423] = split_registers::<4>(words[105]);
	raw_estimate += 1.0 / (1u64 << register_420) as f32 + 1.0 / (1u64 << register_421) as f32 + 1.0 / (1u64 << register_422) as f32 + 1.0 / (1u64 << register_423) as f32;

	let [register_424, register_425, register_426, register_427] = split_registers::<4>(words[106]);
	raw_estimate += 1.0 / (1u64 << register_424) as f32 + 1.0 / (1u64 << register_425) as f32 + 1.0 / (1u64 << register_426) as f32 + 1.0 / (1u64 << register_427) as f32;

	let [register_428, register_429, register_430, register_431] = split_registers::<4>(words[107]);
	raw_estimate += 1.0 / (1u64 << register_428) as f32 + 1.0 / (1u64 << register_429) as f32 + 1.0 / (1u64 << register_430) as f32 + 1.0 / (1u64 << register_431) as f32;

	let [register_432, register_433, register_434, register_435] = split_registers::<4>(words[108]);
	raw_estimate += 1.0 / (1u64 << register_432) as f32 + 1.0 / (1u64 << register_433) as f32 + 1.0 / (1u64 << register_434) as f32 + 1.0 / (1u64 << register_435) as f32;

	let [register_436, register_437, register_438, register_439] = split_registers::<4>(words[109]);
	raw_estimate += 1.0 / (1u64 << register_436) as f32 + 1.0 / (1u64 << register_437) as f32 + 1.0 / (1u64 << register_438) as f32 + 1.0 / (1u64 << register_439) as f32;

	let [register_440, register_441, register_442, register_443] = split_registers::<4>(words[110]);
	raw_estimate += 1.0 / (1u64 << register_440) as f32 + 1.0 / (1u64 << register_441) as f32 + 1.0 / (1u64 << register_442) as f32 + 1.0 / (1u64 << register_443) as f32;

	let [register_444, register_445, register_446, register_447] = split_registers::<4>(words[111]);
	raw_estimate += 1.0 / (1u64 << register_444) as f32 + 1.0 / (1u64 << register_445) as f32 + 1.0 / (1u64 << register_446) as f32 + 1.0 / (1u64 << register_447) as f32;

	let [register_448, register_449, register_450, register_451] = split_registers::<4>(words[112]);
	raw_estimate += 1.0 / (1u64 << register_448) as f32 + 1.0 / (1u64 << register_449) as f32 + 1.0 / (1u64 << register_450) as f32 + 1.0 / (1u64 << register_451) as f32;

	let [register_452, register_453, register_454, register_455] = split_registers::<4>(words[113]);
	raw_estimate += 1.0 / (1u64 << register_452) as f32 + 1.0 / (1u64 << register_453) as f32 + 1.0 / (1u64 << register_454) as f32 + 1.0 / (1u64 << register_455) as f32;

	let [register_456, register_457, register_458, register_459] = split_registers::<4>(words[114]);
	raw_estimate += 1.0 / (1u64 << register_456) as f32 + 1.0 / (1u64 << register_457) as f32 + 1.0 / (1u64 << register_458) as f32 + 1.0 / (1u64 << register_459) as f32;

	let [register_460, register_461, register_462, register_463] = split_registers::<4>(words[115]);
	raw_estimate += 1.0 / (1u64 << register_460) as f32 + 1.0 / (1u64 << register_461) as f32 + 1.0 / (1u64 << register_462) as f32 + 1.0 / (1u64 << register_463) as f32;

	let [register_464, register_465, register_466, register_467] = split_registers::<4>(words[116]);
	raw_estimate += 1.0 / (1u64 << register_464) as f32 + 1.0 / (1u64 << register_465) as f32 + 1.0 / (1u64 << register_466) as f32 + 1.0 / (1u64 << register_467) as f32;

	let [register_468, register_469, register_470, register_471] = split_registers::<4>(words[117]);
	raw_estimate += 1.0 / (1u64 << register_468) as f32 + 1.0 / (1u64 << register_469) as f32 + 1.0 / (1u64 << register_470) as f32 + 1.0 / (1u64 << register_471) as f32;

	let [register_472, register_473, register_474, register_475] = split_registers::<4>(words[118]);
	raw_estimate += 1.0 / (1u64 << register_472) as f32 + 1.0 / (1u64 << register_473) as f32 + 1.0 / (1u64 << register_474) as f32 + 1.0 / (1u64 << register_475) as f32;

	let [register_476, register_477, register_478, register_479] = split_registers::<4>(words[119]);
	raw_estimate += 1.0 / (1u64 << register_476) as f32 + 1.0 / (1u64 << register_477) as f32 + 1.0 / (1u64 << register_478) as f32 + 1.0 / (1u64 << register_479) as f32;

	let [register_480, register_481, register_482, register_483] = split_registers::<4>(words[120]);
	raw_estimate += 1.0 / (1u64 << register_480) as f32 + 1.0 / (1u64 << register_481) as f32 + 1.0 / (1u64 << register_482) as f32 + 1.0 / (1u64 << register_483) as f32;

	let [register_484, register_485, register_486, register_487] = split_registers::<4>(words[121]);
	raw_estimate += 1.0 / (1u64 << register_484) as f32 + 1.0 / (1u64 << register_485) as f32 + 1.0 / (1u64 << register_486) as f32 + 1.0 / (1u64 << register_487) as f32;

	let [register_488, register_489, register_490, register_491] = split_registers::<4>(words[122]);
	raw_estimate += 1.0 / (1u64 << register_488) as f32 + 1.0 / (1u64 << register_489) as f32 + 1.0 / (1u64 << register_490) as f32 + 1.0 / (1u64 << register_491) as f32;

	let [register_492, register_493, register_494, register_495] = split_registers::<4>(words[123]);
	raw_estimate += 1.0 / (1u64 << register_492) as f32 + 1.0 / (1u64 << register_493) as f32 + 1.0 / (1u64 << register_494) as f32 + 1.0 / (1u64 << register_495) as f32;

	let [register_496, register_497, register_498, register_499] = split_registers::<4>(words[124]);
	raw_estimate += 1.0 / (1u64 << register_496) as f32 + 1.0 / (1u64 << register_497) as f32 + 1.0 / (1u64 << register_498) as f32 + 1.0 / (1u64 << register_499) as f32;

	let [register_500, register_501, register_502, register_503] = split_registers::<4>(words[125]);
	raw_estimate += 1.0 / (1u64 << register_500) as f32 + 1.0 / (1u64 << register_501) as f32 + 1.0 / (1u64 << register_502) as f32 + 1.0 / (1u64 << register_503) as f32;

	let [register_504, register_505, register_506, register_507] = split_registers::<4>(words[126]);
	raw_estimate += 1.0 / (1u64 << register_504) as f32 + 1.0 / (1u64 << register_505) as f32 + 1.0 / (1u64 << register_506) as f32 + 1.0 / (1u64 << register_507) as f32;

	let [register_508, register_509, register_510, register_511] = split_registers::<4>(words[127]);
	raw_estimate += 1.0 / (1u64 << register_508) as f32 + 1.0 / (1u64 << register_509) as f32 + 1.0 / (1u64 << register_510) as f32 + 1.0 / (1u64 << register_511) as f32;

	let [register_512, register_513, register_514, register_515] = split_registers::<4>(words[128]);
	raw_estimate += 1.0 / (1u64 << register_512) as f32 + 1.0 / (1u64 << register_513) as f32 + 1.0 / (1u64 << register_514) as f32 + 1.0 / (1u64 << register_515) as f32;

	let [register_516, register_517, register_518, register_519] = split_registers::<4>(words[129]);
	raw_estimate += 1.0 / (1u64 << register_516) as f32 + 1.0 / (1u64 << register_517) as f32 + 1.0 / (1u64 << register_518) as f32 + 1.0 / (1u64 << register_519) as f32;

	let [register_520, register_521, register_522, register_523] = split_registers::<4>(words[130]);
	raw_estimate += 1.0 / (1u64 << register_520) as f32 + 1.0 / (1u64 << register_521) as f32 + 1.0 / (1u64 << register_522) as f32 + 1.0 / (1u64 << register_523) as f32;

	let [register_524, register_525, register_526, register_527] = split_registers::<4>(words[131]);
	raw_estimate += 1.0 / (1u64 << register_524) as f32 + 1.0 / (1u64 << register_525) as f32 + 1.0 / (1u64 << register_526) as f32 + 1.0 / (1u64 << register_527) as f32;

	let [register_528, register_529, register_530, register_531] = split_registers::<4>(words[132]);
	raw_estimate += 1.0 / (1u64 << register_528) as f32 + 1.0 / (1u64 << register_529) as f32 + 1.0 / (1u64 << register_530) as f32 + 1.0 / (1u64 << register_531) as f32;

	let [register_532, register_533, register_534, register_535] = split_registers::<4>(words[133]);
	raw_estimate += 1.0 / (1u64 << register_532) as f32 + 1.0 / (1u64 << register_533) as f32 + 1.0 / (1u64 << register_534) as f32 + 1.0 / (1u64 << register_535) as f32;

	let [register_536, register_537, register_538, register_539] = split_registers::<4>(words[134]);
	raw_estimate += 1.0 / (1u64 << register_536) as f32 + 1.0 / (1u64 << register_537) as f32 + 1.0 / (1u64 << register_538) as f32 + 1.0 / (1u64 << register_539) as f32;

	let [register_540, register_541, register_542, register_543] = split_registers::<4>(words[135]);
	raw_estimate += 1.0 / (1u64 << register_540) as f32 + 1.0 / (1u64 << register_541) as f32 + 1.0 / (1u64 << register_542) as f32 + 1.0 / (1u64 << register_543) as f32;

	let [register_544, register_545, register_546, register_547] = split_registers::<4>(words[136]);
	raw_estimate += 1.0 / (1u64 << register_544) as f32 + 1.0 / (1u64 << register_545) as f32 + 1.0 / (1u64 << register_546) as f32 + 1.0 / (1u64 << register_547) as f32;

	let [register_548, register_549, register_550, register_551] = split_registers::<4>(words[137]);
	raw_estimate += 1.0 / (1u64 << register_548) as f32 + 1.0 / (1u64 << register_549) as f32 + 1.0 / (1u64 << register_550) as f32 + 1.0 / (1u64 << register_551) as f32;

	let [register_552, register_553, register_554, register_555] = split_registers::<4>(words[138]);
	raw_estimate += 1.0 / (1u64 << register_552) as f32 + 1.0 / (1u64 << register_553) as f32 + 1.0 / (1u64 << register_554) as f32 + 1.0 / (1u64 << register_555) as f32;

	let [register_556, register_557, register_558, register_559] = split_registers::<4>(words[139]);
	raw_estimate += 1.0 / (1u64 << register_556) as f32 + 1.0 / (1u64 << register_557) as f32 + 1.0 / (1u64 << register_558) as f32 + 1.0 / (1u64 << register_559) as f32;

	let [register_560, register_561, register_562, register_563] = split_registers::<4>(words[140]);
	raw_estimate += 1.0 / (1u64 << register_560) as f32 + 1.0 / (1u64 << register_561) as f32 + 1.0 / (1u64 << register_562) as f32 + 1.0 / (1u64 << register_563) as f32;

	let [register_564, register_565, register_566, register_567] = split_registers::<4>(words[141]);
	raw_estimate += 1.0 / (1u64 << register_564) as f32 + 1.0 / (1u64 << register_565) as f32 + 1.0 / (1u64 << register_566) as f32 + 1.0 / (1u64 << register_567) as f32;

	let [register_568, register_569, register_570, register_571] = split_registers::<4>(words[142]);
	raw_estimate += 1.0 / (1u64 << register_568) as f32 + 1.0 / (1u64 << register_569) as f32 + 1.0 / (1u64 << register_570) as f32 + 1.0 / (1u64 << register_571) as f32;

	let [register_572, register_573, register_574, register_575] = split_registers::<4>(words[143]);
	raw_estimate += 1.0 / (1u64 << register_572) as f32 + 1.0 / (1u64 << register_573) as f32 + 1.0 / (1u64 << register_574) as f32 + 1.0 / (1u64 << register_575) as f32;

	let [register_576, register_577, register_578, register_579] = split_registers::<4>(words[144]);
	raw_estimate += 1.0 / (1u64 << register_576) as f32 + 1.0 / (1u64 << register_577) as f32 + 1.0 / (1u64 << register_578) as f32 + 1.0 / (1u64 << register_579) as f32;

	let [register_580, register_581, register_582, register_583] = split_registers::<4>(words[145]);
	raw_estimate += 1.0 / (1u64 << register_580) as f32 + 1.0 / (1u64 << register_581) as f32 + 1.0 / (1u64 << register_582) as f32 + 1.0 / (1u64 << register_583) as f32;

	let [register_584, register_585, register_586, register_587] = split_registers::<4>(words[146]);
	raw_estimate += 1.0 / (1u64 << register_584) as f32 + 1.0 / (1u64 << register_585) as f32 + 1.0 / (1u64 << register_586) as f32 + 1.0 / (1u64 << register_587) as f32;

	let [register_588, register_589, register_590, register_591] = split_registers::<4>(words[147]);
	raw_estimate += 1.0 / (1u64 << register_588) as f32 + 1.0 / (1u64 << register_589) as f32 + 1.0 / (1u64 << register_590) as f32 + 1.0 / (1u64 << register_591) as f32;

	let [register_592, register_593, register_594, register_595] = split_registers::<4>(words[148]);
	raw_estimate += 1.0 / (1u64 << register_592) as f32 + 1.0 / (1u64 << register_593) as f32 + 1.0 / (1u64 << register_594) as f32 + 1.0 / (1u64 << register_595) as f32;

	let [register_596, register_597, register_598, register_599] = split_registers::<4>(words[149]);
	raw_estimate += 1.0 / (1u64 << register_596) as f32 + 1.0 / (1u64 << register_597) as f32 + 1.0 / (1u64 << register_598) as f32 + 1.0 / (1u64 << register_599) as f32;

	let [register_600, register_601, register_602, register_603] = split_registers::<4>(words[150]);
	raw_estimate += 1.0 / (1u64 << register_600) as f32 + 1.0 / (1u64 << register_601) as f32 + 1.0 / (1u64 << register_602) as f32 + 1.0 / (1u64 << register_603) as f32;

	let [register_604, register_605, register_606, register_607] = split_registers::<4>(words[151]);
	raw_estimate += 1.0 / (1u64 << register_604) as f32 + 1.0 / (1u64 << register_605) as f32 + 1.0 / (1u64 << register_606) as f32 + 1.0 / (1u64 << register_607) as f32;

	let [register_608, register_609, register_610, register_611] = split_registers::<4>(words[152]);
	raw_estimate += 1.0 / (1u64 << register_608) as f32 + 1.0 / (1u64 << register_609) as f32 + 1.0 / (1u64 << register_610) as f32 + 1.0 / (1u64 << register_611) as f32;

	let [register_612, register_613, register_614, register_615] = split_registers::<4>(words[153]);
	raw_estimate += 1.0 / (1u64 << register_612) as f32 + 1.0 / (1u64 << register_613) as f32 + 1.0 / (1u64 << register_614) as f32 + 1.0 / (1u64 << register_615) as f32;

	let [register_616, register_617, register_618, register_619] = split_registers::<4>(words[154]);
	raw_estimate += 1.0 / (1u64 << register_616) as f32 + 1.0 / (1u64 << register_617) as f32 + 1.0 / (1u64 << register_618) as f32 + 1.0 / (1u64 << register_619) as f32;

	let [register_620, register_621, register_622, register_623] = split_registers::<4>(words[155]);
	raw_estimate += 1.0 / (1u64 << register_620) as f32 + 1.0 / (1u64 << register_621) as f32 + 1.0 / (1u64 << register_622) as f32 + 1.0 / (1u64 << register_623) as f32;

	let [register_624, register_625, register_626, register_627] = split_registers::<4>(words[156]);
	raw_estimate += 1.0 / (1u64 << register_624) as f32 + 1.0 / (1u64 << register_625) as f32 + 1.0 / (1u64 << register_626) as f32 + 1.0 / (1u64 << register_627) as f32;

	let [register_628, register_629, register_630, register_631] = split_registers::<4>(words[157]);
	raw_estimate += 1.0 / (1u64 << register_628) as f32 + 1.0 / (1u64 << register_629) as f32 + 1.0 / (1u64 << register_630) as f32 + 1.0 / (1u64 << register_631) as f32;

	let [register_632, register_633, register_634, register_635] = split_registers::<4>(words[158]);
	raw_estimate += 1.0 / (1u64 << register_632) as f32 + 1.0 / (1u64 << register_633) as f32 + 1.0 / (1u64 << register_634) as f32 + 1.0 / (1u64 << register_635) as f32;

	let [register_636, register_637, register_638, register_639] = split_registers::<4>(words[159]);
	raw_estimate += 1.0 / (1u64 << register_636) as f32 + 1.0 / (1u64 << register_637) as f32 + 1.0 / (1u64 << register_638) as f32 + 1.0 / (1u64 << register_639) as f32;

	let [register_640, register_641, register_642, register_643] = split_registers::<4>(words[160]);
	raw_estimate += 1.0 / (1u64 << register_640) as f32 + 1.0 / (1u64 << register_641) as f32 + 1.0 / (1u64 << register_642) as f32 + 1.0 / (1u64 << register_643) as f32;

	let [register_644, register_645, register_646, register_647] = split_registers::<4>(words[161]);
	raw_estimate += 1.0 / (1u64 << register_644) as f32 + 1.0 / (1u64 << register_645) as f32 + 1.0 / (1u64 << register_646) as f32 + 1.0 / (1u64 << register_647) as f32;

	let [register_648, register_649, register_650, register_651] = split_registers::<4>(words[162]);
	raw_estimate += 1.0 / (1u64 << register_648) as f32 + 1.0 / (1u64 << register_649) as f32 + 1.0 / (1u64 << register_650) as f32 + 1.0 / (1u64 << register_651) as f32;

	let [register_652, register_653, register_654, register_655] = split_registers::<4>(words[163]);
	raw_estimate += 1.0 / (1u64 << register_652) as f32 + 1.0 / (1u64 << register_653) as f32 + 1.0 / (1u64 << register_654) as f32 + 1.0 / (1u64 << register_655) as f32;

	let [register_656, register_657, register_658, register_659] = split_registers::<4>(words[164]);
	raw_estimate += 1.0 / (1u64 << register_656) as f32 + 1.0 / (1u64 << register_657) as f32 + 1.0 / (1u64 << register_658) as f32 + 1.0 / (1u64 << register_659) as f32;

	let [register_660, register_661, register_662, register_663] = split_registers::<4>(words[165]);
	raw_estimate += 1.0 / (1u64 << register_660) as f32 + 1.0 / (1u64 << register_661) as f32 + 1.0 / (1u64 << register_662) as f32 + 1.0 / (1u64 << register_663) as f32;

	let [register_664, register_665, register_666, register_667] = split_registers::<4>(words[166]);
	raw_estimate += 1.0 / (1u64 << register_664) as f32 + 1.0 / (1u64 << register_665) as f32 + 1.0 / (1u64 << register_666) as f32 + 1.0 / (1u64 << register_667) as f32;

	let [register_668, register_669, register_670, register_671] = split_registers::<4>(words[167]);
	raw_estimate += 1.0 / (1u64 << register_668) as f32 + 1.0 / (1u64 << register_669) as f32 + 1.0 / (1u64 << register_670) as f32 + 1.0 / (1u64 << register_671) as f32;

	let [register_672, register_673, register_674, register_675] = split_registers::<4>(words[168]);
	raw_estimate += 1.0 / (1u64 << register_672) as f32 + 1.0 / (1u64 << register_673) as f32 + 1.0 / (1u64 << register_674) as f32 + 1.0 / (1u64 << register_675) as f32;

	let [register_676, register_677, register_678, register_679] = split_registers::<4>(words[169]);
	raw_estimate += 1.0 / (1u64 << register_676) as f32 + 1.0 / (1u64 << register_677) as f32 + 1.0 / (1u64 << register_678) as f32 + 1.0 / (1u64 << register_679) as f32;

	let [register_680, register_681, register_682, register_683] = split_registers::<4>(words[170]);
	raw_estimate += 1.0 / (1u64 << register_680) as f32 + 1.0 / (1u64 << register_681) as f32 + 1.0 / (1u64 << register_682) as f32 + 1.0 / (1u64 << register_683) as f32;

	let [register_684, register_685, register_686, register_687] = split_registers::<4>(words[171]);
	raw_estimate += 1.0 / (1u64 << register_684) as f32 + 1.0 / (1u64 << register_685) as f32 + 1.0 / (1u64 << register_686) as f32 + 1.0 / (1u64 << register_687) as f32;

	let [register_688, register_689, register_690, register_691] = split_registers::<4>(words[172]);
	raw_estimate += 1.0 / (1u64 << register_688) as f32 + 1.0 / (1u64 << register_689) as f32 + 1.0 / (1u64 << register_690) as f32 + 1.0 / (1u64 << register_691) as f32;

	let [register_692, register_693, register_694, register_695] = split_registers::<4>(words[173]);
	raw_estimate += 1.0 / (1u64 << register_692) as f32 + 1.0 / (1u64 << register_693) as f32 + 1.0 / (1u64 << register_694) as f32 + 1.0 / (1u64 << register_695) as f32;

	let [register_696, register_697, register_698, register_699] = split_registers::<4>(words[174]);
	raw_estimate += 1.0 / (1u64 << register_696) as f32 + 1.0 / (1u64 << register_697) as f32 + 1.0 / (1u64 << register_698) as f32 + 1.0 / (1u64 << register_699) as f32;

	let [register_700, register_701, register_702, register_703] = split_registers::<4>(words[175]);
	raw_estimate += 1.0 / (1u64 << register_700) as f32 + 1.0 / (1u64 << register_701) as f32 + 1.0 / (1u64 << register_702) as f32 + 1.0 / (1u64 << register_703) as f32;

	let [register_704, register_705, register_706, register_707] = split_registers::<4>(words[176]);
	raw_estimate += 1.0 / (1u64 << register_704) as f32 + 1.0 / (1u64 << register_705) as f32 + 1.0 / (1u64 << register_706) as f32 + 1.0 / (1u64 << register_707) as f32;

	let [register_708, register_709, register_710, register_711] = split_registers::<4>(words[177]);
	raw_estimate += 1.0 / (1u64 << register_708) as f32 + 1.0 / (1u64 << register_709) as f32 + 1.0 / (1u64 << register_710) as f32 + 1.0 / (1u64 << register_711) as f32;

	let [register_712, register_713, register_714, register_715] = split_registers::<4>(words[178]);
	raw_estimate += 1.0 / (1u64 << register_712) as f32 + 1.0 / (1u64 << register_713) as f32 + 1.0 / (1u64 << register_714) as f32 + 1.0 / (1u64 << register_715) as f32;

	let [register_716, register_717, register_718, register_719] = split_registers::<4>(words[179]);
	raw_estimate += 1.0 / (1u64 << register_716) as f32 + 1.0 / (1u64 << register_717) as f32 + 1.0 / (1u64 << register_718) as f32 + 1.0 / (1u64 << register_719) as f32;

	let [register_720, register_721, register_722, register_723] = split_registers::<4>(words[180]);
	raw_estimate += 1.0 / (1u64 << register_720) as f32 + 1.0 / (1u64 << register_721) as f32 + 1.0 / (1u64 << register_722) as f32 + 1.0 / (1u64 << register_723) as f32;

	let [register_724, register_725, register_726, register_727] = split_registers::<4>(words[181]);
	raw_estimate += 1.0 / (1u64 << register_724) as f32 + 1.0 / (1u64 << register_725) as f32 + 1.0 / (1u64 << register_726) as f32 + 1.0 / (1u64 << register_727) as f32;

	let [register_728, register_729, register_730, register_731] = split_registers::<4>(words[182]);
	raw_estimate += 1.0 / (1u64 << register_728) as f32 + 1.0 / (1u64 << register_729) as f32 + 1.0 / (1u64 << register_730) as f32 + 1.0 / (1u64 << register_731) as f32;

	let [register_732, register_733, register_734, register_735] = split_registers::<4>(words[183]);
	raw_estimate += 1.0 / (1u64 << register_732) as f32 + 1.0 / (1u64 << register_733) as f32 + 1.0 / (1u64 << register_734) as f32 + 1.0 / (1u64 << register_735) as f32;

	let [register_736, register_737, register_738, register_739] = split_registers::<4>(words[184]);
	raw_estimate += 1.0 / (1u64 << register_736) as f32 + 1.0 / (1u64 << register_737) as f32 + 1.0 / (1u64 << register_738) as f32 + 1.0 / (1u64 << register_739) as f32;

	let [register_740, register_741, register_742, register_743] = split_registers::<4>(words[185]);
	raw_estimate += 1.0 / (1u64 << register_740) as f32 + 1.0 / (1u64 << register_741) as f32 + 1.0 / (1u64 << register_742) as f32 + 1.0 / (1u64 << register_743) as f32;

	let [register_744, register_745, register_746, register_747] = split_registers::<4>(words[186]);
	raw_estimate += 1.0 / (1u64 << register_744) as f32 + 1.0 / (1u64 << register_745) as f32 + 1.0 / (1u64 << register_746) as f32 + 1.0 / (1u64 << register_747) as f32;

	let [register_748, register_749, register_750, register_751] = split_registers::<4>(words[187]);
	raw_estimate += 1.0 / (1u64 << register_748) as f32 + 1.0 / (1u64 << register_749) as f32 + 1.0 / (1u64 << register_750) as f32 + 1.0 / (1u64 << register_751) as f32;

	let [register_752, register_753, register_754, register_755] = split_registers::<4>(words[188]);
	raw_estimate += 1.0 / (1u64 << register_752) as f32 + 1.0 / (1u64 << register_753) as f32 + 1.0 / (1u64 << register_754) as f32 + 1.0 / (1u64 << register_755) as f32;

	let [register_756, register_757, register_758, register_759] = split_registers::<4>(words[189]);
	raw_estimate += 1.0 / (1u64 << register_756) as f32 + 1.0 / (1u64 << register_757) as f32 + 1.0 / (1u64 << register_758) as f32 + 1.0 / (1u64 << register_759) as f32;

	let [register_760, register_761, register_762, register_763] = split_registers::<4>(words[190]);
	raw_estimate += 1.0 / (1u64 << register_760) as f32 + 1.0 / (1u64 << register_761) as f32 + 1.0 / (1u64 << register_762) as f32 + 1.0 / (1u64 << register_763) as f32;

	let [register_764, register_765, register_766, register_767] = split_registers::<4>(words[191]);
	raw_estimate += 1.0 / (1u64 << register_764) as f32 + 1.0 / (1u64 << register_765) as f32 + 1.0 / (1u64 << register_766) as f32 + 1.0 / (1u64 << register_767) as f32;

	let [register_768, register_769, register_770, register_771] = split_registers::<4>(words[192]);
	raw_estimate += 1.0 / (1u64 << register_768) as f32 + 1.0 / (1u64 << register_769) as f32 + 1.0 / (1u64 << register_770) as f32 + 1.0 / (1u64 << register_771) as f32;

	let [register_772, register_773, register_774, register_775] = split_registers::<4>(words[193]);
	raw_estimate += 1.0 / (1u64 << register_772) as f32 + 1.0 / (1u64 << register_773) as f32 + 1.0 / (1u64 << register_774) as f32 + 1.0 / (1u64 << register_775) as f32;

	let [register_776, register_777, register_778, register_779] = split_registers::<4>(words[194]);
	raw_estimate += 1.0 / (1u64 << register_776) as f32 + 1.0 / (1u64 << register_777) as f32 + 1.0 / (1u64 << register_778) as f32 + 1.0 / (1u64 << register_779) as f32;

	let [register_780, register_781, register_782, register_783] = split_registers::<4>(words[195]);
	raw_estimate += 1.0 / (1u64 << register_780) as f32 + 1.0 / (1u64 << register_781) as f32 + 1.0 / (1u64 << register_782) as f32 + 1.0 / (1u64 << register_783) as f32;

	let [register_784, register_785, register_786, register_787] = split_registers::<4>(words[196]);
	raw_estimate += 1.0 / (1u64 << register_784) as f32 + 1.0 / (1u64 << register_785) as f32 + 1.0 / (1u64 << register_786) as f32 + 1.0 / (1u64 << register_787) as f32;

	let [register_788, register_789, register_790, register_791] = split_registers::<4>(words[197]);
	raw_estimate += 1.0 / (1u64 << register_788) as f32 + 1.0 / (1u64 << register_789) as f32 + 1.0 / (1u64 << register_790) as f32 + 1.0 / (1u64 << register_791) as f32;

	let [register_792, register_793, register_794, register_795] = split_registers::<4>(words[198]);
	raw_estimate += 1.0 / (1u64 << register_792) as f32 + 1.0 / (1u64 << register_793) as f32 + 1.0 / (1u64 << register_794) as f32 + 1.0 / (1u64 << register_795) as f32;

	let [register_796, register_797, register_798, register_799] = split_registers::<4>(words[199]);
	raw_estimate += 1.0 / (1u64 << register_796) as f32 + 1.0 / (1u64 << register_797) as f32 + 1.0 / (1u64 << register_798) as f32 + 1.0 / (1u64 << register_799) as f32;

	let [register_800, register_801, register_802, register_803] = split_registers::<4>(words[200]);
	raw_estimate += 1.0 / (1u64 << register_800) as f32 + 1.0 / (1u64 << register_801) as f32 + 1.0 / (1u64 << register_802) as f32 + 1.0 / (1u64 << register_803) as f32;

	let [register_804, register_805, register_806, register_807] = split_registers::<4>(words[201]);
	raw_estimate += 1.0 / (1u64 << register_804) as f32 + 1.0 / (1u64 << register_805) as f32 + 1.0 / (1u64 << register_806) as f32 + 1.0 / (1u64 << register_807) as f32;

	let [register_808, register_809, register_810, register_811] = split_registers::<4>(words[202]);
	raw_estimate += 1.0 / (1u64 << register_808) as f32 + 1.0 / (1u64 << register_809) as f32 + 1.0 / (1u64 << register_810) as f32 + 1.0 / (1u64 << register_811) as f32;

	let [register_812, register_813, register_814, register_815] = split_registers::<4>(words[203]);
	raw_estimate += 1.0 / (1u64 << register_812) as f32 + 1.0 / (1u64 << register_813) as f32 + 1.0 / (1u64 << register_814) as f32 + 1.0 / (1u64 << register_815) as f32;

	let [register_816, register_817, register_818, register_819] = split_registers::<4>(words[204]);
	raw_estimate += 1.0 / (1u64 << register_816) as f32 + 1.0 / (1u64 << register_817) as f32 + 1.0 / (1u64 << register_818) as f32 + 1.0 / (1u64 << register_819) as f32;

	let [register_820, register_821, register_822, register_823] = split_registers::<4>(words[205]);
	raw_estimate += 1.0 / (1u64 << register_820) as f32 + 1.0 / (1u64 << register_821) as f32 + 1.0 / (1u64 << register_822) as f32 + 1.0 / (1u64 << register_823) as f32;

	let [register_824, register_825, register_826, register_827] = split_registers::<4>(words[206]);
	raw_estimate += 1.0 / (1u64 << register_824) as f32 + 1.0 / (1u64 << register_825) as f32 + 1.0 / (1u64 << register_826) as f32 + 1.0 / (1u64 << register_827) as f32;

	let [register_828, register_829, register_830, register_831] = split_registers::<4>(words[207]);
	raw_estimate += 1.0 / (1u64 << register_828) as f32 + 1.0 / (1u64 << register_829) as f32 + 1.0 / (1u64 << register_830) as f32 + 1.0 / (1u64 << register_831) as f32;

	let [register_832, register_833, register_834, register_835] = split_registers::<4>(words[208]);
	raw_estimate += 1.0 / (1u64 << register_832) as f32 + 1.0 / (1u64 << register_833) as f32 + 1.0 / (1u64 << register_834) as f32 + 1.0 / (1u64 << register_835) as f32;

	let [register_836, register_837, register_838, register_839] = split_registers::<4>(words[209]);
	raw_estimate += 1.0 / (1u64 << register_836) as f32 + 1.0 / (1u64 << register_837) as f32 + 1.0 / (1u64 << register_838) as f32 + 1.0 / (1u64 << register_839) as f32;

	let [register_840, register_841, register_842, register_843] = split_registers::<4>(words[210]);
	raw_estimate += 1.0 / (1u64 << register_840) as f32 + 1.0 / (1u64 << register_841) as f32 + 1.0 / (1u64 << register_842) as f32 + 1.0 / (1u64 << register_843) as f32;

	let [register_844, register_845, register_846, register_847] = split_registers::<4>(words[211]);
	raw_estimate += 1.0 / (1u64 << register_844) as f32 + 1.0 / (1u64 << register_845) as f32 + 1.0 / (1u64 << register_846) as f32 + 1.0 / (1u64 << register_847) as f32;

	let [register_848, register_849, register_850, register_851] = split_registers::<4>(words[212]);
	raw_estimate += 1.0 / (1u64 << register_848) as f32 + 1.0 / (1u64 << register_849) as f32 + 1.0 / (1u64 << register_850) as f32 + 1.0 / (1u64 << register_851) as f32;

	let [register_852, register_853, register_854, register_855] = split_registers::<4>(words[213]);
	raw_estimate += 1.0 / (1u64 << register_852) as f32 + 1.0 / (1u64 << register_853) as f32 + 1.0 / (1u64 << register_854) as f32 + 1.0 / (1u64 << register_855) as f32;

	let [register_856, register_857, register_858, register_859] = split_registers::<4>(words[214]);
	raw_estimate += 1.0 / (1u64 << register_856) as f32 + 1.0 / (1u64 << register_857) as f32 + 1.0 / (1u64 << register_858) as f32 + 1.0 / (1u64 << register_859) as f32;

	let [register_860, register_861, register_862, register_863] = split_registers::<4>(words[215]);
	raw_estimate += 1.0 / (1u64 << register_860) as f32 + 1.0 / (1u64 << register_861) as f32 + 1.0 / (1u64 << register_862) as f32 + 1.0 / (1u64 << register_863) as f32;

	let [register_864, register_865, register_866, register_867] = split_registers::<4>(words[216]);
	raw_estimate += 1.0 / (1u64 << register_864) as f32 + 1.0 / (1u64 << register_865) as f32 + 1.0 / (1u64 << register_866) as f32 + 1.0 / (1u64 << register_867) as f32;

	let [register_868, register_869, register_870, register_871] = split_registers::<4>(words[217]);
	raw_estimate += 1.0 / (1u64 << register_868) as f32 + 1.0 / (1u64 << register_869) as f32 + 1.0 / (1u64 << register_870) as f32 + 1.0 / (1u64 << register_871) as f32;

	let [register_872, register_873, register_874, register_875] = split_registers::<4>(words[218]);
	raw_estimate += 1.0 / (1u64 << register_872) as f32 + 1.0 / (1u64 << register_873) as f32 + 1.0 / (1u64 << register_874) as f32 + 1.0 / (1u64 << register_875) as f32;

	let [register_876, register_877, register_878, register_879] = split_registers::<4>(words[219]);
	raw_estimate += 1.0 / (1u64 << register_876) as f32 + 1.0 / (1u64 << register_877) as f32 + 1.0 / (1u64 << register_878) as f32 + 1.0 / (1u64 << register_879) as f32;

	let [register_880, register_881, register_882, register_883] = split_registers::<4>(words[220]);
	raw_estimate += 1.0 / (1u64 << register_880) as f32 + 1.0 / (1u64 << register_881) as f32 + 1.0 / (1u64 << register_882) as f32 + 1.0 / (1u64 << register_883) as f32;

	let [register_884, register_885, register_886, register_887] = split_registers::<4>(words[221]);
	raw_estimate += 1.0 / (1u64 << register_884) as f32 + 1.0 / (1u64 << register_885) as f32 + 1.0 / (1u64 << register_886) as f32 + 1.0 / (1u64 << register_887) as f32;

	let [register_888, register_889, register_890, register_891] = split_registers::<4>(words[222]);
	raw_estimate += 1.0 / (1u64 << register_888) as f32 + 1.0 / (1u64 << register_889) as f32 + 1.0 / (1u64 << register_890) as f32 + 1.0 / (1u64 << register_891) as f32;

	let [register_892, register_893, register_894, register_895] = split_registers::<4>(words[223]);
	raw_estimate += 1.0 / (1u64 << register_892) as f32 + 1.0 / (1u64 << register_893) as f32 + 1.0 / (1u64 << register_894) as f32 + 1.0 / (1u64 << register_895) as f32;

	let [register_896, register_897, register_898, register_899] = split_registers::<4>(words[224]);
	raw_estimate += 1.0 / (1u64 << register_896) as f32 + 1.0 / (1u64 << register_897) as f32 + 1.0 / (1u64 << register_898) as f32 + 1.0 / (1u64 << register_899) as f32;

	let [register_900, register_901, register_902, register_903] = split_registers::<4>(words[225]);
	raw_estimate += 1.0 / (1u64 << register_900) as f32 + 1.0 / (1u64 << register_901) as f32 + 1.0 / (1u64 << register_902) as f32 + 1.0 / (1u64 << register_903) as f32;

	let [register_904, register_905, register_906, register_907] = split_registers::<4>(words[226]);
	raw_estimate += 1.0 / (1u64 << register_904) as f32 + 1.0 / (1u64 << register_905) as f32 + 1.0 / (1u64 << register_906) as f32 + 1.0 / (1u64 << register_907) as f32;

	let [register_908, register_909, register_910, register_911] = split_registers::<4>(words[227]);
	raw_estimate += 1.0 / (1u64 << register_908) as f32 + 1.0 / (1u64 << register_909) as f32 + 1.0 / (1u64 << register_910) as f32 + 1.0 / (1u64 << register_911) as f32;

	let [register_912, register_913, register_914, register_915] = split_registers::<4>(words[228]);
	raw_estimate += 1.0 / (1u64 << register_912) as f32 + 1.0 / (1u64 << register_913) as f32 + 1.0 / (1u64 << register_914) as f32 + 1.0 / (1u64 << register_915) as f32;

	let [register_916, register_917, register_918, register_919] = split_registers::<4>(words[229]);
	raw_estimate += 1.0 / (1u64 << register_916) as f32 + 1.0 / (1u64 << register_917) as f32 + 1.0 / (1u64 << register_918) as f32 + 1.0 / (1u64 << register_919) as f32;

	let [register_920, register_921, register_922, register_923] = split_registers::<4>(words[230]);
	raw_estimate += 1.0 / (1u64 << register_920) as f32 + 1.0 / (1u64 << register_921) as f32 + 1.0 / (1u64 << register_922) as f32 + 1.0 / (1u64 << register_923) as f32;

	let [register_924, register_925, register_926, register_927] = split_registers::<4>(words[231]);
	raw_estimate += 1.0 / (1u64 << register_924) as f32 + 1.0 / (1u64 << register_925) as f32 + 1.0 / (1u64 << register_926) as f32 + 1.0 / (1u64 << register_927) as f32;

	let [register_928, register_929, register_930, register_931] = split_registers::<4>(words[232]);
	raw_estimate += 1.0 / (1u64 << register_928) as f32 + 1.0 / (1u64 << register_929) as f32 + 1.0 / (1u64 << register_930) as f32 + 1.0 / (1u64 << register_931) as f32;

	let [register_932, register_933, register_934, register_935] = split_registers::<4>(words[233]);
	raw_estimate += 1.0 / (1u64 << register_932) as f32 + 1.0 / (1u64 << register_933) as f32 + 1.0 / (1u64 << register_934) as f32 + 1.0 / (1u64 << register_935) as f32;

	let [register_936, register_937, register_938, register_939] = split_registers::<4>(words[234]);
	raw_estimate += 1.0 / (1u64 << register_936) as f32 + 1.0 / (1u64 << register_937) as f32 + 1.0 / (1u64 << register_938) as f32 + 1.0 / (1u64 << register_939) as f32;

	let [register_940, register_941, register_942, register_943] = split_registers::<4>(words[235]);
	raw_estimate += 1.0 / (1u64 << register_940) as f32 + 1.0 / (1u64 << register_941) as f32 + 1.0 / (1u64 << register_942) as f32 + 1.0 / (1u64 << register_943) as f32;

	let [register_944, register_945, register_946, register_947] = split_registers::<4>(words[236]);
	raw_estimate += 1.0 / (1u64 << register_944) as f32 + 1.0 / (1u64 << register_945) as f32 + 1.0 / (1u64 << register_946) as f32 + 1.0 / (1u64 << register_947) as f32;

	let [register_948, register_949, register_950, register_951] = split_registers::<4>(words[237]);
	raw_estimate += 1.0 / (1u64 << register_948) as f32 + 1.0 / (1u64 << register_949) as f32 + 1.0 / (1u64 << register_950) as f32 + 1.0 / (1u64 << register_951) as f32;

	let [register_952, register_953, register_954, register_955] = split_registers::<4>(words[238]);
	raw_estimate += 1.0 / (1u64 << register_952) as f32 + 1.0 / (1u64 << register_953) as f32 + 1.0 / (1u64 << register_954) as f32 + 1.0 / (1u64 << register_955) as f32;

	let [register_956, register_957, register_958, register_959] = split_registers::<4>(words[239]);
	raw_estimate += 1.0 / (1u64 << register_956) as f32 + 1.0 / (1u64 << register_957) as f32 + 1.0 / (1u64 << register_958) as f32 + 1.0 / (1u64 << register_959) as f32;

	let [register_960, register_961, register_962, register_963] = split_registers::<4>(words[240]);
	raw_estimate += 1.0 / (1u64 << register_960) as f32 + 1.0 / (1u64 << register_961) as f32 + 1.0 / (1u64 << register_962) as f32 + 1.0 / (1u64 << register_963) as f32;

	let [register_964, register_965, register_966, register_967] = split_registers::<4>(words[241]);
	raw_estimate += 1.0 / (1u64 << register_964) as f32 + 1.0 / (1u64 << register_965) as f32 + 1.0 / (1u64 << register_966) as f32 + 1.0 / (1u64 << register_967) as f32;

	let [register_968, register_969, register_970, register_971] = split_registers::<4>(words[242]);
	raw_estimate += 1.0 / (1u64 << register_968) as f32 + 1.0 / (1u64 << register_969) as f32 + 1.0 / (1u64 << register_970) as f32 + 1.0 / (1u64 << register_971) as f32;

	let [register_972, register_973, register_974, register_975] = split_registers::<4>(words[243]);
	raw_estimate += 1.0 / (1u64 << register_972) as f32 + 1.0 / (1u64 << register_973) as f32 + 1.0 / (1u64 << register_974) as f32 + 1.0 / (1u64 << register_975) as f32;

	let [register_976, register_977, register_978, register_979] = split_registers::<4>(words[244]);
	raw_estimate += 1.0 / (1u64 << register_976) as f32 + 1.0 / (1u64 << register_977) as f32 + 1.0 / (1u64 << register_978) as f32 + 1.0 / (1u64 << register_979) as f32;

	let [register_980, register_981, register_982, register_983] = split_registers::<4>(words[245]);
	raw_estimate += 1.0 / (1u64 << register_980) as f32 + 1.0 / (1u64 << register_981) as f32 + 1.0 / (1u64 << register_982) as f32 + 1.0 / (1u64 << register_983) as f32;

	let [register_984, register_985, register_986, register_987] = split_registers::<4>(words[246]);
	raw_estimate += 1.0 / (1u64 << register_984) as f32 + 1.0 / (1u64 << register_985) as f32 + 1.0 / (1u64 << register_986) as f32 + 1.0 / (1u64 << register_987) as f32;

	let [register_988, register_989, register_990, register_991] = split_registers::<4>(words[247]);
	raw_estimate += 1.0 / (1u64 << register_988) as f32 + 1.0 / (1u64 << register_989) as f32 + 1.0 / (1u64 << register_990) as f32 + 1.0 / (1u64 << register_991) as f32;

	let [register_992, register_993, register_994, register_995] = split_registers::<4>(words[248]);
	raw_estimate += 1.0 / (1u64 << register_992) as f32 + 1.0 / (1u64 << register_993) as f32 + 1.0 / (1u64 << register_994) as f32 + 1.0 / (1u64 << register_995) as f32;

	let [register_996, register_997, register_998, register_999] = split_registers::<4>(words[249]);
	raw_estimate += 1.0 / (1u64 << register_996) as f32 + 1.0 / (1u64 << register_997) as f32 + 1.0 / (1u64 << register_998) as f32 + 1.0 / (1u64 << register_999) as f32;

	let [register_1000, register_1001, register_1002, register_1003] = split_registers::<4>(words[250]);
	raw_estimate += 1.0 / (1u64 << register_1000) as f32 + 1.0 / (1u64 << register_1001) as f32 + 1.0 / (1u64 << register_1002) as f32 + 1.0 / (1u64 << register_1003) as f32;

	let [register_1004, register_1005, register_1006, register_1007] = split_registers::<4>(words[251]);
	raw_estimate += 1.0 / (1u64 << register_1004) as f32 + 1.0 / (1u64 << register_1005) as f32 + 1.0 / (1u64 << register_1006) as f32 + 1.0 / (1u64 << register_1007) as f32;

	let [register_1008, register_1009, register_1010, register_1011] = split_registers::<4>(words[252]);
	raw_estimate += 1.0 / (1u64 << register_1008) as f32 + 1.0 / (1u64 << register_1009) as f32 + 1.0 / (1u64 << register_1010) as f32 + 1.0 / (1u64 << register_1011) as f32;

	let [register_1012, register_1013, register_1014, register_1015] = split_registers::<4>(words[253]);
	raw_estimate += 1.0 / (1u64 << register_1012) as f32 + 1.0 / (1u64 << register_1013) as f32 + 1.0 / (1u64 << register_1014) as f32 + 1.0 / (1u64 << register_1015) as f32;

	let [register_1016, register_1017, register_1018, register_1019] = split_registers::<4>(words[254]);
	raw_estimate += 1.0 / (1u64 << register_1016) as f32 + 1.0 / (1u64 << register_1017) as f32 + 1.0 / (1u64 << register_1018) as f32 + 1.0 / (1u64 << register_1019) as f32;

	let [register_1020, register_1021, register_1022, register_1023] = split_registers::<4>(words[255]);
	raw_estimate += 1.0 / (1u64 << register_1020) as f32 + 1.0 / (1u64 << register_1021) as f32 + 1.0 / (1u64 << register_1022) as f32 + 1.0 / (1u64 << register_1023) as f32;

	let [register_1024, register_1025, register_1026, register_1027] = split_registers::<4>(words[256]);
	raw_estimate += 1.0 / (1u64 << register_1024) as f32 + 1.0 / (1u64 << register_1025) as f32 + 1.0 / (1u64 << register_1026) as f32 + 1.0 / (1u64 << register_1027) as f32;

	let [register_1028, register_1029, register_1030, register_1031] = split_registers::<4>(words[257]);
	raw_estimate += 1.0 / (1u64 << register_1028) as f32 + 1.0 / (1u64 << register_1029) as f32 + 1.0 / (1u64 << register_1030) as f32 + 1.0 / (1u64 << register_1031) as f32;

	let [register_1032, register_1033, register_1034, register_1035] = split_registers::<4>(words[258]);
	raw_estimate += 1.0 / (1u64 << register_1032) as f32 + 1.0 / (1u64 << register_1033) as f32 + 1.0 / (1u64 << register_1034) as f32 + 1.0 / (1u64 << register_1035) as f32;

	let [register_1036, register_1037, register_1038, register_1039] = split_registers::<4>(words[259]);
	raw_estimate += 1.0 / (1u64 << register_1036) as f32 + 1.0 / (1u64 << register_1037) as f32 + 1.0 / (1u64 << register_1038) as f32 + 1.0 / (1u64 << register_1039) as f32;

	let [register_1040, register_1041, register_1042, register_1043] = split_registers::<4>(words[260]);
	raw_estimate += 1.0 / (1u64 << register_1040) as f32 + 1.0 / (1u64 << register_1041) as f32 + 1.0 / (1u64 << register_1042) as f32 + 1.0 / (1u64 << register_1043) as f32;

	let [register_1044, register_1045, register_1046, register_1047] = split_registers::<4>(words[261]);
	raw_estimate += 1.0 / (1u64 << register_1044) as f32 + 1.0 / (1u64 << register_1045) as f32 + 1.0 / (1u64 << register_1046) as f32 + 1.0 / (1u64 << register_1047) as f32;

	let [register_1048, register_1049, register_1050, register_1051] = split_registers::<4>(words[262]);
	raw_estimate += 1.0 / (1u64 << register_1048) as f32 + 1.0 / (1u64 << register_1049) as f32 + 1.0 / (1u64 << register_1050) as f32 + 1.0 / (1u64 << register_1051) as f32;

	let [register_1052, register_1053, register_1054, register_1055] = split_registers::<4>(words[263]);
	raw_estimate += 1.0 / (1u64 << register_1052) as f32 + 1.0 / (1u64 << register_1053) as f32 + 1.0 / (1u64 << register_1054) as f32 + 1.0 / (1u64 << register_1055) as f32;

	let [register_1056, register_1057, register_1058, register_1059] = split_registers::<4>(words[264]);
	raw_estimate += 1.0 / (1u64 << register_1056) as f32 + 1.0 / (1u64 << register_1057) as f32 + 1.0 / (1u64 << register_1058) as f32 + 1.0 / (1u64 << register_1059) as f32;

	let [register_1060, register_1061, register_1062, register_1063] = split_registers::<4>(words[265]);
	raw_estimate += 1.0 / (1u64 << register_1060) as f32 + 1.0 / (1u64 << register_1061) as f32 + 1.0 / (1u64 << register_1062) as f32 + 1.0 / (1u64 << register_1063) as f32;

	let [register_1064, register_1065, register_1066, register_1067] = split_registers::<4>(words[266]);
	raw_estimate += 1.0 / (1u64 << register_1064) as f32 + 1.0 / (1u64 << register_1065) as f32 + 1.0 / (1u64 << register_1066) as f32 + 1.0 / (1u64 << register_1067) as f32;

	let [register_1068, register_1069, register_1070, register_1071] = split_registers::<4>(words[267]);
	raw_estimate += 1.0 / (1u64 << register_1068) as f32 + 1.0 / (1u64 << register_1069) as f32 + 1.0 / (1u64 << register_1070) as f32 + 1.0 / (1u64 << register_1071) as f32;

	let [register_1072, register_1073, register_1074, register_1075] = split_registers::<4>(words[268]);
	raw_estimate += 1.0 / (1u64 << register_1072) as f32 + 1.0 / (1u64 << register_1073) as f32 + 1.0 / (1u64 << register_1074) as f32 + 1.0 / (1u64 << register_1075) as f32;

	let [register_1076, register_1077, register_1078, register_1079] = split_registers::<4>(words[269]);
	raw_estimate += 1.0 / (1u64 << register_1076) as f32 + 1.0 / (1u64 << register_1077) as f32 + 1.0 / (1u64 << register_1078) as f32 + 1.0 / (1u64 << register_1079) as f32;

	let [register_1080, register_1081, register_1082, register_1083] = split_registers::<4>(words[270]);
	raw_estimate += 1.0 / (1u64 << register_1080) as f32 + 1.0 / (1u64 << register_1081) as f32 + 1.0 / (1u64 << register_1082) as f32 + 1.0 / (1u64 << register_1083) as f32;

	let [register_1084, register_1085, register_1086, register_1087] = split_registers::<4>(words[271]);
	raw_estimate += 1.0 / (1u64 << register_1084) as f32 + 1.0 / (1u64 << register_1085) as f32 + 1.0 / (1u64 << register_1086) as f32 + 1.0 / (1u64 << register_1087) as f32;

	let [register_1088, register_1089, register_1090, register_1091] = split_registers::<4>(words[272]);
	raw_estimate += 1.0 / (1u64 << register_1088) as f32 + 1.0 / (1u64 << register_1089) as f32 + 1.0 / (1u64 << register_1090) as f32 + 1.0 / (1u64 << register_1091) as f32;

	let [register_1092, register_1093, register_1094, register_1095] = split_registers::<4>(words[273]);
	raw_estimate += 1.0 / (1u64 << register_1092) as f32 + 1.0 / (1u64 << register_1093) as f32 + 1.0 / (1u64 << register_1094) as f32 + 1.0 / (1u64 << register_1095) as f32;

	let [register_1096, register_1097, register_1098, register_1099] = split_registers::<4>(words[274]);
	raw_estimate += 1.0 / (1u64 << register_1096) as f32 + 1.0 / (1u64 << register_1097) as f32 + 1.0 / (1u64 << register_1098) as f32 + 1.0 / (1u64 << register_1099) as f32;

	let [register_1100, register_1101, register_1102, register_1103] = split_registers::<4>(words[275]);
	raw_estimate += 1.0 / (1u64 << register_1100) as f32 + 1.0 / (1u64 << register_1101) as f32 + 1.0 / (1u64 << register_1102) as f32 + 1.0 / (1u64 << register_1103) as f32;

	let [register_1104, register_1105, register_1106, register_1107] = split_registers::<4>(words[276]);
	raw_estimate += 1.0 / (1u64 << register_1104) as f32 + 1.0 / (1u64 << register_1105) as f32 + 1.0 / (1u64 << register_1106) as f32 + 1.0 / (1u64 << register_1107) as f32;

	let [register_1108, register_1109, register_1110, register_1111] = split_registers::<4>(words[277]);
	raw_estimate += 1.0 / (1u64 << register_1108) as f32 + 1.0 / (1u64 << register_1109) as f32 + 1.0 / (1u64 << register_1110) as f32 + 1.0 / (1u64 << register_1111) as f32;

	let [register_1112, register_1113, register_1114, register_1115] = split_registers::<4>(words[278]);
	raw_estimate += 1.0 / (1u64 << register_1112) as f32 + 1.0 / (1u64 << register_1113) as f32 + 1.0 / (1u64 << register_1114) as f32 + 1.0 / (1u64 << register_1115) as f32;

	let [register_1116, register_1117, register_1118, register_1119] = split_registers::<4>(words[279]);
	raw_estimate += 1.0 / (1u64 << register_1116) as f32 + 1.0 / (1u64 << register_1117) as f32 + 1.0 / (1u64 << register_1118) as f32 + 1.0 / (1u64 << register_1119) as f32;

	let [register_1120, register_1121, register_1122, register_1123] = split_registers::<4>(words[280]);
	raw_estimate += 1.0 / (1u64 << register_1120) as f32 + 1.0 / (1u64 << register_1121) as f32 + 1.0 / (1u64 << register_1122) as f32 + 1.0 / (1u64 << register_1123) as f32;

	let [register_1124, register_1125, register_1126, register_1127] = split_registers::<4>(words[281]);
	raw_estimate += 1.0 / (1u64 << register_1124) as f32 + 1.0 / (1u64 << register_1125) as f32 + 1.0 / (1u64 << register_1126) as f32 + 1.0 / (1u64 << register_1127) as f32;

	let [register_1128, register_1129, register_1130, register_1131] = split_registers::<4>(words[282]);
	raw_estimate += 1.0 / (1u64 << register_1128) as f32 + 1.0 / (1u64 << register_1129) as f32 + 1.0 / (1u64 << register_1130) as f32 + 1.0 / (1u64 << register_1131) as f32;

	let [register_1132, register_1133, register_1134, register_1135] = split_registers::<4>(words[283]);
	raw_estimate += 1.0 / (1u64 << register_1132) as f32 + 1.0 / (1u64 << register_1133) as f32 + 1.0 / (1u64 << register_1134) as f32 + 1.0 / (1u64 << register_1135) as f32;

	let [register_1136, register_1137, register_1138, register_1139] = split_registers::<4>(words[284]);
	raw_estimate += 1.0 / (1u64 << register_1136) as f32 + 1.0 / (1u64 << register_1137) as f32 + 1.0 / (1u64 << register_1138) as f32 + 1.0 / (1u64 << register_1139) as f32;

	let [register_1140, register_1141, register_1142, register_1143] = split_registers::<4>(words[285]);
	raw_estimate += 1.0 / (1u64 << register_1140) as f32 + 1.0 / (1u64 << register_1141) as f32 + 1.0 / (1u64 << register_1142) as f32 + 1.0 / (1u64 << register_1143) as f32;

	let [register_1144, register_1145, register_1146, register_1147] = split_registers::<4>(words[286]);
	raw_estimate += 1.0 / (1u64 << register_1144) as f32 + 1.0 / (1u64 << register_1145) as f32 + 1.0 / (1u64 << register_1146) as f32 + 1.0 / (1u64 << register_1147) as f32;

	let [register_1148, register_1149, register_1150, register_1151] = split_registers::<4>(words[287]);
	raw_estimate += 1.0 / (1u64 << register_1148) as f32 + 1.0 / (1u64 << register_1149) as f32 + 1.0 / (1u64 << register_1150) as f32 + 1.0 / (1u64 << register_1151) as f32;

	let [register_1152, register_1153, register_1154, register_1155] = split_registers::<4>(words[288]);
	raw_estimate += 1.0 / (1u64 << register_1152) as f32 + 1.0 / (1u64 << register_1153) as f32 + 1.0 / (1u64 << register_1154) as f32 + 1.0 / (1u64 << register_1155) as f32;

	let [register_1156, register_1157, register_1158, register_1159] = split_registers::<4>(words[289]);
	raw_estimate += 1.0 / (1u64 << register_1156) as f32 + 1.0 / (1u64 << register_1157) as f32 + 1.0 / (1u64 << register_1158) as f32 + 1.0 / (1u64 << register_1159) as f32;

	let [register_1160, register_1161, register_1162, register_1163] = split_registers::<4>(words[290]);
	raw_estimate += 1.0 / (1u64 << register_1160) as f32 + 1.0 / (1u64 << register_1161) as f32 + 1.0 / (1u64 << register_1162) as f32 + 1.0 / (1u64 << register_1163) as f32;

	let [register_1164, register_1165, register_1166, register_1167] = split_registers::<4>(words[291]);
	raw_estimate += 1.0 / (1u64 << register_1164) as f32 + 1.0 / (1u64 << register_1165) as f32 + 1.0 / (1u64 << register_1166) as f32 + 1.0 / (1u64 << register_1167) as f32;

	let [register_1168, register_1169, register_1170, register_1171] = split_registers::<4>(words[292]);
	raw_estimate += 1.0 / (1u64 << register_1168) as f32 + 1.0 / (1u64 << register_1169) as f32 + 1.0 / (1u64 << register_1170) as f32 + 1.0 / (1u64 << register_1171) as f32;

	let [register_1172, register_1173, register_1174, register_1175] = split_registers::<4>(words[293]);
	raw_estimate += 1.0 / (1u64 << register_1172) as f32 + 1.0 / (1u64 << register_1173) as f32 + 1.0 / (1u64 << register_1174) as f32 + 1.0 / (1u64 << register_1175) as f32;

	let [register_1176, register_1177, register_1178, register_1179] = split_registers::<4>(words[294]);
	raw_estimate += 1.0 / (1u64 << register_1176) as f32 + 1.0 / (1u64 << register_1177) as f32 + 1.0 / (1u64 << register_1178) as f32 + 1.0 / (1u64 << register_1179) as f32;

	let [register_1180, register_1181, register_1182, register_1183] = split_registers::<4>(words[295]);
	raw_estimate += 1.0 / (1u64 << register_1180) as f32 + 1.0 / (1u64 << register_1181) as f32 + 1.0 / (1u64 << register_1182) as f32 + 1.0 / (1u64 << register_1183) as f32;

	let [register_1184, register_1185, register_1186, register_1187] = split_registers::<4>(words[296]);
	raw_estimate += 1.0 / (1u64 << register_1184) as f32 + 1.0 / (1u64 << register_1185) as f32 + 1.0 / (1u64 << register_1186) as f32 + 1.0 / (1u64 << register_1187) as f32;

	let [register_1188, register_1189, register_1190, register_1191] = split_registers::<4>(words[297]);
	raw_estimate += 1.0 / (1u64 << register_1188) as f32 + 1.0 / (1u64 << register_1189) as f32 + 1.0 / (1u64 << register_1190) as f32 + 1.0 / (1u64 << register_1191) as f32;

	let [register_1192, register_1193, register_1194, register_1195] = split_registers::<4>(words[298]);
	raw_estimate += 1.0 / (1u64 << register_1192) as f32 + 1.0 / (1u64 << register_1193) as f32 + 1.0 / (1u64 << register_1194) as f32 + 1.0 / (1u64 << register_1195) as f32;

	let [register_1196, register_1197, register_1198, register_1199] = split_registers::<4>(words[299]);
	raw_estimate += 1.0 / (1u64 << register_1196) as f32 + 1.0 / (1u64 << register_1197) as f32 + 1.0 / (1u64 << register_1198) as f32 + 1.0 / (1u64 << register_1199) as f32;

	let [register_1200, register_1201, register_1202, register_1203] = split_registers::<4>(words[300]);
	raw_estimate += 1.0 / (1u64 << register_1200) as f32 + 1.0 / (1u64 << register_1201) as f32 + 1.0 / (1u64 << register_1202) as f32 + 1.0 / (1u64 << register_1203) as f32;

	let [register_1204, register_1205, register_1206, register_1207] = split_registers::<4>(words[301]);
	raw_estimate += 1.0 / (1u64 << register_1204) as f32 + 1.0 / (1u64 << register_1205) as f32 + 1.0 / (1u64 << register_1206) as f32 + 1.0 / (1u64 << register_1207) as f32;

	let [register_1208, register_1209, register_1210, register_1211] = split_registers::<4>(words[302]);
	raw_estimate += 1.0 / (1u64 << register_1208) as f32 + 1.0 / (1u64 << register_1209) as f32 + 1.0 / (1u64 << register_1210) as f32 + 1.0 / (1u64 << register_1211) as f32;

	let [register_1212, register_1213, register_1214, register_1215] = split_registers::<4>(words[303]);
	raw_estimate += 1.0 / (1u64 << register_1212) as f32 + 1.0 / (1u64 << register_1213) as f32 + 1.0 / (1u64 << register_1214) as f32 + 1.0 / (1u64 << register_1215) as f32;

	let [register_1216, register_1217, register_1218, register_1219] = split_registers::<4>(words[304]);
	raw_estimate += 1.0 / (1u64 << register_1216) as f32 + 1.0 / (1u64 << register_1217) as f32 + 1.0 / (1u64 << register_1218) as f32 + 1.0 / (1u64 << register_1219) as f32;

	let [register_1220, register_1221, register_1222, register_1223] = split_registers::<4>(words[305]);
	raw_estimate += 1.0 / (1u64 << register_1220) as f32 + 1.0 / (1u64 << register_1221) as f32 + 1.0 / (1u64 << register_1222) as f32 + 1.0 / (1u64 << register_1223) as f32;

	let [register_1224, register_1225, register_1226, register_1227] = split_registers::<4>(words[306]);
	raw_estimate += 1.0 / (1u64 << register_1224) as f32 + 1.0 / (1u64 << register_1225) as f32 + 1.0 / (1u64 << register_1226) as f32 + 1.0 / (1u64 << register_1227) as f32;

	let [register_1228, register_1229, register_1230, register_1231] = split_registers::<4>(words[307]);
	raw_estimate += 1.0 / (1u64 << register_1228) as f32 + 1.0 / (1u64 << register_1229) as f32 + 1.0 / (1u64 << register_1230) as f32 + 1.0 / (1u64 << register_1231) as f32;

	let [register_1232, register_1233, register_1234, register_1235] = split_registers::<4>(words[308]);
	raw_estimate += 1.0 / (1u64 << register_1232) as f32 + 1.0 / (1u64 << register_1233) as f32 + 1.0 / (1u64 << register_1234) as f32 + 1.0 / (1u64 << register_1235) as f32;

	let [register_1236, register_1237, register_1238, register_1239] = split_registers::<4>(words[309]);
	raw_estimate += 1.0 / (1u64 << register_1236) as f32 + 1.0 / (1u64 << register_1237) as f32 + 1.0 / (1u64 << register_1238) as f32 + 1.0 / (1u64 << register_1239) as f32;

	let [register_1240, register_1241, register_1242, register_1243] = split_registers::<4>(words[310]);
	raw_estimate += 1.0 / (1u64 << register_1240) as f32 + 1.0 / (1u64 << register_1241) as f32 + 1.0 / (1u64 << register_1242) as f32 + 1.0 / (1u64 << register_1243) as f32;

	let [register_1244, register_1245, register_1246, register_1247] = split_registers::<4>(words[311]);
	raw_estimate += 1.0 / (1u64 << register_1244) as f32 + 1.0 / (1u64 << register_1245) as f32 + 1.0 / (1u64 << register_1246) as f32 + 1.0 / (1u64 << register_1247) as f32;

	let [register_1248, register_1249, register_1250, register_1251] = split_registers::<4>(words[312]);
	raw_estimate += 1.0 / (1u64 << register_1248) as f32 + 1.0 / (1u64 << register_1249) as f32 + 1.0 / (1u64 << register_1250) as f32 + 1.0 / (1u64 << register_1251) as f32;

	let [register_1252, register_1253, register_1254, register_1255] = split_registers::<4>(words[313]);
	raw_estimate += 1.0 / (1u64 << register_1252) as f32 + 1.0 / (1u64 << register_1253) as f32 + 1.0 / (1u64 << register_1254) as f32 + 1.0 / (1u64 << register_1255) as f32;

	let [register_1256, register_1257, register_1258, register_1259] = split_registers::<4>(words[314]);
	raw_estimate += 1.0 / (1u64 << register_1256) as f32 + 1.0 / (1u64 << register_1257) as f32 + 1.0 / (1u64 << register_1258) as f32 + 1.0 / (1u64 << register_1259) as f32;

	let [register_1260, register_1261, register_1262, register_1263] = split_registers::<4>(words[315]);
	raw_estimate += 1.0 / (1u64 << register_1260) as f32 + 1.0 / (1u64 << register_1261) as f32 + 1.0 / (1u64 << register_1262) as f32 + 1.0 / (1u64 << register_1263) as f32;

	let [register_1264, register_1265, register_1266, register_1267] = split_registers::<4>(words[316]);
	raw_estimate += 1.0 / (1u64 << register_1264) as f32 + 1.0 / (1u64 << register_1265) as f32 + 1.0 / (1u64 << register_1266) as f32 + 1.0 / (1u64 << register_1267) as f32;

	let [register_1268, register_1269, register_1270, register_1271] = split_registers::<4>(words[317]);
	raw_estimate += 1.0 / (1u64 << register_1268) as f32 + 1.0 / (1u64 << register_1269) as f32 + 1.0 / (1u64 << register_1270) as f32 + 1.0 / (1u64 << register_1271) as f32;

	let [register_1272, register_1273, register_1274, register_1275] = split_registers::<4>(words[318]);
	raw_estimate += 1.0 / (1u64 << register_1272) as f32 + 1.0 / (1u64 << register_1273) as f32 + 1.0 / (1u64 << register_1274) as f32 + 1.0 / (1u64 << register_1275) as f32;

	let [register_1276, register_1277, register_1278, register_1279] = split_registers::<4>(words[319]);
	raw_estimate += 1.0 / (1u64 << register_1276) as f32 + 1.0 / (1u64 << register_1277) as f32 + 1.0 / (1u64 << register_1278) as f32 + 1.0 / (1u64 << register_1279) as f32;

	let [register_1280, register_1281, register_1282, register_1283] = split_registers::<4>(words[320]);
	raw_estimate += 1.0 / (1u64 << register_1280) as f32 + 1.0 / (1u64 << register_1281) as f32 + 1.0 / (1u64 << register_1282) as f32 + 1.0 / (1u64 << register_1283) as f32;

	let [register_1284, register_1285, register_1286, register_1287] = split_registers::<4>(words[321]);
	raw_estimate += 1.0 / (1u64 << register_1284) as f32 + 1.0 / (1u64 << register_1285) as f32 + 1.0 / (1u64 << register_1286) as f32 + 1.0 / (1u64 << register_1287) as f32;

	let [register_1288, register_1289, register_1290, register_1291] = split_registers::<4>(words[322]);
	raw_estimate += 1.0 / (1u64 << register_1288) as f32 + 1.0 / (1u64 << register_1289) as f32 + 1.0 / (1u64 << register_1290) as f32 + 1.0 / (1u64 << register_1291) as f32;

	let [register_1292, register_1293, register_1294, register_1295] = split_registers::<4>(words[323]);
	raw_estimate += 1.0 / (1u64 << register_1292) as f32 + 1.0 / (1u64 << register_1293) as f32 + 1.0 / (1u64 << register_1294) as f32 + 1.0 / (1u64 << register_1295) as f32;

	let [register_1296, register_1297, register_1298, register_1299] = split_registers::<4>(words[324]);
	raw_estimate += 1.0 / (1u64 << register_1296) as f32 + 1.0 / (1u64 << register_1297) as f32 + 1.0 / (1u64 << register_1298) as f32 + 1.0 / (1u64 << register_1299) as f32;

	let [register_1300, register_1301, register_1302, register_1303] = split_registers::<4>(words[325]);
	raw_estimate += 1.0 / (1u64 << register_1300) as f32 + 1.0 / (1u64 << register_1301) as f32 + 1.0 / (1u64 << register_1302) as f32 + 1.0 / (1u64 << register_1303) as f32;

	let [register_1304, register_1305, register_1306, register_1307] = split_registers::<4>(words[326]);
	raw_estimate += 1.0 / (1u64 << register_1304) as f32 + 1.0 / (1u64 << register_1305) as f32 + 1.0 / (1u64 << register_1306) as f32 + 1.0 / (1u64 << register_1307) as f32;

	let [register_1308, register_1309, register_1310, register_1311] = split_registers::<4>(words[327]);
	raw_estimate += 1.0 / (1u64 << register_1308) as f32 + 1.0 / (1u64 << register_1309) as f32 + 1.0 / (1u64 << register_1310) as f32 + 1.0 / (1u64 << register_1311) as f32;

	let [register_1312, register_1313, register_1314, register_1315] = split_registers::<4>(words[328]);
	raw_estimate += 1.0 / (1u64 << register_1312) as f32 + 1.0 / (1u64 << register_1313) as f32 + 1.0 / (1u64 << register_1314) as f32 + 1.0 / (1u64 << register_1315) as f32;

	let [register_1316, register_1317, register_1318, register_1319] = split_registers::<4>(words[329]);
	raw_estimate += 1.0 / (1u64 << register_1316) as f32 + 1.0 / (1u64 << register_1317) as f32 + 1.0 / (1u64 << register_1318) as f32 + 1.0 / (1u64 << register_1319) as f32;

	let [register_1320, register_1321, register_1322, register_1323] = split_registers::<4>(words[330]);
	raw_estimate += 1.0 / (1u64 << register_1320) as f32 + 1.0 / (1u64 << register_1321) as f32 + 1.0 / (1u64 << register_1322) as f32 + 1.0 / (1u64 << register_1323) as f32;

	let [register_1324, register_1325, register_1326, register_1327] = split_registers::<4>(words[331]);
	raw_estimate += 1.0 / (1u64 << register_1324) as f32 + 1.0 / (1u64 << register_1325) as f32 + 1.0 / (1u64 << register_1326) as f32 + 1.0 / (1u64 << register_1327) as f32;

	let [register_1328, register_1329, register_1330, register_1331] = split_registers::<4>(words[332]);
	raw_estimate += 1.0 / (1u64 << register_1328) as f32 + 1.0 / (1u64 << register_1329) as f32 + 1.0 / (1u64 << register_1330) as f32 + 1.0 / (1u64 << register_1331) as f32;

	let [register_1332, register_1333, register_1334, register_1335] = split_registers::<4>(words[333]);
	raw_estimate += 1.0 / (1u64 << register_1332) as f32 + 1.0 / (1u64 << register_1333) as f32 + 1.0 / (1u64 << register_1334) as f32 + 1.0 / (1u64 << register_1335) as f32;

	let [register_1336, register_1337, register_1338, register_1339] = split_registers::<4>(words[334]);
	raw_estimate += 1.0 / (1u64 << register_1336) as f32 + 1.0 / (1u64 << register_1337) as f32 + 1.0 / (1u64 << register_1338) as f32 + 1.0 / (1u64 << register_1339) as f32;

	let [register_1340, register_1341, register_1342, register_1343] = split_registers::<4>(words[335]);
	raw_estimate += 1.0 / (1u64 << register_1340) as f32 + 1.0 / (1u64 << register_1341) as f32 + 1.0 / (1u64 << register_1342) as f32 + 1.0 / (1u64 << register_1343) as f32;

	let [register_1344, register_1345, register_1346, register_1347] = split_registers::<4>(words[336]);
	raw_estimate += 1.0 / (1u64 << register_1344) as f32 + 1.0 / (1u64 << register_1345) as f32 + 1.0 / (1u64 << register_1346) as f32 + 1.0 / (1u64 << register_1347) as f32;

	let [register_1348, register_1349, register_1350, register_1351] = split_registers::<4>(words[337]);
	raw_estimate += 1.0 / (1u64 << register_1348) as f32 + 1.0 / (1u64 << register_1349) as f32 + 1.0 / (1u64 << register_1350) as f32 + 1.0 / (1u64 << register_1351) as f32;

	let [register_1352, register_1353, register_1354, register_1355] = split_registers::<4>(words[338]);
	raw_estimate += 1.0 / (1u64 << register_1352) as f32 + 1.0 / (1u64 << register_1353) as f32 + 1.0 / (1u64 << register_1354) as f32 + 1.0 / (1u64 << register_1355) as f32;

	let [register_1356, register_1357, register_1358, register_1359] = split_registers::<4>(words[339]);
	raw_estimate += 1.0 / (1u64 << register_1356) as f32 + 1.0 / (1u64 << register_1357) as f32 + 1.0 / (1u64 << register_1358) as f32 + 1.0 / (1u64 << register_1359) as f32;

	let [register_1360, register_1361, register_1362, register_1363] = split_registers::<4>(words[340]);
	raw_estimate += 1.0 / (1u64 << register_1360) as f32 + 1.0 / (1u64 << register_1361) as f32 + 1.0 / (1u64 << register_1362) as f32 + 1.0 / (1u64 << register_1363) as f32;

	let [register_1364, register_1365, register_1366, register_1367] = split_registers::<4>(words[341]);
	raw_estimate += 1.0 / (1u64 << register_1364) as f32 + 1.0 / (1u64 << register_1365) as f32 + 1.0 / (1u64 << register_1366) as f32 + 1.0 / (1u64 << register_1367) as f32;

	let [register_1368, register_1369, register_1370, register_1371] = split_registers::<4>(words[342]);
	raw_estimate += 1.0 / (1u64 << register_1368) as f32 + 1.0 / (1u64 << register_1369) as f32 + 1.0 / (1u64 << register_1370) as f32 + 1.0 / (1u64 << register_1371) as f32;

	let [register_1372, register_1373, register_1374, register_1375] = split_registers::<4>(words[343]);
	raw_estimate += 1.0 / (1u64 << register_1372) as f32 + 1.0 / (1u64 << register_1373) as f32 + 1.0 / (1u64 << register_1374) as f32 + 1.0 / (1u64 << register_1375) as f32;

	let [register_1376, register_1377, register_1378, register_1379] = split_registers::<4>(words[344]);
	raw_estimate += 1.0 / (1u64 << register_1376) as f32 + 1.0 / (1u64 << register_1377) as f32 + 1.0 / (1u64 << register_1378) as f32 + 1.0 / (1u64 << register_1379) as f32;

	let [register_1380, register_1381, register_1382, register_1383] = split_registers::<4>(words[345]);
	raw_estimate += 1.0 / (1u64 << register_1380) as f32 + 1.0 / (1u64 << register_1381) as f32 + 1.0 / (1u64 << register_1382) as f32 + 1.0 / (1u64 << register_1383) as f32;

	let [register_1384, register_1385, register_1386, register_1387] = split_registers::<4>(words[346]);
	raw_estimate += 1.0 / (1u64 << register_1384) as f32 + 1.0 / (1u64 << register_1385) as f32 + 1.0 / (1u64 << register_1386) as f32 + 1.0 / (1u64 << register_1387) as f32;

	let [register_1388, register_1389, register_1390, register_1391] = split_registers::<4>(words[347]);
	raw_estimate += 1.0 / (1u64 << register_1388) as f32 + 1.0 / (1u64 << register_1389) as f32 + 1.0 / (1u64 << register_1390) as f32 + 1.0 / (1u64 << register_1391) as f32;

	let [register_1392, register_1393, register_1394, register_1395] = split_registers::<4>(words[348]);
	raw_estimate += 1.0 / (1u64 << register_1392) as f32 + 1.0 / (1u64 << register_1393) as f32 + 1.0 / (1u64 << register_1394) as f32 + 1.0 / (1u64 << register_1395) as f32;

	let [register_1396, register_1397, register_1398, register_1399] = split_registers::<4>(words[349]);
	raw_estimate += 1.0 / (1u64 << register_1396) as f32 + 1.0 / (1u64 << register_1397) as f32 + 1.0 / (1u64 << register_1398) as f32 + 1.0 / (1u64 << register_1399) as f32;

	let [register_1400, register_1401, register_1402, register_1403] = split_registers::<4>(words[350]);
	raw_estimate += 1.0 / (1u64 << register_1400) as f32 + 1.0 / (1u64 << register_1401) as f32 + 1.0 / (1u64 << register_1402) as f32 + 1.0 / (1u64 << register_1403) as f32;

	let [register_1404, register_1405, register_1406, register_1407] = split_registers::<4>(words[351]);
	raw_estimate += 1.0 / (1u64 << register_1404) as f32 + 1.0 / (1u64 << register_1405) as f32 + 1.0 / (1u64 << register_1406) as f32 + 1.0 / (1u64 << register_1407) as f32;

	let [register_1408, register_1409, register_1410, register_1411] = split_registers::<4>(words[352]);
	raw_estimate += 1.0 / (1u64 << register_1408) as f32 + 1.0 / (1u64 << register_1409) as f32 + 1.0 / (1u64 << register_1410) as f32 + 1.0 / (1u64 << register_1411) as f32;

	let [register_1412, register_1413, register_1414, register_1415] = split_registers::<4>(words[353]);
	raw_estimate += 1.0 / (1u64 << register_1412) as f32 + 1.0 / (1u64 << register_1413) as f32 + 1.0 / (1u64 << register_1414) as f32 + 1.0 / (1u64 << register_1415) as f32;

	let [register_1416, register_1417, register_1418, register_1419] = split_registers::<4>(words[354]);
	raw_estimate += 1.0 / (1u64 << register_1416) as f32 + 1.0 / (1u64 << register_1417) as f32 + 1.0 / (1u64 << register_1418) as f32 + 1.0 / (1u64 << register_1419) as f32;

	let [register_1420, register_1421, register_1422, register_1423] = split_registers::<4>(words[355]);
	raw_estimate += 1.0 / (1u64 << register_1420) as f32 + 1.0 / (1u64 << register_1421) as f32 + 1.0 / (1u64 << register_1422) as f32 + 1.0 / (1u64 << register_1423) as f32;

	let [register_1424, register_1425, register_1426, register_1427] = split_registers::<4>(words[356]);
	raw_estimate += 1.0 / (1u64 << register_1424) as f32 + 1.0 / (1u64 << register_1425) as f32 + 1.0 / (1u64 << register_1426) as f32 + 1.0 / (1u64 << register_1427) as f32;

	let [register_1428, register_1429, register_1430, register_1431] = split_registers::<4>(words[357]);
	raw_estimate += 1.0 / (1u64 << register_1428) as f32 + 1.0 / (1u64 << register_1429) as f32 + 1.0 / (1u64 << register_1430) as f32 + 1.0 / (1u64 << register_1431) as f32;

	let [register_1432, register_1433, register_1434, register_1435] = split_registers::<4>(words[358]);
	raw_estimate += 1.0 / (1u64 << register_1432) as f32 + 1.0 / (1u64 << register_1433) as f32 + 1.0 / (1u64 << register_1434) as f32 + 1.0 / (1u64 << register_1435) as f32;

	let [register_1436, register_1437, register_1438, register_1439] = split_registers::<4>(words[359]);
	raw_estimate += 1.0 / (1u64 << register_1436) as f32 + 1.0 / (1u64 << register_1437) as f32 + 1.0 / (1u64 << register_1438) as f32 + 1.0 / (1u64 << register_1439) as f32;

	let [register_1440, register_1441, register_1442, register_1443] = split_registers::<4>(words[360]);
	raw_estimate += 1.0 / (1u64 << register_1440) as f32 + 1.0 / (1u64 << register_1441) as f32 + 1.0 / (1u64 << register_1442) as f32 + 1.0 / (1u64 << register_1443) as f32;

	let [register_1444, register_1445, register_1446, register_1447] = split_registers::<4>(words[361]);
	raw_estimate += 1.0 / (1u64 << register_1444) as f32 + 1.0 / (1u64 << register_1445) as f32 + 1.0 / (1u64 << register_1446) as f32 + 1.0 / (1u64 << register_1447) as f32;

	let [register_1448, register_1449, register_1450, register_1451] = split_registers::<4>(words[362]);
	raw_estimate += 1.0 / (1u64 << register_1448) as f32 + 1.0 / (1u64 << register_1449) as f32 + 1.0 / (1u64 << register_1450) as f32 + 1.0 / (1u64 << register_1451) as f32;

	let [register_1452, register_1453, register_1454, register_1455] = split_registers::<4>(words[363]);
	raw_estimate += 1.0 / (1u64 << register_1452) as f32 + 1.0 / (1u64 << register_1453) as f32 + 1.0 / (1u64 << register_1454) as f32 + 1.0 / (1u64 << register_1455) as f32;

	let [register_1456, register_1457, register_1458, register_1459] = split_registers::<4>(words[364]);
	raw_estimate += 1.0 / (1u64 << register_1456) as f32 + 1.0 / (1u64 << register_1457) as f32 + 1.0 / (1u64 << register_1458) as f32 + 1.0 / (1u64 << register_1459) as f32;

	let [register_1460, register_1461, register_1462, register_1463] = split_registers::<4>(words[365]);
	raw_estimate += 1.0 / (1u64 << register_1460) as f32 + 1.0 / (1u64 << register_1461) as f32 + 1.0 / (1u64 << register_1462) as f32 + 1.0 / (1u64 << register_1463) as f32;

	let [register_1464, register_1465, register_1466, register_1467] = split_registers::<4>(words[366]);
	raw_estimate += 1.0 / (1u64 << register_1464) as f32 + 1.0 / (1u64 << register_1465) as f32 + 1.0 / (1u64 << register_1466) as f32 + 1.0 / (1u64 << register_1467) as f32;

	let [register_1468, register_1469, register_1470, register_1471] = split_registers::<4>(words[367]);
	raw_estimate += 1.0 / (1u64 << register_1468) as f32 + 1.0 / (1u64 << register_1469) as f32 + 1.0 / (1u64 << register_1470) as f32 + 1.0 / (1u64 << register_1471) as f32;

	let [register_1472, register_1473, register_1474, register_1475] = split_registers::<4>(words[368]);
	raw_estimate += 1.0 / (1u64 << register_1472) as f32 + 1.0 / (1u64 << register_1473) as f32 + 1.0 / (1u64 << register_1474) as f32 + 1.0 / (1u64 << register_1475) as f32;

	let [register_1476, register_1477, register_1478, register_1479] = split_registers::<4>(words[369]);
	raw_estimate += 1.0 / (1u64 << register_1476) as f32 + 1.0 / (1u64 << register_1477) as f32 + 1.0 / (1u64 << register_1478) as f32 + 1.0 / (1u64 << register_1479) as f32;

	let [register_1480, register_1481, register_1482, register_1483] = split_registers::<4>(words[370]);
	raw_estimate += 1.0 / (1u64 << register_1480) as f32 + 1.0 / (1u64 << register_1481) as f32 + 1.0 / (1u64 << register_1482) as f32 + 1.0 / (1u64 << register_1483) as f32;

	let [register_1484, register_1485, register_1486, register_1487] = split_registers::<4>(words[371]);
	raw_estimate += 1.0 / (1u64 << register_1484) as f32 + 1.0 / (1u64 << register_1485) as f32 + 1.0 / (1u64 << register_1486) as f32 + 1.0 / (1u64 << register_1487) as f32;

	let [register_1488, register_1489, register_1490, register_1491] = split_registers::<4>(words[372]);
	raw_estimate += 1.0 / (1u64 << register_1488) as f32 + 1.0 / (1u64 << register_1489) as f32 + 1.0 / (1u64 << register_1490) as f32 + 1.0 / (1u64 << register_1491) as f32;

	let [register_1492, register_1493, register_1494, register_1495] = split_registers::<4>(words[373]);
	raw_estimate += 1.0 / (1u64 << register_1492) as f32 + 1.0 / (1u64 << register_1493) as f32 + 1.0 / (1u64 << register_1494) as f32 + 1.0 / (1u64 << register_1495) as f32;

	let [register_1496, register_1497, register_1498, register_1499] = split_registers::<4>(words[374]);
	raw_estimate += 1.0 / (1u64 << register_1496) as f32 + 1.0 / (1u64 << register_1497) as f32 + 1.0 / (1u64 << register_1498) as f32 + 1.0 / (1u64 << register_1499) as f32;

	let [register_1500, register_1501, register_1502, register_1503] = split_registers::<4>(words[375]);
	raw_estimate += 1.0 / (1u64 << register_1500) as f32 + 1.0 / (1u64 << register_1501) as f32 + 1.0 / (1u64 << register_1502) as f32 + 1.0 / (1u64 << register_1503) as f32;

	let [register_1504, register_1505, register_1506, register_1507] = split_registers::<4>(words[376]);
	raw_estimate += 1.0 / (1u64 << register_1504) as f32 + 1.0 / (1u64 << register_1505) as f32 + 1.0 / (1u64 << register_1506) as f32 + 1.0 / (1u64 << register_1507) as f32;

	let [register_1508, register_1509, register_1510, register_1511] = split_registers::<4>(words[377]);
	raw_estimate += 1.0 / (1u64 << register_1508) as f32 + 1.0 / (1u64 << register_1509) as f32 + 1.0 / (1u64 << register_1510) as f32 + 1.0 / (1u64 << register_1511) as f32;

	let [register_1512, register_1513, register_1514, register_1515] = split_registers::<4>(words[378]);
	raw_estimate += 1.0 / (1u64 << register_1512) as f32 + 1.0 / (1u64 << register_1513) as f32 + 1.0 / (1u64 << register_1514) as f32 + 1.0 / (1u64 << register_1515) as f32;

	let [register_1516, register_1517, register_1518, register_1519] = split_registers::<4>(words[379]);
	raw_estimate += 1.0 / (1u64 << register_1516) as f32 + 1.0 / (1u64 << register_1517) as f32 + 1.0 / (1u64 << register_1518) as f32 + 1.0 / (1u64 << register_1519) as f32;

	let [register_1520, register_1521, register_1522, register_1523] = split_registers::<4>(words[380]);
	raw_estimate += 1.0 / (1u64 << register_1520) as f32 + 1.0 / (1u64 << register_1521) as f32 + 1.0 / (1u64 << register_1522) as f32 + 1.0 / (1u64 << register_1523) as f32;

	let [register_1524, register_1525, register_1526, register_1527] = split_registers::<4>(words[381]);
	raw_estimate += 1.0 / (1u64 << register_1524) as f32 + 1.0 / (1u64 << register_1525) as f32 + 1.0 / (1u64 << register_1526) as f32 + 1.0 / (1u64 << register_1527) as f32;

	let [register_1528, register_1529, register_1530, register_1531] = split_registers::<4>(words[382]);
	raw_estimate += 1.0 / (1u64 << register_1528) as f32 + 1.0 / (1u64 << register_1529) as f32 + 1.0 / (1u64 << register_1530) as f32 + 1.0 / (1u64 << register_1531) as f32;

	let [register_1532, register_1533, register_1534, register_1535] = split_registers::<4>(words[383]);
	raw_estimate += 1.0 / (1u64 << register_1532) as f32 + 1.0 / (1u64 << register_1533) as f32 + 1.0 / (1u64 << register_1534) as f32 + 1.0 / (1u64 << register_1535) as f32;

	let [register_1536, register_1537, register_1538, register_1539] = split_registers::<4>(words[384]);
	raw_estimate += 1.0 / (1u64 << register_1536) as f32 + 1.0 / (1u64 << register_1537) as f32 + 1.0 / (1u64 << register_1538) as f32 + 1.0 / (1u64 << register_1539) as f32;

	let [register_1540, register_1541, register_1542, register_1543] = split_registers::<4>(words[385]);
	raw_estimate += 1.0 / (1u64 << register_1540) as f32 + 1.0 / (1u64 << register_1541) as f32 + 1.0 / (1u64 << register_1542) as f32 + 1.0 / (1u64 << register_1543) as f32;

	let [register_1544, register_1545, register_1546, register_1547] = split_registers::<4>(words[386]);
	raw_estimate += 1.0 / (1u64 << register_1544) as f32 + 1.0 / (1u64 << register_1545) as f32 + 1.0 / (1u64 << register_1546) as f32 + 1.0 / (1u64 << register_1547) as f32;

	let [register_1548, register_1549, register_1550, register_1551] = split_registers::<4>(words[387]);
	raw_estimate += 1.0 / (1u64 << register_1548) as f32 + 1.0 / (1u64 << register_1549) as f32 + 1.0 / (1u64 << register_1550) as f32 + 1.0 / (1u64 << register_1551) as f32;

	let [register_1552, register_1553, register_1554, register_1555] = split_registers::<4>(words[388]);
	raw_estimate += 1.0 / (1u64 << register_1552) as f32 + 1.0 / (1u64 << register_1553) as f32 + 1.0 / (1u64 << register_1554) as f32 + 1.0 / (1u64 << register_1555) as f32;

	let [register_1556, register_1557, register_1558, register_1559] = split_registers::<4>(words[389]);
	raw_estimate += 1.0 / (1u64 << register_1556) as f32 + 1.0 / (1u64 << register_1557) as f32 + 1.0 / (1u64 << register_1558) as f32 + 1.0 / (1u64 << register_1559) as f32;

	let [register_1560, register_1561, register_1562, register_1563] = split_registers::<4>(words[390]);
	raw_estimate += 1.0 / (1u64 << register_1560) as f32 + 1.0 / (1u64 << register_1561) as f32 + 1.0 / (1u64 << register_1562) as f32 + 1.0 / (1u64 << register_1563) as f32;

	let [register_1564, register_1565, register_1566, register_1567] = split_registers::<4>(words[391]);
	raw_estimate += 1.0 / (1u64 << register_1564) as f32 + 1.0 / (1u64 << register_1565) as f32 + 1.0 / (1u64 << register_1566) as f32 + 1.0 / (1u64 << register_1567) as f32;

	let [register_1568, register_1569, register_1570, register_1571] = split_registers::<4>(words[392]);
	raw_estimate += 1.0 / (1u64 << register_1568) as f32 + 1.0 / (1u64 << register_1569) as f32 + 1.0 / (1u64 << register_1570) as f32 + 1.0 / (1u64 << register_1571) as f32;

	let [register_1572, register_1573, register_1574, register_1575] = split_registers::<4>(words[393]);
	raw_estimate += 1.0 / (1u64 << register_1572) as f32 + 1.0 / (1u64 << register_1573) as f32 + 1.0 / (1u64 << register_1574) as f32 + 1.0 / (1u64 << register_1575) as f32;

	let [register_1576, register_1577, register_1578, register_1579] = split_registers::<4>(words[394]);
	raw_estimate += 1.0 / (1u64 << register_1576) as f32 + 1.0 / (1u64 << register_1577) as f32 + 1.0 / (1u64 << register_1578) as f32 + 1.0 / (1u64 << register_1579) as f32;

	let [register_1580, register_1581, register_1582, register_1583] = split_registers::<4>(words[395]);
	raw_estimate += 1.0 / (1u64 << register_1580) as f32 + 1.0 / (1u64 << register_1581) as f32 + 1.0 / (1u64 << register_1582) as f32 + 1.0 / (1u64 << register_1583) as f32;

	let [register_1584, register_1585, register_1586, register_1587] = split_registers::<4>(words[396]);
	raw_estimate += 1.0 / (1u64 << register_1584) as f32 + 1.0 / (1u64 << register_1585) as f32 + 1.0 / (1u64 << register_1586) as f32 + 1.0 / (1u64 << register_1587) as f32;

	let [register_1588, register_1589, register_1590, register_1591] = split_registers::<4>(words[397]);
	raw_estimate += 1.0 / (1u64 << register_1588) as f32 + 1.0 / (1u64 << register_1589) as f32 + 1.0 / (1u64 << register_1590) as f32 + 1.0 / (1u64 << register_1591) as f32;

	let [register_1592, register_1593, register_1594, register_1595] = split_registers::<4>(words[398]);
	raw_estimate += 1.0 / (1u64 << register_1592) as f32 + 1.0 / (1u64 << register_1593) as f32 + 1.0 / (1u64 << register_1594) as f32 + 1.0 / (1u64 << register_1595) as f32;

	let [register_1596, register_1597, register_1598, register_1599] = split_registers::<4>(words[399]);
	raw_estimate += 1.0 / (1u64 << register_1596) as f32 + 1.0 / (1u64 << register_1597) as f32 + 1.0 / (1u64 << register_1598) as f32 + 1.0 / (1u64 << register_1599) as f32;

	let [register_1600, register_1601, register_1602, register_1603] = split_registers::<4>(words[400]);
	raw_estimate += 1.0 / (1u64 << register_1600) as f32 + 1.0 / (1u64 << register_1601) as f32 + 1.0 / (1u64 << register_1602) as f32 + 1.0 / (1u64 << register_1603) as f32;

	let [register_1604, register_1605, register_1606, register_1607] = split_registers::<4>(words[401]);
	raw_estimate += 1.0 / (1u64 << register_1604) as f32 + 1.0 / (1u64 << register_1605) as f32 + 1.0 / (1u64 << register_1606) as f32 + 1.0 / (1u64 << register_1607) as f32;

	let [register_1608, register_1609, register_1610, register_1611] = split_registers::<4>(words[402]);
	raw_estimate += 1.0 / (1u64 << register_1608) as f32 + 1.0 / (1u64 << register_1609) as f32 + 1.0 / (1u64 << register_1610) as f32 + 1.0 / (1u64 << register_1611) as f32;

	let [register_1612, register_1613, register_1614, register_1615] = split_registers::<4>(words[403]);
	raw_estimate += 1.0 / (1u64 << register_1612) as f32 + 1.0 / (1u64 << register_1613) as f32 + 1.0 / (1u64 << register_1614) as f32 + 1.0 / (1u64 << register_1615) as f32;

	let [register_1616, register_1617, register_1618, register_1619] = split_registers::<4>(words[404]);
	raw_estimate += 1.0 / (1u64 << register_1616) as f32 + 1.0 / (1u64 << register_1617) as f32 + 1.0 / (1u64 << register_1618) as f32 + 1.0 / (1u64 << register_1619) as f32;

	let [register_1620, register_1621, register_1622, register_1623] = split_registers::<4>(words[405]);
	raw_estimate += 1.0 / (1u64 << register_1620) as f32 + 1.0 / (1u64 << register_1621) as f32 + 1.0 / (1u64 << register_1622) as f32 + 1.0 / (1u64 << register_1623) as f32;

	let [register_1624, register_1625, register_1626, register_1627] = split_registers::<4>(words[406]);
	raw_estimate += 1.0 / (1u64 << register_1624) as f32 + 1.0 / (1u64 << register_1625) as f32 + 1.0 / (1u64 << register_1626) as f32 + 1.0 / (1u64 << register_1627) as f32;

	let [register_1628, register_1629, register_1630, register_1631] = split_registers::<4>(words[407]);
	raw_estimate += 1.0 / (1u64 << register_1628) as f32 + 1.0 / (1u64 << register_1629) as f32 + 1.0 / (1u64 << register_1630) as f32 + 1.0 / (1u64 << register_1631) as f32;

	let [register_1632, register_1633, register_1634, register_1635] = split_registers::<4>(words[408]);
	raw_estimate += 1.0 / (1u64 << register_1632) as f32 + 1.0 / (1u64 << register_1633) as f32 + 1.0 / (1u64 << register_1634) as f32 + 1.0 / (1u64 << register_1635) as f32;

	let [register_1636, register_1637, register_1638, register_1639] = split_registers::<4>(words[409]);
	raw_estimate += 1.0 / (1u64 << register_1636) as f32 + 1.0 / (1u64 << register_1637) as f32 + 1.0 / (1u64 << register_1638) as f32 + 1.0 / (1u64 << register_1639) as f32;

	let [register_1640, register_1641, register_1642, register_1643] = split_registers::<4>(words[410]);
	raw_estimate += 1.0 / (1u64 << register_1640) as f32 + 1.0 / (1u64 << register_1641) as f32 + 1.0 / (1u64 << register_1642) as f32 + 1.0 / (1u64 << register_1643) as f32;

	let [register_1644, register_1645, register_1646, register_1647] = split_registers::<4>(words[411]);
	raw_estimate += 1.0 / (1u64 << register_1644) as f32 + 1.0 / (1u64 << register_1645) as f32 + 1.0 / (1u64 << register_1646) as f32 + 1.0 / (1u64 << register_1647) as f32;

	let [register_1648, register_1649, register_1650, register_1651] = split_registers::<4>(words[412]);
	raw_estimate += 1.0 / (1u64 << register_1648) as f32 + 1.0 / (1u64 << register_1649) as f32 + 1.0 / (1u64 << register_1650) as f32 + 1.0 / (1u64 << register_1651) as f32;

	let [register_1652, register_1653, register_1654, register_1655] = split_registers::<4>(words[413]);
	raw_estimate += 1.0 / (1u64 << register_1652) as f32 + 1.0 / (1u64 << register_1653) as f32 + 1.0 / (1u64 << register_1654) as f32 + 1.0 / (1u64 << register_1655) as f32;

	let [register_1656, register_1657, register_1658, register_1659] = split_registers::<4>(words[414]);
	raw_estimate += 1.0 / (1u64 << register_1656) as f32 + 1.0 / (1u64 << register_1657) as f32 + 1.0 / (1u64 << register_1658) as f32 + 1.0 / (1u64 << register_1659) as f32;

	let [register_1660, register_1661, register_1662, register_1663] = split_registers::<4>(words[415]);
	raw_estimate += 1.0 / (1u64 << register_1660) as f32 + 1.0 / (1u64 << register_1661) as f32 + 1.0 / (1u64 << register_1662) as f32 + 1.0 / (1u64 << register_1663) as f32;

	let [register_1664, register_1665, register_1666, register_1667] = split_registers::<4>(words[416]);
	raw_estimate += 1.0 / (1u64 << register_1664) as f32 + 1.0 / (1u64 << register_1665) as f32 + 1.0 / (1u64 << register_1666) as f32 + 1.0 / (1u64 << register_1667) as f32;

	let [register_1668, register_1669, register_1670, register_1671] = split_registers::<4>(words[417]);
	raw_estimate += 1.0 / (1u64 << register_1668) as f32 + 1.0 / (1u64 << register_1669) as f32 + 1.0 / (1u64 << register_1670) as f32 + 1.0 / (1u64 << register_1671) as f32;

	let [register_1672, register_1673, register_1674, register_1675] = split_registers::<4>(words[418]);
	raw_estimate += 1.0 / (1u64 << register_1672) as f32 + 1.0 / (1u64 << register_1673) as f32 + 1.0 / (1u64 << register_1674) as f32 + 1.0 / (1u64 << register_1675) as f32;

	let [register_1676, register_1677, register_1678, register_1679] = split_registers::<4>(words[419]);
	raw_estimate += 1.0 / (1u64 << register_1676) as f32 + 1.0 / (1u64 << register_1677) as f32 + 1.0 / (1u64 << register_1678) as f32 + 1.0 / (1u64 << register_1679) as f32;

	let [register_1680, register_1681, register_1682, register_1683] = split_registers::<4>(words[420]);
	raw_estimate += 1.0 / (1u64 << register_1680) as f32 + 1.0 / (1u64 << register_1681) as f32 + 1.0 / (1u64 << register_1682) as f32 + 1.0 / (1u64 << register_1683) as f32;

	let [register_1684, register_1685, register_1686, register_1687] = split_registers::<4>(words[421]);
	raw_estimate += 1.0 / (1u64 << register_1684) as f32 + 1.0 / (1u64 << register_1685) as f32 + 1.0 / (1u64 << register_1686) as f32 + 1.0 / (1u64 << register_1687) as f32;

	let [register_1688, register_1689, register_1690, register_1691] = split_registers::<4>(words[422]);
	raw_estimate += 1.0 / (1u64 << register_1688) as f32 + 1.0 / (1u64 << register_1689) as f32 + 1.0 / (1u64 << register_1690) as f32 + 1.0 / (1u64 << register_1691) as f32;

	let [register_1692, register_1693, register_1694, register_1695] = split_registers::<4>(words[423]);
	raw_estimate += 1.0 / (1u64 << register_1692) as f32 + 1.0 / (1u64 << register_1693) as f32 + 1.0 / (1u64 << register_1694) as f32 + 1.0 / (1u64 << register_1695) as f32;

	let [register_1696, register_1697, register_1698, register_1699] = split_registers::<4>(words[424]);
	raw_estimate += 1.0 / (1u64 << register_1696) as f32 + 1.0 / (1u64 << register_1697) as f32 + 1.0 / (1u64 << register_1698) as f32 + 1.0 / (1u64 << register_1699) as f32;

	let [register_1700, register_1701, register_1702, register_1703] = split_registers::<4>(words[425]);
	raw_estimate += 1.0 / (1u64 << register_1700) as f32 + 1.0 / (1u64 << register_1701) as f32 + 1.0 / (1u64 << register_1702) as f32 + 1.0 / (1u64 << register_1703) as f32;

	let [register_1704, register_1705, register_1706, register_1707] = split_registers::<4>(words[426]);
	raw_estimate += 1.0 / (1u64 << register_1704) as f32 + 1.0 / (1u64 << register_1705) as f32 + 1.0 / (1u64 << register_1706) as f32 + 1.0 / (1u64 << register_1707) as f32;

	let [register_1708, register_1709, register_1710, register_1711] = split_registers::<4>(words[427]);
	raw_estimate += 1.0 / (1u64 << register_1708) as f32 + 1.0 / (1u64 << register_1709) as f32 + 1.0 / (1u64 << register_1710) as f32 + 1.0 / (1u64 << register_1711) as f32;

	let [register_1712, register_1713, register_1714, register_1715] = split_registers::<4>(words[428]);
	raw_estimate += 1.0 / (1u64 << register_1712) as f32 + 1.0 / (1u64 << register_1713) as f32 + 1.0 / (1u64 << register_1714) as f32 + 1.0 / (1u64 << register_1715) as f32;

	let [register_1716, register_1717, register_1718, register_1719] = split_registers::<4>(words[429]);
	raw_estimate += 1.0 / (1u64 << register_1716) as f32 + 1.0 / (1u64 << register_1717) as f32 + 1.0 / (1u64 << register_1718) as f32 + 1.0 / (1u64 << register_1719) as f32;

	let [register_1720, register_1721, register_1722, register_1723] = split_registers::<4>(words[430]);
	raw_estimate += 1.0 / (1u64 << register_1720) as f32 + 1.0 / (1u64 << register_1721) as f32 + 1.0 / (1u64 << register_1722) as f32 + 1.0 / (1u64 << register_1723) as f32;

	let [register_1724, register_1725, register_1726, register_1727] = split_registers::<4>(words[431]);
	raw_estimate += 1.0 / (1u64 << register_1724) as f32 + 1.0 / (1u64 << register_1725) as f32 + 1.0 / (1u64 << register_1726) as f32 + 1.0 / (1u64 << register_1727) as f32;

	let [register_1728, register_1729, register_1730, register_1731] = split_registers::<4>(words[432]);
	raw_estimate += 1.0 / (1u64 << register_1728) as f32 + 1.0 / (1u64 << register_1729) as f32 + 1.0 / (1u64 << register_1730) as f32 + 1.0 / (1u64 << register_1731) as f32;

	let [register_1732, register_1733, register_1734, register_1735] = split_registers::<4>(words[433]);
	raw_estimate += 1.0 / (1u64 << register_1732) as f32 + 1.0 / (1u64 << register_1733) as f32 + 1.0 / (1u64 << register_1734) as f32 + 1.0 / (1u64 << register_1735) as f32;

	let [register_1736, register_1737, register_1738, register_1739] = split_registers::<4>(words[434]);
	raw_estimate += 1.0 / (1u64 << register_1736) as f32 + 1.0 / (1u64 << register_1737) as f32 + 1.0 / (1u64 << register_1738) as f32 + 1.0 / (1u64 << register_1739) as f32;

	let [register_1740, register_1741, register_1742, register_1743] = split_registers::<4>(words[435]);
	raw_estimate += 1.0 / (1u64 << register_1740) as f32 + 1.0 / (1u64 << register_1741) as f32 + 1.0 / (1u64 << register_1742) as f32 + 1.0 / (1u64 << register_1743) as f32;

	let [register_1744, register_1745, register_1746, register_1747] = split_registers::<4>(words[436]);
	raw_estimate += 1.0 / (1u64 << register_1744) as f32 + 1.0 / (1u64 << register_1745) as f32 + 1.0 / (1u64 << register_1746) as f32 + 1.0 / (1u64 << register_1747) as f32;

	let [register_1748, register_1749, register_1750, register_1751] = split_registers::<4>(words[437]);
	raw_estimate += 1.0 / (1u64 << register_1748) as f32 + 1.0 / (1u64 << register_1749) as f32 + 1.0 / (1u64 << register_1750) as f32 + 1.0 / (1u64 << register_1751) as f32;

	let [register_1752, register_1753, register_1754, register_1755] = split_registers::<4>(words[438]);
	raw_estimate += 1.0 / (1u64 << register_1752) as f32 + 1.0 / (1u64 << register_1753) as f32 + 1.0 / (1u64 << register_1754) as f32 + 1.0 / (1u64 << register_1755) as f32;

	let [register_1756, register_1757, register_1758, register_1759] = split_registers::<4>(words[439]);
	raw_estimate += 1.0 / (1u64 << register_1756) as f32 + 1.0 / (1u64 << register_1757) as f32 + 1.0 / (1u64 << register_1758) as f32 + 1.0 / (1u64 << register_1759) as f32;

	let [register_1760, register_1761, register_1762, register_1763] = split_registers::<4>(words[440]);
	raw_estimate += 1.0 / (1u64 << register_1760) as f32 + 1.0 / (1u64 << register_1761) as f32 + 1.0 / (1u64 << register_1762) as f32 + 1.0 / (1u64 << register_1763) as f32;

	let [register_1764, register_1765, register_1766, register_1767] = split_registers::<4>(words[441]);
	raw_estimate += 1.0 / (1u64 << register_1764) as f32 + 1.0 / (1u64 << register_1765) as f32 + 1.0 / (1u64 << register_1766) as f32 + 1.0 / (1u64 << register_1767) as f32;

	let [register_1768, register_1769, register_1770, register_1771] = split_registers::<4>(words[442]);
	raw_estimate += 1.0 / (1u64 << register_1768) as f32 + 1.0 / (1u64 << register_1769) as f32 + 1.0 / (1u64 << register_1770) as f32 + 1.0 / (1u64 << register_1771) as f32;

	let [register_1772, register_1773, register_1774, register_1775] = split_registers::<4>(words[443]);
	raw_estimate += 1.0 / (1u64 << register_1772) as f32 + 1.0 / (1u64 << register_1773) as f32 + 1.0 / (1u64 << register_1774) as f32 + 1.0 / (1u64 << register_1775) as f32;

	let [register_1776, register_1777, register_1778, register_1779] = split_registers::<4>(words[444]);
	raw_estimate += 1.0 / (1u64 << register_1776) as f32 + 1.0 / (1u64 << register_1777) as f32 + 1.0 / (1u64 << register_1778) as f32 + 1.0 / (1u64 << register_1779) as f32;

	let [register_1780, register_1781, register_1782, register_1783] = split_registers::<4>(words[445]);
	raw_estimate += 1.0 / (1u64 << register_1780) as f32 + 1.0 / (1u64 << register_1781) as f32 + 1.0 / (1u64 << register_1782) as f32 + 1.0 / (1u64 << register_1783) as f32;

	let [register_1784, register_1785, register_1786, register_1787] = split_registers::<4>(words[446]);
	raw_estimate += 1.0 / (1u64 << register_1784) as f32 + 1.0 / (1u64 << register_1785) as f32 + 1.0 / (1u64 << register_1786) as f32 + 1.0 / (1u64 << register_1787) as f32;

	let [register_1788, register_1789, register_1790, register_1791] = split_registers::<4>(words[447]);
	raw_estimate += 1.0 / (1u64 << register_1788) as f32 + 1.0 / (1u64 << register_1789) as f32 + 1.0 / (1u64 << register_1790) as f32 + 1.0 / (1u64 << register_1791) as f32;

	let [register_1792, register_1793, register_1794, register_1795] = split_registers::<4>(words[448]);
	raw_estimate += 1.0 / (1u64 << register_1792) as f32 + 1.0 / (1u64 << register_1793) as f32 + 1.0 / (1u64 << register_1794) as f32 + 1.0 / (1u64 << register_1795) as f32;

	let [register_1796, register_1797, register_1798, register_1799] = split_registers::<4>(words[449]);
	raw_estimate += 1.0 / (1u64 << register_1796) as f32 + 1.0 / (1u64 << register_1797) as f32 + 1.0 / (1u64 << register_1798) as f32 + 1.0 / (1u64 << register_1799) as f32;

	let [register_1800, register_1801, register_1802, register_1803] = split_registers::<4>(words[450]);
	raw_estimate += 1.0 / (1u64 << register_1800) as f32 + 1.0 / (1u64 << register_1801) as f32 + 1.0 / (1u64 << register_1802) as f32 + 1.0 / (1u64 << register_1803) as f32;

	let [register_1804, register_1805, register_1806, register_1807] = split_registers::<4>(words[451]);
	raw_estimate += 1.0 / (1u64 << register_1804) as f32 + 1.0 / (1u64 << register_1805) as f32 + 1.0 / (1u64 << register_1806) as f32 + 1.0 / (1u64 << register_1807) as f32;

	let [register_1808, register_1809, register_1810, register_1811] = split_registers::<4>(words[452]);
	raw_estimate += 1.0 / (1u64 << register_1808) as f32 + 1.0 / (1u64 << register_1809) as f32 + 1.0 / (1u64 << register_1810) as f32 + 1.0 / (1u64 << register_1811) as f32;

	let [register_1812, register_1813, register_1814, register_1815] = split_registers::<4>(words[453]);
	raw_estimate += 1.0 / (1u64 << register_1812) as f32 + 1.0 / (1u64 << register_1813) as f32 + 1.0 / (1u64 << register_1814) as f32 + 1.0 / (1u64 << register_1815) as f32;

	let [register_1816, register_1817, register_1818, register_1819] = split_registers::<4>(words[454]);
	raw_estimate += 1.0 / (1u64 << register_1816) as f32 + 1.0 / (1u64 << register_1817) as f32 + 1.0 / (1u64 << register_1818) as f32 + 1.0 / (1u64 << register_1819) as f32;

	let [register_1820, register_1821, register_1822, register_1823] = split_registers::<4>(words[455]);
	raw_estimate += 1.0 / (1u64 << register_1820) as f32 + 1.0 / (1u64 << register_1821) as f32 + 1.0 / (1u64 << register_1822) as f32 + 1.0 / (1u64 << register_1823) as f32;

	let [register_1824, register_1825, register_1826, register_1827] = split_registers::<4>(words[456]);
	raw_estimate += 1.0 / (1u64 << register_1824) as f32 + 1.0 / (1u64 << register_1825) as f32 + 1.0 / (1u64 << register_1826) as f32 + 1.0 / (1u64 << register_1827) as f32;

	let [register_1828, register_1829, register_1830, register_1831] = split_registers::<4>(words[457]);
	raw_estimate += 1.0 / (1u64 << register_1828) as f32 + 1.0 / (1u64 << register_1829) as f32 + 1.0 / (1u64 << register_1830) as f32 + 1.0 / (1u64 << register_1831) as f32;

	let [register_1832, register_1833, register_1834, register_1835] = split_registers::<4>(words[458]);
	raw_estimate += 1.0 / (1u64 << register_1832) as f32 + 1.0 / (1u64 << register_1833) as f32 + 1.0 / (1u64 << register_1834) as f32 + 1.0 / (1u64 << register_1835) as f32;

	let [register_1836, register_1837, register_1838, register_1839] = split_registers::<4>(words[459]);
	raw_estimate += 1.0 / (1u64 << register_1836) as f32 + 1.0 / (1u64 << register_1837) as f32 + 1.0 / (1u64 << register_1838) as f32 + 1.0 / (1u64 << register_1839) as f32;

	let [register_1840, register_1841, register_1842, register_1843] = split_registers::<4>(words[460]);
	raw_estimate += 1.0 / (1u64 << register_1840) as f32 + 1.0 / (1u64 << register_1841) as f32 + 1.0 / (1u64 << register_1842) as f32 + 1.0 / (1u64 << register_1843) as f32;

	let [register_1844, register_1845, register_1846, register_1847] = split_registers::<4>(words[461]);
	raw_estimate += 1.0 / (1u64 << register_1844) as f32 + 1.0 / (1u64 << register_1845) as f32 + 1.0 / (1u64 << register_1846) as f32 + 1.0 / (1u64 << register_1847) as f32;

	let [register_1848, register_1849, register_1850, register_1851] = split_registers::<4>(words[462]);
	raw_estimate += 1.0 / (1u64 << register_1848) as f32 + 1.0 / (1u64 << register_1849) as f32 + 1.0 / (1u64 << register_1850) as f32 + 1.0 / (1u64 << register_1851) as f32;

	let [register_1852, register_1853, register_1854, register_1855] = split_registers::<4>(words[463]);
	raw_estimate += 1.0 / (1u64 << register_1852) as f32 + 1.0 / (1u64 << register_1853) as f32 + 1.0 / (1u64 << register_1854) as f32 + 1.0 / (1u64 << register_1855) as f32;

	let [register_1856, register_1857, register_1858, register_1859] = split_registers::<4>(words[464]);
	raw_estimate += 1.0 / (1u64 << register_1856) as f32 + 1.0 / (1u64 << register_1857) as f32 + 1.0 / (1u64 << register_1858) as f32 + 1.0 / (1u64 << register_1859) as f32;

	let [register_1860, register_1861, register_1862, register_1863] = split_registers::<4>(words[465]);
	raw_estimate += 1.0 / (1u64 << register_1860) as f32 + 1.0 / (1u64 << register_1861) as f32 + 1.0 / (1u64 << register_1862) as f32 + 1.0 / (1u64 << register_1863) as f32;

	let [register_1864, register_1865, register_1866, register_1867] = split_registers::<4>(words[466]);
	raw_estimate += 1.0 / (1u64 << register_1864) as f32 + 1.0 / (1u64 << register_1865) as f32 + 1.0 / (1u64 << register_1866) as f32 + 1.0 / (1u64 << register_1867) as f32;

	let [register_1868, register_1869, register_1870, register_1871] = split_registers::<4>(words[467]);
	raw_estimate += 1.0 / (1u64 << register_1868) as f32 + 1.0 / (1u64 << register_1869) as f32 + 1.0 / (1u64 << register_1870) as f32 + 1.0 / (1u64 << register_1871) as f32;

	let [register_1872, register_1873, register_1874, register_1875] = split_registers::<4>(words[468]);
	raw_estimate += 1.0 / (1u64 << register_1872) as f32 + 1.0 / (1u64 << register_1873) as f32 + 1.0 / (1u64 << register_1874) as f32 + 1.0 / (1u64 << register_1875) as f32;

	let [register_1876, register_1877, register_1878, register_1879] = split_registers::<4>(words[469]);
	raw_estimate += 1.0 / (1u64 << register_1876) as f32 + 1.0 / (1u64 << register_1877) as f32 + 1.0 / (1u64 << register_1878) as f32 + 1.0 / (1u64 << register_1879) as f32;

	let [register_1880, register_1881, register_1882, register_1883] = split_registers::<4>(words[470]);
	raw_estimate += 1.0 / (1u64 << register_1880) as f32 + 1.0 / (1u64 << register_1881) as f32 + 1.0 / (1u64 << register_1882) as f32 + 1.0 / (1u64 << register_1883) as f32;

	let [register_1884, register_1885, register_1886, register_1887] = split_registers::<4>(words[471]);
	raw_estimate += 1.0 / (1u64 << register_1884) as f32 + 1.0 / (1u64 << register_1885) as f32 + 1.0 / (1u64 << register_1886) as f32 + 1.0 / (1u64 << register_1887) as f32;

	let [register_1888, register_1889, register_1890, register_1891] = split_registers::<4>(words[472]);
	raw_estimate += 1.0 / (1u64 << register_1888) as f32 + 1.0 / (1u64 << register_1889) as f32 + 1.0 / (1u64 << register_1890) as f32 + 1.0 / (1u64 << register_1891) as f32;

	let [register_1892, register_1893, register_1894, register_1895] = split_registers::<4>(words[473]);
	raw_estimate += 1.0 / (1u64 << register_1892) as f32 + 1.0 / (1u64 << register_1893) as f32 + 1.0 / (1u64 << register_1894) as f32 + 1.0 / (1u64 << register_1895) as f32;

	let [register_1896, register_1897, register_1898, register_1899] = split_registers::<4>(words[474]);
	raw_estimate += 1.0 / (1u64 << register_1896) as f32 + 1.0 / (1u64 << register_1897) as f32 + 1.0 / (1u64 << register_1898) as f32 + 1.0 / (1u64 << register_1899) as f32;

	let [register_1900, register_1901, register_1902, register_1903] = split_registers::<4>(words[475]);
	raw_estimate += 1.0 / (1u64 << register_1900) as f32 + 1.0 / (1u64 << register_1901) as f32 + 1.0 / (1u64 << register_1902) as f32 + 1.0 / (1u64 << register_1903) as f32;

	let [register_1904, register_1905, register_1906, register_1907] = split_registers::<4>(words[476]);
	raw_estimate += 1.0 / (1u64 << register_1904) as f32 + 1.0 / (1u64 << register_1905) as f32 + 1.0 / (1u64 << register_1906) as f32 + 1.0 / (1u64 << register_1907) as f32;

	let [register_1908, register_1909, register_1910, register_1911] = split_registers::<4>(words[477]);
	raw_estimate += 1.0 / (1u64 << register_1908) as f32 + 1.0 / (1u64 << register_1909) as f32 + 1.0 / (1u64 << register_1910) as f32 + 1.0 / (1u64 << register_1911) as f32;

	let [register_1912, register_1913, register_1914, register_1915] = split_registers::<4>(words[478]);
	raw_estimate += 1.0 / (1u64 << register_1912) as f32 + 1.0 / (1u64 << register_1913) as f32 + 1.0 / (1u64 << register_1914) as f32 + 1.0 / (1u64 << register_1915) as f32;

	let [register_1916, register_1917, register_1918, register_1919] = split_registers::<4>(words[479]);
	raw_estimate += 1.0 / (1u64 << register_1916) as f32 + 1.0 / (1u64 << register_1917) as f32 + 1.0 / (1u64 << register_1918) as f32 + 1.0 / (1u64 << register_1919) as f32;

	let [register_1920, register_1921, register_1922, register_1923] = split_registers::<4>(words[480]);
	raw_estimate += 1.0 / (1u64 << register_1920) as f32 + 1.0 / (1u64 << register_1921) as f32 + 1.0 / (1u64 << register_1922) as f32 + 1.0 / (1u64 << register_1923) as f32;

	let [register_1924, register_1925, register_1926, register_1927] = split_registers::<4>(words[481]);
	raw_estimate += 1.0 / (1u64 << register_1924) as f32 + 1.0 / (1u64 << register_1925) as f32 + 1.0 / (1u64 << register_1926) as f32 + 1.0 / (1u64 << register_1927) as f32;

	let [register_1928, register_1929, register_1930, register_1931] = split_registers::<4>(words[482]);
	raw_estimate += 1.0 / (1u64 << register_1928) as f32 + 1.0 / (1u64 << register_1929) as f32 + 1.0 / (1u64 << register_1930) as f32 + 1.0 / (1u64 << register_1931) as f32;

	let [register_1932, register_1933, register_1934, register_1935] = split_registers::<4>(words[483]);
	raw_estimate += 1.0 / (1u64 << register_1932) as f32 + 1.0 / (1u64 << register_1933) as f32 + 1.0 / (1u64 << register_1934) as f32 + 1.0 / (1u64 << register_1935) as f32;

	let [register_1936, register_1937, register_1938, register_1939] = split_registers::<4>(words[484]);
	raw_estimate += 1.0 / (1u64 << register_1936) as f32 + 1.0 / (1u64 << register_1937) as f32 + 1.0 / (1u64 << register_1938) as f32 + 1.0 / (1u64 << register_1939) as f32;

	let [register_1940, register_1941, register_1942, register_1943] = split_registers::<4>(words[485]);
	raw_estimate += 1.0 / (1u64 << register_1940) as f32 + 1.0 / (1u64 << register_1941) as f32 + 1.0 / (1u64 << register_1942) as f32 + 1.0 / (1u64 << register_1943) as f32;

	let [register_1944, register_1945, register_1946, register_1947] = split_registers::<4>(words[486]);
	raw_estimate += 1.0 / (1u64 << register_1944) as f32 + 1.0 / (1u64 << register_1945) as f32 + 1.0 / (1u64 << register_1946) as f32 + 1.0 / (1u64 << register_1947) as f32;

	let [register_1948, register_1949, register_1950, register_1951] = split_registers::<4>(words[487]);
	raw_estimate += 1.0 / (1u64 << register_1948) as f32 + 1.0 / (1u64 << register_1949) as f32 + 1.0 / (1u64 << register_1950) as f32 + 1.0 / (1u64 << register_1951) as f32;

	let [register_1952, register_1953, register_1954, register_1955] = split_registers::<4>(words[488]);
	raw_estimate += 1.0 / (1u64 << register_1952) as f32 + 1.0 / (1u64 << register_1953) as f32 + 1.0 / (1u64 << register_1954) as f32 + 1.0 / (1u64 << register_1955) as f32;

	let [register_1956, register_1957, register_1958, register_1959] = split_registers::<4>(words[489]);
	raw_estimate += 1.0 / (1u64 << register_1956) as f32 + 1.0 / (1u64 << register_1957) as f32 + 1.0 / (1u64 << register_1958) as f32 + 1.0 / (1u64 << register_1959) as f32;

	let [register_1960, register_1961, register_1962, register_1963] = split_registers::<4>(words[490]);
	raw_estimate += 1.0 / (1u64 << register_1960) as f32 + 1.0 / (1u64 << register_1961) as f32 + 1.0 / (1u64 << register_1962) as f32 + 1.0 / (1u64 << register_1963) as f32;

	let [register_1964, register_1965, register_1966, register_1967] = split_registers::<4>(words[491]);
	raw_estimate += 1.0 / (1u64 << register_1964) as f32 + 1.0 / (1u64 << register_1965) as f32 + 1.0 / (1u64 << register_1966) as f32 + 1.0 / (1u64 << register_1967) as f32;

	let [register_1968, register_1969, register_1970, register_1971] = split_registers::<4>(words[492]);
	raw_estimate += 1.0 / (1u64 << register_1968) as f32 + 1.0 / (1u64 << register_1969) as f32 + 1.0 / (1u64 << register_1970) as f32 + 1.0 / (1u64 << register_1971) as f32;

	let [register_1972, register_1973, register_1974, register_1975] = split_registers::<4>(words[493]);
	raw_estimate += 1.0 / (1u64 << register_1972) as f32 + 1.0 / (1u64 << register_1973) as f32 + 1.0 / (1u64 << register_1974) as f32 + 1.0 / (1u64 << register_1975) as f32;

	let [register_1976, register_1977, register_1978, register_1979] = split_registers::<4>(words[494]);
	raw_estimate += 1.0 / (1u64 << register_1976) as f32 + 1.0 / (1u64 << register_1977) as f32 + 1.0 / (1u64 << register_1978) as f32 + 1.0 / (1u64 << register_1979) as f32;

	let [register_1980, register_1981, register_1982, register_1983] = split_registers::<4>(words[495]);
	raw_estimate += 1.0 / (1u64 << register_1980) as f32 + 1.0 / (1u64 << register_1981) as f32 + 1.0 / (1u64 << register_1982) as f32 + 1.0 / (1u64 << register_1983) as f32;

	let [register_1984, register_1985, register_1986, register_1987] = split_registers::<4>(words[496]);
	raw_estimate += 1.0 / (1u64 << register_1984) as f32 + 1.0 / (1u64 << register_1985) as f32 + 1.0 / (1u64 << register_1986) as f32 + 1.0 / (1u64 << register_1987) as f32;

	let [register_1988, register_1989, register_1990, register_1991] = split_registers::<4>(words[497]);
	raw_estimate += 1.0 / (1u64 << register_1988) as f32 + 1.0 / (1u64 << register_1989) as f32 + 1.0 / (1u64 << register_1990) as f32 + 1.0 / (1u64 << register_1991) as f32;

	let [register_1992, register_1993, register_1994, register_1995] = split_registers::<4>(words[498]);
	raw_estimate += 1.0 / (1u64 << register_1992) as f32 + 1.0 / (1u64 << register_1993) as f32 + 1.0 / (1u64 << register_1994) as f32 + 1.0 / (1u64 << register_1995) as f32;

	let [register_1996, register_1997, register_1998, register_1999] = split_registers::<4>(words[499]);
	raw_estimate += 1.0 / (1u64 << register_1996) as f32 + 1.0 / (1u64 << register_1997) as f32 + 1.0 / (1u64 << register_1998) as f32 + 1.0 / (1u64 << register_1999) as f32;

	let [register_2000, register_2001, register_2002, register_2003] = split_registers::<4>(words[500]);
	raw_estimate += 1.0 / (1u64 << register_2000) as f32 + 1.0 / (1u64 << register_2001) as f32 + 1.0 / (1u64 << register_2002) as f32 + 1.0 / (1u64 << register_2003) as f32;

	let [register_2004, register_2005, register_2006, register_2007] = split_registers::<4>(words[501]);
	raw_estimate += 1.0 / (1u64 << register_2004) as f32 + 1.0 / (1u64 << register_2005) as f32 + 1.0 / (1u64 << register_2006) as f32 + 1.0 / (1u64 << register_2007) as f32;

	let [register_2008, register_2009, register_2010, register_2011] = split_registers::<4>(words[502]);
	raw_estimate += 1.0 / (1u64 << register_2008) as f32 + 1.0 / (1u64 << register_2009) as f32 + 1.0 / (1u64 << register_2010) as f32 + 1.0 / (1u64 << register_2011) as f32;

	let [register_2012, register_2013, register_2014, register_2015] = split_registers::<4>(words[503]);
	raw_estimate += 1.0 / (1u64 << register_2012) as f32 + 1.0 / (1u64 << register_2013) as f32 + 1.0 / (1u64 << register_2014) as f32 + 1.0 / (1u64 << register_2015) as f32;

	let [register_2016, register_2017, register_2018, register_2019] = split_registers::<4>(words[504]);
	raw_estimate += 1.0 / (1u64 << register_2016) as f32 + 1.0 / (1u64 << register_2017) as f32 + 1.0 / (1u64 << register_2018) as f32 + 1.0 / (1u64 << register_2019) as f32;

	let [register_2020, register_2021, register_2022, register_2023] = split_registers::<4>(words[505]);
	raw_estimate += 1.0 / (1u64 << register_2020) as f32 + 1.0 / (1u64 << register_2021) as f32 + 1.0 / (1u64 << register_2022) as f32 + 1.0 / (1u64 << register_2023) as f32;

	let [register_2024, register_2025, register_2026, register_2027] = split_registers::<4>(words[506]);
	raw_estimate += 1.0 / (1u64 << register_2024) as f32 + 1.0 / (1u64 << register_2025) as f32 + 1.0 / (1u64 << register_2026) as f32 + 1.0 / (1u64 << register_2027) as f32;

	let [register_2028, register_2029, register_2030, register_2031] = split_registers::<4>(words[507]);
	raw_estimate += 1.0 / (1u64 << register_2028) as f32 + 1.0 / (1u64 << register_2029) as f32 + 1.0 / (1u64 << register_2030) as f32 + 1.0 / (1u64 << register_2031) as f32;

	let [register_2032, register_2033, register_2034, register_2035] = split_registers::<4>(words[508]);
	raw_estimate += 1.0 / (1u64 << register_2032) as f32 + 1.0 / (1u64 << register_2033) as f32 + 1.0 / (1u64 << register_2034) as f32 + 1.0 / (1u64 << register_2035) as f32;

	let [register_2036, register_2037, register_2038, register_2039] = split_registers::<4>(words[509]);
	raw_estimate += 1.0 / (1u64 << register_2036) as f32 + 1.0 / (1u64 << register_2037) as f32 + 1.0 / (1u64 << register_2038) as f32 + 1.0 / (1u64 << register_2039) as f32;

	let [register_2040, register_2041, register_2042, register_2043] = split_registers::<4>(words[510]);
	raw_estimate += 1.0 / (1u64 << register_2040) as f32 + 1.0 / (1u64 << register_2041) as f32 + 1.0 / (1u64 << register_2042) as f32 + 1.0 / (1u64 << register_2043) as f32;

	let [register_2044, register_2045, register_2046, register_2047] = split_registers::<4>(words[511]);
	raw_estimate += 1.0 / (1u64 << register_2044) as f32 + 1.0 / (1u64 << register_2045) as f32 + 1.0 / (1u64 << register_2046) as f32 + 1.0 / (1u64 << register_2047) as f32;

	let [register_2048, register_2049, register_2050, register_2051] = split_registers::<4>(words[512]);
	raw_estimate += 1.0 / (1u64 << register_2048) as f32 + 1.0 / (1u64 << register_2049) as f32 + 1.0 / (1u64 << register_2050) as f32 + 1.0 / (1u64 << register_2051) as f32;

	let [register_2052, register_2053, register_2054, register_2055] = split_registers::<4>(words[513]);
	raw_estimate += 1.0 / (1u64 << register_2052) as f32 + 1.0 / (1u64 << register_2053) as f32 + 1.0 / (1u64 << register_2054) as f32 + 1.0 / (1u64 << register_2055) as f32;

	let [register_2056, register_2057, register_2058, register_2059] = split_registers::<4>(words[514]);
	raw_estimate += 1.0 / (1u64 << register_2056) as f32 + 1.0 / (1u64 << register_2057) as f32 + 1.0 / (1u64 << register_2058) as f32 + 1.0 / (1u64 << register_2059) as f32;

	let [register_2060, register_2061, register_2062, register_2063] = split_registers::<4>(words[515]);
	raw_estimate += 1.0 / (1u64 << register_2060) as f32 + 1.0 / (1u64 << register_2061) as f32 + 1.0 / (1u64 << register_2062) as f32 + 1.0 / (1u64 << register_2063) as f32;

	let [register_2064, register_2065, register_2066, register_2067] = split_registers::<4>(words[516]);
	raw_estimate += 1.0 / (1u64 << register_2064) as f32 + 1.0 / (1u64 << register_2065) as f32 + 1.0 / (1u64 << register_2066) as f32 + 1.0 / (1u64 << register_2067) as f32;

	let [register_2068, register_2069, register_2070, register_2071] = split_registers::<4>(words[517]);
	raw_estimate += 1.0 / (1u64 << register_2068) as f32 + 1.0 / (1u64 << register_2069) as f32 + 1.0 / (1u64 << register_2070) as f32 + 1.0 / (1u64 << register_2071) as f32;

	let [register_2072, register_2073, register_2074, register_2075] = split_registers::<4>(words[518]);
	raw_estimate += 1.0 / (1u64 << register_2072) as f32 + 1.0 / (1u64 << register_2073) as f32 + 1.0 / (1u64 << register_2074) as f32 + 1.0 / (1u64 << register_2075) as f32;

	let [register_2076, register_2077, register_2078, register_2079] = split_registers::<4>(words[519]);
	raw_estimate += 1.0 / (1u64 << register_2076) as f32 + 1.0 / (1u64 << register_2077) as f32 + 1.0 / (1u64 << register_2078) as f32 + 1.0 / (1u64 << register_2079) as f32;

	let [register_2080, register_2081, register_2082, register_2083] = split_registers::<4>(words[520]);
	raw_estimate += 1.0 / (1u64 << register_2080) as f32 + 1.0 / (1u64 << register_2081) as f32 + 1.0 / (1u64 << register_2082) as f32 + 1.0 / (1u64 << register_2083) as f32;

	let [register_2084, register_2085, register_2086, register_2087] = split_registers::<4>(words[521]);
	raw_estimate += 1.0 / (1u64 << register_2084) as f32 + 1.0 / (1u64 << register_2085) as f32 + 1.0 / (1u64 << register_2086) as f32 + 1.0 / (1u64 << register_2087) as f32;

	let [register_2088, register_2089, register_2090, register_2091] = split_registers::<4>(words[522]);
	raw_estimate += 1.0 / (1u64 << register_2088) as f32 + 1.0 / (1u64 << register_2089) as f32 + 1.0 / (1u64 << register_2090) as f32 + 1.0 / (1u64 << register_2091) as f32;

	let [register_2092, register_2093, register_2094, register_2095] = split_registers::<4>(words[523]);
	raw_estimate += 1.0 / (1u64 << register_2092) as f32 + 1.0 / (1u64 << register_2093) as f32 + 1.0 / (1u64 << register_2094) as f32 + 1.0 / (1u64 << register_2095) as f32;

	let [register_2096, register_2097, register_2098, register_2099] = split_registers::<4>(words[524]);
	raw_estimate += 1.0 / (1u64 << register_2096) as f32 + 1.0 / (1u64 << register_2097) as f32 + 1.0 / (1u64 << register_2098) as f32 + 1.0 / (1u64 << register_2099) as f32;

	let [register_2100, register_2101, register_2102, register_2103] = split_registers::<4>(words[525]);
	raw_estimate += 1.0 / (1u64 << register_2100) as f32 + 1.0 / (1u64 << register_2101) as f32 + 1.0 / (1u64 << register_2102) as f32 + 1.0 / (1u64 << register_2103) as f32;

	let [register_2104, register_2105, register_2106, register_2107] = split_registers::<4>(words[526]);
	raw_estimate += 1.0 / (1u64 << register_2104) as f32 + 1.0 / (1u64 << register_2105) as f32 + 1.0 / (1u64 << register_2106) as f32 + 1.0 / (1u64 << register_2107) as f32;

	let [register_2108, register_2109, register_2110, register_2111] = split_registers::<4>(words[527]);
	raw_estimate += 1.0 / (1u64 << register_2108) as f32 + 1.0 / (1u64 << register_2109) as f32 + 1.0 / (1u64 << register_2110) as f32 + 1.0 / (1u64 << register_2111) as f32;

	let [register_2112, register_2113, register_2114, register_2115] = split_registers::<4>(words[528]);
	raw_estimate += 1.0 / (1u64 << register_2112) as f32 + 1.0 / (1u64 << register_2113) as f32 + 1.0 / (1u64 << register_2114) as f32 + 1.0 / (1u64 << register_2115) as f32;

	let [register_2116, register_2117, register_2118, register_2119] = split_registers::<4>(words[529]);
	raw_estimate += 1.0 / (1u64 << register_2116) as f32 + 1.0 / (1u64 << register_2117) as f32 + 1.0 / (1u64 << register_2118) as f32 + 1.0 / (1u64 << register_2119) as f32;

	let [register_2120, register_2121, register_2122, register_2123] = split_registers::<4>(words[530]);
	raw_estimate += 1.0 / (1u64 << register_2120) as f32 + 1.0 / (1u64 << register_2121) as f32 + 1.0 / (1u64 << register_2122) as f32 + 1.0 / (1u64 << register_2123) as f32;

	let [register_2124, register_2125, register_2126, register_2127] = split_registers::<4>(words[531]);
	raw_estimate += 1.0 / (1u64 << register_2124) as f32 + 1.0 / (1u64 << register_2125) as f32 + 1.0 / (1u64 << register_2126) as f32 + 1.0 / (1u64 << register_2127) as f32;

	let [register_2128, register_2129, register_2130, register_2131] = split_registers::<4>(words[532]);
	raw_estimate += 1.0 / (1u64 << register_2128) as f32 + 1.0 / (1u64 << register_2129) as f32 + 1.0 / (1u64 << register_2130) as f32 + 1.0 / (1u64 << register_2131) as f32;

	let [register_2132, register_2133, register_2134, register_2135] = split_registers::<4>(words[533]);
	raw_estimate += 1.0 / (1u64 << register_2132) as f32 + 1.0 / (1u64 << register_2133) as f32 + 1.0 / (1u64 << register_2134) as f32 + 1.0 / (1u64 << register_2135) as f32;

	let [register_2136, register_2137, register_2138, register_2139] = split_registers::<4>(words[534]);
	raw_estimate += 1.0 / (1u64 << register_2136) as f32 + 1.0 / (1u64 << register_2137) as f32 + 1.0 / (1u64 << register_2138) as f32 + 1.0 / (1u64 << register_2139) as f32;

	let [register_2140, register_2141, register_2142, register_2143] = split_registers::<4>(words[535]);
	raw_estimate += 1.0 / (1u64 << register_2140) as f32 + 1.0 / (1u64 << register_2141) as f32 + 1.0 / (1u64 << register_2142) as f32 + 1.0 / (1u64 << register_2143) as f32;

	let [register_2144, register_2145, register_2146, register_2147] = split_registers::<4>(words[536]);
	raw_estimate += 1.0 / (1u64 << register_2144) as f32 + 1.0 / (1u64 << register_2145) as f32 + 1.0 / (1u64 << register_2146) as f32 + 1.0 / (1u64 << register_2147) as f32;

	let [register_2148, register_2149, register_2150, register_2151] = split_registers::<4>(words[537]);
	raw_estimate += 1.0 / (1u64 << register_2148) as f32 + 1.0 / (1u64 << register_2149) as f32 + 1.0 / (1u64 << register_2150) as f32 + 1.0 / (1u64 << register_2151) as f32;

	let [register_2152, register_2153, register_2154, register_2155] = split_registers::<4>(words[538]);
	raw_estimate += 1.0 / (1u64 << register_2152) as f32 + 1.0 / (1u64 << register_2153) as f32 + 1.0 / (1u64 << register_2154) as f32 + 1.0 / (1u64 << register_2155) as f32;

	let [register_2156, register_2157, register_2158, register_2159] = split_registers::<4>(words[539]);
	raw_estimate += 1.0 / (1u64 << register_2156) as f32 + 1.0 / (1u64 << register_2157) as f32 + 1.0 / (1u64 << register_2158) as f32 + 1.0 / (1u64 << register_2159) as f32;

	let [register_2160, register_2161, register_2162, register_2163] = split_registers::<4>(words[540]);
	raw_estimate += 1.0 / (1u64 << register_2160) as f32 + 1.0 / (1u64 << register_2161) as f32 + 1.0 / (1u64 << register_2162) as f32 + 1.0 / (1u64 << register_2163) as f32;

	let [register_2164, register_2165, register_2166, register_2167] = split_registers::<4>(words[541]);
	raw_estimate += 1.0 / (1u64 << register_2164) as f32 + 1.0 / (1u64 << register_2165) as f32 + 1.0 / (1u64 << register_2166) as f32 + 1.0 / (1u64 << register_2167) as f32;

	let [register_2168, register_2169, register_2170, register_2171] = split_registers::<4>(words[542]);
	raw_estimate += 1.0 / (1u64 << register_2168) as f32 + 1.0 / (1u64 << register_2169) as f32 + 1.0 / (1u64 << register_2170) as f32 + 1.0 / (1u64 << register_2171) as f32;

	let [register_2172, register_2173, register_2174, register_2175] = split_registers::<4>(words[543]);
	raw_estimate += 1.0 / (1u64 << register_2172) as f32 + 1.0 / (1u64 << register_2173) as f32 + 1.0 / (1u64 << register_2174) as f32 + 1.0 / (1u64 << register_2175) as f32;

	let [register_2176, register_2177, register_2178, register_2179] = split_registers::<4>(words[544]);
	raw_estimate += 1.0 / (1u64 << register_2176) as f32 + 1.0 / (1u64 << register_2177) as f32 + 1.0 / (1u64 << register_2178) as f32 + 1.0 / (1u64 << register_2179) as f32;

	let [register_2180, register_2181, register_2182, register_2183] = split_registers::<4>(words[545]);
	raw_estimate += 1.0 / (1u64 << register_2180) as f32 + 1.0 / (1u64 << register_2181) as f32 + 1.0 / (1u64 << register_2182) as f32 + 1.0 / (1u64 << register_2183) as f32;

	let [register_2184, register_2185, register_2186, register_2187] = split_registers::<4>(words[546]);
	raw_estimate += 1.0 / (1u64 << register_2184) as f32 + 1.0 / (1u64 << register_2185) as f32 + 1.0 / (1u64 << register_2186) as f32 + 1.0 / (1u64 << register_2187) as f32;

	let [register_2188, register_2189, register_2190, register_2191] = split_registers::<4>(words[547]);
	raw_estimate += 1.0 / (1u64 << register_2188) as f32 + 1.0 / (1u64 << register_2189) as f32 + 1.0 / (1u64 << register_2190) as f32 + 1.0 / (1u64 << register_2191) as f32;

	let [register_2192, register_2193, register_2194, register_2195] = split_registers::<4>(words[548]);
	raw_estimate += 1.0 / (1u64 << register_2192) as f32 + 1.0 / (1u64 << register_2193) as f32 + 1.0 / (1u64 << register_2194) as f32 + 1.0 / (1u64 << register_2195) as f32;

	let [register_2196, register_2197, register_2198, register_2199] = split_registers::<4>(words[549]);
	raw_estimate += 1.0 / (1u64 << register_2196) as f32 + 1.0 / (1u64 << register_2197) as f32 + 1.0 / (1u64 << register_2198) as f32 + 1.0 / (1u64 << register_2199) as f32;

	let [register_2200, register_2201, register_2202, register_2203] = split_registers::<4>(words[550]);
	raw_estimate += 1.0 / (1u64 << register_2200) as f32 + 1.0 / (1u64 << register_2201) as f32 + 1.0 / (1u64 << register_2202) as f32 + 1.0 / (1u64 << register_2203) as f32;

	let [register_2204, register_2205, register_2206, register_2207] = split_registers::<4>(words[551]);
	raw_estimate += 1.0 / (1u64 << register_2204) as f32 + 1.0 / (1u64 << register_2205) as f32 + 1.0 / (1u64 << register_2206) as f32 + 1.0 / (1u64 << register_2207) as f32;

	let [register_2208, register_2209, register_2210, register_2211] = split_registers::<4>(words[552]);
	raw_estimate += 1.0 / (1u64 << register_2208) as f32 + 1.0 / (1u64 << register_2209) as f32 + 1.0 / (1u64 << register_2210) as f32 + 1.0 / (1u64 << register_2211) as f32;

	let [register_2212, register_2213, register_2214, register_2215] = split_registers::<4>(words[553]);
	raw_estimate += 1.0 / (1u64 << register_2212) as f32 + 1.0 / (1u64 << register_2213) as f32 + 1.0 / (1u64 << register_2214) as f32 + 1.0 / (1u64 << register_2215) as f32;

	let [register_2216, register_2217, register_2218, register_2219] = split_registers::<4>(words[554]);
	raw_estimate += 1.0 / (1u64 << register_2216) as f32 + 1.0 / (1u64 << register_2217) as f32 + 1.0 / (1u64 << register_2218) as f32 + 1.0 / (1u64 << register_2219) as f32;

	let [register_2220, register_2221, register_2222, register_2223] = split_registers::<4>(words[555]);
	raw_estimate += 1.0 / (1u64 << register_2220) as f32 + 1.0 / (1u64 << register_2221) as f32 + 1.0 / (1u64 << register_2222) as f32 + 1.0 / (1u64 << register_2223) as f32;

	let [register_2224, register_2225, register_2226, register_2227] = split_registers::<4>(words[556]);
	raw_estimate += 1.0 / (1u64 << register_2224) as f32 + 1.0 / (1u64 << register_2225) as f32 + 1.0 / (1u64 << register_2226) as f32 + 1.0 / (1u64 << register_2227) as f32;

	let [register_2228, register_2229, register_2230, register_2231] = split_registers::<4>(words[557]);
	raw_estimate += 1.0 / (1u64 << register_2228) as f32 + 1.0 / (1u64 << register_2229) as f32 + 1.0 / (1u64 << register_2230) as f32 + 1.0 / (1u64 << register_2231) as f32;

	let [register_2232, register_2233, register_2234, register_2235] = split_registers::<4>(words[558]);
	raw_estimate += 1.0 / (1u64 << register_2232) as f32 + 1.0 / (1u64 << register_2233) as f32 + 1.0 / (1u64 << register_2234) as f32 + 1.0 / (1u64 << register_2235) as f32;

	let [register_2236, register_2237, register_2238, register_2239] = split_registers::<4>(words[559]);
	raw_estimate += 1.0 / (1u64 << register_2236) as f32 + 1.0 / (1u64 << register_2237) as f32 + 1.0 / (1u64 << register_2238) as f32 + 1.0 / (1u64 << register_2239) as f32;

	let [register_2240, register_2241, register_2242, register_2243] = split_registers::<4>(words[560]);
	raw_estimate += 1.0 / (1u64 << register_2240) as f32 + 1.0 / (1u64 << register_2241) as f32 + 1.0 / (1u64 << register_2242) as f32 + 1.0 / (1u64 << register_2243) as f32;

	let [register_2244, register_2245, register_2246, register_2247] = split_registers::<4>(words[561]);
	raw_estimate += 1.0 / (1u64 << register_2244) as f32 + 1.0 / (1u64 << register_2245) as f32 + 1.0 / (1u64 << register_2246) as f32 + 1.0 / (1u64 << register_2247) as f32;

	let [register_2248, register_2249, register_2250, register_2251] = split_registers::<4>(words[562]);
	raw_estimate += 1.0 / (1u64 << register_2248) as f32 + 1.0 / (1u64 << register_2249) as f32 + 1.0 / (1u64 << register_2250) as f32 + 1.0 / (1u64 << register_2251) as f32;

	let [register_2252, register_2253, register_2254, register_2255] = split_registers::<4>(words[563]);
	raw_estimate += 1.0 / (1u64 << register_2252) as f32 + 1.0 / (1u64 << register_2253) as f32 + 1.0 / (1u64 << register_2254) as f32 + 1.0 / (1u64 << register_2255) as f32;

	let [register_2256, register_2257, register_2258, register_2259] = split_registers::<4>(words[564]);
	raw_estimate += 1.0 / (1u64 << register_2256) as f32 + 1.0 / (1u64 << register_2257) as f32 + 1.0 / (1u64 << register_2258) as f32 + 1.0 / (1u64 << register_2259) as f32;

	let [register_2260, register_2261, register_2262, register_2263] = split_registers::<4>(words[565]);
	raw_estimate += 1.0 / (1u64 << register_2260) as f32 + 1.0 / (1u64 << register_2261) as f32 + 1.0 / (1u64 << register_2262) as f32 + 1.0 / (1u64 << register_2263) as f32;

	let [register_2264, register_2265, register_2266, register_2267] = split_registers::<4>(words[566]);
	raw_estimate += 1.0 / (1u64 << register_2264) as f32 + 1.0 / (1u64 << register_2265) as f32 + 1.0 / (1u64 << register_2266) as f32 + 1.0 / (1u64 << register_2267) as f32;

	let [register_2268, register_2269, register_2270, register_2271] = split_registers::<4>(words[567]);
	raw_estimate += 1.0 / (1u64 << register_2268) as f32 + 1.0 / (1u64 << register_2269) as f32 + 1.0 / (1u64 << register_2270) as f32 + 1.0 / (1u64 << register_2271) as f32;

	let [register_2272, register_2273, register_2274, register_2275] = split_registers::<4>(words[568]);
	raw_estimate += 1.0 / (1u64 << register_2272) as f32 + 1.0 / (1u64 << register_2273) as f32 + 1.0 / (1u64 << register_2274) as f32 + 1.0 / (1u64 << register_2275) as f32;

	let [register_2276, register_2277, register_2278, register_2279] = split_registers::<4>(words[569]);
	raw_estimate += 1.0 / (1u64 << register_2276) as f32 + 1.0 / (1u64 << register_2277) as f32 + 1.0 / (1u64 << register_2278) as f32 + 1.0 / (1u64 << register_2279) as f32;

	let [register_2280, register_2281, register_2282, register_2283] = split_registers::<4>(words[570]);
	raw_estimate += 1.0 / (1u64 << register_2280) as f32 + 1.0 / (1u64 << register_2281) as f32 + 1.0 / (1u64 << register_2282) as f32 + 1.0 / (1u64 << register_2283) as f32;

	let [register_2284, register_2285, register_2286, register_2287] = split_registers::<4>(words[571]);
	raw_estimate += 1.0 / (1u64 << register_2284) as f32 + 1.0 / (1u64 << register_2285) as f32 + 1.0 / (1u64 << register_2286) as f32 + 1.0 / (1u64 << register_2287) as f32;

	let [register_2288, register_2289, register_2290, register_2291] = split_registers::<4>(words[572]);
	raw_estimate += 1.0 / (1u64 << register_2288) as f32 + 1.0 / (1u64 << register_2289) as f32 + 1.0 / (1u64 << register_2290) as f32 + 1.0 / (1u64 << register_2291) as f32;

	let [register_2292, register_2293, register_2294, register_2295] = split_registers::<4>(words[573]);
	raw_estimate += 1.0 / (1u64 << register_2292) as f32 + 1.0 / (1u64 << register_2293) as f32 + 1.0 / (1u64 << register_2294) as f32 + 1.0 / (1u64 << register_2295) as f32;

	let [register_2296, register_2297, register_2298, register_2299] = split_registers::<4>(words[574]);
	raw_estimate += 1.0 / (1u64 << register_2296) as f32 + 1.0 / (1u64 << register_2297) as f32 + 1.0 / (1u64 << register_2298) as f32 + 1.0 / (1u64 << register_2299) as f32;

	let [register_2300, register_2301, register_2302, register_2303] = split_registers::<4>(words[575]);
	raw_estimate += 1.0 / (1u64 << register_2300) as f32 + 1.0 / (1u64 << register_2301) as f32 + 1.0 / (1u64 << register_2302) as f32 + 1.0 / (1u64 << register_2303) as f32;

	let [register_2304, register_2305, register_2306, register_2307] = split_registers::<4>(words[576]);
	raw_estimate += 1.0 / (1u64 << register_2304) as f32 + 1.0 / (1u64 << register_2305) as f32 + 1.0 / (1u64 << register_2306) as f32 + 1.0 / (1u64 << register_2307) as f32;

	let [register_2308, register_2309, register_2310, register_2311] = split_registers::<4>(words[577]);
	raw_estimate += 1.0 / (1u64 << register_2308) as f32 + 1.0 / (1u64 << register_2309) as f32 + 1.0 / (1u64 << register_2310) as f32 + 1.0 / (1u64 << register_2311) as f32;

	let [register_2312, register_2313, register_2314, register_2315] = split_registers::<4>(words[578]);
	raw_estimate += 1.0 / (1u64 << register_2312) as f32 + 1.0 / (1u64 << register_2313) as f32 + 1.0 / (1u64 << register_2314) as f32 + 1.0 / (1u64 << register_2315) as f32;

	let [register_2316, register_2317, register_2318, register_2319] = split_registers::<4>(words[579]);
	raw_estimate += 1.0 / (1u64 << register_2316) as f32 + 1.0 / (1u64 << register_2317) as f32 + 1.0 / (1u64 << register_2318) as f32 + 1.0 / (1u64 << register_2319) as f32;

	let [register_2320, register_2321, register_2322, register_2323] = split_registers::<4>(words[580]);
	raw_estimate += 1.0 / (1u64 << register_2320) as f32 + 1.0 / (1u64 << register_2321) as f32 + 1.0 / (1u64 << register_2322) as f32 + 1.0 / (1u64 << register_2323) as f32;

	let [register_2324, register_2325, register_2326, register_2327] = split_registers::<4>(words[581]);
	raw_estimate += 1.0 / (1u64 << register_2324) as f32 + 1.0 / (1u64 << register_2325) as f32 + 1.0 / (1u64 << register_2326) as f32 + 1.0 / (1u64 << register_2327) as f32;

	let [register_2328, register_2329, register_2330, register_2331] = split_registers::<4>(words[582]);
	raw_estimate += 1.0 / (1u64 << register_2328) as f32 + 1.0 / (1u64 << register_2329) as f32 + 1.0 / (1u64 << register_2330) as f32 + 1.0 / (1u64 << register_2331) as f32;

	let [register_2332, register_2333, register_2334, register_2335] = split_registers::<4>(words[583]);
	raw_estimate += 1.0 / (1u64 << register_2332) as f32 + 1.0 / (1u64 << register_2333) as f32 + 1.0 / (1u64 << register_2334) as f32 + 1.0 / (1u64 << register_2335) as f32;

	let [register_2336, register_2337, register_2338, register_2339] = split_registers::<4>(words[584]);
	raw_estimate += 1.0 / (1u64 << register_2336) as f32 + 1.0 / (1u64 << register_2337) as f32 + 1.0 / (1u64 << register_2338) as f32 + 1.0 / (1u64 << register_2339) as f32;

	let [register_2340, register_2341, register_2342, register_2343] = split_registers::<4>(words[585]);
	raw_estimate += 1.0 / (1u64 << register_2340) as f32 + 1.0 / (1u64 << register_2341) as f32 + 1.0 / (1u64 << register_2342) as f32 + 1.0 / (1u64 << register_2343) as f32;

	let [register_2344, register_2345, register_2346, register_2347] = split_registers::<4>(words[586]);
	raw_estimate += 1.0 / (1u64 << register_2344) as f32 + 1.0 / (1u64 << register_2345) as f32 + 1.0 / (1u64 << register_2346) as f32 + 1.0 / (1u64 << register_2347) as f32;

	let [register_2348, register_2349, register_2350, register_2351] = split_registers::<4>(words[587]);
	raw_estimate += 1.0 / (1u64 << register_2348) as f32 + 1.0 / (1u64 << register_2349) as f32 + 1.0 / (1u64 << register_2350) as f32 + 1.0 / (1u64 << register_2351) as f32;

	let [register_2352, register_2353, register_2354, register_2355] = split_registers::<4>(words[588]);
	raw_estimate += 1.0 / (1u64 << register_2352) as f32 + 1.0 / (1u64 << register_2353) as f32 + 1.0 / (1u64 << register_2354) as f32 + 1.0 / (1u64 << register_2355) as f32;

	let [register_2356, register_2357, register_2358, register_2359] = split_registers::<4>(words[589]);
	raw_estimate += 1.0 / (1u64 << register_2356) as f32 + 1.0 / (1u64 << register_2357) as f32 + 1.0 / (1u64 << register_2358) as f32 + 1.0 / (1u64 << register_2359) as f32;

	let [register_2360, register_2361, register_2362, register_2363] = split_registers::<4>(words[590]);
	raw_estimate += 1.0 / (1u64 << register_2360) as f32 + 1.0 / (1u64 << register_2361) as f32 + 1.0 / (1u64 << register_2362) as f32 + 1.0 / (1u64 << register_2363) as f32;

	let [register_2364, register_2365, register_2366, register_2367] = split_registers::<4>(words[591]);
	raw_estimate += 1.0 / (1u64 << register_2364) as f32 + 1.0 / (1u64 << register_2365) as f32 + 1.0 / (1u64 << register_2366) as f32 + 1.0 / (1u64 << register_2367) as f32;

	let [register_2368, register_2369, register_2370, register_2371] = split_registers::<4>(words[592]);
	raw_estimate += 1.0 / (1u64 << register_2368) as f32 + 1.0 / (1u64 << register_2369) as f32 + 1.0 / (1u64 << register_2370) as f32 + 1.0 / (1u64 << register_2371) as f32;

	let [register_2372, register_2373, register_2374, register_2375] = split_registers::<4>(words[593]);
	raw_estimate += 1.0 / (1u64 << register_2372) as f32 + 1.0 / (1u64 << register_2373) as f32 + 1.0 / (1u64 << register_2374) as f32 + 1.0 / (1u64 << register_2375) as f32;

	let [register_2376, register_2377, register_2378, register_2379] = split_registers::<4>(words[594]);
	raw_estimate += 1.0 / (1u64 << register_2376) as f32 + 1.0 / (1u64 << register_2377) as f32 + 1.0 / (1u64 << register_2378) as f32 + 1.0 / (1u64 << register_2379) as f32;

	let [register_2380, register_2381, register_2382, register_2383] = split_registers::<4>(words[595]);
	raw_estimate += 1.0 / (1u64 << register_2380) as f32 + 1.0 / (1u64 << register_2381) as f32 + 1.0 / (1u64 << register_2382) as f32 + 1.0 / (1u64 << register_2383) as f32;

	let [register_2384, register_2385, register_2386, register_2387] = split_registers::<4>(words[596]);
	raw_estimate += 1.0 / (1u64 << register_2384) as f32 + 1.0 / (1u64 << register_2385) as f32 + 1.0 / (1u64 << register_2386) as f32 + 1.0 / (1u64 << register_2387) as f32;

	let [register_2388, register_2389, register_2390, register_2391] = split_registers::<4>(words[597]);
	raw_estimate += 1.0 / (1u64 << register_2388) as f32 + 1.0 / (1u64 << register_2389) as f32 + 1.0 / (1u64 << register_2390) as f32 + 1.0 / (1u64 << register_2391) as f32;

	let [register_2392, register_2393, register_2394, register_2395] = split_registers::<4>(words[598]);
	raw_estimate += 1.0 / (1u64 << register_2392) as f32 + 1.0 / (1u64 << register_2393) as f32 + 1.0 / (1u64 << register_2394) as f32 + 1.0 / (1u64 << register_2395) as f32;

	let [register_2396, register_2397, register_2398, register_2399] = split_registers::<4>(words[599]);
	raw_estimate += 1.0 / (1u64 << register_2396) as f32 + 1.0 / (1u64 << register_2397) as f32 + 1.0 / (1u64 << register_2398) as f32 + 1.0 / (1u64 << register_2399) as f32;

	let [register_2400, register_2401, register_2402, register_2403] = split_registers::<4>(words[600]);
	raw_estimate += 1.0 / (1u64 << register_2400) as f32 + 1.0 / (1u64 << register_2401) as f32 + 1.0 / (1u64 << register_2402) as f32 + 1.0 / (1u64 << register_2403) as f32;

	let [register_2404, register_2405, register_2406, register_2407] = split_registers::<4>(words[601]);
	raw_estimate += 1.0 / (1u64 << register_2404) as f32 + 1.0 / (1u64 << register_2405) as f32 + 1.0 / (1u64 << register_2406) as f32 + 1.0 / (1u64 << register_2407) as f32;

	let [register_2408, register_2409, register_2410, register_2411] = split_registers::<4>(words[602]);
	raw_estimate += 1.0 / (1u64 << register_2408) as f32 + 1.0 / (1u64 << register_2409) as f32 + 1.0 / (1u64 << register_2410) as f32 + 1.0 / (1u64 << register_2411) as f32;

	let [register_2412, register_2413, register_2414, register_2415] = split_registers::<4>(words[603]);
	raw_estimate += 1.0 / (1u64 << register_2412) as f32 + 1.0 / (1u64 << register_2413) as f32 + 1.0 / (1u64 << register_2414) as f32 + 1.0 / (1u64 << register_2415) as f32;

	let [register_2416, register_2417, register_2418, register_2419] = split_registers::<4>(words[604]);
	raw_estimate += 1.0 / (1u64 << register_2416) as f32 + 1.0 / (1u64 << register_2417) as f32 + 1.0 / (1u64 << register_2418) as f32 + 1.0 / (1u64 << register_2419) as f32;

	let [register_2420, register_2421, register_2422, register_2423] = split_registers::<4>(words[605]);
	raw_estimate += 1.0 / (1u64 << register_2420) as f32 + 1.0 / (1u64 << register_2421) as f32 + 1.0 / (1u64 << register_2422) as f32 + 1.0 / (1u64 << register_2423) as f32;

	let [register_2424, register_2425, register_2426, register_2427] = split_registers::<4>(words[606]);
	raw_estimate += 1.0 / (1u64 << register_2424) as f32 + 1.0 / (1u64 << register_2425) as f32 + 1.0 / (1u64 << register_2426) as f32 + 1.0 / (1u64 << register_2427) as f32;

	let [register_2428, register_2429, register_2430, register_2431] = split_registers::<4>(words[607]);
	raw_estimate += 1.0 / (1u64 << register_2428) as f32 + 1.0 / (1u64 << register_2429) as f32 + 1.0 / (1u64 << register_2430) as f32 + 1.0 / (1u64 << register_2431) as f32;

	let [register_2432, register_2433, register_2434, register_2435] = split_registers::<4>(words[608]);
	raw_estimate += 1.0 / (1u64 << register_2432) as f32 + 1.0 / (1u64 << register_2433) as f32 + 1.0 / (1u64 << register_2434) as f32 + 1.0 / (1u64 << register_2435) as f32;

	let [register_2436, register_2437, register_2438, register_2439] = split_registers::<4>(words[609]);
	raw_estimate += 1.0 / (1u64 << register_2436) as f32 + 1.0 / (1u64 << register_2437) as f32 + 1.0 / (1u64 << register_2438) as f32 + 1.0 / (1u64 << register_2439) as f32;

	let [register_2440, register_2441, register_2442, register_2443] = split_registers::<4>(words[610]);
	raw_estimate += 1.0 / (1u64 << register_2440) as f32 + 1.0 / (1u64 << register_2441) as f32 + 1.0 / (1u64 << register_2442) as f32 + 1.0 / (1u64 << register_2443) as f32;

	let [register_2444, register_2445, register_2446, register_2447] = split_registers::<4>(words[611]);
	raw_estimate += 1.0 / (1u64 << register_2444) as f32 + 1.0 / (1u64 << register_2445) as f32 + 1.0 / (1u64 << register_2446) as f32 + 1.0 / (1u64 << register_2447) as f32;

	let [register_2448, register_2449, register_2450, register_2451] = split_registers::<4>(words[612]);
	raw_estimate += 1.0 / (1u64 << register_2448) as f32 + 1.0 / (1u64 << register_2449) as f32 + 1.0 / (1u64 << register_2450) as f32 + 1.0 / (1u64 << register_2451) as f32;

	let [register_2452, register_2453, register_2454, register_2455] = split_registers::<4>(words[613]);
	raw_estimate += 1.0 / (1u64 << register_2452) as f32 + 1.0 / (1u64 << register_2453) as f32 + 1.0 / (1u64 << register_2454) as f32 + 1.0 / (1u64 << register_2455) as f32;

	let [register_2456, register_2457, register_2458, register_2459] = split_registers::<4>(words[614]);
	raw_estimate += 1.0 / (1u64 << register_2456) as f32 + 1.0 / (1u64 << register_2457) as f32 + 1.0 / (1u64 << register_2458) as f32 + 1.0 / (1u64 << register_2459) as f32;

	let [register_2460, register_2461, register_2462, register_2463] = split_registers::<4>(words[615]);
	raw_estimate += 1.0 / (1u64 << register_2460) as f32 + 1.0 / (1u64 << register_2461) as f32 + 1.0 / (1u64 << register_2462) as f32 + 1.0 / (1u64 << register_2463) as f32;

	let [register_2464, register_2465, register_2466, register_2467] = split_registers::<4>(words[616]);
	raw_estimate += 1.0 / (1u64 << register_2464) as f32 + 1.0 / (1u64 << register_2465) as f32 + 1.0 / (1u64 << register_2466) as f32 + 1.0 / (1u64 << register_2467) as f32;

	let [register_2468, register_2469, register_2470, register_2471] = split_registers::<4>(words[617]);
	raw_estimate += 1.0 / (1u64 << register_2468) as f32 + 1.0 / (1u64 << register_2469) as f32 + 1.0 / (1u64 << register_2470) as f32 + 1.0 / (1u64 << register_2471) as f32;

	let [register_2472, register_2473, register_2474, register_2475] = split_registers::<4>(words[618]);
	raw_estimate += 1.0 / (1u64 << register_2472) as f32 + 1.0 / (1u64 << register_2473) as f32 + 1.0 / (1u64 << register_2474) as f32 + 1.0 / (1u64 << register_2475) as f32;

	let [register_2476, register_2477, register_2478, register_2479] = split_registers::<4>(words[619]);
	raw_estimate += 1.0 / (1u64 << register_2476) as f32 + 1.0 / (1u64 << register_2477) as f32 + 1.0 / (1u64 << register_2478) as f32 + 1.0 / (1u64 << register_2479) as f32;

	let [register_2480, register_2481, register_2482, register_2483] = split_registers::<4>(words[620]);
	raw_estimate += 1.0 / (1u64 << register_2480) as f32 + 1.0 / (1u64 << register_2481) as f32 + 1.0 / (1u64 << register_2482) as f32 + 1.0 / (1u64 << register_2483) as f32;

	let [register_2484, register_2485, register_2486, register_2487] = split_registers::<4>(words[621]);
	raw_estimate += 1.0 / (1u64 << register_2484) as f32 + 1.0 / (1u64 << register_2485) as f32 + 1.0 / (1u64 << register_2486) as f32 + 1.0 / (1u64 << register_2487) as f32;

	let [register_2488, register_2489, register_2490, register_2491] = split_registers::<4>(words[622]);
	raw_estimate += 1.0 / (1u64 << register_2488) as f32 + 1.0 / (1u64 << register_2489) as f32 + 1.0 / (1u64 << register_2490) as f32 + 1.0 / (1u64 << register_2491) as f32;

	let [register_2492, register_2493, register_2494, register_2495] = split_registers::<4>(words[623]);
	raw_estimate += 1.0 / (1u64 << register_2492) as f32 + 1.0 / (1u64 << register_2493) as f32 + 1.0 / (1u64 << register_2494) as f32 + 1.0 / (1u64 << register_2495) as f32;

	let [register_2496, register_2497, register_2498, register_2499] = split_registers::<4>(words[624]);
	raw_estimate += 1.0 / (1u64 << register_2496) as f32 + 1.0 / (1u64 << register_2497) as f32 + 1.0 / (1u64 << register_2498) as f32 + 1.0 / (1u64 << register_2499) as f32;

	let [register_2500, register_2501, register_2502, register_2503] = split_registers::<4>(words[625]);
	raw_estimate += 1.0 / (1u64 << register_2500) as f32 + 1.0 / (1u64 << register_2501) as f32 + 1.0 / (1u64 << register_2502) as f32 + 1.0 / (1u64 << register_2503) as f32;

	let [register_2504, register_2505, register_2506, register_2507] = split_registers::<4>(words[626]);
	raw_estimate += 1.0 / (1u64 << register_2504) as f32 + 1.0 / (1u64 << register_2505) as f32 + 1.0 / (1u64 << register_2506) as f32 + 1.0 / (1u64 << register_2507) as f32;

	let [register_2508, register_2509, register_2510, register_2511] = split_registers::<4>(words[627]);
	raw_estimate += 1.0 / (1u64 << register_2508) as f32 + 1.0 / (1u64 << register_2509) as f32 + 1.0 / (1u64 << register_2510) as f32 + 1.0 / (1u64 << register_2511) as f32;

	let [register_2512, register_2513, register_2514, register_2515] = split_registers::<4>(words[628]);
	raw_estimate += 1.0 / (1u64 << register_2512) as f32 + 1.0 / (1u64 << register_2513) as f32 + 1.0 / (1u64 << register_2514) as f32 + 1.0 / (1u64 << register_2515) as f32;

	let [register_2516, register_2517, register_2518, register_2519] = split_registers::<4>(words[629]);
	raw_estimate += 1.0 / (1u64 << register_2516) as f32 + 1.0 / (1u64 << register_2517) as f32 + 1.0 / (1u64 << register_2518) as f32 + 1.0 / (1u64 << register_2519) as f32;

	let [register_2520, register_2521, register_2522, register_2523] = split_registers::<4>(words[630]);
	raw_estimate += 1.0 / (1u64 << register_2520) as f32 + 1.0 / (1u64 << register_2521) as f32 + 1.0 / (1u64 << register_2522) as f32 + 1.0 / (1u64 << register_2523) as f32;

	let [register_2524, register_2525, register_2526, register_2527] = split_registers::<4>(words[631]);
	raw_estimate += 1.0 / (1u64 << register_2524) as f32 + 1.0 / (1u64 << register_2525) as f32 + 1.0 / (1u64 << register_2526) as f32 + 1.0 / (1u64 << register_2527) as f32;

	let [register_2528, register_2529, register_2530, register_2531] = split_registers::<4>(words[632]);
	raw_estimate += 1.0 / (1u64 << register_2528) as f32 + 1.0 / (1u64 << register_2529) as f32 + 1.0 / (1u64 << register_2530) as f32 + 1.0 / (1u64 << register_2531) as f32;

	let [register_2532, register_2533, register_2534, register_2535] = split_registers::<4>(words[633]);
	raw_estimate += 1.0 / (1u64 << register_2532) as f32 + 1.0 / (1u64 << register_2533) as f32 + 1.0 / (1u64 << register_2534) as f32 + 1.0 / (1u64 << register_2535) as f32;

	let [register_2536, register_2537, register_2538, register_2539] = split_registers::<4>(words[634]);
	raw_estimate += 1.0 / (1u64 << register_2536) as f32 + 1.0 / (1u64 << register_2537) as f32 + 1.0 / (1u64 << register_2538) as f32 + 1.0 / (1u64 << register_2539) as f32;

	let [register_2540, register_2541, register_2542, register_2543] = split_registers::<4>(words[635]);
	raw_estimate += 1.0 / (1u64 << register_2540) as f32 + 1.0 / (1u64 << register_2541) as f32 + 1.0 / (1u64 << register_2542) as f32 + 1.0 / (1u64 << register_2543) as f32;

	let [register_2544, register_2545, register_2546, register_2547] = split_registers::<4>(words[636]);
	raw_estimate += 1.0 / (1u64 << register_2544) as f32 + 1.0 / (1u64 << register_2545) as f32 + 1.0 / (1u64 << register_2546) as f32 + 1.0 / (1u64 << register_2547) as f32;

	let [register_2548, register_2549, register_2550, register_2551] = split_registers::<4>(words[637]);
	raw_estimate += 1.0 / (1u64 << register_2548) as f32 + 1.0 / (1u64 << register_2549) as f32 + 1.0 / (1u64 << register_2550) as f32 + 1.0 / (1u64 << register_2551) as f32;

	let [register_2552, register_2553, register_2554, register_2555] = split_registers::<4>(words[638]);
	raw_estimate += 1.0 / (1u64 << register_2552) as f32 + 1.0 / (1u64 << register_2553) as f32 + 1.0 / (1u64 << register_2554) as f32 + 1.0 / (1u64 << register_2555) as f32;

	let [register_2556, register_2557, register_2558, register_2559] = split_registers::<4>(words[639]);
	raw_estimate += 1.0 / (1u64 << register_2556) as f32 + 1.0 / (1u64 << register_2557) as f32 + 1.0 / (1u64 << register_2558) as f32 + 1.0 / (1u64 << register_2559) as f32;

	let [register_2560, register_2561, register_2562, register_2563] = split_registers::<4>(words[640]);
	raw_estimate += 1.0 / (1u64 << register_2560) as f32 + 1.0 / (1u64 << register_2561) as f32 + 1.0 / (1u64 << register_2562) as f32 + 1.0 / (1u64 << register_2563) as f32;

	let [register_2564, register_2565, register_2566, register_2567] = split_registers::<4>(words[641]);
	raw_estimate += 1.0 / (1u64 << register_2564) as f32 + 1.0 / (1u64 << register_2565) as f32 + 1.0 / (1u64 << register_2566) as f32 + 1.0 / (1u64 << register_2567) as f32;

	let [register_2568, register_2569, register_2570, register_2571] = split_registers::<4>(words[642]);
	raw_estimate += 1.0 / (1u64 << register_2568) as f32 + 1.0 / (1u64 << register_2569) as f32 + 1.0 / (1u64 << register_2570) as f32 + 1.0 / (1u64 << register_2571) as f32;

	let [register_2572, register_2573, register_2574, register_2575] = split_registers::<4>(words[643]);
	raw_estimate += 1.0 / (1u64 << register_2572) as f32 + 1.0 / (1u64 << register_2573) as f32 + 1.0 / (1u64 << register_2574) as f32 + 1.0 / (1u64 << register_2575) as f32;

	let [register_2576, register_2577, register_2578, register_2579] = split_registers::<4>(words[644]);
	raw_estimate += 1.0 / (1u64 << register_2576) as f32 + 1.0 / (1u64 << register_2577) as f32 + 1.0 / (1u64 << register_2578) as f32 + 1.0 / (1u64 << register_2579) as f32;

	let [register_2580, register_2581, register_2582, register_2583] = split_registers::<4>(words[645]);
	raw_estimate += 1.0 / (1u64 << register_2580) as f32 + 1.0 / (1u64 << register_2581) as f32 + 1.0 / (1u64 << register_2582) as f32 + 1.0 / (1u64 << register_2583) as f32;

	let [register_2584, register_2585, register_2586, register_2587] = split_registers::<4>(words[646]);
	raw_estimate += 1.0 / (1u64 << register_2584) as f32 + 1.0 / (1u64 << register_2585) as f32 + 1.0 / (1u64 << register_2586) as f32 + 1.0 / (1u64 << register_2587) as f32;

	let [register_2588, register_2589, register_2590, register_2591] = split_registers::<4>(words[647]);
	raw_estimate += 1.0 / (1u64 << register_2588) as f32 + 1.0 / (1u64 << register_2589) as f32 + 1.0 / (1u64 << register_2590) as f32 + 1.0 / (1u64 << register_2591) as f32;

	let [register_2592, register_2593, register_2594, register_2595] = split_registers::<4>(words[648]);
	raw_estimate += 1.0 / (1u64 << register_2592) as f32 + 1.0 / (1u64 << register_2593) as f32 + 1.0 / (1u64 << register_2594) as f32 + 1.0 / (1u64 << register_2595) as f32;

	let [register_2596, register_2597, register_2598, register_2599] = split_registers::<4>(words[649]);
	raw_estimate += 1.0 / (1u64 << register_2596) as f32 + 1.0 / (1u64 << register_2597) as f32 + 1.0 / (1u64 << register_2598) as f32 + 1.0 / (1u64 << register_2599) as f32;

	let [register_2600, register_2601, register_2602, register_2603] = split_registers::<4>(words[650]);
	raw_estimate += 1.0 / (1u64 << register_2600) as f32 + 1.0 / (1u64 << register_2601) as f32 + 1.0 / (1u64 << register_2602) as f32 + 1.0 / (1u64 << register_2603) as f32;

	let [register_2604, register_2605, register_2606, register_2607] = split_registers::<4>(words[651]);
	raw_estimate += 1.0 / (1u64 << register_2604) as f32 + 1.0 / (1u64 << register_2605) as f32 + 1.0 / (1u64 << register_2606) as f32 + 1.0 / (1u64 << register_2607) as f32;

	let [register_2608, register_2609, register_2610, register_2611] = split_registers::<4>(words[652]);
	raw_estimate += 1.0 / (1u64 << register_2608) as f32 + 1.0 / (1u64 << register_2609) as f32 + 1.0 / (1u64 << register_2610) as f32 + 1.0 / (1u64 << register_2611) as f32;

	let [register_2612, register_2613, register_2614, register_2615] = split_registers::<4>(words[653]);
	raw_estimate += 1.0 / (1u64 << register_2612) as f32 + 1.0 / (1u64 << register_2613) as f32 + 1.0 / (1u64 << register_2614) as f32 + 1.0 / (1u64 << register_2615) as f32;

	let [register_2616, register_2617, register_2618, register_2619] = split_registers::<4>(words[654]);
	raw_estimate += 1.0 / (1u64 << register_2616) as f32 + 1.0 / (1u64 << register_2617) as f32 + 1.0 / (1u64 << register_2618) as f32 + 1.0 / (1u64 << register_2619) as f32;

	let [register_2620, register_2621, register_2622, register_2623] = split_registers::<4>(words[655]);
	raw_estimate += 1.0 / (1u64 << register_2620) as f32 + 1.0 / (1u64 << register_2621) as f32 + 1.0 / (1u64 << register_2622) as f32 + 1.0 / (1u64 << register_2623) as f32;

	let [register_2624, register_2625, register_2626, register_2627] = split_registers::<4>(words[656]);
	raw_estimate += 1.0 / (1u64 << register_2624) as f32 + 1.0 / (1u64 << register_2625) as f32 + 1.0 / (1u64 << register_2626) as f32 + 1.0 / (1u64 << register_2627) as f32;

	let [register_2628, register_2629, register_2630, register_2631] = split_registers::<4>(words[657]);
	raw_estimate += 1.0 / (1u64 << register_2628) as f32 + 1.0 / (1u64 << register_2629) as f32 + 1.0 / (1u64 << register_2630) as f32 + 1.0 / (1u64 << register_2631) as f32;

	let [register_2632, register_2633, register_2634, register_2635] = split_registers::<4>(words[658]);
	raw_estimate += 1.0 / (1u64 << register_2632) as f32 + 1.0 / (1u64 << register_2633) as f32 + 1.0 / (1u64 << register_2634) as f32 + 1.0 / (1u64 << register_2635) as f32;

	let [register_2636, register_2637, register_2638, register_2639] = split_registers::<4>(words[659]);
	raw_estimate += 1.0 / (1u64 << register_2636) as f32 + 1.0 / (1u64 << register_2637) as f32 + 1.0 / (1u64 << register_2638) as f32 + 1.0 / (1u64 << register_2639) as f32;

	let [register_2640, register_2641, register_2642, register_2643] = split_registers::<4>(words[660]);
	raw_estimate += 1.0 / (1u64 << register_2640) as f32 + 1.0 / (1u64 << register_2641) as f32 + 1.0 / (1u64 << register_2642) as f32 + 1.0 / (1u64 << register_2643) as f32;

	let [register_2644, register_2645, register_2646, register_2647] = split_registers::<4>(words[661]);
	raw_estimate += 1.0 / (1u64 << register_2644) as f32 + 1.0 / (1u64 << register_2645) as f32 + 1.0 / (1u64 << register_2646) as f32 + 1.0 / (1u64 << register_2647) as f32;

	let [register_2648, register_2649, register_2650, register_2651] = split_registers::<4>(words[662]);
	raw_estimate += 1.0 / (1u64 << register_2648) as f32 + 1.0 / (1u64 << register_2649) as f32 + 1.0 / (1u64 << register_2650) as f32 + 1.0 / (1u64 << register_2651) as f32;

	let [register_2652, register_2653, register_2654, register_2655] = split_registers::<4>(words[663]);
	raw_estimate += 1.0 / (1u64 << register_2652) as f32 + 1.0 / (1u64 << register_2653) as f32 + 1.0 / (1u64 << register_2654) as f32 + 1.0 / (1u64 << register_2655) as f32;

	let [register_2656, register_2657, register_2658, register_2659] = split_registers::<4>(words[664]);
	raw_estimate += 1.0 / (1u64 << register_2656) as f32 + 1.0 / (1u64 << register_2657) as f32 + 1.0 / (1u64 << register_2658) as f32 + 1.0 / (1u64 << register_2659) as f32;

	let [register_2660, register_2661, register_2662, register_2663] = split_registers::<4>(words[665]);
	raw_estimate += 1.0 / (1u64 << register_2660) as f32 + 1.0 / (1u64 << register_2661) as f32 + 1.0 / (1u64 << register_2662) as f32 + 1.0 / (1u64 << register_2663) as f32;

	let [register_2664, register_2665, register_2666, register_2667] = split_registers::<4>(words[666]);
	raw_estimate += 1.0 / (1u64 << register_2664) as f32 + 1.0 / (1u64 << register_2665) as f32 + 1.0 / (1u64 << register_2666) as f32 + 1.0 / (1u64 << register_2667) as f32;

	let [register_2668, register_2669, register_2670, register_2671] = split_registers::<4>(words[667]);
	raw_estimate += 1.0 / (1u64 << register_2668) as f32 + 1.0 / (1u64 << register_2669) as f32 + 1.0 / (1u64 << register_2670) as f32 + 1.0 / (1u64 << register_2671) as f32;

	let [register_2672, register_2673, register_2674, register_2675] = split_registers::<4>(words[668]);
	raw_estimate += 1.0 / (1u64 << register_2672) as f32 + 1.0 / (1u64 << register_2673) as f32 + 1.0 / (1u64 << register_2674) as f32 + 1.0 / (1u64 << register_2675) as f32;

	let [register_2676, register_2677, register_2678, register_2679] = split_registers::<4>(words[669]);
	raw_estimate += 1.0 / (1u64 << register_2676) as f32 + 1.0 / (1u64 << register_2677) as f32 + 1.0 / (1u64 << register_2678) as f32 + 1.0 / (1u64 << register_2679) as f32;

	let [register_2680, register_2681, register_2682, register_2683] = split_registers::<4>(words[670]);
	raw_estimate += 1.0 / (1u64 << register_2680) as f32 + 1.0 / (1u64 << register_2681) as f32 + 1.0 / (1u64 << register_2682) as f32 + 1.0 / (1u64 << register_2683) as f32;

	let [register_2684, register_2685, register_2686, register_2687] = split_registers::<4>(words[671]);
	raw_estimate += 1.0 / (1u64 << register_2684) as f32 + 1.0 / (1u64 << register_2685) as f32 + 1.0 / (1u64 << register_2686) as f32 + 1.0 / (1u64 << register_2687) as f32;

	let [register_2688, register_2689, register_2690, register_2691] = split_registers::<4>(words[672]);
	raw_estimate += 1.0 / (1u64 << register_2688) as f32 + 1.0 / (1u64 << register_2689) as f32 + 1.0 / (1u64 << register_2690) as f32 + 1.0 / (1u64 << register_2691) as f32;

	let [register_2692, register_2693, register_2694, register_2695] = split_registers::<4>(words[673]);
	raw_estimate += 1.0 / (1u64 << register_2692) as f32 + 1.0 / (1u64 << register_2693) as f32 + 1.0 / (1u64 << register_2694) as f32 + 1.0 / (1u64 << register_2695) as f32;

	let [register_2696, register_2697, register_2698, register_2699] = split_registers::<4>(words[674]);
	raw_estimate += 1.0 / (1u64 << register_2696) as f32 + 1.0 / (1u64 << register_2697) as f32 + 1.0 / (1u64 << register_2698) as f32 + 1.0 / (1u64 << register_2699) as f32;

	let [register_2700, register_2701, register_2702, register_2703] = split_registers::<4>(words[675]);
	raw_estimate += 1.0 / (1u64 << register_2700) as f32 + 1.0 / (1u64 << register_2701) as f32 + 1.0 / (1u64 << register_2702) as f32 + 1.0 / (1u64 << register_2703) as f32;

	let [register_2704, register_2705, register_2706, register_2707] = split_registers::<4>(words[676]);
	raw_estimate += 1.0 / (1u64 << register_2704) as f32 + 1.0 / (1u64 << register_2705) as f32 + 1.0 / (1u64 << register_2706) as f32 + 1.0 / (1u64 << register_2707) as f32;

	let [register_2708, register_2709, register_2710, register_2711] = split_registers::<4>(words[677]);
	raw_estimate += 1.0 / (1u64 << register_2708) as f32 + 1.0 / (1u64 << register_2709) as f32 + 1.0 / (1u64 << register_2710) as f32 + 1.0 / (1u64 << register_2711) as f32;

	let [register_2712, register_2713, register_2714, register_2715] = split_registers::<4>(words[678]);
	raw_estimate += 1.0 / (1u64 << register_2712) as f32 + 1.0 / (1u64 << register_2713) as f32 + 1.0 / (1u64 << register_2714) as f32 + 1.0 / (1u64 << register_2715) as f32;

	let [register_2716, register_2717, register_2718, register_2719] = split_registers::<4>(words[679]);
	raw_estimate += 1.0 / (1u64 << register_2716) as f32 + 1.0 / (1u64 << register_2717) as f32 + 1.0 / (1u64 << register_2718) as f32 + 1.0 / (1u64 << register_2719) as f32;

	let [register_2720, register_2721, register_2722, register_2723] = split_registers::<4>(words[680]);
	raw_estimate += 1.0 / (1u64 << register_2720) as f32 + 1.0 / (1u64 << register_2721) as f32 + 1.0 / (1u64 << register_2722) as f32 + 1.0 / (1u64 << register_2723) as f32;

	let [register_2724, register_2725, register_2726, register_2727] = split_registers::<4>(words[681]);
	raw_estimate += 1.0 / (1u64 << register_2724) as f32 + 1.0 / (1u64 << register_2725) as f32 + 1.0 / (1u64 << register_2726) as f32 + 1.0 / (1u64 << register_2727) as f32;

	let [register_2728, register_2729, register_2730, register_2731] = split_registers::<4>(words[682]);
	raw_estimate += 1.0 / (1u64 << register_2728) as f32 + 1.0 / (1u64 << register_2729) as f32 + 1.0 / (1u64 << register_2730) as f32 + 1.0 / (1u64 << register_2731) as f32;

	let [register_2732, register_2733, register_2734, register_2735] = split_registers::<4>(words[683]);
	raw_estimate += 1.0 / (1u64 << register_2732) as f32 + 1.0 / (1u64 << register_2733) as f32 + 1.0 / (1u64 << register_2734) as f32 + 1.0 / (1u64 << register_2735) as f32;

	let [register_2736, register_2737, register_2738, register_2739] = split_registers::<4>(words[684]);
	raw_estimate += 1.0 / (1u64 << register_2736) as f32 + 1.0 / (1u64 << register_2737) as f32 + 1.0 / (1u64 << register_2738) as f32 + 1.0 / (1u64 << register_2739) as f32;

	let [register_2740, register_2741, register_2742, register_2743] = split_registers::<4>(words[685]);
	raw_estimate += 1.0 / (1u64 << register_2740) as f32 + 1.0 / (1u64 << register_2741) as f32 + 1.0 / (1u64 << register_2742) as f32 + 1.0 / (1u64 << register_2743) as f32;

	let [register_2744, register_2745, register_2746, register_2747] = split_registers::<4>(words[686]);
	raw_estimate += 1.0 / (1u64 << register_2744) as f32 + 1.0 / (1u64 << register_2745) as f32 + 1.0 / (1u64 << register_2746) as f32 + 1.0 / (1u64 << register_2747) as f32;

	let [register_2748, register_2749, register_2750, register_2751] = split_registers::<4>(words[687]);
	raw_estimate += 1.0 / (1u64 << register_2748) as f32 + 1.0 / (1u64 << register_2749) as f32 + 1.0 / (1u64 << register_2750) as f32 + 1.0 / (1u64 << register_2751) as f32;

	let [register_2752, register_2753, register_2754, register_2755] = split_registers::<4>(words[688]);
	raw_estimate += 1.0 / (1u64 << register_2752) as f32 + 1.0 / (1u64 << register_2753) as f32 + 1.0 / (1u64 << register_2754) as f32 + 1.0 / (1u64 << register_2755) as f32;

	let [register_2756, register_2757, register_2758, register_2759] = split_registers::<4>(words[689]);
	raw_estimate += 1.0 / (1u64 << register_2756) as f32 + 1.0 / (1u64 << register_2757) as f32 + 1.0 / (1u64 << register_2758) as f32 + 1.0 / (1u64 << register_2759) as f32;

	let [register_2760, register_2761, register_2762, register_2763] = split_registers::<4>(words[690]);
	raw_estimate += 1.0 / (1u64 << register_2760) as f32 + 1.0 / (1u64 << register_2761) as f32 + 1.0 / (1u64 << register_2762) as f32 + 1.0 / (1u64 << register_2763) as f32;

	let [register_2764, register_2765, register_2766, register_2767] = split_registers::<4>(words[691]);
	raw_estimate += 1.0 / (1u64 << register_2764) as f32 + 1.0 / (1u64 << register_2765) as f32 + 1.0 / (1u64 << register_2766) as f32 + 1.0 / (1u64 << register_2767) as f32;

	let [register_2768, register_2769, register_2770, register_2771] = split_registers::<4>(words[692]);
	raw_estimate += 1.0 / (1u64 << register_2768) as f32 + 1.0 / (1u64 << register_2769) as f32 + 1.0 / (1u64 << register_2770) as f32 + 1.0 / (1u64 << register_2771) as f32;

	let [register_2772, register_2773, register_2774, register_2775] = split_registers::<4>(words[693]);
	raw_estimate += 1.0 / (1u64 << register_2772) as f32 + 1.0 / (1u64 << register_2773) as f32 + 1.0 / (1u64 << register_2774) as f32 + 1.0 / (1u64 << register_2775) as f32;

	let [register_2776, register_2777, register_2778, register_2779] = split_registers::<4>(words[694]);
	raw_estimate += 1.0 / (1u64 << register_2776) as f32 + 1.0 / (1u64 << register_2777) as f32 + 1.0 / (1u64 << register_2778) as f32 + 1.0 / (1u64 << register_2779) as f32;

	let [register_2780, register_2781, register_2782, register_2783] = split_registers::<4>(words[695]);
	raw_estimate += 1.0 / (1u64 << register_2780) as f32 + 1.0 / (1u64 << register_2781) as f32 + 1.0 / (1u64 << register_2782) as f32 + 1.0 / (1u64 << register_2783) as f32;

	let [register_2784, register_2785, register_2786, register_2787] = split_registers::<4>(words[696]);
	raw_estimate += 1.0 / (1u64 << register_2784) as f32 + 1.0 / (1u64 << register_2785) as f32 + 1.0 / (1u64 << register_2786) as f32 + 1.0 / (1u64 << register_2787) as f32;

	let [register_2788, register_2789, register_2790, register_2791] = split_registers::<4>(words[697]);
	raw_estimate += 1.0 / (1u64 << register_2788) as f32 + 1.0 / (1u64 << register_2789) as f32 + 1.0 / (1u64 << register_2790) as f32 + 1.0 / (1u64 << register_2791) as f32;

	let [register_2792, register_2793, register_2794, register_2795] = split_registers::<4>(words[698]);
	raw_estimate += 1.0 / (1u64 << register_2792) as f32 + 1.0 / (1u64 << register_2793) as f32 + 1.0 / (1u64 << register_2794) as f32 + 1.0 / (1u64 << register_2795) as f32;

	let [register_2796, register_2797, register_2798, register_2799] = split_registers::<4>(words[699]);
	raw_estimate += 1.0 / (1u64 << register_2796) as f32 + 1.0 / (1u64 << register_2797) as f32 + 1.0 / (1u64 << register_2798) as f32 + 1.0 / (1u64 << register_2799) as f32;

	let [register_2800, register_2801, register_2802, register_2803] = split_registers::<4>(words[700]);
	raw_estimate += 1.0 / (1u64 << register_2800) as f32 + 1.0 / (1u64 << register_2801) as f32 + 1.0 / (1u64 << register_2802) as f32 + 1.0 / (1u64 << register_2803) as f32;

	let [register_2804, register_2805, register_2806, register_2807] = split_registers::<4>(words[701]);
	raw_estimate += 1.0 / (1u64 << register_2804) as f32 + 1.0 / (1u64 << register_2805) as f32 + 1.0 / (1u64 << register_2806) as f32 + 1.0 / (1u64 << register_2807) as f32;

	let [register_2808, register_2809, register_2810, register_2811] = split_registers::<4>(words[702]);
	raw_estimate += 1.0 / (1u64 << register_2808) as f32 + 1.0 / (1u64 << register_2809) as f32 + 1.0 / (1u64 << register_2810) as f32 + 1.0 / (1u64 << register_2811) as f32;

	let [register_2812, register_2813, register_2814, register_2815] = split_registers::<4>(words[703]);
	raw_estimate += 1.0 / (1u64 << register_2812) as f32 + 1.0 / (1u64 << register_2813) as f32 + 1.0 / (1u64 << register_2814) as f32 + 1.0 / (1u64 << register_2815) as f32;

	let [register_2816, register_2817, register_2818, register_2819] = split_registers::<4>(words[704]);
	raw_estimate += 1.0 / (1u64 << register_2816) as f32 + 1.0 / (1u64 << register_2817) as f32 + 1.0 / (1u64 << register_2818) as f32 + 1.0 / (1u64 << register_2819) as f32;

	let [register_2820, register_2821, register_2822, register_2823] = split_registers::<4>(words[705]);
	raw_estimate += 1.0 / (1u64 << register_2820) as f32 + 1.0 / (1u64 << register_2821) as f32 + 1.0 / (1u64 << register_2822) as f32 + 1.0 / (1u64 << register_2823) as f32;

	let [register_2824, register_2825, register_2826, register_2827] = split_registers::<4>(words[706]);
	raw_estimate += 1.0 / (1u64 << register_2824) as f32 + 1.0 / (1u64 << register_2825) as f32 + 1.0 / (1u64 << register_2826) as f32 + 1.0 / (1u64 << register_2827) as f32;

	let [register_2828, register_2829, register_2830, register_2831] = split_registers::<4>(words[707]);
	raw_estimate += 1.0 / (1u64 << register_2828) as f32 + 1.0 / (1u64 << register_2829) as f32 + 1.0 / (1u64 << register_2830) as f32 + 1.0 / (1u64 << register_2831) as f32;

	let [register_2832, register_2833, register_2834, register_2835] = split_registers::<4>(words[708]);
	raw_estimate += 1.0 / (1u64 << register_2832) as f32 + 1.0 / (1u64 << register_2833) as f32 + 1.0 / (1u64 << register_2834) as f32 + 1.0 / (1u64 << register_2835) as f32;

	let [register_2836, register_2837, register_2838, register_2839] = split_registers::<4>(words[709]);
	raw_estimate += 1.0 / (1u64 << register_2836) as f32 + 1.0 / (1u64 << register_2837) as f32 + 1.0 / (1u64 << register_2838) as f32 + 1.0 / (1u64 << register_2839) as f32;

	let [register_2840, register_2841, register_2842, register_2843] = split_registers::<4>(words[710]);
	raw_estimate += 1.0 / (1u64 << register_2840) as f32 + 1.0 / (1u64 << register_2841) as f32 + 1.0 / (1u64 << register_2842) as f32 + 1.0 / (1u64 << register_2843) as f32;

	let [register_2844, register_2845, register_2846, register_2847] = split_registers::<4>(words[711]);
	raw_estimate += 1.0 / (1u64 << register_2844) as f32 + 1.0 / (1u64 << register_2845) as f32 + 1.0 / (1u64 << register_2846) as f32 + 1.0 / (1u64 << register_2847) as f32;

	let [register_2848, register_2849, register_2850, register_2851] = split_registers::<4>(words[712]);
	raw_estimate += 1.0 / (1u64 << register_2848) as f32 + 1.0 / (1u64 << register_2849) as f32 + 1.0 / (1u64 << register_2850) as f32 + 1.0 / (1u64 << register_2851) as f32;

	let [register_2852, register_2853, register_2854, register_2855] = split_registers::<4>(words[713]);
	raw_estimate += 1.0 / (1u64 << register_2852) as f32 + 1.0 / (1u64 << register_2853) as f32 + 1.0 / (1u64 << register_2854) as f32 + 1.0 / (1u64 << register_2855) as f32;

	let [register_2856, register_2857, register_2858, register_2859] = split_registers::<4>(words[714]);
	raw_estimate += 1.0 / (1u64 << register_2856) as f32 + 1.0 / (1u64 << register_2857) as f32 + 1.0 / (1u64 << register_2858) as f32 + 1.0 / (1u64 << register_2859) as f32;

	let [register_2860, register_2861, register_2862, register_2863] = split_registers::<4>(words[715]);
	raw_estimate += 1.0 / (1u64 << register_2860) as f32 + 1.0 / (1u64 << register_2861) as f32 + 1.0 / (1u64 << register_2862) as f32 + 1.0 / (1u64 << register_2863) as f32;

	let [register_2864, register_2865, register_2866, register_2867] = split_registers::<4>(words[716]);
	raw_estimate += 1.0 / (1u64 << register_2864) as f32 + 1.0 / (1u64 << register_2865) as f32 + 1.0 / (1u64 << register_2866) as f32 + 1.0 / (1u64 << register_2867) as f32;

	let [register_2868, register_2869, register_2870, register_2871] = split_registers::<4>(words[717]);
	raw_estimate += 1.0 / (1u64 << register_2868) as f32 + 1.0 / (1u64 << register_2869) as f32 + 1.0 / (1u64 << register_2870) as f32 + 1.0 / (1u64 << register_2871) as f32;

	let [register_2872, register_2873, register_2874, register_2875] = split_registers::<4>(words[718]);
	raw_estimate += 1.0 / (1u64 << register_2872) as f32 + 1.0 / (1u64 << register_2873) as f32 + 1.0 / (1u64 << register_2874) as f32 + 1.0 / (1u64 << register_2875) as f32;

	let [register_2876, register_2877, register_2878, register_2879] = split_registers::<4>(words[719]);
	raw_estimate += 1.0 / (1u64 << register_2876) as f32 + 1.0 / (1u64 << register_2877) as f32 + 1.0 / (1u64 << register_2878) as f32 + 1.0 / (1u64 << register_2879) as f32;

	let [register_2880, register_2881, register_2882, register_2883] = split_registers::<4>(words[720]);
	raw_estimate += 1.0 / (1u64 << register_2880) as f32 + 1.0 / (1u64 << register_2881) as f32 + 1.0 / (1u64 << register_2882) as f32 + 1.0 / (1u64 << register_2883) as f32;

	let [register_2884, register_2885, register_2886, register_2887] = split_registers::<4>(words[721]);
	raw_estimate += 1.0 / (1u64 << register_2884) as f32 + 1.0 / (1u64 << register_2885) as f32 + 1.0 / (1u64 << register_2886) as f32 + 1.0 / (1u64 << register_2887) as f32;

	let [register_2888, register_2889, register_2890, register_2891] = split_registers::<4>(words[722]);
	raw_estimate += 1.0 / (1u64 << register_2888) as f32 + 1.0 / (1u64 << register_2889) as f32 + 1.0 / (1u64 << register_2890) as f32 + 1.0 / (1u64 << register_2891) as f32;

	let [register_2892, register_2893, register_2894, register_2895] = split_registers::<4>(words[723]);
	raw_estimate += 1.0 / (1u64 << register_2892) as f32 + 1.0 / (1u64 << register_2893) as f32 + 1.0 / (1u64 << register_2894) as f32 + 1.0 / (1u64 << register_2895) as f32;

	let [register_2896, register_2897, register_2898, register_2899] = split_registers::<4>(words[724]);
	raw_estimate += 1.0 / (1u64 << register_2896) as f32 + 1.0 / (1u64 << register_2897) as f32 + 1.0 / (1u64 << register_2898) as f32 + 1.0 / (1u64 << register_2899) as f32;

	let [register_2900, register_2901, register_2902, register_2903] = split_registers::<4>(words[725]);
	raw_estimate += 1.0 / (1u64 << register_2900) as f32 + 1.0 / (1u64 << register_2901) as f32 + 1.0 / (1u64 << register_2902) as f32 + 1.0 / (1u64 << register_2903) as f32;

	let [register_2904, register_2905, register_2906, register_2907] = split_registers::<4>(words[726]);
	raw_estimate += 1.0 / (1u64 << register_2904) as f32 + 1.0 / (1u64 << register_2905) as f32 + 1.0 / (1u64 << register_2906) as f32 + 1.0 / (1u64 << register_2907) as f32;

	let [register_2908, register_2909, register_2910, register_2911] = split_registers::<4>(words[727]);
	raw_estimate += 1.0 / (1u64 << register_2908) as f32 + 1.0 / (1u64 << register_2909) as f32 + 1.0 / (1u64 << register_2910) as f32 + 1.0 / (1u64 << register_2911) as f32;

	let [register_2912, register_2913, register_2914, register_2915] = split_registers::<4>(words[728]);
	raw_estimate += 1.0 / (1u64 << register_2912) as f32 + 1.0 / (1u64 << register_2913) as f32 + 1.0 / (1u64 << register_2914) as f32 + 1.0 / (1u64 << register_2915) as f32;

	let [register_2916, register_2917, register_2918, register_2919] = split_registers::<4>(words[729]);
	raw_estimate += 1.0 / (1u64 << register_2916) as f32 + 1.0 / (1u64 << register_2917) as f32 + 1.0 / (1u64 << register_2918) as f32 + 1.0 / (1u64 << register_2919) as f32;

	let [register_2920, register_2921, register_2922, register_2923] = split_registers::<4>(words[730]);
	raw_estimate += 1.0 / (1u64 << register_2920) as f32 + 1.0 / (1u64 << register_2921) as f32 + 1.0 / (1u64 << register_2922) as f32 + 1.0 / (1u64 << register_2923) as f32;

	let [register_2924, register_2925, register_2926, register_2927] = split_registers::<4>(words[731]);
	raw_estimate += 1.0 / (1u64 << register_2924) as f32 + 1.0 / (1u64 << register_2925) as f32 + 1.0 / (1u64 << register_2926) as f32 + 1.0 / (1u64 << register_2927) as f32;

	let [register_2928, register_2929, register_2930, register_2931] = split_registers::<4>(words[732]);
	raw_estimate += 1.0 / (1u64 << register_2928) as f32 + 1.0 / (1u64 << register_2929) as f32 + 1.0 / (1u64 << register_2930) as f32 + 1.0 / (1u64 << register_2931) as f32;

	let [register_2932, register_2933, register_2934, register_2935] = split_registers::<4>(words[733]);
	raw_estimate += 1.0 / (1u64 << register_2932) as f32 + 1.0 / (1u64 << register_2933) as f32 + 1.0 / (1u64 << register_2934) as f32 + 1.0 / (1u64 << register_2935) as f32;

	let [register_2936, register_2937, register_2938, register_2939] = split_registers::<4>(words[734]);
	raw_estimate += 1.0 / (1u64 << register_2936) as f32 + 1.0 / (1u64 << register_2937) as f32 + 1.0 / (1u64 << register_2938) as f32 + 1.0 / (1u64 << register_2939) as f32;

	let [register_2940, register_2941, register_2942, register_2943] = split_registers::<4>(words[735]);
	raw_estimate += 1.0 / (1u64 << register_2940) as f32 + 1.0 / (1u64 << register_2941) as f32 + 1.0 / (1u64 << register_2942) as f32 + 1.0 / (1u64 << register_2943) as f32;

	let [register_2944, register_2945, register_2946, register_2947] = split_registers::<4>(words[736]);
	raw_estimate += 1.0 / (1u64 << register_2944) as f32 + 1.0 / (1u64 << register_2945) as f32 + 1.0 / (1u64 << register_2946) as f32 + 1.0 / (1u64 << register_2947) as f32;

	let [register_2948, register_2949, register_2950, register_2951] = split_registers::<4>(words[737]);
	raw_estimate += 1.0 / (1u64 << register_2948) as f32 + 1.0 / (1u64 << register_2949) as f32 + 1.0 / (1u64 << register_2950) as f32 + 1.0 / (1u64 << register_2951) as f32;

	let [register_2952, register_2953, register_2954, register_2955] = split_registers::<4>(words[738]);
	raw_estimate += 1.0 / (1u64 << register_2952) as f32 + 1.0 / (1u64 << register_2953) as f32 + 1.0 / (1u64 << register_2954) as f32 + 1.0 / (1u64 << register_2955) as f32;

	let [register_2956, register_2957, register_2958, register_2959] = split_registers::<4>(words[739]);
	raw_estimate += 1.0 / (1u64 << register_2956) as f32 + 1.0 / (1u64 << register_2957) as f32 + 1.0 / (1u64 << register_2958) as f32 + 1.0 / (1u64 << register_2959) as f32;

	let [register_2960, register_2961, register_2962, register_2963] = split_registers::<4>(words[740]);
	raw_estimate += 1.0 / (1u64 << register_2960) as f32 + 1.0 / (1u64 << register_2961) as f32 + 1.0 / (1u64 << register_2962) as f32 + 1.0 / (1u64 << register_2963) as f32;

	let [register_2964, register_2965, register_2966, register_2967] = split_registers::<4>(words[741]);
	raw_estimate += 1.0 / (1u64 << register_2964) as f32 + 1.0 / (1u64 << register_2965) as f32 + 1.0 / (1u64 << register_2966) as f32 + 1.0 / (1u64 << register_2967) as f32;

	let [register_2968, register_2969, register_2970, register_2971] = split_registers::<4>(words[742]);
	raw_estimate += 1.0 / (1u64 << register_2968) as f32 + 1.0 / (1u64 << register_2969) as f32 + 1.0 / (1u64 << register_2970) as f32 + 1.0 / (1u64 << register_2971) as f32;

	let [register_2972, register_2973, register_2974, register_2975] = split_registers::<4>(words[743]);
	raw_estimate += 1.0 / (1u64 << register_2972) as f32 + 1.0 / (1u64 << register_2973) as f32 + 1.0 / (1u64 << register_2974) as f32 + 1.0 / (1u64 << register_2975) as f32;

	let [register_2976, register_2977, register_2978, register_2979] = split_registers::<4>(words[744]);
	raw_estimate += 1.0 / (1u64 << register_2976) as f32 + 1.0 / (1u64 << register_2977) as f32 + 1.0 / (1u64 << register_2978) as f32 + 1.0 / (1u64 << register_2979) as f32;

	let [register_2980, register_2981, register_2982, register_2983] = split_registers::<4>(words[745]);
	raw_estimate += 1.0 / (1u64 << register_2980) as f32 + 1.0 / (1u64 << register_2981) as f32 + 1.0 / (1u64 << register_2982) as f32 + 1.0 / (1u64 << register_2983) as f32;

	let [register_2984, register_2985, register_2986, register_2987] = split_registers::<4>(words[746]);
	raw_estimate += 1.0 / (1u64 << register_2984) as f32 + 1.0 / (1u64 << register_2985) as f32 + 1.0 / (1u64 << register_2986) as f32 + 1.0 / (1u64 << register_2987) as f32;

	let [register_2988, register_2989, register_2990, register_2991] = split_registers::<4>(words[747]);
	raw_estimate += 1.0 / (1u64 << register_2988) as f32 + 1.0 / (1u64 << register_2989) as f32 + 1.0 / (1u64 << register_2990) as f32 + 1.0 / (1u64 << register_2991) as f32;

	let [register_2992, register_2993, register_2994, register_2995] = split_registers::<4>(words[748]);
	raw_estimate += 1.0 / (1u64 << register_2992) as f32 + 1.0 / (1u64 << register_2993) as f32 + 1.0 / (1u64 << register_2994) as f32 + 1.0 / (1u64 << register_2995) as f32;

	let [register_2996, register_2997, register_2998, register_2999] = split_registers::<4>(words[749]);
	raw_estimate += 1.0 / (1u64 << register_2996) as f32 + 1.0 / (1u64 << register_2997) as f32 + 1.0 / (1u64 << register_2998) as f32 + 1.0 / (1u64 << register_2999) as f32;

	let [register_3000, register_3001, register_3002, register_3003] = split_registers::<4>(words[750]);
	raw_estimate += 1.0 / (1u64 << register_3000) as f32 + 1.0 / (1u64 << register_3001) as f32 + 1.0 / (1u64 << register_3002) as f32 + 1.0 / (1u64 << register_3003) as f32;

	let [register_3004, register_3005, register_3006, register_3007] = split_registers::<4>(words[751]);
	raw_estimate += 1.0 / (1u64 << register_3004) as f32 + 1.0 / (1u64 << register_3005) as f32 + 1.0 / (1u64 << register_3006) as f32 + 1.0 / (1u64 << register_3007) as f32;

	let [register_3008, register_3009, register_3010, register_3011] = split_registers::<4>(words[752]);
	raw_estimate += 1.0 / (1u64 << register_3008) as f32 + 1.0 / (1u64 << register_3009) as f32 + 1.0 / (1u64 << register_3010) as f32 + 1.0 / (1u64 << register_3011) as f32;

	let [register_3012, register_3013, register_3014, register_3015] = split_registers::<4>(words[753]);
	raw_estimate += 1.0 / (1u64 << register_3012) as f32 + 1.0 / (1u64 << register_3013) as f32 + 1.0 / (1u64 << register_3014) as f32 + 1.0 / (1u64 << register_3015) as f32;

	let [register_3016, register_3017, register_3018, register_3019] = split_registers::<4>(words[754]);
	raw_estimate += 1.0 / (1u64 << register_3016) as f32 + 1.0 / (1u64 << register_3017) as f32 + 1.0 / (1u64 << register_3018) as f32 + 1.0 / (1u64 << register_3019) as f32;

	let [register_3020, register_3021, register_3022, register_3023] = split_registers::<4>(words[755]);
	raw_estimate += 1.0 / (1u64 << register_3020) as f32 + 1.0 / (1u64 << register_3021) as f32 + 1.0 / (1u64 << register_3022) as f32 + 1.0 / (1u64 << register_3023) as f32;

	let [register_3024, register_3025, register_3026, register_3027] = split_registers::<4>(words[756]);
	raw_estimate += 1.0 / (1u64 << register_3024) as f32 + 1.0 / (1u64 << register_3025) as f32 + 1.0 / (1u64 << register_3026) as f32 + 1.0 / (1u64 << register_3027) as f32;

	let [register_3028, register_3029, register_3030, register_3031] = split_registers::<4>(words[757]);
	raw_estimate += 1.0 / (1u64 << register_3028) as f32 + 1.0 / (1u64 << register_3029) as f32 + 1.0 / (1u64 << register_3030) as f32 + 1.0 / (1u64 << register_3031) as f32;

	let [register_3032, register_3033, register_3034, register_3035] = split_registers::<4>(words[758]);
	raw_estimate += 1.0 / (1u64 << register_3032) as f32 + 1.0 / (1u64 << register_3033) as f32 + 1.0 / (1u64 << register_3034) as f32 + 1.0 / (1u64 << register_3035) as f32;

	let [register_3036, register_3037, register_3038, register_3039] = split_registers::<4>(words[759]);
	raw_estimate += 1.0 / (1u64 << register_3036) as f32 + 1.0 / (1u64 << register_3037) as f32 + 1.0 / (1u64 << register_3038) as f32 + 1.0 / (1u64 << register_3039) as f32;

	let [register_3040, register_3041, register_3042, register_3043] = split_registers::<4>(words[760]);
	raw_estimate += 1.0 / (1u64 << register_3040) as f32 + 1.0 / (1u64 << register_3041) as f32 + 1.0 / (1u64 << register_3042) as f32 + 1.0 / (1u64 << register_3043) as f32;

	let [register_3044, register_3045, register_3046, register_3047] = split_registers::<4>(words[761]);
	raw_estimate += 1.0 / (1u64 << register_3044) as f32 + 1.0 / (1u64 << register_3045) as f32 + 1.0 / (1u64 << register_3046) as f32 + 1.0 / (1u64 << register_3047) as f32;

	let [register_3048, register_3049, register_3050, register_3051] = split_registers::<4>(words[762]);
	raw_estimate += 1.0 / (1u64 << register_3048) as f32 + 1.0 / (1u64 << register_3049) as f32 + 1.0 / (1u64 << register_3050) as f32 + 1.0 / (1u64 << register_3051) as f32;

	let [register_3052, register_3053, register_3054, register_3055] = split_registers::<4>(words[763]);
	raw_estimate += 1.0 / (1u64 << register_3052) as f32 + 1.0 / (1u64 << register_3053) as f32 + 1.0 / (1u64 << register_3054) as f32 + 1.0 / (1u64 << register_3055) as f32;

	let [register_3056, register_3057, register_3058, register_3059] = split_registers::<4>(words[764]);
	raw_estimate += 1.0 / (1u64 << register_3056) as f32 + 1.0 / (1u64 << register_3057) as f32 + 1.0 / (1u64 << register_3058) as f32 + 1.0 / (1u64 << register_3059) as f32;

	let [register_3060, register_3061, register_3062, register_3063] = split_registers::<4>(words[765]);
	raw_estimate += 1.0 / (1u64 << register_3060) as f32 + 1.0 / (1u64 << register_3061) as f32 + 1.0 / (1u64 << register_3062) as f32 + 1.0 / (1u64 << register_3063) as f32;

	let [register_3064, register_3065, register_3066, register_3067] = split_registers::<4>(words[766]);
	raw_estimate += 1.0 / (1u64 << register_3064) as f32 + 1.0 / (1u64 << register_3065) as f32 + 1.0 / (1u64 << register_3066) as f32 + 1.0 / (1u64 << register_3067) as f32;

	let [register_3068, register_3069, register_3070, register_3071] = split_registers::<4>(words[767]);
	raw_estimate += 1.0 / (1u64 << register_3068) as f32 + 1.0 / (1u64 << register_3069) as f32 + 1.0 / (1u64 << register_3070) as f32 + 1.0 / (1u64 << register_3071) as f32;

	let [register_3072, register_3073, register_3074, register_3075] = split_registers::<4>(words[768]);
	raw_estimate += 1.0 / (1u64 << register_3072) as f32 + 1.0 / (1u64 << register_3073) as f32 + 1.0 / (1u64 << register_3074) as f32 + 1.0 / (1u64 << register_3075) as f32;

	let [register_3076, register_3077, register_3078, register_3079] = split_registers::<4>(words[769]);
	raw_estimate += 1.0 / (1u64 << register_3076) as f32 + 1.0 / (1u64 << register_3077) as f32 + 1.0 / (1u64 << register_3078) as f32 + 1.0 / (1u64 << register_3079) as f32;

	let [register_3080, register_3081, register_3082, register_3083] = split_registers::<4>(words[770]);
	raw_estimate += 1.0 / (1u64 << register_3080) as f32 + 1.0 / (1u64 << register_3081) as f32 + 1.0 / (1u64 << register_3082) as f32 + 1.0 / (1u64 << register_3083) as f32;

	let [register_3084, register_3085, register_3086, register_3087] = split_registers::<4>(words[771]);
	raw_estimate += 1.0 / (1u64 << register_3084) as f32 + 1.0 / (1u64 << register_3085) as f32 + 1.0 / (1u64 << register_3086) as f32 + 1.0 / (1u64 << register_3087) as f32;

	let [register_3088, register_3089, register_3090, register_3091] = split_registers::<4>(words[772]);
	raw_estimate += 1.0 / (1u64 << register_3088) as f32 + 1.0 / (1u64 << register_3089) as f32 + 1.0 / (1u64 << register_3090) as f32 + 1.0 / (1u64 << register_3091) as f32;

	let [register_3092, register_3093, register_3094, register_3095] = split_registers::<4>(words[773]);
	raw_estimate += 1.0 / (1u64 << register_3092) as f32 + 1.0 / (1u64 << register_3093) as f32 + 1.0 / (1u64 << register_3094) as f32 + 1.0 / (1u64 << register_3095) as f32;

	let [register_3096, register_3097, register_3098, register_3099] = split_registers::<4>(words[774]);
	raw_estimate += 1.0 / (1u64 << register_3096) as f32 + 1.0 / (1u64 << register_3097) as f32 + 1.0 / (1u64 << register_3098) as f32 + 1.0 / (1u64 << register_3099) as f32;

	let [register_3100, register_3101, register_3102, register_3103] = split_registers::<4>(words[775]);
	raw_estimate += 1.0 / (1u64 << register_3100) as f32 + 1.0 / (1u64 << register_3101) as f32 + 1.0 / (1u64 << register_3102) as f32 + 1.0 / (1u64 << register_3103) as f32;

	let [register_3104, register_3105, register_3106, register_3107] = split_registers::<4>(words[776]);
	raw_estimate += 1.0 / (1u64 << register_3104) as f32 + 1.0 / (1u64 << register_3105) as f32 + 1.0 / (1u64 << register_3106) as f32 + 1.0 / (1u64 << register_3107) as f32;

	let [register_3108, register_3109, register_3110, register_3111] = split_registers::<4>(words[777]);
	raw_estimate += 1.0 / (1u64 << register_3108) as f32 + 1.0 / (1u64 << register_3109) as f32 + 1.0 / (1u64 << register_3110) as f32 + 1.0 / (1u64 << register_3111) as f32;

	let [register_3112, register_3113, register_3114, register_3115] = split_registers::<4>(words[778]);
	raw_estimate += 1.0 / (1u64 << register_3112) as f32 + 1.0 / (1u64 << register_3113) as f32 + 1.0 / (1u64 << register_3114) as f32 + 1.0 / (1u64 << register_3115) as f32;

	let [register_3116, register_3117, register_3118, register_3119] = split_registers::<4>(words[779]);
	raw_estimate += 1.0 / (1u64 << register_3116) as f32 + 1.0 / (1u64 << register_3117) as f32 + 1.0 / (1u64 << register_3118) as f32 + 1.0 / (1u64 << register_3119) as f32;

	let [register_3120, register_3121, register_3122, register_3123] = split_registers::<4>(words[780]);
	raw_estimate += 1.0 / (1u64 << register_3120) as f32 + 1.0 / (1u64 << register_3121) as f32 + 1.0 / (1u64 << register_3122) as f32 + 1.0 / (1u64 << register_3123) as f32;

	let [register_3124, register_3125, register_3126, register_3127] = split_registers::<4>(words[781]);
	raw_estimate += 1.0 / (1u64 << register_3124) as f32 + 1.0 / (1u64 << register_3125) as f32 + 1.0 / (1u64 << register_3126) as f32 + 1.0 / (1u64 << register_3127) as f32;

	let [register_3128, register_3129, register_3130, register_3131] = split_registers::<4>(words[782]);
	raw_estimate += 1.0 / (1u64 << register_3128) as f32 + 1.0 / (1u64 << register_3129) as f32 + 1.0 / (1u64 << register_3130) as f32 + 1.0 / (1u64 << register_3131) as f32;

	let [register_3132, register_3133, register_3134, register_3135] = split_registers::<4>(words[783]);
	raw_estimate += 1.0 / (1u64 << register_3132) as f32 + 1.0 / (1u64 << register_3133) as f32 + 1.0 / (1u64 << register_3134) as f32 + 1.0 / (1u64 << register_3135) as f32;

	let [register_3136, register_3137, register_3138, register_3139] = split_registers::<4>(words[784]);
	raw_estimate += 1.0 / (1u64 << register_3136) as f32 + 1.0 / (1u64 << register_3137) as f32 + 1.0 / (1u64 << register_3138) as f32 + 1.0 / (1u64 << register_3139) as f32;

	let [register_3140, register_3141, register_3142, register_3143] = split_registers::<4>(words[785]);
	raw_estimate += 1.0 / (1u64 << register_3140) as f32 + 1.0 / (1u64 << register_3141) as f32 + 1.0 / (1u64 << register_3142) as f32 + 1.0 / (1u64 << register_3143) as f32;

	let [register_3144, register_3145, register_3146, register_3147] = split_registers::<4>(words[786]);
	raw_estimate += 1.0 / (1u64 << register_3144) as f32 + 1.0 / (1u64 << register_3145) as f32 + 1.0 / (1u64 << register_3146) as f32 + 1.0 / (1u64 << register_3147) as f32;

	let [register_3148, register_3149, register_3150, register_3151] = split_registers::<4>(words[787]);
	raw_estimate += 1.0 / (1u64 << register_3148) as f32 + 1.0 / (1u64 << register_3149) as f32 + 1.0 / (1u64 << register_3150) as f32 + 1.0 / (1u64 << register_3151) as f32;

	let [register_3152, register_3153, register_3154, register_3155] = split_registers::<4>(words[788]);
	raw_estimate += 1.0 / (1u64 << register_3152) as f32 + 1.0 / (1u64 << register_3153) as f32 + 1.0 / (1u64 << register_3154) as f32 + 1.0 / (1u64 << register_3155) as f32;

	let [register_3156, register_3157, register_3158, register_3159] = split_registers::<4>(words[789]);
	raw_estimate += 1.0 / (1u64 << register_3156) as f32 + 1.0 / (1u64 << register_3157) as f32 + 1.0 / (1u64 << register_3158) as f32 + 1.0 / (1u64 << register_3159) as f32;

	let [register_3160, register_3161, register_3162, register_3163] = split_registers::<4>(words[790]);
	raw_estimate += 1.0 / (1u64 << register_3160) as f32 + 1.0 / (1u64 << register_3161) as f32 + 1.0 / (1u64 << register_3162) as f32 + 1.0 / (1u64 << register_3163) as f32;

	let [register_3164, register_3165, register_3166, register_3167] = split_registers::<4>(words[791]);
	raw_estimate += 1.0 / (1u64 << register_3164) as f32 + 1.0 / (1u64 << register_3165) as f32 + 1.0 / (1u64 << register_3166) as f32 + 1.0 / (1u64 << register_3167) as f32;

	let [register_3168, register_3169, register_3170, register_3171] = split_registers::<4>(words[792]);
	raw_estimate += 1.0 / (1u64 << register_3168) as f32 + 1.0 / (1u64 << register_3169) as f32 + 1.0 / (1u64 << register_3170) as f32 + 1.0 / (1u64 << register_3171) as f32;

	let [register_3172, register_3173, register_3174, register_3175] = split_registers::<4>(words[793]);
	raw_estimate += 1.0 / (1u64 << register_3172) as f32 + 1.0 / (1u64 << register_3173) as f32 + 1.0 / (1u64 << register_3174) as f32 + 1.0 / (1u64 << register_3175) as f32;

	let [register_3176, register_3177, register_3178, register_3179] = split_registers::<4>(words[794]);
	raw_estimate += 1.0 / (1u64 << register_3176) as f32 + 1.0 / (1u64 << register_3177) as f32 + 1.0 / (1u64 << register_3178) as f32 + 1.0 / (1u64 << register_3179) as f32;

	let [register_3180, register_3181, register_3182, register_3183] = split_registers::<4>(words[795]);
	raw_estimate += 1.0 / (1u64 << register_3180) as f32 + 1.0 / (1u64 << register_3181) as f32 + 1.0 / (1u64 << register_3182) as f32 + 1.0 / (1u64 << register_3183) as f32;

	let [register_3184, register_3185, register_3186, register_3187] = split_registers::<4>(words[796]);
	raw_estimate += 1.0 / (1u64 << register_3184) as f32 + 1.0 / (1u64 << register_3185) as f32 + 1.0 / (1u64 << register_3186) as f32 + 1.0 / (1u64 << register_3187) as f32;

	let [register_3188, register_3189, register_3190, register_3191] = split_registers::<4>(words[797]);
	raw_estimate += 1.0 / (1u64 << register_3188) as f32 + 1.0 / (1u64 << register_3189) as f32 + 1.0 / (1u64 << register_3190) as f32 + 1.0 / (1u64 << register_3191) as f32;

	let [register_3192, register_3193, register_3194, register_3195] = split_registers::<4>(words[798]);
	raw_estimate += 1.0 / (1u64 << register_3192) as f32 + 1.0 / (1u64 << register_3193) as f32 + 1.0 / (1u64 << register_3194) as f32 + 1.0 / (1u64 << register_3195) as f32;

	let [register_3196, register_3197, register_3198, register_3199] = split_registers::<4>(words[799]);
	raw_estimate += 1.0 / (1u64 << register_3196) as f32 + 1.0 / (1u64 << register_3197) as f32 + 1.0 / (1u64 << register_3198) as f32 + 1.0 / (1u64 << register_3199) as f32;

	let [register_3200, register_3201, register_3202, register_3203] = split_registers::<4>(words[800]);
	raw_estimate += 1.0 / (1u64 << register_3200) as f32 + 1.0 / (1u64 << register_3201) as f32 + 1.0 / (1u64 << register_3202) as f32 + 1.0 / (1u64 << register_3203) as f32;

	let [register_3204, register_3205, register_3206, register_3207] = split_registers::<4>(words[801]);
	raw_estimate += 1.0 / (1u64 << register_3204) as f32 + 1.0 / (1u64 << register_3205) as f32 + 1.0 / (1u64 << register_3206) as f32 + 1.0 / (1u64 << register_3207) as f32;

	let [register_3208, register_3209, register_3210, register_3211] = split_registers::<4>(words[802]);
	raw_estimate += 1.0 / (1u64 << register_3208) as f32 + 1.0 / (1u64 << register_3209) as f32 + 1.0 / (1u64 << register_3210) as f32 + 1.0 / (1u64 << register_3211) as f32;

	let [register_3212, register_3213, register_3214, register_3215] = split_registers::<4>(words[803]);
	raw_estimate += 1.0 / (1u64 << register_3212) as f32 + 1.0 / (1u64 << register_3213) as f32 + 1.0 / (1u64 << register_3214) as f32 + 1.0 / (1u64 << register_3215) as f32;

	let [register_3216, register_3217, register_3218, register_3219] = split_registers::<4>(words[804]);
	raw_estimate += 1.0 / (1u64 << register_3216) as f32 + 1.0 / (1u64 << register_3217) as f32 + 1.0 / (1u64 << register_3218) as f32 + 1.0 / (1u64 << register_3219) as f32;

	let [register_3220, register_3221, register_3222, register_3223] = split_registers::<4>(words[805]);
	raw_estimate += 1.0 / (1u64 << register_3220) as f32 + 1.0 / (1u64 << register_3221) as f32 + 1.0 / (1u64 << register_3222) as f32 + 1.0 / (1u64 << register_3223) as f32;

	let [register_3224, register_3225, register_3226, register_3227] = split_registers::<4>(words[806]);
	raw_estimate += 1.0 / (1u64 << register_3224) as f32 + 1.0 / (1u64 << register_3225) as f32 + 1.0 / (1u64 << register_3226) as f32 + 1.0 / (1u64 << register_3227) as f32;

	let [register_3228, register_3229, register_3230, register_3231] = split_registers::<4>(words[807]);
	raw_estimate += 1.0 / (1u64 << register_3228) as f32 + 1.0 / (1u64 << register_3229) as f32 + 1.0 / (1u64 << register_3230) as f32 + 1.0 / (1u64 << register_3231) as f32;

	let [register_3232, register_3233, register_3234, register_3235] = split_registers::<4>(words[808]);
	raw_estimate += 1.0 / (1u64 << register_3232) as f32 + 1.0 / (1u64 << register_3233) as f32 + 1.0 / (1u64 << register_3234) as f32 + 1.0 / (1u64 << register_3235) as f32;

	let [register_3236, register_3237, register_3238, register_3239] = split_registers::<4>(words[809]);
	raw_estimate += 1.0 / (1u64 << register_3236) as f32 + 1.0 / (1u64 << register_3237) as f32 + 1.0 / (1u64 << register_3238) as f32 + 1.0 / (1u64 << register_3239) as f32;

	let [register_3240, register_3241, register_3242, register_3243] = split_registers::<4>(words[810]);
	raw_estimate += 1.0 / (1u64 << register_3240) as f32 + 1.0 / (1u64 << register_3241) as f32 + 1.0 / (1u64 << register_3242) as f32 + 1.0 / (1u64 << register_3243) as f32;

	let [register_3244, register_3245, register_3246, register_3247] = split_registers::<4>(words[811]);
	raw_estimate += 1.0 / (1u64 << register_3244) as f32 + 1.0 / (1u64 << register_3245) as f32 + 1.0 / (1u64 << register_3246) as f32 + 1.0 / (1u64 << register_3247) as f32;

	let [register_3248, register_3249, register_3250, register_3251] = split_registers::<4>(words[812]);
	raw_estimate += 1.0 / (1u64 << register_3248) as f32 + 1.0 / (1u64 << register_3249) as f32 + 1.0 / (1u64 << register_3250) as f32 + 1.0 / (1u64 << register_3251) as f32;

	let [register_3252, register_3253, register_3254, register_3255] = split_registers::<4>(words[813]);
	raw_estimate += 1.0 / (1u64 << register_3252) as f32 + 1.0 / (1u64 << register_3253) as f32 + 1.0 / (1u64 << register_3254) as f32 + 1.0 / (1u64 << register_3255) as f32;

	let [register_3256, register_3257, register_3258, register_3259] = split_registers::<4>(words[814]);
	raw_estimate += 1.0 / (1u64 << register_3256) as f32 + 1.0 / (1u64 << register_3257) as f32 + 1.0 / (1u64 << register_3258) as f32 + 1.0 / (1u64 << register_3259) as f32;

	let [register_3260, register_3261, register_3262, register_3263] = split_registers::<4>(words[815]);
	raw_estimate += 1.0 / (1u64 << register_3260) as f32 + 1.0 / (1u64 << register_3261) as f32 + 1.0 / (1u64 << register_3262) as f32 + 1.0 / (1u64 << register_3263) as f32;

	let [register_3264, register_3265, register_3266, register_3267] = split_registers::<4>(words[816]);
	raw_estimate += 1.0 / (1u64 << register_3264) as f32 + 1.0 / (1u64 << register_3265) as f32 + 1.0 / (1u64 << register_3266) as f32 + 1.0 / (1u64 << register_3267) as f32;

	let [register_3268, register_3269, register_3270, register_3271] = split_registers::<4>(words[817]);
	raw_estimate += 1.0 / (1u64 << register_3268) as f32 + 1.0 / (1u64 << register_3269) as f32 + 1.0 / (1u64 << register_3270) as f32 + 1.0 / (1u64 << register_3271) as f32;

	let [register_3272, register_3273, register_3274, register_3275] = split_registers::<4>(words[818]);
	raw_estimate += 1.0 / (1u64 << register_3272) as f32 + 1.0 / (1u64 << register_3273) as f32 + 1.0 / (1u64 << register_3274) as f32 + 1.0 / (1u64 << register_3275) as f32;

	let [register_3276, register_3277, register_3278, register_3279] = split_registers::<4>(words[819]);
	raw_estimate += 1.0 / (1u64 << register_3276) as f32 + 1.0 / (1u64 << register_3277) as f32 + 1.0 / (1u64 << register_3278) as f32 + 1.0 / (1u64 << register_3279) as f32;

	let [register_3280, register_3281, register_3282, register_3283] = split_registers::<4>(words[820]);
	raw_estimate += 1.0 / (1u64 << register_3280) as f32 + 1.0 / (1u64 << register_3281) as f32 + 1.0 / (1u64 << register_3282) as f32 + 1.0 / (1u64 << register_3283) as f32;

	let [register_3284, register_3285, register_3286, register_3287] = split_registers::<4>(words[821]);
	raw_estimate += 1.0 / (1u64 << register_3284) as f32 + 1.0 / (1u64 << register_3285) as f32 + 1.0 / (1u64 << register_3286) as f32 + 1.0 / (1u64 << register_3287) as f32;

	let [register_3288, register_3289, register_3290, register_3291] = split_registers::<4>(words[822]);
	raw_estimate += 1.0 / (1u64 << register_3288) as f32 + 1.0 / (1u64 << register_3289) as f32 + 1.0 / (1u64 << register_3290) as f32 + 1.0 / (1u64 << register_3291) as f32;

	let [register_3292, register_3293, register_3294, register_3295] = split_registers::<4>(words[823]);
	raw_estimate += 1.0 / (1u64 << register_3292) as f32 + 1.0 / (1u64 << register_3293) as f32 + 1.0 / (1u64 << register_3294) as f32 + 1.0 / (1u64 << register_3295) as f32;

	let [register_3296, register_3297, register_3298, register_3299] = split_registers::<4>(words[824]);
	raw_estimate += 1.0 / (1u64 << register_3296) as f32 + 1.0 / (1u64 << register_3297) as f32 + 1.0 / (1u64 << register_3298) as f32 + 1.0 / (1u64 << register_3299) as f32;

	let [register_3300, register_3301, register_3302, register_3303] = split_registers::<4>(words[825]);
	raw_estimate += 1.0 / (1u64 << register_3300) as f32 + 1.0 / (1u64 << register_3301) as f32 + 1.0 / (1u64 << register_3302) as f32 + 1.0 / (1u64 << register_3303) as f32;

	let [register_3304, register_3305, register_3306, register_3307] = split_registers::<4>(words[826]);
	raw_estimate += 1.0 / (1u64 << register_3304) as f32 + 1.0 / (1u64 << register_3305) as f32 + 1.0 / (1u64 << register_3306) as f32 + 1.0 / (1u64 << register_3307) as f32;

	let [register_3308, register_3309, register_3310, register_3311] = split_registers::<4>(words[827]);
	raw_estimate += 1.0 / (1u64 << register_3308) as f32 + 1.0 / (1u64 << register_3309) as f32 + 1.0 / (1u64 << register_3310) as f32 + 1.0 / (1u64 << register_3311) as f32;

	let [register_3312, register_3313, register_3314, register_3315] = split_registers::<4>(words[828]);
	raw_estimate += 1.0 / (1u64 << register_3312) as f32 + 1.0 / (1u64 << register_3313) as f32 + 1.0 / (1u64 << register_3314) as f32 + 1.0 / (1u64 << register_3315) as f32;

	let [register_3316, register_3317, register_3318, register_3319] = split_registers::<4>(words[829]);
	raw_estimate += 1.0 / (1u64 << register_3316) as f32 + 1.0 / (1u64 << register_3317) as f32 + 1.0 / (1u64 << register_3318) as f32 + 1.0 / (1u64 << register_3319) as f32;

	let [register_3320, register_3321, register_3322, register_3323] = split_registers::<4>(words[830]);
	raw_estimate += 1.0 / (1u64 << register_3320) as f32 + 1.0 / (1u64 << register_3321) as f32 + 1.0 / (1u64 << register_3322) as f32 + 1.0 / (1u64 << register_3323) as f32;

	let [register_3324, register_3325, register_3326, register_3327] = split_registers::<4>(words[831]);
	raw_estimate += 1.0 / (1u64 << register_3324) as f32 + 1.0 / (1u64 << register_3325) as f32 + 1.0 / (1u64 << register_3326) as f32 + 1.0 / (1u64 << register_3327) as f32;

	let [register_3328, register_3329, register_3330, register_3331] = split_registers::<4>(words[832]);
	raw_estimate += 1.0 / (1u64 << register_3328) as f32 + 1.0 / (1u64 << register_3329) as f32 + 1.0 / (1u64 << register_3330) as f32 + 1.0 / (1u64 << register_3331) as f32;

	let [register_3332, register_3333, register_3334, register_3335] = split_registers::<4>(words[833]);
	raw_estimate += 1.0 / (1u64 << register_3332) as f32 + 1.0 / (1u64 << register_3333) as f32 + 1.0 / (1u64 << register_3334) as f32 + 1.0 / (1u64 << register_3335) as f32;

	let [register_3336, register_3337, register_3338, register_3339] = split_registers::<4>(words[834]);
	raw_estimate += 1.0 / (1u64 << register_3336) as f32 + 1.0 / (1u64 << register_3337) as f32 + 1.0 / (1u64 << register_3338) as f32 + 1.0 / (1u64 << register_3339) as f32;

	let [register_3340, register_3341, register_3342, register_3343] = split_registers::<4>(words[835]);
	raw_estimate += 1.0 / (1u64 << register_3340) as f32 + 1.0 / (1u64 << register_3341) as f32 + 1.0 / (1u64 << register_3342) as f32 + 1.0 / (1u64 << register_3343) as f32;

	let [register_3344, register_3345, register_3346, register_3347] = split_registers::<4>(words[836]);
	raw_estimate += 1.0 / (1u64 << register_3344) as f32 + 1.0 / (1u64 << register_3345) as f32 + 1.0 / (1u64 << register_3346) as f32 + 1.0 / (1u64 << register_3347) as f32;

	let [register_3348, register_3349, register_3350, register_3351] = split_registers::<4>(words[837]);
	raw_estimate += 1.0 / (1u64 << register_3348) as f32 + 1.0 / (1u64 << register_3349) as f32 + 1.0 / (1u64 << register_3350) as f32 + 1.0 / (1u64 << register_3351) as f32;

	let [register_3352, register_3353, register_3354, register_3355] = split_registers::<4>(words[838]);
	raw_estimate += 1.0 / (1u64 << register_3352) as f32 + 1.0 / (1u64 << register_3353) as f32 + 1.0 / (1u64 << register_3354) as f32 + 1.0 / (1u64 << register_3355) as f32;

	let [register_3356, register_3357, register_3358, register_3359] = split_registers::<4>(words[839]);
	raw_estimate += 1.0 / (1u64 << register_3356) as f32 + 1.0 / (1u64 << register_3357) as f32 + 1.0 / (1u64 << register_3358) as f32 + 1.0 / (1u64 << register_3359) as f32;

	let [register_3360, register_3361, register_3362, register_3363] = split_registers::<4>(words[840]);
	raw_estimate += 1.0 / (1u64 << register_3360) as f32 + 1.0 / (1u64 << register_3361) as f32 + 1.0 / (1u64 << register_3362) as f32 + 1.0 / (1u64 << register_3363) as f32;

	let [register_3364, register_3365, register_3366, register_3367] = split_registers::<4>(words[841]);
	raw_estimate += 1.0 / (1u64 << register_3364) as f32 + 1.0 / (1u64 << register_3365) as f32 + 1.0 / (1u64 << register_3366) as f32 + 1.0 / (1u64 << register_3367) as f32;

	let [register_3368, register_3369, register_3370, register_3371] = split_registers::<4>(words[842]);
	raw_estimate += 1.0 / (1u64 << register_3368) as f32 + 1.0 / (1u64 << register_3369) as f32 + 1.0 / (1u64 << register_3370) as f32 + 1.0 / (1u64 << register_3371) as f32;

	let [register_3372, register_3373, register_3374, register_3375] = split_registers::<4>(words[843]);
	raw_estimate += 1.0 / (1u64 << register_3372) as f32 + 1.0 / (1u64 << register_3373) as f32 + 1.0 / (1u64 << register_3374) as f32 + 1.0 / (1u64 << register_3375) as f32;

	let [register_3376, register_3377, register_3378, register_3379] = split_registers::<4>(words[844]);
	raw_estimate += 1.0 / (1u64 << register_3376) as f32 + 1.0 / (1u64 << register_3377) as f32 + 1.0 / (1u64 << register_3378) as f32 + 1.0 / (1u64 << register_3379) as f32;

	let [register_3380, register_3381, register_3382, register_3383] = split_registers::<4>(words[845]);
	raw_estimate += 1.0 / (1u64 << register_3380) as f32 + 1.0 / (1u64 << register_3381) as f32 + 1.0 / (1u64 << register_3382) as f32 + 1.0 / (1u64 << register_3383) as f32;

	let [register_3384, register_3385, register_3386, register_3387] = split_registers::<4>(words[846]);
	raw_estimate += 1.0 / (1u64 << register_3384) as f32 + 1.0 / (1u64 << register_3385) as f32 + 1.0 / (1u64 << register_3386) as f32 + 1.0 / (1u64 << register_3387) as f32;

	let [register_3388, register_3389, register_3390, register_3391] = split_registers::<4>(words[847]);
	raw_estimate += 1.0 / (1u64 << register_3388) as f32 + 1.0 / (1u64 << register_3389) as f32 + 1.0 / (1u64 << register_3390) as f32 + 1.0 / (1u64 << register_3391) as f32;

	let [register_3392, register_3393, register_3394, register_3395] = split_registers::<4>(words[848]);
	raw_estimate += 1.0 / (1u64 << register_3392) as f32 + 1.0 / (1u64 << register_3393) as f32 + 1.0 / (1u64 << register_3394) as f32 + 1.0 / (1u64 << register_3395) as f32;

	let [register_3396, register_3397, register_3398, register_3399] = split_registers::<4>(words[849]);
	raw_estimate += 1.0 / (1u64 << register_3396) as f32 + 1.0 / (1u64 << register_3397) as f32 + 1.0 / (1u64 << register_3398) as f32 + 1.0 / (1u64 << register_3399) as f32;

	let [register_3400, register_3401, register_3402, register_3403] = split_registers::<4>(words[850]);
	raw_estimate += 1.0 / (1u64 << register_3400) as f32 + 1.0 / (1u64 << register_3401) as f32 + 1.0 / (1u64 << register_3402) as f32 + 1.0 / (1u64 << register_3403) as f32;

	let [register_3404, register_3405, register_3406, register_3407] = split_registers::<4>(words[851]);
	raw_estimate += 1.0 / (1u64 << register_3404) as f32 + 1.0 / (1u64 << register_3405) as f32 + 1.0 / (1u64 << register_3406) as f32 + 1.0 / (1u64 << register_3407) as f32;

	let [register_3408, register_3409, register_3410, register_3411] = split_registers::<4>(words[852]);
	raw_estimate += 1.0 / (1u64 << register_3408) as f32 + 1.0 / (1u64 << register_3409) as f32 + 1.0 / (1u64 << register_3410) as f32 + 1.0 / (1u64 << register_3411) as f32;

	let [register_3412, register_3413, register_3414, register_3415] = split_registers::<4>(words[853]);
	raw_estimate += 1.0 / (1u64 << register_3412) as f32 + 1.0 / (1u64 << register_3413) as f32 + 1.0 / (1u64 << register_3414) as f32 + 1.0 / (1u64 << register_3415) as f32;

	let [register_3416, register_3417, register_3418, register_3419] = split_registers::<4>(words[854]);
	raw_estimate += 1.0 / (1u64 << register_3416) as f32 + 1.0 / (1u64 << register_3417) as f32 + 1.0 / (1u64 << register_3418) as f32 + 1.0 / (1u64 << register_3419) as f32;

	let [register_3420, register_3421, register_3422, register_3423] = split_registers::<4>(words[855]);
	raw_estimate += 1.0 / (1u64 << register_3420) as f32 + 1.0 / (1u64 << register_3421) as f32 + 1.0 / (1u64 << register_3422) as f32 + 1.0 / (1u64 << register_3423) as f32;

	let [register_3424, register_3425, register_3426, register_3427] = split_registers::<4>(words[856]);
	raw_estimate += 1.0 / (1u64 << register_3424) as f32 + 1.0 / (1u64 << register_3425) as f32 + 1.0 / (1u64 << register_3426) as f32 + 1.0 / (1u64 << register_3427) as f32;

	let [register_3428, register_3429, register_3430, register_3431] = split_registers::<4>(words[857]);
	raw_estimate += 1.0 / (1u64 << register_3428) as f32 + 1.0 / (1u64 << register_3429) as f32 + 1.0 / (1u64 << register_3430) as f32 + 1.0 / (1u64 << register_3431) as f32;

	let [register_3432, register_3433, register_3434, register_3435] = split_registers::<4>(words[858]);
	raw_estimate += 1.0 / (1u64 << register_3432) as f32 + 1.0 / (1u64 << register_3433) as f32 + 1.0 / (1u64 << register_3434) as f32 + 1.0 / (1u64 << register_3435) as f32;

	let [register_3436, register_3437, register_3438, register_3439] = split_registers::<4>(words[859]);
	raw_estimate += 1.0 / (1u64 << register_3436) as f32 + 1.0 / (1u64 << register_3437) as f32 + 1.0 / (1u64 << register_3438) as f32 + 1.0 / (1u64 << register_3439) as f32;

	let [register_3440, register_3441, register_3442, register_3443] = split_registers::<4>(words[860]);
	raw_estimate += 1.0 / (1u64 << register_3440) as f32 + 1.0 / (1u64 << register_3441) as f32 + 1.0 / (1u64 << register_3442) as f32 + 1.0 / (1u64 << register_3443) as f32;

	let [register_3444, register_3445, register_3446, register_3447] = split_registers::<4>(words[861]);
	raw_estimate += 1.0 / (1u64 << register_3444) as f32 + 1.0 / (1u64 << register_3445) as f32 + 1.0 / (1u64 << register_3446) as f32 + 1.0 / (1u64 << register_3447) as f32;

	let [register_3448, register_3449, register_3450, register_3451] = split_registers::<4>(words[862]);
	raw_estimate += 1.0 / (1u64 << register_3448) as f32 + 1.0 / (1u64 << register_3449) as f32 + 1.0 / (1u64 << register_3450) as f32 + 1.0 / (1u64 << register_3451) as f32;

	let [register_3452, register_3453, register_3454, register_3455] = split_registers::<4>(words[863]);
	raw_estimate += 1.0 / (1u64 << register_3452) as f32 + 1.0 / (1u64 << register_3453) as f32 + 1.0 / (1u64 << register_3454) as f32 + 1.0 / (1u64 << register_3455) as f32;

	let [register_3456, register_3457, register_3458, register_3459] = split_registers::<4>(words[864]);
	raw_estimate += 1.0 / (1u64 << register_3456) as f32 + 1.0 / (1u64 << register_3457) as f32 + 1.0 / (1u64 << register_3458) as f32 + 1.0 / (1u64 << register_3459) as f32;

	let [register_3460, register_3461, register_3462, register_3463] = split_registers::<4>(words[865]);
	raw_estimate += 1.0 / (1u64 << register_3460) as f32 + 1.0 / (1u64 << register_3461) as f32 + 1.0 / (1u64 << register_3462) as f32 + 1.0 / (1u64 << register_3463) as f32;

	let [register_3464, register_3465, register_3466, register_3467] = split_registers::<4>(words[866]);
	raw_estimate += 1.0 / (1u64 << register_3464) as f32 + 1.0 / (1u64 << register_3465) as f32 + 1.0 / (1u64 << register_3466) as f32 + 1.0 / (1u64 << register_3467) as f32;

	let [register_3468, register_3469, register_3470, register_3471] = split_registers::<4>(words[867]);
	raw_estimate += 1.0 / (1u64 << register_3468) as f32 + 1.0 / (1u64 << register_3469) as f32 + 1.0 / (1u64 << register_3470) as f32 + 1.0 / (1u64 << register_3471) as f32;

	let [register_3472, register_3473, register_3474, register_3475] = split_registers::<4>(words[868]);
	raw_estimate += 1.0 / (1u64 << register_3472) as f32 + 1.0 / (1u64 << register_3473) as f32 + 1.0 / (1u64 << register_3474) as f32 + 1.0 / (1u64 << register_3475) as f32;

	let [register_3476, register_3477, register_3478, register_3479] = split_registers::<4>(words[869]);
	raw_estimate += 1.0 / (1u64 << register_3476) as f32 + 1.0 / (1u64 << register_3477) as f32 + 1.0 / (1u64 << register_3478) as f32 + 1.0 / (1u64 << register_3479) as f32;

	let [register_3480, register_3481, register_3482, register_3483] = split_registers::<4>(words[870]);
	raw_estimate += 1.0 / (1u64 << register_3480) as f32 + 1.0 / (1u64 << register_3481) as f32 + 1.0 / (1u64 << register_3482) as f32 + 1.0 / (1u64 << register_3483) as f32;

	let [register_3484, register_3485, register_3486, register_3487] = split_registers::<4>(words[871]);
	raw_estimate += 1.0 / (1u64 << register_3484) as f32 + 1.0 / (1u64 << register_3485) as f32 + 1.0 / (1u64 << register_3486) as f32 + 1.0 / (1u64 << register_3487) as f32;

	let [register_3488, register_3489, register_3490, register_3491] = split_registers::<4>(words[872]);
	raw_estimate += 1.0 / (1u64 << register_3488) as f32 + 1.0 / (1u64 << register_3489) as f32 + 1.0 / (1u64 << register_3490) as f32 + 1.0 / (1u64 << register_3491) as f32;

	let [register_3492, register_3493, register_3494, register_3495] = split_registers::<4>(words[873]);
	raw_estimate += 1.0 / (1u64 << register_3492) as f32 + 1.0 / (1u64 << register_3493) as f32 + 1.0 / (1u64 << register_3494) as f32 + 1.0 / (1u64 << register_3495) as f32;

	let [register_3496, register_3497, register_3498, register_3499] = split_registers::<4>(words[874]);
	raw_estimate += 1.0 / (1u64 << register_3496) as f32 + 1.0 / (1u64 << register_3497) as f32 + 1.0 / (1u64 << register_3498) as f32 + 1.0 / (1u64 << register_3499) as f32;

	let [register_3500, register_3501, register_3502, register_3503] = split_registers::<4>(words[875]);
	raw_estimate += 1.0 / (1u64 << register_3500) as f32 + 1.0 / (1u64 << register_3501) as f32 + 1.0 / (1u64 << register_3502) as f32 + 1.0 / (1u64 << register_3503) as f32;

	let [register_3504, register_3505, register_3506, register_3507] = split_registers::<4>(words[876]);
	raw_estimate += 1.0 / (1u64 << register_3504) as f32 + 1.0 / (1u64 << register_3505) as f32 + 1.0 / (1u64 << register_3506) as f32 + 1.0 / (1u64 << register_3507) as f32;

	let [register_3508, register_3509, register_3510, register_3511] = split_registers::<4>(words[877]);
	raw_estimate += 1.0 / (1u64 << register_3508) as f32 + 1.0 / (1u64 << register_3509) as f32 + 1.0 / (1u64 << register_3510) as f32 + 1.0 / (1u64 << register_3511) as f32;

	let [register_3512, register_3513, register_3514, register_3515] = split_registers::<4>(words[878]);
	raw_estimate += 1.0 / (1u64 << register_3512) as f32 + 1.0 / (1u64 << register_3513) as f32 + 1.0 / (1u64 << register_3514) as f32 + 1.0 / (1u64 << register_3515) as f32;

	let [register_3516, register_3517, register_3518, register_3519] = split_registers::<4>(words[879]);
	raw_estimate += 1.0 / (1u64 << register_3516) as f32 + 1.0 / (1u64 << register_3517) as f32 + 1.0 / (1u64 << register_3518) as f32 + 1.0 / (1u64 << register_3519) as f32;

	let [register_3520, register_3521, register_3522, register_3523] = split_registers::<4>(words[880]);
	raw_estimate += 1.0 / (1u64 << register_3520) as f32 + 1.0 / (1u64 << register_3521) as f32 + 1.0 / (1u64 << register_3522) as f32 + 1.0 / (1u64 << register_3523) as f32;

	let [register_3524, register_3525, register_3526, register_3527] = split_registers::<4>(words[881]);
	raw_estimate += 1.0 / (1u64 << register_3524) as f32 + 1.0 / (1u64 << register_3525) as f32 + 1.0 / (1u64 << register_3526) as f32 + 1.0 / (1u64 << register_3527) as f32;

	let [register_3528, register_3529, register_3530, register_3531] = split_registers::<4>(words[882]);
	raw_estimate += 1.0 / (1u64 << register_3528) as f32 + 1.0 / (1u64 << register_3529) as f32 + 1.0 / (1u64 << register_3530) as f32 + 1.0 / (1u64 << register_3531) as f32;

	let [register_3532, register_3533, register_3534, register_3535] = split_registers::<4>(words[883]);
	raw_estimate += 1.0 / (1u64 << register_3532) as f32 + 1.0 / (1u64 << register_3533) as f32 + 1.0 / (1u64 << register_3534) as f32 + 1.0 / (1u64 << register_3535) as f32;

	let [register_3536, register_3537, register_3538, register_3539] = split_registers::<4>(words[884]);
	raw_estimate += 1.0 / (1u64 << register_3536) as f32 + 1.0 / (1u64 << register_3537) as f32 + 1.0 / (1u64 << register_3538) as f32 + 1.0 / (1u64 << register_3539) as f32;

	let [register_3540, register_3541, register_3542, register_3543] = split_registers::<4>(words[885]);
	raw_estimate += 1.0 / (1u64 << register_3540) as f32 + 1.0 / (1u64 << register_3541) as f32 + 1.0 / (1u64 << register_3542) as f32 + 1.0 / (1u64 << register_3543) as f32;

	let [register_3544, register_3545, register_3546, register_3547] = split_registers::<4>(words[886]);
	raw_estimate += 1.0 / (1u64 << register_3544) as f32 + 1.0 / (1u64 << register_3545) as f32 + 1.0 / (1u64 << register_3546) as f32 + 1.0 / (1u64 << register_3547) as f32;

	let [register_3548, register_3549, register_3550, register_3551] = split_registers::<4>(words[887]);
	raw_estimate += 1.0 / (1u64 << register_3548) as f32 + 1.0 / (1u64 << register_3549) as f32 + 1.0 / (1u64 << register_3550) as f32 + 1.0 / (1u64 << register_3551) as f32;

	let [register_3552, register_3553, register_3554, register_3555] = split_registers::<4>(words[888]);
	raw_estimate += 1.0 / (1u64 << register_3552) as f32 + 1.0 / (1u64 << register_3553) as f32 + 1.0 / (1u64 << register_3554) as f32 + 1.0 / (1u64 << register_3555) as f32;

	let [register_3556, register_3557, register_3558, register_3559] = split_registers::<4>(words[889]);
	raw_estimate += 1.0 / (1u64 << register_3556) as f32 + 1.0 / (1u64 << register_3557) as f32 + 1.0 / (1u64 << register_3558) as f32 + 1.0 / (1u64 << register_3559) as f32;

	let [register_3560, register_3561, register_3562, register_3563] = split_registers::<4>(words[890]);
	raw_estimate += 1.0 / (1u64 << register_3560) as f32 + 1.0 / (1u64 << register_3561) as f32 + 1.0 / (1u64 << register_3562) as f32 + 1.0 / (1u64 << register_3563) as f32;

	let [register_3564, register_3565, register_3566, register_3567] = split_registers::<4>(words[891]);
	raw_estimate += 1.0 / (1u64 << register_3564) as f32 + 1.0 / (1u64 << register_3565) as f32 + 1.0 / (1u64 << register_3566) as f32 + 1.0 / (1u64 << register_3567) as f32;

	let [register_3568, register_3569, register_3570, register_3571] = split_registers::<4>(words[892]);
	raw_estimate += 1.0 / (1u64 << register_3568) as f32 + 1.0 / (1u64 << register_3569) as f32 + 1.0 / (1u64 << register_3570) as f32 + 1.0 / (1u64 << register_3571) as f32;

	let [register_3572, register_3573, register_3574, register_3575] = split_registers::<4>(words[893]);
	raw_estimate += 1.0 / (1u64 << register_3572) as f32 + 1.0 / (1u64 << register_3573) as f32 + 1.0 / (1u64 << register_3574) as f32 + 1.0 / (1u64 << register_3575) as f32;

	let [register_3576, register_3577, register_3578, register_3579] = split_registers::<4>(words[894]);
	raw_estimate += 1.0 / (1u64 << register_3576) as f32 + 1.0 / (1u64 << register_3577) as f32 + 1.0 / (1u64 << register_3578) as f32 + 1.0 / (1u64 << register_3579) as f32;

	let [register_3580, register_3581, register_3582, register_3583] = split_registers::<4>(words[895]);
	raw_estimate += 1.0 / (1u64 << register_3580) as f32 + 1.0 / (1u64 << register_3581) as f32 + 1.0 / (1u64 << register_3582) as f32 + 1.0 / (1u64 << register_3583) as f32;

	let [register_3584, register_3585, register_3586, register_3587] = split_registers::<4>(words[896]);
	raw_estimate += 1.0 / (1u64 << register_3584) as f32 + 1.0 / (1u64 << register_3585) as f32 + 1.0 / (1u64 << register_3586) as f32 + 1.0 / (1u64 << register_3587) as f32;

	let [register_3588, register_3589, register_3590, register_3591] = split_registers::<4>(words[897]);
	raw_estimate += 1.0 / (1u64 << register_3588) as f32 + 1.0 / (1u64 << register_3589) as f32 + 1.0 / (1u64 << register_3590) as f32 + 1.0 / (1u64 << register_3591) as f32;

	let [register_3592, register_3593, register_3594, register_3595] = split_registers::<4>(words[898]);
	raw_estimate += 1.0 / (1u64 << register_3592) as f32 + 1.0 / (1u64 << register_3593) as f32 + 1.0 / (1u64 << register_3594) as f32 + 1.0 / (1u64 << register_3595) as f32;

	let [register_3596, register_3597, register_3598, register_3599] = split_registers::<4>(words[899]);
	raw_estimate += 1.0 / (1u64 << register_3596) as f32 + 1.0 / (1u64 << register_3597) as f32 + 1.0 / (1u64 << register_3598) as f32 + 1.0 / (1u64 << register_3599) as f32;

	let [register_3600, register_3601, register_3602, register_3603] = split_registers::<4>(words[900]);
	raw_estimate += 1.0 / (1u64 << register_3600) as f32 + 1.0 / (1u64 << register_3601) as f32 + 1.0 / (1u64 << register_3602) as f32 + 1.0 / (1u64 << register_3603) as f32;

	let [register_3604, register_3605, register_3606, register_3607] = split_registers::<4>(words[901]);
	raw_estimate += 1.0 / (1u64 << register_3604) as f32 + 1.0 / (1u64 << register_3605) as f32 + 1.0 / (1u64 << register_3606) as f32 + 1.0 / (1u64 << register_3607) as f32;

	let [register_3608, register_3609, register_3610, register_3611] = split_registers::<4>(words[902]);
	raw_estimate += 1.0 / (1u64 << register_3608) as f32 + 1.0 / (1u64 << register_3609) as f32 + 1.0 / (1u64 << register_3610) as f32 + 1.0 / (1u64 << register_3611) as f32;

	let [register_3612, register_3613, register_3614, register_3615] = split_registers::<4>(words[903]);
	raw_estimate += 1.0 / (1u64 << register_3612) as f32 + 1.0 / (1u64 << register_3613) as f32 + 1.0 / (1u64 << register_3614) as f32 + 1.0 / (1u64 << register_3615) as f32;

	let [register_3616, register_3617, register_3618, register_3619] = split_registers::<4>(words[904]);
	raw_estimate += 1.0 / (1u64 << register_3616) as f32 + 1.0 / (1u64 << register_3617) as f32 + 1.0 / (1u64 << register_3618) as f32 + 1.0 / (1u64 << register_3619) as f32;

	let [register_3620, register_3621, register_3622, register_3623] = split_registers::<4>(words[905]);
	raw_estimate += 1.0 / (1u64 << register_3620) as f32 + 1.0 / (1u64 << register_3621) as f32 + 1.0 / (1u64 << register_3622) as f32 + 1.0 / (1u64 << register_3623) as f32;

	let [register_3624, register_3625, register_3626, register_3627] = split_registers::<4>(words[906]);
	raw_estimate += 1.0 / (1u64 << register_3624) as f32 + 1.0 / (1u64 << register_3625) as f32 + 1.0 / (1u64 << register_3626) as f32 + 1.0 / (1u64 << register_3627) as f32;

	let [register_3628, register_3629, register_3630, register_3631] = split_registers::<4>(words[907]);
	raw_estimate += 1.0 / (1u64 << register_3628) as f32 + 1.0 / (1u64 << register_3629) as f32 + 1.0 / (1u64 << register_3630) as f32 + 1.0 / (1u64 << register_3631) as f32;

	let [register_3632, register_3633, register_3634, register_3635] = split_registers::<4>(words[908]);
	raw_estimate += 1.0 / (1u64 << register_3632) as f32 + 1.0 / (1u64 << register_3633) as f32 + 1.0 / (1u64 << register_3634) as f32 + 1.0 / (1u64 << register_3635) as f32;

	let [register_3636, register_3637, register_3638, register_3639] = split_registers::<4>(words[909]);
	raw_estimate += 1.0 / (1u64 << register_3636) as f32 + 1.0 / (1u64 << register_3637) as f32 + 1.0 / (1u64 << register_3638) as f32 + 1.0 / (1u64 << register_3639) as f32;

	let [register_3640, register_3641, register_3642, register_3643] = split_registers::<4>(words[910]);
	raw_estimate += 1.0 / (1u64 << register_3640) as f32 + 1.0 / (1u64 << register_3641) as f32 + 1.0 / (1u64 << register_3642) as f32 + 1.0 / (1u64 << register_3643) as f32;

	let [register_3644, register_3645, register_3646, register_3647] = split_registers::<4>(words[911]);
	raw_estimate += 1.0 / (1u64 << register_3644) as f32 + 1.0 / (1u64 << register_3645) as f32 + 1.0 / (1u64 << register_3646) as f32 + 1.0 / (1u64 << register_3647) as f32;

	let [register_3648, register_3649, register_3650, register_3651] = split_registers::<4>(words[912]);
	raw_estimate += 1.0 / (1u64 << register_3648) as f32 + 1.0 / (1u64 << register_3649) as f32 + 1.0 / (1u64 << register_3650) as f32 + 1.0 / (1u64 << register_3651) as f32;

	let [register_3652, register_3653, register_3654, register_3655] = split_registers::<4>(words[913]);
	raw_estimate += 1.0 / (1u64 << register_3652) as f32 + 1.0 / (1u64 << register_3653) as f32 + 1.0 / (1u64 << register_3654) as f32 + 1.0 / (1u64 << register_3655) as f32;

	let [register_3656, register_3657, register_3658, register_3659] = split_registers::<4>(words[914]);
	raw_estimate += 1.0 / (1u64 << register_3656) as f32 + 1.0 / (1u64 << register_3657) as f32 + 1.0 / (1u64 << register_3658) as f32 + 1.0 / (1u64 << register_3659) as f32;

	let [register_3660, register_3661, register_3662, register_3663] = split_registers::<4>(words[915]);
	raw_estimate += 1.0 / (1u64 << register_3660) as f32 + 1.0 / (1u64 << register_3661) as f32 + 1.0 / (1u64 << register_3662) as f32 + 1.0 / (1u64 << register_3663) as f32;

	let [register_3664, register_3665, register_3666, register_3667] = split_registers::<4>(words[916]);
	raw_estimate += 1.0 / (1u64 << register_3664) as f32 + 1.0 / (1u64 << register_3665) as f32 + 1.0 / (1u64 << register_3666) as f32 + 1.0 / (1u64 << register_3667) as f32;

	let [register_3668, register_3669, register_3670, register_3671] = split_registers::<4>(words[917]);
	raw_estimate += 1.0 / (1u64 << register_3668) as f32 + 1.0 / (1u64 << register_3669) as f32 + 1.0 / (1u64 << register_3670) as f32 + 1.0 / (1u64 << register_3671) as f32;

	let [register_3672, register_3673, register_3674, register_3675] = split_registers::<4>(words[918]);
	raw_estimate += 1.0 / (1u64 << register_3672) as f32 + 1.0 / (1u64 << register_3673) as f32 + 1.0 / (1u64 << register_3674) as f32 + 1.0 / (1u64 << register_3675) as f32;

	let [register_3676, register_3677, register_3678, register_3679] = split_registers::<4>(words[919]);
	raw_estimate += 1.0 / (1u64 << register_3676) as f32 + 1.0 / (1u64 << register_3677) as f32 + 1.0 / (1u64 << register_3678) as f32 + 1.0 / (1u64 << register_3679) as f32;

	let [register_3680, register_3681, register_3682, register_3683] = split_registers::<4>(words[920]);
	raw_estimate += 1.0 / (1u64 << register_3680) as f32 + 1.0 / (1u64 << register_3681) as f32 + 1.0 / (1u64 << register_3682) as f32 + 1.0 / (1u64 << register_3683) as f32;

	let [register_3684, register_3685, register_3686, register_3687] = split_registers::<4>(words[921]);
	raw_estimate += 1.0 / (1u64 << register_3684) as f32 + 1.0 / (1u64 << register_3685) as f32 + 1.0 / (1u64 << register_3686) as f32 + 1.0 / (1u64 << register_3687) as f32;

	let [register_3688, register_3689, register_3690, register_3691] = split_registers::<4>(words[922]);
	raw_estimate += 1.0 / (1u64 << register_3688) as f32 + 1.0 / (1u64 << register_3689) as f32 + 1.0 / (1u64 << register_3690) as f32 + 1.0 / (1u64 << register_3691) as f32;

	let [register_3692, register_3693, register_3694, register_3695] = split_registers::<4>(words[923]);
	raw_estimate += 1.0 / (1u64 << register_3692) as f32 + 1.0 / (1u64 << register_3693) as f32 + 1.0 / (1u64 << register_3694) as f32 + 1.0 / (1u64 << register_3695) as f32;

	let [register_3696, register_3697, register_3698, register_3699] = split_registers::<4>(words[924]);
	raw_estimate += 1.0 / (1u64 << register_3696) as f32 + 1.0 / (1u64 << register_3697) as f32 + 1.0 / (1u64 << register_3698) as f32 + 1.0 / (1u64 << register_3699) as f32;

	let [register_3700, register_3701, register_3702, register_3703] = split_registers::<4>(words[925]);
	raw_estimate += 1.0 / (1u64 << register_3700) as f32 + 1.0 / (1u64 << register_3701) as f32 + 1.0 / (1u64 << register_3702) as f32 + 1.0 / (1u64 << register_3703) as f32;

	let [register_3704, register_3705, register_3706, register_3707] = split_registers::<4>(words[926]);
	raw_estimate += 1.0 / (1u64 << register_3704) as f32 + 1.0 / (1u64 << register_3705) as f32 + 1.0 / (1u64 << register_3706) as f32 + 1.0 / (1u64 << register_3707) as f32;

	let [register_3708, register_3709, register_3710, register_3711] = split_registers::<4>(words[927]);
	raw_estimate += 1.0 / (1u64 << register_3708) as f32 + 1.0 / (1u64 << register_3709) as f32 + 1.0 / (1u64 << register_3710) as f32 + 1.0 / (1u64 << register_3711) as f32;

	let [register_3712, register_3713, register_3714, register_3715] = split_registers::<4>(words[928]);
	raw_estimate += 1.0 / (1u64 << register_3712) as f32 + 1.0 / (1u64 << register_3713) as f32 + 1.0 / (1u64 << register_3714) as f32 + 1.0 / (1u64 << register_3715) as f32;

	let [register_3716, register_3717, register_3718, register_3719] = split_registers::<4>(words[929]);
	raw_estimate += 1.0 / (1u64 << register_3716) as f32 + 1.0 / (1u64 << register_3717) as f32 + 1.0 / (1u64 << register_3718) as f32 + 1.0 / (1u64 << register_3719) as f32;

	let [register_3720, register_3721, register_3722, register_3723] = split_registers::<4>(words[930]);
	raw_estimate += 1.0 / (1u64 << register_3720) as f32 + 1.0 / (1u64 << register_3721) as f32 + 1.0 / (1u64 << register_3722) as f32 + 1.0 / (1u64 << register_3723) as f32;

	let [register_3724, register_3725, register_3726, register_3727] = split_registers::<4>(words[931]);
	raw_estimate += 1.0 / (1u64 << register_3724) as f32 + 1.0 / (1u64 << register_3725) as f32 + 1.0 / (1u64 << register_3726) as f32 + 1.0 / (1u64 << register_3727) as f32;

	let [register_3728, register_3729, register_3730, register_3731] = split_registers::<4>(words[932]);
	raw_estimate += 1.0 / (1u64 << register_3728) as f32 + 1.0 / (1u64 << register_3729) as f32 + 1.0 / (1u64 << register_3730) as f32 + 1.0 / (1u64 << register_3731) as f32;

	let [register_3732, register_3733, register_3734, register_3735] = split_registers::<4>(words[933]);
	raw_estimate += 1.0 / (1u64 << register_3732) as f32 + 1.0 / (1u64 << register_3733) as f32 + 1.0 / (1u64 << register_3734) as f32 + 1.0 / (1u64 << register_3735) as f32;

	let [register_3736, register_3737, register_3738, register_3739] = split_registers::<4>(words[934]);
	raw_estimate += 1.0 / (1u64 << register_3736) as f32 + 1.0 / (1u64 << register_3737) as f32 + 1.0 / (1u64 << register_3738) as f32 + 1.0 / (1u64 << register_3739) as f32;

	let [register_3740, register_3741, register_3742, register_3743] = split_registers::<4>(words[935]);
	raw_estimate += 1.0 / (1u64 << register_3740) as f32 + 1.0 / (1u64 << register_3741) as f32 + 1.0 / (1u64 << register_3742) as f32 + 1.0 / (1u64 << register_3743) as f32;

	let [register_3744, register_3745, register_3746, register_3747] = split_registers::<4>(words[936]);
	raw_estimate += 1.0 / (1u64 << register_3744) as f32 + 1.0 / (1u64 << register_3745) as f32 + 1.0 / (1u64 << register_3746) as f32 + 1.0 / (1u64 << register_3747) as f32;

	let [register_3748, register_3749, register_3750, register_3751] = split_registers::<4>(words[937]);
	raw_estimate += 1.0 / (1u64 << register_3748) as f32 + 1.0 / (1u64 << register_3749) as f32 + 1.0 / (1u64 << register_3750) as f32 + 1.0 / (1u64 << register_3751) as f32;

	let [register_3752, register_3753, register_3754, register_3755] = split_registers::<4>(words[938]);
	raw_estimate += 1.0 / (1u64 << register_3752) as f32 + 1.0 / (1u64 << register_3753) as f32 + 1.0 / (1u64 << register_3754) as f32 + 1.0 / (1u64 << register_3755) as f32;

	let [register_3756, register_3757, register_3758, register_3759] = split_registers::<4>(words[939]);
	raw_estimate += 1.0 / (1u64 << register_3756) as f32 + 1.0 / (1u64 << register_3757) as f32 + 1.0 / (1u64 << register_3758) as f32 + 1.0 / (1u64 << register_3759) as f32;

	let [register_3760, register_3761, register_3762, register_3763] = split_registers::<4>(words[940]);
	raw_estimate += 1.0 / (1u64 << register_3760) as f32 + 1.0 / (1u64 << register_3761) as f32 + 1.0 / (1u64 << register_3762) as f32 + 1.0 / (1u64 << register_3763) as f32;

	let [register_3764, register_3765, register_3766, register_3767] = split_registers::<4>(words[941]);
	raw_estimate += 1.0 / (1u64 << register_3764) as f32 + 1.0 / (1u64 << register_3765) as f32 + 1.0 / (1u64 << register_3766) as f32 + 1.0 / (1u64 << register_3767) as f32;

	let [register_3768, register_3769, register_3770, register_3771] = split_registers::<4>(words[942]);
	raw_estimate += 1.0 / (1u64 << register_3768) as f32 + 1.0 / (1u64 << register_3769) as f32 + 1.0 / (1u64 << register_3770) as f32 + 1.0 / (1u64 << register_3771) as f32;

	let [register_3772, register_3773, register_3774, register_3775] = split_registers::<4>(words[943]);
	raw_estimate += 1.0 / (1u64 << register_3772) as f32 + 1.0 / (1u64 << register_3773) as f32 + 1.0 / (1u64 << register_3774) as f32 + 1.0 / (1u64 << register_3775) as f32;

	let [register_3776, register_3777, register_3778, register_3779] = split_registers::<4>(words[944]);
	raw_estimate += 1.0 / (1u64 << register_3776) as f32 + 1.0 / (1u64 << register_3777) as f32 + 1.0 / (1u64 << register_3778) as f32 + 1.0 / (1u64 << register_3779) as f32;

	let [register_3780, register_3781, register_3782, register_3783] = split_registers::<4>(words[945]);
	raw_estimate += 1.0 / (1u64 << register_3780) as f32 + 1.0 / (1u64 << register_3781) as f32 + 1.0 / (1u64 << register_3782) as f32 + 1.0 / (1u64 << register_3783) as f32;

	let [register_3784, register_3785, register_3786, register_3787] = split_registers::<4>(words[946]);
	raw_estimate += 1.0 / (1u64 << register_3784) as f32 + 1.0 / (1u64 << register_3785) as f32 + 1.0 / (1u64 << register_3786) as f32 + 1.0 / (1u64 << register_3787) as f32;

	let [register_3788, register_3789, register_3790, register_3791] = split_registers::<4>(words[947]);
	raw_estimate += 1.0 / (1u64 << register_3788) as f32 + 1.0 / (1u64 << register_3789) as f32 + 1.0 / (1u64 << register_3790) as f32 + 1.0 / (1u64 << register_3791) as f32;

	let [register_3792, register_3793, register_3794, register_3795] = split_registers::<4>(words[948]);
	raw_estimate += 1.0 / (1u64 << register_3792) as f32 + 1.0 / (1u64 << register_3793) as f32 + 1.0 / (1u64 << register_3794) as f32 + 1.0 / (1u64 << register_3795) as f32;

	let [register_3796, register_3797, register_3798, register_3799] = split_registers::<4>(words[949]);
	raw_estimate += 1.0 / (1u64 << register_3796) as f32 + 1.0 / (1u64 << register_3797) as f32 + 1.0 / (1u64 << register_3798) as f32 + 1.0 / (1u64 << register_3799) as f32;

	let [register_3800, register_3801, register_3802, register_3803] = split_registers::<4>(words[950]);
	raw_estimate += 1.0 / (1u64 << register_3800) as f32 + 1.0 / (1u64 << register_3801) as f32 + 1.0 / (1u64 << register_3802) as f32 + 1.0 / (1u64 << register_3803) as f32;

	let [register_3804, register_3805, register_3806, register_3807] = split_registers::<4>(words[951]);
	raw_estimate += 1.0 / (1u64 << register_3804) as f32 + 1.0 / (1u64 << register_3805) as f32 + 1.0 / (1u64 << register_3806) as f32 + 1.0 / (1u64 << register_3807) as f32;

	let [register_3808, register_3809, register_3810, register_3811] = split_registers::<4>(words[952]);
	raw_estimate += 1.0 / (1u64 << register_3808) as f32 + 1.0 / (1u64 << register_3809) as f32 + 1.0 / (1u64 << register_3810) as f32 + 1.0 / (1u64 << register_3811) as f32;

	let [register_3812, register_3813, register_3814, register_3815] = split_registers::<4>(words[953]);
	raw_estimate += 1.0 / (1u64 << register_3812) as f32 + 1.0 / (1u64 << register_3813) as f32 + 1.0 / (1u64 << register_3814) as f32 + 1.0 / (1u64 << register_3815) as f32;

	let [register_3816, register_3817, register_3818, register_3819] = split_registers::<4>(words[954]);
	raw_estimate += 1.0 / (1u64 << register_3816) as f32 + 1.0 / (1u64 << register_3817) as f32 + 1.0 / (1u64 << register_3818) as f32 + 1.0 / (1u64 << register_3819) as f32;

	let [register_3820, register_3821, register_3822, register_3823] = split_registers::<4>(words[955]);
	raw_estimate += 1.0 / (1u64 << register_3820) as f32 + 1.0 / (1u64 << register_3821) as f32 + 1.0 / (1u64 << register_3822) as f32 + 1.0 / (1u64 << register_3823) as f32;

	let [register_3824, register_3825, register_3826, register_3827] = split_registers::<4>(words[956]);
	raw_estimate += 1.0 / (1u64 << register_3824) as f32 + 1.0 / (1u64 << register_3825) as f32 + 1.0 / (1u64 << register_3826) as f32 + 1.0 / (1u64 << register_3827) as f32;

	let [register_3828, register_3829, register_3830, register_3831] = split_registers::<4>(words[957]);
	raw_estimate += 1.0 / (1u64 << register_3828) as f32 + 1.0 / (1u64 << register_3829) as f32 + 1.0 / (1u64 << register_3830) as f32 + 1.0 / (1u64 << register_3831) as f32;

	let [register_3832, register_3833, register_3834, register_3835] = split_registers::<4>(words[958]);
	raw_estimate += 1.0 / (1u64 << register_3832) as f32 + 1.0 / (1u64 << register_3833) as f32 + 1.0 / (1u64 << register_3834) as f32 + 1.0 / (1u64 << register_3835) as f32;

	let [register_3836, register_3837, register_3838, register_3839] = split_registers::<4>(words[959]);
	raw_estimate += 1.0 / (1u64 << register_3836) as f32 + 1.0 / (1u64 << register_3837) as f32 + 1.0 / (1u64 << register_3838) as f32 + 1.0 / (1u64 << register_3839) as f32;

	let [register_3840, register_3841, register_3842, register_3843] = split_registers::<4>(words[960]);
	raw_estimate += 1.0 / (1u64 << register_3840) as f32 + 1.0 / (1u64 << register_3841) as f32 + 1.0 / (1u64 << register_3842) as f32 + 1.0 / (1u64 << register_3843) as f32;

	let [register_3844, register_3845, register_3846, register_3847] = split_registers::<4>(words[961]);
	raw_estimate += 1.0 / (1u64 << register_3844) as f32 + 1.0 / (1u64 << register_3845) as f32 + 1.0 / (1u64 << register_3846) as f32 + 1.0 / (1u64 << register_3847) as f32;

	let [register_3848, register_3849, register_3850, register_3851] = split_registers::<4>(words[962]);
	raw_estimate += 1.0 / (1u64 << register_3848) as f32 + 1.0 / (1u64 << register_3849) as f32 + 1.0 / (1u64 << register_3850) as f32 + 1.0 / (1u64 << register_3851) as f32;

	let [register_3852, register_3853, register_3854, register_3855] = split_registers::<4>(words[963]);
	raw_estimate += 1.0 / (1u64 << register_3852) as f32 + 1.0 / (1u64 << register_3853) as f32 + 1.0 / (1u64 << register_3854) as f32 + 1.0 / (1u64 << register_3855) as f32;

	let [register_3856, register_3857, register_3858, register_3859] = split_registers::<4>(words[964]);
	raw_estimate += 1.0 / (1u64 << register_3856) as f32 + 1.0 / (1u64 << register_3857) as f32 + 1.0 / (1u64 << register_3858) as f32 + 1.0 / (1u64 << register_3859) as f32;

	let [register_3860, register_3861, register_3862, register_3863] = split_registers::<4>(words[965]);
	raw_estimate += 1.0 / (1u64 << register_3860) as f32 + 1.0 / (1u64 << register_3861) as f32 + 1.0 / (1u64 << register_3862) as f32 + 1.0 / (1u64 << register_3863) as f32;

	let [register_3864, register_3865, register_3866, register_3867] = split_registers::<4>(words[966]);
	raw_estimate += 1.0 / (1u64 << register_3864) as f32 + 1.0 / (1u64 << register_3865) as f32 + 1.0 / (1u64 << register_3866) as f32 + 1.0 / (1u64 << register_3867) as f32;

	let [register_3868, register_3869, register_3870, register_3871] = split_registers::<4>(words[967]);
	raw_estimate += 1.0 / (1u64 << register_3868) as f32 + 1.0 / (1u64 << register_3869) as f32 + 1.0 / (1u64 << register_3870) as f32 + 1.0 / (1u64 << register_3871) as f32;

	let [register_3872, register_3873, register_3874, register_3875] = split_registers::<4>(words[968]);
	raw_estimate += 1.0 / (1u64 << register_3872) as f32 + 1.0 / (1u64 << register_3873) as f32 + 1.0 / (1u64 << register_3874) as f32 + 1.0 / (1u64 << register_3875) as f32;

	let [register_3876, register_3877, register_3878, register_3879] = split_registers::<4>(words[969]);
	raw_estimate += 1.0 / (1u64 << register_3876) as f32 + 1.0 / (1u64 << register_3877) as f32 + 1.0 / (1u64 << register_3878) as f32 + 1.0 / (1u64 << register_3879) as f32;

	let [register_3880, register_3881, register_3882, register_3883] = split_registers::<4>(words[970]);
	raw_estimate += 1.0 / (1u64 << register_3880) as f32 + 1.0 / (1u64 << register_3881) as f32 + 1.0 / (1u64 << register_3882) as f32 + 1.0 / (1u64 << register_3883) as f32;

	let [register_3884, register_3885, register_3886, register_3887] = split_registers::<4>(words[971]);
	raw_estimate += 1.0 / (1u64 << register_3884) as f32 + 1.0 / (1u64 << register_3885) as f32 + 1.0 / (1u64 << register_3886) as f32 + 1.0 / (1u64 << register_3887) as f32;

	let [register_3888, register_3889, register_3890, register_3891] = split_registers::<4>(words[972]);
	raw_estimate += 1.0 / (1u64 << register_3888) as f32 + 1.0 / (1u64 << register_3889) as f32 + 1.0 / (1u64 << register_3890) as f32 + 1.0 / (1u64 << register_3891) as f32;

	let [register_3892, register_3893, register_3894, register_3895] = split_registers::<4>(words[973]);
	raw_estimate += 1.0 / (1u64 << register_3892) as f32 + 1.0 / (1u64 << register_3893) as f32 + 1.0 / (1u64 << register_3894) as f32 + 1.0 / (1u64 << register_3895) as f32;

	let [register_3896, register_3897, register_3898, register_3899] = split_registers::<4>(words[974]);
	raw_estimate += 1.0 / (1u64 << register_3896) as f32 + 1.0 / (1u64 << register_3897) as f32 + 1.0 / (1u64 << register_3898) as f32 + 1.0 / (1u64 << register_3899) as f32;

	let [register_3900, register_3901, register_3902, register_3903] = split_registers::<4>(words[975]);
	raw_estimate += 1.0 / (1u64 << register_3900) as f32 + 1.0 / (1u64 << register_3901) as f32 + 1.0 / (1u64 << register_3902) as f32 + 1.0 / (1u64 << register_3903) as f32;

	let [register_3904, register_3905, register_3906, register_3907] = split_registers::<4>(words[976]);
	raw_estimate += 1.0 / (1u64 << register_3904) as f32 + 1.0 / (1u64 << register_3905) as f32 + 1.0 / (1u64 << register_3906) as f32 + 1.0 / (1u64 << register_3907) as f32;

	let [register_3908, register_3909, register_3910, register_3911] = split_registers::<4>(words[977]);
	raw_estimate += 1.0 / (1u64 << register_3908) as f32 + 1.0 / (1u64 << register_3909) as f32 + 1.0 / (1u64 << register_3910) as f32 + 1.0 / (1u64 << register_3911) as f32;

	let [register_3912, register_3913, register_3914, register_3915] = split_registers::<4>(words[978]);
	raw_estimate += 1.0 / (1u64 << register_3912) as f32 + 1.0 / (1u64 << register_3913) as f32 + 1.0 / (1u64 << register_3914) as f32 + 1.0 / (1u64 << register_3915) as f32;

	let [register_3916, register_3917, register_3918, register_3919] = split_registers::<4>(words[979]);
	raw_estimate += 1.0 / (1u64 << register_3916) as f32 + 1.0 / (1u64 << register_3917) as f32 + 1.0 / (1u64 << register_3918) as f32 + 1.0 / (1u64 << register_3919) as f32;

	let [register_3920, register_3921, register_3922, register_3923] = split_registers::<4>(words[980]);
	raw_estimate += 1.0 / (1u64 << register_3920) as f32 + 1.0 / (1u64 << register_3921) as f32 + 1.0 / (1u64 << register_3922) as f32 + 1.0 / (1u64 << register_3923) as f32;

	let [register_3924, register_3925, register_3926, register_3927] = split_registers::<4>(words[981]);
	raw_estimate += 1.0 / (1u64 << register_3924) as f32 + 1.0 / (1u64 << register_3925) as f32 + 1.0 / (1u64 << register_3926) as f32 + 1.0 / (1u64 << register_3927) as f32;

	let [register_3928, register_3929, register_3930, register_3931] = split_registers::<4>(words[982]);
	raw_estimate += 1.0 / (1u64 << register_3928) as f32 + 1.0 / (1u64 << register_3929) as f32 + 1.0 / (1u64 << register_3930) as f32 + 1.0 / (1u64 << register_3931) as f32;

	let [register_3932, register_3933, register_3934, register_3935] = split_registers::<4>(words[983]);
	raw_estimate += 1.0 / (1u64 << register_3932) as f32 + 1.0 / (1u64 << register_3933) as f32 + 1.0 / (1u64 << register_3934) as f32 + 1.0 / (1u64 << register_3935) as f32;

	let [register_3936, register_3937, register_3938, register_3939] = split_registers::<4>(words[984]);
	raw_estimate += 1.0 / (1u64 << register_3936) as f32 + 1.0 / (1u64 << register_3937) as f32 + 1.0 / (1u64 << register_3938) as f32 + 1.0 / (1u64 << register_3939) as f32;

	let [register_3940, register_3941, register_3942, register_3943] = split_registers::<4>(words[985]);
	raw_estimate += 1.0 / (1u64 << register_3940) as f32 + 1.0 / (1u64 << register_3941) as f32 + 1.0 / (1u64 << register_3942) as f32 + 1.0 / (1u64 << register_3943) as f32;

	let [register_3944, register_3945, register_3946, register_3947] = split_registers::<4>(words[986]);
	raw_estimate += 1.0 / (1u64 << register_3944) as f32 + 1.0 / (1u64 << register_3945) as f32 + 1.0 / (1u64 << register_3946) as f32 + 1.0 / (1u64 << register_3947) as f32;

	let [register_3948, register_3949, register_3950, register_3951] = split_registers::<4>(words[987]);
	raw_estimate += 1.0 / (1u64 << register_3948) as f32 + 1.0 / (1u64 << register_3949) as f32 + 1.0 / (1u64 << register_3950) as f32 + 1.0 / (1u64 << register_3951) as f32;

	let [register_3952, register_3953, register_3954, register_3955] = split_registers::<4>(words[988]);
	raw_estimate += 1.0 / (1u64 << register_3952) as f32 + 1.0 / (1u64 << register_3953) as f32 + 1.0 / (1u64 << register_3954) as f32 + 1.0 / (1u64 << register_3955) as f32;

	let [register_3956, register_3957, register_3958, register_3959] = split_registers::<4>(words[989]);
	raw_estimate += 1.0 / (1u64 << register_3956) as f32 + 1.0 / (1u64 << register_3957) as f32 + 1.0 / (1u64 << register_3958) as f32 + 1.0 / (1u64 << register_3959) as f32;

	let [register_3960, register_3961, register_3962, register_3963] = split_registers::<4>(words[990]);
	raw_estimate += 1.0 / (1u64 << register_3960) as f32 + 1.0 / (1u64 << register_3961) as f32 + 1.0 / (1u64 << register_3962) as f32 + 1.0 / (1u64 << register_3963) as f32;

	let [register_3964, register_3965, register_3966, register_3967] = split_registers::<4>(words[991]);
	raw_estimate += 1.0 / (1u64 << register_3964) as f32 + 1.0 / (1u64 << register_3965) as f32 + 1.0 / (1u64 << register_3966) as f32 + 1.0 / (1u64 << register_3967) as f32;

	let [register_3968, register_3969, register_3970, register_3971] = split_registers::<4>(words[992]);
	raw_estimate += 1.0 / (1u64 << register_3968) as f32 + 1.0 / (1u64 << register_3969) as f32 + 1.0 / (1u64 << register_3970) as f32 + 1.0 / (1u64 << register_3971) as f32;

	let [register_3972, register_3973, register_3974, register_3975] = split_registers::<4>(words[993]);
	raw_estimate += 1.0 / (1u64 << register_3972) as f32 + 1.0 / (1u64 << register_3973) as f32 + 1.0 / (1u64 << register_3974) as f32 + 1.0 / (1u64 << register_3975) as f32;

	let [register_3976, register_3977, register_3978, register_3979] = split_registers::<4>(words[994]);
	raw_estimate += 1.0 / (1u64 << register_3976) as f32 + 1.0 / (1u64 << register_3977) as f32 + 1.0 / (1u64 << register_3978) as f32 + 1.0 / (1u64 << register_3979) as f32;

	let [register_3980, register_3981, register_3982, register_3983] = split_registers::<4>(words[995]);
	raw_estimate += 1.0 / (1u64 << register_3980) as f32 + 1.0 / (1u64 << register_3981) as f32 + 1.0 / (1u64 << register_3982) as f32 + 1.0 / (1u64 << register_3983) as f32;

	let [register_3984, register_3985, register_3986, register_3987] = split_registers::<4>(words[996]);
	raw_estimate += 1.0 / (1u64 << register_3984) as f32 + 1.0 / (1u64 << register_3985) as f32 + 1.0 / (1u64 << register_3986) as f32 + 1.0 / (1u64 << register_3987) as f32;

	let [register_3988, register_3989, register_3990, register_3991] = split_registers::<4>(words[997]);
	raw_estimate += 1.0 / (1u64 << register_3988) as f32 + 1.0 / (1u64 << register_3989) as f32 + 1.0 / (1u64 << register_3990) as f32 + 1.0 / (1u64 << register_3991) as f32;

	let [register_3992, register_3993, register_3994, register_3995] = split_registers::<4>(words[998]);
	raw_estimate += 1.0 / (1u64 << register_3992) as f32 + 1.0 / (1u64 << register_3993) as f32 + 1.0 / (1u64 << register_3994) as f32 + 1.0 / (1u64 << register_3995) as f32;

	let [register_3996, register_3997, register_3998, register_3999] = split_registers::<4>(words[999]);
	raw_estimate += 1.0 / (1u64 << register_3996) as f32 + 1.0 / (1u64 << register_3997) as f32 + 1.0 / (1u64 << register_3998) as f32 + 1.0 / (1u64 << register_3999) as f32;

	let [register_4000, register_4001, register_4002, register_4003] = split_registers::<4>(words[1000]);
	raw_estimate += 1.0 / (1u64 << register_4000) as f32 + 1.0 / (1u64 << register_4001) as f32 + 1.0 / (1u64 << register_4002) as f32 + 1.0 / (1u64 << register_4003) as f32;

	let [register_4004, register_4005, register_4006, register_4007] = split_registers::<4>(words[1001]);
	raw_estimate += 1.0 / (1u64 << register_4004) as f32 + 1.0 / (1u64 << register_4005) as f32 + 1.0 / (1u64 << register_4006) as f32 + 1.0 / (1u64 << register_4007) as f32;

	let [register_4008, register_4009, register_4010, register_4011] = split_registers::<4>(words[1002]);
	raw_estimate += 1.0 / (1u64 << register_4008) as f32 + 1.0 / (1u64 << register_4009) as f32 + 1.0 / (1u64 << register_4010) as f32 + 1.0 / (1u64 << register_4011) as f32;

	let [register_4012, register_4013, register_4014, register_4015] = split_registers::<4>(words[1003]);
	raw_estimate += 1.0 / (1u64 << register_4012) as f32 + 1.0 / (1u64 << register_4013) as f32 + 1.0 / (1u64 << register_4014) as f32 + 1.0 / (1u64 << register_4015) as f32;

	let [register_4016, register_4017, register_4018, register_4019] = split_registers::<4>(words[1004]);
	raw_estimate += 1.0 / (1u64 << register_4016) as f32 + 1.0 / (1u64 << register_4017) as f32 + 1.0 / (1u64 << register_4018) as f32 + 1.0 / (1u64 << register_4019) as f32;

	let [register_4020, register_4021, register_4022, register_4023] = split_registers::<4>(words[1005]);
	raw_estimate += 1.0 / (1u64 << register_4020) as f32 + 1.0 / (1u64 << register_4021) as f32 + 1.0 / (1u64 << register_4022) as f32 + 1.0 / (1u64 << register_4023) as f32;

	let [register_4024, register_4025, register_4026, register_4027] = split_registers::<4>(words[1006]);
	raw_estimate += 1.0 / (1u64 << register_4024) as f32 + 1.0 / (1u64 << register_4025) as f32 + 1.0 / (1u64 << register_4026) as f32 + 1.0 / (1u64 << register_4027) as f32;

	let [register_4028, register_4029, register_4030, register_4031] = split_registers::<4>(words[1007]);
	raw_estimate += 1.0 / (1u64 << register_4028) as f32 + 1.0 / (1u64 << register_4029) as f32 + 1.0 / (1u64 << register_4030) as f32 + 1.0 / (1u64 << register_4031) as f32;

	let [register_4032, register_4033, register_4034, register_4035] = split_registers::<4>(words[1008]);
	raw_estimate += 1.0 / (1u64 << register_4032) as f32 + 1.0 / (1u64 << register_4033) as f32 + 1.0 / (1u64 << register_4034) as f32 + 1.0 / (1u64 << register_4035) as f32;

	let [register_4036, register_4037, register_4038, register_4039] = split_registers::<4>(words[1009]);
	raw_estimate += 1.0 / (1u64 << register_4036) as f32 + 1.0 / (1u64 << register_4037) as f32 + 1.0 / (1u64 << register_4038) as f32 + 1.0 / (1u64 << register_4039) as f32;

	let [register_4040, register_4041, register_4042, register_4043] = split_registers::<4>(words[1010]);
	raw_estimate += 1.0 / (1u64 << register_4040) as f32 + 1.0 / (1u64 << register_4041) as f32 + 1.0 / (1u64 << register_4042) as f32 + 1.0 / (1u64 << register_4043) as f32;

	let [register_4044, register_4045, register_4046, register_4047] = split_registers::<4>(words[1011]);
	raw_estimate += 1.0 / (1u64 << register_4044) as f32 + 1.0 / (1u64 << register_4045) as f32 + 1.0 / (1u64 << register_4046) as f32 + 1.0 / (1u64 << register_4047) as f32;

	let [register_4048, register_4049, register_4050, register_4051] = split_registers::<4>(words[1012]);
	raw_estimate += 1.0 / (1u64 << register_4048) as f32 + 1.0 / (1u64 << register_4049) as f32 + 1.0 / (1u64 << register_4050) as f32 + 1.0 / (1u64 << register_4051) as f32;

	let [register_4052, register_4053, register_4054, register_4055] = split_registers::<4>(words[1013]);
	raw_estimate += 1.0 / (1u64 << register_4052) as f32 + 1.0 / (1u64 << register_4053) as f32 + 1.0 / (1u64 << register_4054) as f32 + 1.0 / (1u64 << register_4055) as f32;

	let [register_4056, register_4057, register_4058, register_4059] = split_registers::<4>(words[1014]);
	raw_estimate += 1.0 / (1u64 << register_4056) as f32 + 1.0 / (1u64 << register_4057) as f32 + 1.0 / (1u64 << register_4058) as f32 + 1.0 / (1u64 << register_4059) as f32;

	let [register_4060, register_4061, register_4062, register_4063] = split_registers::<4>(words[1015]);
	raw_estimate += 1.0 / (1u64 << register_4060) as f32 + 1.0 / (1u64 << register_4061) as f32 + 1.0 / (1u64 << register_4062) as f32 + 1.0 / (1u64 << register_4063) as f32;

	let [register_4064, register_4065, register_4066, register_4067] = split_registers::<4>(words[1016]);
	raw_estimate += 1.0 / (1u64 << register_4064) as f32 + 1.0 / (1u64 << register_4065) as f32 + 1.0 / (1u64 << register_4066) as f32 + 1.0 / (1u64 << register_4067) as f32;

	let [register_4068, register_4069, register_4070, register_4071] = split_registers::<4>(words[1017]);
	raw_estimate += 1.0 / (1u64 << register_4068) as f32 + 1.0 / (1u64 << register_4069) as f32 + 1.0 / (1u64 << register_4070) as f32 + 1.0 / (1u64 << register_4071) as f32;

	let [register_4072, register_4073, register_4074, register_4075] = split_registers::<4>(words[1018]);
	raw_estimate += 1.0 / (1u64 << register_4072) as f32 + 1.0 / (1u64 << register_4073) as f32 + 1.0 / (1u64 << register_4074) as f32 + 1.0 / (1u64 << register_4075) as f32;

	let [register_4076, register_4077, register_4078, register_4079] = split_registers::<4>(words[1019]);
	raw_estimate += 1.0 / (1u64 << register_4076) as f32 + 1.0 / (1u64 << register_4077) as f32 + 1.0 / (1u64 << register_4078) as f32 + 1.0 / (1u64 << register_4079) as f32;

	let [register_4080, register_4081, register_4082, register_4083] = split_registers::<4>(words[1020]);
	raw_estimate += 1.0 / (1u64 << register_4080) as f32 + 1.0 / (1u64 << register_4081) as f32 + 1.0 / (1u64 << register_4082) as f32 + 1.0 / (1u64 << register_4083) as f32;

	let [register_4084, register_4085, register_4086, register_4087] = split_registers::<4>(words[1021]);
	raw_estimate += 1.0 / (1u64 << register_4084) as f32 + 1.0 / (1u64 << register_4085) as f32 + 1.0 / (1u64 << register_4086) as f32 + 1.0 / (1u64 << register_4087) as f32;

	let [register_4088, register_4089, register_4090, register_4091] = split_registers::<4>(words[1022]);
	raw_estimate += 1.0 / (1u64 << register_4088) as f32 + 1.0 / (1u64 << register_4089) as f32 + 1.0 / (1u64 << register_4090) as f32 + 1.0 / (1u64 << register_4091) as f32;

	let [register_4092, register_4093, register_4094, register_4095] = split_registers::<4>(words[1023]);
	raw_estimate += 1.0 / (1u64 << register_4092) as f32 + 1.0 / (1u64 << register_4093) as f32 + 1.0 / (1u64 << register_4094) as f32 + 1.0 / (1u64 << register_4095) as f32;


    raw_estimate
}
