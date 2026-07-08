//! [`MultiVec`]: a growable struct-of-arrays vector backed by a single allocation,
//! indexed by a typed index.
//!
//! The allocation and its layout maths live in [`Columns`]. Field arrays are stored in
//! descending alignment order, which packs them with no padding. This makes all offsets
//! simple multiples of compile-time constants, so hot paths use plain unchecked arithmetic,
//! justified by overflow checks performed once at allocation time (in the cold grow / clone paths).

#![expect(
    rustdoc::private_intra_doc_links,
    reason = "items are only reachable via `#[doc(hidden)]`, so links to private items are \
    only ever seen in internal docs (`--document-private-items`), where they resolve"
)]

use std::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
};

use oxc_index::Idx;

use crate::assert_unchecked;

use super::{
    clone::SrcAndDstPtrs,
    columns::Columns,
    fields::{CloneFields, Fields, SliceFields},
    iter::{IntoIter, Iter, IterMut},
    shape::CopyArray,
};

/// Assumed memory page size (4 KiB - the norm on x86_64 and most platforms).
///
/// Used only to pick allocation-friendly sizes in [`MultiVec::normalize_capacity`],
/// not for correctness. On 16 KiB-page platforms (e.g. Apple Silicon) the system allocator
/// is class-based, where 4 KiB multiples remain good request sizes.
const PAGE_SIZE: usize = 4096;

/// [`MultiVec::normalize_capacity`] rounds up to page multiples with a bit-mask,
/// which requires `PAGE_SIZE` to be a power of 2.
const _: () = assert!(PAGE_SIZE.is_power_of_two());

/// Allocator metadata reserve, subtracted from page-multiple allocation targets.
///
/// glibc's `malloc` serves large allocations (above its mmap threshold, 128 KiB by default)
/// with a dedicated `mmap`, page-granular, and stores a 16-byte header inline before the
/// data - so a request of exactly N pages consumes N + 1. Requesting 16 bytes short of a
/// page multiple avoids the spill.
///
/// Class-based allocators (mimalloc, jemalloc, macOS's) store metadata out-of-band -
/// there the 16 bytes are merely left unused, negligible at these sizes.
const ALLOCATOR_METADATA_SIZE: usize = 16;

/// A growable struct-of-arrays vector, backed by a single allocation, indexed by
/// a typed index `I`.
///
/// Instead of N separate `IndexVec`s each with their own `ptr + len + capacity`,
/// `MultiVec` stores one array (column) per field of `F` within a single allocation,
/// with a single `len` and `capacity`.
///
/// Usually used via the [`multi_vec!`] macro, which generates a named wrapper around
/// a `MultiVec`, and the [`Fields`] impl that `MultiVec` requires.
///
/// The columns - the allocation, its layout, and the pointer arithmetic to address any field
/// of any element - are described by [`Columns`]. `MultiVec` wraps a `Columns` with *ownership*
/// (of the elements and the allocation) and the typed index `I`. [`Fields`] is only a thin
/// typed translation layer over the field pointers that `Columns` computes.
///
/// # Invariants
///
/// * `columns`' invariants hold (see [`Columns`]) - they are geometry only.
/// * `columns.capacity <= Self::MAX_CAPACITY` (tightens `Columns`' allocation-size limit
///   with the index type's range).
/// * The first `len` elements of every column are initialized (`Columns` itself is
///   contents-agnostic).
///
/// Field types may need dropping (e.g. `String`), so [`Drop`] drops the first `len` elements
/// of every column before freeing the allocation. Elements only ever exist as scattered field
/// values, so `F`'s own `Drop` impl (if any) is never invoked - only the individual field
/// values are dropped.
///
/// [`new`] is the only constructor, and trivially establishes the invariants ([`Columns::empty`]).
/// The invariants are preserved by every method:
///
/// * [`grow`] is the only method that changes `capacity` or `base_ptr`. It panics unless
///   the new capacity is `<= MAX_CAPACITY`, allocates, moves the `len` initialized elements
///   to the new allocation (a bitwise copy, after which the old copies are dead), and only
///   then updates `base_ptr` and `capacity` - so `self` is untouched if allocation fails or panics.
/// * [`push`] is the only method that increases `len`. It initializes element `len` in
///   every column before incrementing `len`.
/// * [`clone`] builds a fresh `MultiVec`, setting the clone's `len` only after
///   [`CloneFields::clone_columns`] has initialized all its elements (per [`CloneFields`]' contract).
/// * [`into_iter`] transfers ownership of the elements and allocation to an [`IntoIter`]
///   (which snapshots the `Columns`), then forgets the `MultiVec` without running its `Drop`.
///   The `IntoIter` reads / drops the elements directly, and frees the allocation
///   ([`Columns::deallocate`]) when dropped.
///
/// Every unsafe operation's SAFETY comment argues from these invariants.
///
/// [`multi_vec!`]: super::multi_vec
/// [`Drop`]: MultiVec::drop
/// [`new`]: MultiVec::new
/// [`grow`]: MultiVec::grow
/// [`push`]: MultiVec::push
/// [`clone`]: MultiVec::clone
/// [`into_iter`]: IntoIterator::into_iter
pub struct MultiVec<I: Idx, F: Fields> {
    /// The columns: the allocation, `len`, and `capacity`.
    columns: Columns<F>,
    /// `I` is a branding parameter only - no `I` is ever stored (indices round-trip through
    /// `usize` at the API boundary), so this marker has no bearing on `Send`/`Sync` (decided by
    /// the manual impls below).
    ///
    /// It is `fn(I) -> I`, not `PhantomData<I>`, to:
    ///
    /// * keep `MultiVec` invariant in `I` - the correct variance for a brand used in both
    ///   argument and return position, load-bearing once index types carry lifetimes,
    /// * own no `I`, for drop-check,
    /// * leave the other auto traits (`Unpin` etc.) independent of `I`.
    index_marker: PhantomData<fn(I) -> I>,
}

