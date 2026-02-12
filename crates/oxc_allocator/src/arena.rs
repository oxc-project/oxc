//! Arena type used by allocator-backed collections.
//!
//! Currently this is an alias of [`Bump`], but gives us an abstraction point so
//! collections do not need to reference `Bump` directly.

use crate::bump::Bump;

/// Arena allocation backend.
pub type Arena = Bump;
