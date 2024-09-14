#[allow(clippy::wildcard_imports)]
use super::*;

/// A slice that only accepts indices of a specific type. Note that the intended
/// usage is as `IndexSlice<I, [T]>`.
///
/// This is a thin wrapper around a `[T]`, to the point where the backing  is a
/// public property (called `raw`). This is in part because I know this API is
/// not a complete mirror of Vec's (patches welcome). In the worst case, you can
/// always do what you need to the slice itself.
///
/// ## Some notes on the APIs
///
/// - Most of the Slice APIs are present.
///     - Any that aren't can be trivially accessed on the underlying `raw`
///       field, which is public.
///
/// - Apis that take or return usizes referring to the positions of items were
///   replaced with ones that take Idx.
///
/// - Apis that take `R: RangeBounds<usize>` take an
///   [`IdxRangeBounds<I>`][IdxRangeBounds], which is basically a
///   `RangeBounds<I>`.
/// - Apis that take `SliceIndex<usize>` take an
///   [`IdxSliceIndex<I>`][IdxSliceIndex], which is basically a `SliceIndex<I>`.
///
/// - Most iterator functions where `the_iter().enumerate()` would refer to
///   indices have been given `_enumerated` variants. E.g.
///   [`IndexSlice::iter_enumerated`], etc. This is because
///   `v.iter().enumerate()` would be `(usize, &T)`, but you want `(I, &T)`.
///
/// The following extensions are added:
///
/// - [`IndexSlice::indices`]: an Iterator over the indices of type `I`.
/// - Various `enumerated` iterators mentioned earlier
/// - [`IndexSlice::position`], [`IndexSlice::rposition`] as
///   `self.iter().position()` will return a `Option<usize>`
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct IndexSlice<I: Idx, T: ?Sized> {
    _marker: PhantomData<fn(&I)>,
    pub raw: T,
}

// SAFETY: Whether `IndexSlice` is `Send` depends only on the data,
// not the phantom data.
unsafe impl<I: Idx, T> Send for IndexSlice<I, [T]> where T: Send {}

impl<I: Idx, T: fmt::Debug + ?Sized> fmt::Debug for IndexSlice<I, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.raw, fmt)
    }
}
/// `IndexBox<I, [T]>`: An alias for indexed boxed slice.
pub type IndexBox<I, T> = Box<IndexSlice<I, T>>;

type SliceMapped<Iter, I, T> = iter::Map<Iter, fn(&[T]) -> &IndexSlice<I, [T]>>;
type SliceMappedMut<Iter, I, T> = iter::Map<Iter, fn(&mut [T]) -> &mut IndexSlice<I, [T]>>;

impl<I: Idx, T> IndexSlice<I, [T]> {
    /// Construct a new IdxSlice by wrapping an existing slice.
    #[inline(always)]
    pub fn new<S: AsRef<[T]> + ?Sized>(s: &S) -> &Self {
        Self::from_slice(s.as_ref())
    }

    /// Construct a new mutable IdxSlice by wrapping an existing mutable slice.
    #[inline(always)]
    pub fn new_mut<S: AsMut<[T]> + ?Sized>(s: &mut S) -> &mut Self {
        Self::from_slice_mut(s.as_mut())
    }

    /// Construct a new IdxSlice by wrapping an existing slice.
    #[inline(always)]
    pub fn from_slice(s: &[T]) -> &Self {
        // SAFETY: `IndexSlice` is a thin wrapper around `[T]` with the added marker for the index.
        unsafe { &*(core::ptr::from_ref(s) as *const Self) }
    }

    /// Construct a new mutable IdxSlice by wrapping an existing mutable slice.
    #[inline(always)]
    pub fn from_slice_mut(s: &mut [T]) -> &mut Self {
        // SAFETY: `IndexSlice` is a thin wrapper around `[T]` with the added marker for the index.
        unsafe { &mut *(core::ptr::from_mut(s) as *mut Self) }
    }

    /// Copies `self` into a new `IndexVec`.
    #[inline]
    pub fn to_vec(&self) -> IndexVec<I, T>
    where
        T: Clone,
    {
        IndexVec::from_vec(self.raw.to_vec())
    }

