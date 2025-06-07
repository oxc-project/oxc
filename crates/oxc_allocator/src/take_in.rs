use std::{cell::Cell, mem, num};

use crate::{Allocator, AllocatorAccessor, Box, Vec};

/// A trait to replace an existing AST node with a dummy.
pub trait TakeIn<'a>: Dummy<'a> {
    /// Replace node with a dummy.
    #[must_use]
    fn take_in<A: AllocatorAccessor<'a>>(&mut self, allocator_accessor: A) -> Self {
        let allocator = allocator_accessor.allocator();
        let dummy = Dummy::dummy(allocator);
        mem::replace(self, dummy)
    }

    /// Replace node with a boxed dummy.
    #[must_use]
    fn take_in_box<A: AllocatorAccessor<'a>>(&mut self, allocator_accessor: A) -> Box<'a, Self> {
        let allocator = allocator_accessor.allocator();
        let dummy = Dummy::dummy(allocator);
        Box::new_in(mem::replace(self, dummy), allocator)
    }
}

impl<'a, T> TakeIn<'a> for Vec<'a, T> {}

/// A trait to create a dummy AST node.
pub trait Dummy<'a>: Sized {
    /// Create a dummy node.
    fn dummy(allocator: &'a Allocator) -> Self;
}

impl<'a, T> Dummy<'a> for Option<T> {
    /// Create a dummy [`Option`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        None
    }
}

impl<'a, T: Dummy<'a>> Dummy<'a> for Box<'a, T> {
    /// Create a dummy [`Box`].
    #[inline]
    fn dummy(allocator: &'a Allocator) -> Self {
        Box::new_in(Dummy::dummy(allocator), allocator)
    }
}

impl<'a, T> Dummy<'a> for Vec<'a, T> {
    /// Create a dummy [`Vec`].
    #[inline]
    fn dummy(allocator: &'a Allocator) -> Self {
        Vec::new_in(allocator)
    }
}

impl<'a, T: Dummy<'a>> Dummy<'a> for Cell<T> {
    /// Create a dummy [`Cell`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(allocator: &'a Allocator) -> Self {
        Cell::new(Dummy::dummy(allocator))
    }
}

impl<'a> Dummy<'a> for () {
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) {}
}

impl<'a> Dummy<'a> for bool {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        false
    }
}

impl<'a> Dummy<'a> for &'a str {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        ""
    }
}

macro_rules! dummy_impl_int {
    ($ty:ident) => {
        impl<'a> Dummy<'a> for $ty {
            #[inline(always)]
            fn dummy(_allocator: &'a Allocator) -> Self {
                0
            }
        }
    };
}

dummy_impl_int!(u8);
dummy_impl_int!(u16);
dummy_impl_int!(u32);
dummy_impl_int!(u64);
dummy_impl_int!(u128);
dummy_impl_int!(usize);
dummy_impl_int!(i8);
dummy_impl_int!(i16);
dummy_impl_int!(i32);
dummy_impl_int!(i64);
dummy_impl_int!(i128);
dummy_impl_int!(isize);

macro_rules! dummy_impl_float {
    ($ty:ident) => {
        impl<'a> Dummy<'a> for $ty {
            #[inline(always)]
            fn dummy(_allocator: &'a Allocator) -> Self {
                0.0
            }
        }
    };
}

dummy_impl_float!(f32);
dummy_impl_float!(f64);

macro_rules! dummy_impl_non_zero {
    ($ty:ident) => {
        impl<'a> Dummy<'a> for num::$ty {
            #[inline(always)]
            fn dummy(_allocator: &'a Allocator) -> Self {
                Self::MIN
            }
        }
    };
}

dummy_impl_non_zero!(NonZeroU8);
dummy_impl_non_zero!(NonZeroU16);
dummy_impl_non_zero!(NonZeroU32);
dummy_impl_non_zero!(NonZeroU64);
dummy_impl_non_zero!(NonZeroU128);
dummy_impl_non_zero!(NonZeroUsize);
dummy_impl_non_zero!(NonZeroI8);
dummy_impl_non_zero!(NonZeroI16);
dummy_impl_non_zero!(NonZeroI32);
dummy_impl_non_zero!(NonZeroI64);
dummy_impl_non_zero!(NonZeroI128);
dummy_impl_non_zero!(NonZeroIsize);
