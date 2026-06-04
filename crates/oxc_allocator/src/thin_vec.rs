//! Arena-backed thin vector.
//!
//! `ThinVec` stores only one pointer in the vector object. Length and capacity live in a small
//! header allocated together with the elements in the arena.

#![expect(clippy::inline_always)]

use std::{
    alloc::Layout,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops,
    ptr::{self, NonNull},
    slice,
};

#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

#[cfg(feature = "serialize")]
use oxc_estree::{ConcatElement, ESTree, SequenceSerializer, Serializer as ESTreeSerializer};

use crate::{Allocator, CloneIn};

#[derive(Debug)]
struct Header {
    len: u32,
    cap: u32,
}

/// A pointer-sized arena vector.
///
/// `ThinVec` is useful for storing many vectors where most are empty. Empty vectors do not allocate,
/// and the vector value itself is one pointer wide. In exchange, operations that can grow the vector
/// take an [`Allocator`] argument instead of storing an allocator pointer in each vector.
#[repr(transparent)]
pub struct ThinVec<'alloc, T> {
    header: Option<NonNull<Header>>,
    _marker: PhantomData<&'alloc [T]>,
}

/// SAFETY: `ThinVec` does not expose access to the allocator it was allocated in. Shared access only
/// permits reading initialized elements.
unsafe impl<T: Sync> Sync for ThinVec<'_, T> {}

impl<'alloc, T> ThinVec<'alloc, T> {
    const ASSERT_T_IS_NOT_DROP: () =
        assert!(!std::mem::needs_drop::<T>(), "Cannot create a ThinVec<T> where T is a Drop type");

