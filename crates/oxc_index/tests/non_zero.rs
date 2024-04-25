#![allow(
    clippy::assertions_on_constants,
    clippy::eq_op,
    clippy::uninlined_format_args,
    clippy::should_panic_without_expect,
    clippy::cast_possible_truncation
)]

oxc_index::define_index_type! {
    #[non_zero]
    pub struct NonZeroIdxU8 = u8;
}

oxc_index::define_index_type! {
    #[non_zero]
    pub struct NonZeroIdxU16 = u16;
}

oxc_index::define_index_type! {
    #[non_zero]
    pub struct NonZeroIdxU32 = u32;
}

oxc_index::define_index_type! {
    #[non_zero]
    pub struct NonZeroIdxUsize = usize;
}

#[test]
#[should_panic]
fn test_non_zero_idx_new_0_panics() {
    NonZeroIdxU8::new(0);
    NonZeroIdxU16::new(0);
    NonZeroIdxU32::new(0);
    NonZeroIdxUsize::new(0);
}
