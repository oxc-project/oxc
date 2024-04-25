#![allow(
    clippy::assertions_on_constants,
    clippy::eq_op,
    clippy::uninlined_format_args,
    clippy::should_panic_without_expect,
    clippy::cast_possible_truncation
)]

use std::num::{NonZeroU16, NonZeroU32, NonZeroUsize};

oxc_index::define_index_type! {
    #[non_zero(u16)]
    pub struct NonZeroIdx16 = NonZeroU16;
}

oxc_index::define_index_type! {
    #[non_zero(u32)]
    pub struct NonZeroIdx32 = NonZeroU32;
}

oxc_index::define_index_type! {
    #[non_zero(usize)]
    pub struct NonZeroIdxSz = NonZeroUsize;
}

#[test]
#[should_panic]
fn test_idx_new_0() {
    NonZeroIdxSz::new(0);
}