    /// Constructs a new, empty `ThinVec<T>`.
    #[inline(always)]
    pub const fn new() -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self { header: None, _marker: PhantomData }
    }

    /// Constructs a new, empty `ThinVec<T>`.
    ///
    /// The allocator is accepted for API symmetry with other arena collections. Empty `ThinVec`s do
    /// not allocate, so the argument is unused until a later growing operation.
    #[inline(always)]
    pub const fn new_in(_allocator: &'alloc Allocator) -> Self {
        Self::new()
    }

    /// Constructs a new, empty `ThinVec<T>` with at least the specified capacity.
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'alloc Allocator) -> Self {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let mut vec = Self::new();
        if capacity != 0 {
            vec.grow_to(capacity, allocator);
        }
        vec
    }

    /// Returns the number of elements in the vector.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.header().map_or(0, |header| header.len as usize)
    }

    /// Returns `true` if the vector contains no elements.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements the vector can hold without reallocating.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        if size_of::<T>() == 0 {
            return usize::MAX;
        }

        self.header().map_or(0, |header| header.cap as usize)
    }

    /// Extracts a slice containing the entire vector.
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        let Some(header) = self.header() else { return &[] };
        let len = header.len as usize;
        // SAFETY: `data_ptr` points to `cap` elements, of which `len` are initialized.
        unsafe { slice::from_raw_parts(self.data_ptr(), len) }
    }

    /// Extracts a mutable slice containing the entire vector.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let Some(header) = self.header() else { return &mut [] };
        let len = header.len as usize;
        // SAFETY: `data_mut_ptr` points to `cap` elements, of which `len` are initialized.
        unsafe { slice::from_raw_parts_mut(self.data_mut_ptr(), len) }
    }

    /// Returns an iterator over the vector.
    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns a mutable iterator over the vector.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }

    /// Appends an element to the back of the vector.
    #[inline]
    pub fn push(&mut self, value: T, allocator: &'alloc Allocator) {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let len = self.len();
        if self.header.is_none() || len == self.capacity() {
            self.grow_for_push(allocator);
        }

        // SAFETY: We just guaranteed there is capacity for one more element.
        unsafe {
            self.data_mut_ptr().add(len).write(value);
            self.set_len(len + 1);
        }
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let len = self.len();
        assert!(index < len);

        // SAFETY: `index` is in bounds. The last element is also in bounds because `len > 0`.
        unsafe {
            let ptr = self.data_mut_ptr();
            let value = ptr.add(index).read();
            let last_index = len - 1;
            if index != last_index {
                ptr.add(index).write(ptr.add(last_index).read());
            }
            self.set_len(last_index);
            value
        }
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let len = self.len();
        if len == 0 {
            return;
        }

        let slice = self.as_mut_slice();
        let mut deleted = 0usize;

        for index in 0..len {
            if !f(&slice[index]) {
                deleted += 1;
            } else if deleted != 0 {
                slice.swap(index - deleted, index);
            }
        }

        if deleted != 0 {
            // SAFETY: `len - deleted` elements remain initialized. `T` is not `Drop`, so removed
            // elements do not need dropping. If `f` panics, this code is not reached and the vector
            // keeps its original length with all slots initialized.
            unsafe {
                self.set_len(len - deleted);
            }
        }
    }

    #[inline(always)]
    fn header(&self) -> Option<&Header> {
        // SAFETY: `header` points to a valid header allocated by `grow_to`.
        self.header.map(|header| unsafe { header.as_ref() })
    }

    #[inline(always)]
    fn header_mut(&mut self) -> Option<&mut Header> {
        let header = self.header.as_mut()?;
        // SAFETY: `header` points to a valid header allocated by `grow_to`, and `&mut self`
        // guarantees unique access.
        Some(unsafe { header.as_mut() })
    }

    #[inline(always)]
    fn data_ptr(&self) -> *const T {
        if size_of::<T>() == 0 {
            return NonNull::<T>::dangling().as_ptr();
        }

        let header = self.header.expect("ThinVec has no allocation");
        let (_, offset) = Self::layout(self.capacity());
        // SAFETY: `offset` is the data offset from the allocation layout.
        unsafe { header.as_ptr().cast::<u8>().add(offset).cast::<T>() }
    }

    #[inline(always)]
    fn data_mut_ptr(&self) -> *mut T {
        self.data_ptr().cast_mut()
    }

    #[inline(always)]
    unsafe fn set_len(&mut self, len: usize) {
        let header = self.header_mut().expect("ThinVec has no allocation");
        debug_assert!(len <= header.cap as usize);
        header.len = u32::try_from(len).expect("ThinVec length overflow");
    }

    #[cold]
    #[inline(never)]
    fn grow_for_push(&mut self, allocator: &'alloc Allocator) {
        let len = self.len();
        let required_cap = len.checked_add(1).expect("ThinVec capacity overflow");
        let cap = self.capacity();
        let new_cap = if cap == 0 { Self::min_non_zero_cap() } else { cap.saturating_mul(2) };
        self.grow_to(new_cap.max(required_cap), allocator);
    }

    #[cold]
    #[inline(never)]
    fn grow_to(&mut self, capacity: usize, allocator: &'alloc Allocator) {
        let len = self.len();

        if size_of::<T>() == 0 {
            let layout = Layout::new::<Header>();
            let header = allocator.alloc_layout(layout).cast::<Header>();
            // SAFETY: `header` points to a fresh allocation for `Header`.
            unsafe {
                header.write(Header {
                    len: u32::try_from(len).expect("ThinVec length overflow"),
                    cap: u32::MAX,
                });
            }
            self.header = Some(header);
            return;
        }

        let cap = u32::try_from(capacity).expect("ThinVec capacity overflow");
        let (layout, offset) = Self::layout(capacity);
        let new_header = allocator.alloc_layout(layout).cast::<Header>();

        // SAFETY: `new_header` points to a fresh allocation with space for the header and `capacity`
        // elements. If an old allocation exists, its first `len` elements are initialized and copied
        // into the new allocation.
        unsafe {
            new_header
                .write(Header { len: u32::try_from(len).expect("ThinVec length overflow"), cap });

            if self.header.is_some() && len != 0 {
                let dst = new_header.as_ptr().cast::<u8>().add(offset).cast::<T>();
                ptr::copy_nonoverlapping(self.data_ptr(), dst, len);
            }
        }

        self.header = Some(new_header);
    }

    #[inline]
    fn layout(capacity: usize) -> (Layout, usize) {
        debug_assert!(size_of::<T>() != 0);

        let data_layout = Layout::array::<T>(capacity).expect("ThinVec capacity overflow");
        let (layout, offset) =
            Layout::new::<Header>().extend(data_layout).expect("ThinVec capacity overflow");
        (layout.pad_to_align(), offset)
    }

    #[inline(always)]
    const fn min_non_zero_cap() -> usize {
        let elem_size = size_of::<T>();
        if elem_size > (!0) / 8 { 1 } else { 4 }
    }
}

