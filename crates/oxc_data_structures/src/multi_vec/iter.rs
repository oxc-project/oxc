//! Iterators over a [`MultiVec`]'s elements.
//!
//! * [`Iter`] yields references to every field of each element ([`Fields::Ref`]).
//! * [`IterMut`] yields mutable references to every field of each element ([`Fields::Mut`]).
//! * [`IntoIter`] consumes the `MultiVec`, yielding each element as an owned `F` value.
//!
//! All 3 wrap a [`RawIter`]: a copy of the `MultiVec`'s [`Columns`] (base pointer, `len`,
//! and capacity), plus the index of the next element to yield - a fixed 4 words,
//! regardless of the number of fields. `next` computes each field's pointer with the
//! same addressing `MultiVec` itself uses ([`Columns::field_ptrs`]):
//!
//! ```text
//! field_ptr = base_ptr + capacity * FIELD_OFFSETS[i] + index * size_of(field i)
//! ```
//!
//! `FIELD_OFFSETS[i]` and `size_of(field i)` are compile-time constants, and
//! `capacity * FIELD_OFFSETS[i]` is loop-invariant, so in iteration loops the optimizer
//! hoists the invariants and strength-reduces `index * size` to a pointer increment -
//! producing assembly identical to a hand-written per-field pointer walk (verified),
//! with pointers for unused fields eliminated entirely.

use std::{iter::FusedIterator, marker::PhantomData, mem, ptr::NonNull};

use oxc_index::Idx;

use super::{MultiVec, columns::Columns, fields::Fields};

/// Shared core of [`Iter`], [`IterMut`], and [`IntoIter`]: an iterator which walks the columns,
/// yielding each element's field pointers ([`F::Array<NonNull<u8>>`]).
///
/// # Invariants
///
/// * `columns`' invariants hold (see [`Columns`]) - they are geometry only.
///   `columns.len` is the end of iteration - `len` of the [`MultiVec`] the iterator was created from.
/// * `index <= columns.len`.
/// * Elements `index..columns.len` of every column are initialized.
///   Elements before `index` have been yielded - for [`IntoIter`], moved out.
///   This is `RawIter`'s refinement of `Columns`' contents-agnostic invariants.
/// * `RawIter` does not borrow the `MultiVec` - the wrapping iterator is responsible for
///   keeping the elements and the allocation live (by borrowing or owning the `MultiVec`),
///   and determines what may be done through the yielded pointers (read / write / move out).
///
/// [`F::Array<NonNull<u8>>`]: Fields::Array
struct RawIter<F: Fields> {
    /// A copy of the `MultiVec`'s [`Columns`], taken at creation.
    columns: Columns<F>,
    /// Index of the next element to yield.
    index: usize,
}

impl<F: Fields> RawIter<F> {
    /// Create a `RawIter` over `vec`'s elements.
    fn new<I: Idx>(vec: &MultiVec<I, F>) -> Self {
        // `vec`'s invariants establish `self`'s, with `index = 0` -
        // `columns`' invariants hold, and the first `len` elements of every column are initialized.
        Self { columns: vec.columns(), index: 0 }
    }

    /// Number of elements remaining.
    #[inline]
    fn remaining(&self) -> usize {
        // Cannot underflow. `index <= columns.len`.
        self.columns.len - self.index
    }
}

impl<F: Fields> Iterator for RawIter<F> {
    type Item = F::Array<NonNull<u8>>;

    /// Get pointers to the next element's field values, and advance past it.
    /// Returns `None` if no elements remain.
    ///
    /// The returned pointers are aligned and point to initialized values of the field types.
    /// What may be done through them is determined by the wrapping iterator
    /// (see `RawIter`'s invariants).
    #[inline]
    fn next(&mut self) -> Option<F::Array<NonNull<u8>>> {
        if self.index == self.columns.len {
            return None;
        }

        // SAFETY: `index < columns.len` (checked above) `<= capacity`
        let ptrs = unsafe { self.columns.field_ptrs(self.index) };
        self.index += 1;

        Some(ptrs)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining(), Some(self.remaining()))
    }
}

impl<F: Fields> ExactSizeIterator for RawIter<F> {}

// `next` returns `None` every time once `remaining == 0`
impl<F: Fields> FusedIterator for RawIter<F> {}

/// Iterator over a [`MultiVec`]'s elements, yielding references to every field of
/// each element ([`Fields::Ref`]).
///
/// Returned by [`MultiVec::iter`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'v, F: Fields> {
    raw: RawIter<F>,
    /// `Iter` acts as a shared borrow of the `MultiVec`'s stored values.
    marker: PhantomData<&'v F>,
}

