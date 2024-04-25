#![allow(
    clippy::assertions_on_constants,
    clippy::eq_op,
    clippy::uninlined_format_args,
    clippy::should_panic_without_expect,
    clippy::cast_possible_truncation
)]

use std::num::NonZeroUsize;

oxc_index::define_index_type! {
    pub struct Index = usize;
}

oxc_index::define_index_type! {
    #[non_zero]
    pub struct NonZeroIndex = NonZeroUsize;
}

#[test]
fn test_idx_new_0() {}