// SAFETY: `MultiVec` owns its data, and its API upholds Rust's aliasing rules (shared data is
// only exposed via `&self` methods, exclusive via `&mut self`). So it is `Send`/`Sync` whenever
// the field types are, like a `Vec` of each.
//
// The bound is on `F` only, not `I`. No `I` is ever stored, so sending a `MultiVec` transfers no
// `I` across threads - indices are only built from a `usize` (`push`, `iter_ids`) or turned back
// into one (`get`), always on the calling thread. `!Send` forbids *moving* an existing value to
// another thread, not *creating* a fresh one there, and no `I` ever crosses the boundary - so
// `I: !Send` cannot be broken. `I: Send` would be needlessly restrictive.
unsafe impl<I: Idx, F: Fields + Send> Send for MultiVec<I, F> {}

// SAFETY: See `Send` impl above
unsafe impl<I: Idx, F: Fields + Sync> Sync for MultiVec<I, F> {}

impl<I: Idx, F: Fields> MultiVec<I, F> {
    /// Maximum capacity.
    ///
    /// Capacity is limited by 2 factors:
    ///
    /// 1. All valid indices must be representable as `I`: `I::MAX + 1`.
    /// 2. Allocations cannot exceed `isize::MAX` bytes: [`F::SHAPE.max_alloc_capacity()`].
    ///
    /// The 2nd limit comes into play on 32-bit platforms (e.g. WASM), and could in theory on
    /// 64-bit too if field types are massive.
    ///
    /// `MAX_CAPACITY <= isize::MAX`, so `capacity * 2` in `grow` cannot overflow.
    ///
    /// The `saturating_add` just avoids overflow when `I::MAX == usize::MAX` -
    /// `max_alloc_capacity` is necessarily far lower in that case.
    ///
    /// [`F::SHAPE.max_alloc_capacity()`]: super::shape::Shape::max_alloc_capacity
    pub const MAX_CAPACITY: usize = min(I::MAX.saturating_add(1), F::SHAPE.max_alloc_capacity());

    /// Maximum capacity whose allocation fills one page or less.
    ///
    /// Above this is where [`normalize_capacity`] switches from power-of-2 to page-multiple allocation targets.
    ///
    /// 0 if a single element exceeds a page (the power-of-2 branch is then unreachable).
    ///
    /// [`normalize_capacity`]: Self::normalize_capacity
    const MAX_CAPACITY_UNDER_ONE_PAGE: usize = PAGE_SIZE / F::SHAPE.element_size();

    /// Minimum capacity after a [`grow`] operation ([`push`] or [`reserve`]).
    ///
    /// Depends on element size, like `Vec`'s minimum (same tiers as std's `RawVec::MIN_NON_ZERO_CAP`) -
    /// skip the tiny capacities where reallocation is inevitable, without demanding multi-element
    /// allocations for huge elements.
    ///
    /// This minimum does not apply to [`with_capacity`].
    ///
    /// [`grow`]: Self::grow
    /// [`push`]: Self::push
    /// [`reserve`]: Self::reserve
    /// [`with_capacity`]: Self::with_capacity
    const MIN_GROW_CAPACITY: usize = {
        let element_size = F::SHAPE.element_size();

        let capacity = if element_size == 1 {
            8
        } else if element_size <= 1024 {
            4
        } else {
            1
        };

        // Clamp to max capacity. Could be necessary when range of index type is very small.
        let capacity = min(capacity, Self::MAX_CAPACITY);

        // This assertion is infallible. Only here because `normalize_capacity` relies on this invariant.
        assert!(
            capacity <= Self::MAX_CAPACITY_UNDER_ONE_PAGE || Self::MAX_CAPACITY_UNDER_ONE_PAGE == 0
        );

        capacity
    };

