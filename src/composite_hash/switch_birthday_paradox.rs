#[cfg(feature = "precision_4")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0.00009058199999997754f64,
        0.030699435f64,
        0.04057332400000006f64,
        0.050256866666666664f64,
        0.069141455f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 0u32, 0u32, 4u32, 5u32, 6u32, 7u32];
}
#[cfg(feature = "precision_4")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0.00001703f64,
        0.00003426666666666667f64,
        0.0000522f64,
        0.00006924000000000001f64,
        0.0001215275f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 8u32];
}
#[cfg(feature = "precision_4")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00001497f64,
        0.00003063333333333333f64,
        0.000046785f64,
        0.0000622f64,
        0.00007832f64,
        0.0001092325f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 2u32, 3u32, 4u32, 5u32, 6u32, 8u32];
}
#[cfg(feature = "precision_5")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0.00001719f64,
        0.00003506f64,
        0.0000533f64,
        0.000071216f64,
        0.000124915f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 1u32, 2u32, 3u32, 4u32, 5u32, 8u32];
}
#[cfg(feature = "precision_5")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00003180666666666667f64,
        0.00006472f64,
        0.00009667999999999998f64,
        0.000113315f64,
        0.00014643599999999998f64,
        0.00017943833333333332f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 5u32, 7u32, 8u32, 10u32, 12u32];
}
#[cfg(feature = "precision_5")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00003023333333333333f64,
        0.000061476f64,
        0.00009189714285714287f64,
        0.0001076f64,
        0.00013908399999999998f64,
        0.00017036166666666665f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 5u32, 7u32, 8u32, 10u32, 12u32];
}
#[cfg(feature = "precision_6")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00001689f64,
        0.00003431333333333333f64,
        0.000068884f64,
        0.000155262f64,
        0.00022475571428571427f64,
        0.00026008875f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 2u32, 3u32, 5u32, 10u32, 14u32, 16u32];
}
#[cfg(feature = "precision_6")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00003105333333333333f64,
        0.000062236f64,
        0.000140144f64,
        0.0002028842857142857f64,
        0.00021870933333333333f64,
        0.0002971800000000001f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 5u32, 10u32, 14u32, 15u32, 20u32];
}
#[cfg(feature = "precision_6")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000030386666666666665f64,
        0.000060827999999999994f64,
        0.000136944f64,
        0.00019819428571428568f64,
        0.0002136813333333333f64,
        0.00035117416666666674f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 5u32, 10u32, 14u32, 15u32, 24u32];
}
#[cfg(feature = "precision_7")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0001117625f64,
        0.00025666705882352935f64,
        0.000304923f64,
        0.00036893666666666663f64,
        0.0004013061538461539f64,
        0.000497673125f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 8u32, 17u32, 20u32, 24u32, 26u32, 32u32];
}
#[cfg(feature = "precision_7")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00003046f64,
        0.000106585f64,
        0.00021411466666666667f64,
        0.0002450717647058823f64,
        0.0003523883333333333f64,
        0.0005977324999999998f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 8u32, 15u32, 17u32, 24u32, 40u32];
}
#[cfg(feature = "precision_7")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00003021333333333333f64,
        0.00010585f64,
        0.00021249599999999998f64,
        0.00024321058823529405f64,
        0.0003497291666666667f64,
        0.0007149370833333334f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 8u32, 15u32, 17u32, 24u32, 48u32];
}
#[cfg(feature = "precision_8")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000002979047619047619f64,
        0.00003309023255813953f64,
        0.00006258454545454545f64,
        0.00009142888888888887f64,
        0.00011963565217391303f64,
        0.00055828375f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        42u32,
        43u32,
        44u32,
        45u32,
        46u32,
        64u32,
    ];
}
#[cfg(feature = "precision_8")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000003838490566037737f64,
        0.00003358148148148148f64,
        0.00006282581818181818f64,
        0.00009136821428571428f64,
        0.00011959859649122809f64,
        0.0006780994999999997f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        53u32,
        54u32,
        55u32,
        56u32,
        57u32,
        80u32,
    ];
}
#[cfg(feature = "precision_8")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000046084375f64,
        0.000034318461538461535f64,
        0.00006358090909090908f64,
        0.00009233731343283582f64,
        0.00012092558823529409f64,
        0.0008063427083333333f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        64u32,
        65u32,
        66u32,
        67u32,
        68u32,
        96u32,
    ];
}
#[cfg(feature = "precision_9")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000006085882352941176f64,
        0.000035782558139534885f64,
        0.00006519655172413792f64,
        0.00009415113636363637f64,
        0.00012279887640449439f64,
        0.00107845859375f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        85u32,
        86u32,
        87u32,
        88u32,
        89u32,
        128u32,
    ];
}
#[cfg(feature = "precision_9")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000007804716981132077f64,
        0.0000374626168224299f64,
        0.00006707777777777778f64,
        0.00009664770642201835f64,
        0.0001257509090909091f64,
        0.0013527818750000001f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        106u32,
        107u32,
        108u32,
        109u32,
        110u32,
        160u32,
    ];
}
#[cfg(feature = "precision_9")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000090625f64,
        0.00003879457364341086f64,
        0.00006821153846153845f64,
        0.00009762519083969466f64,
        0.0001267280303030303f64,
        0.0016080390624999994f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        128u32,
        129u32,
        130u32,
        131u32,
        132u32,
        192u32,
    ];
}
#[cfg(feature = "precision_10")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000012226470588235295f64,
        0.00004189883040935672f64,
        0.00007131511627906978f64,
        0.00010065260115606937f64,
        0.00012973275862068962f64,
        0.002135128515625f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        170u32,
        171u32,
        172u32,
        173u32,
        174u32,
        255u32,
    ];
}
#[cfg(feature = "precision_10")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000004196666666666667f64,
        0.000015308920187793428f64,
        0.000044765420560747654f64,
        0.00007429488372093022f64,
        0.00010354537037037034f64,
        0.002656480000000001f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        60u32,
        213u32,
        214u32,
        215u32,
        216u32,
        319u32,
    ];
}
#[cfg(feature = "precision_10")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000006640625f64,
        0.000009630714285714285f64,
        0.000012113068181818182f64,
        0.000013986274509803922f64,
        0.000016115384615384613f64,
        0.0347327420253164f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        96u32,
        140u32,
        176u32,
        204u32,
        234u32,
        380u32,
    ];
}
#[cfg(feature = "precision_11")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00000036666666666666667f64,
        0.00000441774193548387f64,
        0.000017061764705882355f64,
        0.0000532783625730994f64,
        0.002146338625592414f64,
        0.0041505408203125f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        6u32,
        62u32,
        238u32,
        342u32,
        421u32,
        510u32,
    ];
}
#[cfg(feature = "precision_11")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00001607400881057269f64,
        0.000025704143646408834f64,
        0.009842826315789387f64,
        0.015293391231343302f64,
        0.020041806782608678f64,
        0.02932594255319148f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        227u32,
        362u32,
        489u32,
        528u32,
        563u32,
        637u32,
    ];
}
#[cfg(feature = "precision_11")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0.00000018f64,
        0.00000029999999999999993f64,
        0.0000008533333333333334f64,
        0.0000017827586206896552f64,
        0.000032730859375f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 1u32, 5u32, 7u32, 15u32, 29u32, 512u32];
}
#[cfg(feature = "precision_12")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000158372197309417f64,
        0.00004232070707070707f64,
        0.000044547763578274746f64,
        0.0039012694406548444f64,
        0.005979957611548543f64,
        0.0234970211832061f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        223u32,
        594u32,
        626u32,
        730u32,
        757u32,
        1023u32,
    ];
}
#[cfg(feature = "precision_12")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000000033333333333333334f64,
        0.00000015f64,
        0.0000005f64,
        0.0000009933333333333333f64,
        0.000001726923076923077f64,
        0.00005839871043376321f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [0u32, 3u32, 4u32, 8u32, 15u32, 26u32, 853u32];
}
#[cfg(feature = "precision_12")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000000125f64,
        0.0000004125f64,
        0.0000008666666666666668f64,
        0.000001340909090909091f64,
        0.0000015153846153846153f64,
        0.000063001171875f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        4u32,
        8u32,
        15u32,
        22u32,
        26u32,
        1024u32,
    ];
}
#[cfg(feature = "precision_13")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000000075f64,
        0.00000026666666666666667f64,
        0.0000009f64,
        0.0000010823529411764707f64,
        0.000001375f64,
        0.00009567450549450544f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        4u32,
        6u32,
        15u32,
        17u32,
        20u32,
        1365u32,
    ];
}
#[cfg(feature = "precision_13")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000000075f64,
        0.00000026666666666666667f64,
        0.0000008333333333333334f64,
        0.0000010176470588235293f64,
        0.0000012949999999999999f64,
        0.00010848487690504116f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        4u32,
        6u32,
        15u32,
        17u32,
        20u32,
        1706u32,
    ];
}
#[cfg(feature = "precision_13")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000000075f64,
        0.00000023333333333333336f64,
        0.0000007666666666666668f64,
        0.0000009470588235294116f64,
        0.00000121f64,
        0.000123515771484375f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        4u32,
        6u32,
        15u32,
        17u32,
        20u32,
        2048u32,
    ];
}
#[cfg(feature = "precision_14")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0.0000005833984375f64,
        0.000001122953216374269f64,
        0.0000012604968339016076f64,
        0.000003793870656370656f64,
        0.00008116384615384608f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        5u32,
        2048u32,
        2052u32,
        2053u32,
        2072u32,
        2730u32,
    ];
}
#[cfg(feature = "precision_14")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0f64,
        0.0000008525575946895746f64,
        0.0000021858475894245723f64,
        0.000004370926640926641f64,
        0.0000917892762965133f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        1u32,
        18u32,
        2561u32,
        2572u32,
        2590u32,
        3413u32,
    ];
}
#[cfg(feature = "precision_14")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0f64,
        0.000000002857142857142857f64,
        0.0000008828776041666666f64,
        0.0000010038073543768304f64,
        0.0000019587471600129833f64,
        0.0001077841064453125f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        2u32,
        35u32,
        3072u32,
        3073u32,
        3081u32,
        4096u32,
    ];
}
#[cfg(feature = "precision_15")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00000038274831964152354f64,
        0.000003032692775480418f64,
        0.000005761335591579966f64,
        0.000007738900939985538f64,
        0.000008478531889290012f64,
        0.0001506255081486906f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        1339u32,
        4111u32,
        4133u32,
        4149u32,
        4155u32,
        5460u32,
    ];
}
#[cfg(feature = "precision_15")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00000012911111111111108f64,
        0.0000011421065799349512f64,
        0.0000019461163153786105f64,
        0.000002661988304093567f64,
        0.0000029009158222915043f64,
        0.00017997336653970104f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        450u32,
        3997u32,
        5124u32,
        5130u32,
        5132u32,
        6825u32,
    ];
}
#[cfg(feature = "precision_15")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00000011694510739856803f64,
        0.0000004165403170227429f64,
        0.0000025865712892212644f64,
        0.0000060155339805825235f64,
        0.00015171878731590855f64,
        0.00021430556640625f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        419u32,
        1451u32,
        6151u32,
        6180u32,
        7536u32,
        8190u32,
    ];
}
#[cfg(feature = "precision_16")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000000342477140482128f64,
        0.0000009250614250614251f64,
        0.0000018339493710203446f64,
        0.00013913132107556579f64,
        0.0002520149871782695f64,
        0.0002893745284746376f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        1203u32,
        3256u32,
        6439u32,
        9408u32,
        10526u32,
        10919u32,
    ];
}
#[cfg(feature = "precision_16")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000006131764705882354f64,
        0.0000021128450396283137f64,
        0.000014710116054158613f64,
        0.00016613024844190204f64,
        0.0003460043561724753f64,
        0.0003561820272447635f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        2125u32,
        7318u32,
        10340u32,
        11711u32,
        13539u32,
        13649u32,
    ];
}
#[cfg(feature = "precision_16")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000015948893287745185f64,
        0.0000025395920207370674f64,
        0.0000032661895618781643f64,
        0.0002620209995227384f64,
        0.00034927298444130134f64,
        0.0004276811130774405f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        5557u32,
        8873u32,
        11458u32,
        14663u32,
        15549u32,
        16380u32,
    ];
}
#[cfg(feature = "precision_17")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000001485880077369439f64,
        0.0000022969924812030075f64,
        0.0000040390597508621295f64,
        0.00000452718853638696f64,
        0.00010938909763570145f64,
        0.0005674481101857784f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        5170u32,
        7980u32,
        14209u32,
        15981u32,
        17297u32,
        21841u32,
    ];
}
#[cfg(feature = "precision_17")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.00000372074774982691f64,
        0.00000542378752886836f64,
        0.000050441834827206056f64,
        0.0006047362696080302f64,
        0.0006451062000075107f64,
        0.0007096018664226898f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        12999u32,
        19052u32,
        20862u32,
        26185u32,
        26612u32,
        27305u32,
    ];
}
#[cfg(feature = "precision_17")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000007250950570342205f64,
        0.0000034275901545506583f64,
        0.000004945588235294117f64,
        0.000005620180108463106f64,
        0.0004365559921414539f64,
        0.0008522246577848236f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        2630u32,
        12229u32,
        17680u32,
        20099u32,
        28492u32,
        32773u32,
    ];
}
#[cfg(feature = "precision_18")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.0000037853277209861693f64,
        0.000005092096450100469f64,
        0.000005491811774461028f64,
        0.0008980910301416293f64,
        0.0011162530346754617f64,
        0.0011251810973241013f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        13304u32,
        17916u32,
        19296u32,
        41268u32,
        43613u32,
        43711u32,
    ];
}
#[cfg(feature = "precision_18")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000004486362477695641f64,
        0.000005307807807807807f64,
        0.000010771927974479976f64,
        0.000011530388509768174f64,
        0.0006085044373599862f64,
        0.0014101962073879207f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        15692u32,
        18648u32,
        37931u32,
        40591u32,
        46396u32,
        54661u32,
    ];
}
#[cfg(feature = "precision_18")]
impl crate::composite_hash::BirthDayParadoxCorrection
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    const RELATIVE_ERRORS: [f64; 7usize] = [
        0f64,
        0.000012169086630256208f64,
        0.00001317503642744269f64,
        0.001698112636493116f64,
        0.010907943308155818f64,
        0.019166733849439403f64,
        0.01986437587986861f64,
    ];
    const CARDINALITIES: [u32; 7usize] = [
        0u32,
        45041u32,
        48726u32,
        50466u32,
        58272u32,
        66136u32,
        66828u32,
    ];
}