    /// Converts `self` into a vector without clones or allocation.
    ///
    /// The resulting vector can be converted back into a box via
    /// `IndexVec<I, T>`'s `into_boxed_slice` method.
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    pub fn into_vec(self: Box<Self>) -> IndexVec<I, T> {
        // SAFETY: Both the `IndexSlice` and the `IndexVec` are
        // thin wrappers around `[T]` and `Vec<T>` with the added marker for the index.

        unsafe {
            let len = self.len();
            let b = Box::into_raw(self);
            let xs = Vec::from_raw_parts(b.cast::<T>(), len, len);
            IndexVec::from_vec(xs)
        }
    }

    /// Returns the underlying slice.
    #[inline(always)]
    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        &mut self.raw
    }

    /// Returns the underlying slice.
    #[inline(always)]
    pub fn as_raw_slice(&self) -> &[T] {
        &self.raw
    }

    /// Returns an unsafe mutable pointer to the slice's buffer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.raw.as_mut_ptr()
    }

    /// Returns an unsafe pointer to the slice's buffer.
    ///
    /// # Panics
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    /// Return the index of the last element, or panic.
    ///
    /// # Panics
    #[inline]
    pub fn last_idx(&self) -> I {
        // TODO: should this still be a panic even when `I` has disabled
        // checking?
        assert!(!self.is_empty());
        I::from_usize(self.len() - 1)
    }

    /// Returns the length of our slice.
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns the length of our slice as an `I`.
    #[inline]
    pub fn len_idx(&self) -> I {
        I::from_usize(self.raw.len())
    }

    /// Returns true if we're empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Get a iterator over reverences to our values.
    ///
    /// See also [`IndexSlice::iter_enumerated`], which gives you indices (of the
    /// correct type) as you iterate.
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.raw.iter()
    }

    /// Get a iterator over mut reverences to our values.
    ///
    /// See also [`IndexSlice::iter_mut_enumerated`], which gives you indices (of
    /// the correct type) as you iterate.
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
        self.raw.iter_mut()
    }

    /// Similar to `self.iter().enumerate()` but with indices of `I` and not
    /// `usize`.
    #[inline(always)]
    pub fn iter_enumerated(&self) -> Enumerated<slice::Iter<'_, T>, I, &T> {
        self.raw.iter().enumerate().map(|(i, t)| (I::from_usize(i), t))
    }

    /// Get an iterator over all our indices.
    #[inline(always)]
    pub fn indices(&self) -> iter::Map<Range<usize>, fn(usize) -> I> {
        (0..self.raw.len()).map(I::from_usize)
    }

    /// Similar to `self.iter_mut().enumerate()` but with indices of `I` and not
    /// `usize`.
    #[inline(always)]
    pub fn iter_mut_enumerated(&mut self) -> Enumerated<slice::IterMut<'_, T>, I, &mut T> {
        self.raw.iter_mut().enumerate().map(|(i, t)| (I::from_usize(i), t))
    }

    /// Forwards to the slice's `sort` implementation.
    #[inline]
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.raw.sort();
    }

    /// Forwards to the slice's `sort_by` implementation.
    #[inline]
    pub fn sort_by<F: FnMut(&T, &T) -> core::cmp::Ordering>(&mut self, compare: F) {
        self.raw.sort_by(compare);
    }

    /// Forwards to the slice's `sort_by_key` implementation.
    #[inline]
    pub fn sort_by_key<F: FnMut(&T) -> K, K: Ord>(&mut self, f: F) {
        self.raw.sort_by_key(f);
    }

    /// Forwards to the slice's `sort_by_cached_key` implementation.
    #[inline]
    pub fn sort_by_cached_key<F: FnMut(&T) -> K, K: Ord>(&mut self, f: F) {
        self.raw.sort_by_cached_key(f);
    }

    /// Forwards to the slice's `sort_unstable` implementation.
    #[inline]
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.raw.sort_unstable();
    }

    /// Forwards to the slice's `sort_unstable_by` implementation.
    #[inline]
    pub fn sort_unstable_by<F: FnMut(&T, &T) -> core::cmp::Ordering>(&mut self, compare: F) {
        self.raw.sort_unstable_by(compare);
    }

    /// Forwards to the slice's `sort_unstable_by_key` implementation.
    #[inline]
    pub fn sort_unstable_by_key<F: FnMut(&T) -> K, K: Ord>(&mut self, f: F) {
        self.raw.sort_unstable_by_key(f);
    }

    /// Forwards to the slice's `ends_with` implementation.
    #[inline]
    pub fn ends_with<S: AsRef<[T]> + ?Sized>(&self, needle: &S) -> bool
    where
        T: PartialEq,
    {
        self.raw.ends_with(needle.as_ref())
    }

    /// Forwards to the slice's `starts_with` implementation.
    #[inline]
    pub fn starts_with<S: AsRef<[T]> + ?Sized>(&self, needle: &S) -> bool
    where
        T: PartialEq,
    {
        self.raw.starts_with(needle.as_ref())
    }

    /// Forwards to the slice's `contains` implementation.
    #[inline]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.raw.contains(x)
    }

    /// Forwards to the slice's `reverse` implementation.
    #[inline]
    pub fn reverse(&mut self) {
        self.raw.reverse();
    }

    /// Call `slice::binary_search` converting the indices it gives us back as
    /// needed.
    ///
    /// # Errors
    #[inline]
    pub fn binary_search(&self, value: &T) -> Result<I, I>
    where
        T: Ord,
    {
        match self.raw.binary_search(value) {
            Ok(i) => Ok(I::from_usize(i)),
            Err(i) => Err(I::from_usize(i)),
        }
    }

    /// Binary searches this sorted vec with a comparator function, converting
    /// the indices it gives us back to our Idx type.
    ///
    /// # Errors
    #[inline]
    pub fn binary_search_by<'a, F: FnMut(&'a T) -> core::cmp::Ordering>(
        &'a self,
        f: F,
    ) -> Result<I, I> {
        match self.raw.binary_search_by(f) {
            Ok(i) => Ok(I::from_usize(i)),
            Err(i) => Err(I::from_usize(i)),
        }
    }

    /// Copies all elements from `src` into `self`, using a memcpy.
    #[inline]
    pub fn copy_from_slice(&mut self, src: &Self)
    where
        T: Copy,
    {
        self.raw.copy_from_slice(&src.raw);
    }

    /// Copies the elements from `src` into `self`.
    #[inline]
    pub fn clone_from_slice(&mut self, src: &Self)
    where
        T: Clone,
    {
        self.raw.clone_from_slice(&src.raw);
    }

    /// Swaps all elements in `self` with those in `other`.
    #[inline]
    pub fn swap_with_slice(&mut self, other: &mut Self) {
        self.raw.swap_with_slice(&mut other.raw);
    }

    /// Binary searches this sorted vec with a key extraction function, converting
    /// the indices it gives us back to our Idx type.
    ///
    /// # Errors
    #[inline]
    pub fn binary_search_by_key<'a, B: Ord, F: FnMut(&'a T) -> B>(
        &'a self,
        b: &B,
        f: F,
    ) -> Result<I, I> {
        match self.raw.binary_search_by_key(b, f) {
            Ok(i) => Ok(I::from_usize(i)),
            Err(i) => Err(I::from_usize(i)),
        }
    }

    /// Searches for an element in an iterator, returning its index. This is
    /// equivalent to `Iterator::position`, but returns `I` and not `usize`.
    #[inline(always)]
    pub fn position<F: FnMut(&T) -> bool>(&self, f: F) -> Option<I> {
        self.raw.iter().position(f).map(I::from_usize)
    }

    /// Searches for an element in an iterator from the right, returning its
    /// index. This is equivalent to `Iterator::position`, but returns `I` and
    /// not `usize`.
    #[inline(always)]
    pub fn rposition<F: FnMut(&T) -> bool>(&self, f: F) -> Option<I> {
        self.raw.iter().rposition(f).map(I::from_usize)
    }

    /// Swaps two elements in our vector.
    #[inline]
    pub fn swap(&mut self, a: I, b: I) {
        self.raw.swap(a.index(), b.index());
    }

    /// Divides our slice into two at an index.
    #[inline]
    pub fn split_at(&self, a: I) -> (&Self, &Self) {
        let (a, b) = self.raw.split_at(a.index());
        (Self::new(a), Self::new(b))
    }

    /// Divides our slice into two at an index.
    #[inline]
    pub fn split_at_mut(&mut self, a: I) -> (&mut Self, &mut Self) {
        let (a, b) = self.raw.split_at_mut(a.index());
        (Self::new_mut(a), Self::new_mut(b))
    }

    /// Rotates our data in-place such that the first `mid` elements of the
    /// slice move to the end while the last `self.len() - mid` elements move to
    /// the front
    #[inline]
    pub fn rotate_left(&mut self, mid: I) {
        self.raw.rotate_left(mid.index());
    }

    /// Rotates our data in-place such that the first `self.len() - k` elements
    /// of the slice move to the end while the last `k` elements move to the
    /// front
    #[inline]
    pub fn rotate_right(&mut self, k: I) {
        self.raw.rotate_right(k.index());
    }

    /// Return the the last element, if we are not empty.
    #[inline(always)]
    pub fn last(&self) -> Option<&T> {
        self.raw.last()
    }

    /// Return the the last element, if we are not empty.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.raw.last_mut()
    }

    /// Return the the first element, if we are not empty.
    #[inline]
    pub fn first(&self) -> Option<&T> {
        self.raw.first()
    }

    /// Return the the first element, if we are not empty.
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.raw.first_mut()
    }

    /// Copies elements from one part of the slice to another part of itself,
    /// using a memmove.
    #[inline]
    pub fn copy_within<R: IdxRangeBounds<I>>(&mut self, src: R, dst: I)
    where
        T: Copy,
    {
        self.raw.copy_within(src.into_range(), dst.index());
    }

    /// Get a ref to the item at the provided index, or None for out of bounds.
    #[inline]
    pub fn get<J: IdxSliceIndex<I, T>>(&self, index: J) -> Option<&J::Output> {
        index.get(self)
    }

    /// Get a mut ref to the item at the provided index, or None for out of
    /// bounds
    #[inline]
    pub fn get_mut<J: IdxSliceIndex<I, T>>(&mut self, index: J) -> Option<&mut J::Output> {
        index.get_mut(self)
    }

    /// Wraps the underlying slice's `windows` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn windows(&self, size: usize) -> SliceMapped<slice::Windows<'_, T>, I, T> {
        self.raw.windows(size).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `chunks` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn chunks(&self, size: usize) -> SliceMapped<slice::Chunks<'_, T>, I, T> {
        self.raw.chunks(size).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `chunks_mut` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn chunks_mut(&mut self, size: usize) -> SliceMappedMut<slice::ChunksMut<'_, T>, I, T> {
        self.raw.chunks_mut(size).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `chunks_exact` iterator with one that
    /// yields `IndexSlice`s with the correct index type.
    #[inline]
    pub fn chunks_exact(&self, chunk_size: usize) -> SliceMapped<slice::ChunksExact<'_, T>, I, T> {
        self.raw.chunks_exact(chunk_size).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `chunks_exact_mut` iterator with one that
    /// yields `IndexSlice`s with the correct index type.
    #[inline]
    pub fn chunks_exact_mut(
        &mut self,
        chunk_size: usize,
    ) -> SliceMappedMut<slice::ChunksExactMut<'_, T>, I, T> {
        self.raw.chunks_exact_mut(chunk_size).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `rchunks` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rchunks(&self, size: usize) -> SliceMapped<slice::RChunks<'_, T>, I, T> {
        self.raw.rchunks(size).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `rchunks_mut` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rchunks_mut(&mut self, size: usize) -> SliceMappedMut<slice::RChunksMut<'_, T>, I, T> {
        self.raw.rchunks_mut(size).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `rchunks_exact` iterator with one that
    /// yields `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rchunks_exact(
        &self,
        chunk_size: usize,
    ) -> SliceMapped<slice::RChunksExact<'_, T>, I, T> {
        self.raw.rchunks_exact(chunk_size).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `rchunks_exact_mut` iterator with one that
    /// yields `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rchunks_exact_mut(
        &mut self,
        chunk_size: usize,
    ) -> SliceMappedMut<slice::RChunksExactMut<'_, T>, I, T> {
        self.raw.rchunks_exact_mut(chunk_size).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `split` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn split<F: FnMut(&T) -> bool>(&self, f: F) -> SliceMapped<slice::Split<'_, T, F>, I, T> {
        self.raw.split(f).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `split_mut` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn split_mut<F: FnMut(&T) -> bool>(
        &mut self,
        f: F,
    ) -> SliceMappedMut<slice::SplitMut<'_, T, F>, I, T> {
        self.raw.split_mut(f).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `rsplit` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rsplit<F: FnMut(&T) -> bool>(&self, f: F) -> SliceMapped<slice::RSplit<'_, T, F>, I, T> {
        self.raw.rsplit(f).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `rsplit_mut` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rsplit_mut<F: FnMut(&T) -> bool>(
        &mut self,
        f: F,
    ) -> SliceMappedMut<slice::RSplitMut<'_, T, F>, I, T> {
        self.raw.rsplit_mut(f).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `splitn` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn splitn<F: FnMut(&T) -> bool>(
        &self,
        n: usize,
        f: F,
    ) -> SliceMapped<slice::SplitN<'_, T, F>, I, T> {
        self.raw.splitn(n, f).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `splitn_mut` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn splitn_mut<F: FnMut(&T) -> bool>(
        &mut self,
        n: usize,
        f: F,
    ) -> SliceMappedMut<slice::SplitNMut<'_, T, F>, I, T> {
        self.raw.splitn_mut(n, f).map(IndexSlice::new_mut)
    }

    /// Wraps the underlying slice's `rsplitn` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rsplitn<F: FnMut(&T) -> bool>(
        &self,
        n: usize,
        f: F,
    ) -> SliceMapped<slice::RSplitN<'_, T, F>, I, T> {
        self.raw.rsplitn(n, f).map(IndexSlice::new)
    }

    /// Wraps the underlying slice's `rsplitn_mut` iterator with one that yields
    /// `IndexSlice`s with the correct index type.
    #[inline]
    pub fn rsplitn_mut<F: FnMut(&T) -> bool>(
        &mut self,
        n: usize,
        f: F,
    ) -> SliceMappedMut<slice::RSplitNMut<'_, T, F>, I, T> {
        self.raw.rsplitn_mut(n, f).map(IndexSlice::new_mut)
    }

    /// Create a IdxSlice from its pointer and length.
    ///
    /// # Safety
    ///
    /// This is equivalent to `core::slice::from_raw_parts` and has the same
    /// safety caveats.
    #[inline]

    pub unsafe fn from_raw_parts<'a>(data: *const T, len: usize) -> &'a Self {
        Self::new(slice::from_raw_parts(data, len))
    }

    /// Create a mutable IdxSlice from its pointer and length.
    ///
    /// # Safety
    ///
    /// This is equivalent to `core::slice::from_raw_parts_mut` and has the same
    /// safety caveats.
    #[inline]

    pub unsafe fn from_raw_parts_mut<'a>(data: *mut T, len: usize) -> &'a mut Self {
        Self::new_mut(slice::from_raw_parts_mut(data, len))
    }

    /// Returns the first and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    pub fn split_first(&self) -> Option<(&T, &IndexSlice<I, [T]>)> {
        if self.is_empty() {
            None
        } else {
            Some((&self[I::from_usize(0)], &self[I::from_usize(1)..]))
        }
    }

    /// Returns the first and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    pub fn split_first_mut(&mut self) -> Option<(&mut T, &mut IndexSlice<I, [T]>)> {
        if self.is_empty() {
            None
        } else {
            let split = self.split_at_mut(I::from_usize(1));
            Some((&mut split.0[I::from_usize(0)], split.1))
        }
    }

    /// Returns the last and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    pub fn split_last(&self) -> Option<(&T, &IndexSlice<I, [T]>)> {
        if self.is_empty() {
            None
        } else {
            let last = self.last_idx();
            Some((&self[last], &self[..last]))
        }
    }

    /// Returns the last and all the rest of the elements of the slice, or `None` if it is empty.
    #[inline]
    pub fn split_last_mut(&mut self) -> Option<(&mut T, &mut IndexSlice<I, [T]>)> {
        if self.is_empty() {
            None
        } else {
            let last = self.last_idx();
            let split = self.split_at_mut(last);
            Some((&mut split.1[I::from_usize(0)], split.0))
        }
    }
}