    /// Create a new empty `MultiVec`.
    ///
    /// Does not allocate.
    ///
    /// Field sets consisting only of zero-sized types are rejected at compile time, when the
    /// table is defined. The [`multi_vec!`] macro's [`Fields`] impl gives `F::SHAPE` a concrete
    /// type, so rustc eagerly evaluates it, and [`Shape::new`] panics on an all-ZST field set
    /// (see the macro's docs for an example).
    ///
    /// [`multi_vec!`]: super::multi_vec
    /// [`Fields`]: super::fields::Fields
    /// [`Shape::new`]: super::shape::Shape::new
    pub const fn new() -> Self {
        Self { columns: Columns::empty(), index_marker: PhantomData }
    }

    /// Create a new `MultiVec` with capacity for at least `capacity` elements.
    ///
    /// Does not allocate if `capacity == 0`.
    ///
    /// The actual capacity can exceed the request - it is rounded up to fill the allocation
    /// ([`normalize_capacity`]), and clamped to [`MAX_CAPACITY`].
    ///
    /// # Panics
    ///
    /// Panics if `capacity` exceeds [`MAX_CAPACITY`].
    ///
    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
    /// [`normalize_capacity`]: Self::normalize_capacity
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        assert!(capacity <= Self::MAX_CAPACITY, "`capacity` exceeds maximum capacity");

        // Round up capacity to what the allocator would serve for the request.
        // `APPLY_MIN_GROW: false`: a request below `MIN_GROW_CAPACITY` is honored as-is (except rounded up) -
        // `MIN_GROW_CAPACITY` applies for `grow()` only.
        //
        // SAFETY: `0 < capacity <= MAX_CAPACITY` (zero returned early, maximum checked above)
        let capacity = unsafe { Self::normalize_capacity::<false>(capacity) };

