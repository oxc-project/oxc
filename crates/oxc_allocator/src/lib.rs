use std::ops::Deref;

mod arena;

pub use arena::{Box, String, Vec};
use bumpalo::Bump;
pub use bumpalo::vec as oxc_vec;

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}
