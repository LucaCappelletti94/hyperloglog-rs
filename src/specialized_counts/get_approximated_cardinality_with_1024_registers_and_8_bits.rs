
use crate::prelude::*;

#[inline]
pub fn get_approximated_cardinality_with_1024_registers_and_8_bits(words: &[u32; 256]) -> f32 {
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


    raw_estimate
}
