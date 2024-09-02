#[cfg(feature = "precision_4")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision4,
    crate::bits::Bits4,
> {
    type Code8 = super::prefix_free_codes::Golomb<16usize>;
    type Code16 = ();
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_5")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<12usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_5")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision5,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<12usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<11usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<11usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_6")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision6,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<11usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<10usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<10usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_7")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision7,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<10usize>;
    type Code24 = ();
    type Code32 = ();
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<9usize>;
    type Code24 = super::prefix_free_codes::Rice<18usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<9usize>;
    type Code24 = super::prefix_free_codes::Rice<18usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_8")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision8,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<9usize>;
    type Code24 = super::prefix_free_codes::Rice<17usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<8usize>;
    type Code24 = super::prefix_free_codes::Rice<17usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<8usize>;
    type Code24 = super::prefix_free_codes::Rice<17usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_9")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision9,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<7usize>;
    type Code24 = super::prefix_free_codes::Rice<16usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<7usize>;
    type Code24 = super::prefix_free_codes::Rice<16usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<7usize>;
    type Code24 = super::prefix_free_codes::Rice<15usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_10")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision10,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = super::prefix_free_codes::Rice<6usize>;
    type Code24 = super::prefix_free_codes::Rice<15usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<15usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<14usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_11")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision11,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<14usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<14usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<13usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_12")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision12,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<13usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<13usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<12usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_13")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision13,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<12usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<12usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<11usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_14")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision14,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<11usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<10usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<10usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_15")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision15,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<10usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<9usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<9usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_16")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision16,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<9usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<8usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<8usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_17")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision17,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<8usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits4,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<7usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits5,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = super::prefix_free_codes::Rice<7usize>;
    type Code32 = ();
}
#[cfg(feature = "precision_18")]
impl super::PrefixFreeCode
for crate::composite_hash::SwitchHash<
    crate::precisions::Precision18,
    crate::bits::Bits6,
> {
    type Code8 = ();
    type Code16 = ();
    type Code24 = ();
    type Code32 = super::prefix_free_codes::Rice<15usize>;
}
