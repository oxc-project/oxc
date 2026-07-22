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

#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

#[cfg(feature = "serialize")]
use oxc_estree::{ConcatElement, ESTree, SequenceSerializer, Serializer as ESTreeSerializer};

use crate::{Box, GetAllocator, arena::Arena, vec2::Vec as InnerVecGeneric};

type InnerVec<'a, T> = InnerVecGeneric<'a, T, Arena>;

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
#[derive(Eq)]
#[repr(transparent)]
pub struct Vec<'alloc, T>(InnerVec<'alloc, T>);

/// SAFETY: Even though `Arena` is not `Sync`, we can make `Vec<T>` `Sync` if `T` is `Sync` because:
///
/// 1. No public methods allow access to the `&Arena` that `Vec` contains (in `RawVec`),
///    so user cannot illegally obtain 2 `&Arena`s on different threads via `Vec`.
///
/// 2. All internal methods which access the `&Arena` take a `&mut self`.
///    `&mut Vec` cannot be transferred across threads, and nor can an owned `Vec` (`Vec` is not `Send`).
///    Therefore these methods taking `&mut self` can be sure they're not operating on a `Vec`
///    which has been moved across threads.
///
/// Note: `Vec` CANNOT be `Send`, even if `T` is `Send`, because that would allow 2 `Vec`s on different
/// threads to both allocate into same arena simultaneously. `Arena` is not thread-safe, and this would
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
    /// let allocator = &allocator;
    ///
    /// let mut vec: Vec<i32> = Vec::new_in(&allocator);
    /// assert!(vec.is_empty());
    /// ```
    #[inline(always)]
    pub fn new_in<A: GetAllocator<'alloc>>(allocator: &A) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(InnerVec::new_in(allocator.allocator().arena()))
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
    /// let allocator = &allocator;
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
    pub fn with_capacity_in<A: GetAllocator<'alloc>>(capacity: usize, allocator: &A) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self(InnerVec::with_capacity_in(capacity, allocator.allocator().arena()))
    }

    /// Create a new [`Vec`] whose elements are taken from an iterator and
    /// allocated in the given `allocator`.
    ///
    /// This is behaviorially identical to [`FromIterator::from_iter`].
    #[inline]
    pub fn from_iter_in<I: IntoIterator<Item = T>, A: GetAllocator<'alloc>>(
        iter: I,
        allocator: &A,
    ) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let capacity = hint.1.unwrap_or(hint.0);
        let mut vec = InnerVec::with_capacity_in(capacity, allocator.allocator().arena());
        vec.extend(iter);
        Self(vec)
    }

    /// Create a new [`Vec`] containing only a single value, allocated in the given `allocator`.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    /// let allocator = &allocator;
    ///
    /// let value = 123u32;
    /// let vec = Vec::from_value_in(value, &allocator);
    /// assert_eq!(vec, [123]);
    /// ```
    #[inline]
    pub fn from_value_in<A: GetAllocator<'alloc>>(value: T, allocator: &A) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let boxed = Box::new_in(value, allocator);
        let ptr = Box::into_non_null(boxed);

        // SAFETY: `ptr` contains a valid `T`, allocated in `allocator`'s arena.
        // A `Vec` with length 1, capacity 1 can own the same allocation.
        unsafe { Self::from_raw_parts_in(ptr, 1, 1, allocator) }
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
    /// let allocator = &allocator;
    ///
    /// let array: [u32; 4] = [1, 2, 3, 4];
    /// let vec = Vec::from_array_in(array, &allocator);
    /// ```
    #[inline]
    pub fn from_array_in<const N: usize, A: GetAllocator<'alloc>>(
        array: [T; N],
        allocator: &A,
    ) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        if N == 0 {
            return Vec::new_in(allocator);
        }

        let boxed = Box::new_in(array, allocator);
        let ptr = Box::into_non_null(boxed).cast::<T>();

        // SAFETY: `ptr` has correct alignment - it was just allocated as `[T; N]`.
        // `ptr` was allocated with correct size for `[T; N]`.
        // `len` and `capacity` are both `N`.
        // Allocated size cannot be larger than `isize::MAX`, or `Box::new_in` would have failed.
        unsafe { Self::from_raw_parts_in(ptr, N, N, allocator) }
    }

    /// Create a [`Vec<T>`] directly from a pointer, a length, and a capacity, allocated in the given `allocator`.
    ///
    /// # SAFETY
    ///
    /// This is highly unsafe, due to the number of invariants that aren't checked:
    ///
    /// * `ptr` needs to have been previously allocated via [`Vec<T>`] in `allocator`'s arena
    ///   (at least, it's highly likely to be incorrect if it wasn't).
    /// * `ptr`'s `T` needs to have the same size and alignment as it was allocated with.
    /// * `length` needs to be less than or equal to `capacity`.
    /// * `capacity` needs to be the capacity that the pointer was allocated with.
    /// * The memory must remain valid for the lifetime of the returned `Vec` -
    ///   i.e. the `Vec`'s lifetime must not exceed `allocator`'s.
    ///
    /// Violating these may cause problems like corrupting the allocator's internal data structures.
    /// For example it is **not** safe to build a `Vec<u8>` from a pointer to a C `char` array and
    /// a `size_t`.
    ///
    /// The ownership of `ptr` is effectively transferred to the `Vec<T>`, which may then reallocate
    /// or change the contents of the memory pointed to by the pointer at will.
    /// Ensure that nothing else uses the pointer after calling this function.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    /// use std::{mem, ptr::{self, NonNull}};
    ///
    /// let allocator = Allocator::default();
    /// let allocator = &allocator;
    ///
    /// let mut v = Vec::from_iter_in([1, 2, 3], &allocator);
    ///
    /// // Pull out the various important pieces of information about `v`
    /// let p = NonNull::new(v.as_mut_ptr()).unwrap();
    /// let len = v.len();
    /// let cap = v.capacity();
    ///
    /// let rebuilt = unsafe {
    ///     // Forget `v` so we are in complete control of the allocation to which `p` points.
    ///     mem::forget(v);
    ///
    ///     // Overwrite memory with 4, 5, 6
    ///     for i in 0..len {
    ///         p.add(i).write(4 + i);
    ///     }
    ///
    ///     // Put everything back together into a Vec
    ///     Vec::from_raw_parts_in(p, len, cap, &allocator)
    /// };
    /// assert_eq!(rebuilt, [4, 5, 6]);
    /// ```
    #[inline(always)]
    pub unsafe fn from_raw_parts_in<A: GetAllocator<'alloc>>(
        ptr: NonNull<T>,
        length: usize,
        capacity: usize,
        allocator: &A,
    ) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let arena = allocator.allocator().arena();

        // SAFETY: Caller guarantees `from_raw_parts_in`'s requirements
        let vec = unsafe { InnerVec::from_raw_parts_in(ptr, length, capacity, arena) };
        Self(vec)
    }

    /// Convert [`Vec<T>`] into [`Box<[T]>`].
    ///
    /// Any spare capacity in the `Vec` is lost.
    ///
    /// [`Box<[T]>`]: Box
    #[inline]
    pub fn into_boxed_slice(self) -> Box<'alloc, [T]> {
        let slice = self.0.into_arena_slice_mut();
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
    /// let allocator = &allocator;
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// let slice = vec.into_arena_slice();
    /// assert_eq!(slice, [1, 2, 3]);
    /// ```
    #[inline]
    pub fn into_arena_slice(self) -> &'alloc [T] {
        self.0.into_arena_slice()
    }

    /// Converts [`Vec<T>`] into [`&'alloc mut [T]`].
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::default();
    /// let allocator = &allocator;
    ///
    /// let vec = Vec::from_iter_in([1, 2, 3], &allocator);
    /// let slice = vec.into_arena_slice_mut();
    /// slice[0] = 4;
    /// assert_eq!(slice, [4, 2, 3]);
    /// ```
    #[inline]
    pub fn into_arena_slice_mut(self) -> &'alloc mut [T] {
        self.0.into_arena_slice_mut()
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

