//! Arena Vec.
//!
//! Originally based on [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/ast/src/arena.rs)

// All methods which just delegate to `allocator_api2::vec::Vec` methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::{
    self,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    ops,
    ptr::NonNull,
    slice::SliceIndex,
};

use bumpalo::Bump;
#[cfg(any(feature = "serialize", test))]
use serde::{Serialize, Serializer as SerdeSerializer};

#[cfg(any(feature = "serialize", test))]
use oxc_estree::{ConcatElement, ESTree, SequenceSerializer, Serializer as ESTreeSerializer};

use crate::{Allocator, Box, vec2::Vec as InnerVecGeneric};

type InnerVec<'a, T> = InnerVecGeneric<'a, T, Bump>;

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
#[repr(transparent)]
pub struct Vec<'alloc, T>(InnerVec<'alloc, T>);

/// SAFETY: Even though `Bump` is not `Sync`, we can make `Vec<T>` `Sync` if `T` is `Sync` because:
///
/// 1. No public methods allow access to the `&Bump` that `Vec` contains (in `RawVec`),
///    so user cannot illegally obtain 2 `&Bump`s on different threads via `Vec`.
///
/// 2. All internal methods which access the `&Bump` take a `&mut self`.
///    `&mut Vec` cannot be transferred across threads, and nor can an owned `Vec` (`Vec` is not `Send`).
///    Therefore these methods taking `&mut self` can be sure they're not operating on a `Vec`
///    which has been moved across threads.
///
/// Note: `Vec` CANNOT be `Send`, even if `T` is `Send`, because that would allow 2 `Vec`s on different
/// threads to both allocate into same arena simultaneously. `Bump` is not thread-safe, and this would
/// be undefined behavior.
unsafe impl<T: Sync> Sync for Vec<'_, T> {}

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
    /// let allocator = Allocator::default();
    ///
    /// let mut vec: Vec<i32> = Vec::new_in(&allocator);
    /// assert!(vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn new_in(allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(InnerVec::new_in(allocator.bump()))
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
    /// and the capacity will always be `u32::MAX`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    ///
    /// let mut vec = Vec::with_capacity_in(10, &allocator);
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
    /// let vec_units = Vec::<()>::with_capacity_in(10, &allocator);
    /// assert_eq!(vec_units.capacity(), usize::MAX);
    /// ```
    #[inline(always)]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(InnerVec::with_capacity_in(capacity, allocator.bump()))
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
        let mut vec = InnerVec::with_capacity_in(capacity, allocator.bump());
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
        Self(vec)
    }

    /// Convert [`Vec<T>`] into [`Box<[T]>`].
    ///
    /// Any spare capacity in the `Vec` is lost.
    ///
    /// [`Box<[T]>`]: Box
    #[inline]
    pub fn into_boxed_slice(self) -> Box<'alloc, [T]> {
        let slice = self.0.into_bump_slice_mut();
        let ptr = NonNull::from(slice);
        // SAFETY: `ptr` points to a valid `[T]`.
        // Contents of the `Vec` are in an arena.
        // The returned `Box` has same lifetime as the `Vec`.
        // `Vec` is not `Drop`, so we don't need to free any unused capacity in the `Vec`.
        unsafe { Box::from_non_null(ptr) }
    }

    /// Converts [`Vec<T>`] into [`&'alloc [T]`].
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// let slice = vec.into_bump_slice();
    /// assert_eq!(slice, [1, 2, 3]);
    /// ```
    #[inline]
    pub fn into_bump_slice(self) -> &'alloc [T] {
        self.0.into_bump_slice()
    }
}

impl<'alloc, T> ops::Deref for Vec<'alloc, T> {
    type Target = InnerVec<'alloc, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'alloc, T> ops::DerefMut for Vec<'alloc, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut InnerVec<'alloc, T> {
        &mut self.0
    }
}

impl<'alloc, T> IntoIterator for Vec<'alloc, T> {
    type IntoIter = <InnerVec<'alloc, T> as IntoIterator>::IntoIter;
    type Item = T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

impl<'a, T: 'a> From<Vec<'a, T>> for Box<'a, [T]> {
    #[inline(always)]
    fn from(v: Vec<'a, T>) -> Box<'a, [T]> {
        v.into_boxed_slice()
    }
}

#[cfg(any(feature = "serialize", test))]
impl<T: Serialize> Serialize for Vec<'_, T> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(any(feature = "serialize", test))]
impl<T: ESTree> ESTree for Vec<'_, T> {
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        self.as_slice().serialize(serializer);
    }
}

#[cfg(feature = "serialize")]
impl<T: ESTree> ConcatElement for Vec<'_, T> {
    fn push_to_sequence<S: SequenceSerializer>(&self, seq: &mut S) {
        for element in self {
            seq.serialize_element(element);
        }
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
        f.debug_tuple("Vec").field(&self.0).finish()
    }
}

#[cfg(test)]
mod test {
    use super::Vec;
    use crate::Allocator;

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
    fn vec_into_boxed_slice() {
        let allocator = Allocator::default();
        let mut v = Vec::with_capacity_in(4, &allocator);
        v.push("x");
        v.push("y");
        let boxed_slice = v.into_boxed_slice();
        assert_eq!(boxed_slice.as_ref(), &["x", "y"]);
    }

    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, r#"["x"]"#);
    }

    #[test]
    fn vec_serialize_estree() {
        use oxc_estree::{CompactTSSerializer, ESTree};

        let allocator = Allocator::default();
        let mut v = Vec::new_in(&allocator);
        v.push("x");

        let mut serializer = CompactTSSerializer::default();
        v.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(s, r#"["x"]"#);
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }
}
