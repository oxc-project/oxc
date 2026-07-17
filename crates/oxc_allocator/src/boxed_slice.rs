//! Arena boxed slice.
//!
//! PROTOTYPE: [`crate::Box`] is now a 4-byte compressed pointer, and requires `T: Sized`.
//! Owned slices, which used to be `Box<'alloc, [T]>`, are this separate type instead.
//! `BoxedSlice` keeps the old fat-pointer representation (`NonNull<[T]>`, 16 bytes) -
//! compressing slices is a possible follow-up, but boxed slices never appear in AST types,
//! so there's little to gain.

use std::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

/// An owned slice without [`Drop`], which stores its data in the arena allocator.
///
/// The arena equivalent of `std::boxed::Box<[T]>`.
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Static checks make it impossible to create a `BoxedSlice` of a [`Drop`] type.
#[repr(transparent)]
pub struct BoxedSlice<'alloc, T>(NonNull<[T]>, PhantomData<(&'alloc (), T)>);

impl<T> BoxedSlice<'_, T> {
    /// Const assertion that `T` is not `Drop`.
    /// Must be referenced in all methods which create a `BoxedSlice`.
    const ASSERT_T_IS_NOT_DROP: () = assert!(
        !std::mem::needs_drop::<T>(),
        "Cannot create a BoxedSlice<T> where T is a Drop type"
    );
}

impl<T> BoxedSlice<'static, T> {
    /// Create a new empty `BoxedSlice<T>`.
    ///
    /// This method does not allocate. The returned boxed slice is represented by a dangling,
    /// correctly-aligned pointer with length 0, similar to how `Vec::new_in` produces an empty vector.
    #[inline]
    pub fn new_empty() -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        // `NonNull::<T>::dangling()` yields a non-null, properly aligned pointer.
        // We pair it with length 0 to construct a `NonNull<[T]>` representing an empty slice.
        // Correct alignment is the only requirement for it to be sound to dereference this pointer
        // to a slice, because the slice is empty.
        // See: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html
        let ptr = NonNull::dangling();
        let slice_ptr = NonNull::slice_from_raw_parts(ptr, 0);

        Self(slice_ptr, PhantomData)
    }
}

impl<'alloc, T> BoxedSlice<'alloc, T> {
    /// Create a [`BoxedSlice`] from a [`NonNull`] pointer to a slice.
    ///
    /// # SAFETY
    ///
    /// * Pointer must point to a valid `[T]`.
    /// * Pointer must point to within an `Allocator` (or be dangling with length 0).
    /// * Caller must ensure that the pointer is valid for the lifetime of the `BoxedSlice`.
    pub const unsafe fn from_non_null(ptr: NonNull<[T]>) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(ptr, PhantomData)
    }

    /// Get a [`NonNull`] pointer pointing to the [`BoxedSlice`]'s contents.
    ///
    /// The pointer is not valid for writes.
    ///
    /// The caller must ensure that the `BoxedSlice` outlives the pointer this
    /// function returns, or else it will end up dangling.
    //
    // `#[inline(always)]` because this is a no-op
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn as_non_null(boxed: &Self) -> NonNull<[T]> {
        boxed.0
    }

    /// Consume a [`BoxedSlice`] and return a [`NonNull`] pointer to its contents.
    //
    // `#[inline(always)]` because this is a no-op
    #[expect(clippy::inline_always, clippy::needless_pass_by_value)]
    #[inline(always)]
    pub fn into_non_null(boxed: Self) -> NonNull<[T]> {
        boxed.0
    }

    /// Convert a [`BoxedSlice<T>`] into slice [`&'alloc [T]`].
    ///
    /// The returned slice has the same lifetime as the allocator.
    //
    // `#[inline(always)]` because this is a no-op. `BoxedSlice<T>` and `&[T]` have the same layout.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn into_arena_slice(self) -> &'alloc [T] {
        let r = self.as_ref();
        // Extend lifetime of reference to lifetime of the allocator.
        // SAFETY: `self` is consumed by this method, so there cannot be any mutable references to it.
        // The reference lives until the allocator is dropped or reset (`'alloc` lifetime).
        // Don't need `mem::forget(self)` here, because `BoxedSlice` does not implement `Drop`.
        unsafe { mem::transmute::<&[T], &'alloc [T]>(r) }
    }

    /// Convert a [`BoxedSlice<T>`] into mutable slice [`&'alloc mut [T]`].
    ///
    /// The returned slice has the same lifetime as the allocator.
    //
    // `#[inline(always)]` because this is a no-op. `BoxedSlice<T>` and `&mut [T]` have the same layout.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn into_arena_slice_mut(mut self) -> &'alloc mut [T] {
        let r = self.as_mut();
        // Extend lifetime of reference to lifetime of the allocator.
        // SAFETY: `self` is consumed by this method, so there cannot be any other references to it.
        // The reference lives until the allocator is dropped or reset (`'alloc` lifetime).
        // Don't need `mem::forget(self)` here, because `BoxedSlice` does not implement `Drop`.
        unsafe { mem::transmute::<&mut [T], &'alloc mut [T]>(r) }
    }
}

impl<T> Deref for BoxedSlice<'_, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        // SAFETY: `self.0` is always a unique pointer to a slice allocated from an `Arena`,
        // or an empty slice created by `BoxedSlice::new_empty`
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for BoxedSlice<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        // SAFETY: `self.0` is always a unique pointer to a slice allocated from an `Arena`,
        // or an empty slice created by `BoxedSlice::new_empty`
        unsafe { self.0.as_mut() }
    }
}

impl<T> AsRef<[T]> for BoxedSlice<'_, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for BoxedSlice<'_, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Debug> Debug for BoxedSlice<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

#[cfg(feature = "serialize")]
impl<T: Serialize> Serialize for BoxedSlice<'_, T> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.deref().serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<T: ESTree> ESTree for BoxedSlice<'_, T> {
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        self.deref().serialize(serializer);
    }
}

impl<T: Hash> Hash for BoxedSlice<'_, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

#[cfg(test)]
mod test {
    use crate::{Allocator, Vec};

    use super::BoxedSlice;

    #[test]
    fn new_empty() {
        let b = BoxedSlice::<u32>::new_empty();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
        assert_eq!(&*b, &[] as &[u32]);
    }

    #[test]
    fn boxed_slice_into_arena_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let v = Vec::from_iter_in([1, 2, 3], &allocator);
        let b = v.into_boxed_slice();
        let slice = b.into_arena_slice();
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn boxed_slice_into_arena_slice_mut() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let v = Vec::from_iter_in([10, 20, 30], &allocator);
        let b = v.into_boxed_slice();
        let slice = b.into_arena_slice_mut();
        slice[1] = 99;
        assert_eq!(slice, &[10, 99, 30]);
    }
}
