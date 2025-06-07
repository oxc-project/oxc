#![expect(clippy::inline_always)]

use crate::{Allocator, Box};

/// This trait works similarly to the standard library [`From`] trait.
///
/// It comes with a similar implementation containing blanket implementation for [`IntoIn`],
/// reflective implementation and a bunch of primitive conversions from Rust types to their arena equivalent.
pub trait FromIn<'a, T>: Sized {
    /// Converts to this type from the input type within the given `allocator`.
    fn from_in(value: T, allocator: &'a Allocator) -> Self;
}

/// This trait works similarly to the standard library [`Into`] trait.
/// It is similar to [`FromIn`] is reflective, A [`FromIn`] implementation also implicitly implements
/// [`IntoIn`] for the opposite type.
pub trait IntoIn<'a, T>: Sized {
    /// Converts this type into the (usually inferred) input type within the given `allocator`.
    fn into_in(self, allocator: &'a Allocator) -> T;
}

/// `FromIn` is reflective
impl<'a, T> FromIn<'a, T> for T {
    #[inline(always)]
    fn from_in(t: T, _: &'a Allocator) -> T {
        t
    }
}

/// `FromIn` implicitly implements `IntoIn`.
impl<'a, T, U> IntoIn<'a, U> for T
where
    U: FromIn<'a, T>,
{
    #[inline]
    fn into_in(self, allocator: &'a Allocator) -> U {
        U::from_in(self, allocator)
    }
}

// ---------------- Primitive allocations ----------------

impl<'a> FromIn<'a, String> for &'a str {
    #[inline(always)]
    fn from_in(value: String, allocator: &'a Allocator) -> Self {
        allocator.alloc_str(value.as_str())
    }
}

impl<'a, T> FromIn<'a, T> for Box<'a, T> {
    #[inline(always)]
    fn from_in(value: T, allocator: &'a Allocator) -> Self {
        Box::new_in(value, allocator)
    }
}

impl<'a, T> FromIn<'a, Option<T>> for Option<Box<'a, T>> {
    #[inline(always)]
    fn from_in(value: Option<T>, allocator: &'a Allocator) -> Self {
        value.map(|it| Box::new_in(it, allocator))
    }
}