impl<I: Idx, A, B> PartialEq<IndexSlice<I, [B]>> for IndexSlice<I, [A]>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &IndexSlice<I, [B]>) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }

    #[inline]
    fn ne(&self, other: &IndexSlice<I, [B]>) -> bool {
        PartialEq::ne(&self.raw, &other.raw)
    }
}

impl<I: Idx, A: Eq> Eq for IndexSlice<I, [A]> {}

impl<I: Idx, A, B> PartialEq<[B]> for IndexSlice<I, [A]>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &[B]) -> bool {
        PartialEq::eq(&self.raw, other)
    }

    #[inline]
    fn ne(&self, other: &[B]) -> bool {
        PartialEq::ne(&self.raw, other)
    }
}

impl<I: Idx, T: PartialOrd> PartialOrd for IndexSlice<I, [T]> {
    #[inline]
    fn partial_cmp(&self, other: &IndexSlice<I, [T]>) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.raw, &other.raw)
    }
}

impl<I: Idx, T: core::cmp::Ord> core::cmp::Ord for IndexSlice<I, [T]> {
    #[inline]
    fn cmp(&self, other: &IndexSlice<I, [T]>) -> core::cmp::Ordering {
        core::cmp::Ord::cmp(&self.raw, &other.raw)
    }
}