        // SAFETY: `0 < capacity <= MAX_CAPACITY <= MAX_ALLOC_CAPACITY`
        // (`normalize_capacity` returns at least its non-zero input, at most `MAX_CAPACITY`)
        let columns = unsafe { Columns::<F>::allocate(capacity) };
        Self { columns, index_marker: PhantomData }
    }

    /// Returns the number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        let len = self.columns.len;
        // Communicate the bound on the returned value to compiler
        // (e.g. so `a.len() + b.len()` is provably overflow-free).
        // SAFETY: `len <= capacity <= MAX_CAPACITY`.
        unsafe { assert_unchecked!(len <= Self::MAX_CAPACITY) };
        len
    }

    /// Returns `true` if there are no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.columns.len == 0
    }

    /// Returns the number of elements the `MultiVec` can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        let capacity = self.columns.capacity;
        // Communicate the bound on the returned value to compiler.
        // SAFETY: `capacity <= MAX_CAPACITY`.
        unsafe { assert_unchecked!(capacity <= Self::MAX_CAPACITY) };
        capacity
    }

    /// Get a clone of the [`Columns`].
    ///
    /// Used by the iterators (in the `iter` module) to snapshot the [`MultiVec`]'s state.
    #[inline]
    pub(super) fn columns(&self) -> Columns<F> {
        self.columns.clone()
    }

    /// Push a new element (splitting its fields across the columns).
    /// Returns the ID of the new element.
    ///
    /// # Panics
    ///
    /// Panics if the new length would exceed [`MAX_CAPACITY`].
    ///
    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
    #[inline]
    pub fn push(&mut self, value: F) -> I {
        // If full, grow. Growth is `#[cold] #[inline(never)]` as it's a rare event.
        // Then fall through to the single shared write.
        //
        // Keep the write on this one path. Do NOT complete the push inside a cold
        // "grow, then write" continuation that takes `value`. Handing `value` to a separate
        // function stages it to the stack every iteration, and stops the compiler holding
        // `len` / `capacity` in registers - it reloads them from memory each iteration instead.
        //
        // As written, a bulk-push loop keeps `len` / `capacity` in registers (reloading only
        // after a grow) and stores `value`'s fields straight from registers into the heap,
        // rather than writing to the stack then copying to the heap. `value` rides through the
        // rare grow call in callee-saved registers, so it costs no hot-path spill.
        if self.columns.len == self.columns.capacity {
            // SAFETY: Just checked `len == capacity`
            unsafe { self.grow_for_push() };
        }

        // SAFETY: If it was full, `grow_for_push` above grew the allocation.
        // Either way there's now space for `value` (`len < capacity`).
        unsafe { self.push_unchecked(value) }
    }

    /// Grow the allocation to hold at least 1 more element when the current allocation is full.
    ///
    /// # SAFETY
    ///
    /// `self.columns.len` must be equal to `self.columns.capacity`.
    ///
    /// # Panics
    ///
    /// Panics if capacity is already [`MAX_CAPACITY`].
    ///
    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
    #[cold]
    #[inline(never)]
    unsafe fn grow_for_push(&mut self) {
        // Inform compiler that `len == capacity`.
        // If `grow` is inlined, compiler can treat the 2 as equivalent, rather than reading both.
        // SAFETY: Caller guarantees `len == capacity`
        unsafe { assert_unchecked!(self.columns.len == self.columns.capacity) };

        assert!(self.columns.capacity < Self::MAX_CAPACITY, "Maximum capacity exceeded");

        let min_capacity = self.columns.capacity + 1;
        // SAFETY:
        // `min_capacity > self.columns.capacity`.
        // `capacity < MAX_CAPACITY` therefore `min_capacity <= MAX_CAPACITY`
        unsafe { self.grow(min_capacity) };
    }

    /// Push a new element, without checking capacity.
    ///
    /// # SAFETY
    ///
    /// Caller must ensure that `self.len < self.capacity`.
    #[expect(clippy::inline_always, reason = "trivial writes on the hot path")]
    #[inline(always)]
    unsafe fn push_unchecked(&mut self, value: F) -> I {
        debug_assert!(self.columns.len < self.columns.capacity);

        let index = self.columns.len;

        // SAFETY: `index = len < capacity` per this function's safety contract, so
        // `field_ptrs` returns pointers to the element's (uninitialized) field slots
        // within the allocation, valid for writes and aligned, as `F::write` requires.
        unsafe { F::write(value, self.columns.field_ptrs(index)) };

        self.columns.len = index + 1;

        // SAFETY: `index < capacity <= MAX_CAPACITY <= I::MAX + 1`, so `index <= I::MAX`
        unsafe { I::from_usize_unchecked(index) }
    }

    /// Reserve capacity for at least `additional` more elements.
    ///
    /// # Panics
    ///
    /// Panics if the required capacity exceeds [`MAX_CAPACITY`] (which includes the case
    /// where it overflows `usize`).
    ///
    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        // Equivalent to `len + additional > capacity`, but a single check that cannot overflow.
        // The subtraction cannot underflow (`len <= capacity` always), and if `len + additional`
        // would overflow `usize`, then `additional` necessarily exceeds `capacity - len`.
        if additional > self.columns.capacity - self.columns.len {
            // SAFETY: Just checked `additional > capacity - len`
            unsafe { self.grow_for_reserve(additional) };
        }
    }

    /// Grow the allocation to hold at least `additional` more elements.
    ///
    /// # SAFETY
    ///
    /// `additional` must be `> self.columns.capacity - self.columns.len` - i.e. there is not
    /// enough spare capacity for `additional` more elements.
    ///
    /// # Panics
    ///
    /// Panics if the required capacity (`len + additional`) exceeds [`MAX_CAPACITY`].
    ///
    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
    #[cold]
    #[inline(never)]
    unsafe fn grow_for_reserve(&mut self, additional: usize) {
        // This check detects both `len + additional` exceeding `MAX_CAPACITY` and overflowing `usize`.
        // The subtraction cannot underflow: `len <= MAX_CAPACITY`.
        assert!(additional <= Self::MAX_CAPACITY - self.columns.len, "Maximum capacity exceeded");

        // Cannot overflow: `additional <= MAX_CAPACITY - len` (checked above)
        let min_capacity = self.columns.len + additional;
        // SAFETY:
        // Caller guarantees `additional > capacity - len`, so `min_capacity > capacity`.
        // Assertion above guarantees `min_capacity <= MAX_CAPACITY`.
        unsafe { self.grow(min_capacity) };
    }

    /// Grow the allocation to hold at least `min_capacity` elements.
    ///
    /// # SAFETY
    ///
    /// * `min_capacity` must be `> self.columns.capacity`.
    /// * `min_capacity` must be `<= MAX_CAPACITY`.
    unsafe fn grow(&mut self, min_capacity: usize) {
        debug_assert!(min_capacity > self.columns.capacity);
        debug_assert!(min_capacity <= Self::MAX_CAPACITY);

        // Grow by doubling (clamped to `MAX_CAPACITY`), then normalize
        // - apply the minimum `MIN_GROW_CAPACITY`
        // - round up to fill the allocation
        //
        // Doubling cannot overflow - `capacity <= MAX_CAPACITY <= isize::MAX`, so `capacity * 2 <= usize::MAX - 1`.
        // The clamp establishes `normalize_capacity`'s safety requirement.
        let double_capacity_clamped = min(self.columns.capacity * 2, Self::MAX_CAPACITY);
        let new_capacity = max(min_capacity, double_capacity_clamped);
        // SAFETY: `0 < min_capacity <= new_capacity <= MAX_CAPACITY` (established above)
        let new_capacity = unsafe { Self::normalize_capacity::<true>(new_capacity) };

        debug_assert!(new_capacity > self.columns.capacity);
        debug_assert!(new_capacity <= Self::MAX_CAPACITY);

        // Allocation happens before any of `self`'s fields are mutated and before the old
        // allocation is deallocated, so `self`'s invariants hold even if `allocate` aborts.
        //
        // SAFETY: `0 < new_capacity <= MAX_CAPACITY <= MAX_ALLOC_CAPACITY`
        // (`normalize_capacity` returns at least its input, `>= min_capacity >= 1`, and at most `MAX_CAPACITY`)
        let new_columns = unsafe { Columns::<F>::allocate(new_capacity) };
        let new_base_ptr = new_columns.base_ptr;

        if self.columns.capacity > 0 {
            // SAFETY: The first `len` elements of every column are initialized. The new
            // allocation has layout `layout_for(new_capacity)`, and
            // `self.len <= self.capacity < new_capacity`. It is freshly allocated, so the
            // two allocations do not overlap.
            // The old copies are never used again - the old allocation is deallocated
            // immediately below, without reading or dropping them.
            unsafe { self.columns.copy_fields(new_base_ptr, new_capacity) };
            // SAFETY: `capacity > 0` (checked above). The old allocation is never used again -
            // `base_ptr` and `capacity` are replaced immediately below, and the `Columns` was
            // not copied anywhere.
            unsafe { self.columns.deallocate() };
        }

        self.columns.base_ptr = new_base_ptr;
        self.columns.capacity = new_capacity;
    }

    /// Normalize a capacity into the capacity to actually allocate.
    ///
    /// Round up so the allocation fills what the allocator would serve, and clamp to [`MAX_CAPACITY`].
    ///
    /// The allocation target is:
    ///
    /// * Within one page: The next power-of-2 size.
    ///   Small requests are served from size classes, and powers of 2 are exact classes
    ///   in all the common allocators.
    /// * Above one page: The next page multiple, minus [`ALLOCATOR_METADATA_SIZE`]
    ///   (see its docs - the reserve keeps the request plus glibc's inline header within whole pages).
    ///
    /// Either way, the slack the allocator would round the request up by becomes usable elements instead,
    /// and growth needs fewer reallocations.
    ///
    /// `APPLY_MIN_GROW` first raises `capacity` to [`MIN_GROW_CAPACITY`].
    /// [`grow`] applies the minimum, [`with_capacity`] honors smaller requests.
    ///
    /// Returns at least `capacity`, at most [`MAX_CAPACITY`].
    ///
    /// # SAFETY
    ///
    /// * `capacity` must not be 0.
    /// * `capacity` must be `<= MAX_CAPACITY`.
    ///
    /// [`MAX_CAPACITY`]: Self::MAX_CAPACITY
    /// [`MIN_GROW_CAPACITY`]: Self::MIN_GROW_CAPACITY
    /// [`grow`]: Self::grow
    /// [`with_capacity`]: Self::with_capacity
    unsafe fn normalize_capacity<const APPLY_MIN_GROW: bool>(mut capacity: usize) -> usize {
        debug_assert!(capacity > 0);
        debug_assert!(capacity <= Self::MAX_CAPACITY);

        let element_size = const { F::SHAPE.element_size() };

        // If element size is large (a page or more for a single element),
        // then the round-to-power-of-2 branch is unreachable and can be removed
        let always_round_to_page = const { Self::MAX_CAPACITY_UNDER_ONE_PAGE == 0 };

        // If index type's range is very small, and can never reach a page,
        // then the round-to-page branch is unreachable and can be removed
        let never_round_to_page = const { Self::MAX_CAPACITY_UNDER_ONE_PAGE >= Self::MAX_CAPACITY };

        let less_than_page = !always_round_to_page
            && (capacity <= Self::MAX_CAPACITY_UNDER_ONE_PAGE || never_round_to_page);

        if less_than_page {
            // Allocation is within one page. Target the next power-of-2 size.

            // Increase capacity to minimum if `APPLY_MIN_GROW == true` (grow).
            // Skip this if `MIN_GROW_CAPACITY == 1`. `capacity` is `>= 1` already.
            if APPLY_MIN_GROW && Self::MIN_GROW_CAPACITY > 1 {
                // `MIN_GROW_CAPACITY` <= `MAX_CAPACITY_UNDER_ONE_PAGE` unless `MAX_CAPACITY_UNDER_ONE_PAGE` is 0,
                // in which case this branch is unreachable.
                // So bumping up to `MIN_GROW_CAPACITY`, still leaves `capacity` <= `MAX_CAPACITY_UNDER_ONE_PAGE`.
                capacity = max(capacity, Self::MIN_GROW_CAPACITY);
            }

            // Cannot overflow - `bytes <= PAGE_SIZE`
            // (`capacity <= MAX_CAPACITY_UNDER_ONE_PAGE`, preserved by the bump above)
            let bytes = capacity * element_size;
            let bytes = bytes.next_power_of_two();
            let mut rounded_capacity = bytes / element_size;

            // Next power of 2 can exceed max capacity when the index type's range is small
            // (e.g. `MAX_CAPACITY == 6` with 4-byte elements: 5 elements = 20 bytes -> 32 bytes = 8 elements).
            // But it can never exceed `MAX_CAPACITY_UNDER_ONE_PAGE` - `bytes` was `<= PAGE_SIZE`,
            // and rounding up to a power of 2 cannot pass `PAGE_SIZE`, which is itself a power of 2.
            // So the clamp is only needed in the uncommon case where `MAX_CAPACITY` is the smaller limit.
            let rounding_can_exceed_max =
                const { Self::MAX_CAPACITY_UNDER_ONE_PAGE > Self::MAX_CAPACITY };
            if rounding_can_exceed_max {
                rounded_capacity = min(rounded_capacity, Self::MAX_CAPACITY);
            }

            rounded_capacity
        } else {
            // Allocation exceeds one page.
            // Target the next page multiple, minus the metadata reserve.
            // Round `bytes` plus the reserve up to whole pages, then subtract the reserve again.
            // The result cannot go below `bytes` - the sum was rounded up with the reserve included.
            // No `MIN_GROW_CAPACITY` bump is needed - `capacity` is already at or above the minimum.
            //
            // Cannot overflow - `capacity <= MAX_CAPACITY <= max_alloc_capacity`, so `bytes <= isize::MAX`,
            // and the sum stays far below `usize::MAX` (asserted below).
            const _: () = {
                assert!(PAGE_SIZE.is_power_of_two());
                assert!((ALLOCATOR_METADATA_SIZE + PAGE_SIZE - 1) < isize::MAX as usize);
            };

            let bytes = capacity * element_size;
            let target = (bytes + ALLOCATOR_METADATA_SIZE + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);
            let rounded_capacity = (target - ALLOCATOR_METADATA_SIZE) / element_size;

            // Rounding can exceed `MAX_CAPACITY`, so clamp back to `MAX_CAPACITY`.
            // The clamp cannot bring the result below original `capacity` passed in to this function.
            min(rounded_capacity, Self::MAX_CAPACITY)
        }
    }

    /// Get references to every field of the element at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn get(&self, index: I) -> F::Ref<'_> {
        let index = self.bounds_checked_index(index);

        // SAFETY: `index < len <= capacity`, so `field_ptrs` returns pointers to the
        // element's field values, which are initialized. The references borrow `self`,
        // so the values cannot be mutated while they live.
        unsafe { F::create_ref(self.columns.field_ptrs(index)) }
    }

    /// Get mutable references to every field of the element at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: I) -> F::Mut<'_> {
        let index = self.bounds_checked_index(index);

        // SAFETY: Same as `get`, plus the references borrow `self` exclusively,
        // so the values cannot be otherwise accessed while they live.
        unsafe { F::create_mut(self.columns.field_ptrs(index)) }
    }

    /// Check `index` is in bounds and convert it to `usize`.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len`.
    #[inline]
    fn bounds_checked_index(&self, index: I) -> usize {
        let index = index.index();
        if index >= self.columns.len {
            // Cold function, so the `fmt::Arguments` construction for the panic message
            // does not put stack frame + spill instructions on the hot path.
            // Takes `&self` rather than `len`, so the hot path's bounds check can compare
            // against `len` in memory without loading it into a register.
            self.out_of_bounds(index);
        }
        index
    }

    /// Panic with out-of-bounds message.
    #[cold]
    #[inline(never)]
    fn out_of_bounds(&self, index: usize) -> ! {
        let len = self.columns.len;
        panic!("Index out of bounds: `len` is {len} but `index` is {index}");
    }

    /// Get references to every field of the element at `index`, without checking
    /// that `index` is in bounds.
    ///
    /// # SAFETY
    ///
    /// `index` must be in bounds: `index.index() < self.len()`.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: I) -> F::Ref<'_> {
        let index = index.index();
        debug_assert!(index < self.columns.len);

        // SAFETY: `index < len` per this function's safety contract, and `len <= capacity`,
        // so `field_ptrs` returns pointers to the element's field values, which are initialized.
        // The references borrow `self`, so the values cannot be mutated while they live.
        unsafe { F::create_ref(self.columns.field_ptrs(index)) }
    }

    /// Get mutable references to every field of the element at `index`, without
    /// checking that `index` is in bounds.
    ///
    /// # SAFETY
    ///
    /// `index` must be in bounds: `index.index() < self.len()`.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: I) -> F::Mut<'_> {
        let index = index.index();
        debug_assert!(index < self.columns.len);

        // SAFETY: Same as `get_unchecked`, plus the references borrow `self` exclusively,
        // so the values cannot be otherwise accessed while they live
        unsafe { F::create_mut(self.columns.field_ptrs(index)) }
    }

    /// Get slices over every column.
    //
    // Bounded `F: SliceFields<I>` (not just `Fields`) because the slice views are keyed
    // by the index type `I`, so they live on the `SliceFields<I>` trait - see its docs.
    #[inline]
    pub fn slices(&self) -> F::Slices<'_>
    where
        F: SliceFields<I>,
    {
        let len = self.columns.len;

        // Communicate the bound on the slices' length to compiler.
        // SAFETY: `len <= capacity <= MAX_CAPACITY`.
        unsafe { assert_unchecked!(len <= Self::MAX_CAPACITY) };

        // SAFETY: `slice_ptrs` returns aligned pointers to the columns' starts
        // (dangling if `capacity == 0`, but `len == 0` then too),
        // whose first `len` elements are initialized.
        // The slices borrow `self`, so the values cannot be mutated while they live.
        unsafe { F::create_slices(self.columns.slice_ptrs(), len) }
    }

    /// Get mutable slices over every column.
    //
    // Bounded `F: SliceFields<I>` - see `slices`.
    #[inline]
    pub fn slices_mut(&mut self) -> F::SlicesMut<'_>
    where
        F: SliceFields<I>,
    {
        let len = self.columns.len;

        // Communicate the bound on the slices' length to compiler.
        // SAFETY: `len <= capacity <= MAX_CAPACITY`.
        unsafe { assert_unchecked!(len <= Self::MAX_CAPACITY) };

        // SAFETY: Same as `slices`, plus the slices borrow `self` exclusively,
        // so the values cannot be otherwise accessed while they live
        unsafe { F::create_slices_mut(self.columns.slice_ptrs(), len) }
    }

    /// Iterate over all valid indices.
    ///
    /// The returned iterator does not borrow `self` (the `use<...>` clause omits
    /// the `&self` lifetime, opting out of edition 2024's capture-everything default).
    /// It snapshots `len`, so the table can be mutated while the iterator lives.
    /// IDs of elements pushed after the `iter_ids` call are not included.
    #[inline]
    pub fn iter_ids(&self) -> impl ExactSizeIterator<Item = I> + FusedIterator + use<I, F> {
        let len = self.columns.len;

        // Communicate the bound on the returned iterator's length (`size_hint`) to compiler.
        // SAFETY: `len <= capacity <= MAX_CAPACITY`.
        unsafe { assert_unchecked!(len <= Self::MAX_CAPACITY) };

        (0..len).map(|i| {
            // Communicate the bound on the yielded index to compiler.
            // SAFETY: `i < len <= MAX_CAPACITY`, as `i` ranges over `0..len`.
            unsafe { assert_unchecked!(i < Self::MAX_CAPACITY) };

            // SAFETY: `i < len <= MAX_CAPACITY <= I::MAX + 1`, so `i <= I::MAX`
            unsafe { I::from_usize_unchecked(i) }
        })
    }

    /// Iterate over the elements, yielding references to every field of each element.
    #[inline]
    pub fn iter(&self) -> Iter<'_, F> {
        Iter::new(self)
    }

    /// Iterate over the elements, yielding mutable references to every field of each element.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, F> {
        IterMut::new(self)
    }

    /// Iterate over the elements, yielding each element's ID and references to
    /// every field of it.
    #[inline]
    pub fn iter_enumerated(
        &self,
    ) -> impl ExactSizeIterator<Item = (I, F::Ref<'_>)> + FusedIterator {
        self.iter().enumerate().map(|(index, item)| {
            // SAFETY: `index < len <= MAX_CAPACITY <= I::MAX + 1`, so `index <= I::MAX`
            let index = unsafe { I::from_usize_unchecked(index) };
            (index, item)
        })
    }

    /// Iterate over the elements, yielding each element's ID and mutable references to
    /// every field of it.
    #[inline]
    pub fn iter_mut_enumerated(
        &mut self,
    ) -> impl ExactSizeIterator<Item = (I, F::Mut<'_>)> + FusedIterator {
        self.iter_mut().enumerate().map(|(index, item)| {
            // SAFETY: `index < len <= MAX_CAPACITY <= I::MAX + 1`, so `index <= I::MAX`
            let index = unsafe { I::from_usize_unchecked(index) };
            (index, item)
        })
    }

    /// Consume the `MultiVec`, yielding each element's ID and the element as an owned `F`
    /// value (reassembled from its stored field values).
    #[inline]
    pub fn into_iter_enumerated(self) -> impl ExactSizeIterator<Item = (I, F)> + FusedIterator {
        self.into_iter().enumerate().map(|(index, item)| {
            // SAFETY: `index < len <= MAX_CAPACITY <= I::MAX + 1`, so `index <= I::MAX`
            let index = unsafe { I::from_usize_unchecked(index) };
            (index, item)
        })
    }
}

