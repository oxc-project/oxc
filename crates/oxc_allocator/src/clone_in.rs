use std::{alloc::Layout, cell::Cell, hash::Hash, ptr::NonNull, slice};

use crate::{Allocator, Box, HashMap, Vec};

/// A trait to explicitly clone an object into an arena allocator.
///
/// As a convention `Cloned` associated type should always be the same as `Self`,
/// It'd only differ in the lifetime, Here's an example:
///
/// ```
/// # use oxc_allocator::{Allocator, CloneIn, Vec};
/// # struct Struct<'a> {a: Vec<'a, u8>, b: u8}
///
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
    /// The type of the cloned object.
    ///
    /// This should always be `Self` with a different lifetime.
    type Cloned;

    /// Clone `self` into the given `allocator`. `allocator` may be the same one
    /// that `self` is already in.
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned;

    /// Almost identical as `clone_in`, but for some special type, it will also clone the semantic ids.
    /// Please use this method only if you make sure semantic info is synced with the ast node.
    #[inline]
    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        self.clone_in(allocator)
    }
}

impl<'alloc, T, C> CloneIn<'alloc> for Option<T>
where
    T: CloneIn<'alloc, Cloned = C>,
{
    type Cloned = Option<C>;

    fn clone_in(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(allocator))
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'alloc Allocator) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in_with_semantic_ids(allocator))
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for Box<'_, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    type Cloned = Box<'new_alloc, C>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Box::new_in(self.as_ref().clone_in(allocator), allocator)
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Box::new_in(self.as_ref().clone_in_with_semantic_ids(allocator), allocator)
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for Box<'_, [T]>
where
    T: CloneIn<'new_alloc, Cloned = C>,
{
    type Cloned = Box<'new_alloc, [C]>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let slice = self.as_ref();

        // Compile-time check that `T` and `C` have identical size and alignment - which they always will
        // with intended usage that `T` and `C` are same types, just with different lifetimes.
        // This guarantees that layout of clone is same as layout of `slice`,
        // so we can create `Layout` with `for_value`, which has no runtime checks.
        const {
            assert!(
                size_of::<C>() == size_of::<T>() && align_of::<C>() == align_of::<T>(),
                "Size and alignment of `T` and `<T as CloneIn>::Cloned` must be the same"
            );
        }
        let layout = Layout::for_value(slice);

        let dst_ptr = allocator.alloc_layout(layout).cast::<C>();

        // SAFETY: We allocated space for `slice.len()` items of type `C`, starting at `dst_ptr`.
        // Therefore, writing `slice.len()` elements to that memory region is safe.
        // `C` isn't `Drop`, and allocation is in the arena, so we don't need to worry about a panic
        // in the loop - can't lead to a memory leak.
        unsafe {
            let mut ptr = dst_ptr;
            for item in slice {
                ptr.write(item.clone_in(allocator));
                ptr = ptr.add(1);
            }
        }

        // SAFETY: We just initialized `slice.len()` x `C`s, starting at `dst_ptr`
        let new_slice = unsafe { slice::from_raw_parts_mut(dst_ptr.as_ptr(), slice.len()) };
        // SAFETY: `NonNull::from(new_slice)` produces a valid pointer. The data is in the arena.
        // Lifetime of returned `Box` matches the `Allocator` the data was allocated in.
        unsafe { Box::from_non_null(NonNull::from(new_slice)) }
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let slice = self.as_ref();

        // Compile-time check that `T` and `C` have identical size and alignment - which they always will
        // with intended usage that `T` and `C` are same types, just with different lifetimes.
        // This guarantees that layout of clone is same as layout of `slice`,
        // so we can create `Layout` with `for_value`, which has no runtime checks.
        const {
            assert!(
                size_of::<C>() == size_of::<T>() && align_of::<C>() == align_of::<T>(),
                "Size and alignment of `T` and `<T as CloneIn>::Cloned` must be the same"
            );
        }
        let layout = Layout::for_value(slice);

        let dst_ptr = allocator.alloc_layout(layout).cast::<C>();

        // SAFETY: We allocated space for `slice.len()` items of type `C`, starting at `dst_ptr`.
        // Therefore, writing `slice.len()` elements to that memory region is safe.
        // `C` isn't `Drop`, and allocation is in the arena, so we don't need to worry about a panic
        // in the loop - can't lead to a memory leak.
        unsafe {
            let mut ptr = dst_ptr;
            for item in slice {
                ptr.write(item.clone_in_with_semantic_ids(allocator));
                ptr = ptr.add(1);
            }
        }

        // SAFETY: We just initialized `slice.len()` x `C`s, starting at `dst_ptr`
        let new_slice = unsafe { slice::from_raw_parts_mut(dst_ptr.as_ptr(), slice.len()) };
        // SAFETY: `NonNull::from(new_slice)` produces a valid pointer. The data is in the arena.
        // Lifetime of returned `Box` matches the `Allocator` the data was allocated in.
        unsafe { Box::from_non_null(NonNull::from(new_slice)) }
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for Vec<'_, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
    // TODO: This lifetime bound possibly shouldn't be required.
    // https://github.com/oxc-project/oxc/pull/9656#issuecomment-2719762898
    C: 'new_alloc,
{
    type Cloned = Vec<'new_alloc, C>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        // The implementation below is equivalent to:
        // `Vec::from_iter_in(self.iter().map(|it| it.clone_in(allocator)), allocator)`
        // But `Vec::from_iter_in` is inefficient because it performs a bounds check for each item.
        // This is unnecessary in this case as we know the length of the slice with certainty.
        // This implementation takes advantage of that invariant, and skips those checks.

        let slice = self.as_slice();

        let mut vec = Vec::<C>::with_capacity_in(slice.len(), allocator);

        // SAFETY: We allocated capacity for `slice.len()` elements in `vec`.
        // Therefore, writing `slice.len()` elements to that memory region is safe.
        // `C` and `Vec` aren't `Drop`, and allocation is in the arena, so we don't need to worry about
        // a panic in this loop - can't lead to a memory leak. We just set length at the end.
        unsafe {
            let mut ptr = vec.as_mut_ptr();
            for item in slice {
                ptr.write(item.clone_in(allocator));
                ptr = ptr.add(1);
            }
            vec.set_len(slice.len());
        }

        vec
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let slice = self.as_slice();

        let mut vec = Vec::<C>::with_capacity_in(slice.len(), allocator);

        // SAFETY: We allocated capacity for `slice.len()` elements in `vec`.
        // Therefore, writing `slice.len()` elements to that memory region is safe.
        // `C` and `Vec` aren't `Drop`, and allocation is in the arena, so we don't need to worry about
        // a panic in this loop - can't lead to a memory leak. We just set length at the end.
        unsafe {
            let mut ptr = vec.as_mut_ptr();
            for item in slice {
                ptr.write(item.clone_in_with_semantic_ids(allocator));
                ptr = ptr.add(1);
            }
            vec.set_len(slice.len());
        }

        vec
    }
}

impl<'new_alloc, K, V, CK, CV> CloneIn<'new_alloc> for HashMap<'_, K, V>
where
    K: CloneIn<'new_alloc, Cloned = CK>,
    V: CloneIn<'new_alloc, Cloned = CV>,
    CK: Hash + Eq,
{
    type Cloned = HashMap<'new_alloc, CK, CV>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        // Keys in original hash map are guaranteed to be unique.
        // Unfortunately, we have no static guarantee that `CloneIn` maintains that uniqueness
        // - original keys (`K`) are guaranteed unique, but cloned keys (`CK`) might not be.
        // If we did have that guarantee, we could use the faster `insert_unique_unchecked` here.
        // `hashbrown::HashMap` also has a faster cloning method in its `Clone` implementation,
        // but those APIs are not exposed, and `Clone` doesn't support custom allocators.
        // So sadly this is a lot slower than it could be, especially for `Copy` types.
        let mut cloned = HashMap::with_capacity_in(self.len(), allocator);
        for (key, value) in self {
            cloned.insert(key.clone_in(allocator), value.clone_in(allocator));
        }
        cloned
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let mut cloned = HashMap::with_capacity_in(self.len(), allocator);
        for (key, value) in self {
            cloned.insert(
                key.clone_in_with_semantic_ids(allocator),
                value.clone_in_with_semantic_ids(allocator),
            );
        }
        cloned
    }
}

