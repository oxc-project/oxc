//! Arena Vec.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/ast/src/arena.rs)

// All methods which just delegate to `allocator_api2::vec::Vec` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    self,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    mem::ManuallyDrop,
    ops,
    ptr::NonNull,
    slice::SliceIndex,
};

use allocator_api2::vec::Vec as InnerVec;
use bumpalo::Bump;
#[cfg(any(feature = "serialize", test))]
use serde::{ser::SerializeSeq, Serialize, Serializer};

use crate::{Allocator, Box};

/// A `Vec` without [`Drop`], which stores its data in the arena allocator.
///
/// # No `Drop`s
///
/// Objects allocated into Oxc memory arenas are never [`Dropped`](Drop). Memory is released in bulk
/// when the allocator is dropped, without dropping the individual objects in the arena.
///
/// Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
/// which own memory allocations outside the arena.
///
/// Static checks make this impossible to do. [`Vec::new_in`] and all other methods which create
/// a [`Vec`] will refuse to compile if called with a [`Drop`] type.
#[derive(PartialEq, Eq)]
pub struct Vec<'alloc, T>(pub(crate) ManuallyDrop<InnerVec<T, &'alloc Bump>>);

/// SAFETY: Not actually safe, but for enabling `Send` for downstream crates.
unsafe impl<T> Send for Vec<'_, T> {}
/// SAFETY: Not actually safe, but for enabling `Sync` for downstream crates.
unsafe impl<T> Sync for Vec<'_, T> {}

impl<'alloc, T> Vec<'alloc, T> {
    /// Const assertion that `T` is not `Drop`.
    /// Must be referenced in all methods which create a `Vec`.
    const ASSERT_T_IS_NOT_DROP: () =
        assert!(!std::mem::needs_drop::<T>(), "Cannot create a Vec<T> where T is a Drop type");

    /// Constructs a new, empty `Vec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let arena = Allocator::default();
    ///
    /// let mut vec: Vec<i32> = Vec::new_in(&arena);
    /// assert!(vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(ManuallyDrop::new(InnerVec::new_in(allocator.bump())))
    }

    /// Constructs a new, empty `Vec<T>` with at least the specified capacity
    /// with the provided allocator.
    ///
    /// The vector will be able to hold at least `capacity` elements without
    /// reallocating. This method is allowed to allocate for more elements than
    /// `capacity`. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// minimum *capacity* specified, the vector will have a zero *length*.
    ///
    /// For `Vec<T>` where `T` is a zero-sized type, there will be no allocation
    /// and the capacity will always be `usize::MAX`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let arena = Allocator::default();
    ///
    /// let mut vec = Vec::with_capacity_in(10, &arena);
    ///
    /// // The vector contains no items, even though it has capacity for more
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // These are all done without reallocating...
    /// for i in 0..10 {
    ///     vec.push(i);
    /// }
    /// assert_eq!(vec.len(), 10);
    /// assert_eq!(vec.capacity(), 10);
    ///
    /// // ...but this may make the vector reallocate
    /// vec.push(11);
    /// assert_eq!(vec.len(), 11);
    /// assert!(vec.capacity() >= 11);
    ///
    /// // A vector of a zero-sized type will always over-allocate, since no
    /// // allocation is necessary
    /// let vec_units = Vec::<()>::with_capacity_in(10, &arena);
    /// assert_eq!(vec_units.capacity(), usize::MAX);
    /// ```
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(ManuallyDrop::new(InnerVec::with_capacity_in(capacity, allocator.bump())))
    }

    /// Create a new [`Vec`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorially identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let capacity = hint.1.unwrap_or(hint.0);
        let mut vec = ManuallyDrop::new(InnerVec::with_capacity_in(capacity, allocator.bump()));
        vec.extend(iter);
        Self(vec)
    }

    /// Create a new [`Vec`] from a fixed-size array, allocated in the given `allocator`.
    ///
    /// This is preferable to `from_iter_in` where source is an array, as size is statically known,
    /// and compiler is more likely to construct the values directly in arena, rather than constructing
    /// on stack and then copying to arena.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    ///
    /// let array: [u32; 4] = [1, 2, 3, 4];
    /// let vec = Vec::from_array_in(array, &allocator);
    /// ```
    #[inline]
    pub fn from_array_in<const N: usize>(array: [T; N], allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let boxed = Box::new_in(array, allocator);
        let ptr = Box::into_non_null(boxed).as_ptr().cast::<T>();
        // SAFETY: `ptr` has correct alignment - it was just allocated as `[T; N]`.
        // `ptr` was allocated with correct size for `[T; N]`.
        // `len` and `capacity` are both `N`.
        // Allocated size cannot be larger than `isize::MAX`, or `Box::new_in` would have failed.
        let vec = unsafe { InnerVec::from_raw_parts_in(ptr, N, N, allocator.bump()) };
        Self(ManuallyDrop::new(vec))
    }

    /// Converts the vector into [`Box<[T]>`][owned slice].
    ///
    /// Any excess capacity the vector has will not be included in the slice.
    /// The excess memory will be leaked in the arena (i.e. not reused by another allocation).
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    /// let mut v = Vec::with_capacity_in(10, &allocator);
    /// v.extend([1, 2, 3]);
    /// let b = v.into_boxed_slice();
    ///
    /// assert_eq!(&*b, &[1, 2, 3]);
    /// assert_eq!(b.len(), 3);
    /// ```
    ///
    /// [owned slice]: Box
    #[inline]
    pub fn into_boxed_slice(self) -> Box<'alloc, [T]> {
        let inner = ManuallyDrop::into_inner(self.0);
        let slice = inner.leak();
        let ptr = NonNull::from(slice);
        // SAFETY: `ptr` points to a valid slice `[T]`.
        // `allocator_api2::vec::Vec::leak` consumes the inner `Vec` without dropping it.
        // Lifetime of returned `Box<'alloc, [T]>` is same as lifetime of consumed `Vec<'alloc, T>`,
        // so data in the `Box` must be valid for its lifetime.
        // `Vec` uniquely owned the data, and we have consumed the `Vec`, so the new `Box` has
        // unique ownership of the data (no aliasing).
        // `ptr` was created from a `&mut [T]`.
        unsafe { Box::from_non_null(ptr) }
    }
}

