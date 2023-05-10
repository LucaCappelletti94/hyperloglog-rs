
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_512_registers_and_5_bits(words: &[u32; 86]) -> f32 {
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

	let [register_126, register_127, register_128, register_129, register_130, register_131] = split_registers::<6>(words[21]);
	raw_estimate += 1.0 / (1u64 << register_126) as f32 + 1.0 / (1u64 << register_127) as f32 + 1.0 / (1u64 << register_128) as f32 + 1.0 / (1u64 << register_129) as f32 + 1.0 / (1u64 << register_130) as f32 + 1.0 / (1u64 << register_131) as f32;

	let [register_132, register_133, register_134, register_135, register_136, register_137] = split_registers::<6>(words[22]);
	raw_estimate += 1.0 / (1u64 << register_132) as f32 + 1.0 / (1u64 << register_133) as f32 + 1.0 / (1u64 << register_134) as f32 + 1.0 / (1u64 << register_135) as f32 + 1.0 / (1u64 << register_136) as f32 + 1.0 / (1u64 << register_137) as f32;

	let [register_138, register_139, register_140, register_141, register_142, register_143] = split_registers::<6>(words[23]);
	raw_estimate += 1.0 / (1u64 << register_138) as f32 + 1.0 / (1u64 << register_139) as f32 + 1.0 / (1u64 << register_140) as f32 + 1.0 / (1u64 << register_141) as f32 + 1.0 / (1u64 << register_142) as f32 + 1.0 / (1u64 << register_143) as f32;

	let [register_144, register_145, register_146, register_147, register_148, register_149] = split_registers::<6>(words[24]);
	raw_estimate += 1.0 / (1u64 << register_144) as f32 + 1.0 / (1u64 << register_145) as f32 + 1.0 / (1u64 << register_146) as f32 + 1.0 / (1u64 << register_147) as f32 + 1.0 / (1u64 << register_148) as f32 + 1.0 / (1u64 << register_149) as f32;

	let [register_150, register_151, register_152, register_153, register_154, register_155] = split_registers::<6>(words[25]);
	raw_estimate += 1.0 / (1u64 << register_150) as f32 + 1.0 / (1u64 << register_151) as f32 + 1.0 / (1u64 << register_152) as f32 + 1.0 / (1u64 << register_153) as f32 + 1.0 / (1u64 << register_154) as f32 + 1.0 / (1u64 << register_155) as f32;

	let [register_156, register_157, register_158, register_159, register_160, register_161] = split_registers::<6>(words[26]);
	raw_estimate += 1.0 / (1u64 << register_156) as f32 + 1.0 / (1u64 << register_157) as f32 + 1.0 / (1u64 << register_158) as f32 + 1.0 / (1u64 << register_159) as f32 + 1.0 / (1u64 << register_160) as f32 + 1.0 / (1u64 << register_161) as f32;

	let [register_162, register_163, register_164, register_165, register_166, register_167] = split_registers::<6>(words[27]);
	raw_estimate += 1.0 / (1u64 << register_162) as f32 + 1.0 / (1u64 << register_163) as f32 + 1.0 / (1u64 << register_164) as f32 + 1.0 / (1u64 << register_165) as f32 + 1.0 / (1u64 << register_166) as f32 + 1.0 / (1u64 << register_167) as f32;

	let [register_168, register_169, register_170, register_171, register_172, register_173] = split_registers::<6>(words[28]);
	raw_estimate += 1.0 / (1u64 << register_168) as f32 + 1.0 / (1u64 << register_169) as f32 + 1.0 / (1u64 << register_170) as f32 + 1.0 / (1u64 << register_171) as f32 + 1.0 / (1u64 << register_172) as f32 + 1.0 / (1u64 << register_173) as f32;

	let [register_174, register_175, register_176, register_177, register_178, register_179] = split_registers::<6>(words[29]);
	raw_estimate += 1.0 / (1u64 << register_174) as f32 + 1.0 / (1u64 << register_175) as f32 + 1.0 / (1u64 << register_176) as f32 + 1.0 / (1u64 << register_177) as f32 + 1.0 / (1u64 << register_178) as f32 + 1.0 / (1u64 << register_179) as f32;

	let [register_180, register_181, register_182, register_183, register_184, register_185] = split_registers::<6>(words[30]);
	raw_estimate += 1.0 / (1u64 << register_180) as f32 + 1.0 / (1u64 << register_181) as f32 + 1.0 / (1u64 << register_182) as f32 + 1.0 / (1u64 << register_183) as f32 + 1.0 / (1u64 << register_184) as f32 + 1.0 / (1u64 << register_185) as f32;

	let [register_186, register_187, register_188, register_189, register_190, register_191] = split_registers::<6>(words[31]);
	raw_estimate += 1.0 / (1u64 << register_186) as f32 + 1.0 / (1u64 << register_187) as f32 + 1.0 / (1u64 << register_188) as f32 + 1.0 / (1u64 << register_189) as f32 + 1.0 / (1u64 << register_190) as f32 + 1.0 / (1u64 << register_191) as f32;

	let [register_192, register_193, register_194, register_195, register_196, register_197] = split_registers::<6>(words[32]);
	raw_estimate += 1.0 / (1u64 << register_192) as f32 + 1.0 / (1u64 << register_193) as f32 + 1.0 / (1u64 << register_194) as f32 + 1.0 / (1u64 << register_195) as f32 + 1.0 / (1u64 << register_196) as f32 + 1.0 / (1u64 << register_197) as f32;

	let [register_198, register_199, register_200, register_201, register_202, register_203] = split_registers::<6>(words[33]);
	raw_estimate += 1.0 / (1u64 << register_198) as f32 + 1.0 / (1u64 << register_199) as f32 + 1.0 / (1u64 << register_200) as f32 + 1.0 / (1u64 << register_201) as f32 + 1.0 / (1u64 << register_202) as f32 + 1.0 / (1u64 << register_203) as f32;

	let [register_204, register_205, register_206, register_207, register_208, register_209] = split_registers::<6>(words[34]);
	raw_estimate += 1.0 / (1u64 << register_204) as f32 + 1.0 / (1u64 << register_205) as f32 + 1.0 / (1u64 << register_206) as f32 + 1.0 / (1u64 << register_207) as f32 + 1.0 / (1u64 << register_208) as f32 + 1.0 / (1u64 << register_209) as f32;

	let [register_210, register_211, register_212, register_213, register_214, register_215] = split_registers::<6>(words[35]);
	raw_estimate += 1.0 / (1u64 << register_210) as f32 + 1.0 / (1u64 << register_211) as f32 + 1.0 / (1u64 << register_212) as f32 + 1.0 / (1u64 << register_213) as f32 + 1.0 / (1u64 << register_214) as f32 + 1.0 / (1u64 << register_215) as f32;

	let [register_216, register_217, register_218, register_219, register_220, register_221] = split_registers::<6>(words[36]);
	raw_estimate += 1.0 / (1u64 << register_216) as f32 + 1.0 / (1u64 << register_217) as f32 + 1.0 / (1u64 << register_218) as f32 + 1.0 / (1u64 << register_219) as f32 + 1.0 / (1u64 << register_220) as f32 + 1.0 / (1u64 << register_221) as f32;

	let [register_222, register_223, register_224, register_225, register_226, register_227] = split_registers::<6>(words[37]);
	raw_estimate += 1.0 / (1u64 << register_222) as f32 + 1.0 / (1u64 << register_223) as f32 + 1.0 / (1u64 << register_224) as f32 + 1.0 / (1u64 << register_225) as f32 + 1.0 / (1u64 << register_226) as f32 + 1.0 / (1u64 << register_227) as f32;

	let [register_228, register_229, register_230, register_231, register_232, register_233] = split_registers::<6>(words[38]);
	raw_estimate += 1.0 / (1u64 << register_228) as f32 + 1.0 / (1u64 << register_229) as f32 + 1.0 / (1u64 << register_230) as f32 + 1.0 / (1u64 << register_231) as f32 + 1.0 / (1u64 << register_232) as f32 + 1.0 / (1u64 << register_233) as f32;

	let [register_234, register_235, register_236, register_237, register_238, register_239] = split_registers::<6>(words[39]);
	raw_estimate += 1.0 / (1u64 << register_234) as f32 + 1.0 / (1u64 << register_235) as f32 + 1.0 / (1u64 << register_236) as f32 + 1.0 / (1u64 << register_237) as f32 + 1.0 / (1u64 << register_238) as f32 + 1.0 / (1u64 << register_239) as f32;

	let [register_240, register_241, register_242, register_243, register_244, register_245] = split_registers::<6>(words[40]);
	raw_estimate += 1.0 / (1u64 << register_240) as f32 + 1.0 / (1u64 << register_241) as f32 + 1.0 / (1u64 << register_242) as f32 + 1.0 / (1u64 << register_243) as f32 + 1.0 / (1u64 << register_244) as f32 + 1.0 / (1u64 << register_245) as f32;

	let [register_246, register_247, register_248, register_249, register_250, register_251] = split_registers::<6>(words[41]);
	raw_estimate += 1.0 / (1u64 << register_246) as f32 + 1.0 / (1u64 << register_247) as f32 + 1.0 / (1u64 << register_248) as f32 + 1.0 / (1u64 << register_249) as f32 + 1.0 / (1u64 << register_250) as f32 + 1.0 / (1u64 << register_251) as f32;

	let [register_252, register_253, register_254, register_255, register_256, register_257] = split_registers::<6>(words[42]);
	raw_estimate += 1.0 / (1u64 << register_252) as f32 + 1.0 / (1u64 << register_253) as f32 + 1.0 / (1u64 << register_254) as f32 + 1.0 / (1u64 << register_255) as f32 + 1.0 / (1u64 << register_256) as f32 + 1.0 / (1u64 << register_257) as f32;

	let [register_258, register_259, register_260, register_261, register_262, register_263] = split_registers::<6>(words[43]);
	raw_estimate += 1.0 / (1u64 << register_258) as f32 + 1.0 / (1u64 << register_259) as f32 + 1.0 / (1u64 << register_260) as f32 + 1.0 / (1u64 << register_261) as f32 + 1.0 / (1u64 << register_262) as f32 + 1.0 / (1u64 << register_263) as f32;

	let [register_264, register_265, register_266, register_267, register_268, register_269] = split_registers::<6>(words[44]);
	raw_estimate += 1.0 / (1u64 << register_264) as f32 + 1.0 / (1u64 << register_265) as f32 + 1.0 / (1u64 << register_266) as f32 + 1.0 / (1u64 << register_267) as f32 + 1.0 / (1u64 << register_268) as f32 + 1.0 / (1u64 << register_269) as f32;

	let [register_270, register_271, register_272, register_273, register_274, register_275] = split_registers::<6>(words[45]);
	raw_estimate += 1.0 / (1u64 << register_270) as f32 + 1.0 / (1u64 << register_271) as f32 + 1.0 / (1u64 << register_272) as f32 + 1.0 / (1u64 << register_273) as f32 + 1.0 / (1u64 << register_274) as f32 + 1.0 / (1u64 << register_275) as f32;

	let [register_276, register_277, register_278, register_279, register_280, register_281] = split_registers::<6>(words[46]);
	raw_estimate += 1.0 / (1u64 << register_276) as f32 + 1.0 / (1u64 << register_277) as f32 + 1.0 / (1u64 << register_278) as f32 + 1.0 / (1u64 << register_279) as f32 + 1.0 / (1u64 << register_280) as f32 + 1.0 / (1u64 << register_281) as f32;

	let [register_282, register_283, register_284, register_285, register_286, register_287] = split_registers::<6>(words[47]);
	raw_estimate += 1.0 / (1u64 << register_282) as f32 + 1.0 / (1u64 << register_283) as f32 + 1.0 / (1u64 << register_284) as f32 + 1.0 / (1u64 << register_285) as f32 + 1.0 / (1u64 << register_286) as f32 + 1.0 / (1u64 << register_287) as f32;

	let [register_288, register_289, register_290, register_291, register_292, register_293] = split_registers::<6>(words[48]);
	raw_estimate += 1.0 / (1u64 << register_288) as f32 + 1.0 / (1u64 << register_289) as f32 + 1.0 / (1u64 << register_290) as f32 + 1.0 / (1u64 << register_291) as f32 + 1.0 / (1u64 << register_292) as f32 + 1.0 / (1u64 << register_293) as f32;

	let [register_294, register_295, register_296, register_297, register_298, register_299] = split_registers::<6>(words[49]);
	raw_estimate += 1.0 / (1u64 << register_294) as f32 + 1.0 / (1u64 << register_295) as f32 + 1.0 / (1u64 << register_296) as f32 + 1.0 / (1u64 << register_297) as f32 + 1.0 / (1u64 << register_298) as f32 + 1.0 / (1u64 << register_299) as f32;

	let [register_300, register_301, register_302, register_303, register_304, register_305] = split_registers::<6>(words[50]);
	raw_estimate += 1.0 / (1u64 << register_300) as f32 + 1.0 / (1u64 << register_301) as f32 + 1.0 / (1u64 << register_302) as f32 + 1.0 / (1u64 << register_303) as f32 + 1.0 / (1u64 << register_304) as f32 + 1.0 / (1u64 << register_305) as f32;

	let [register_306, register_307, register_308, register_309, register_310, register_311] = split_registers::<6>(words[51]);
	raw_estimate += 1.0 / (1u64 << register_306) as f32 + 1.0 / (1u64 << register_307) as f32 + 1.0 / (1u64 << register_308) as f32 + 1.0 / (1u64 << register_309) as f32 + 1.0 / (1u64 << register_310) as f32 + 1.0 / (1u64 << register_311) as f32;

	let [register_312, register_313, register_314, register_315, register_316, register_317] = split_registers::<6>(words[52]);
	raw_estimate += 1.0 / (1u64 << register_312) as f32 + 1.0 / (1u64 << register_313) as f32 + 1.0 / (1u64 << register_314) as f32 + 1.0 / (1u64 << register_315) as f32 + 1.0 / (1u64 << register_316) as f32 + 1.0 / (1u64 << register_317) as f32;

	let [register_318, register_319, register_320, register_321, register_322, register_323] = split_registers::<6>(words[53]);
	raw_estimate += 1.0 / (1u64 << register_318) as f32 + 1.0 / (1u64 << register_319) as f32 + 1.0 / (1u64 << register_320) as f32 + 1.0 / (1u64 << register_321) as f32 + 1.0 / (1u64 << register_322) as f32 + 1.0 / (1u64 << register_323) as f32;

	let [register_324, register_325, register_326, register_327, register_328, register_329] = split_registers::<6>(words[54]);
	raw_estimate += 1.0 / (1u64 << register_324) as f32 + 1.0 / (1u64 << register_325) as f32 + 1.0 / (1u64 << register_326) as f32 + 1.0 / (1u64 << register_327) as f32 + 1.0 / (1u64 << register_328) as f32 + 1.0 / (1u64 << register_329) as f32;

	let [register_330, register_331, register_332, register_333, register_334, register_335] = split_registers::<6>(words[55]);
	raw_estimate += 1.0 / (1u64 << register_330) as f32 + 1.0 / (1u64 << register_331) as f32 + 1.0 / (1u64 << register_332) as f32 + 1.0 / (1u64 << register_333) as f32 + 1.0 / (1u64 << register_334) as f32 + 1.0 / (1u64 << register_335) as f32;

	let [register_336, register_337, register_338, register_339, register_340, register_341] = split_registers::<6>(words[56]);
	raw_estimate += 1.0 / (1u64 << register_336) as f32 + 1.0 / (1u64 << register_337) as f32 + 1.0 / (1u64 << register_338) as f32 + 1.0 / (1u64 << register_339) as f32 + 1.0 / (1u64 << register_340) as f32 + 1.0 / (1u64 << register_341) as f32;

	let [register_342, register_343, register_344, register_345, register_346, register_347] = split_registers::<6>(words[57]);
	raw_estimate += 1.0 / (1u64 << register_342) as f32 + 1.0 / (1u64 << register_343) as f32 + 1.0 / (1u64 << register_344) as f32 + 1.0 / (1u64 << register_345) as f32 + 1.0 / (1u64 << register_346) as f32 + 1.0 / (1u64 << register_347) as f32;

	let [register_348, register_349, register_350, register_351, register_352, register_353] = split_registers::<6>(words[58]);
	raw_estimate += 1.0 / (1u64 << register_348) as f32 + 1.0 / (1u64 << register_349) as f32 + 1.0 / (1u64 << register_350) as f32 + 1.0 / (1u64 << register_351) as f32 + 1.0 / (1u64 << register_352) as f32 + 1.0 / (1u64 << register_353) as f32;

	let [register_354, register_355, register_356, register_357, register_358, register_359] = split_registers::<6>(words[59]);
	raw_estimate += 1.0 / (1u64 << register_354) as f32 + 1.0 / (1u64 << register_355) as f32 + 1.0 / (1u64 << register_356) as f32 + 1.0 / (1u64 << register_357) as f32 + 1.0 / (1u64 << register_358) as f32 + 1.0 / (1u64 << register_359) as f32;

	let [register_360, register_361, register_362, register_363, register_364, register_365] = split_registers::<6>(words[60]);
	raw_estimate += 1.0 / (1u64 << register_360) as f32 + 1.0 / (1u64 << register_361) as f32 + 1.0 / (1u64 << register_362) as f32 + 1.0 / (1u64 << register_363) as f32 + 1.0 / (1u64 << register_364) as f32 + 1.0 / (1u64 << register_365) as f32;

	let [register_366, register_367, register_368, register_369, register_370, register_371] = split_registers::<6>(words[61]);
	raw_estimate += 1.0 / (1u64 << register_366) as f32 + 1.0 / (1u64 << register_367) as f32 + 1.0 / (1u64 << register_368) as f32 + 1.0 / (1u64 << register_369) as f32 + 1.0 / (1u64 << register_370) as f32 + 1.0 / (1u64 << register_371) as f32;

	let [register_372, register_373, register_374, register_375, register_376, register_377] = split_registers::<6>(words[62]);
	raw_estimate += 1.0 / (1u64 << register_372) as f32 + 1.0 / (1u64 << register_373) as f32 + 1.0 / (1u64 << register_374) as f32 + 1.0 / (1u64 << register_375) as f32 + 1.0 / (1u64 << register_376) as f32 + 1.0 / (1u64 << register_377) as f32;

	let [register_378, register_379, register_380, register_381, register_382, register_383] = split_registers::<6>(words[63]);
	raw_estimate += 1.0 / (1u64 << register_378) as f32 + 1.0 / (1u64 << register_379) as f32 + 1.0 / (1u64 << register_380) as f32 + 1.0 / (1u64 << register_381) as f32 + 1.0 / (1u64 << register_382) as f32 + 1.0 / (1u64 << register_383) as f32;

	let [register_384, register_385, register_386, register_387, register_388, register_389] = split_registers::<6>(words[64]);
	raw_estimate += 1.0 / (1u64 << register_384) as f32 + 1.0 / (1u64 << register_385) as f32 + 1.0 / (1u64 << register_386) as f32 + 1.0 / (1u64 << register_387) as f32 + 1.0 / (1u64 << register_388) as f32 + 1.0 / (1u64 << register_389) as f32;

	let [register_390, register_391, register_392, register_393, register_394, register_395] = split_registers::<6>(words[65]);
	raw_estimate += 1.0 / (1u64 << register_390) as f32 + 1.0 / (1u64 << register_391) as f32 + 1.0 / (1u64 << register_392) as f32 + 1.0 / (1u64 << register_393) as f32 + 1.0 / (1u64 << register_394) as f32 + 1.0 / (1u64 << register_395) as f32;

	let [register_396, register_397, register_398, register_399, register_400, register_401] = split_registers::<6>(words[66]);
	raw_estimate += 1.0 / (1u64 << register_396) as f32 + 1.0 / (1u64 << register_397) as f32 + 1.0 / (1u64 << register_398) as f32 + 1.0 / (1u64 << register_399) as f32 + 1.0 / (1u64 << register_400) as f32 + 1.0 / (1u64 << register_401) as f32;

	let [register_402, register_403, register_404, register_405, register_406, register_407] = split_registers::<6>(words[67]);
	raw_estimate += 1.0 / (1u64 << register_402) as f32 + 1.0 / (1u64 << register_403) as f32 + 1.0 / (1u64 << register_404) as f32 + 1.0 / (1u64 << register_405) as f32 + 1.0 / (1u64 << register_406) as f32 + 1.0 / (1u64 << register_407) as f32;

	let [register_408, register_409, register_410, register_411, register_412, register_413] = split_registers::<6>(words[68]);
	raw_estimate += 1.0 / (1u64 << register_408) as f32 + 1.0 / (1u64 << register_409) as f32 + 1.0 / (1u64 << register_410) as f32 + 1.0 / (1u64 << register_411) as f32 + 1.0 / (1u64 << register_412) as f32 + 1.0 / (1u64 << register_413) as f32;

	let [register_414, register_415, register_416, register_417, register_418, register_419] = split_registers::<6>(words[69]);
	raw_estimate += 1.0 / (1u64 << register_414) as f32 + 1.0 / (1u64 << register_415) as f32 + 1.0 / (1u64 << register_416) as f32 + 1.0 / (1u64 << register_417) as f32 + 1.0 / (1u64 << register_418) as f32 + 1.0 / (1u64 << register_419) as f32;

	let [register_420, register_421, register_422, register_423, register_424, register_425] = split_registers::<6>(words[70]);
	raw_estimate += 1.0 / (1u64 << register_420) as f32 + 1.0 / (1u64 << register_421) as f32 + 1.0 / (1u64 << register_422) as f32 + 1.0 / (1u64 << register_423) as f32 + 1.0 / (1u64 << register_424) as f32 + 1.0 / (1u64 << register_425) as f32;

	let [register_426, register_427, register_428, register_429, register_430, register_431] = split_registers::<6>(words[71]);
	raw_estimate += 1.0 / (1u64 << register_426) as f32 + 1.0 / (1u64 << register_427) as f32 + 1.0 / (1u64 << register_428) as f32 + 1.0 / (1u64 << register_429) as f32 + 1.0 / (1u64 << register_430) as f32 + 1.0 / (1u64 << register_431) as f32;

	let [register_432, register_433, register_434, register_435, register_436, register_437] = split_registers::<6>(words[72]);
	raw_estimate += 1.0 / (1u64 << register_432) as f32 + 1.0 / (1u64 << register_433) as f32 + 1.0 / (1u64 << register_434) as f32 + 1.0 / (1u64 << register_435) as f32 + 1.0 / (1u64 << register_436) as f32 + 1.0 / (1u64 << register_437) as f32;

	let [register_438, register_439, register_440, register_441, register_442, register_443] = split_registers::<6>(words[73]);
	raw_estimate += 1.0 / (1u64 << register_438) as f32 + 1.0 / (1u64 << register_439) as f32 + 1.0 / (1u64 << register_440) as f32 + 1.0 / (1u64 << register_441) as f32 + 1.0 / (1u64 << register_442) as f32 + 1.0 / (1u64 << register_443) as f32;

	let [register_444, register_445, register_446, register_447, register_448, register_449] = split_registers::<6>(words[74]);
	raw_estimate += 1.0 / (1u64 << register_444) as f32 + 1.0 / (1u64 << register_445) as f32 + 1.0 / (1u64 << register_446) as f32 + 1.0 / (1u64 << register_447) as f32 + 1.0 / (1u64 << register_448) as f32 + 1.0 / (1u64 << register_449) as f32;

	let [register_450, register_451, register_452, register_453, register_454, register_455] = split_registers::<6>(words[75]);
	raw_estimate += 1.0 / (1u64 << register_450) as f32 + 1.0 / (1u64 << register_451) as f32 + 1.0 / (1u64 << register_452) as f32 + 1.0 / (1u64 << register_453) as f32 + 1.0 / (1u64 << register_454) as f32 + 1.0 / (1u64 << register_455) as f32;

	let [register_456, register_457, register_458, register_459, register_460, register_461] = split_registers::<6>(words[76]);
	raw_estimate += 1.0 / (1u64 << register_456) as f32 + 1.0 / (1u64 << register_457) as f32 + 1.0 / (1u64 << register_458) as f32 + 1.0 / (1u64 << register_459) as f32 + 1.0 / (1u64 << register_460) as f32 + 1.0 / (1u64 << register_461) as f32;

	let [register_462, register_463, register_464, register_465, register_466, register_467] = split_registers::<6>(words[77]);
	raw_estimate += 1.0 / (1u64 << register_462) as f32 + 1.0 / (1u64 << register_463) as f32 + 1.0 / (1u64 << register_464) as f32 + 1.0 / (1u64 << register_465) as f32 + 1.0 / (1u64 << register_466) as f32 + 1.0 / (1u64 << register_467) as f32;

	let [register_468, register_469, register_470, register_471, register_472, register_473] = split_registers::<6>(words[78]);
	raw_estimate += 1.0 / (1u64 << register_468) as f32 + 1.0 / (1u64 << register_469) as f32 + 1.0 / (1u64 << register_470) as f32 + 1.0 / (1u64 << register_471) as f32 + 1.0 / (1u64 << register_472) as f32 + 1.0 / (1u64 << register_473) as f32;

	let [register_474, register_475, register_476, register_477, register_478, register_479] = split_registers::<6>(words[79]);
	raw_estimate += 1.0 / (1u64 << register_474) as f32 + 1.0 / (1u64 << register_475) as f32 + 1.0 / (1u64 << register_476) as f32 + 1.0 / (1u64 << register_477) as f32 + 1.0 / (1u64 << register_478) as f32 + 1.0 / (1u64 << register_479) as f32;

	let [register_480, register_481, register_482, register_483, register_484, register_485] = split_registers::<6>(words[80]);
	raw_estimate += 1.0 / (1u64 << register_480) as f32 + 1.0 / (1u64 << register_481) as f32 + 1.0 / (1u64 << register_482) as f32 + 1.0 / (1u64 << register_483) as f32 + 1.0 / (1u64 << register_484) as f32 + 1.0 / (1u64 << register_485) as f32;

	let [register_486, register_487, register_488, register_489, register_490, register_491] = split_registers::<6>(words[81]);
	raw_estimate += 1.0 / (1u64 << register_486) as f32 + 1.0 / (1u64 << register_487) as f32 + 1.0 / (1u64 << register_488) as f32 + 1.0 / (1u64 << register_489) as f32 + 1.0 / (1u64 << register_490) as f32 + 1.0 / (1u64 << register_491) as f32;

	let [register_492, register_493, register_494, register_495, register_496, register_497] = split_registers::<6>(words[82]);
	raw_estimate += 1.0 / (1u64 << register_492) as f32 + 1.0 / (1u64 << register_493) as f32 + 1.0 / (1u64 << register_494) as f32 + 1.0 / (1u64 << register_495) as f32 + 1.0 / (1u64 << register_496) as f32 + 1.0 / (1u64 << register_497) as f32;

	let [register_498, register_499, register_500, register_501, register_502, register_503] = split_registers::<6>(words[83]);
	raw_estimate += 1.0 / (1u64 << register_498) as f32 + 1.0 / (1u64 << register_499) as f32 + 1.0 / (1u64 << register_500) as f32 + 1.0 / (1u64 << register_501) as f32 + 1.0 / (1u64 << register_502) as f32 + 1.0 / (1u64 << register_503) as f32;

	let [register_504, register_505, register_506, register_507, register_508, register_509] = split_registers::<6>(words[84]);
	raw_estimate += 1.0 / (1u64 << register_504) as f32 + 1.0 / (1u64 << register_505) as f32 + 1.0 / (1u64 << register_506) as f32 + 1.0 / (1u64 << register_507) as f32 + 1.0 / (1u64 << register_508) as f32 + 1.0 / (1u64 << register_509) as f32;

	let [register_510, register_511, _, _, _, _] = split_registers::<6>(words[85]);
	raw_estimate += 1.0 / (1u64 << register_510) as f32 + 1.0 / (1u64 << register_511) as f32;


    raw_estimate
}
