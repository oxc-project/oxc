use std::{convert::From, ops::Deref};

mod arena;

pub use arena::{Box, String, Vec};
use bumpalo::Bump;

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    pub fn into_bump(self) -> Bump {
        self.bump
    }
}

impl From<Bump> for Allocator {
    fn from(bump: Bump) -> Self {
        Self { bump }
    }
}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::Allocator;
    use bumpalo::Bump;

    #[test]
    fn test_api() {
        let bump = Bump::new();
        let allocator: Allocator = bump.into();
        #[allow(clippy::explicit_deref_methods)]
        {
            _ = allocator.deref();
        }
    }
}
