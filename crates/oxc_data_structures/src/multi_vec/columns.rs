//! [`Columns`]: the columns of a [`MultiVec`], and the layout maths for addressing them.
//!
//! [`MultiVec`]: super::MultiVec

use std::{
    alloc,
    alloc::Layout,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use super::{
    fields::Fields,
    shape::{CopyArray, FieldSizeAndOffset},
};

/// The columns of a [`MultiVec`]: the single allocation holding one array (column)
/// per field of `F`, and how much of it is in use.
///
/// This is a plain *description* - the numbers `(base_ptr, len, capacity)` plus the layout
/// maths to locate any field of any element. It owns nothing (no `Drop` impl). Ownership of
/// the elements and the allocation lies with the containing type - [`MultiVec`], or the
/// iterators in the `iter` module, which embed clones of a `MultiVec`'s `Columns`. It is
/// `Clone` but deliberately not `Copy`, so every duplication site is an explicit `.clone()` call.
///
/// `Columns` values are constructed by [`empty`], and otherwise built / mutated only by
/// `MultiVec` (in the `vec` module), which upholds the invariants. The `iter` module holds
/// clones and only reads them.
///
/// # Allocation layout
///
/// All columns live in a single allocation, sorted by descending alignment (ties broken by
/// declaration order). No padding is ever needed between columns, because each earlier
/// column's element size is a multiple of its own alignment (true of every Rust type), which
/// is in turn a multiple of every later column's (smaller or equal, power of 2) alignment.
///
/// e.g. for fields `(a: u64, b: u8, c: u32)` and capacity 3, the allocation contains:
///
/// ```text
/// [a a a c c c b]
/// ```
///
/// With no padding, each column starts at `capacity * field_offsets[i]`. The layout quantities
/// used here - `field_offsets`, `element_size`, `align`, and `max_alloc_capacity` below - are
/// compile-time constants, precomputed from the field types' layouts and read from [`F::SHAPE`].
/// The allocation's total size is `capacity * element_size`, and its alignment is `align` (the
/// maximum field alignment), which the first column is aligned to.
///
/// So `Columns` can locate any field of any element with simple unchecked arithmetic:
///
/// ```text
/// field_ptr = base_ptr + capacity * field_offsets[i] + index * size_of(field i)
/// ```
///
/// Overflow is checked once, at allocation time. `capacity` never exceeds `max_alloc_capacity`,
/// which guarantees `capacity * element_size <= isize::MAX`, and every offset the hot paths
/// compute is at most `capacity * element_size`.
///
/// # Invariants
///
/// * `len <= capacity <= max_alloc_capacity`.
/// * If `capacity == 0`: `base_ptr` is dangling and aligned to `align`. No allocation exists.
/// * If `capacity > 0`: `base_ptr` points to a live allocation created by
///   [`allocate`] with the same `capacity` (so with layout [`layout_for(capacity)`]).
///
/// These invariants are deliberately *geometry only* - they say nothing about the
/// columns' contents. What the elements mean is the containing type's contract:
///
/// * [`MultiVec`]: the first `len` elements of every column are initialized.
/// * `RawIter` (in the `iter` module): elements `index..len` of every column are
///   initialized (elements before `index` have been yielded, and for `IntoIter`,
///   moved out).
///
/// [`MultiVec`]: super::MultiVec
/// [`F::SHAPE`]: Fields::SHAPE
/// [`empty`]: Columns::empty
/// [`allocate`]: Columns::allocate
/// [`layout_for(capacity)`]: Columns::layout_for
pub struct Columns<F: Fields> {
    /// Pointer to the base of the single allocation (dangling if `capacity == 0`).
    pub base_ptr: NonNull<u8>,
    /// Number of elements of every column which are in use.
    pub len: usize,
    /// Capacity of every column.
    pub capacity: usize,
    /// The columns conceptually contain `F` values (split across the columns).
    /// Whether they are *owned* is the containing type's concern.
    fields_marker: PhantomData<F>,
}

// Manual impl rather than derive, so it is not bounded on `F: Clone`.
// `Columns` is a non-owning description (see type docs), so duplicating it is valid
// regardless of `F`. It is `Clone` but deliberately not `Copy` - each duplicate is an
// independent claim on the same allocation, so duplication must be explicit
// (a `.clone()` at the site), never implicit.
impl<F: Fields> Clone for Columns<F> {
    fn clone(&self) -> Self {
        Self::new(self.base_ptr, self.len, self.capacity)
    }
}

impl<F: Fields> Columns<F> {
    /// Create a new empty [`Columns`].
    ///
    /// `len == capacity == 0`, with a dangling `base_ptr` aligned to the allocation's
    /// alignment. Trivially establishes the invariants.
    pub const fn empty() -> Self {
        let base_ptr = const { F::SHAPE.dangling_ptr() };
        Self::new(base_ptr, 0, 0)
    }

    /// Create a [`Columns`] with a fresh allocation for `capacity` elements (`len == 0`).
    ///
    /// Aborts on allocation failure.
    ///
    /// The counterpart to [`empty`], which returns unallocated [`Columns`] (`capacity == 0`).
    ///
    /// IMPORTANT: `Columns` does not own the allocation - it is not freed when `Columns` is
    /// dropped. The caller must free it by calling [`deallocate`].
    ///
    /// # SAFETY
    ///
    /// Caller must ensure:
    /// * `capacity > 0`
    /// * `capacity <= F::SHAPE.max_alloc_capacity()`
    ///
    /// [`empty`]: Self::empty
    /// [`deallocate`]: Self::deallocate
    pub unsafe fn allocate(capacity: usize) -> Self {
        debug_assert!(capacity > 0 && capacity <= F::SHAPE.max_alloc_capacity());

        // SAFETY: Caller guarantees `capacity <= max_alloc_capacity`
        let layout = unsafe { Self::layout_for(capacity) };

        // SAFETY: `layout` has non-zero size - caller guarantees `capacity > 0`, and at least
        // one field has non-zero size (`Shape::new` rejects all-ZST field sets at compile time).
        let base_ptr = unsafe { alloc::alloc(layout) };
        let Some(base_ptr) = NonNull::new(base_ptr) else {
            alloc::handle_alloc_error(layout);
        };
        Self::new(base_ptr, 0, capacity)
    }

    /// Create a new [`Columns`].
    const fn new(base_ptr: NonNull<u8>, len: usize, capacity: usize) -> Self {
        Self { base_ptr, len, capacity, fields_marker: PhantomData }
    }

    /// Free the allocation.
    ///
    /// Does not drop any elements - the caller must drop them before this call.
    ///
    /// Shared between `MultiVec`'s and `IntoIter`'s `Drop` impls, and `MultiVec::grow`.
    ///
    /// # SAFETY
    ///
    /// * `self.capacity > 0`.
    /// * The allocation must not be used after this call - neither through `self`,
    ///   nor through any copy of these `Columns`.
    pub unsafe fn deallocate(&self) {
        debug_assert!(self.capacity > 0);

        // SAFETY: `capacity > 0`, so `base_ptr` points to a live allocation created by
        // `allocate(capacity)`, hence with layout `Self::layout_for(capacity)` = `self.layout()`.
        // This layout is deterministic - the same one it was allocated with.
        unsafe { alloc::dealloc(self.base_ptr.as_ptr(), self.layout()) };
    }

    /// Get the [`Layout`] of the current allocation.
    fn layout(&self) -> Layout {
        // SAFETY: `self.capacity <= max_alloc_capacity`
        unsafe { Self::layout_for(self.capacity) }
    }

    /// Compute the layout of an allocation with capacity `capacity`.
    ///
    /// # SAFETY
    ///
    /// `capacity <= F::SHAPE.max_alloc_capacity()`.
    unsafe fn layout_for(capacity: usize) -> Layout {
        debug_assert!(capacity <= F::SHAPE.max_alloc_capacity());

        // Cannot overflow: `capacity <= max_alloc_capacity`,
        // so `capacity * element_size <= isize::MAX - (align - 1)`
        let size = const { F::SHAPE.element_size() } * capacity;

        // SAFETY: `align` is a field type's alignment, so a non-zero power of 2.
        // `size <= isize::MAX - (align - 1)` (see above), so `size` rounded up to the
        // nearest multiple of `align` does not overflow `isize::MAX`.
        unsafe { Layout::from_size_align_unchecked(size, const { F::SHAPE.align() }) }
    }

    /// Get pointers to the start of each column.
    ///
    /// If `self.capacity == 0`, the returned pointers are dangling but aligned, usable only
    /// for zero-length slices.
    #[expect(clippy::inline_always, reason = "trivial pointer arithmetic on the hot path")]
    #[inline(always)]
    pub fn slice_ptrs(&self) -> F::Array<NonNull<u8>> {
        let field_offsets = const { F::SHAPE.field_offsets() };

        // SAFETY: `base_ptr` points to an allocation with layout `self.layout()` (or is dangling
        // and aligned to `align`, if `capacity == 0`).
        // `add` stays in bounds of the allocation, and cannot overflow:
        // `field_offsets[i] <= element_size`, so `capacity * field_offsets[i]` is at most
        // `capacity * element_size` = the allocation's size, which was `<= isize::MAX` at creation.
        // (If `capacity == 0`, the offset is 0, and `base_ptr.add(0)` is trivially in bounds.)
        //
        // The resulting pointers are aligned for their field's type:
        // * `base_ptr` is aligned to `align` >= the field's alignment.
        // * Every field sorted before this one has alignment >= this field's, and size a
        //   multiple of its own alignment (per `Fields`' safety contract), hence a multiple of
        //   this field's (smaller or equal, power of 2) alignment. So `field_offsets[i]` (their
        //   sizes' sum), and therefore `capacity * field_offsets[i]`, is a multiple of this
        //   field's alignment.
        CopyArray::from_fn(|i| unsafe { self.base_ptr.add(self.capacity * field_offsets[i]) })
    }

    /// Get pointers to each field's value for the element at `index`.
    ///
    /// If `index == self.capacity`, the returned pointers are one past the end of each
    /// column - valid for pointer arithmetic, but not for reads or writes.
    ///
    /// # SAFETY
    ///
    /// `index <= self.capacity`.
    #[expect(clippy::inline_always, reason = "trivial pointer arithmetic on the hot path")]
    #[inline(always)]
    pub unsafe fn field_ptrs(&self, index: usize) -> F::Array<NonNull<u8>> {
        debug_assert!(index <= self.capacity);

        let slice_ptrs = self.slice_ptrs();
        let field_sizes = const { F::SHAPE.field_sizes() };

        // SAFETY: The column's start plus `index` elements stays within the column, or one past
        // its end when `index == capacity` (caller guarantees `index <= capacity`) - still within
        // the allocation, or one past its end.
        // Cannot overflow: the allocation's size is `<= isize::MAX`.
        // Alignment is preserved: `index * size` is a multiple of the field's alignment
        // (size is a multiple of alignment, per `Fields`' safety contract).
        CopyArray::from_fn(|i| unsafe { slice_ptrs[i].add(index * field_sizes[i]) })
    }

    /// Copy the first `len` elements of every column to another allocation.
    ///
    /// The copies are untyped (bytewise), which [`ptr::copy_nonoverlapping`] permits even for
    /// types with padding. Uninitialized (padding) bytes are propagated to the destination,
    /// landing at the same positions - so destination elements are exactly as initialized as
    /// source elements.
    ///
    /// This is a *move* of the elements, valid for any field type. A bitwise copy is only a valid
    /// *clone* for `Copy` types, so after the copy the source elements must never be used again
    /// (not read, not dropped).
    ///
    /// # SAFETY
    ///
    /// * The first `len` elements of every column must be initialized.
    /// * `dst_ptr` must point to an allocation with layout [`Self::layout_for(dst_capacity)`].
    /// * `self.len <= dst_capacity`.
    /// * `dst_ptr`'s allocation must not overlap `self`'s.
    /// * The source elements must never be used after the copy.
    ///
    /// [`Self::layout_for(dst_capacity)`]: Columns::layout_for
    pub unsafe fn copy_fields(&self, dst_ptr: NonNull<u8>, dst_capacity: usize) {
        // Copy the columns in memory order (ascending offset), not declaration order.
        // Both allocations are then walked monotonically forward, which streaming prefetchers
        // follow across column boundaries.
        //
        // With few fields, the loop fully unrolls. With many (LLVM stops fully unrolling somewhere
        // between 20 and 32 fields), it reads the pre-sorted table from constant data, with no
        // bounds checks and no data-dependent loads.
        //
        // Zero-sized field types need no special case. Their copies are zero-byte `memcpy`s -
        // free, and deleted outright by LLVM when the loop is unrolled.
        // An explicit `if field_size == 0 { continue; }` would add a data-dependent branch
        // when loop is not unrolled, which would penalize the common case of non-ZSTs.
        //
        // The counted index is deliberate. Do not "simplify" this to iterating
        // `ordered_field_sizes_and_offsets()` by value - an array's by-value `IntoIterator`
        // moves the array into the iterator, so when the loop does not fully unroll,
        // the whole table gets `memcpy`ed to the stack (512 bytes for 32 fields).
        let field_sizes_and_offsets = const { F::SHAPE.ordered_field_sizes_and_offsets() };
        for i in 0..<F::Array<usize>>::LEN {
            let FieldSizeAndOffset { size: field_size, offset: field_offset } =
                field_sizes_and_offsets[i];

            // Cannot overflow - each column is within its allocation, whose size is
            // `<= isize::MAX`, and `size * len <= size * capacity` for both allocations
            let bytes = field_size * self.len;

            // SAFETY: `self.base_ptr` points to an allocation with layout `self.layout()`, and
            // caller guarantees the first `len` elements of every column are initialized (if
            // `capacity == 0`, `base_ptr` is dangling but aligned, and `len == 0`, so `bytes == 0`,
            // for which a dangling aligned pointer is valid).
            // Caller guarantees `dst_ptr` points to a non-overlapping allocation with layout
            // `Self::layout_for(dst_capacity)` and `self.len <= dst_capacity`, so both
            // byte ranges are in bounds.
            unsafe {
                ptr::copy_nonoverlapping(
                    self.base_ptr.add(self.capacity * field_offset).as_ptr(),
                    dst_ptr.add(dst_capacity * field_offset).as_ptr(),
                    bytes,
                );
            }
        }
    }
}