impl<I: Idx, F: Fields> IntoIterator for MultiVec<I, F> {
    type Item = F;
    type IntoIter = IntoIter<F>;

    /// Consume the `MultiVec`, yielding each element as an owned `F` value (reassembled
    /// from its stored field values).
    fn into_iter(self) -> IntoIter<F> {
        IntoIter::new(self)
    }
}

impl<'v, I: Idx, F: Fields> IntoIterator for &'v MultiVec<I, F> {
    type Item = F::Ref<'v>;
    type IntoIter = Iter<'v, F>;

    fn into_iter(self) -> Iter<'v, F> {
        self.iter()
    }
}

impl<'v, I: Idx, F: Fields> IntoIterator for &'v mut MultiVec<I, F> {
    type Item = F::Mut<'v>;
    type IntoIter = IterMut<'v, F>;

    fn into_iter(self) -> IterMut<'v, F> {
        self.iter_mut()
    }
}

impl<I: Idx, F: Fields> Default for MultiVec<I, F> {
    fn default() -> Self {
        Self::new()
    }
}

/// Cloning is *field-wise* - each column is cloned in turn, element by element.
/// `F`'s own `Clone` impl is never invoked - elements only ever exist as scattered field
/// values, so there is no `F` value to clone. [`CloneFields`] is implemented by the
/// `multi_vec!` macro only for tables declared with `#[derive(Clone)]`, and the macro
/// derives `Clone` on the fields struct itself, so the two notions of cloning agree
/// (see the trait's docs).
impl<I: Idx, F: CloneFields> Clone for MultiVec<I, F> {
    fn clone(&self) -> Self {
        let len = self.columns.len;
        if len == 0 {
            return Self::new();
        }

        // Allocate exactly `len` (not `capacity`), like `Vec::clone`.
        // `new.len` remains 0 until all elements are cloned. If an element's `clone` panics,
        // `new` is dropped with `len == 0`, so its `Drop` frees the allocation without touching
        // the partially-initialized elements. The already-cloned elements are dropped during
        // unwinding by `clone_columns`' drop guards, so nothing is leaked.
        // SAFETY: `0 < len <= capacity <= MAX_ALLOC_CAPACITY` (`len > 0` checked above).
        let new_columns = unsafe { Columns::<F>::allocate(len) };
        let mut new_vec = Self { columns: new_columns, index_marker: PhantomData };

        let src_ptrs = self.columns.slice_ptrs();
        let dst_ptrs = new_vec.columns.slice_ptrs();
        let src_and_dst_ptrs =
            CopyArray::from_fn(|i| SrcAndDstPtrs { src_ptr: src_ptrs[i], dst_ptr: dst_ptrs[i] });

        // SAFETY: `slice_ptrs` returns aligned pointers to each column's start, so each
        // `src_and_dst_ptrs[i]` pairs the `i`th `self` (source) column with the `i`th `new`
        // (destination) column. The first `len` elements of every `self` column are initialized.
        // `new`'s columns are valid for writes of `len` elements (`new.capacity == len`).
        // `new`'s allocation is freshly allocated, so the two do not overlap.
        unsafe { F::clone_columns(src_and_dst_ptrs, len) };

        // `clone_columns` initialized the first `len` elements of every `new` column
        // (guaranteed by `CloneFields`' contract), so `new`'s invariants hold with `len` set
        new_vec.columns.len = len;

        new_vec
    }
}

