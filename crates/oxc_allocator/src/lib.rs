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

impl Allocator {
    /// Construct a new arena with the specified byte capacity to bump allocate into.
    ///
    /// ## Example
    ///
    /// ```
    /// let allocator = oxc_allocator::Allocator::with_capacity(100);
    /// # let _ = allocator;
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bump: Bump::with_capacity(capacity) }
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
