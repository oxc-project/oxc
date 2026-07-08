//! Column-by-column cloning for [`MultiVec`], used by the expansion of the [`multi_vec!`] macro.
//!
//! Cloning a struct-of-arrays vector is *field-wise* - each column is cloned in turn
//! by [`clone_column`], element by element. For tables with fields which are `Copy`
//! and have no padding (e.g. primitives like `u32`), field-wise cloning is much faster
//! than cloning *element-wise*, because a whole column for a `Copy` type can be cloned
//! with a single `memcpy`.
//!
//! The only caller is [`CloneFields::clone_columns`]. The [`multi_vec!`] macro implements it -
//! one `clone_column` call per field - only for tables declared with `#[derive(Clone)]`.
//! Tables without the derive use nothing in this module.
//!
//! Cloning is panic-safe, via [`ColumnDropGuard`]s at two levels. If a value's `clone` panics,
//! the guard inside [`clone_column`] drops the values already cloned into the current column,
//! and the guards held by `clone_columns` drop every earlier, fully-cloned column.
//! Nothing is leaked.
//!
//! [`MultiVec`]: super::MultiVec
//! [`multi_vec!`]: super::multi_vec
//! [`CloneFields::clone_columns`]: super::fields::CloneFields::clone_columns

#![expect(
    rustdoc::private_intra_doc_links,
    reason = "items are only reachable via `#[doc(hidden)]`, so links to private items are \
    only ever seen in internal docs (`--document-private-items`), where they resolve"
)]

use std::{
    mem::{MaybeUninit, needs_drop},
    ptr::{self, NonNull},
    slice,
};

/// One column's source and destination pointers.
///
/// [`CloneFields::clone_columns`] takes one per column, zipped into a single array,
/// and passes each on to its column's [`clone_column`] call.
///
/// A single array (rather than separate `src` and `dst` arrays) lets the macro-generated
/// `clone_columns` destructure it with one binding per field, like every other
/// [`Fields`] method - two arrays would need two `paste!`-derived binding names per field.
///
/// [`CloneFields::clone_columns`]: super::fields::CloneFields::clone_columns
/// [`Fields`]: super::fields::Fields
#[derive(Clone, Copy)]
pub struct SrcAndDstPtrs {
    /// Pointer to the start of the source column.
    pub src_ptr: NonNull<u8>,
    /// Pointer to the start of the destination column.
    pub dst_ptr: NonNull<u8>,
}

/// Clone the first `len` values of a field array from its source column to its destination
/// column, element by element.
///
/// Returns a fully-armed [`ColumnDropGuard`] for the destination column.
/// The caller must hold it until all columns are cloned, then [`mem::forget`] it
/// (see [`ColumnDropGuard`]).
///
/// If a value's `clone` panics, the values already written to the destination column
/// are dropped (by this function's own guard) during unwinding.
///
/// # No `Copy` fast path
///
/// Every column is cloned element-wise. Do NOT add a bitwise fast path keyed on `T: Copy`.
/// `Copy` does not forbid a custom, effectful `Clone` impl - `Clone`'s docs only say the two
/// *should* agree - and a bitwise path would silently skip it, diverging from
/// `#[derive(Clone)]` semantics. `copy_type_with_custom_clone` (tests.rs) pins this.
///
/// std's equivalent fast paths (`Vec::clone` etc.) avoid the same trap by specializing on the
/// internal `TrivialClone` marker, which only `#[derive(Clone, Copy)]` (and std itself)
/// implements - correctly excluding custom impls. That marker is perma-unstable, so the only
/// key available on stable is `T: Copy`, and it is the wrong one.
///
/// A fast path would gain almost nothing anyway. For trivially-cloneable types, the loop
/// already collapses to a single `memcpy` per column (asm-verified). The exception is `Copy`
/// types with padding, which get an element-wise field-copy loop instead of one `memcpy` -
/// acceptable in a clone path.
///
/// # SAFETY
///
/// * `src_ptr` must be aligned for `T`, and valid for reads of `len` consecutive
///   values of `T`, all initialized.
/// * `dst_ptr` must be aligned for `T`, and valid for writes of `len` consecutive
///   values of `T`.
/// * The two ranges must not overlap.
/// * Dangling but aligned pointers are sufficient if `len == 0` or `T` is zero-sized.
///
/// [`mem::forget`]: std::mem::forget
#[inline]
pub unsafe fn clone_column<T: Clone>(
    src_and_dst_ptrs: SrcAndDstPtrs,
    len: usize,
) -> ColumnDropGuard<T> {
    // Do NOT skip zero-sized `T`.
    // A ZST's `clone` can be effectful (a counter, a guard type - paired with its `Drop`).
    // Skipping would conjure destination values whose clones never ran.
    // For ZSTs with a trivial `Clone` implementation, compiler will recognise this all
    // boils down to a 0-bytes `memcpy`, and reduce this function to just constructing the
    // returned guard. So adding an early exit for ZSTs would not gain anything anyway.

    // The loop lives in an inner function taking the columns as slices. LLVM IR `noalias` is only
    // emitted for reference-typed function *parameters* (references created mid-function carry
    // no aliasing information), and survives inlining as scoped-alias metadata - so this shape
    // lets LLVM prove the two slices are disjoint.
    //
    // For trivially-cloneable types (e.g. `Copy` types), that collapses the loop to a single `memcpy`.
    // For types with real `Clone` impls, it enables vectorization without a runtime overlap check.
    //
    // Note: The guard's `initialized` updates don't defeat this.
    // For field types with no drop code, the guard is a no-op (see its `Drop` impl), so nothing
    // ever reads `initialized`, and the counter and its updates are deleted.
    // For types with drop code whose `clone` cannot panic, the counter is read only in later columns'
    // unwind paths, so the in-loop increments are still removed - those paths use `len` directly.
    #[inline]
    fn clone_into<T: Clone>(src: &[T], dst: &mut [MaybeUninit<T>], initialized: &mut usize) {
        for (value, out) in src.iter().zip(dst.iter_mut()) {
            out.write(value.clone());
            // Only incremented after the value is fully written, so if the *next*
            // value's `clone` panics, the guard drops exactly the initialized values.
            *initialized += 1;
        }
    }

    let SrcAndDstPtrs { src_ptr, dst_ptr } = src_and_dst_ptrs;

    let mut guard = ColumnDropGuard { ptr: dst_ptr.cast::<T>(), initialized: 0 };

    // SAFETY: Caller guarantees `src` is aligned and valid for reads of `len`
    // consecutive initialized values of `T`, which are not mutated while the slice lives
    // (the only writes are through `dst`, which does not overlap).
    let src = unsafe { slice::from_raw_parts(src_ptr.cast::<T>().as_ptr(), len) };

    // SAFETY: Caller guarantees `dst` is aligned and valid for reads and writes of `len`
    // consecutive values of `T`. `MaybeUninit<T>` has the same layout as `T`, valid even for
    // uninitialized memory. The values are not accessed through any other pointer while the
    // slice lives (`src` does not overlap, and the guard's pointer is read only when the guard
    // drops, after this slice is dead).
    let dst = unsafe { slice::from_raw_parts_mut(dst_ptr.cast::<MaybeUninit<T>>().as_ptr(), len) };

    clone_into(src, dst, &mut guard.initialized);

    // All `len` values cloned without panicking. Return the guard to caller.
    // At this point `guard.initialized == len`, so dropping the guard drops all the values.
    guard
}

