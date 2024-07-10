use std::{
    convert::From,
    ops::{Deref, DerefMut},
};

mod arena;
mod convert;

use bumpalo::Bump;

pub use arena::{Box, String, Vec};
pub use convert::{FromIn, IntoIn};

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
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

impl DerefMut for Allocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bump
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use bumpalo::Bump;

    use crate::Allocator;

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