// SAFETY: `Iter` only yields shared references to the stored field values, so it can be
// sent or shared across threads under the same conditions as `&MultiVec`: when the field
// types are `Sync` (see `MultiVec`'s `Send` / `Sync` impls).
//
// The bound is deliberately on `F`, not on the item type (`F::Ref<'_>: Send`).
// Sending the iterator grants another thread *shared access to the stored `F` data*.
// `next` reads it via `create_ref` on that thread, while the origin thread can still access
// it through its own `&MultiVec` (shared borrows coexist with the iterator's).
// "May be accessed from two threads at once" is exactly `F: Sync`.
//
// An item-based bound would measure the wrong thing. The `Fields` contract allows a
// hand-written impl's `Ref` type to *copy* field values out (reading through the pointers
// is permitted) instead of holding references. e.g. for a `Cell<u32>` field, a `Ref`
// copying the cell's value to an owned `u32` is unconditionally `Send` - so a
// `F::Ref<'_>: Send` bound would make `Iter: Send` despite the `Cell`. `next()` on the
// other thread would then read the cell's memory, racing with the origin thread mutating
// it through its `&MultiVec` (e.g. `slices()` + `Cell::set` - interior mutability needs
// only a shared reference): a data race. `F: Sync` rejects this (`Cell: !Sync`).
//
// (For the macro-generated `Ref` - a struct of `&FieldTy` fields - the two formulations
// coincide: `&T: Send` ⇔ `T: Sync`, for every field.)
unsafe impl<F: Fields + Sync> Send for Iter<'_, F> {}

// SAFETY: See `Send` impl above.
unsafe impl<F: Fields + Sync> Sync for Iter<'_, F> {}

impl<'v, F: Fields> Iter<'v, F> {
    /// Create an `Iter` over `vec`'s elements.
    pub(super) fn new<I: Idx>(vec: &'v MultiVec<I, F>) -> Self {
        // The iterator holds a shared borrow of `vec` for `'v`, so the elements remain live,
        // and cannot be mutated, while it lives
        Self { raw: RawIter::new(vec), marker: PhantomData }
    }
}

impl<'v, F: Fields> Iterator for Iter<'v, F> {
    type Item = F::Ref<'v>;

    #[inline]
    fn next(&mut self) -> Option<F::Ref<'v>> {
        let ptrs = self.raw.next()?;
        // SAFETY: `RawIter::next` returned aligned pointers to one element's initialized
        // field values. The values are not mutated while the references live - the iterator
        // holds a shared borrow of the `MultiVec` for `'v` (see `Iter::new`).
        Some(unsafe { F::create_ref(ptrs) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}

impl<F: Fields> ExactSizeIterator for Iter<'_, F> {}

// `next` delegates to `RawIter`, which is fused
impl<F: Fields> FusedIterator for Iter<'_, F> {}

/// Iterator over a [`MultiVec`]'s elements, yielding mutable references to every field of
/// each element ([`Fields::Mut`]).
///
/// Returned by [`MultiVec::iter_mut`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IterMut<'v, F: Fields> {
    raw: RawIter<F>,
    /// `IterMut` acts as an exclusive borrow of the `MultiVec`'s stored values.
    marker: PhantomData<&'v mut F>,
}

// SAFETY: `IterMut` yields mutable references to the stored field values, so it can be
// sent across threads under the same conditions as `&mut MultiVec`: when the field types
// are `Send` (see `MultiVec`'s `Send` / `Sync` impls). Through a mutable reference, values
// can be moved out (e.g. `mem::replace`), so sending the iterator can move the stored data
// itself across threads - that needs `F: Send` on the data, not on the `F::Mut<'_>` item type
// (see `Iter`'s `Send` impl for why the bound is on the data).
unsafe impl<F: Fields + Send> Send for IterMut<'_, F> {}

// SAFETY: A shared `&IterMut` exposes no access to the values (only `size_hint`), but
// `Sync` also makes `&IterMut: Send`, which allows shared access to `F` values from another
// thread if `Iterator::Item` references escape. So require field types to be `Sync`, matching
// `&mut MultiVec: Sync`.
unsafe impl<F: Fields + Sync> Sync for IterMut<'_, F> {}

impl<'v, F: Fields> IterMut<'v, F> {
    /// Create an `IterMut` over `vec`'s elements.
    #[expect(
        clippy::needless_pass_by_ref_mut,
        reason = "the exclusive borrow is the point: it guarantees the elements cannot be \
            otherwise accessed while the iterator (which yields `&mut`s derived from it) lives"
    )]
    pub(super) fn new<I: Idx>(vec: &'v mut MultiVec<I, F>) -> Self {
        // The iterator holds an exclusive borrow of `vec` for `'v`, so the elements
        // remain live, and cannot be otherwise accessed, while it lives
        Self { raw: RawIter::new(vec), marker: PhantomData }
    }
}

impl<'v, F: Fields> Iterator for IterMut<'v, F> {
    type Item = F::Mut<'v>;

