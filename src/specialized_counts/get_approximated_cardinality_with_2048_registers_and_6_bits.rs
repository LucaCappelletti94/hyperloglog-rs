
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_2048_registers_and_6_bits(words: &[u32; 410]) -> f32 {
    let mut raw_estimate = 0.0;

	let [register_0, register_1, register_2, register_3, register_4] = split_registers::<5>(words[0]);
	raw_estimate += 1.0 / (1u64 << register_0) as f32 + 1.0 / (1u64 << register_1) as f32 + 1.0 / (1u64 << register_2) as f32 + 1.0 / (1u64 << register_3) as f32 + 1.0 / (1u64 << register_4) as f32;

	let [register_5, register_6, register_7, register_8, register_9] = split_registers::<5>(words[1]);
	raw_estimate += 1.0 / (1u64 << register_5) as f32 + 1.0 / (1u64 << register_6) as f32 + 1.0 / (1u64 << register_7) as f32 + 1.0 / (1u64 << register_8) as f32 + 1.0 / (1u64 << register_9) as f32;

	let [register_10, register_11, register_12, register_13, register_14] = split_registers::<5>(words[2]);
	raw_estimate += 1.0 / (1u64 << register_10) as f32 + 1.0 / (1u64 << register_11) as f32 + 1.0 / (1u64 << register_12) as f32 + 1.0 / (1u64 << register_13) as f32 + 1.0 / (1u64 << register_14) as f32;

	let [register_15, register_16, register_17, register_18, register_19] = split_registers::<5>(words[3]);
	raw_estimate += 1.0 / (1u64 << register_15) as f32 + 1.0 / (1u64 << register_16) as f32 + 1.0 / (1u64 << register_17) as f32 + 1.0 / (1u64 << register_18) as f32 + 1.0 / (1u64 << register_19) as f32;

	let [register_20, register_21, register_22, register_23, register_24] = split_registers::<5>(words[4]);
	raw_estimate += 1.0 / (1u64 << register_20) as f32 + 1.0 / (1u64 << register_21) as f32 + 1.0 / (1u64 << register_22) as f32 + 1.0 / (1u64 << register_23) as f32 + 1.0 / (1u64 << register_24) as f32;

	let [register_25, register_26, register_27, register_28, register_29] = split_registers::<5>(words[5]);
	raw_estimate += 1.0 / (1u64 << register_25) as f32 + 1.0 / (1u64 << register_26) as f32 + 1.0 / (1u64 << register_27) as f32 + 1.0 / (1u64 << register_28) as f32 + 1.0 / (1u64 << register_29) as f32;

	let [register_30, register_31, register_32, register_33, register_34] = split_registers::<5>(words[6]);
	raw_estimate += 1.0 / (1u64 << register_30) as f32 + 1.0 / (1u64 << register_31) as f32 + 1.0 / (1u64 << register_32) as f32 + 1.0 / (1u64 << register_33) as f32 + 1.0 / (1u64 << register_34) as f32;

	let [register_35, register_36, register_37, register_38, register_39] = split_registers::<5>(words[7]);
	raw_estimate += 1.0 / (1u64 << register_35) as f32 + 1.0 / (1u64 << register_36) as f32 + 1.0 / (1u64 << register_37) as f32 + 1.0 / (1u64 << register_38) as f32 + 1.0 / (1u64 << register_39) as f32;

	let [register_40, register_41, register_42, register_43, register_44] = split_registers::<5>(words[8]);
	raw_estimate += 1.0 / (1u64 << register_40) as f32 + 1.0 / (1u64 << register_41) as f32 + 1.0 / (1u64 << register_42) as f32 + 1.0 / (1u64 << register_43) as f32 + 1.0 / (1u64 << register_44) as f32;

	let [register_45, register_46, register_47, register_48, register_49] = split_registers::<5>(words[9]);
	raw_estimate += 1.0 / (1u64 << register_45) as f32 + 1.0 / (1u64 << register_46) as f32 + 1.0 / (1u64 << register_47) as f32 + 1.0 / (1u64 << register_48) as f32 + 1.0 / (1u64 << register_49) as f32;

	let [register_50, register_51, register_52, register_53, register_54] = split_registers::<5>(words[10]);
	raw_estimate += 1.0 / (1u64 << register_50) as f32 + 1.0 / (1u64 << register_51) as f32 + 1.0 / (1u64 << register_52) as f32 + 1.0 / (1u64 << register_53) as f32 + 1.0 / (1u64 << register_54) as f32;

	let [register_55, register_56, register_57, register_58, register_59] = split_registers::<5>(words[11]);
	raw_estimate += 1.0 / (1u64 << register_55) as f32 + 1.0 / (1u64 << register_56) as f32 + 1.0 / (1u64 << register_57) as f32 + 1.0 / (1u64 << register_58) as f32 + 1.0 / (1u64 << register_59) as f32;

	let [register_60, register_61, register_62, register_63, register_64] = split_registers::<5>(words[12]);
	raw_estimate += 1.0 / (1u64 << register_60) as f32 + 1.0 / (1u64 << register_61) as f32 + 1.0 / (1u64 << register_62) as f32 + 1.0 / (1u64 << register_63) as f32 + 1.0 / (1u64 << register_64) as f32;

	let [register_65, register_66, register_67, register_68, register_69] = split_registers::<5>(words[13]);
	raw_estimate += 1.0 / (1u64 << register_65) as f32 + 1.0 / (1u64 << register_66) as f32 + 1.0 / (1u64 << register_67) as f32 + 1.0 / (1u64 << register_68) as f32 + 1.0 / (1u64 << register_69) as f32;

	let [register_70, register_71, register_72, register_73, register_74] = split_registers::<5>(words[14]);
	raw_estimate += 1.0 / (1u64 << register_70) as f32 + 1.0 / (1u64 << register_71) as f32 + 1.0 / (1u64 << register_72) as f32 + 1.0 / (1u64 << register_73) as f32 + 1.0 / (1u64 << register_74) as f32;

	let [register_75, register_76, register_77, register_78, register_79] = split_registers::<5>(words[15]);
	raw_estimate += 1.0 / (1u64 << register_75) as f32 + 1.0 / (1u64 << register_76) as f32 + 1.0 / (1u64 << register_77) as f32 + 1.0 / (1u64 << register_78) as f32 + 1.0 / (1u64 << register_79) as f32;

	let [register_80, register_81, register_82, register_83, register_84] = split_registers::<5>(words[16]);
	raw_estimate += 1.0 / (1u64 << register_80) as f32 + 1.0 / (1u64 << register_81) as f32 + 1.0 / (1u64 << register_82) as f32 + 1.0 / (1u64 << register_83) as f32 + 1.0 / (1u64 << register_84) as f32;

	let [register_85, register_86, register_87, register_88, register_89] = split_registers::<5>(words[17]);
	raw_estimate += 1.0 / (1u64 << register_85) as f32 + 1.0 / (1u64 << register_86) as f32 + 1.0 / (1u64 << register_87) as f32 + 1.0 / (1u64 << register_88) as f32 + 1.0 / (1u64 << register_89) as f32;

	let [register_90, register_91, register_92, register_93, register_94] = split_registers::<5>(words[18]);
	raw_estimate += 1.0 / (1u64 << register_90) as f32 + 1.0 / (1u64 << register_91) as f32 + 1.0 / (1u64 << register_92) as f32 + 1.0 / (1u64 << register_93) as f32 + 1.0 / (1u64 << register_94) as f32;

	let [register_95, register_96, register_97, register_98, register_99] = split_registers::<5>(words[19]);
	raw_estimate += 1.0 / (1u64 << register_95) as f32 + 1.0 / (1u64 << register_96) as f32 + 1.0 / (1u64 << register_97) as f32 + 1.0 / (1u64 << register_98) as f32 + 1.0 / (1u64 << register_99) as f32;

	let [register_100, register_101, register_102, register_103, register_104] = split_registers::<5>(words[20]);
	raw_estimate += 1.0 / (1u64 << register_100) as f32 + 1.0 / (1u64 << register_101) as f32 + 1.0 / (1u64 << register_102) as f32 + 1.0 / (1u64 << register_103) as f32 + 1.0 / (1u64 << register_104) as f32;

	let [register_105, register_106, register_107, register_108, register_109] = split_registers::<5>(words[21]);
	raw_estimate += 1.0 / (1u64 << register_105) as f32 + 1.0 / (1u64 << register_106) as f32 + 1.0 / (1u64 << register_107) as f32 + 1.0 / (1u64 << register_108) as f32 + 1.0 / (1u64 << register_109) as f32;

	let [register_110, register_111, register_112, register_113, register_114] = split_registers::<5>(words[22]);
	raw_estimate += 1.0 / (1u64 << register_110) as f32 + 1.0 / (1u64 << register_111) as f32 + 1.0 / (1u64 << register_112) as f32 + 1.0 / (1u64 << register_113) as f32 + 1.0 / (1u64 << register_114) as f32;

	let [register_115, register_116, register_117, register_118, register_119] = split_registers::<5>(words[23]);
	raw_estimate += 1.0 / (1u64 << register_115) as f32 + 1.0 / (1u64 << register_116) as f32 + 1.0 / (1u64 << register_117) as f32 + 1.0 / (1u64 << register_118) as f32 + 1.0 / (1u64 << register_119) as f32;

	let [register_120, register_121, register_122, register_123, register_124] = split_registers::<5>(words[24]);
	raw_estimate += 1.0 / (1u64 << register_120) as f32 + 1.0 / (1u64 << register_121) as f32 + 1.0 / (1u64 << register_122) as f32 + 1.0 / (1u64 << register_123) as f32 + 1.0 / (1u64 << register_124) as f32;

	let [register_125, register_126, register_127, register_128, register_129] = split_registers::<5>(words[25]);
	raw_estimate += 1.0 / (1u64 << register_125) as f32 + 1.0 / (1u64 << register_126) as f32 + 1.0 / (1u64 << register_127) as f32 + 1.0 / (1u64 << register_128) as f32 + 1.0 / (1u64 << register_129) as f32;

	let [register_130, register_131, register_132, register_133, register_134] = split_registers::<5>(words[26]);
	raw_estimate += 1.0 / (1u64 << register_130) as f32 + 1.0 / (1u64 << register_131) as f32 + 1.0 / (1u64 << register_132) as f32 + 1.0 / (1u64 << register_133) as f32 + 1.0 / (1u64 << register_134) as f32;

	let [register_135, register_136, register_137, register_138, register_139] = split_registers::<5>(words[27]);
	raw_estimate += 1.0 / (1u64 << register_135) as f32 + 1.0 / (1u64 << register_136) as f32 + 1.0 / (1u64 << register_137) as f32 + 1.0 / (1u64 << register_138) as f32 + 1.0 / (1u64 << register_139) as f32;

	let [register_140, register_141, register_142, register_143, register_144] = split_registers::<5>(words[28]);
	raw_estimate += 1.0 / (1u64 << register_140) as f32 + 1.0 / (1u64 << register_141) as f32 + 1.0 / (1u64 << register_142) as f32 + 1.0 / (1u64 << register_143) as f32 + 1.0 / (1u64 << register_144) as f32;

	let [register_145, register_146, register_147, register_148, register_149] = split_registers::<5>(words[29]);
	raw_estimate += 1.0 / (1u64 << register_145) as f32 + 1.0 / (1u64 << register_146) as f32 + 1.0 / (1u64 << register_147) as f32 + 1.0 / (1u64 << register_148) as f32 + 1.0 / (1u64 << register_149) as f32;

	let [register_150, register_151, register_152, register_153, register_154] = split_registers::<5>(words[30]);
	raw_estimate += 1.0 / (1u64 << register_150) as f32 + 1.0 / (1u64 << register_151) as f32 + 1.0 / (1u64 << register_152) as f32 + 1.0 / (1u64 << register_153) as f32 + 1.0 / (1u64 << register_154) as f32;

	let [register_155, register_156, register_157, register_158, register_159] = split_registers::<5>(words[31]);
	raw_estimate += 1.0 / (1u64 << register_155) as f32 + 1.0 / (1u64 << register_156) as f32 + 1.0 / (1u64 << register_157) as f32 + 1.0 / (1u64 << register_158) as f32 + 1.0 / (1u64 << register_159) as f32;

	let [register_160, register_161, register_162, register_163, register_164] = split_registers::<5>(words[32]);
	raw_estimate += 1.0 / (1u64 << register_160) as f32 + 1.0 / (1u64 << register_161) as f32 + 1.0 / (1u64 << register_162) as f32 + 1.0 / (1u64 << register_163) as f32 + 1.0 / (1u64 << register_164) as f32;

	let [register_165, register_166, register_167, register_168, register_169] = split_registers::<5>(words[33]);
	raw_estimate += 1.0 / (1u64 << register_165) as f32 + 1.0 / (1u64 << register_166) as f32 + 1.0 / (1u64 << register_167) as f32 + 1.0 / (1u64 << register_168) as f32 + 1.0 / (1u64 << register_169) as f32;

	let [register_170, register_171, register_172, register_173, register_174] = split_registers::<5>(words[34]);
	raw_estimate += 1.0 / (1u64 << register_170) as f32 + 1.0 / (1u64 << register_171) as f32 + 1.0 / (1u64 << register_172) as f32 + 1.0 / (1u64 << register_173) as f32 + 1.0 / (1u64 << register_174) as f32;

	let [register_175, register_176, register_177, register_178, register_179] = split_registers::<5>(words[35]);
	raw_estimate += 1.0 / (1u64 << register_175) as f32 + 1.0 / (1u64 << register_176) as f32 + 1.0 / (1u64 << register_177) as f32 + 1.0 / (1u64 << register_178) as f32 + 1.0 / (1u64 << register_179) as f32;

	let [register_180, register_181, register_182, register_183, register_184] = split_registers::<5>(words[36]);
	raw_estimate += 1.0 / (1u64 << register_180) as f32 + 1.0 / (1u64 << register_181) as f32 + 1.0 / (1u64 << register_182) as f32 + 1.0 / (1u64 << register_183) as f32 + 1.0 / (1u64 << register_184) as f32;

	let [register_185, register_186, register_187, register_188, register_189] = split_registers::<5>(words[37]);
	raw_estimate += 1.0 / (1u64 << register_185) as f32 + 1.0 / (1u64 << register_186) as f32 + 1.0 / (1u64 << register_187) as f32 + 1.0 / (1u64 << register_188) as f32 + 1.0 / (1u64 << register_189) as f32;

	let [register_190, register_191, register_192, register_193, register_194] = split_registers::<5>(words[38]);
	raw_estimate += 1.0 / (1u64 << register_190) as f32 + 1.0 / (1u64 << register_191) as f32 + 1.0 / (1u64 << register_192) as f32 + 1.0 / (1u64 << register_193) as f32 + 1.0 / (1u64 << register_194) as f32;

	let [register_195, register_196, register_197, register_198, register_199] = split_registers::<5>(words[39]);
	raw_estimate += 1.0 / (1u64 << register_195) as f32 + 1.0 / (1u64 << register_196) as f32 + 1.0 / (1u64 << register_197) as f32 + 1.0 / (1u64 << register_198) as f32 + 1.0 / (1u64 << register_199) as f32;

	let [register_200, register_201, register_202, register_203, register_204] = split_registers::<5>(words[40]);
	raw_estimate += 1.0 / (1u64 << register_200) as f32 + 1.0 / (1u64 << register_201) as f32 + 1.0 / (1u64 << register_202) as f32 + 1.0 / (1u64 << register_203) as f32 + 1.0 / (1u64 << register_204) as f32;

	let [register_205, register_206, register_207, register_208, register_209] = split_registers::<5>(words[41]);
	raw_estimate += 1.0 / (1u64 << register_205) as f32 + 1.0 / (1u64 << register_206) as f32 + 1.0 / (1u64 << register_207) as f32 + 1.0 / (1u64 << register_208) as f32 + 1.0 / (1u64 << register_209) as f32;

	let [register_210, register_211, register_212, register_213, register_214] = split_registers::<5>(words[42]);
	raw_estimate += 1.0 / (1u64 << register_210) as f32 + 1.0 / (1u64 << register_211) as f32 + 1.0 / (1u64 << register_212) as f32 + 1.0 / (1u64 << register_213) as f32 + 1.0 / (1u64 << register_214) as f32;

	let [register_215, register_216, register_217, register_218, register_219] = split_registers::<5>(words[43]);
	raw_estimate += 1.0 / (1u64 << register_215) as f32 + 1.0 / (1u64 << register_216) as f32 + 1.0 / (1u64 << register_217) as f32 + 1.0 / (1u64 << register_218) as f32 + 1.0 / (1u64 << register_219) as f32;

	let [register_220, register_221, register_222, register_223, register_224] = split_registers::<5>(words[44]);
	raw_estimate += 1.0 / (1u64 << register_220) as f32 + 1.0 / (1u64 << register_221) as f32 + 1.0 / (1u64 << register_222) as f32 + 1.0 / (1u64 << register_223) as f32 + 1.0 / (1u64 << register_224) as f32;

	let [register_225, register_226, register_227, register_228, register_229] = split_registers::<5>(words[45]);
	raw_estimate += 1.0 / (1u64 << register_225) as f32 + 1.0 / (1u64 << register_226) as f32 + 1.0 / (1u64 << register_227) as f32 + 1.0 / (1u64 << register_228) as f32 + 1.0 / (1u64 << register_229) as f32;

	let [register_230, register_231, register_232, register_233, register_234] = split_registers::<5>(words[46]);
	raw_estimate += 1.0 / (1u64 << register_230) as f32 + 1.0 / (1u64 << register_231) as f32 + 1.0 / (1u64 << register_232) as f32 + 1.0 / (1u64 << register_233) as f32 + 1.0 / (1u64 << register_234) as f32;

	let [register_235, register_236, register_237, register_238, register_239] = split_registers::<5>(words[47]);
	raw_estimate += 1.0 / (1u64 << register_235) as f32 + 1.0 / (1u64 << register_236) as f32 + 1.0 / (1u64 << register_237) as f32 + 1.0 / (1u64 << register_238) as f32 + 1.0 / (1u64 << register_239) as f32;

	let [register_240, register_241, register_242, register_243, register_244] = split_registers::<5>(words[48]);
	raw_estimate += 1.0 / (1u64 << register_240) as f32 + 1.0 / (1u64 << register_241) as f32 + 1.0 / (1u64 << register_242) as f32 + 1.0 / (1u64 << register_243) as f32 + 1.0 / (1u64 << register_244) as f32;

	let [register_245, register_246, register_247, register_248, register_249] = split_registers::<5>(words[49]);
	raw_estimate += 1.0 / (1u64 << register_245) as f32 + 1.0 / (1u64 << register_246) as f32 + 1.0 / (1u64 << register_247) as f32 + 1.0 / (1u64 << register_248) as f32 + 1.0 / (1u64 << register_249) as f32;

	let [register_250, register_251, register_252, register_253, register_254] = split_registers::<5>(words[50]);
	raw_estimate += 1.0 / (1u64 << register_250) as f32 + 1.0 / (1u64 << register_251) as f32 + 1.0 / (1u64 << register_252) as f32 + 1.0 / (1u64 << register_253) as f32 + 1.0 / (1u64 << register_254) as f32;

	let [register_255, register_256, register_257, register_258, register_259] = split_registers::<5>(words[51]);
	raw_estimate += 1.0 / (1u64 << register_255) as f32 + 1.0 / (1u64 << register_256) as f32 + 1.0 / (1u64 << register_257) as f32 + 1.0 / (1u64 << register_258) as f32 + 1.0 / (1u64 << register_259) as f32;

	let [register_260, register_261, register_262, register_263, register_264] = split_registers::<5>(words[52]);
	raw_estimate += 1.0 / (1u64 << register_260) as f32 + 1.0 / (1u64 << register_261) as f32 + 1.0 / (1u64 << register_262) as f32 + 1.0 / (1u64 << register_263) as f32 + 1.0 / (1u64 << register_264) as f32;

	let [register_265, register_266, register_267, register_268, register_269] = split_registers::<5>(words[53]);
	raw_estimate += 1.0 / (1u64 << register_265) as f32 + 1.0 / (1u64 << register_266) as f32 + 1.0 / (1u64 << register_267) as f32 + 1.0 / (1u64 << register_268) as f32 + 1.0 / (1u64 << register_269) as f32;

	let [register_270, register_271, register_272, register_273, register_274] = split_registers::<5>(words[54]);
	raw_estimate += 1.0 / (1u64 << register_270) as f32 + 1.0 / (1u64 << register_271) as f32 + 1.0 / (1u64 << register_272) as f32 + 1.0 / (1u64 << register_273) as f32 + 1.0 / (1u64 << register_274) as f32;

	let [register_275, register_276, register_277, register_278, register_279] = split_registers::<5>(words[55]);
	raw_estimate += 1.0 / (1u64 << register_275) as f32 + 1.0 / (1u64 << register_276) as f32 + 1.0 / (1u64 << register_277) as f32 + 1.0 / (1u64 << register_278) as f32 + 1.0 / (1u64 << register_279) as f32;

	let [register_280, register_281, register_282, register_283, register_284] = split_registers::<5>(words[56]);
	raw_estimate += 1.0 / (1u64 << register_280) as f32 + 1.0 / (1u64 << register_281) as f32 + 1.0 / (1u64 << register_282) as f32 + 1.0 / (1u64 << register_283) as f32 + 1.0 / (1u64 << register_284) as f32;

	let [register_285, register_286, register_287, register_288, register_289] = split_registers::<5>(words[57]);
	raw_estimate += 1.0 / (1u64 << register_285) as f32 + 1.0 / (1u64 << register_286) as f32 + 1.0 / (1u64 << register_287) as f32 + 1.0 / (1u64 << register_288) as f32 + 1.0 / (1u64 << register_289) as f32;

	let [register_290, register_291, register_292, register_293, register_294] = split_registers::<5>(words[58]);
	raw_estimate += 1.0 / (1u64 << register_290) as f32 + 1.0 / (1u64 << register_291) as f32 + 1.0 / (1u64 << register_292) as f32 + 1.0 / (1u64 << register_293) as f32 + 1.0 / (1u64 << register_294) as f32;

	let [register_295, register_296, register_297, register_298, register_299] = split_registers::<5>(words[59]);
	raw_estimate += 1.0 / (1u64 << register_295) as f32 + 1.0 / (1u64 << register_296) as f32 + 1.0 / (1u64 << register_297) as f32 + 1.0 / (1u64 << register_298) as f32 + 1.0 / (1u64 << register_299) as f32;

	let [register_300, register_301, register_302, register_303, register_304] = split_registers::<5>(words[60]);
	raw_estimate += 1.0 / (1u64 << register_300) as f32 + 1.0 / (1u64 << register_301) as f32 + 1.0 / (1u64 << register_302) as f32 + 1.0 / (1u64 << register_303) as f32 + 1.0 / (1u64 << register_304) as f32;

	let [register_305, register_306, register_307, register_308, register_309] = split_registers::<5>(words[61]);
	raw_estimate += 1.0 / (1u64 << register_305) as f32 + 1.0 / (1u64 << register_306) as f32 + 1.0 / (1u64 << register_307) as f32 + 1.0 / (1u64 << register_308) as f32 + 1.0 / (1u64 << register_309) as f32;

	let [register_310, register_311, register_312, register_313, register_314] = split_registers::<5>(words[62]);
	raw_estimate += 1.0 / (1u64 << register_310) as f32 + 1.0 / (1u64 << register_311) as f32 + 1.0 / (1u64 << register_312) as f32 + 1.0 / (1u64 << register_313) as f32 + 1.0 / (1u64 << register_314) as f32;

	let [register_315, register_316, register_317, register_318, register_319] = split_registers::<5>(words[63]);
	raw_estimate += 1.0 / (1u64 << register_315) as f32 + 1.0 / (1u64 << register_316) as f32 + 1.0 / (1u64 << register_317) as f32 + 1.0 / (1u64 << register_318) as f32 + 1.0 / (1u64 << register_319) as f32;

	let [register_320, register_321, register_322, register_323, register_324] = split_registers::<5>(words[64]);
	raw_estimate += 1.0 / (1u64 << register_320) as f32 + 1.0 / (1u64 << register_321) as f32 + 1.0 / (1u64 << register_322) as f32 + 1.0 / (1u64 << register_323) as f32 + 1.0 / (1u64 << register_324) as f32;

	let [register_325, register_326, register_327, register_328, register_329] = split_registers::<5>(words[65]);
	raw_estimate += 1.0 / (1u64 << register_325) as f32 + 1.0 / (1u64 << register_326) as f32 + 1.0 / (1u64 << register_327) as f32 + 1.0 / (1u64 << register_328) as f32 + 1.0 / (1u64 << register_329) as f32;

	let [register_330, register_331, register_332, register_333, register_334] = split_registers::<5>(words[66]);
	raw_estimate += 1.0 / (1u64 << register_330) as f32 + 1.0 / (1u64 << register_331) as f32 + 1.0 / (1u64 << register_332) as f32 + 1.0 / (1u64 << register_333) as f32 + 1.0 / (1u64 << register_334) as f32;

	let [register_335, register_336, register_337, register_338, register_339] = split_registers::<5>(words[67]);
	raw_estimate += 1.0 / (1u64 << register_335) as f32 + 1.0 / (1u64 << register_336) as f32 + 1.0 / (1u64 << register_337) as f32 + 1.0 / (1u64 << register_338) as f32 + 1.0 / (1u64 << register_339) as f32;

	let [register_340, register_341, register_342, register_343, register_344] = split_registers::<5>(words[68]);
	raw_estimate += 1.0 / (1u64 << register_340) as f32 + 1.0 / (1u64 << register_341) as f32 + 1.0 / (1u64 << register_342) as f32 + 1.0 / (1u64 << register_343) as f32 + 1.0 / (1u64 << register_344) as f32;

	let [register_345, register_346, register_347, register_348, register_349] = split_registers::<5>(words[69]);
	raw_estimate += 1.0 / (1u64 << register_345) as f32 + 1.0 / (1u64 << register_346) as f32 + 1.0 / (1u64 << register_347) as f32 + 1.0 / (1u64 << register_348) as f32 + 1.0 / (1u64 << register_349) as f32;

	let [register_350, register_351, register_352, register_353, register_354] = split_registers::<5>(words[70]);
	raw_estimate += 1.0 / (1u64 << register_350) as f32 + 1.0 / (1u64 << register_351) as f32 + 1.0 / (1u64 << register_352) as f32 + 1.0 / (1u64 << register_353) as f32 + 1.0 / (1u64 << register_354) as f32;

	let [register_355, register_356, register_357, register_358, register_359] = split_registers::<5>(words[71]);
	raw_estimate += 1.0 / (1u64 << register_355) as f32 + 1.0 / (1u64 << register_356) as f32 + 1.0 / (1u64 << register_357) as f32 + 1.0 / (1u64 << register_358) as f32 + 1.0 / (1u64 << register_359) as f32;

	let [register_360, register_361, register_362, register_363, register_364] = split_registers::<5>(words[72]);
	raw_estimate += 1.0 / (1u64 << register_360) as f32 + 1.0 / (1u64 << register_361) as f32 + 1.0 / (1u64 << register_362) as f32 + 1.0 / (1u64 << register_363) as f32 + 1.0 / (1u64 << register_364) as f32;

	let [register_365, register_366, register_367, register_368, register_369] = split_registers::<5>(words[73]);
	raw_estimate += 1.0 / (1u64 << register_365) as f32 + 1.0 / (1u64 << register_366) as f32 + 1.0 / (1u64 << register_367) as f32 + 1.0 / (1u64 << register_368) as f32 + 1.0 / (1u64 << register_369) as f32;

	let [register_370, register_371, register_372, register_373, register_374] = split_registers::<5>(words[74]);
	raw_estimate += 1.0 / (1u64 << register_370) as f32 + 1.0 / (1u64 << register_371) as f32 + 1.0 / (1u64 << register_372) as f32 + 1.0 / (1u64 << register_373) as f32 + 1.0 / (1u64 << register_374) as f32;

	let [register_375, register_376, register_377, register_378, register_379] = split_registers::<5>(words[75]);
	raw_estimate += 1.0 / (1u64 << register_375) as f32 + 1.0 / (1u64 << register_376) as f32 + 1.0 / (1u64 << register_377) as f32 + 1.0 / (1u64 << register_378) as f32 + 1.0 / (1u64 << register_379) as f32;

	let [register_380, register_381, register_382, register_383, register_384] = split_registers::<5>(words[76]);
	raw_estimate += 1.0 / (1u64 << register_380) as f32 + 1.0 / (1u64 << register_381) as f32 + 1.0 / (1u64 << register_382) as f32 + 1.0 / (1u64 << register_383) as f32 + 1.0 / (1u64 << register_384) as f32;

	let [register_385, register_386, register_387, register_388, register_389] = split_registers::<5>(words[77]);
	raw_estimate += 1.0 / (1u64 << register_385) as f32 + 1.0 / (1u64 << register_386) as f32 + 1.0 / (1u64 << register_387) as f32 + 1.0 / (1u64 << register_388) as f32 + 1.0 / (1u64 << register_389) as f32;

	let [register_390, register_391, register_392, register_393, register_394] = split_registers::<5>(words[78]);
	raw_estimate += 1.0 / (1u64 << register_390) as f32 + 1.0 / (1u64 << register_391) as f32 + 1.0 / (1u64 << register_392) as f32 + 1.0 / (1u64 << register_393) as f32 + 1.0 / (1u64 << register_394) as f32;

	let [register_395, register_396, register_397, register_398, register_399] = split_registers::<5>(words[79]);
	raw_estimate += 1.0 / (1u64 << register_395) as f32 + 1.0 / (1u64 << register_396) as f32 + 1.0 / (1u64 << register_397) as f32 + 1.0 / (1u64 << register_398) as f32 + 1.0 / (1u64 << register_399) as f32;

	let [register_400, register_401, register_402, register_403, register_404] = split_registers::<5>(words[80]);
	raw_estimate += 1.0 / (1u64 << register_400) as f32 + 1.0 / (1u64 << register_401) as f32 + 1.0 / (1u64 << register_402) as f32 + 1.0 / (1u64 << register_403) as f32 + 1.0 / (1u64 << register_404) as f32;

	let [register_405, register_406, register_407, register_408, register_409] = split_registers::<5>(words[81]);
	raw_estimate += 1.0 / (1u64 << register_405) as f32 + 1.0 / (1u64 << register_406) as f32 + 1.0 / (1u64 << register_407) as f32 + 1.0 / (1u64 << register_408) as f32 + 1.0 / (1u64 << register_409) as f32;

	let [register_410, register_411, register_412, register_413, register_414] = split_registers::<5>(words[82]);
	raw_estimate += 1.0 / (1u64 << register_410) as f32 + 1.0 / (1u64 << register_411) as f32 + 1.0 / (1u64 << register_412) as f32 + 1.0 / (1u64 << register_413) as f32 + 1.0 / (1u64 << register_414) as f32;

	let [register_415, register_416, register_417, register_418, register_419] = split_registers::<5>(words[83]);
	raw_estimate += 1.0 / (1u64 << register_415) as f32 + 1.0 / (1u64 << register_416) as f32 + 1.0 / (1u64 << register_417) as f32 + 1.0 / (1u64 << register_418) as f32 + 1.0 / (1u64 << register_419) as f32;

	let [register_420, register_421, register_422, register_423, register_424] = split_registers::<5>(words[84]);
	raw_estimate += 1.0 / (1u64 << register_420) as f32 + 1.0 / (1u64 << register_421) as f32 + 1.0 / (1u64 << register_422) as f32 + 1.0 / (1u64 << register_423) as f32 + 1.0 / (1u64 << register_424) as f32;

	let [register_425, register_426, register_427, register_428, register_429] = split_registers::<5>(words[85]);
	raw_estimate += 1.0 / (1u64 << register_425) as f32 + 1.0 / (1u64 << register_426) as f32 + 1.0 / (1u64 << register_427) as f32 + 1.0 / (1u64 << register_428) as f32 + 1.0 / (1u64 << register_429) as f32;

	let [register_430, register_431, register_432, register_433, register_434] = split_registers::<5>(words[86]);
	raw_estimate += 1.0 / (1u64 << register_430) as f32 + 1.0 / (1u64 << register_431) as f32 + 1.0 / (1u64 << register_432) as f32 + 1.0 / (1u64 << register_433) as f32 + 1.0 / (1u64 << register_434) as f32;

	let [register_435, register_436, register_437, register_438, register_439] = split_registers::<5>(words[87]);
	raw_estimate += 1.0 / (1u64 << register_435) as f32 + 1.0 / (1u64 << register_436) as f32 + 1.0 / (1u64 << register_437) as f32 + 1.0 / (1u64 << register_438) as f32 + 1.0 / (1u64 << register_439) as f32;

	let [register_440, register_441, register_442, register_443, register_444] = split_registers::<5>(words[88]);
	raw_estimate += 1.0 / (1u64 << register_440) as f32 + 1.0 / (1u64 << register_441) as f32 + 1.0 / (1u64 << register_442) as f32 + 1.0 / (1u64 << register_443) as f32 + 1.0 / (1u64 << register_444) as f32;

	let [register_445, register_446, register_447, register_448, register_449] = split_registers::<5>(words[89]);
	raw_estimate += 1.0 / (1u64 << register_445) as f32 + 1.0 / (1u64 << register_446) as f32 + 1.0 / (1u64 << register_447) as f32 + 1.0 / (1u64 << register_448) as f32 + 1.0 / (1u64 << register_449) as f32;

	let [register_450, register_451, register_452, register_453, register_454] = split_registers::<5>(words[90]);
	raw_estimate += 1.0 / (1u64 << register_450) as f32 + 1.0 / (1u64 << register_451) as f32 + 1.0 / (1u64 << register_452) as f32 + 1.0 / (1u64 << register_453) as f32 + 1.0 / (1u64 << register_454) as f32;

	let [register_455, register_456, register_457, register_458, register_459] = split_registers::<5>(words[91]);
	raw_estimate += 1.0 / (1u64 << register_455) as f32 + 1.0 / (1u64 << register_456) as f32 + 1.0 / (1u64 << register_457) as f32 + 1.0 / (1u64 << register_458) as f32 + 1.0 / (1u64 << register_459) as f32;

	let [register_460, register_461, register_462, register_463, register_464] = split_registers::<5>(words[92]);
	raw_estimate += 1.0 / (1u64 << register_460) as f32 + 1.0 / (1u64 << register_461) as f32 + 1.0 / (1u64 << register_462) as f32 + 1.0 / (1u64 << register_463) as f32 + 1.0 / (1u64 << register_464) as f32;

	let [register_465, register_466, register_467, register_468, register_469] = split_registers::<5>(words[93]);
	raw_estimate += 1.0 / (1u64 << register_465) as f32 + 1.0 / (1u64 << register_466) as f32 + 1.0 / (1u64 << register_467) as f32 + 1.0 / (1u64 << register_468) as f32 + 1.0 / (1u64 << register_469) as f32;

	let [register_470, register_471, register_472, register_473, register_474] = split_registers::<5>(words[94]);
	raw_estimate += 1.0 / (1u64 << register_470) as f32 + 1.0 / (1u64 << register_471) as f32 + 1.0 / (1u64 << register_472) as f32 + 1.0 / (1u64 << register_473) as f32 + 1.0 / (1u64 << register_474) as f32;

	let [register_475, register_476, register_477, register_478, register_479] = split_registers::<5>(words[95]);
	raw_estimate += 1.0 / (1u64 << register_475) as f32 + 1.0 / (1u64 << register_476) as f32 + 1.0 / (1u64 << register_477) as f32 + 1.0 / (1u64 << register_478) as f32 + 1.0 / (1u64 << register_479) as f32;

	let [register_480, register_481, register_482, register_483, register_484] = split_registers::<5>(words[96]);
	raw_estimate += 1.0 / (1u64 << register_480) as f32 + 1.0 / (1u64 << register_481) as f32 + 1.0 / (1u64 << register_482) as f32 + 1.0 / (1u64 << register_483) as f32 + 1.0 / (1u64 << register_484) as f32;

	let [register_485, register_486, register_487, register_488, register_489] = split_registers::<5>(words[97]);
	raw_estimate += 1.0 / (1u64 << register_485) as f32 + 1.0 / (1u64 << register_486) as f32 + 1.0 / (1u64 << register_487) as f32 + 1.0 / (1u64 << register_488) as f32 + 1.0 / (1u64 << register_489) as f32;

	let [register_490, register_491, register_492, register_493, register_494] = split_registers::<5>(words[98]);
	raw_estimate += 1.0 / (1u64 << register_490) as f32 + 1.0 / (1u64 << register_491) as f32 + 1.0 / (1u64 << register_492) as f32 + 1.0 / (1u64 << register_493) as f32 + 1.0 / (1u64 << register_494) as f32;

	let [register_495, register_496, register_497, register_498, register_499] = split_registers::<5>(words[99]);
	raw_estimate += 1.0 / (1u64 << register_495) as f32 + 1.0 / (1u64 << register_496) as f32 + 1.0 / (1u64 << register_497) as f32 + 1.0 / (1u64 << register_498) as f32 + 1.0 / (1u64 << register_499) as f32;

	let [register_500, register_501, register_502, register_503, register_504] = split_registers::<5>(words[100]);
	raw_estimate += 1.0 / (1u64 << register_500) as f32 + 1.0 / (1u64 << register_501) as f32 + 1.0 / (1u64 << register_502) as f32 + 1.0 / (1u64 << register_503) as f32 + 1.0 / (1u64 << register_504) as f32;

	let [register_505, register_506, register_507, register_508, register_509] = split_registers::<5>(words[101]);
	raw_estimate += 1.0 / (1u64 << register_505) as f32 + 1.0 / (1u64 << register_506) as f32 + 1.0 / (1u64 << register_507) as f32 + 1.0 / (1u64 << register_508) as f32 + 1.0 / (1u64 << register_509) as f32;

	let [register_510, register_511, register_512, register_513, register_514] = split_registers::<5>(words[102]);
	raw_estimate += 1.0 / (1u64 << register_510) as f32 + 1.0 / (1u64 << register_511) as f32 + 1.0 / (1u64 << register_512) as f32 + 1.0 / (1u64 << register_513) as f32 + 1.0 / (1u64 << register_514) as f32;

	let [register_515, register_516, register_517, register_518, register_519] = split_registers::<5>(words[103]);
	raw_estimate += 1.0 / (1u64 << register_515) as f32 + 1.0 / (1u64 << register_516) as f32 + 1.0 / (1u64 << register_517) as f32 + 1.0 / (1u64 << register_518) as f32 + 1.0 / (1u64 << register_519) as f32;

	let [register_520, register_521, register_522, register_523, register_524] = split_registers::<5>(words[104]);
	raw_estimate += 1.0 / (1u64 << register_520) as f32 + 1.0 / (1u64 << register_521) as f32 + 1.0 / (1u64 << register_522) as f32 + 1.0 / (1u64 << register_523) as f32 + 1.0 / (1u64 << register_524) as f32;

	let [register_525, register_526, register_527, register_528, register_529] = split_registers::<5>(words[105]);
	raw_estimate += 1.0 / (1u64 << register_525) as f32 + 1.0 / (1u64 << register_526) as f32 + 1.0 / (1u64 << register_527) as f32 + 1.0 / (1u64 << register_528) as f32 + 1.0 / (1u64 << register_529) as f32;

	let [register_530, register_531, register_532, register_533, register_534] = split_registers::<5>(words[106]);
	raw_estimate += 1.0 / (1u64 << register_530) as f32 + 1.0 / (1u64 << register_531) as f32 + 1.0 / (1u64 << register_532) as f32 + 1.0 / (1u64 << register_533) as f32 + 1.0 / (1u64 << register_534) as f32;

	let [register_535, register_536, register_537, register_538, register_539] = split_registers::<5>(words[107]);
	raw_estimate += 1.0 / (1u64 << register_535) as f32 + 1.0 / (1u64 << register_536) as f32 + 1.0 / (1u64 << register_537) as f32 + 1.0 / (1u64 << register_538) as f32 + 1.0 / (1u64 << register_539) as f32;

	let [register_540, register_541, register_542, register_543, register_544] = split_registers::<5>(words[108]);
	raw_estimate += 1.0 / (1u64 << register_540) as f32 + 1.0 / (1u64 << register_541) as f32 + 1.0 / (1u64 << register_542) as f32 + 1.0 / (1u64 << register_543) as f32 + 1.0 / (1u64 << register_544) as f32;

	let [register_545, register_546, register_547, register_548, register_549] = split_registers::<5>(words[109]);
	raw_estimate += 1.0 / (1u64 << register_545) as f32 + 1.0 / (1u64 << register_546) as f32 + 1.0 / (1u64 << register_547) as f32 + 1.0 / (1u64 << register_548) as f32 + 1.0 / (1u64 << register_549) as f32;

	let [register_550, register_551, register_552, register_553, register_554] = split_registers::<5>(words[110]);
	raw_estimate += 1.0 / (1u64 << register_550) as f32 + 1.0 / (1u64 << register_551) as f32 + 1.0 / (1u64 << register_552) as f32 + 1.0 / (1u64 << register_553) as f32 + 1.0 / (1u64 << register_554) as f32;

	let [register_555, register_556, register_557, register_558, register_559] = split_registers::<5>(words[111]);
	raw_estimate += 1.0 / (1u64 << register_555) as f32 + 1.0 / (1u64 << register_556) as f32 + 1.0 / (1u64 << register_557) as f32 + 1.0 / (1u64 << register_558) as f32 + 1.0 / (1u64 << register_559) as f32;

	let [register_560, register_561, register_562, register_563, register_564] = split_registers::<5>(words[112]);
	raw_estimate += 1.0 / (1u64 << register_560) as f32 + 1.0 / (1u64 << register_561) as f32 + 1.0 / (1u64 << register_562) as f32 + 1.0 / (1u64 << register_563) as f32 + 1.0 / (1u64 << register_564) as f32;

	let [register_565, register_566, register_567, register_568, register_569] = split_registers::<5>(words[113]);
	raw_estimate += 1.0 / (1u64 << register_565) as f32 + 1.0 / (1u64 << register_566) as f32 + 1.0 / (1u64 << register_567) as f32 + 1.0 / (1u64 << register_568) as f32 + 1.0 / (1u64 << register_569) as f32;

	let [register_570, register_571, register_572, register_573, register_574] = split_registers::<5>(words[114]);
	raw_estimate += 1.0 / (1u64 << register_570) as f32 + 1.0 / (1u64 << register_571) as f32 + 1.0 / (1u64 << register_572) as f32 + 1.0 / (1u64 << register_573) as f32 + 1.0 / (1u64 << register_574) as f32;

	let [register_575, register_576, register_577, register_578, register_579] = split_registers::<5>(words[115]);
	raw_estimate += 1.0 / (1u64 << register_575) as f32 + 1.0 / (1u64 << register_576) as f32 + 1.0 / (1u64 << register_577) as f32 + 1.0 / (1u64 << register_578) as f32 + 1.0 / (1u64 << register_579) as f32;

	let [register_580, register_581, register_582, register_583, register_584] = split_registers::<5>(words[116]);
	raw_estimate += 1.0 / (1u64 << register_580) as f32 + 1.0 / (1u64 << register_581) as f32 + 1.0 / (1u64 << register_582) as f32 + 1.0 / (1u64 << register_583) as f32 + 1.0 / (1u64 << register_584) as f32;

	let [register_585, register_586, register_587, register_588, register_589] = split_registers::<5>(words[117]);
	raw_estimate += 1.0 / (1u64 << register_585) as f32 + 1.0 / (1u64 << register_586) as f32 + 1.0 / (1u64 << register_587) as f32 + 1.0 / (1u64 << register_588) as f32 + 1.0 / (1u64 << register_589) as f32;

	let [register_590, register_591, register_592, register_593, register_594] = split_registers::<5>(words[118]);
	raw_estimate += 1.0 / (1u64 << register_590) as f32 + 1.0 / (1u64 << register_591) as f32 + 1.0 / (1u64 << register_592) as f32 + 1.0 / (1u64 << register_593) as f32 + 1.0 / (1u64 << register_594) as f32;

	let [register_595, register_596, register_597, register_598, register_599] = split_registers::<5>(words[119]);
	raw_estimate += 1.0 / (1u64 << register_595) as f32 + 1.0 / (1u64 << register_596) as f32 + 1.0 / (1u64 << register_597) as f32 + 1.0 / (1u64 << register_598) as f32 + 1.0 / (1u64 << register_599) as f32;

	let [register_600, register_601, register_602, register_603, register_604] = split_registers::<5>(words[120]);
	raw_estimate += 1.0 / (1u64 << register_600) as f32 + 1.0 / (1u64 << register_601) as f32 + 1.0 / (1u64 << register_602) as f32 + 1.0 / (1u64 << register_603) as f32 + 1.0 / (1u64 << register_604) as f32;

	let [register_605, register_606, register_607, register_608, register_609] = split_registers::<5>(words[121]);
	raw_estimate += 1.0 / (1u64 << register_605) as f32 + 1.0 / (1u64 << register_606) as f32 + 1.0 / (1u64 << register_607) as f32 + 1.0 / (1u64 << register_608) as f32 + 1.0 / (1u64 << register_609) as f32;

	let [register_610, register_611, register_612, register_613, register_614] = split_registers::<5>(words[122]);
	raw_estimate += 1.0 / (1u64 << register_610) as f32 + 1.0 / (1u64 << register_611) as f32 + 1.0 / (1u64 << register_612) as f32 + 1.0 / (1u64 << register_613) as f32 + 1.0 / (1u64 << register_614) as f32;

	let [register_615, register_616, register_617, register_618, register_619] = split_registers::<5>(words[123]);
	raw_estimate += 1.0 / (1u64 << register_615) as f32 + 1.0 / (1u64 << register_616) as f32 + 1.0 / (1u64 << register_617) as f32 + 1.0 / (1u64 << register_618) as f32 + 1.0 / (1u64 << register_619) as f32;

	let [register_620, register_621, register_622, register_623, register_624] = split_registers::<5>(words[124]);
	raw_estimate += 1.0 / (1u64 << register_620) as f32 + 1.0 / (1u64 << register_621) as f32 + 1.0 / (1u64 << register_622) as f32 + 1.0 / (1u64 << register_623) as f32 + 1.0 / (1u64 << register_624) as f32;

	let [register_625, register_626, register_627, register_628, register_629] = split_registers::<5>(words[125]);
	raw_estimate += 1.0 / (1u64 << register_625) as f32 + 1.0 / (1u64 << register_626) as f32 + 1.0 / (1u64 << register_627) as f32 + 1.0 / (1u64 << register_628) as f32 + 1.0 / (1u64 << register_629) as f32;

	let [register_630, register_631, register_632, register_633, register_634] = split_registers::<5>(words[126]);
	raw_estimate += 1.0 / (1u64 << register_630) as f32 + 1.0 / (1u64 << register_631) as f32 + 1.0 / (1u64 << register_632) as f32 + 1.0 / (1u64 << register_633) as f32 + 1.0 / (1u64 << register_634) as f32;

	let [register_635, register_636, register_637, register_638, register_639] = split_registers::<5>(words[127]);
	raw_estimate += 1.0 / (1u64 << register_635) as f32 + 1.0 / (1u64 << register_636) as f32 + 1.0 / (1u64 << register_637) as f32 + 1.0 / (1u64 << register_638) as f32 + 1.0 / (1u64 << register_639) as f32;

	let [register_640, register_641, register_642, register_643, register_644] = split_registers::<5>(words[128]);
	raw_estimate += 1.0 / (1u64 << register_640) as f32 + 1.0 / (1u64 << register_641) as f32 + 1.0 / (1u64 << register_642) as f32 + 1.0 / (1u64 << register_643) as f32 + 1.0 / (1u64 << register_644) as f32;

	let [register_645, register_646, register_647, register_648, register_649] = split_registers::<5>(words[129]);
	raw_estimate += 1.0 / (1u64 << register_645) as f32 + 1.0 / (1u64 << register_646) as f32 + 1.0 / (1u64 << register_647) as f32 + 1.0 / (1u64 << register_648) as f32 + 1.0 / (1u64 << register_649) as f32;

	let [register_650, register_651, register_652, register_653, register_654] = split_registers::<5>(words[130]);
	raw_estimate += 1.0 / (1u64 << register_650) as f32 + 1.0 / (1u64 << register_651) as f32 + 1.0 / (1u64 << register_652) as f32 + 1.0 / (1u64 << register_653) as f32 + 1.0 / (1u64 << register_654) as f32;

	let [register_655, register_656, register_657, register_658, register_659] = split_registers::<5>(words[131]);
	raw_estimate += 1.0 / (1u64 << register_655) as f32 + 1.0 / (1u64 << register_656) as f32 + 1.0 / (1u64 << register_657) as f32 + 1.0 / (1u64 << register_658) as f32 + 1.0 / (1u64 << register_659) as f32;

	let [register_660, register_661, register_662, register_663, register_664] = split_registers::<5>(words[132]);
	raw_estimate += 1.0 / (1u64 << register_660) as f32 + 1.0 / (1u64 << register_661) as f32 + 1.0 / (1u64 << register_662) as f32 + 1.0 / (1u64 << register_663) as f32 + 1.0 / (1u64 << register_664) as f32;

	let [register_665, register_666, register_667, register_668, register_669] = split_registers::<5>(words[133]);
	raw_estimate += 1.0 / (1u64 << register_665) as f32 + 1.0 / (1u64 << register_666) as f32 + 1.0 / (1u64 << register_667) as f32 + 1.0 / (1u64 << register_668) as f32 + 1.0 / (1u64 << register_669) as f32;

	let [register_670, register_671, register_672, register_673, register_674] = split_registers::<5>(words[134]);
	raw_estimate += 1.0 / (1u64 << register_670) as f32 + 1.0 / (1u64 << register_671) as f32 + 1.0 / (1u64 << register_672) as f32 + 1.0 / (1u64 << register_673) as f32 + 1.0 / (1u64 << register_674) as f32;

	let [register_675, register_676, register_677, register_678, register_679] = split_registers::<5>(words[135]);
	raw_estimate += 1.0 / (1u64 << register_675) as f32 + 1.0 / (1u64 << register_676) as f32 + 1.0 / (1u64 << register_677) as f32 + 1.0 / (1u64 << register_678) as f32 + 1.0 / (1u64 << register_679) as f32;

	let [register_680, register_681, register_682, register_683, register_684] = split_registers::<5>(words[136]);
	raw_estimate += 1.0 / (1u64 << register_680) as f32 + 1.0 / (1u64 << register_681) as f32 + 1.0 / (1u64 << register_682) as f32 + 1.0 / (1u64 << register_683) as f32 + 1.0 / (1u64 << register_684) as f32;

	let [register_685, register_686, register_687, register_688, register_689] = split_registers::<5>(words[137]);
	raw_estimate += 1.0 / (1u64 << register_685) as f32 + 1.0 / (1u64 << register_686) as f32 + 1.0 / (1u64 << register_687) as f32 + 1.0 / (1u64 << register_688) as f32 + 1.0 / (1u64 << register_689) as f32;

	let [register_690, register_691, register_692, register_693, register_694] = split_registers::<5>(words[138]);
	raw_estimate += 1.0 / (1u64 << register_690) as f32 + 1.0 / (1u64 << register_691) as f32 + 1.0 / (1u64 << register_692) as f32 + 1.0 / (1u64 << register_693) as f32 + 1.0 / (1u64 << register_694) as f32;

	let [register_695, register_696, register_697, register_698, register_699] = split_registers::<5>(words[139]);
	raw_estimate += 1.0 / (1u64 << register_695) as f32 + 1.0 / (1u64 << register_696) as f32 + 1.0 / (1u64 << register_697) as f32 + 1.0 / (1u64 << register_698) as f32 + 1.0 / (1u64 << register_699) as f32;

	let [register_700, register_701, register_702, register_703, register_704] = split_registers::<5>(words[140]);
	raw_estimate += 1.0 / (1u64 << register_700) as f32 + 1.0 / (1u64 << register_701) as f32 + 1.0 / (1u64 << register_702) as f32 + 1.0 / (1u64 << register_703) as f32 + 1.0 / (1u64 << register_704) as f32;

	let [register_705, register_706, register_707, register_708, register_709] = split_registers::<5>(words[141]);
	raw_estimate += 1.0 / (1u64 << register_705) as f32 + 1.0 / (1u64 << register_706) as f32 + 1.0 / (1u64 << register_707) as f32 + 1.0 / (1u64 << register_708) as f32 + 1.0 / (1u64 << register_709) as f32;

	let [register_710, register_711, register_712, register_713, register_714] = split_registers::<5>(words[142]);
	raw_estimate += 1.0 / (1u64 << register_710) as f32 + 1.0 / (1u64 << register_711) as f32 + 1.0 / (1u64 << register_712) as f32 + 1.0 / (1u64 << register_713) as f32 + 1.0 / (1u64 << register_714) as f32;

	let [register_715, register_716, register_717, register_718, register_719] = split_registers::<5>(words[143]);
	raw_estimate += 1.0 / (1u64 << register_715) as f32 + 1.0 / (1u64 << register_716) as f32 + 1.0 / (1u64 << register_717) as f32 + 1.0 / (1u64 << register_718) as f32 + 1.0 / (1u64 << register_719) as f32;

	let [register_720, register_721, register_722, register_723, register_724] = split_registers::<5>(words[144]);
	raw_estimate += 1.0 / (1u64 << register_720) as f32 + 1.0 / (1u64 << register_721) as f32 + 1.0 / (1u64 << register_722) as f32 + 1.0 / (1u64 << register_723) as f32 + 1.0 / (1u64 << register_724) as f32;

	let [register_725, register_726, register_727, register_728, register_729] = split_registers::<5>(words[145]);
	raw_estimate += 1.0 / (1u64 << register_725) as f32 + 1.0 / (1u64 << register_726) as f32 + 1.0 / (1u64 << register_727) as f32 + 1.0 / (1u64 << register_728) as f32 + 1.0 / (1u64 << register_729) as f32;

	let [register_730, register_731, register_732, register_733, register_734] = split_registers::<5>(words[146]);
	raw_estimate += 1.0 / (1u64 << register_730) as f32 + 1.0 / (1u64 << register_731) as f32 + 1.0 / (1u64 << register_732) as f32 + 1.0 / (1u64 << register_733) as f32 + 1.0 / (1u64 << register_734) as f32;

	let [register_735, register_736, register_737, register_738, register_739] = split_registers::<5>(words[147]);
	raw_estimate += 1.0 / (1u64 << register_735) as f32 + 1.0 / (1u64 << register_736) as f32 + 1.0 / (1u64 << register_737) as f32 + 1.0 / (1u64 << register_738) as f32 + 1.0 / (1u64 << register_739) as f32;

	let [register_740, register_741, register_742, register_743, register_744] = split_registers::<5>(words[148]);
	raw_estimate += 1.0 / (1u64 << register_740) as f32 + 1.0 / (1u64 << register_741) as f32 + 1.0 / (1u64 << register_742) as f32 + 1.0 / (1u64 << register_743) as f32 + 1.0 / (1u64 << register_744) as f32;

	let [register_745, register_746, register_747, register_748, register_749] = split_registers::<5>(words[149]);
	raw_estimate += 1.0 / (1u64 << register_745) as f32 + 1.0 / (1u64 << register_746) as f32 + 1.0 / (1u64 << register_747) as f32 + 1.0 / (1u64 << register_748) as f32 + 1.0 / (1u64 << register_749) as f32;

	let [register_750, register_751, register_752, register_753, register_754] = split_registers::<5>(words[150]);
	raw_estimate += 1.0 / (1u64 << register_750) as f32 + 1.0 / (1u64 << register_751) as f32 + 1.0 / (1u64 << register_752) as f32 + 1.0 / (1u64 << register_753) as f32 + 1.0 / (1u64 << register_754) as f32;

	let [register_755, register_756, register_757, register_758, register_759] = split_registers::<5>(words[151]);
	raw_estimate += 1.0 / (1u64 << register_755) as f32 + 1.0 / (1u64 << register_756) as f32 + 1.0 / (1u64 << register_757) as f32 + 1.0 / (1u64 << register_758) as f32 + 1.0 / (1u64 << register_759) as f32;

	let [register_760, register_761, register_762, register_763, register_764] = split_registers::<5>(words[152]);
	raw_estimate += 1.0 / (1u64 << register_760) as f32 + 1.0 / (1u64 << register_761) as f32 + 1.0 / (1u64 << register_762) as f32 + 1.0 / (1u64 << register_763) as f32 + 1.0 / (1u64 << register_764) as f32;

	let [register_765, register_766, register_767, register_768, register_769] = split_registers::<5>(words[153]);
	raw_estimate += 1.0 / (1u64 << register_765) as f32 + 1.0 / (1u64 << register_766) as f32 + 1.0 / (1u64 << register_767) as f32 + 1.0 / (1u64 << register_768) as f32 + 1.0 / (1u64 << register_769) as f32;

	let [register_770, register_771, register_772, register_773, register_774] = split_registers::<5>(words[154]);
	raw_estimate += 1.0 / (1u64 << register_770) as f32 + 1.0 / (1u64 << register_771) as f32 + 1.0 / (1u64 << register_772) as f32 + 1.0 / (1u64 << register_773) as f32 + 1.0 / (1u64 << register_774) as f32;

	let [register_775, register_776, register_777, register_778, register_779] = split_registers::<5>(words[155]);
	raw_estimate += 1.0 / (1u64 << register_775) as f32 + 1.0 / (1u64 << register_776) as f32 + 1.0 / (1u64 << register_777) as f32 + 1.0 / (1u64 << register_778) as f32 + 1.0 / (1u64 << register_779) as f32;

	let [register_780, register_781, register_782, register_783, register_784] = split_registers::<5>(words[156]);
	raw_estimate += 1.0 / (1u64 << register_780) as f32 + 1.0 / (1u64 << register_781) as f32 + 1.0 / (1u64 << register_782) as f32 + 1.0 / (1u64 << register_783) as f32 + 1.0 / (1u64 << register_784) as f32;

	let [register_785, register_786, register_787, register_788, register_789] = split_registers::<5>(words[157]);
	raw_estimate += 1.0 / (1u64 << register_785) as f32 + 1.0 / (1u64 << register_786) as f32 + 1.0 / (1u64 << register_787) as f32 + 1.0 / (1u64 << register_788) as f32 + 1.0 / (1u64 << register_789) as f32;

	let [register_790, register_791, register_792, register_793, register_794] = split_registers::<5>(words[158]);
	raw_estimate += 1.0 / (1u64 << register_790) as f32 + 1.0 / (1u64 << register_791) as f32 + 1.0 / (1u64 << register_792) as f32 + 1.0 / (1u64 << register_793) as f32 + 1.0 / (1u64 << register_794) as f32;

	let [register_795, register_796, register_797, register_798, register_799] = split_registers::<5>(words[159]);
	raw_estimate += 1.0 / (1u64 << register_795) as f32 + 1.0 / (1u64 << register_796) as f32 + 1.0 / (1u64 << register_797) as f32 + 1.0 / (1u64 << register_798) as f32 + 1.0 / (1u64 << register_799) as f32;

	let [register_800, register_801, register_802, register_803, register_804] = split_registers::<5>(words[160]);
	raw_estimate += 1.0 / (1u64 << register_800) as f32 + 1.0 / (1u64 << register_801) as f32 + 1.0 / (1u64 << register_802) as f32 + 1.0 / (1u64 << register_803) as f32 + 1.0 / (1u64 << register_804) as f32;

	let [register_805, register_806, register_807, register_808, register_809] = split_registers::<5>(words[161]);
	raw_estimate += 1.0 / (1u64 << register_805) as f32 + 1.0 / (1u64 << register_806) as f32 + 1.0 / (1u64 << register_807) as f32 + 1.0 / (1u64 << register_808) as f32 + 1.0 / (1u64 << register_809) as f32;

	let [register_810, register_811, register_812, register_813, register_814] = split_registers::<5>(words[162]);
	raw_estimate += 1.0 / (1u64 << register_810) as f32 + 1.0 / (1u64 << register_811) as f32 + 1.0 / (1u64 << register_812) as f32 + 1.0 / (1u64 << register_813) as f32 + 1.0 / (1u64 << register_814) as f32;

	let [register_815, register_816, register_817, register_818, register_819] = split_registers::<5>(words[163]);
	raw_estimate += 1.0 / (1u64 << register_815) as f32 + 1.0 / (1u64 << register_816) as f32 + 1.0 / (1u64 << register_817) as f32 + 1.0 / (1u64 << register_818) as f32 + 1.0 / (1u64 << register_819) as f32;

	let [register_820, register_821, register_822, register_823, register_824] = split_registers::<5>(words[164]);
	raw_estimate += 1.0 / (1u64 << register_820) as f32 + 1.0 / (1u64 << register_821) as f32 + 1.0 / (1u64 << register_822) as f32 + 1.0 / (1u64 << register_823) as f32 + 1.0 / (1u64 << register_824) as f32;

	let [register_825, register_826, register_827, register_828, register_829] = split_registers::<5>(words[165]);
	raw_estimate += 1.0 / (1u64 << register_825) as f32 + 1.0 / (1u64 << register_826) as f32 + 1.0 / (1u64 << register_827) as f32 + 1.0 / (1u64 << register_828) as f32 + 1.0 / (1u64 << register_829) as f32;

	let [register_830, register_831, register_832, register_833, register_834] = split_registers::<5>(words[166]);
	raw_estimate += 1.0 / (1u64 << register_830) as f32 + 1.0 / (1u64 << register_831) as f32 + 1.0 / (1u64 << register_832) as f32 + 1.0 / (1u64 << register_833) as f32 + 1.0 / (1u64 << register_834) as f32;

	let [register_835, register_836, register_837, register_838, register_839] = split_registers::<5>(words[167]);
	raw_estimate += 1.0 / (1u64 << register_835) as f32 + 1.0 / (1u64 << register_836) as f32 + 1.0 / (1u64 << register_837) as f32 + 1.0 / (1u64 << register_838) as f32 + 1.0 / (1u64 << register_839) as f32;

	let [register_840, register_841, register_842, register_843, register_844] = split_registers::<5>(words[168]);
	raw_estimate += 1.0 / (1u64 << register_840) as f32 + 1.0 / (1u64 << register_841) as f32 + 1.0 / (1u64 << register_842) as f32 + 1.0 / (1u64 << register_843) as f32 + 1.0 / (1u64 << register_844) as f32;

	let [register_845, register_846, register_847, register_848, register_849] = split_registers::<5>(words[169]);
	raw_estimate += 1.0 / (1u64 << register_845) as f32 + 1.0 / (1u64 << register_846) as f32 + 1.0 / (1u64 << register_847) as f32 + 1.0 / (1u64 << register_848) as f32 + 1.0 / (1u64 << register_849) as f32;

	let [register_850, register_851, register_852, register_853, register_854] = split_registers::<5>(words[170]);
	raw_estimate += 1.0 / (1u64 << register_850) as f32 + 1.0 / (1u64 << register_851) as f32 + 1.0 / (1u64 << register_852) as f32 + 1.0 / (1u64 << register_853) as f32 + 1.0 / (1u64 << register_854) as f32;

	let [register_855, register_856, register_857, register_858, register_859] = split_registers::<5>(words[171]);
	raw_estimate += 1.0 / (1u64 << register_855) as f32 + 1.0 / (1u64 << register_856) as f32 + 1.0 / (1u64 << register_857) as f32 + 1.0 / (1u64 << register_858) as f32 + 1.0 / (1u64 << register_859) as f32;

	let [register_860, register_861, register_862, register_863, register_864] = split_registers::<5>(words[172]);
	raw_estimate += 1.0 / (1u64 << register_860) as f32 + 1.0 / (1u64 << register_861) as f32 + 1.0 / (1u64 << register_862) as f32 + 1.0 / (1u64 << register_863) as f32 + 1.0 / (1u64 << register_864) as f32;

	let [register_865, register_866, register_867, register_868, register_869] = split_registers::<5>(words[173]);
	raw_estimate += 1.0 / (1u64 << register_865) as f32 + 1.0 / (1u64 << register_866) as f32 + 1.0 / (1u64 << register_867) as f32 + 1.0 / (1u64 << register_868) as f32 + 1.0 / (1u64 << register_869) as f32;

	let [register_870, register_871, register_872, register_873, register_874] = split_registers::<5>(words[174]);
	raw_estimate += 1.0 / (1u64 << register_870) as f32 + 1.0 / (1u64 << register_871) as f32 + 1.0 / (1u64 << register_872) as f32 + 1.0 / (1u64 << register_873) as f32 + 1.0 / (1u64 << register_874) as f32;

	let [register_875, register_876, register_877, register_878, register_879] = split_registers::<5>(words[175]);
	raw_estimate += 1.0 / (1u64 << register_875) as f32 + 1.0 / (1u64 << register_876) as f32 + 1.0 / (1u64 << register_877) as f32 + 1.0 / (1u64 << register_878) as f32 + 1.0 / (1u64 << register_879) as f32;

	let [register_880, register_881, register_882, register_883, register_884] = split_registers::<5>(words[176]);
	raw_estimate += 1.0 / (1u64 << register_880) as f32 + 1.0 / (1u64 << register_881) as f32 + 1.0 / (1u64 << register_882) as f32 + 1.0 / (1u64 << register_883) as f32 + 1.0 / (1u64 << register_884) as f32;

	let [register_885, register_886, register_887, register_888, register_889] = split_registers::<5>(words[177]);
	raw_estimate += 1.0 / (1u64 << register_885) as f32 + 1.0 / (1u64 << register_886) as f32 + 1.0 / (1u64 << register_887) as f32 + 1.0 / (1u64 << register_888) as f32 + 1.0 / (1u64 << register_889) as f32;

	let [register_890, register_891, register_892, register_893, register_894] = split_registers::<5>(words[178]);
	raw_estimate += 1.0 / (1u64 << register_890) as f32 + 1.0 / (1u64 << register_891) as f32 + 1.0 / (1u64 << register_892) as f32 + 1.0 / (1u64 << register_893) as f32 + 1.0 / (1u64 << register_894) as f32;

	let [register_895, register_896, register_897, register_898, register_899] = split_registers::<5>(words[179]);
	raw_estimate += 1.0 / (1u64 << register_895) as f32 + 1.0 / (1u64 << register_896) as f32 + 1.0 / (1u64 << register_897) as f32 + 1.0 / (1u64 << register_898) as f32 + 1.0 / (1u64 << register_899) as f32;

	let [register_900, register_901, register_902, register_903, register_904] = split_registers::<5>(words[180]);
	raw_estimate += 1.0 / (1u64 << register_900) as f32 + 1.0 / (1u64 << register_901) as f32 + 1.0 / (1u64 << register_902) as f32 + 1.0 / (1u64 << register_903) as f32 + 1.0 / (1u64 << register_904) as f32;

	let [register_905, register_906, register_907, register_908, register_909] = split_registers::<5>(words[181]);
	raw_estimate += 1.0 / (1u64 << register_905) as f32 + 1.0 / (1u64 << register_906) as f32 + 1.0 / (1u64 << register_907) as f32 + 1.0 / (1u64 << register_908) as f32 + 1.0 / (1u64 << register_909) as f32;

	let [register_910, register_911, register_912, register_913, register_914] = split_registers::<5>(words[182]);
	raw_estimate += 1.0 / (1u64 << register_910) as f32 + 1.0 / (1u64 << register_911) as f32 + 1.0 / (1u64 << register_912) as f32 + 1.0 / (1u64 << register_913) as f32 + 1.0 / (1u64 << register_914) as f32;

	let [register_915, register_916, register_917, register_918, register_919] = split_registers::<5>(words[183]);
	raw_estimate += 1.0 / (1u64 << register_915) as f32 + 1.0 / (1u64 << register_916) as f32 + 1.0 / (1u64 << register_917) as f32 + 1.0 / (1u64 << register_918) as f32 + 1.0 / (1u64 << register_919) as f32;

	let [register_920, register_921, register_922, register_923, register_924] = split_registers::<5>(words[184]);
	raw_estimate += 1.0 / (1u64 << register_920) as f32 + 1.0 / (1u64 << register_921) as f32 + 1.0 / (1u64 << register_922) as f32 + 1.0 / (1u64 << register_923) as f32 + 1.0 / (1u64 << register_924) as f32;

	let [register_925, register_926, register_927, register_928, register_929] = split_registers::<5>(words[185]);
	raw_estimate += 1.0 / (1u64 << register_925) as f32 + 1.0 / (1u64 << register_926) as f32 + 1.0 / (1u64 << register_927) as f32 + 1.0 / (1u64 << register_928) as f32 + 1.0 / (1u64 << register_929) as f32;

	let [register_930, register_931, register_932, register_933, register_934] = split_registers::<5>(words[186]);
	raw_estimate += 1.0 / (1u64 << register_930) as f32 + 1.0 / (1u64 << register_931) as f32 + 1.0 / (1u64 << register_932) as f32 + 1.0 / (1u64 << register_933) as f32 + 1.0 / (1u64 << register_934) as f32;

	let [register_935, register_936, register_937, register_938, register_939] = split_registers::<5>(words[187]);
	raw_estimate += 1.0 / (1u64 << register_935) as f32 + 1.0 / (1u64 << register_936) as f32 + 1.0 / (1u64 << register_937) as f32 + 1.0 / (1u64 << register_938) as f32 + 1.0 / (1u64 << register_939) as f32;

	let [register_940, register_941, register_942, register_943, register_944] = split_registers::<5>(words[188]);
	raw_estimate += 1.0 / (1u64 << register_940) as f32 + 1.0 / (1u64 << register_941) as f32 + 1.0 / (1u64 << register_942) as f32 + 1.0 / (1u64 << register_943) as f32 + 1.0 / (1u64 << register_944) as f32;

	let [register_945, register_946, register_947, register_948, register_949] = split_registers::<5>(words[189]);
	raw_estimate += 1.0 / (1u64 << register_945) as f32 + 1.0 / (1u64 << register_946) as f32 + 1.0 / (1u64 << register_947) as f32 + 1.0 / (1u64 << register_948) as f32 + 1.0 / (1u64 << register_949) as f32;

	let [register_950, register_951, register_952, register_953, register_954] = split_registers::<5>(words[190]);
	raw_estimate += 1.0 / (1u64 << register_950) as f32 + 1.0 / (1u64 << register_951) as f32 + 1.0 / (1u64 << register_952) as f32 + 1.0 / (1u64 << register_953) as f32 + 1.0 / (1u64 << register_954) as f32;

	let [register_955, register_956, register_957, register_958, register_959] = split_registers::<5>(words[191]);
	raw_estimate += 1.0 / (1u64 << register_955) as f32 + 1.0 / (1u64 << register_956) as f32 + 1.0 / (1u64 << register_957) as f32 + 1.0 / (1u64 << register_958) as f32 + 1.0 / (1u64 << register_959) as f32;

	let [register_960, register_961, register_962, register_963, register_964] = split_registers::<5>(words[192]);
	raw_estimate += 1.0 / (1u64 << register_960) as f32 + 1.0 / (1u64 << register_961) as f32 + 1.0 / (1u64 << register_962) as f32 + 1.0 / (1u64 << register_963) as f32 + 1.0 / (1u64 << register_964) as f32;

	let [register_965, register_966, register_967, register_968, register_969] = split_registers::<5>(words[193]);
	raw_estimate += 1.0 / (1u64 << register_965) as f32 + 1.0 / (1u64 << register_966) as f32 + 1.0 / (1u64 << register_967) as f32 + 1.0 / (1u64 << register_968) as f32 + 1.0 / (1u64 << register_969) as f32;

	let [register_970, register_971, register_972, register_973, register_974] = split_registers::<5>(words[194]);
	raw_estimate += 1.0 / (1u64 << register_970) as f32 + 1.0 / (1u64 << register_971) as f32 + 1.0 / (1u64 << register_972) as f32 + 1.0 / (1u64 << register_973) as f32 + 1.0 / (1u64 << register_974) as f32;

	let [register_975, register_976, register_977, register_978, register_979] = split_registers::<5>(words[195]);
	raw_estimate += 1.0 / (1u64 << register_975) as f32 + 1.0 / (1u64 << register_976) as f32 + 1.0 / (1u64 << register_977) as f32 + 1.0 / (1u64 << register_978) as f32 + 1.0 / (1u64 << register_979) as f32;

	let [register_980, register_981, register_982, register_983, register_984] = split_registers::<5>(words[196]);
	raw_estimate += 1.0 / (1u64 << register_980) as f32 + 1.0 / (1u64 << register_981) as f32 + 1.0 / (1u64 << register_982) as f32 + 1.0 / (1u64 << register_983) as f32 + 1.0 / (1u64 << register_984) as f32;

	let [register_985, register_986, register_987, register_988, register_989] = split_registers::<5>(words[197]);
	raw_estimate += 1.0 / (1u64 << register_985) as f32 + 1.0 / (1u64 << register_986) as f32 + 1.0 / (1u64 << register_987) as f32 + 1.0 / (1u64 << register_988) as f32 + 1.0 / (1u64 << register_989) as f32;

	let [register_990, register_991, register_992, register_993, register_994] = split_registers::<5>(words[198]);
	raw_estimate += 1.0 / (1u64 << register_990) as f32 + 1.0 / (1u64 << register_991) as f32 + 1.0 / (1u64 << register_992) as f32 + 1.0 / (1u64 << register_993) as f32 + 1.0 / (1u64 << register_994) as f32;

	let [register_995, register_996, register_997, register_998, register_999] = split_registers::<5>(words[199]);
	raw_estimate += 1.0 / (1u64 << register_995) as f32 + 1.0 / (1u64 << register_996) as f32 + 1.0 / (1u64 << register_997) as f32 + 1.0 / (1u64 << register_998) as f32 + 1.0 / (1u64 << register_999) as f32;

	let [register_1000, register_1001, register_1002, register_1003, register_1004] = split_registers::<5>(words[200]);
	raw_estimate += 1.0 / (1u64 << register_1000) as f32 + 1.0 / (1u64 << register_1001) as f32 + 1.0 / (1u64 << register_1002) as f32 + 1.0 / (1u64 << register_1003) as f32 + 1.0 / (1u64 << register_1004) as f32;

	let [register_1005, register_1006, register_1007, register_1008, register_1009] = split_registers::<5>(words[201]);
	raw_estimate += 1.0 / (1u64 << register_1005) as f32 + 1.0 / (1u64 << register_1006) as f32 + 1.0 / (1u64 << register_1007) as f32 + 1.0 / (1u64 << register_1008) as f32 + 1.0 / (1u64 << register_1009) as f32;

	let [register_1010, register_1011, register_1012, register_1013, register_1014] = split_registers::<5>(words[202]);
	raw_estimate += 1.0 / (1u64 << register_1010) as f32 + 1.0 / (1u64 << register_1011) as f32 + 1.0 / (1u64 << register_1012) as f32 + 1.0 / (1u64 << register_1013) as f32 + 1.0 / (1u64 << register_1014) as f32;

	let [register_1015, register_1016, register_1017, register_1018, register_1019] = split_registers::<5>(words[203]);
	raw_estimate += 1.0 / (1u64 << register_1015) as f32 + 1.0 / (1u64 << register_1016) as f32 + 1.0 / (1u64 << register_1017) as f32 + 1.0 / (1u64 << register_1018) as f32 + 1.0 / (1u64 << register_1019) as f32;

	let [register_1020, register_1021, register_1022, register_1023, register_1024] = split_registers::<5>(words[204]);
	raw_estimate += 1.0 / (1u64 << register_1020) as f32 + 1.0 / (1u64 << register_1021) as f32 + 1.0 / (1u64 << register_1022) as f32 + 1.0 / (1u64 << register_1023) as f32 + 1.0 / (1u64 << register_1024) as f32;

	let [register_1025, register_1026, register_1027, register_1028, register_1029] = split_registers::<5>(words[205]);
	raw_estimate += 1.0 / (1u64 << register_1025) as f32 + 1.0 / (1u64 << register_1026) as f32 + 1.0 / (1u64 << register_1027) as f32 + 1.0 / (1u64 << register_1028) as f32 + 1.0 / (1u64 << register_1029) as f32;

	let [register_1030, register_1031, register_1032, register_1033, register_1034] = split_registers::<5>(words[206]);
	raw_estimate += 1.0 / (1u64 << register_1030) as f32 + 1.0 / (1u64 << register_1031) as f32 + 1.0 / (1u64 << register_1032) as f32 + 1.0 / (1u64 << register_1033) as f32 + 1.0 / (1u64 << register_1034) as f32;

	let [register_1035, register_1036, register_1037, register_1038, register_1039] = split_registers::<5>(words[207]);
	raw_estimate += 1.0 / (1u64 << register_1035) as f32 + 1.0 / (1u64 << register_1036) as f32 + 1.0 / (1u64 << register_1037) as f32 + 1.0 / (1u64 << register_1038) as f32 + 1.0 / (1u64 << register_1039) as f32;

	let [register_1040, register_1041, register_1042, register_1043, register_1044] = split_registers::<5>(words[208]);
	raw_estimate += 1.0 / (1u64 << register_1040) as f32 + 1.0 / (1u64 << register_1041) as f32 + 1.0 / (1u64 << register_1042) as f32 + 1.0 / (1u64 << register_1043) as f32 + 1.0 / (1u64 << register_1044) as f32;

	let [register_1045, register_1046, register_1047, register_1048, register_1049] = split_registers::<5>(words[209]);
	raw_estimate += 1.0 / (1u64 << register_1045) as f32 + 1.0 / (1u64 << register_1046) as f32 + 1.0 / (1u64 << register_1047) as f32 + 1.0 / (1u64 << register_1048) as f32 + 1.0 / (1u64 << register_1049) as f32;

	let [register_1050, register_1051, register_1052, register_1053, register_1054] = split_registers::<5>(words[210]);
	raw_estimate += 1.0 / (1u64 << register_1050) as f32 + 1.0 / (1u64 << register_1051) as f32 + 1.0 / (1u64 << register_1052) as f32 + 1.0 / (1u64 << register_1053) as f32 + 1.0 / (1u64 << register_1054) as f32;

	let [register_1055, register_1056, register_1057, register_1058, register_1059] = split_registers::<5>(words[211]);
	raw_estimate += 1.0 / (1u64 << register_1055) as f32 + 1.0 / (1u64 << register_1056) as f32 + 1.0 / (1u64 << register_1057) as f32 + 1.0 / (1u64 << register_1058) as f32 + 1.0 / (1u64 << register_1059) as f32;

	let [register_1060, register_1061, register_1062, register_1063, register_1064] = split_registers::<5>(words[212]);
	raw_estimate += 1.0 / (1u64 << register_1060) as f32 + 1.0 / (1u64 << register_1061) as f32 + 1.0 / (1u64 << register_1062) as f32 + 1.0 / (1u64 << register_1063) as f32 + 1.0 / (1u64 << register_1064) as f32;

	let [register_1065, register_1066, register_1067, register_1068, register_1069] = split_registers::<5>(words[213]);
	raw_estimate += 1.0 / (1u64 << register_1065) as f32 + 1.0 / (1u64 << register_1066) as f32 + 1.0 / (1u64 << register_1067) as f32 + 1.0 / (1u64 << register_1068) as f32 + 1.0 / (1u64 << register_1069) as f32;

	let [register_1070, register_1071, register_1072, register_1073, register_1074] = split_registers::<5>(words[214]);
	raw_estimate += 1.0 / (1u64 << register_1070) as f32 + 1.0 / (1u64 << register_1071) as f32 + 1.0 / (1u64 << register_1072) as f32 + 1.0 / (1u64 << register_1073) as f32 + 1.0 / (1u64 << register_1074) as f32;

	let [register_1075, register_1076, register_1077, register_1078, register_1079] = split_registers::<5>(words[215]);
	raw_estimate += 1.0 / (1u64 << register_1075) as f32 + 1.0 / (1u64 << register_1076) as f32 + 1.0 / (1u64 << register_1077) as f32 + 1.0 / (1u64 << register_1078) as f32 + 1.0 / (1u64 << register_1079) as f32;

	let [register_1080, register_1081, register_1082, register_1083, register_1084] = split_registers::<5>(words[216]);
	raw_estimate += 1.0 / (1u64 << register_1080) as f32 + 1.0 / (1u64 << register_1081) as f32 + 1.0 / (1u64 << register_1082) as f32 + 1.0 / (1u64 << register_1083) as f32 + 1.0 / (1u64 << register_1084) as f32;

	let [register_1085, register_1086, register_1087, register_1088, register_1089] = split_registers::<5>(words[217]);
	raw_estimate += 1.0 / (1u64 << register_1085) as f32 + 1.0 / (1u64 << register_1086) as f32 + 1.0 / (1u64 << register_1087) as f32 + 1.0 / (1u64 << register_1088) as f32 + 1.0 / (1u64 << register_1089) as f32;

	let [register_1090, register_1091, register_1092, register_1093, register_1094] = split_registers::<5>(words[218]);
	raw_estimate += 1.0 / (1u64 << register_1090) as f32 + 1.0 / (1u64 << register_1091) as f32 + 1.0 / (1u64 << register_1092) as f32 + 1.0 / (1u64 << register_1093) as f32 + 1.0 / (1u64 << register_1094) as f32;

	let [register_1095, register_1096, register_1097, register_1098, register_1099] = split_registers::<5>(words[219]);
	raw_estimate += 1.0 / (1u64 << register_1095) as f32 + 1.0 / (1u64 << register_1096) as f32 + 1.0 / (1u64 << register_1097) as f32 + 1.0 / (1u64 << register_1098) as f32 + 1.0 / (1u64 << register_1099) as f32;

	let [register_1100, register_1101, register_1102, register_1103, register_1104] = split_registers::<5>(words[220]);
	raw_estimate += 1.0 / (1u64 << register_1100) as f32 + 1.0 / (1u64 << register_1101) as f32 + 1.0 / (1u64 << register_1102) as f32 + 1.0 / (1u64 << register_1103) as f32 + 1.0 / (1u64 << register_1104) as f32;

	let [register_1105, register_1106, register_1107, register_1108, register_1109] = split_registers::<5>(words[221]);
	raw_estimate += 1.0 / (1u64 << register_1105) as f32 + 1.0 / (1u64 << register_1106) as f32 + 1.0 / (1u64 << register_1107) as f32 + 1.0 / (1u64 << register_1108) as f32 + 1.0 / (1u64 << register_1109) as f32;

	let [register_1110, register_1111, register_1112, register_1113, register_1114] = split_registers::<5>(words[222]);
	raw_estimate += 1.0 / (1u64 << register_1110) as f32 + 1.0 / (1u64 << register_1111) as f32 + 1.0 / (1u64 << register_1112) as f32 + 1.0 / (1u64 << register_1113) as f32 + 1.0 / (1u64 << register_1114) as f32;

	let [register_1115, register_1116, register_1117, register_1118, register_1119] = split_registers::<5>(words[223]);
	raw_estimate += 1.0 / (1u64 << register_1115) as f32 + 1.0 / (1u64 << register_1116) as f32 + 1.0 / (1u64 << register_1117) as f32 + 1.0 / (1u64 << register_1118) as f32 + 1.0 / (1u64 << register_1119) as f32;

	let [register_1120, register_1121, register_1122, register_1123, register_1124] = split_registers::<5>(words[224]);
	raw_estimate += 1.0 / (1u64 << register_1120) as f32 + 1.0 / (1u64 << register_1121) as f32 + 1.0 / (1u64 << register_1122) as f32 + 1.0 / (1u64 << register_1123) as f32 + 1.0 / (1u64 << register_1124) as f32;

	let [register_1125, register_1126, register_1127, register_1128, register_1129] = split_registers::<5>(words[225]);
	raw_estimate += 1.0 / (1u64 << register_1125) as f32 + 1.0 / (1u64 << register_1126) as f32 + 1.0 / (1u64 << register_1127) as f32 + 1.0 / (1u64 << register_1128) as f32 + 1.0 / (1u64 << register_1129) as f32;

	let [register_1130, register_1131, register_1132, register_1133, register_1134] = split_registers::<5>(words[226]);
	raw_estimate += 1.0 / (1u64 << register_1130) as f32 + 1.0 / (1u64 << register_1131) as f32 + 1.0 / (1u64 << register_1132) as f32 + 1.0 / (1u64 << register_1133) as f32 + 1.0 / (1u64 << register_1134) as f32;

	let [register_1135, register_1136, register_1137, register_1138, register_1139] = split_registers::<5>(words[227]);
	raw_estimate += 1.0 / (1u64 << register_1135) as f32 + 1.0 / (1u64 << register_1136) as f32 + 1.0 / (1u64 << register_1137) as f32 + 1.0 / (1u64 << register_1138) as f32 + 1.0 / (1u64 << register_1139) as f32;

	let [register_1140, register_1141, register_1142, register_1143, register_1144] = split_registers::<5>(words[228]);
	raw_estimate += 1.0 / (1u64 << register_1140) as f32 + 1.0 / (1u64 << register_1141) as f32 + 1.0 / (1u64 << register_1142) as f32 + 1.0 / (1u64 << register_1143) as f32 + 1.0 / (1u64 << register_1144) as f32;

	let [register_1145, register_1146, register_1147, register_1148, register_1149] = split_registers::<5>(words[229]);
	raw_estimate += 1.0 / (1u64 << register_1145) as f32 + 1.0 / (1u64 << register_1146) as f32 + 1.0 / (1u64 << register_1147) as f32 + 1.0 / (1u64 << register_1148) as f32 + 1.0 / (1u64 << register_1149) as f32;

	let [register_1150, register_1151, register_1152, register_1153, register_1154] = split_registers::<5>(words[230]);
	raw_estimate += 1.0 / (1u64 << register_1150) as f32 + 1.0 / (1u64 << register_1151) as f32 + 1.0 / (1u64 << register_1152) as f32 + 1.0 / (1u64 << register_1153) as f32 + 1.0 / (1u64 << register_1154) as f32;

	let [register_1155, register_1156, register_1157, register_1158, register_1159] = split_registers::<5>(words[231]);
	raw_estimate += 1.0 / (1u64 << register_1155) as f32 + 1.0 / (1u64 << register_1156) as f32 + 1.0 / (1u64 << register_1157) as f32 + 1.0 / (1u64 << register_1158) as f32 + 1.0 / (1u64 << register_1159) as f32;

	let [register_1160, register_1161, register_1162, register_1163, register_1164] = split_registers::<5>(words[232]);
	raw_estimate += 1.0 / (1u64 << register_1160) as f32 + 1.0 / (1u64 << register_1161) as f32 + 1.0 / (1u64 << register_1162) as f32 + 1.0 / (1u64 << register_1163) as f32 + 1.0 / (1u64 << register_1164) as f32;

	let [register_1165, register_1166, register_1167, register_1168, register_1169] = split_registers::<5>(words[233]);
	raw_estimate += 1.0 / (1u64 << register_1165) as f32 + 1.0 / (1u64 << register_1166) as f32 + 1.0 / (1u64 << register_1167) as f32 + 1.0 / (1u64 << register_1168) as f32 + 1.0 / (1u64 << register_1169) as f32;

	let [register_1170, register_1171, register_1172, register_1173, register_1174] = split_registers::<5>(words[234]);
	raw_estimate += 1.0 / (1u64 << register_1170) as f32 + 1.0 / (1u64 << register_1171) as f32 + 1.0 / (1u64 << register_1172) as f32 + 1.0 / (1u64 << register_1173) as f32 + 1.0 / (1u64 << register_1174) as f32;

	let [register_1175, register_1176, register_1177, register_1178, register_1179] = split_registers::<5>(words[235]);
	raw_estimate += 1.0 / (1u64 << register_1175) as f32 + 1.0 / (1u64 << register_1176) as f32 + 1.0 / (1u64 << register_1177) as f32 + 1.0 / (1u64 << register_1178) as f32 + 1.0 / (1u64 << register_1179) as f32;

	let [register_1180, register_1181, register_1182, register_1183, register_1184] = split_registers::<5>(words[236]);
	raw_estimate += 1.0 / (1u64 << register_1180) as f32 + 1.0 / (1u64 << register_1181) as f32 + 1.0 / (1u64 << register_1182) as f32 + 1.0 / (1u64 << register_1183) as f32 + 1.0 / (1u64 << register_1184) as f32;

	let [register_1185, register_1186, register_1187, register_1188, register_1189] = split_registers::<5>(words[237]);
	raw_estimate += 1.0 / (1u64 << register_1185) as f32 + 1.0 / (1u64 << register_1186) as f32 + 1.0 / (1u64 << register_1187) as f32 + 1.0 / (1u64 << register_1188) as f32 + 1.0 / (1u64 << register_1189) as f32;

	let [register_1190, register_1191, register_1192, register_1193, register_1194] = split_registers::<5>(words[238]);
	raw_estimate += 1.0 / (1u64 << register_1190) as f32 + 1.0 / (1u64 << register_1191) as f32 + 1.0 / (1u64 << register_1192) as f32 + 1.0 / (1u64 << register_1193) as f32 + 1.0 / (1u64 << register_1194) as f32;

	let [register_1195, register_1196, register_1197, register_1198, register_1199] = split_registers::<5>(words[239]);
	raw_estimate += 1.0 / (1u64 << register_1195) as f32 + 1.0 / (1u64 << register_1196) as f32 + 1.0 / (1u64 << register_1197) as f32 + 1.0 / (1u64 << register_1198) as f32 + 1.0 / (1u64 << register_1199) as f32;

	let [register_1200, register_1201, register_1202, register_1203, register_1204] = split_registers::<5>(words[240]);
	raw_estimate += 1.0 / (1u64 << register_1200) as f32 + 1.0 / (1u64 << register_1201) as f32 + 1.0 / (1u64 << register_1202) as f32 + 1.0 / (1u64 << register_1203) as f32 + 1.0 / (1u64 << register_1204) as f32;

	let [register_1205, register_1206, register_1207, register_1208, register_1209] = split_registers::<5>(words[241]);
	raw_estimate += 1.0 / (1u64 << register_1205) as f32 + 1.0 / (1u64 << register_1206) as f32 + 1.0 / (1u64 << register_1207) as f32 + 1.0 / (1u64 << register_1208) as f32 + 1.0 / (1u64 << register_1209) as f32;

	let [register_1210, register_1211, register_1212, register_1213, register_1214] = split_registers::<5>(words[242]);
	raw_estimate += 1.0 / (1u64 << register_1210) as f32 + 1.0 / (1u64 << register_1211) as f32 + 1.0 / (1u64 << register_1212) as f32 + 1.0 / (1u64 << register_1213) as f32 + 1.0 / (1u64 << register_1214) as f32;

	let [register_1215, register_1216, register_1217, register_1218, register_1219] = split_registers::<5>(words[243]);
	raw_estimate += 1.0 / (1u64 << register_1215) as f32 + 1.0 / (1u64 << register_1216) as f32 + 1.0 / (1u64 << register_1217) as f32 + 1.0 / (1u64 << register_1218) as f32 + 1.0 / (1u64 << register_1219) as f32;

	let [register_1220, register_1221, register_1222, register_1223, register_1224] = split_registers::<5>(words[244]);
	raw_estimate += 1.0 / (1u64 << register_1220) as f32 + 1.0 / (1u64 << register_1221) as f32 + 1.0 / (1u64 << register_1222) as f32 + 1.0 / (1u64 << register_1223) as f32 + 1.0 / (1u64 << register_1224) as f32;

	let [register_1225, register_1226, register_1227, register_1228, register_1229] = split_registers::<5>(words[245]);
	raw_estimate += 1.0 / (1u64 << register_1225) as f32 + 1.0 / (1u64 << register_1226) as f32 + 1.0 / (1u64 << register_1227) as f32 + 1.0 / (1u64 << register_1228) as f32 + 1.0 / (1u64 << register_1229) as f32;

	let [register_1230, register_1231, register_1232, register_1233, register_1234] = split_registers::<5>(words[246]);
	raw_estimate += 1.0 / (1u64 << register_1230) as f32 + 1.0 / (1u64 << register_1231) as f32 + 1.0 / (1u64 << register_1232) as f32 + 1.0 / (1u64 << register_1233) as f32 + 1.0 / (1u64 << register_1234) as f32;

	let [register_1235, register_1236, register_1237, register_1238, register_1239] = split_registers::<5>(words[247]);
	raw_estimate += 1.0 / (1u64 << register_1235) as f32 + 1.0 / (1u64 << register_1236) as f32 + 1.0 / (1u64 << register_1237) as f32 + 1.0 / (1u64 << register_1238) as f32 + 1.0 / (1u64 << register_1239) as f32;

	let [register_1240, register_1241, register_1242, register_1243, register_1244] = split_registers::<5>(words[248]);
	raw_estimate += 1.0 / (1u64 << register_1240) as f32 + 1.0 / (1u64 << register_1241) as f32 + 1.0 / (1u64 << register_1242) as f32 + 1.0 / (1u64 << register_1243) as f32 + 1.0 / (1u64 << register_1244) as f32;

	let [register_1245, register_1246, register_1247, register_1248, register_1249] = split_registers::<5>(words[249]);
	raw_estimate += 1.0 / (1u64 << register_1245) as f32 + 1.0 / (1u64 << register_1246) as f32 + 1.0 / (1u64 << register_1247) as f32 + 1.0 / (1u64 << register_1248) as f32 + 1.0 / (1u64 << register_1249) as f32;

	let [register_1250, register_1251, register_1252, register_1253, register_1254] = split_registers::<5>(words[250]);
	raw_estimate += 1.0 / (1u64 << register_1250) as f32 + 1.0 / (1u64 << register_1251) as f32 + 1.0 / (1u64 << register_1252) as f32 + 1.0 / (1u64 << register_1253) as f32 + 1.0 / (1u64 << register_1254) as f32;

	let [register_1255, register_1256, register_1257, register_1258, register_1259] = split_registers::<5>(words[251]);
	raw_estimate += 1.0 / (1u64 << register_1255) as f32 + 1.0 / (1u64 << register_1256) as f32 + 1.0 / (1u64 << register_1257) as f32 + 1.0 / (1u64 << register_1258) as f32 + 1.0 / (1u64 << register_1259) as f32;

	let [register_1260, register_1261, register_1262, register_1263, register_1264] = split_registers::<5>(words[252]);
	raw_estimate += 1.0 / (1u64 << register_1260) as f32 + 1.0 / (1u64 << register_1261) as f32 + 1.0 / (1u64 << register_1262) as f32 + 1.0 / (1u64 << register_1263) as f32 + 1.0 / (1u64 << register_1264) as f32;

	let [register_1265, register_1266, register_1267, register_1268, register_1269] = split_registers::<5>(words[253]);
	raw_estimate += 1.0 / (1u64 << register_1265) as f32 + 1.0 / (1u64 << register_1266) as f32 + 1.0 / (1u64 << register_1267) as f32 + 1.0 / (1u64 << register_1268) as f32 + 1.0 / (1u64 << register_1269) as f32;

	let [register_1270, register_1271, register_1272, register_1273, register_1274] = split_registers::<5>(words[254]);
	raw_estimate += 1.0 / (1u64 << register_1270) as f32 + 1.0 / (1u64 << register_1271) as f32 + 1.0 / (1u64 << register_1272) as f32 + 1.0 / (1u64 << register_1273) as f32 + 1.0 / (1u64 << register_1274) as f32;

	let [register_1275, register_1276, register_1277, register_1278, register_1279] = split_registers::<5>(words[255]);
	raw_estimate += 1.0 / (1u64 << register_1275) as f32 + 1.0 / (1u64 << register_1276) as f32 + 1.0 / (1u64 << register_1277) as f32 + 1.0 / (1u64 << register_1278) as f32 + 1.0 / (1u64 << register_1279) as f32;

	let [register_1280, register_1281, register_1282, register_1283, register_1284] = split_registers::<5>(words[256]);
	raw_estimate += 1.0 / (1u64 << register_1280) as f32 + 1.0 / (1u64 << register_1281) as f32 + 1.0 / (1u64 << register_1282) as f32 + 1.0 / (1u64 << register_1283) as f32 + 1.0 / (1u64 << register_1284) as f32;

	let [register_1285, register_1286, register_1287, register_1288, register_1289] = split_registers::<5>(words[257]);
	raw_estimate += 1.0 / (1u64 << register_1285) as f32 + 1.0 / (1u64 << register_1286) as f32 + 1.0 / (1u64 << register_1287) as f32 + 1.0 / (1u64 << register_1288) as f32 + 1.0 / (1u64 << register_1289) as f32;

	let [register_1290, register_1291, register_1292, register_1293, register_1294] = split_registers::<5>(words[258]);
	raw_estimate += 1.0 / (1u64 << register_1290) as f32 + 1.0 / (1u64 << register_1291) as f32 + 1.0 / (1u64 << register_1292) as f32 + 1.0 / (1u64 << register_1293) as f32 + 1.0 / (1u64 << register_1294) as f32;

	let [register_1295, register_1296, register_1297, register_1298, register_1299] = split_registers::<5>(words[259]);
	raw_estimate += 1.0 / (1u64 << register_1295) as f32 + 1.0 / (1u64 << register_1296) as f32 + 1.0 / (1u64 << register_1297) as f32 + 1.0 / (1u64 << register_1298) as f32 + 1.0 / (1u64 << register_1299) as f32;

	let [register_1300, register_1301, register_1302, register_1303, register_1304] = split_registers::<5>(words[260]);
	raw_estimate += 1.0 / (1u64 << register_1300) as f32 + 1.0 / (1u64 << register_1301) as f32 + 1.0 / (1u64 << register_1302) as f32 + 1.0 / (1u64 << register_1303) as f32 + 1.0 / (1u64 << register_1304) as f32;

	let [register_1305, register_1306, register_1307, register_1308, register_1309] = split_registers::<5>(words[261]);
	raw_estimate += 1.0 / (1u64 << register_1305) as f32 + 1.0 / (1u64 << register_1306) as f32 + 1.0 / (1u64 << register_1307) as f32 + 1.0 / (1u64 << register_1308) as f32 + 1.0 / (1u64 << register_1309) as f32;

	let [register_1310, register_1311, register_1312, register_1313, register_1314] = split_registers::<5>(words[262]);
	raw_estimate += 1.0 / (1u64 << register_1310) as f32 + 1.0 / (1u64 << register_1311) as f32 + 1.0 / (1u64 << register_1312) as f32 + 1.0 / (1u64 << register_1313) as f32 + 1.0 / (1u64 << register_1314) as f32;

	let [register_1315, register_1316, register_1317, register_1318, register_1319] = split_registers::<5>(words[263]);
	raw_estimate += 1.0 / (1u64 << register_1315) as f32 + 1.0 / (1u64 << register_1316) as f32 + 1.0 / (1u64 << register_1317) as f32 + 1.0 / (1u64 << register_1318) as f32 + 1.0 / (1u64 << register_1319) as f32;

	let [register_1320, register_1321, register_1322, register_1323, register_1324] = split_registers::<5>(words[264]);
	raw_estimate += 1.0 / (1u64 << register_1320) as f32 + 1.0 / (1u64 << register_1321) as f32 + 1.0 / (1u64 << register_1322) as f32 + 1.0 / (1u64 << register_1323) as f32 + 1.0 / (1u64 << register_1324) as f32;

	let [register_1325, register_1326, register_1327, register_1328, register_1329] = split_registers::<5>(words[265]);
	raw_estimate += 1.0 / (1u64 << register_1325) as f32 + 1.0 / (1u64 << register_1326) as f32 + 1.0 / (1u64 << register_1327) as f32 + 1.0 / (1u64 << register_1328) as f32 + 1.0 / (1u64 << register_1329) as f32;

	let [register_1330, register_1331, register_1332, register_1333, register_1334] = split_registers::<5>(words[266]);
	raw_estimate += 1.0 / (1u64 << register_1330) as f32 + 1.0 / (1u64 << register_1331) as f32 + 1.0 / (1u64 << register_1332) as f32 + 1.0 / (1u64 << register_1333) as f32 + 1.0 / (1u64 << register_1334) as f32;

	let [register_1335, register_1336, register_1337, register_1338, register_1339] = split_registers::<5>(words[267]);
	raw_estimate += 1.0 / (1u64 << register_1335) as f32 + 1.0 / (1u64 << register_1336) as f32 + 1.0 / (1u64 << register_1337) as f32 + 1.0 / (1u64 << register_1338) as f32 + 1.0 / (1u64 << register_1339) as f32;

	let [register_1340, register_1341, register_1342, register_1343, register_1344] = split_registers::<5>(words[268]);
	raw_estimate += 1.0 / (1u64 << register_1340) as f32 + 1.0 / (1u64 << register_1341) as f32 + 1.0 / (1u64 << register_1342) as f32 + 1.0 / (1u64 << register_1343) as f32 + 1.0 / (1u64 << register_1344) as f32;

	let [register_1345, register_1346, register_1347, register_1348, register_1349] = split_registers::<5>(words[269]);
	raw_estimate += 1.0 / (1u64 << register_1345) as f32 + 1.0 / (1u64 << register_1346) as f32 + 1.0 / (1u64 << register_1347) as f32 + 1.0 / (1u64 << register_1348) as f32 + 1.0 / (1u64 << register_1349) as f32;

	let [register_1350, register_1351, register_1352, register_1353, register_1354] = split_registers::<5>(words[270]);
	raw_estimate += 1.0 / (1u64 << register_1350) as f32 + 1.0 / (1u64 << register_1351) as f32 + 1.0 / (1u64 << register_1352) as f32 + 1.0 / (1u64 << register_1353) as f32 + 1.0 / (1u64 << register_1354) as f32;

	let [register_1355, register_1356, register_1357, register_1358, register_1359] = split_registers::<5>(words[271]);
	raw_estimate += 1.0 / (1u64 << register_1355) as f32 + 1.0 / (1u64 << register_1356) as f32 + 1.0 / (1u64 << register_1357) as f32 + 1.0 / (1u64 << register_1358) as f32 + 1.0 / (1u64 << register_1359) as f32;

	let [register_1360, register_1361, register_1362, register_1363, register_1364] = split_registers::<5>(words[272]);
	raw_estimate += 1.0 / (1u64 << register_1360) as f32 + 1.0 / (1u64 << register_1361) as f32 + 1.0 / (1u64 << register_1362) as f32 + 1.0 / (1u64 << register_1363) as f32 + 1.0 / (1u64 << register_1364) as f32;

	let [register_1365, register_1366, register_1367, register_1368, register_1369] = split_registers::<5>(words[273]);
	raw_estimate += 1.0 / (1u64 << register_1365) as f32 + 1.0 / (1u64 << register_1366) as f32 + 1.0 / (1u64 << register_1367) as f32 + 1.0 / (1u64 << register_1368) as f32 + 1.0 / (1u64 << register_1369) as f32;

	let [register_1370, register_1371, register_1372, register_1373, register_1374] = split_registers::<5>(words[274]);
	raw_estimate += 1.0 / (1u64 << register_1370) as f32 + 1.0 / (1u64 << register_1371) as f32 + 1.0 / (1u64 << register_1372) as f32 + 1.0 / (1u64 << register_1373) as f32 + 1.0 / (1u64 << register_1374) as f32;

	let [register_1375, register_1376, register_1377, register_1378, register_1379] = split_registers::<5>(words[275]);
	raw_estimate += 1.0 / (1u64 << register_1375) as f32 + 1.0 / (1u64 << register_1376) as f32 + 1.0 / (1u64 << register_1377) as f32 + 1.0 / (1u64 << register_1378) as f32 + 1.0 / (1u64 << register_1379) as f32;

	let [register_1380, register_1381, register_1382, register_1383, register_1384] = split_registers::<5>(words[276]);
	raw_estimate += 1.0 / (1u64 << register_1380) as f32 + 1.0 / (1u64 << register_1381) as f32 + 1.0 / (1u64 << register_1382) as f32 + 1.0 / (1u64 << register_1383) as f32 + 1.0 / (1u64 << register_1384) as f32;

	let [register_1385, register_1386, register_1387, register_1388, register_1389] = split_registers::<5>(words[277]);
	raw_estimate += 1.0 / (1u64 << register_1385) as f32 + 1.0 / (1u64 << register_1386) as f32 + 1.0 / (1u64 << register_1387) as f32 + 1.0 / (1u64 << register_1388) as f32 + 1.0 / (1u64 << register_1389) as f32;

	let [register_1390, register_1391, register_1392, register_1393, register_1394] = split_registers::<5>(words[278]);
	raw_estimate += 1.0 / (1u64 << register_1390) as f32 + 1.0 / (1u64 << register_1391) as f32 + 1.0 / (1u64 << register_1392) as f32 + 1.0 / (1u64 << register_1393) as f32 + 1.0 / (1u64 << register_1394) as f32;

	let [register_1395, register_1396, register_1397, register_1398, register_1399] = split_registers::<5>(words[279]);
	raw_estimate += 1.0 / (1u64 << register_1395) as f32 + 1.0 / (1u64 << register_1396) as f32 + 1.0 / (1u64 << register_1397) as f32 + 1.0 / (1u64 << register_1398) as f32 + 1.0 / (1u64 << register_1399) as f32;

	let [register_1400, register_1401, register_1402, register_1403, register_1404] = split_registers::<5>(words[280]);
	raw_estimate += 1.0 / (1u64 << register_1400) as f32 + 1.0 / (1u64 << register_1401) as f32 + 1.0 / (1u64 << register_1402) as f32 + 1.0 / (1u64 << register_1403) as f32 + 1.0 / (1u64 << register_1404) as f32;

	let [register_1405, register_1406, register_1407, register_1408, register_1409] = split_registers::<5>(words[281]);
	raw_estimate += 1.0 / (1u64 << register_1405) as f32 + 1.0 / (1u64 << register_1406) as f32 + 1.0 / (1u64 << register_1407) as f32 + 1.0 / (1u64 << register_1408) as f32 + 1.0 / (1u64 << register_1409) as f32;

	let [register_1410, register_1411, register_1412, register_1413, register_1414] = split_registers::<5>(words[282]);
	raw_estimate += 1.0 / (1u64 << register_1410) as f32 + 1.0 / (1u64 << register_1411) as f32 + 1.0 / (1u64 << register_1412) as f32 + 1.0 / (1u64 << register_1413) as f32 + 1.0 / (1u64 << register_1414) as f32;

	let [register_1415, register_1416, register_1417, register_1418, register_1419] = split_registers::<5>(words[283]);
	raw_estimate += 1.0 / (1u64 << register_1415) as f32 + 1.0 / (1u64 << register_1416) as f32 + 1.0 / (1u64 << register_1417) as f32 + 1.0 / (1u64 << register_1418) as f32 + 1.0 / (1u64 << register_1419) as f32;

	let [register_1420, register_1421, register_1422, register_1423, register_1424] = split_registers::<5>(words[284]);
	raw_estimate += 1.0 / (1u64 << register_1420) as f32 + 1.0 / (1u64 << register_1421) as f32 + 1.0 / (1u64 << register_1422) as f32 + 1.0 / (1u64 << register_1423) as f32 + 1.0 / (1u64 << register_1424) as f32;

	let [register_1425, register_1426, register_1427, register_1428, register_1429] = split_registers::<5>(words[285]);
	raw_estimate += 1.0 / (1u64 << register_1425) as f32 + 1.0 / (1u64 << register_1426) as f32 + 1.0 / (1u64 << register_1427) as f32 + 1.0 / (1u64 << register_1428) as f32 + 1.0 / (1u64 << register_1429) as f32;

	let [register_1430, register_1431, register_1432, register_1433, register_1434] = split_registers::<5>(words[286]);
	raw_estimate += 1.0 / (1u64 << register_1430) as f32 + 1.0 / (1u64 << register_1431) as f32 + 1.0 / (1u64 << register_1432) as f32 + 1.0 / (1u64 << register_1433) as f32 + 1.0 / (1u64 << register_1434) as f32;

	let [register_1435, register_1436, register_1437, register_1438, register_1439] = split_registers::<5>(words[287]);
	raw_estimate += 1.0 / (1u64 << register_1435) as f32 + 1.0 / (1u64 << register_1436) as f32 + 1.0 / (1u64 << register_1437) as f32 + 1.0 / (1u64 << register_1438) as f32 + 1.0 / (1u64 << register_1439) as f32;

	let [register_1440, register_1441, register_1442, register_1443, register_1444] = split_registers::<5>(words[288]);
	raw_estimate += 1.0 / (1u64 << register_1440) as f32 + 1.0 / (1u64 << register_1441) as f32 + 1.0 / (1u64 << register_1442) as f32 + 1.0 / (1u64 << register_1443) as f32 + 1.0 / (1u64 << register_1444) as f32;

	let [register_1445, register_1446, register_1447, register_1448, register_1449] = split_registers::<5>(words[289]);
	raw_estimate += 1.0 / (1u64 << register_1445) as f32 + 1.0 / (1u64 << register_1446) as f32 + 1.0 / (1u64 << register_1447) as f32 + 1.0 / (1u64 << register_1448) as f32 + 1.0 / (1u64 << register_1449) as f32;

	let [register_1450, register_1451, register_1452, register_1453, register_1454] = split_registers::<5>(words[290]);
	raw_estimate += 1.0 / (1u64 << register_1450) as f32 + 1.0 / (1u64 << register_1451) as f32 + 1.0 / (1u64 << register_1452) as f32 + 1.0 / (1u64 << register_1453) as f32 + 1.0 / (1u64 << register_1454) as f32;

	let [register_1455, register_1456, register_1457, register_1458, register_1459] = split_registers::<5>(words[291]);
	raw_estimate += 1.0 / (1u64 << register_1455) as f32 + 1.0 / (1u64 << register_1456) as f32 + 1.0 / (1u64 << register_1457) as f32 + 1.0 / (1u64 << register_1458) as f32 + 1.0 / (1u64 << register_1459) as f32;

	let [register_1460, register_1461, register_1462, register_1463, register_1464] = split_registers::<5>(words[292]);
	raw_estimate += 1.0 / (1u64 << register_1460) as f32 + 1.0 / (1u64 << register_1461) as f32 + 1.0 / (1u64 << register_1462) as f32 + 1.0 / (1u64 << register_1463) as f32 + 1.0 / (1u64 << register_1464) as f32;

	let [register_1465, register_1466, register_1467, register_1468, register_1469] = split_registers::<5>(words[293]);
	raw_estimate += 1.0 / (1u64 << register_1465) as f32 + 1.0 / (1u64 << register_1466) as f32 + 1.0 / (1u64 << register_1467) as f32 + 1.0 / (1u64 << register_1468) as f32 + 1.0 / (1u64 << register_1469) as f32;

	let [register_1470, register_1471, register_1472, register_1473, register_1474] = split_registers::<5>(words[294]);
	raw_estimate += 1.0 / (1u64 << register_1470) as f32 + 1.0 / (1u64 << register_1471) as f32 + 1.0 / (1u64 << register_1472) as f32 + 1.0 / (1u64 << register_1473) as f32 + 1.0 / (1u64 << register_1474) as f32;

	let [register_1475, register_1476, register_1477, register_1478, register_1479] = split_registers::<5>(words[295]);
	raw_estimate += 1.0 / (1u64 << register_1475) as f32 + 1.0 / (1u64 << register_1476) as f32 + 1.0 / (1u64 << register_1477) as f32 + 1.0 / (1u64 << register_1478) as f32 + 1.0 / (1u64 << register_1479) as f32;

	let [register_1480, register_1481, register_1482, register_1483, register_1484] = split_registers::<5>(words[296]);
	raw_estimate += 1.0 / (1u64 << register_1480) as f32 + 1.0 / (1u64 << register_1481) as f32 + 1.0 / (1u64 << register_1482) as f32 + 1.0 / (1u64 << register_1483) as f32 + 1.0 / (1u64 << register_1484) as f32;

	let [register_1485, register_1486, register_1487, register_1488, register_1489] = split_registers::<5>(words[297]);
	raw_estimate += 1.0 / (1u64 << register_1485) as f32 + 1.0 / (1u64 << register_1486) as f32 + 1.0 / (1u64 << register_1487) as f32 + 1.0 / (1u64 << register_1488) as f32 + 1.0 / (1u64 << register_1489) as f32;

	let [register_1490, register_1491, register_1492, register_1493, register_1494] = split_registers::<5>(words[298]);
	raw_estimate += 1.0 / (1u64 << register_1490) as f32 + 1.0 / (1u64 << register_1491) as f32 + 1.0 / (1u64 << register_1492) as f32 + 1.0 / (1u64 << register_1493) as f32 + 1.0 / (1u64 << register_1494) as f32;

	let [register_1495, register_1496, register_1497, register_1498, register_1499] = split_registers::<5>(words[299]);
	raw_estimate += 1.0 / (1u64 << register_1495) as f32 + 1.0 / (1u64 << register_1496) as f32 + 1.0 / (1u64 << register_1497) as f32 + 1.0 / (1u64 << register_1498) as f32 + 1.0 / (1u64 << register_1499) as f32;

	let [register_1500, register_1501, register_1502, register_1503, register_1504] = split_registers::<5>(words[300]);
	raw_estimate += 1.0 / (1u64 << register_1500) as f32 + 1.0 / (1u64 << register_1501) as f32 + 1.0 / (1u64 << register_1502) as f32 + 1.0 / (1u64 << register_1503) as f32 + 1.0 / (1u64 << register_1504) as f32;

	let [register_1505, register_1506, register_1507, register_1508, register_1509] = split_registers::<5>(words[301]);
	raw_estimate += 1.0 / (1u64 << register_1505) as f32 + 1.0 / (1u64 << register_1506) as f32 + 1.0 / (1u64 << register_1507) as f32 + 1.0 / (1u64 << register_1508) as f32 + 1.0 / (1u64 << register_1509) as f32;

	let [register_1510, register_1511, register_1512, register_1513, register_1514] = split_registers::<5>(words[302]);
	raw_estimate += 1.0 / (1u64 << register_1510) as f32 + 1.0 / (1u64 << register_1511) as f32 + 1.0 / (1u64 << register_1512) as f32 + 1.0 / (1u64 << register_1513) as f32 + 1.0 / (1u64 << register_1514) as f32;

	let [register_1515, register_1516, register_1517, register_1518, register_1519] = split_registers::<5>(words[303]);
	raw_estimate += 1.0 / (1u64 << register_1515) as f32 + 1.0 / (1u64 << register_1516) as f32 + 1.0 / (1u64 << register_1517) as f32 + 1.0 / (1u64 << register_1518) as f32 + 1.0 / (1u64 << register_1519) as f32;

	let [register_1520, register_1521, register_1522, register_1523, register_1524] = split_registers::<5>(words[304]);
	raw_estimate += 1.0 / (1u64 << register_1520) as f32 + 1.0 / (1u64 << register_1521) as f32 + 1.0 / (1u64 << register_1522) as f32 + 1.0 / (1u64 << register_1523) as f32 + 1.0 / (1u64 << register_1524) as f32;

	let [register_1525, register_1526, register_1527, register_1528, register_1529] = split_registers::<5>(words[305]);
	raw_estimate += 1.0 / (1u64 << register_1525) as f32 + 1.0 / (1u64 << register_1526) as f32 + 1.0 / (1u64 << register_1527) as f32 + 1.0 / (1u64 << register_1528) as f32 + 1.0 / (1u64 << register_1529) as f32;

	let [register_1530, register_1531, register_1532, register_1533, register_1534] = split_registers::<5>(words[306]);
	raw_estimate += 1.0 / (1u64 << register_1530) as f32 + 1.0 / (1u64 << register_1531) as f32 + 1.0 / (1u64 << register_1532) as f32 + 1.0 / (1u64 << register_1533) as f32 + 1.0 / (1u64 << register_1534) as f32;

	let [register_1535, register_1536, register_1537, register_1538, register_1539] = split_registers::<5>(words[307]);
	raw_estimate += 1.0 / (1u64 << register_1535) as f32 + 1.0 / (1u64 << register_1536) as f32 + 1.0 / (1u64 << register_1537) as f32 + 1.0 / (1u64 << register_1538) as f32 + 1.0 / (1u64 << register_1539) as f32;

	let [register_1540, register_1541, register_1542, register_1543, register_1544] = split_registers::<5>(words[308]);
	raw_estimate += 1.0 / (1u64 << register_1540) as f32 + 1.0 / (1u64 << register_1541) as f32 + 1.0 / (1u64 << register_1542) as f32 + 1.0 / (1u64 << register_1543) as f32 + 1.0 / (1u64 << register_1544) as f32;

	let [register_1545, register_1546, register_1547, register_1548, register_1549] = split_registers::<5>(words[309]);
	raw_estimate += 1.0 / (1u64 << register_1545) as f32 + 1.0 / (1u64 << register_1546) as f32 + 1.0 / (1u64 << register_1547) as f32 + 1.0 / (1u64 << register_1548) as f32 + 1.0 / (1u64 << register_1549) as f32;

	let [register_1550, register_1551, register_1552, register_1553, register_1554] = split_registers::<5>(words[310]);
	raw_estimate += 1.0 / (1u64 << register_1550) as f32 + 1.0 / (1u64 << register_1551) as f32 + 1.0 / (1u64 << register_1552) as f32 + 1.0 / (1u64 << register_1553) as f32 + 1.0 / (1u64 << register_1554) as f32;

	let [register_1555, register_1556, register_1557, register_1558, register_1559] = split_registers::<5>(words[311]);
	raw_estimate += 1.0 / (1u64 << register_1555) as f32 + 1.0 / (1u64 << register_1556) as f32 + 1.0 / (1u64 << register_1557) as f32 + 1.0 / (1u64 << register_1558) as f32 + 1.0 / (1u64 << register_1559) as f32;

	let [register_1560, register_1561, register_1562, register_1563, register_1564] = split_registers::<5>(words[312]);
	raw_estimate += 1.0 / (1u64 << register_1560) as f32 + 1.0 / (1u64 << register_1561) as f32 + 1.0 / (1u64 << register_1562) as f32 + 1.0 / (1u64 << register_1563) as f32 + 1.0 / (1u64 << register_1564) as f32;

	let [register_1565, register_1566, register_1567, register_1568, register_1569] = split_registers::<5>(words[313]);
	raw_estimate += 1.0 / (1u64 << register_1565) as f32 + 1.0 / (1u64 << register_1566) as f32 + 1.0 / (1u64 << register_1567) as f32 + 1.0 / (1u64 << register_1568) as f32 + 1.0 / (1u64 << register_1569) as f32;

	let [register_1570, register_1571, register_1572, register_1573, register_1574] = split_registers::<5>(words[314]);
	raw_estimate += 1.0 / (1u64 << register_1570) as f32 + 1.0 / (1u64 << register_1571) as f32 + 1.0 / (1u64 << register_1572) as f32 + 1.0 / (1u64 << register_1573) as f32 + 1.0 / (1u64 << register_1574) as f32;

	let [register_1575, register_1576, register_1577, register_1578, register_1579] = split_registers::<5>(words[315]);
	raw_estimate += 1.0 / (1u64 << register_1575) as f32 + 1.0 / (1u64 << register_1576) as f32 + 1.0 / (1u64 << register_1577) as f32 + 1.0 / (1u64 << register_1578) as f32 + 1.0 / (1u64 << register_1579) as f32;

	let [register_1580, register_1581, register_1582, register_1583, register_1584] = split_registers::<5>(words[316]);
	raw_estimate += 1.0 / (1u64 << register_1580) as f32 + 1.0 / (1u64 << register_1581) as f32 + 1.0 / (1u64 << register_1582) as f32 + 1.0 / (1u64 << register_1583) as f32 + 1.0 / (1u64 << register_1584) as f32;

	let [register_1585, register_1586, register_1587, register_1588, register_1589] = split_registers::<5>(words[317]);
	raw_estimate += 1.0 / (1u64 << register_1585) as f32 + 1.0 / (1u64 << register_1586) as f32 + 1.0 / (1u64 << register_1587) as f32 + 1.0 / (1u64 << register_1588) as f32 + 1.0 / (1u64 << register_1589) as f32;

	let [register_1590, register_1591, register_1592, register_1593, register_1594] = split_registers::<5>(words[318]);
	raw_estimate += 1.0 / (1u64 << register_1590) as f32 + 1.0 / (1u64 << register_1591) as f32 + 1.0 / (1u64 << register_1592) as f32 + 1.0 / (1u64 << register_1593) as f32 + 1.0 / (1u64 << register_1594) as f32;

	let [register_1595, register_1596, register_1597, register_1598, register_1599] = split_registers::<5>(words[319]);
	raw_estimate += 1.0 / (1u64 << register_1595) as f32 + 1.0 / (1u64 << register_1596) as f32 + 1.0 / (1u64 << register_1597) as f32 + 1.0 / (1u64 << register_1598) as f32 + 1.0 / (1u64 << register_1599) as f32;

	let [register_1600, register_1601, register_1602, register_1603, register_1604] = split_registers::<5>(words[320]);
	raw_estimate += 1.0 / (1u64 << register_1600) as f32 + 1.0 / (1u64 << register_1601) as f32 + 1.0 / (1u64 << register_1602) as f32 + 1.0 / (1u64 << register_1603) as f32 + 1.0 / (1u64 << register_1604) as f32;

	let [register_1605, register_1606, register_1607, register_1608, register_1609] = split_registers::<5>(words[321]);
	raw_estimate += 1.0 / (1u64 << register_1605) as f32 + 1.0 / (1u64 << register_1606) as f32 + 1.0 / (1u64 << register_1607) as f32 + 1.0 / (1u64 << register_1608) as f32 + 1.0 / (1u64 << register_1609) as f32;

	let [register_1610, register_1611, register_1612, register_1613, register_1614] = split_registers::<5>(words[322]);
	raw_estimate += 1.0 / (1u64 << register_1610) as f32 + 1.0 / (1u64 << register_1611) as f32 + 1.0 / (1u64 << register_1612) as f32 + 1.0 / (1u64 << register_1613) as f32 + 1.0 / (1u64 << register_1614) as f32;

	let [register_1615, register_1616, register_1617, register_1618, register_1619] = split_registers::<5>(words[323]);
	raw_estimate += 1.0 / (1u64 << register_1615) as f32 + 1.0 / (1u64 << register_1616) as f32 + 1.0 / (1u64 << register_1617) as f32 + 1.0 / (1u64 << register_1618) as f32 + 1.0 / (1u64 << register_1619) as f32;

	let [register_1620, register_1621, register_1622, register_1623, register_1624] = split_registers::<5>(words[324]);
	raw_estimate += 1.0 / (1u64 << register_1620) as f32 + 1.0 / (1u64 << register_1621) as f32 + 1.0 / (1u64 << register_1622) as f32 + 1.0 / (1u64 << register_1623) as f32 + 1.0 / (1u64 << register_1624) as f32;

	let [register_1625, register_1626, register_1627, register_1628, register_1629] = split_registers::<5>(words[325]);
	raw_estimate += 1.0 / (1u64 << register_1625) as f32 + 1.0 / (1u64 << register_1626) as f32 + 1.0 / (1u64 << register_1627) as f32 + 1.0 / (1u64 << register_1628) as f32 + 1.0 / (1u64 << register_1629) as f32;

	let [register_1630, register_1631, register_1632, register_1633, register_1634] = split_registers::<5>(words[326]);
	raw_estimate += 1.0 / (1u64 << register_1630) as f32 + 1.0 / (1u64 << register_1631) as f32 + 1.0 / (1u64 << register_1632) as f32 + 1.0 / (1u64 << register_1633) as f32 + 1.0 / (1u64 << register_1634) as f32;

	let [register_1635, register_1636, register_1637, register_1638, register_1639] = split_registers::<5>(words[327]);
	raw_estimate += 1.0 / (1u64 << register_1635) as f32 + 1.0 / (1u64 << register_1636) as f32 + 1.0 / (1u64 << register_1637) as f32 + 1.0 / (1u64 << register_1638) as f32 + 1.0 / (1u64 << register_1639) as f32;

	let [register_1640, register_1641, register_1642, register_1643, register_1644] = split_registers::<5>(words[328]);
	raw_estimate += 1.0 / (1u64 << register_1640) as f32 + 1.0 / (1u64 << register_1641) as f32 + 1.0 / (1u64 << register_1642) as f32 + 1.0 / (1u64 << register_1643) as f32 + 1.0 / (1u64 << register_1644) as f32;

	let [register_1645, register_1646, register_1647, register_1648, register_1649] = split_registers::<5>(words[329]);
	raw_estimate += 1.0 / (1u64 << register_1645) as f32 + 1.0 / (1u64 << register_1646) as f32 + 1.0 / (1u64 << register_1647) as f32 + 1.0 / (1u64 << register_1648) as f32 + 1.0 / (1u64 << register_1649) as f32;

	let [register_1650, register_1651, register_1652, register_1653, register_1654] = split_registers::<5>(words[330]);
	raw_estimate += 1.0 / (1u64 << register_1650) as f32 + 1.0 / (1u64 << register_1651) as f32 + 1.0 / (1u64 << register_1652) as f32 + 1.0 / (1u64 << register_1653) as f32 + 1.0 / (1u64 << register_1654) as f32;

	let [register_1655, register_1656, register_1657, register_1658, register_1659] = split_registers::<5>(words[331]);
	raw_estimate += 1.0 / (1u64 << register_1655) as f32 + 1.0 / (1u64 << register_1656) as f32 + 1.0 / (1u64 << register_1657) as f32 + 1.0 / (1u64 << register_1658) as f32 + 1.0 / (1u64 << register_1659) as f32;

	let [register_1660, register_1661, register_1662, register_1663, register_1664] = split_registers::<5>(words[332]);
	raw_estimate += 1.0 / (1u64 << register_1660) as f32 + 1.0 / (1u64 << register_1661) as f32 + 1.0 / (1u64 << register_1662) as f32 + 1.0 / (1u64 << register_1663) as f32 + 1.0 / (1u64 << register_1664) as f32;

	let [register_1665, register_1666, register_1667, register_1668, register_1669] = split_registers::<5>(words[333]);
	raw_estimate += 1.0 / (1u64 << register_1665) as f32 + 1.0 / (1u64 << register_1666) as f32 + 1.0 / (1u64 << register_1667) as f32 + 1.0 / (1u64 << register_1668) as f32 + 1.0 / (1u64 << register_1669) as f32;

	let [register_1670, register_1671, register_1672, register_1673, register_1674] = split_registers::<5>(words[334]);
	raw_estimate += 1.0 / (1u64 << register_1670) as f32 + 1.0 / (1u64 << register_1671) as f32 + 1.0 / (1u64 << register_1672) as f32 + 1.0 / (1u64 << register_1673) as f32 + 1.0 / (1u64 << register_1674) as f32;

	let [register_1675, register_1676, register_1677, register_1678, register_1679] = split_registers::<5>(words[335]);
	raw_estimate += 1.0 / (1u64 << register_1675) as f32 + 1.0 / (1u64 << register_1676) as f32 + 1.0 / (1u64 << register_1677) as f32 + 1.0 / (1u64 << register_1678) as f32 + 1.0 / (1u64 << register_1679) as f32;

	let [register_1680, register_1681, register_1682, register_1683, register_1684] = split_registers::<5>(words[336]);
	raw_estimate += 1.0 / (1u64 << register_1680) as f32 + 1.0 / (1u64 << register_1681) as f32 + 1.0 / (1u64 << register_1682) as f32 + 1.0 / (1u64 << register_1683) as f32 + 1.0 / (1u64 << register_1684) as f32;

	let [register_1685, register_1686, register_1687, register_1688, register_1689] = split_registers::<5>(words[337]);
	raw_estimate += 1.0 / (1u64 << register_1685) as f32 + 1.0 / (1u64 << register_1686) as f32 + 1.0 / (1u64 << register_1687) as f32 + 1.0 / (1u64 << register_1688) as f32 + 1.0 / (1u64 << register_1689) as f32;

	let [register_1690, register_1691, register_1692, register_1693, register_1694] = split_registers::<5>(words[338]);
	raw_estimate += 1.0 / (1u64 << register_1690) as f32 + 1.0 / (1u64 << register_1691) as f32 + 1.0 / (1u64 << register_1692) as f32 + 1.0 / (1u64 << register_1693) as f32 + 1.0 / (1u64 << register_1694) as f32;

	let [register_1695, register_1696, register_1697, register_1698, register_1699] = split_registers::<5>(words[339]);
	raw_estimate += 1.0 / (1u64 << register_1695) as f32 + 1.0 / (1u64 << register_1696) as f32 + 1.0 / (1u64 << register_1697) as f32 + 1.0 / (1u64 << register_1698) as f32 + 1.0 / (1u64 << register_1699) as f32;

	let [register_1700, register_1701, register_1702, register_1703, register_1704] = split_registers::<5>(words[340]);
	raw_estimate += 1.0 / (1u64 << register_1700) as f32 + 1.0 / (1u64 << register_1701) as f32 + 1.0 / (1u64 << register_1702) as f32 + 1.0 / (1u64 << register_1703) as f32 + 1.0 / (1u64 << register_1704) as f32;

	let [register_1705, register_1706, register_1707, register_1708, register_1709] = split_registers::<5>(words[341]);
	raw_estimate += 1.0 / (1u64 << register_1705) as f32 + 1.0 / (1u64 << register_1706) as f32 + 1.0 / (1u64 << register_1707) as f32 + 1.0 / (1u64 << register_1708) as f32 + 1.0 / (1u64 << register_1709) as f32;

	let [register_1710, register_1711, register_1712, register_1713, register_1714] = split_registers::<5>(words[342]);
	raw_estimate += 1.0 / (1u64 << register_1710) as f32 + 1.0 / (1u64 << register_1711) as f32 + 1.0 / (1u64 << register_1712) as f32 + 1.0 / (1u64 << register_1713) as f32 + 1.0 / (1u64 << register_1714) as f32;

	let [register_1715, register_1716, register_1717, register_1718, register_1719] = split_registers::<5>(words[343]);
	raw_estimate += 1.0 / (1u64 << register_1715) as f32 + 1.0 / (1u64 << register_1716) as f32 + 1.0 / (1u64 << register_1717) as f32 + 1.0 / (1u64 << register_1718) as f32 + 1.0 / (1u64 << register_1719) as f32;

	let [register_1720, register_1721, register_1722, register_1723, register_1724] = split_registers::<5>(words[344]);
	raw_estimate += 1.0 / (1u64 << register_1720) as f32 + 1.0 / (1u64 << register_1721) as f32 + 1.0 / (1u64 << register_1722) as f32 + 1.0 / (1u64 << register_1723) as f32 + 1.0 / (1u64 << register_1724) as f32;

	let [register_1725, register_1726, register_1727, register_1728, register_1729] = split_registers::<5>(words[345]);
	raw_estimate += 1.0 / (1u64 << register_1725) as f32 + 1.0 / (1u64 << register_1726) as f32 + 1.0 / (1u64 << register_1727) as f32 + 1.0 / (1u64 << register_1728) as f32 + 1.0 / (1u64 << register_1729) as f32;

	let [register_1730, register_1731, register_1732, register_1733, register_1734] = split_registers::<5>(words[346]);
	raw_estimate += 1.0 / (1u64 << register_1730) as f32 + 1.0 / (1u64 << register_1731) as f32 + 1.0 / (1u64 << register_1732) as f32 + 1.0 / (1u64 << register_1733) as f32 + 1.0 / (1u64 << register_1734) as f32;

	let [register_1735, register_1736, register_1737, register_1738, register_1739] = split_registers::<5>(words[347]);
	raw_estimate += 1.0 / (1u64 << register_1735) as f32 + 1.0 / (1u64 << register_1736) as f32 + 1.0 / (1u64 << register_1737) as f32 + 1.0 / (1u64 << register_1738) as f32 + 1.0 / (1u64 << register_1739) as f32;

	let [register_1740, register_1741, register_1742, register_1743, register_1744] = split_registers::<5>(words[348]);
	raw_estimate += 1.0 / (1u64 << register_1740) as f32 + 1.0 / (1u64 << register_1741) as f32 + 1.0 / (1u64 << register_1742) as f32 + 1.0 / (1u64 << register_1743) as f32 + 1.0 / (1u64 << register_1744) as f32;

	let [register_1745, register_1746, register_1747, register_1748, register_1749] = split_registers::<5>(words[349]);
	raw_estimate += 1.0 / (1u64 << register_1745) as f32 + 1.0 / (1u64 << register_1746) as f32 + 1.0 / (1u64 << register_1747) as f32 + 1.0 / (1u64 << register_1748) as f32 + 1.0 / (1u64 << register_1749) as f32;

	let [register_1750, register_1751, register_1752, register_1753, register_1754] = split_registers::<5>(words[350]);
	raw_estimate += 1.0 / (1u64 << register_1750) as f32 + 1.0 / (1u64 << register_1751) as f32 + 1.0 / (1u64 << register_1752) as f32 + 1.0 / (1u64 << register_1753) as f32 + 1.0 / (1u64 << register_1754) as f32;

	let [register_1755, register_1756, register_1757, register_1758, register_1759] = split_registers::<5>(words[351]);
	raw_estimate += 1.0 / (1u64 << register_1755) as f32 + 1.0 / (1u64 << register_1756) as f32 + 1.0 / (1u64 << register_1757) as f32 + 1.0 / (1u64 << register_1758) as f32 + 1.0 / (1u64 << register_1759) as f32;

	let [register_1760, register_1761, register_1762, register_1763, register_1764] = split_registers::<5>(words[352]);
	raw_estimate += 1.0 / (1u64 << register_1760) as f32 + 1.0 / (1u64 << register_1761) as f32 + 1.0 / (1u64 << register_1762) as f32 + 1.0 / (1u64 << register_1763) as f32 + 1.0 / (1u64 << register_1764) as f32;

	let [register_1765, register_1766, register_1767, register_1768, register_1769] = split_registers::<5>(words[353]);
	raw_estimate += 1.0 / (1u64 << register_1765) as f32 + 1.0 / (1u64 << register_1766) as f32 + 1.0 / (1u64 << register_1767) as f32 + 1.0 / (1u64 << register_1768) as f32 + 1.0 / (1u64 << register_1769) as f32;

	let [register_1770, register_1771, register_1772, register_1773, register_1774] = split_registers::<5>(words[354]);
	raw_estimate += 1.0 / (1u64 << register_1770) as f32 + 1.0 / (1u64 << register_1771) as f32 + 1.0 / (1u64 << register_1772) as f32 + 1.0 / (1u64 << register_1773) as f32 + 1.0 / (1u64 << register_1774) as f32;

	let [register_1775, register_1776, register_1777, register_1778, register_1779] = split_registers::<5>(words[355]);
	raw_estimate += 1.0 / (1u64 << register_1775) as f32 + 1.0 / (1u64 << register_1776) as f32 + 1.0 / (1u64 << register_1777) as f32 + 1.0 / (1u64 << register_1778) as f32 + 1.0 / (1u64 << register_1779) as f32;

	let [register_1780, register_1781, register_1782, register_1783, register_1784] = split_registers::<5>(words[356]);
	raw_estimate += 1.0 / (1u64 << register_1780) as f32 + 1.0 / (1u64 << register_1781) as f32 + 1.0 / (1u64 << register_1782) as f32 + 1.0 / (1u64 << register_1783) as f32 + 1.0 / (1u64 << register_1784) as f32;

	let [register_1785, register_1786, register_1787, register_1788, register_1789] = split_registers::<5>(words[357]);
	raw_estimate += 1.0 / (1u64 << register_1785) as f32 + 1.0 / (1u64 << register_1786) as f32 + 1.0 / (1u64 << register_1787) as f32 + 1.0 / (1u64 << register_1788) as f32 + 1.0 / (1u64 << register_1789) as f32;

	let [register_1790, register_1791, register_1792, register_1793, register_1794] = split_registers::<5>(words[358]);
	raw_estimate += 1.0 / (1u64 << register_1790) as f32 + 1.0 / (1u64 << register_1791) as f32 + 1.0 / (1u64 << register_1792) as f32 + 1.0 / (1u64 << register_1793) as f32 + 1.0 / (1u64 << register_1794) as f32;

	let [register_1795, register_1796, register_1797, register_1798, register_1799] = split_registers::<5>(words[359]);
	raw_estimate += 1.0 / (1u64 << register_1795) as f32 + 1.0 / (1u64 << register_1796) as f32 + 1.0 / (1u64 << register_1797) as f32 + 1.0 / (1u64 << register_1798) as f32 + 1.0 / (1u64 << register_1799) as f32;

	let [register_1800, register_1801, register_1802, register_1803, register_1804] = split_registers::<5>(words[360]);
	raw_estimate += 1.0 / (1u64 << register_1800) as f32 + 1.0 / (1u64 << register_1801) as f32 + 1.0 / (1u64 << register_1802) as f32 + 1.0 / (1u64 << register_1803) as f32 + 1.0 / (1u64 << register_1804) as f32;

	let [register_1805, register_1806, register_1807, register_1808, register_1809] = split_registers::<5>(words[361]);
	raw_estimate += 1.0 / (1u64 << register_1805) as f32 + 1.0 / (1u64 << register_1806) as f32 + 1.0 / (1u64 << register_1807) as f32 + 1.0 / (1u64 << register_1808) as f32 + 1.0 / (1u64 << register_1809) as f32;

	let [register_1810, register_1811, register_1812, register_1813, register_1814] = split_registers::<5>(words[362]);
	raw_estimate += 1.0 / (1u64 << register_1810) as f32 + 1.0 / (1u64 << register_1811) as f32 + 1.0 / (1u64 << register_1812) as f32 + 1.0 / (1u64 << register_1813) as f32 + 1.0 / (1u64 << register_1814) as f32;

	let [register_1815, register_1816, register_1817, register_1818, register_1819] = split_registers::<5>(words[363]);
	raw_estimate += 1.0 / (1u64 << register_1815) as f32 + 1.0 / (1u64 << register_1816) as f32 + 1.0 / (1u64 << register_1817) as f32 + 1.0 / (1u64 << register_1818) as f32 + 1.0 / (1u64 << register_1819) as f32;

	let [register_1820, register_1821, register_1822, register_1823, register_1824] = split_registers::<5>(words[364]);
	raw_estimate += 1.0 / (1u64 << register_1820) as f32 + 1.0 / (1u64 << register_1821) as f32 + 1.0 / (1u64 << register_1822) as f32 + 1.0 / (1u64 << register_1823) as f32 + 1.0 / (1u64 << register_1824) as f32;

	let [register_1825, register_1826, register_1827, register_1828, register_1829] = split_registers::<5>(words[365]);
	raw_estimate += 1.0 / (1u64 << register_1825) as f32 + 1.0 / (1u64 << register_1826) as f32 + 1.0 / (1u64 << register_1827) as f32 + 1.0 / (1u64 << register_1828) as f32 + 1.0 / (1u64 << register_1829) as f32;

	let [register_1830, register_1831, register_1832, register_1833, register_1834] = split_registers::<5>(words[366]);
	raw_estimate += 1.0 / (1u64 << register_1830) as f32 + 1.0 / (1u64 << register_1831) as f32 + 1.0 / (1u64 << register_1832) as f32 + 1.0 / (1u64 << register_1833) as f32 + 1.0 / (1u64 << register_1834) as f32;

	let [register_1835, register_1836, register_1837, register_1838, register_1839] = split_registers::<5>(words[367]);
	raw_estimate += 1.0 / (1u64 << register_1835) as f32 + 1.0 / (1u64 << register_1836) as f32 + 1.0 / (1u64 << register_1837) as f32 + 1.0 / (1u64 << register_1838) as f32 + 1.0 / (1u64 << register_1839) as f32;

	let [register_1840, register_1841, register_1842, register_1843, register_1844] = split_registers::<5>(words[368]);
	raw_estimate += 1.0 / (1u64 << register_1840) as f32 + 1.0 / (1u64 << register_1841) as f32 + 1.0 / (1u64 << register_1842) as f32 + 1.0 / (1u64 << register_1843) as f32 + 1.0 / (1u64 << register_1844) as f32;

	let [register_1845, register_1846, register_1847, register_1848, register_1849] = split_registers::<5>(words[369]);
	raw_estimate += 1.0 / (1u64 << register_1845) as f32 + 1.0 / (1u64 << register_1846) as f32 + 1.0 / (1u64 << register_1847) as f32 + 1.0 / (1u64 << register_1848) as f32 + 1.0 / (1u64 << register_1849) as f32;

	let [register_1850, register_1851, register_1852, register_1853, register_1854] = split_registers::<5>(words[370]);
	raw_estimate += 1.0 / (1u64 << register_1850) as f32 + 1.0 / (1u64 << register_1851) as f32 + 1.0 / (1u64 << register_1852) as f32 + 1.0 / (1u64 << register_1853) as f32 + 1.0 / (1u64 << register_1854) as f32;

	let [register_1855, register_1856, register_1857, register_1858, register_1859] = split_registers::<5>(words[371]);
	raw_estimate += 1.0 / (1u64 << register_1855) as f32 + 1.0 / (1u64 << register_1856) as f32 + 1.0 / (1u64 << register_1857) as f32 + 1.0 / (1u64 << register_1858) as f32 + 1.0 / (1u64 << register_1859) as f32;

	let [register_1860, register_1861, register_1862, register_1863, register_1864] = split_registers::<5>(words[372]);
	raw_estimate += 1.0 / (1u64 << register_1860) as f32 + 1.0 / (1u64 << register_1861) as f32 + 1.0 / (1u64 << register_1862) as f32 + 1.0 / (1u64 << register_1863) as f32 + 1.0 / (1u64 << register_1864) as f32;

	let [register_1865, register_1866, register_1867, register_1868, register_1869] = split_registers::<5>(words[373]);
	raw_estimate += 1.0 / (1u64 << register_1865) as f32 + 1.0 / (1u64 << register_1866) as f32 + 1.0 / (1u64 << register_1867) as f32 + 1.0 / (1u64 << register_1868) as f32 + 1.0 / (1u64 << register_1869) as f32;

	let [register_1870, register_1871, register_1872, register_1873, register_1874] = split_registers::<5>(words[374]);
	raw_estimate += 1.0 / (1u64 << register_1870) as f32 + 1.0 / (1u64 << register_1871) as f32 + 1.0 / (1u64 << register_1872) as f32 + 1.0 / (1u64 << register_1873) as f32 + 1.0 / (1u64 << register_1874) as f32;

	let [register_1875, register_1876, register_1877, register_1878, register_1879] = split_registers::<5>(words[375]);
	raw_estimate += 1.0 / (1u64 << register_1875) as f32 + 1.0 / (1u64 << register_1876) as f32 + 1.0 / (1u64 << register_1877) as f32 + 1.0 / (1u64 << register_1878) as f32 + 1.0 / (1u64 << register_1879) as f32;

	let [register_1880, register_1881, register_1882, register_1883, register_1884] = split_registers::<5>(words[376]);
	raw_estimate += 1.0 / (1u64 << register_1880) as f32 + 1.0 / (1u64 << register_1881) as f32 + 1.0 / (1u64 << register_1882) as f32 + 1.0 / (1u64 << register_1883) as f32 + 1.0 / (1u64 << register_1884) as f32;

	let [register_1885, register_1886, register_1887, register_1888, register_1889] = split_registers::<5>(words[377]);
	raw_estimate += 1.0 / (1u64 << register_1885) as f32 + 1.0 / (1u64 << register_1886) as f32 + 1.0 / (1u64 << register_1887) as f32 + 1.0 / (1u64 << register_1888) as f32 + 1.0 / (1u64 << register_1889) as f32;

	let [register_1890, register_1891, register_1892, register_1893, register_1894] = split_registers::<5>(words[378]);
	raw_estimate += 1.0 / (1u64 << register_1890) as f32 + 1.0 / (1u64 << register_1891) as f32 + 1.0 / (1u64 << register_1892) as f32 + 1.0 / (1u64 << register_1893) as f32 + 1.0 / (1u64 << register_1894) as f32;

	let [register_1895, register_1896, register_1897, register_1898, register_1899] = split_registers::<5>(words[379]);
	raw_estimate += 1.0 / (1u64 << register_1895) as f32 + 1.0 / (1u64 << register_1896) as f32 + 1.0 / (1u64 << register_1897) as f32 + 1.0 / (1u64 << register_1898) as f32 + 1.0 / (1u64 << register_1899) as f32;

	let [register_1900, register_1901, register_1902, register_1903, register_1904] = split_registers::<5>(words[380]);
	raw_estimate += 1.0 / (1u64 << register_1900) as f32 + 1.0 / (1u64 << register_1901) as f32 + 1.0 / (1u64 << register_1902) as f32 + 1.0 / (1u64 << register_1903) as f32 + 1.0 / (1u64 << register_1904) as f32;

	let [register_1905, register_1906, register_1907, register_1908, register_1909] = split_registers::<5>(words[381]);
	raw_estimate += 1.0 / (1u64 << register_1905) as f32 + 1.0 / (1u64 << register_1906) as f32 + 1.0 / (1u64 << register_1907) as f32 + 1.0 / (1u64 << register_1908) as f32 + 1.0 / (1u64 << register_1909) as f32;

	let [register_1910, register_1911, register_1912, register_1913, register_1914] = split_registers::<5>(words[382]);
	raw_estimate += 1.0 / (1u64 << register_1910) as f32 + 1.0 / (1u64 << register_1911) as f32 + 1.0 / (1u64 << register_1912) as f32 + 1.0 / (1u64 << register_1913) as f32 + 1.0 / (1u64 << register_1914) as f32;

	let [register_1915, register_1916, register_1917, register_1918, register_1919] = split_registers::<5>(words[383]);
	raw_estimate += 1.0 / (1u64 << register_1915) as f32 + 1.0 / (1u64 << register_1916) as f32 + 1.0 / (1u64 << register_1917) as f32 + 1.0 / (1u64 << register_1918) as f32 + 1.0 / (1u64 << register_1919) as f32;

	let [register_1920, register_1921, register_1922, register_1923, register_1924] = split_registers::<5>(words[384]);
	raw_estimate += 1.0 / (1u64 << register_1920) as f32 + 1.0 / (1u64 << register_1921) as f32 + 1.0 / (1u64 << register_1922) as f32 + 1.0 / (1u64 << register_1923) as f32 + 1.0 / (1u64 << register_1924) as f32;

	let [register_1925, register_1926, register_1927, register_1928, register_1929] = split_registers::<5>(words[385]);
	raw_estimate += 1.0 / (1u64 << register_1925) as f32 + 1.0 / (1u64 << register_1926) as f32 + 1.0 / (1u64 << register_1927) as f32 + 1.0 / (1u64 << register_1928) as f32 + 1.0 / (1u64 << register_1929) as f32;

	let [register_1930, register_1931, register_1932, register_1933, register_1934] = split_registers::<5>(words[386]);
	raw_estimate += 1.0 / (1u64 << register_1930) as f32 + 1.0 / (1u64 << register_1931) as f32 + 1.0 / (1u64 << register_1932) as f32 + 1.0 / (1u64 << register_1933) as f32 + 1.0 / (1u64 << register_1934) as f32;

	let [register_1935, register_1936, register_1937, register_1938, register_1939] = split_registers::<5>(words[387]);
	raw_estimate += 1.0 / (1u64 << register_1935) as f32 + 1.0 / (1u64 << register_1936) as f32 + 1.0 / (1u64 << register_1937) as f32 + 1.0 / (1u64 << register_1938) as f32 + 1.0 / (1u64 << register_1939) as f32;

	let [register_1940, register_1941, register_1942, register_1943, register_1944] = split_registers::<5>(words[388]);
	raw_estimate += 1.0 / (1u64 << register_1940) as f32 + 1.0 / (1u64 << register_1941) as f32 + 1.0 / (1u64 << register_1942) as f32 + 1.0 / (1u64 << register_1943) as f32 + 1.0 / (1u64 << register_1944) as f32;

	let [register_1945, register_1946, register_1947, register_1948, register_1949] = split_registers::<5>(words[389]);
	raw_estimate += 1.0 / (1u64 << register_1945) as f32 + 1.0 / (1u64 << register_1946) as f32 + 1.0 / (1u64 << register_1947) as f32 + 1.0 / (1u64 << register_1948) as f32 + 1.0 / (1u64 << register_1949) as f32;

	let [register_1950, register_1951, register_1952, register_1953, register_1954] = split_registers::<5>(words[390]);
	raw_estimate += 1.0 / (1u64 << register_1950) as f32 + 1.0 / (1u64 << register_1951) as f32 + 1.0 / (1u64 << register_1952) as f32 + 1.0 / (1u64 << register_1953) as f32 + 1.0 / (1u64 << register_1954) as f32;

	let [register_1955, register_1956, register_1957, register_1958, register_1959] = split_registers::<5>(words[391]);
	raw_estimate += 1.0 / (1u64 << register_1955) as f32 + 1.0 / (1u64 << register_1956) as f32 + 1.0 / (1u64 << register_1957) as f32 + 1.0 / (1u64 << register_1958) as f32 + 1.0 / (1u64 << register_1959) as f32;

	let [register_1960, register_1961, register_1962, register_1963, register_1964] = split_registers::<5>(words[392]);
	raw_estimate += 1.0 / (1u64 << register_1960) as f32 + 1.0 / (1u64 << register_1961) as f32 + 1.0 / (1u64 << register_1962) as f32 + 1.0 / (1u64 << register_1963) as f32 + 1.0 / (1u64 << register_1964) as f32;

	let [register_1965, register_1966, register_1967, register_1968, register_1969] = split_registers::<5>(words[393]);
	raw_estimate += 1.0 / (1u64 << register_1965) as f32 + 1.0 / (1u64 << register_1966) as f32 + 1.0 / (1u64 << register_1967) as f32 + 1.0 / (1u64 << register_1968) as f32 + 1.0 / (1u64 << register_1969) as f32;

	let [register_1970, register_1971, register_1972, register_1973, register_1974] = split_registers::<5>(words[394]);
	raw_estimate += 1.0 / (1u64 << register_1970) as f32 + 1.0 / (1u64 << register_1971) as f32 + 1.0 / (1u64 << register_1972) as f32 + 1.0 / (1u64 << register_1973) as f32 + 1.0 / (1u64 << register_1974) as f32;

	let [register_1975, register_1976, register_1977, register_1978, register_1979] = split_registers::<5>(words[395]);
	raw_estimate += 1.0 / (1u64 << register_1975) as f32 + 1.0 / (1u64 << register_1976) as f32 + 1.0 / (1u64 << register_1977) as f32 + 1.0 / (1u64 << register_1978) as f32 + 1.0 / (1u64 << register_1979) as f32;

	let [register_1980, register_1981, register_1982, register_1983, register_1984] = split_registers::<5>(words[396]);
	raw_estimate += 1.0 / (1u64 << register_1980) as f32 + 1.0 / (1u64 << register_1981) as f32 + 1.0 / (1u64 << register_1982) as f32 + 1.0 / (1u64 << register_1983) as f32 + 1.0 / (1u64 << register_1984) as f32;

	let [register_1985, register_1986, register_1987, register_1988, register_1989] = split_registers::<5>(words[397]);
	raw_estimate += 1.0 / (1u64 << register_1985) as f32 + 1.0 / (1u64 << register_1986) as f32 + 1.0 / (1u64 << register_1987) as f32 + 1.0 / (1u64 << register_1988) as f32 + 1.0 / (1u64 << register_1989) as f32;

	let [register_1990, register_1991, register_1992, register_1993, register_1994] = split_registers::<5>(words[398]);
	raw_estimate += 1.0 / (1u64 << register_1990) as f32 + 1.0 / (1u64 << register_1991) as f32 + 1.0 / (1u64 << register_1992) as f32 + 1.0 / (1u64 << register_1993) as f32 + 1.0 / (1u64 << register_1994) as f32;

	let [register_1995, register_1996, register_1997, register_1998, register_1999] = split_registers::<5>(words[399]);
	raw_estimate += 1.0 / (1u64 << register_1995) as f32 + 1.0 / (1u64 << register_1996) as f32 + 1.0 / (1u64 << register_1997) as f32 + 1.0 / (1u64 << register_1998) as f32 + 1.0 / (1u64 << register_1999) as f32;

	let [register_2000, register_2001, register_2002, register_2003, register_2004] = split_registers::<5>(words[400]);
	raw_estimate += 1.0 / (1u64 << register_2000) as f32 + 1.0 / (1u64 << register_2001) as f32 + 1.0 / (1u64 << register_2002) as f32 + 1.0 / (1u64 << register_2003) as f32 + 1.0 / (1u64 << register_2004) as f32;

	let [register_2005, register_2006, register_2007, register_2008, register_2009] = split_registers::<5>(words[401]);
	raw_estimate += 1.0 / (1u64 << register_2005) as f32 + 1.0 / (1u64 << register_2006) as f32 + 1.0 / (1u64 << register_2007) as f32 + 1.0 / (1u64 << register_2008) as f32 + 1.0 / (1u64 << register_2009) as f32;

	let [register_2010, register_2011, register_2012, register_2013, register_2014] = split_registers::<5>(words[402]);
	raw_estimate += 1.0 / (1u64 << register_2010) as f32 + 1.0 / (1u64 << register_2011) as f32 + 1.0 / (1u64 << register_2012) as f32 + 1.0 / (1u64 << register_2013) as f32 + 1.0 / (1u64 << register_2014) as f32;

	let [register_2015, register_2016, register_2017, register_2018, register_2019] = split_registers::<5>(words[403]);
	raw_estimate += 1.0 / (1u64 << register_2015) as f32 + 1.0 / (1u64 << register_2016) as f32 + 1.0 / (1u64 << register_2017) as f32 + 1.0 / (1u64 << register_2018) as f32 + 1.0 / (1u64 << register_2019) as f32;

	let [register_2020, register_2021, register_2022, register_2023, register_2024] = split_registers::<5>(words[404]);
	raw_estimate += 1.0 / (1u64 << register_2020) as f32 + 1.0 / (1u64 << register_2021) as f32 + 1.0 / (1u64 << register_2022) as f32 + 1.0 / (1u64 << register_2023) as f32 + 1.0 / (1u64 << register_2024) as f32;

	let [register_2025, register_2026, register_2027, register_2028, register_2029] = split_registers::<5>(words[405]);
	raw_estimate += 1.0 / (1u64 << register_2025) as f32 + 1.0 / (1u64 << register_2026) as f32 + 1.0 / (1u64 << register_2027) as f32 + 1.0 / (1u64 << register_2028) as f32 + 1.0 / (1u64 << register_2029) as f32;

	let [register_2030, register_2031, register_2032, register_2033, register_2034] = split_registers::<5>(words[406]);
	raw_estimate += 1.0 / (1u64 << register_2030) as f32 + 1.0 / (1u64 << register_2031) as f32 + 1.0 / (1u64 << register_2032) as f32 + 1.0 / (1u64 << register_2033) as f32 + 1.0 / (1u64 << register_2034) as f32;

	let [register_2035, register_2036, register_2037, register_2038, register_2039] = split_registers::<5>(words[407]);
	raw_estimate += 1.0 / (1u64 << register_2035) as f32 + 1.0 / (1u64 << register_2036) as f32 + 1.0 / (1u64 << register_2037) as f32 + 1.0 / (1u64 << register_2038) as f32 + 1.0 / (1u64 << register_2039) as f32;

	let [register_2040, register_2041, register_2042, register_2043, register_2044] = split_registers::<5>(words[408]);
	raw_estimate += 1.0 / (1u64 << register_2040) as f32 + 1.0 / (1u64 << register_2041) as f32 + 1.0 / (1u64 << register_2042) as f32 + 1.0 / (1u64 << register_2043) as f32 + 1.0 / (1u64 << register_2044) as f32;

	let [register_2045, register_2046, register_2047, _, _] = split_registers::<5>(words[409]);
	raw_estimate += 1.0 / (1u64 << register_2045) as f32 + 1.0 / (1u64 << register_2046) as f32 + 1.0 / (1u64 << register_2047) as f32;


    raw_estimate
}