// Forward all `PartialEq` comparisons to the inner `Vec`, mirroring the set of impls it provides
// (against another `Vec`, slices, and arrays). These are implemented on the wrapper directly because
// trait resolution does not look through `Deref`.
//
// The `Vec`-vs-`Vec` impl takes the place of `#[derive(PartialEq)]`. The derive would only allow
// comparing two `Vec`s with the same element type `T`, whereas this allows comparing `Vec`s with
// different (but comparable) element types, matching the inner `Vec` and `std::vec::Vec`.
impl<T: PartialEq<U>, U> PartialEq<Vec<'_, U>> for Vec<'_, T> {
    #[inline]
    fn eq(&self, other: &Vec<'_, U>) -> bool {
        self.0 == other.0
    }
}

macro_rules! impl_slice_partial_eq {
    ($rhs:ty) => {
        impl<T: PartialEq<U>, U> PartialEq<$rhs> for Vec<'_, T> {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                self.0 == *other
            }
        }
    };
}

impl_slice_partial_eq!([U]);
impl_slice_partial_eq!(&[U]);
impl_slice_partial_eq!(&mut [U]);

macro_rules! impl_array_partial_eq {
    ($rhs:ty) => {
        impl<T: PartialEq<U>, U, const N: usize> PartialEq<$rhs> for Vec<'_, T> {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                self.0 == *other
            }
        }
    };
}

impl_array_partial_eq!([U; N]);
impl_array_partial_eq!(&[U; N]);
impl_array_partial_eq!(&mut [U; N]);

// Reverse direction: slice on the left, `Vec` on the right (e.g. `&[T] == vec`), forwarding to the
// inner `Vec`'s reverse impls. `std::vec::Vec` provides these, so mirror them here.
macro_rules! impl_slice_partial_eq_reverse {
    ($lhs:ty) => {
        impl<T: PartialEq<U>, U> PartialEq<Vec<'_, U>> for $lhs {
            #[inline]
            fn eq(&self, other: &Vec<'_, U>) -> bool {
                *self == other.0
            }
        }
    };
}