    #[inline]
    fn next(&mut self) -> Option<F::Mut<'v>> {
        let ptrs = self.raw.next()?;
        // SAFETY: `RawIter::next` returned aligned pointers to one element's initialized
        // field values. The values are not accessed through any other pointer while the
        // references live - the iterator holds an exclusive borrow of the `MultiVec` for
        // `'v` (see `IterMut::new`), and each element is yielded at most once
        // (`RawIter::next` advances past it), so references to different elements are disjoint.
        Some(unsafe { F::create_mut(ptrs) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}

impl<F: Fields> ExactSizeIterator for IterMut<'_, F> {}

// `next` delegates to `RawIter`, which is fused
impl<F: Fields> FusedIterator for IterMut<'_, F> {}

/// Consuming iterator over a [`MultiVec`]'s elements, yielding each element as an owned `F`
/// value, reassembled from its stored field values.
///
/// Returned by [`MultiVec`]'s [`IntoIterator`] impl.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoIter<F: Fields> {
    /// The `IntoIter` owns the elements and the allocation of the `MultiVec` it was
    /// created from. Its `Drop` drops the unyielded elements (`raw`'s remaining range),
    /// then frees the allocation (`raw.columns`).
    raw: RawIter<F>,
}

// SAFETY: `IntoIter` owns the stored field values (like the `MultiVec` it was created
// from), so it can be sent across threads when the field types are `Send`
// (see `MultiVec`'s `Send` / `Sync` impls).
unsafe impl<F: Fields + Send> Send for IntoIter<F> {}

// SAFETY: See `Send` impl above. A shared `&IntoIter` exposes no access to the values,
// but `Sync` also makes `&IntoIter: Send`, so require field types to be `Sync`, like `MultiVec`.
unsafe impl<F: Fields + Sync> Sync for IntoIter<F> {}

impl<F: Fields> IntoIter<F> {
    /// Create an `IntoIter` over `vec`'s elements, consuming it.
    pub(super) fn new<I: Idx>(vec: MultiVec<I, F>) -> Self {
        let raw = RawIter::new(&vec);

        // The `IntoIter` takes ownership of the elements and the allocation
        // (`raw` snapshotted the `Columns` above).
        // Forget `vec` so its `Drop` does not drop the elements or free the allocation.
        // The `IntoIter`'s own `Drop` does both instead.
        mem::forget(vec);

        Self { raw }
    }
}

impl<F: Fields> Iterator for IntoIter<F> {
    type Item = F;

    #[inline]
    fn next(&mut self) -> Option<F> {
        let ptrs = self.raw.next()?;
        // SAFETY: `RawIter::next` returned aligned pointers to one element's initialized
        // field values, owned by this `IntoIter` (ownership was transferred to it by
        // `IntoIter::new`). The values are moved out and never used again - `RawIter::next`
        // advanced past them, so they are neither yielded again nor dropped by `IntoIter`'s
        // `Drop` (which frees the allocation without reading them).
        Some(unsafe { F::create_owned(ptrs) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}

impl<F: Fields> ExactSizeIterator for IntoIter<F> {}

// `next` delegates to `RawIter`, which is fused
impl<F: Fields> FusedIterator for IntoIter<F> {}

impl<F: Fields> Drop for IntoIter<F> {
    fn drop(&mut self) {
        // If the `MultiVec` never allocated, there is no allocation to free,
        // and no elements to drop either (`remaining <= len <= capacity == 0`)
        if self.raw.columns.capacity == 0 {
            return;
        }

        // Drop the unyielded elements (`index..len`), if any.
        // The `remaining > 0` check is an optimization. The common case is a fully consumed
        // iterator, where the check avoids computing field pointers and calling `drop_in_place`
        // with a zero-length slice for every column whose type needs dropping (slice drop glue
        // is not always inlined).
        //
        // If an element's `Drop` panics, the remaining elements and the allocation are
        // leaked (`drop` is not re-entered) - the same behavior as `MultiVec`'s `Drop`.
        let remaining = self.raw.remaining();
        if remaining > 0 {
            // SAFETY: `index <= columns.len <= capacity`, as `field_ptrs` requires. It returns
            // aligned pointers to element `index`'s field values, and elements `index..len` (the
            // `remaining` elements starting there) are initialized and valid for reads and writes -
            // and owned by this `IntoIter` (ownership was transferred to it by `IntoIter::new`).
            // They are never used again - the allocation is freed below, without reading them.
            unsafe { F::drop_columns(self.raw.columns.field_ptrs(self.raw.index), remaining) };
        }

        // Free the backing allocation.
        // SAFETY: `capacity > 0` (checked above). The allocation is never used again -
        // the `IntoIter` is being dropped, and `raw.columns` is the only copy of these
        // `Columns` (the `MultiVec` they came from was forgotten by `IntoIter::new`).
        unsafe { self.raw.columns.deallocate() };
    }
}
