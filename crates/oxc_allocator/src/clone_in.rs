use std::cell::Cell;

use crate::{Allocator, Box, Vec};

/// A trait to explicitly clone an object into an arena allocator.
///
/// As a convention `Cloned` associated type should always be the same as `Self`,
/// It'd only differ in the lifetime, Here's an example:
///
/// ```
/// impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Struct<'old_alloc> {
///     type Cloned = Struct<'new_alloc>;
///     fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
///         Struct { a: self.a.clone_in(allocator), b: self.b.clone_in(allocator) }
///     }
/// }
/// ```
///
/// Implementations of this trait on non-allocated items usually short-circuit to `Clone::clone`;
/// However, it **isn't** guaranteed.
///
pub trait CloneIn<'new_alloc>: Sized {
    type Cloned;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned;
}

impl<'alloc, T, C> CloneIn<'alloc> for Option<T>
where
    T: CloneIn<'alloc, Cloned = C>,
{
    type Cloned = Option<C>;

    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(allocator))
    }
}

impl<'old_alloc, 'new_alloc, T, C> CloneIn<'new_alloc> for Box<'old_alloc, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    type Cloned = Box<'new_alloc, C>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Box::new_in(self.as_ref().clone_in(allocator), allocator)
    }
}

impl<'old_alloc, 'new_alloc, T, C> CloneIn<'new_alloc> for Vec<'old_alloc, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    type Cloned = Vec<'new_alloc, C>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Vec::from_iter_in(self.iter().map(|it| it.clone_in(allocator)), allocator)
    }
}

impl<'alloc, T: Copy> CloneIn<'alloc> for Cell<T> {
    type Cloned = Cell<T>;

    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        Cell::new(self.get())
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for &'old_alloc str {
    type Cloned = &'new_alloc str;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        allocator.alloc_str(self)
    }
}

macro_rules! impl_clone_in {
    ($($t:ty)*) => {
        $(
            impl<'alloc> CloneIn<'alloc> for $t {
                type Cloned = Self;
                #[inline(always)]
                fn clone_in(&self, _: &'alloc Allocator) -> Self {
                    *self
                }
            }
        )*
    }
}

impl_clone_in! {
    usize u8 u16 u32 u64 u128
    isize i8 i16 i32 i64 i128
    f32 f64
    bool char
}