impl<T> Default for ThinVec<'_, T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ops::Deref for ThinVec<'_, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> ops::DerefMut for ThinVec<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, I> ops::Index<I> for ThinVec<'_, T>
where
    I: slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<T, I> ops::IndexMut<I> for ThinVec<'_, T>
where
    I: slice::SliceIndex<[T]>,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}

impl<'i, T> IntoIterator for &'i ThinVec<'_, T> {
    type IntoIter = slice::Iter<'i, T>;
    type Item = &'i T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'i, T> IntoIterator for &'i mut ThinVec<'_, T> {
    type IntoIter = slice::IterMut<'i, T>;
    type Item = &'i mut T;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: PartialEq> PartialEq for ThinVec<'_, T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for ThinVec<'_, T> {}

impl<T: Hash> Hash for ThinVec<'_, T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T: Debug> Debug for ThinVec<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ThinVec").field(&self.as_slice()).finish()
    }
}

impl<'new_alloc, T, C> CloneIn<'new_alloc> for ThinVec<'_, T>
where
    T: CloneIn<'new_alloc, Cloned = C>,
    C: 'new_alloc,
{
    type Cloned = ThinVec<'new_alloc, C>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let mut cloned = ThinVec::with_capacity_in(self.len(), allocator);
        for item in self {
            cloned.push(item.clone_in(allocator), allocator);
        }
        cloned
    }

    fn clone_in_with_semantic_ids(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let mut cloned = ThinVec::with_capacity_in(self.len(), allocator);
        for item in self {
            cloned.push(item.clone_in_with_semantic_ids(allocator), allocator);
        }
        cloned
    }
}

#[cfg(feature = "serialize")]
impl<T: Serialize> Serialize for ThinVec<'_, T> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<T: ESTree> ESTree for ThinVec<'_, T> {
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        self.as_slice().serialize(serializer);
    }
}

#[cfg(feature = "serialize")]
impl<T: ESTree> ConcatElement for ThinVec<'_, T> {
    fn push_to_sequence<S: SequenceSerializer>(&self, seq: &mut S) {
        for element in self {
            seq.serialize_element(element);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::ThinVec;
    use crate::{Allocator, CloneIn};

    #[test]
    fn thin_vec_is_pointer_sized() {
        assert_eq!(size_of::<ThinVec<'static, u32>>(), size_of::<usize>());
    }

    #[test]
    fn push_and_grow() {
        let allocator = Allocator::default();
        let mut vec = ThinVec::new();

        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);

        for value in 0..6 {
            vec.push(value, &allocator);
        }

        assert_eq!(vec.as_slice(), &[0, 1, 2, 3, 4, 5]);
        assert_eq!(vec.capacity(), 8);
    }

    #[test]
    fn zero_sized_type_capacity_and_push() {
        let allocator = Allocator::default();
        let mut vec = ThinVec::new();

        assert_eq!(vec.capacity(), usize::MAX);

        vec.push((), &allocator);
        vec.push((), &allocator);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.capacity(), usize::MAX);
    }

    #[test]
    fn swap_remove_and_retain() {
        let allocator = Allocator::default();
        let mut vec = ThinVec::new();
        for value in 0..5 {
            vec.push(value, &allocator);
        }

        assert_eq!(vec.swap_remove(1), 1);
        assert_eq!(vec.as_slice(), &[0, 4, 2, 3]);

        vec.retain(|value| value % 2 == 0);
        assert_eq!(vec.as_slice(), &[0, 4, 2]);
    }

    #[test]
    fn retain_is_panic_safe() {
        let allocator = Allocator::default();
        let mut vec = ThinVec::new();
        for value in 0..4 {
            vec.push(value, &allocator);
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vec.retain(|value| {
                assert_ne!(*value, 2, "stop");
                *value != 0
            });
        }));

        assert!(result.is_err());
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.as_slice(), &[1, 0, 2, 3]);
    }

    #[test]
    fn clone_in_copies_elements() {
        let allocator = Allocator::default();
        let mut vec = ThinVec::new();
        vec.push(1u32, &allocator);
        vec.push(2, &allocator);

        let cloned = vec.clone_in(&allocator);
        assert_eq!(cloned.as_slice(), &[1, 2]);
    }
}