impl<I: Idx, F: Fields> Drop for MultiVec<I, F> {
    fn drop(&mut self) {
        // If never allocated, no backing allocation to free, and no values to drop either
        if self.columns.capacity == 0 {
            return;
        }

        // Drop values.
        // SAFETY: The first `len` elements of every column are initialized, and `slice_ptrs`
        // returns aligned pointers to the columns' starts. The values are never used again -
        // the allocation is freed below without reading them. If an element's `Drop` panics,
        // the remaining elements and the allocation are leaked - `drop` is not re-entered.
        unsafe { F::drop_columns(self.columns.slice_ptrs(), self.columns.len) };

        // Free backing allocation.
        // SAFETY: `capacity > 0` (checked above). The allocation is never used again -
        // `self` is being dropped, and its `Columns` was not copied anywhere (iterators
        // copy it, but only by consuming the `MultiVec` or holding a borrow of it, which
        // has expired).
        unsafe { self.columns.deallocate() };
    }
}

impl<I: Idx, F: Fields> Debug for MultiVec<I, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiVec")
            .field("len", &self.columns.len)
            .field("capacity", &self.columns.capacity)
            .finish_non_exhaustive()
    }
}

/// Get the minimum of two `usize`s.
///
/// Equivalent to [`std::cmp::min`] but can be used in const context.
const fn min(a: usize, b: usize) -> usize {
    if b < a { b } else { a }
}

/// Get the maximum of two `usize`s.
///
/// Equivalent to [`std::cmp::max`] but can be used in const context.
const fn max(a: usize, b: usize) -> usize {
    if b < a { a } else { b }
}