impl_slice_partial_eq_reverse!([T]);
impl_slice_partial_eq_reverse!(&[T]);
impl_slice_partial_eq_reverse!(&mut [T]);

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

#[cfg(feature = "serialize")]
impl<T: Serialize> Serialize for Vec<'_, T> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
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
    use std::cell::Cell;

    use oxc_data_structures::types::implements;

    use crate::Allocator;

    use super::Vec;

    // `Vec` must not be `Send` - 2 `Vec`s on different threads could then allocate into the same
    // arena simultaneously, which would be undefined behavior. See `unsafe impl Sync for Vec`.
    // `Vec` is `Sync` only if `T` is.
    #[test]
    fn vec_send_sync() {
        assert!(implements!(Vec<u32>: !Send));
        assert!(implements!(Vec<u32>: Sync));
        assert!(implements!(Vec<Cell<u32>>: !Sync));
    }

    #[test]
    fn vec_with_capacity() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let v: Vec<i32> = Vec::with_capacity_in(10, &allocator);
        assert!(v.is_empty());
    }

    #[test]
    fn vec_debug() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = format!("{v:?}");
        assert_eq!(v, "Vec([\"x\"])");
    }

    #[test]
    fn vec_into_boxed_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::with_capacity_in(4, &allocator);
        v.push("x");
        v.push("y");
        let boxed_slice = v.into_boxed_slice();
        assert_eq!(boxed_slice.as_ref(), &["x", "y"]);
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, r#"["x"]"#);
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn vec_serialize_estree() {
        use oxc_estree::{CompactSerializer, ESTree};

        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::new_in(&allocator);
        v.push("x");

        let mut serializer = CompactSerializer::default();
        v.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(s, r#"["x"]"#);
    }

    #[test]
    #[expect(clippy::manual_assert_eq, clippy::op_ref)]
    fn vec_partial_eq() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let v = Vec::from_array_in([1, 2, 3], &allocator);
        let same = Vec::from_array_in([1, 2, 3], &allocator);

        // `Vec` vs `Vec` (same element type), by value and by reference.
        assert!(v == same);
        assert_eq!(v, same);
        assert!(&v == &same);

        // `Vec` vs owned array `[U; N]`, and references to it.
        assert!(v == [1, 2, 3]);
        assert_eq!(v, [1, 2, 3]);
        assert!(v == &[1, 2, 3]);
        assert!(v == &mut [1, 2, 3]);

        // `Vec` vs slice `&[U]` / `&mut [U]`.
        let slice: &[i32] = &[1, 2, 3];
        assert!(v == slice);
        let mut_slice: &mut [i32] = &mut [1, 2, 3];
        assert!(v == mut_slice);

        // `Vec` vs unsized slice `[U]` (reached by dereferencing a slice reference).
        assert!(v == *slice);

        // Reverse direction: slice on the left, `Vec` on the right (std parity).
        // Note: arrays on the left (`[1, 2, 3] == v`) are not supported - `std` doesn't provide
        // `[T; N]: PartialEq<Vec>` either, only the slice forms below.
        assert!(&[1, 2, 3][..] == v);
        assert!(slice == v);
        assert!(mut_slice == v);
        assert!(*slice == v);

        // Method-call form (no auto-ref). `v.eq(slice)` resolves through the unsized `[U]` impl.
        assert!(v.eq(slice));
        assert!(v.eq(&same));
        assert!(v.eq(&[1, 2, 3]));
        assert!(slice.eq(&v));

        // Inequality still works.
        assert!(v != [1, 2, 4]);
        assert!(v != Vec::from_array_in([1, 2], &allocator));

        // Cross element type: `T: PartialEq<U>` where `T != U`.
        #[expect(clippy::items_after_statements)]
        #[derive(Clone, Copy)]
        struct Foo(u8);

        #[derive(Clone, Copy)]
        struct Bar(u8);

        impl PartialEq<Bar> for Foo {
            fn eq(&self, other: &Bar) -> bool {
                self.0 == other.0
            }
        }

        let foos = Vec::from_array_in([Foo(1), Foo(2)], &allocator);
        let bars = Vec::from_array_in([Bar(1), Bar(2)], &allocator);
        assert!(foos == bars);
        assert!(foos == [Bar(1), Bar(2)]);
        let bars_slice: &[Bar] = &[Bar(1), Bar(2)];
        assert!(foos == bars_slice);
    }

    #[test]
    fn vec_from_value_in() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::from_value_in(123u32, &allocator);
        assert_eq!(v, [123]);
        assert_eq!(v.len(), 1);
        assert_eq!(v.capacity(), 1);

        // Growing the `Vec` reallocates into the allocator, preserving the original value
        v.push(456);
        assert_eq!(v, [123, 456]);
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }
}