impl<I: Idx, T: core::hash::Hash> core::hash::Hash for IndexSlice<I, [T]> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, h: &mut H) {
        self.raw.hash(h);
    }
}

impl<I: Idx, T> alloc::borrow::ToOwned for IndexSlice<I, [T]>
where
    T: Clone,
{
    type Owned = IndexVec<I, T>;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        IndexVec::from(self.raw.to_vec())
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a IndexSlice<I, [T]> {
    type IntoIter = slice::Iter<'a, T>;
    type Item = &'a T;

    #[inline]
    fn into_iter(self) -> slice::Iter<'a, T> {
        self.raw.iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a mut IndexSlice<I, [T]> {
    type IntoIter = slice::IterMut<'a, T>;
    type Item = &'a mut T;

    #[inline]
    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.raw.iter_mut()
    }
}

impl<I: Idx, T> Default for &IndexSlice<I, [T]> {
    #[inline]
    fn default() -> Self {
        IndexSlice::new(&[])
    }
}

impl<I: Idx, T> Default for &mut IndexSlice<I, [T]> {
    #[inline]
    fn default() -> Self {
        IndexSlice::new_mut(&mut [])
    }
}

impl<'a, I: Idx, T> From<&'a [T]> for &'a IndexSlice<I, [T]> {
    #[inline]
    fn from(a: &'a [T]) -> Self {
        IndexSlice::new(a)
    }
}

