#![expect(
    clippy::allow_attributes,
    clippy::cast_possible_truncation,
    clippy::dbg_macro,
    clippy::explicit_into_iter_loop,
    clippy::items_after_statements,
    clippy::large_enum_variant,
    clippy::large_stack_arrays,
    clippy::legacy_numeric_constants,
    clippy::manual_repeat_n,
    clippy::match_like_matches_macro,
    clippy::match_same_arms,
    clippy::needless_collect,
    clippy::needless_pass_by_value,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::ptr_as_ptr,
    clippy::ptr_cast_constness,
    clippy::ptr_offset_by_literal,
    clippy::redundant_closure,
    clippy::ref_as_ptr,
    clippy::undocumented_unsafe_blocks,
    clippy::uninlined_format_args
)]
// `#[ignore]` attrs only exist when `debug_assertions` is enabled
#![cfg_attr(debug_assertions, expect(clippy::ignore_without_reason))]

mod alloc_fill;
mod alloc_try_with;
mod alloc_with;
mod allocation_limit;
mod capacity;
mod quickcheck;
mod quickchecks;
mod tests;
mod try_alloc_try_with;
mod try_alloc_with;

fn main() {}
