use std::ops::Deref;

mod arena;

pub use arena::{Box, String, Vec};
use bumpalo::Bump;

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

// SAFETY: Make Bump Sync and Send, it's our responsibility to never
// simultaneously mutate across threads.
unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}