impl<'alloc, T> ops::Deref for Vec<'alloc, T> {
    type Target = InnerVec<T, &'alloc Bump>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T> ops::DerefMut for Vec<'alloc, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut InnerVec<T, &'alloc Bump> {
        &mut self.0
    }
}

impl<'alloc, T> IntoIterator for Vec<'alloc, T> {
    type IntoIter = <InnerVec<T, &'alloc Bump> as IntoIterator>::IntoIter;
    type Item = T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        let inner = ManuallyDrop::into_inner(self.0);
        // TODO: `allocator_api2::vec::Vec::IntoIter` is `Drop`.
        // Wrap it in `ManuallyDrop` to prevent that.
        inner.into_iter()
    }
}

impl<'i, T> IntoIterator for &'i Vec<'_, T> {
    type IntoIter = std::slice::Iter<'i, T>;
    type Item = &'i T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'i, T> IntoIterator for &'i mut Vec<'_, T> {
    type IntoIter = std::slice::IterMut<'i, T>;
    type Item = &'i mut T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T, I> ops::Index<I> for Vec<'_, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, I> ops::IndexMut<I> for Vec<'_, T>
where
    I: SliceIndex<[T]>,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[cfg(any(feature = "serialize", test))]
impl<T> Serialize for Vec<'_, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<T: Hash> Hash for Vec<'_, T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: Debug> Debug for Vec<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = &*self.0;
        f.debug_tuple("Vec").field(inner).finish()
    }
}

#[cfg(test)]
mod test {
    use super::Vec;
    use crate::{Allocator, Box};

    #[test]
    fn vec_with_capacity() {
        let allocator = Allocator::default();
        let v: Vec<i32> = Vec::with_capacity_in(10, &allocator);
        assert!(v.is_empty());
    }

    #[test]
    fn vec_debug() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = format!("{v:?}");
        assert_eq!(v, "Vec([\"x\"])");
    }

    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = serde_json::to_string(&v).unwrap();
        assert_eq!(v, "[\"x\"]");
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }

    #[test]
    fn vec_to_boxed_slice() {
        let allocator = Allocator::default();
        let mut v = Vec::with_capacity_in(10, &allocator);
        v.extend([1, 2, 3]);

        let b = v.into_boxed_slice();
        // Check return value is an `oxc_allocator::Box`, not an `allocator_api2::boxed::Box`
        let b: Box<[u8]> = b;

        assert_eq!(&*b, &[1, 2, 3]);
        // Check length of slice is equal to what `v.len()` was, not `v.capacity()`
        assert_eq!(b.len(), 3);
    }
}