/// Drop guard for a partially or fully cloned column, returned by [`clone_column`].
///
/// If dropped, drops the first `initialized` values of the column.
/// This gives cloning panic safety at two levels:
///
/// * Within one column: [`clone_column`] keeps `initialized` up to date as it clones, and holds
///   the guard as a local until it returns - so if a value's `clone` panics, unwinding drops
///   the values already cloned into this column.
///
/// * Across columns: the caller (macro-generated [`CloneFields::clone_columns`]) holds the
///   returned guards (each fully armed, `initialized == len`) in a tuple until *all* columns
///   are cloned, and only then [`mem::forget`]s the tuple. So if a later column's `clone` panics
///   while building the tuple, unwinding drops the already-built guards - and with them, every
///   earlier column in full.
///
/// On the success path the guards are forgotten, and for field types with no drop code,
/// the compiler removes the guard machinery entirely (see the `Drop` impl below).
///
/// [`mem::forget`]: std::mem::forget
/// [`CloneFields::clone_columns`]: super::fields::CloneFields::clone_columns
pub struct ColumnDropGuard<T> {
    /// Pointer to the start of the column.
    ptr: NonNull<T>,
    /// Number of values at the start of the column which are initialized.
    initialized: usize,
}

impl<T> Drop for ColumnDropGuard<T> {
    // This method is shaped to keep the guard cheap on the success path, and landing pads
    // small on the panic path:
    //
    // * `#[inline(always)]`, with the dropping outlined in `drop_column_prefix`, which takes
    //   `ptr` and `initialized` *by value*. After inlining, the guard's address never escapes,
    //   so the guard's fields live in registers (`clone_column`'s `*initialized += 1` is a
    //   register increment, not a memory write), and each landing pad is just
    //   "load 2 registers, call, resume".
    // * `drop_column_prefix` is `#[inline(never)]`, so the drop loop can never be inlined into
    //   (and bloat) landing pads, and `#[cold]` (it only runs during unwinding), placing it in
    //   the cold section.
    // * The `const` `needs_drop` guard is needed *because* of the outlining. `#[inline(never)]`
    //   means the compiler cannot see into `drop_column_prefix` to tell dropping is a no-op when
    //   `T` isn't `Drop`. This check states that fact at the call site, so for such types `drop`
    //   compiles to nothing (no call, no landing pad).
    #[expect(clippy::inline_always, reason = "trivial: a compile-time check + a call")]
    #[inline(always)]
    fn drop(&mut self) {
        if const { needs_drop::<T>() } {
            // SAFETY: `ptr` is aligned and valid for reads and writes of `initialized`
            // initialized values of `T` (upheld by `clone_column`, which creates the
            // guard with `initialized = 0` and increments it only after each value is written).
            // The values are never used again - the guard is only dropped when cloning panicked,
            // and the caller's partially-built `MultiVec` (with `len == 0`) does not read them.
            unsafe { drop_column_prefix(self.ptr, self.initialized) };
        }
    }
}

/// Outlined cleanup for [`ColumnDropGuard`]: drop the first `initialized` values of the column
/// starting at `ptr`.
///
/// Only called during unwinding, from landing pads - see [`ColumnDropGuard`]'s `Drop` impl
/// for why it is `#[cold]` and `#[inline(never)]`, and takes its arguments by value.
///
/// # SAFETY
///
/// * `ptr` must be aligned for `T`, and valid for reads and writes of `initialized`
///   consecutive values of `T`, all initialized.
/// * The values must not be used after this call.
#[cold]
#[inline(never)]
unsafe fn drop_column_prefix<T>(ptr: NonNull<T>, initialized: usize) {
    // SAFETY: Caller guarantees `ptr` is aligned and valid for reads and writes of
    // `initialized` initialized values of `T`, which are never used again
    unsafe {
        ptr::drop_in_place(ptr::slice_from_raw_parts_mut(ptr.as_ptr(), initialized));
    }
}