impl<'a, I: Idx, T> From<&'a mut [T]> for &'a mut IndexSlice<I, [T]> {
    #[inline]
    fn from(a: &'a mut [T]) -> Self {
        IndexSlice::new_mut(a)
    }
}

impl<I: Idx, T> From<Box<[T]>> for Box<IndexSlice<I, [T]>> {
    #[inline]
    fn from(b: Box<[T]>) -> Self {
        // SAFETY: `IndexSlice` is a thin wrapper around `[T]` with the added marker for the index.

        unsafe { Box::from_raw(Box::into_raw(b) as *mut IndexSlice<I, [T]>) }
    }
}

impl<I: Idx, A> AsRef<[A]> for IndexSlice<I, [A]> {
    #[inline]
    fn as_ref(&self) -> &[A] {
        &self.raw
    }
}

impl<I: Idx, A> AsMut<[A]> for IndexSlice<I, [A]> {
    #[inline]
    fn as_mut(&mut self) -> &mut [A] {
        &mut self.raw
    }
}

impl<I: Idx, T: Clone> Clone for Box<IndexSlice<I, [T]>> {
    #[inline]
    fn clone(&self) -> Self {
        // Suboptimal, I think.
        self.to_vec().into_boxed_slice()
    }
}

impl<I: Idx, A> FromIterator<A> for Box<IndexSlice<I, [A]>> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        iter.into_iter().collect::<IndexVec<I, _>>().into_boxed_slice()
    }
}

impl<I: Idx, A> IntoIterator for Box<IndexSlice<I, [A]>> {
    type IntoIter = vec::IntoIter<A>;
    type Item = A;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let v: IndexVec<I, A> = self.into();
        v.into_iter()
    }
}

impl<I: Idx, A> Default for Box<IndexSlice<I, [A]>> {
    #[inline(always)]
    fn default() -> Self {
        index_vec![].into()
    }
}