impl<'alloc, T: Copy> CloneIn<'alloc> for Cell<T> {
    type Cloned = Cell<T>;

    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        Cell::new(self.get())
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for &str {
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

#[cfg(test)]
mod test {
    use super::{Allocator, CloneIn, HashMap, Vec};

    #[test]
    fn clone_in_boxed_slice() {
        let allocator = Allocator::default();

        let mut original = Vec::from_iter_in([1, 2, 3], &allocator).into_boxed_slice();

        let cloned = original.clone_in(&allocator);
        let cloned2 = original.clone_in_with_semantic_ids(&allocator);
        original[1] = 4;

        assert_eq!(original.as_ref(), &[1, 4, 3]);
        assert_eq!(cloned.as_ref(), &[1, 2, 3]);
        assert_eq!(cloned2.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn clone_in_vec() {
        let allocator = Allocator::default();

        let mut original = Vec::with_capacity_in(8, &allocator);
        original.extend_from_slice(&[1, 2, 3]);

        let cloned = original.clone_in(&allocator);
        let cloned2 = original.clone_in_with_semantic_ids(&allocator);
        original[1] = 4;

        assert_eq!(original.as_slice(), &[1, 4, 3]);
        assert_eq!(cloned.as_slice(), &[1, 2, 3]);
        assert_eq!(cloned.capacity(), 3);
        assert_eq!(cloned2.as_slice(), &[1, 2, 3]);
        assert_eq!(cloned2.capacity(), 3);
    }

    #[test]
    fn clone_in_hash_map() {
        let allocator = Allocator::default();

        let mut original = HashMap::with_capacity_in(8, &allocator);
        original.extend(&[("x", "xx"), ("y", "yy"), ("z", "zz")]);

        let cloned = original.clone_in(&allocator);
        let cloned2 = original.clone_in_with_semantic_ids(&allocator);
        *original.get_mut("y").unwrap() = "changed";

        let mut original_as_vec = original.iter().collect::<std::vec::Vec<_>>();
        original_as_vec.sort_unstable();
        assert_eq!(original_as_vec, &[(&"x", &"xx"), (&"y", &"changed"), (&"z", &"zz")]);

        assert_eq!(cloned.capacity(), 3);
        let mut cloned_as_vec = cloned.iter().collect::<std::vec::Vec<_>>();
        cloned_as_vec.sort_unstable();
        assert_eq!(cloned_as_vec, &[(&"x", &"xx"), (&"y", &"yy"), (&"z", &"zz")]);

        assert_eq!(cloned2.capacity(), 3);
        let mut cloned2_as_vec = cloned2.iter().collect::<std::vec::Vec<_>>();
        cloned2_as_vec.sort_unstable();
        assert_eq!(cloned2_as_vec, &[(&"x", &"xx"), (&"y", &"yy"), (&"z", &"zz")]);
    }
}
