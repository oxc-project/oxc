#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

mod alloc_fill;
mod alloc_try_with;
mod alloc_with;
mod allocation_limit;
mod allocator_api;
mod boxed;
mod capacity;
mod collect_in;
mod quickcheck;
mod quickchecks;
mod string;
mod tests;
mod try_alloc_try_with;
mod try_alloc_with;
mod vec;

#[cfg(feature = "serde")]
mod serde;

fn main() {}
