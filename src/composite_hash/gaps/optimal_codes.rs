#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<25usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<25usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<26usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<26usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<20usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<27usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<26usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<27usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<25usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_5")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<13usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<20usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<26usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_5")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<27usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<22usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<23usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<20usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<21usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<22usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<23usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<27usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<7usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<24usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<26usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<25usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<24usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<14usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<23usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<27usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<15usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<20usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<13usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<21usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<14usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<20usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<15usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<14usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<25usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<26usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<14usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<20usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<14usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_5")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<15usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<7usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<15usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<21usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<7usize>;
}
#[cfg(feature = "precision_4")]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::Golomb<45usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<13usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<21usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<13usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<24usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<22usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<15usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<24usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<24usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<27usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<15usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<23usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_4")]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::Golomb<45usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<25usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<22usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<21usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<8usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<23usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<13usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<22usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<21usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<10usize>;
}
#[cfg(feature = "precision_5")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<14usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<17usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<24usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<22usize>;
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<13usize>;
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<11usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<12usize>;
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::ExpGolomb<18usize>;
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<16usize>;
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::ExpGolomb<23usize>;
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<19usize>;
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<7usize>;
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::ExpGolomb<9usize>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_4")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_5")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_6")]
#[cfg(test)]
impl super::PrefixFreeCode<32u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<32u8>;
}
#[cfg(feature = "precision_7")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_7")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_7")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_7")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_7")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_7")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_8")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_8")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_8")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_8")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_8")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_8")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_9")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_10")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_11")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_12")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_13")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_14")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_15")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_16")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_17")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<8u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<8u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<16u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<16u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::CurrentHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
#[cfg(feature = "precision_18")]
#[cfg(test)]
impl super::PrefixFreeCode<24u8>
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code = super::prefix_free_codes::NoPrefixCode<24u8>;
}
