// raw estimates to determine bias as determined by Google.
pub const RAW_ESTIMATE_DATA: [&[f32]; 15] = [
    // precision 4
    &[
        11., 11.717, 12.207, 12.7896, 13.2882, 13.8204, 14.3772, 14.9342, 15.5202, 16.161, 16.7722,
        17.4636, 18.0396, 18.6766, 19.3566, 20.0454, 20.7936, 21.4856, 22.2666, 22.9946, 23.766,
        24.4692, 25.3638, 26.0764, 26.7864, 27.7602, 28.4814, 29.433, 30.2926, 31.0664, 31.9996,
        32.7956, 33.5366, 34.5894, 35.5738, 36.2698, 37.3682, 38.0544, 39.2342, 40.0108, 40.7966,
        41.9298, 42.8704, 43.6358, 44.5194, 45.773, 46.6772, 47.6174, 48.4888, 49.3304, 50.2506,
        51.4996, 52.3824, 53.3078, 54.3984, 55.5838, 56.6618, 57.2174, 58.3514, 59.0802, 60.1482,
        61.0376, 62.3598, 62.8078, 63.9744, 64.914, 65.781, 67.1806, 68.0594, 68.8446, 69.7928,
        70.8248, 71.8324, 72.8598, 73.6246, 74.7014, 75.393, 76.6708, 77.2394,
    ],
    // precision 5
    &[
        23., 23.1194, 23.8208, 24.2318, 24.77, 25.2436, 25.7774, 26.2848, 26.8224, 27.3742,
        27.9336, 28.503, 29.0494, 29.6292, 30.2124, 30.798, 31.367, 31.9728, 32.5944, 33.217,
        33.8438, 34.3696, 35.0956, 35.7044, 36.324, 37.0668, 37.6698, 38.3644, 39.049, 39.6918,
        40.4146, 41.082, 41.687, 42.5398, 43.2462, 43.857, 44.6606, 45.4168, 46.1248, 46.9222,
        47.6804, 48.447, 49.3454, 49.9594, 50.7636, 51.5776, 52.331, 53.19, 53.9676, 54.7564,
        55.5314, 56.4442, 57.3708, 57.9774, 58.9624, 59.8796, 60.755, 61.472, 62.2076, 63.1024,
        63.8908, 64.7338, 65.7728, 66.629, 67.413, 68.3266, 69.1524, 70.2642, 71.1806, 72.0566,
        72.9192, 73.7598, 74.3516, 75.5802, 76.4386, 77.4916, 78.1524, 79.1892, 79.8414, 80.8798,
        81.8376, 82.4698, 83.7656, 84.331, 85.5914, 86.6012, 87.7016, 88.5582, 89.3394, 90.3544,
        91.4912, 92.308, 93.3552, 93.9746, 95.2052, 95.727, 97.1322, 98.3944, 98.7588, 100.242,
        101.1914, 102.2538, 102.8776, 103.6292, 105.1932, 105.9152, 107.0868, 107.6728, 108.7144,
        110.3114, 110.8716, 111.245, 112.7908, 113.7064, 114.636, 115.7464, 116.1788, 117.7464,
        118.4896, 119.6166, 120.5082, 121.7798, 122.9028, 123.4426, 124.8854, 125.705, 126.4652,
        128.3464, 128.3462, 130.0398, 131.0342, 131.0042, 132.4766, 133.511, 134.7252, 135.425,
        136.5172, 138.0572, 138.6694, 139.3712, 140.8598, 141.4594, 142.554, 143.4006, 144.7374,
        146.1634, 146.8994, 147.605, 147.9304, 149.1636, 150.2468, 151.5876, 152.2096, 153.7032,
        154.7146, 155.807, 156.9228, 157.0372, 158.5852,
    ],
    // precision 6
    &[
        46., 46.1902, 47.271, 47.8358, 48.8142, 49.2854, 50.317, 51.354, 51.8924, 52.9436, 53.4596,
        54.5262, 55.6248, 56.1574, 57.2822, 57.837, 58.9636, 60.074, 60.7042, 61.7976, 62.4772,
        63.6564, 64.7942, 65.5004, 66.686, 67.291, 68.5672, 69.8556, 70.4982, 71.8204, 72.4252,
        73.7744, 75.0786, 75.8344, 77.0294, 77.8098, 79.0794, 80.5732, 81.1878, 82.5648, 83.2902,
        84.6784, 85.3352, 86.8946, 88.3712, 89.0852, 90.499, 91.2686, 92.6844, 94.2234, 94.9732,
        96.3356, 97.2286, 98.7262, 100.3284, 101.1048, 102.5962, 103.3562, 105.1272, 106.4184,
        107.4974, 109.0822, 109.856, 111.48, 113.2834, 114.0208, 115.637, 116.5174, 118.0576,
        119.7476, 120.427, 122.1326, 123.2372, 125.2788, 126.6776, 127.7926, 129.1952, 129.9564,
        131.6454, 133.87, 134.5428, 136.2, 137.0294, 138.6278, 139.6782, 141.792, 143.3516,
        144.2832, 146.0394, 147.0748, 148.4912, 150.849, 151.696, 153.5404, 154.073, 156.3714,
        157.7216, 158.7328, 160.4208, 161.4184, 163.9424, 165.2772, 166.411, 168.1308, 168.769,
        170.9258, 172.6828, 173.7502, 175.706, 176.3886, 179.0186, 180.4518, 181.927, 183.4172,
        184.4114, 186.033, 188.5124, 189.5564, 191.6008, 192.4172, 193.8044, 194.997, 197.4548,
        198.8948, 200.2346, 202.3086, 203.1548, 204.8842, 206.6508, 206.6772, 209.7254, 210.4752,
        212.7228, 214.6614, 215.1676, 217.793, 218.0006, 219.9052, 221.66, 223.5588, 225.1636,
        225.6882, 227.7126, 229.4502, 231.1978, 232.9756, 233.1654, 236.727, 238.1974, 237.7474,
        241.1346, 242.3048, 244.1948, 245.3134, 246.879, 249.1204, 249.853, 252.6792, 253.857,
        254.4486, 257.2362, 257.9534, 260.0286, 260.5632, 262.663, 264.723, 265.7566, 267.2566,
        267.1624, 270.62, 272.8216, 273.2166, 275.2056, 276.2202, 278.3726, 280.3344, 281.9284,
        283.9728, 284.1924, 286.4872, 287.587, 289.807, 291.1206, 292.769, 294.8708, 296.665,
        297.1182, 299.4012, 300.6352, 302.1354, 304.1756, 306.1606, 307.3462, 308.5214, 309.4134,
        310.8352, 313.9684, 315.837, 316.7796, 318.9858,
    ],
    // precision 7
    &[
        92., 93.4934, 94.9758, 96.4574, 97.9718, 99.4954, 101.5302, 103.0756, 104.6374, 106.1782,
        107.7888, 109.9522, 111.592, 113.2532, 114.9086, 116.5938, 118.9474, 120.6796, 122.4394,
        124.2176, 125.9768, 128.4214, 130.2528, 132.0102, 133.8658, 135.7278, 138.3044, 140.1316,
        142.093, 144.0032, 145.9092, 148.6306, 150.5294, 152.5756, 154.6508, 156.662, 159.552,
        161.3724, 163.617, 165.5754, 167.7872, 169.8444, 172.7988, 174.8606, 177.2118, 179.3566,
        181.4476, 184.5882, 186.6816, 189.0824, 191.0258, 193.6048, 196.4436, 198.7274, 200.957,
        203.147, 205.4364, 208.7592, 211.3386, 213.781, 215.8028, 218.656, 221.6544, 223.996,
        226.4718, 229.1544, 231.6098, 234.5956, 237.0616, 239.5758, 242.4878, 244.5244, 248.2146,
        250.724, 252.8722, 255.5198, 258.0414, 261.941, 264.9048, 266.87, 269.4304, 272.028,
        274.4708, 278.37, 281.0624, 283.4668, 286.5532, 289.4352, 293.2564, 295.2744, 298.2118,
        300.7472, 304.1456, 307.2928, 309.7504, 312.5528, 315.979, 318.2102, 322.1834, 324.3494,
        327.325, 330.6614, 332.903, 337.2544, 339.9042, 343.215, 345.2864, 348.0814, 352.6764,
        355.301, 357.139, 360.658, 363.1732, 366.5902, 369.9538, 373.0828, 375.922, 378.9902,
        382.7328, 386.4538, 388.1136, 391.2234, 394.0878, 396.708, 401.1556, 404.1852, 406.6372,
        409.6822, 412.7796, 416.6078, 418.4916, 422.131, 424.5376, 428.1988, 432.211, 434.4502,
        438.5282, 440.912, 444.0448, 447.7432, 450.8524, 453.7988, 456.7858, 458.8868, 463.9886,
        466.5064, 468.9124, 472.6616, 475.4682, 478.582, 481.304, 485.2738, 488.6894, 490.329,
        496.106, 497.6908, 501.1374, 504.5322, 506.8848, 510.3324, 513.4512, 516.179, 520.4412,
        522.6066, 526.167, 528.7794, 533.379, 536.067, 538.46, 542.9116, 545.692, 547.9546,
        552.493, 555.2722, 557.335, 562.449, 564.2014, 569.0738, 571.0974, 574.8564, 578.2996,
        581.409, 583.9704, 585.8098, 589.6528, 594.5998, 595.958, 600.068, 603.3278, 608.2016,
        609.9632, 612.864, 615.43, 620.7794, 621.272, 625.8644, 629.206, 633.219, 634.5154,
        638.6102,
    ],
    // precision 8
    &[
        184.2152, 187.2454, 190.2096, 193.6652, 196.6312, 199.6822, 203.249, 206.3296, 210.0038,
        213.2074, 216.4612, 220.27, 223.5178, 227.4412, 230.8032, 234.1634, 238.1688, 241.6074,
        245.6946, 249.2664, 252.8228, 257.0432, 260.6824, 264.9464, 268.6268, 272.2626, 276.8376,
        280.4034, 284.8956, 288.8522, 292.7638, 297.3552, 301.3556, 305.7526, 309.9292, 313.8954,
        318.8198, 322.7668, 327.298, 331.6688, 335.9466, 340.9746, 345.1672, 349.3474, 354.3028,
        358.8912, 364.114, 368.4646, 372.9744, 378.4092, 382.6022, 387.843, 392.5684, 397.1652,
        402.5426, 407.4152, 412.5388, 417.3592, 422.1366, 427.486, 432.3918, 437.5076, 442.509,
        447.3834, 453.3498, 458.0668, 463.7346, 469.1228, 473.4528, 479.7, 484.644, 491.0518,
        495.5774, 500.9068, 506.432, 512.1666, 517.434, 522.6644, 527.4894, 533.6312, 538.3804,
        544.292, 550.5496, 556.0234, 562.8206, 566.6146, 572.4188, 579.117, 583.6762, 590.6576,
        595.7864, 601.509, 607.5334, 612.9204, 619.772, 624.2924, 630.8654, 636.1836, 642.745,
        649.1316, 655.0386, 660.0136, 666.6342, 671.6196, 678.1866, 684.4282, 689.3324, 695.4794,
        702.5038, 708.129, 713.528, 720.3204, 726.463, 732.7928, 739.123, 744.7418, 751.2192,
        756.5102, 762.6066, 769.0184, 775.2224, 781.4014, 787.7618, 794.1436, 798.6506, 805.6378,
        811.766, 819.7514, 824.5776, 828.7322, 837.8048, 843.6302, 849.9336, 854.4798, 861.3388,
        867.9894, 873.8196, 880.3136, 886.2308, 892.4588, 899.0816, 905.4076, 912.0064, 917.3878,
        923.619, 929.998, 937.3482, 943.9506, 947.991, 955.1144, 962.203, 968.8222, 975.7324,
        981.7826, 988.7666, 994.2648, 1_000.313, 1_007.408, 1_013.754, 1_020.338, 1_026.716,
        1_031.748, 1_037.429, 1_045.393, 1_051.228, 1_058.343, 1_062.873, 1_071.884, 1_076.806,
        1_082.918, 1_089.168, 1_095.503, 1_102.525, 1_107.226, 1_115.315, 1_120.93, 1_127.252,
        1_134.15, 1_139.041, 1_147.545, 1_153.33, 1_158.197, 1_166.526, 1_174.333, 1_175.657,
        1_184.422, 1_190.917, 1_197.129, 1_204.461, 1_210.458, 1_218.873, 1_225.334, 1_226.659,
        1_236.577, 1_241.363, 1_249.407, 1_254.657, 1_260.801, 1_266.545, 1_274.519,
    ],
    // precision 9
    &[
        369., 374.8294, 381.2452, 387.6698, 394.1464, 400.2024, 406.8782, 413.6598, 420.462,
        427.2826, 433.7102, 440.7416, 447.9366, 455.1046, 462.285, 469.0668, 476.306, 483.8448,
        491.301, 498.9886, 506.2422, 513.8138, 521.7074, 529.7428, 537.8402, 545.1664, 553.3534,
        561.594, 569.6886, 577.7876, 585.65, 594.228, 602.8036, 611.1666, 620.0818, 628.0824,
        637.2574, 646.302, 655.1644, 664.0056, 672.3802, 681.7192, 690.5234, 700.2084, 708.831,
        718.485, 728.1112, 737.4764, 746.76, 756.3368, 766.5538, 775.5058, 785.2646, 795.5902,
        804.3818, 814.8998, 824.9532, 835.2062, 845.2798, 854.4728, 864.9582, 875.3292, 886.171,
        896.781, 906.5716, 916.7048, 927.5322, 937.875, 949.3972, 958.3464, 969.7274, 980.2834,
        992.1444, 1_003.426, 1_013.017, 1_024.018, 1_035.044, 1_046.34, 1_057.686, 1_068.984,
        1_079.031, 1_091.677, 1_102.319, 1_113.485, 1_124.442, 1_135.739, 1_147.149, 1_158.92,
        1_169.406, 1_181.534, 1_193.283, 1_203.895, 1_216.329, 1_226.215, 1_239.668, 1_251.995,
        1_262.123, 1_275.434, 1_285.738, 1_296.076, 1_308.969, 1_320.496, 1_333.1, 1_343.986,
        1_357.775, 1_368.321, 1_380.484, 1_392.739, 1_406.076, 1_416.91, 1_428.973, 1_440.923,
        1_453.929, 1_462.617, 1_476.05, 1_490.3, 1_500.613, 1_513.739, 1_524.517, 1_536.632,
        1_548.258, 1_562.377, 1_572.423, 1_587.123, 1_596.516, 1_610.594, 1_622.597, 1_633.122,
        1_647.767, 1_658.504, 1_671.57, 1_683.704, 1_695.414, 1_708.71, 1_720.609, 1_732.652,
        1_747.841, 1_756.407, 1_769.979, 1_782.328, 1_797.522, 1_808.319, 1_819.069, 1_834.354,
        1_844.575, 1_856.281, 1_871.129, 1_880.785, 1_893.962, 1_906.342, 1_920.655, 1_932.93,
        1_945.858, 1_955.473, 1_968.825, 1_980.645, 1_995.96, 2_008.349, 2_019.856, 2_033.033,
        2_044.021, 2_059.396, 2_069.917, 2_082.608, 2_093.704, 2_106.611, 2_118.912, 2_132.301,
        2_144.763, 2_159.842, 2_171.021, 2_183.101, 2_193.511, 2_208.052, 2_221.319, 2_233.328,
        2_247.295, 2_257.722, 2_273.342, 2_286.564, 2_299.679, 2_310.811, 2_322.331, 2_335.516,
        2_349.874, 2_363.597, 2_373.865, 2_387.192, 2_401.833, 2_414.85, 2_424.544, 2_436.759,
        2_447.168, 2_464.196, 2_474.344, 2_489.001, 2_497.453, 2_513.659, 2_527.19, 2_540.703,
        2_553.768,
    ],
    // precision 10
    &[
        738.1256, 750.4234, 763.1064, 775.4732, 788.4636, 801.0644, 814.488, 827.9654, 841.0832,
        854.7864, 868.1992, 882.2176, 896.5228, 910.1716, 924.7752, 938.899, 953.6126, 968.6492,
        982.9474, 998.5214, 1_013.106, 1_028.636, 1_044.247, 1_059.459, 1_075.383, 1_091.058,
        1_106.861, 1_123.387, 1_139.506, 1_156.186, 1_172.463, 1_189.339, 1_206.194, 1_223.129,
        1_240.185, 1_257.291, 1_275.332, 1_292.852, 1_310.52, 1_328.485, 1_345.932, 1_364.552,
        1_381.466, 1_400.426, 1_419.849, 1_438.152, 1_456.896, 1_474.879, 1_494.118, 1_513.62,
        1_532.513, 1_551.932, 1_570.773, 1_590.609, 1_610.533, 1_630.592, 1_650.429, 1_669.766,
        1_690.411, 1_710.734, 1_730.901, 1_750.449, 1_770.156, 1_791.634, 1_812.731, 1_833.626,
        1_853.953, 1_874.874, 1_896.833, 1_918.197, 1_939.559, 1_961.07, 1_983.037, 2_003.18,
        2_026.071, 2_047.488, 2_070.085, 2_091.294, 2_114.333, 2_135.963, 2_158.29, 2_181.081,
        2_202.033, 2_224.483, 2_246.39, 2_269.72, 2_292.171, 2_314.236, 2_338.935, 2_360.891,
        2_384.026, 2_408.383, 2_430.154, 2_454.868, 2_476.99, 2_501.437, 2_522.87, 2_548.041,
        2_570.674, 2_593.521, 2_617.016, 2_640.23, 2_664.096, 2_687.499, 2_714.259, 2_735.391,
        2_759.624, 2_781.838, 2_808.007, 2_830.652, 2_856.245, 2_877.214, 2_903.455, 2_926.785,
        2_951.229, 2_976.468, 3_000.867, 3_023.651, 3_049.91, 3_073.598, 3_098.162, 3_121.556,
        3_146.233, 3_170.948, 3_195.59, 3_221.335, 3_242.703, 3_271.611, 3_296.555, 3_317.738,
        3_345.072, 3_369.952, 3_394.326, 3_418.182, 3_444.693, 3_469.086, 3_494.275, 3_517.87,
        3_544.248, 3_565.377, 3_588.723, 3_616.979, 3_643.75, 3_668.681, 3_695.72, 3_719.739,
        3_742.622, 3_770.446, 3_795.66, 3_819.906, 3_844.002, 3_869.517, 3_895.682, 3_920.862,
        3_947.136, 3_973.985, 3_995.477, 4_021.62, 4_046.628, 4_074.65, 4_096.226, 4_121.831,
        4_146.641, 4_173.276, 4_195.074, 4_223.97, 4_251.371, 4_272.997, 4_300.805, 4_326.302,
        4_353.125, 4_374.312, 4_403.032, 4_426.819, 4_450.06, 4_478.521, 4_504.812, 4_528.893,
        4_553.958, 4_578.871, 4_603.838, 4_632.387, 4_655.513, 4_675.821, 4_704.622, 4_731.986,
        4_755.417, 4_781.263, 4_804.332, 4_832.305, 4_862.875, 4_883.415, 4_906.954, 4_935.352,
        4_954.353, 4_984.025, 5_011.217, 5_035.326, 5_057.367, 5_084.183,
    ],
    // precision 11
    &[
        1_477., 1_501.601, 1_526.58, 1_551.794, 1_577.304, 1_603.206, 1_629.84, 1_656.229,
        1_682.946, 1_709.993, 1_737.303, 1_765.425, 1_793.058, 1_821.609, 1_849.626, 1_878.557,
        1_908.527, 1_937.515, 1_967.187, 1_997.388, 2_027.37, 2_058.197, 2_089.573, 2_120.101,
        2_151.967, 2_183.292, 2_216.077, 2_247.858, 2_280.656, 2_313.041, 2_345.714, 2_380.311,
        2_414.181, 2_447.985, 2_481.656, 2_516.346, 2_551.515, 2_586.838, 2_621.745, 2_656.672,
        2_693.572, 2_729.146, 2_765.412, 2_802.873, 2_838.898, 2_876.408, 2_913.493, 2_951.494,
        2_989.678, 3_026.282, 3_065.77, 3_104.101, 3_143.739, 3_181.688, 3_221.187, 3_261.505,
        3_300.021, 3_339.806, 3_381.409, 3_421.414, 3_461.429, 3_502.229, 3_544.651, 3_586.616,
        3_627.337, 3_670.083, 3_711.154, 3_753.509, 3_797.01, 3_838.669, 3_882.168, 3_922.812,
        3_967.998, 4_009.92, 4_054.329, 4_097.571, 4_140.601, 4_185.544, 4_229.598, 4_274.583,
        4_316.944, 4_361.672, 4_406.279, 4_451.863, 4_496.183, 4_543.505, 4_589.182, 4_632.519,
        4_678.229, 4_724.891, 4_769.019, 4_817.052, 4_861.459, 4_910.16, 4_956.434, 5_002.524,
        5_048.13, 5_093.637, 5_142.816, 5_187.789, 5_237.398, 5_285.608, 5_331.086, 5_379.104,
        5_428.626, 5_474.602, 5_522.762, 5_571.582, 5_618.59, 5_667.999, 5_714.88, 5_763.454,
        5_808.698, 5_860.364, 5_910.291, 5_953.571, 6_005.923, 6_055.191, 6_104.588, 6_154.57,
        6_199.704, 6_251.176, 6_298.76, 6_350.03, 6_398.061, 6_448.469, 6_495.933, 6_548.047,
        6_597.717, 6_646.942, 6_695.921, 6_742.633, 6_793.528, 6_842.193, 6_894.237, 6_945.386,
        6_996.923, 7_044.237, 7_094.137, 7_142.227, 7_192.294, 7_238.834, 7_288.901, 7_344.091,
        7_394.854, 7_443.518, 7_490.415, 7_542.931, 7_595.674, 7_641.988, 7_694.369, 7_743.045,
        7_797.522, 7_845.53, 7_899.594, 7_950.313, 7_996.455, 8_050.944, 8_092.911, 8_153.137,
        8_197.447, 8_252.828, 8_301.873, 8_348.678, 8_401.47, 8_453.551, 8_504.66, 8_553.894,
        8_604.128, 8_657.651, 8_710.306, 8_758.908, 8_807.871, 8_862.17, 8_910.467, 8_960.77,
        9_007.277, 9_063.164, 9_121.053, 9_164.135, 9_218.159, 9_267.767, 9_319.059, 9_372.155,
        9_419.713, 9_474.372, 9_520.134, 9_572.368, 9_622.77, 9_675.845, 9_726.54, 9_778.738,
        9_827.655, 9_878.192, 9_928.778, 9_978.398, 10_026.58, 10_076.56, 10_137.16, 10_177.52,
        10_229.92,
    ],
    // precision 12
    &[
        2_954., 3_003.478, 3_053.357, 3_104.367, 3_155.324, 3_206.96, 3_259.648, 3_312.539,
        3_366.147, 3_420.258, 3_474.838, 3_530.608, 3_586.451, 3_643.38, 3_700.41, 3_757.564,
        3_815.968, 3_875.193, 3_934.838, 3_994.855, 4_055.018, 4_117.174, 4_178.448, 4_241.129,
        4_304.478, 4_367.404, 4_431.872, 4_496.373, 4_561.43, 4_627.533, 4_693.949, 4_761.553,
        4_828.726, 4_897.618, 4_965.519, 5_034.453, 5_104.865, 5_174.716, 5_244.683, 5_316.671,
        5_387.831, 5_459.904, 5_532.476, 5_604.865, 5_679.672, 5_753.757, 5_830.207, 5_905.283,
        5_980.043, 6_056.626, 6_134.319, 6_211.575, 6_290.082, 6_367.118, 6_447.98, 6_526.558,
        6_606.186, 6_686.914, 6_766.114, 6_847.082, 6_927.966, 7_010.91, 7_091.082, 7_175.396,
        7_260.345, 7_344.018, 7_426.421, 7_511.311, 7_596.069, 7_679.809, 7_765.818, 7_852.425,
        7_936.834, 8_022.363, 8_109.507, 8_200.455, 8_288.583, 8_373.366, 8_463.481, 8_549.768,
        8_642.052, 8_728.329, 8_820.953, 8_907.727, 9_001.079, 9_091.252, 9_179.988, 9_269.852,
        9_362.639, 9_453.642, 9_546.902, 9_640.662, 9_732.662, 9_824.325, 9_917.748, 10_007.94,
        10_106.75, 10_196.22, 10_289.81, 10_383.55, 10_482.31, 10_576.87, 10_668.79, 10_764.72,
        10_862.02, 10_952.79, 11_049.98, 11_146.07, 11_241.45, 11_339.28, 11_434.23, 11_530.74,
        11_627.61, 11_726.31, 11_821.6, 11_918.84, 12_015.37, 12_113.02, 12_213.04, 12_306.98,
        12_408.45, 12_504.9, 12_604.59, 12_700.93, 12_798.7, 12_898.51, 12_997.05, 13_094.79,
        13_198.48, 13_292.78, 13_392.97, 13_486.86, 13_590.16, 13_686.58, 13_783.63, 13_887.26,
        13_992.1, 14_081.08, 14_190., 14_280.09, 14_382.5, 14_486.44, 14_588.11, 14_686.24,
        14_782.28, 14_888.03, 14_985.19, 15_088.86, 15_187.1, 15_285.03, 15_383.67, 15_495.83,
        15_591.37, 15_694.2, 15_790.33, 15_898.41, 15_997.45, 16_095.5, 16_198.85, 16_291.75,
        16_402.64, 16_499.13, 16_606.24, 16_697.72, 16_796.4, 16_902.34, 17_005.77, 17_100.81,
        17_206.83, 17_305.83, 17_416.07, 17_508.41, 17_617.02, 17_715.46, 17_816.76, 17_920.17,
        18_012.92, 18_119.8, 18_223.22, 18_324.25, 18_426.63, 18_525.09, 18_629.9, 18_733.26,
        18_831.05, 18_940.14, 19_032.27, 19_131.73, 19_243.49, 19_349.69, 19_442.87, 19_547.94,
        19_653.28, 19_754.4, 19_854.07, 19_965.12, 20_065.18, 20_158.22, 20_253.35, 20_366.33,
        20_463.22,
    ],
    // precision 13
    &[
        5_908.505, 6_007.267, 6_107.347, 6_208.579, 6_311.262, 6_414.551, 6_519.338, 6_625.695,
        6_732.599, 6_841.355, 6_950.597, 7_061.308, 7_173.565, 7_287.109, 7_401.822, 7_516.434,
        7_633.38, 7_751.296, 7_870.378, 7_990.292, 8_110.79, 8_233.457, 8_356.604, 8_482.271,
        8_607.771, 8_735.099, 8_863.186, 8_993.475, 9_123.85, 9_255.679, 9_388.545, 9_522.752,
        9_657.311, 9_792.609, 9_930.564, 10_068.79, 10_206.73, 10_347.81, 10_490.32, 10_632.08,
        10_775.99, 10_920.47, 11_066.12, 11_213.07, 11_358.04, 11_508.1, 11_659.17, 11_808.75,
        11_959.49, 12_112.13, 12_265.04, 12_420.38, 12_578.93, 12_734.31, 12_890., 13_047.21,
        13_207.31, 13_368.51, 13_528.02, 13_689.85, 13_852.75, 14_018.32, 14_180.54, 14_346.97,
        14_513.51, 14_677.87, 14_846.22, 15_017.42, 15_184.97, 15_356.34, 15_529.3, 15_697.36,
        15_871.87, 16_042.19, 16_216.41, 16_389.42, 16_565.91, 16_742.33, 16_919., 17_094.76,
        17_273.97, 17_451.83, 17_634.42, 17_810.6, 17_988.92, 18_171.05, 18_354.79, 18_539.47,
        18_721.04, 18_905., 19_081.87, 19_271.91, 19_451.87, 19_637.98, 19_821.29, 20_013.13,
        20_199.39, 20_387.87, 20_572.95, 20_770.78, 20_955.17, 21_144.75, 21_329.99, 21_520.71,
        21_712.7, 21_906.39, 22_096.26, 22_286.05, 22_475.05, 22_665.51, 22_862.85, 23_055.53,
        23_249.61, 23_437.85, 23_636.27, 23_826.09, 24_020.33, 24_213.39, 24_411.74, 24_602.96,
        24_805.79, 24_998.15, 25_193.96, 25_389.02, 25_585.84, 25_780.7, 25_981.27, 26_175.98,
        26_376.53, 26_570.2, 26_773.39, 26_962.98, 27_163.06, 27_368.16, 27_565.05, 27_758.74,
        27_961.13, 28_163.23, 28_362.38, 28_565.77, 28_758.64, 28_956.98, 29_163.47, 29_354.7,
        29_561.12, 29_767.99, 29_960., 30_164.05, 30_366.98, 30_562.53, 30_762.99, 30_976.16,
        31_166.27, 31_376.72, 31_570.37, 31_770.81, 31_974.89, 32_179.53, 32_387.54, 32_582.35,
        32_794.08, 32_989.95, 33_191.84, 33_392.47, 33_595.66, 33_801.87, 34_000.34, 34_200.09,
        34_402.68, 34_610.06, 34_804.01, 35_011.13, 35_218.67, 35_418.66, 35_619.08, 35_830.65,
        36_028.5, 36_229.79, 36_438.64, 36_630.78, 36_833.31, 37_048.67, 37_247.39, 37_453.59,
        37_669.36, 37_854.55, 38_059.31, 38_268.09, 38_470.25, 38_674.71, 38_876.17, 39_068.38,
        39_281.91, 39_492.86, 39_684.86, 39_898.41, 40_093.18, 40_297.69, 40_489.71, 40_717.24,
    ],
    // precision 14
    &[
        11_817.48, 12_015., 12_215.38, 12_417.75, 12_623.18, 12_830.01, 13_040.01, 13_252.5,
        13_466.18, 13_683.27, 13_902.03, 14_123.98, 14_347.39, 14_573.78, 14_802.69, 15_033.68,
        15_266.91, 15_502.86, 15_741.49, 15_980.8, 16_223.89, 16_468.63, 16_715.73, 16_965.57,
        17_217.2, 17_470.67, 17_727.85, 17_986.79, 18_247.69, 18_510.96, 18_775.3, 19_044.75,
        19_314.44, 19_587.2, 19_862.26, 20_135.92, 20_417.03, 20_697.98, 20_979.61, 21_265.03,
        21_550.72, 21_841.69, 22_132.16, 22_428.14, 22_722.13, 23_020.56, 23_319.74, 23_620.4,
        23_925.27, 24_226.92, 24_535.58, 24_845.51, 25_155.96, 25_470.38, 25_785.97, 26_103.78,
        26_420.41, 26_742.02, 27_062.88, 27_388.42, 27_714.6, 28_042.3, 28_365.45, 28_701.15,
        29_031.8, 29_364.22, 29_704.5, 30_037.15, 30_380.11, 30_723.82, 31_059.51, 31_404.95,
        31_751.67, 32_095.27, 32_444.78, 32_794.77, 33_145.2, 33_498.42, 33_847.65, 34_209.01,
        34_560.85, 34_919.48, 35_274.98, 35_635.13, 35_996.33, 36_359.14, 36_722.83, 37_082.85,
        37_447.74, 37_815.96, 38_191.07, 38_559.41, 38_924.81, 39_294.67, 39_663.97, 40_042.26,
        40_416.2, 40_779.2, 41_161.64, 41_540.9, 41_921.2, 42_294.77, 42_678.53, 43_061.35,
        43_432.38, 43_818.43, 44_198.66, 44_583.01, 44_970.48, 45_353.92, 45_729.86, 46_118.22,
        46_511.57, 46_900.74, 47_280.7, 47_668.15, 48_055.68, 48_446.94, 48_838.71, 49_217.73,
        49_613.78, 50_010.75, 50_410.02, 50_793.79, 51_190.25, 51_583.19, 51_971.08, 52_376.53,
        52_763.32, 53_165.55, 53_556.56, 53_948.27, 54_346.35, 54_748.79, 55_138.58, 55_543.48,
        55_941.18, 56_333.78, 56_745.15, 57_142.79, 57_545.22, 57_936., 58_348.53, 58_737.55,
        59_158.6, 59_542.69, 59_958.8, 60_349.38, 60_755.02, 61_147.61, 61_548.19, 61_946.07,
        62_348.6, 62_763.6, 63_162.78, 63_560.64, 63_974.35, 64_366.49, 64_771.59, 65_176.74,
        65_597.39, 65_995.91, 66_394.04, 66_822.94, 67_203.63, 67_612.2, 68_019.01, 68_420.04,
        68_821.22, 69_235.84, 69_640.07, 70_055.15, 70_466.36, 70_863.43, 71_276.25, 71_677.03,
        72_080.2, 72_493.02, 72_893.6, 73_314.59, 73_714.99, 74_125.3, 74_521.21, 74_933.68,
        75_341.59, 75_743.02, 76_166.03, 76_572.13, 76_973.1, 77_381.63, 77_800.61, 78_189.33,
        78_607.1, 79_012.25, 79_407.84, 79_825.73, 80_238.7, 80_646.89, 81_035.64, 81_460.04,
        81_876.39,
    ],
    // precision 15
    &[
        23_635., 24_030.8, 24_431.47, 24_837.15, 25_246.79, 25_661.33, 26_081.35, 26_505.28,
        26_933.99, 27_367.71, 27_805.32, 28_248.8, 28_696.44, 29_148.82, 29_605.51, 30_066.87,
        30_534.23, 31_006.32, 31_480.78, 31_962.24, 32_447.33, 32_938.02, 33_432.73, 33_930.73,
        34_433.99, 34_944.14, 35_457.56, 35_974.6, 36_497.33, 37_021.91, 37_554.33, 38_088.08,
        38_628.88, 39_171.32, 39_723.23, 40_274.56, 40_832.31, 41_390.61, 41_959.59, 42_532.55,
        43_102.03, 43_683.51, 44_266.69, 44_851.28, 45_440.79, 46_038.06, 46_640.32, 47_241.06,
        47_846.15, 48_454.74, 49_076.92, 49_692.54, 50_317.48, 50_939.65, 51_572.56, 52_210.29,
        52_843.74, 53_481.4, 54_127.24, 54_770.41, 55_422.66, 56_078.8, 56_736.72, 57_397.68,
        58_064.58, 58_730.31, 59_404.98, 60_077.09, 60_751.92, 61_444.14, 62_115.82, 62_808.77,
        63_501.48, 64_187.54, 64_883.66, 65_582.75, 66_274.53, 66_976.93, 67_688.78, 68_402.14,
        69_109.63, 69_822.97, 70_543.61, 71_265.52, 71_983.38, 72_708.47, 73_433.38, 74_158.47,
        74_896.49, 75_620.96, 76_362.14, 77_098.32, 77_835.77, 78_582.61, 79_323.99, 80_067.87,
        80_814.93, 81_567.01, 82_310.85, 83_061.99, 83_821.41, 84_580.86, 85_335.55, 86_092.58,
        86_851.65, 87_612.31, 88_381.2, 89_146.33, 89_907.9, 90_676.85, 91_451.41, 92_224.55,
        92_995.87, 93_763.51, 94_551.28, 95_315.19, 96_096.18, 96_881.09, 97_665.68, 98_442.68,
        99_229.3, 100_011.1, 100_790.6, 101_580.2, 102_377.8, 103_152.1, 103_944.3, 104_730.2,
        105_528.6, 106_324.9, 107_117.7, 107_890.4, 108_695.2, 109_485.2, 110_294.8, 111_075.1,
        111_878.1, 112_695.3, 113_464.6, 114_270.1, 115_068.6, 115_884.4, 116_673.3, 117_483.4,
        118_275.1, 119_085.4, 119_879.3, 120_687.6, 121_500., 122_284.9, 123_095.9, 123_912.5,
        124_709.1, 125_503.7, 126_323.3, 127_138.9, 127_943.8, 128_755.6, 129_556.5, 130_375.3,
        131_161.5, 131_971.2, 132_787.5, 133_588.1, 134_431.4, 135_220.3, 136_023.4, 136_846.7,
        137_667., 138_463.7, 139_283.7, 140_074.6, 140_901.3, 141_721.9, 142_543.2, 143_356.1,
        144_173.7, 144_973.1, 145_794.3, 146_609.6, 147_420., 148_238., 149_050.6, 149_854.8,
        150_663.2, 151_494.1, 152_313.1, 153_112.7, 153_935.7, 154_746.9, 155_559.5, 156_402.,
        157_228.7, 158_008.7, 158_820.8, 159_646.9, 160_470.5, 161_279.5, 162_093.3, 162_918.5,
        163_729.3,
    ],
    // precision 16
    &[
        47_271., 48_062.36, 48_862.71, 49_673.15, 50_492.84, 51_322.95, 52_161.03, 53_009.41,
        53_867.64, 54_734.21, 55_610.51, 56_496.21, 57_390.79, 58_297.27, 59_210.64, 60_134.67,
        61_068.03, 62_010.45, 62_962.52, 63_923.57, 64_895.02, 65_876.42, 66_862.61, 67_862.7,
        68_868.89, 69_882.85, 70_911.27, 71_944.09, 72_990.03, 74_040.69, 75_100.63, 76_174.78,
        77_252.6, 78_340.3, 79_438.26, 80_545.5, 81_657.28, 82_784.63, 83_915.51, 85_059.74,
        86_205.94, 87_364.44, 88_530.34, 89_707.37, 90_885.96, 92_080.2, 93_275.57, 94_479.39,
        95_695.92, 96_919.22, 98_148.46, 99_382.35, 100_625.7, 101_878., 103_141.6, 104_409.5,
        105_686.3, 106_967.5, 108_261.6, 109_548.2, 110_852.1, 112_162.2, 113_479., 114_806.3,
        116_137.9, 117_469.5, 118_813.5, 120_165.5, 121_516.3, 122_875.8, 124_250.5, 125_621.2,
        127_003.2, 128_387.9, 129_775.3, 131_181.8, 132_577.3, 133_980., 135_394.1, 136_800.9,
        138_233.2, 139_668.5, 141_085.2, 142_535.2, 143_969.1, 145_420.3, 146_878.1, 148_332.8,
        149_800.3, 151_269.7, 152_743.6, 154_213.1, 155_690.3, 157_169.4, 158_672.2, 160_160.1,
        161_650.7, 163_145.8, 164_645.7, 166_159.2, 167_682.2, 169_177.3, 170_700., 172_228.9,
        173_732.7, 175_265.6, 176_787.8, 178_317.1, 179_856.7, 181_400.9, 182_943.5, 184_486.7,
        186_033.5, 187_583.8, 189_148.2, 190_688.5, 192_250.2, 193_810.9, 195_354.3, 196_938.8,
        198_493.6, 200_079.3, 201_618.9, 203_205.5, 204_765.6, 206_356.1, 207_929.3, 209_498.7,
        211_086.2, 212_675.1, 214_256.8, 215_826.2, 217_412.9, 218_995.7, 220_618.6, 222_207.1,
        223_781., 225_387.4, 227_005.8, 228_590.4, 230_217.9, 231_805.1, 233_408.9, 234_995.3,
        236_601.5, 238_190.8, 239_817.2, 241_411.3, 243_002.4, 244_640.2, 246_255.3, 247_849.4,
        249_480., 251_106.9, 252_705., 254_332.9, 255_935.1, 257_526.9, 259_154.8, 260_777.6,
        262_390.2, 264_004.5, 265_643.6, 267_255.4, 268_873.4, 270_470.7, 272_106.5, 273_722.5,
        275_337.8, 276_945.7, 278_592.9, 280_204.4, 281_841.2, 283_489.2, 285_130.2, 286_735.3,
        288_364.7, 289_961.2, 291_595.5, 293_285.7, 294_899.7, 296_499.3, 298_128., 299_761.9,
        301_394.2, 302_997.7, 304_615.2, 306_269.8, 307_886.1, 309_543.1, 311_153.3, 312_782.8,
        314_421.2, 316_033.2, 317_693., 319_305.3, 320_948.7, 322_566.3, 324_228.4, 325_847.2,
    ],
    // precision 17
    &[
        94_542., 96_125.81, 97_728.02, 99_348.56, 100_988., 102_646.8, 104_324.5, 106_021.7,
        107_736.8, 109_469.3, 111_223.9, 112_995.2, 114_787.4, 116_593.1, 118_422.7, 120_267.2,
        122_134.7, 124_020.9, 125_927.3, 127_851.3, 129_788.9, 131_751., 133_726.8, 135_722.6,
        137_736.8, 139_770.6, 141_821.5, 143_891.3, 145_982.1, 148_095.4, 150_207.5, 152_355.6,
        154_515.6, 156_696., 158_887.8, 161_098.2, 163_329.9, 165_569., 167_837.4, 170_121.6,
        172_420.5, 174_732.6, 177_062.8, 179_412.5, 181_774., 184_151.9, 186_551.7, 188_965.7,
        191_402.8, 193_858., 196_305.1, 198_774.7, 201_271.3, 203_764.8, 206_299.4, 208_818.1,
        211_373.1, 213_946.8, 216_532.1, 219_105.5, 221_714.5, 224_337.5, 226_977.5, 229_613.1,
        232_270.3, 234_952.2, 237_645.4, 240_331.2, 243_034.5, 245_756.1, 248_517.7, 251_232.7,
        254_011.4, 256_786., 259_556.4, 262_368.3, 265_156.9, 267_965.3, 270_785.6, 273_616.,
        276_487.5, 279_346.6, 282_202.5, 285_074.4, 287_942.3, 290_856., 293_774., 296_678.5,
        299_603.6, 302_552.7, 305_493., 308_466.9, 311_392.6, 314_347.5, 317_319.4, 320_286.,
        323_301.7, 326_298.3, 329_301.3, 332_302., 335_309.8, 338_370.8, 341_382.9, 344_431.1,
        347_464.2, 350_507.3, 353_619.2, 356_631.2, 359_685.2, 362_776.8, 365_886.5, 368_958.2,
        372_060.7, 375_165.4, 378_237.9, 381_328.3, 384_430.5, 387_576.4, 390_683.2, 393_839.7,
        396_977.8, 400_102., 403_271.3, 406_409.8, 409_529.5, 412_678.7, 415_847.4, 419_020.8,
        422_157.1, 425_337.8, 428_479.6, 431_700.9, 434_893.2, 438_049.6, 441_210.5, 444_379.2,
        447_577.4, 450_741.9, 453_959.5, 457_137.1, 460_329.8, 463_537.5, 466_732.3, 469_960.6,
        473_164.7, 476_347.6, 479_496.2, 482_813.2, 486_025.7, 489_249.5, 492_460.2, 495_675.9,
        498_908., 502_131.8, 505_374.4, 508_551., 511_806.7, 515_026.8, 518_217., 521_524.,
        524_706., 527_951., 531_210., 534_472.5, 537_750.7, 540_926.9, 544_207.1, 547_429.4,
        550_666.4, 553_975.3, 557_150.7, 560_399.6, 563_662.7, 566_916.7, 570_146.1, 573_447.4,
        576_689.6, 579_874.6, 583_202.3, 586_503., 589_715.6, 592_910.2, 596_214.4, 599_488.,
        602_740.9, 605_983.1, 609_248.7, 612_491.4, 615_787.9, 619_107.5, 622_308., 625_577.3,
        628_840.4, 632_085.2, 635_317.6, 638_691.7, 641_887.5, 645_139.9, 648_441.6, 651_666.2,
        654_941.8,
    ],
    // precision 18
    &[
        189_084.,
        192_250.9,
        195_456.8,
        198_697.,
        201_977.8,
        205_294.4,
        208_651.8,
        212_042.1,
        215_472.3,
        218_941.9,
        222_443.9,
        225_996.9,
        229_568.2,
        233_193.6,
        236_844.5,
        240_543.2,
        244_279.5,
        248_044.3,
        251_854.6,
        255_693.2,
        259_583.6,
        263_494.6,
        267_445.4,
        271_454.1,
        275_468.8,
        279_549.5,
        283_646.5,
        287_788.2,
        291_966.1,
        296_181.2,
        300_431.5,
        304_718.6,
        309_024.,
        313_393.5,
        317_760.8,
        322_209.7,
        326_675.1,
        331_160.6,
        335_654.5,
        340_241.4,
        344_841.8,
        349_467.1,
        354_130.6,
        358_819.4,
        363_574.6,
        368_296.6,
        373_118.5,
        377_914.9,
        382_782.3,
        387_680.7,
        392_602.,
        397_544.3,
        402_529.1,
        407_546.,
        412_593.7,
        417_638.7,
        422_762.9,
        427_886.2,
        433_017.2,
        438_213.3,
        443_441.2,
        448_692.4,
        453_937.5,
        459_239.,
        464_529.6,
        469_910.1,
        475_274.,
        480_684.5,
        486_070.3,
        491_515.2,
        496_995.7,
        502_476.6,
        507_973.6,
        513_497.2,
        519_083.2,
        524_726.5,
        530_305.5,
        535_945.7,
        541_584.4,
        547_274.1,
        552_967.2,
        558_667.9,
        564_360.2,
        570_128.2,
        575_965.1,
        581_701.9,
        587_532.5,
        593_361.1,
        599_246.1,
        605_033.4,
        610_958.8,
        616_837.1,
        622_772.8,
        628_672.,
        634_675.4,
        640_574.8,
        646_585.7,
        652_574.6,
        658_611.2,
        664_642.7,
        670_713.9,
        676_737.7,
        682_797.3,
        688_837.9,
        694_917.9,
        701_009.9,
        707_173.7,
        713_257.2,
        719_415.4,
        725_636.8,
        731_710.7,
        737_906.2,
        744_103.1,
        750_313.4,
        756_504.2,
        762_712.6,
        768_877.,
        775_167.9,
        781_359.,
        787_616.,
        793_863.6,
        800_245.5,
        806_464.6,
        812_785.3,
        819_005.9,
        825_403.1,
        831_676.2,
        837_936.3,
        844_267.,
        850_642.7,
        856_959.8,
        863_322.8,
        869_699.9,
        876_102.5,
        882_355.8,
        888_694.5,
        895_159.9,
        901_536.1,
        907_872.6,
        914_293.7,
        920_615.1,
        927_131.,
        933_409.4,
        939_922.2,
        946_331.5,
        952_745.9,
        959_209.3,
        965_590.2,
        972_077.3,
        978_502.,
        984_953.2,
        991_413.3,
        997_817.5,
        1_004_222.7,
        1_010_725.7,
        1_017_177.1,
        1_023_612.5,
        1_030_098.2,
        1_036_493.7,
        1_043_112.2,
        1_049_537.,
        1_056_008.1,
        1_062_476.2,
        1_068_942.3,
        1_075_524.9,
        1_081_932.9,
        1_088_426.,
        1_094_776.,
        1_101_327.4,
        1_107_901.7,
        1_114_423.6,
        1_120_884.6,
        1_127_324.9,
        1_133_794.2,
        1_140_328.9,
        1_146_849.4,
        1_153_346.7,
        1_159_836.5,
        1_166_478.7,
        1_172_953.3,
        1_179_391.5,
        1_185_951.,
        1_192_544.1,
        1_198_913.4,
        1_205_431.,
        1_212_015.5,
        1_218_674.,
        1_225_121.7,
        1_231_551.1,
        1_238_126.4,
        1_244_673.8,
        1_251_260.6,
        1_257_697.9,
        1_264_321.,
        1_270_736.3,
        1_277_274.7,
        1_283_804.9,
        1_290_211.5,
        1_296_858.6,
        1_303_455.7,
    ],
];
