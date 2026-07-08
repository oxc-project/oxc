//! [`Fields`] trait, describing a set of fields stored as struct-of-arrays in a [`MultiVec`].
//!
//! [`SliceFields`] extends it with the slice views, which are split out because they name the
//! *index* type (see [`SliceFields`] for why).
//!
//! [`MultiVec`]: super::MultiVec

use std::ptr::NonNull;

use oxc_index::Idx;

use super::{
    clone::SrcAndDstPtrs,
    shape::{CopyArray, Shape},
};

/// A type whose fields can be stored as struct-of-arrays in a [`MultiVec`].
///
/// Usually implemented by the [`multi_vec!`] macro.
///
/// A thin translation layer between untyped field pointers and the typed field values /
/// references of `Self`. [`MultiVec`] performs all allocation, layout, and pointer arithmetic
/// itself (using [`SHAPE`]). The trait's methods only cast the provided pointers to the field
/// types and read / write / drop / create references through them - they contain no logic of
/// their own.
///
/// # The `Array` associated type
///
/// [`MultiVec`] handles one pointer (or offset) per field, always as a fixed-size array
/// `[T; N]` where `N` is the number of fields. The trait cannot name `N` - `[T; Self::FIELD_COUNT]`
/// in a trait signature is a generic const expression, which stable Rust rejects wherever it
/// appears (including behind a wrapper like `Array<T, { Self::FIELD_COUNT }>`).
///
/// Instead the array *type* is an associated type constructor, [`Array<T>`], which each impl
/// defines as `[T; N]` with its own concrete `N` - so the field count never appears as a const
/// generic parameter on this trait, [`MultiVec`], or the iterators.
///
/// [`Array<T>`] is bounded by the sealed `CopyArray<T>` trait (only `[T; N]` implements it),
/// which grants the operations [`MultiVec`] needs - indexing, copying - and `CopyArray::from_fn`,
/// since [`MultiVec`] cannot write an array literal of statically unknown length.
///
/// Field types may need dropping (e.g. `String`). [`MultiVec`] drops the stored field values
/// (via [`drop_columns`]) when it is dropped. Elements only ever exist as scattered field values,
/// so `Self`'s own [`Drop`] impl (if any) is never invoked - only the field values are dropped,
/// individually.
///
/// # SAFETY
///
/// Implementations must uphold all of the following:
///
/// * [`Array<T>`] must be `[T; N]`, with the same `N` (the number of fields) for every `T`.
///   The `CopyArray<T>` bound guarantees each `Array<T>` is *some* `[T; N]` - that every `T` uses
///   the *same* `N` is on the implementor. Unbreakable on stable Rust, breakable on nightly via
///   `#![feature(specialization)]`.
/// * The `[Layout; N]` passed to [`Shape::new`] in [`SHAPE`]'s initializer must be the layout of
///   each field's *type* - `Layout::new::<FieldType>()` - in the same field order the methods
///   interpret `ptrs`. A merely valid [`Layout`] is not enough.
///
///   [`MultiVec`] computes the address of element `index` in a field's array as
///   `field_array_start + index * field_size`. That relies on a type's size being a multiple of
///   its alignment (true of all Rust types) - an artificial layout without that property would
///   produce misaligned elements. This is the only field-layout information [`MultiVec`] trusts -
///   every derived quantity (offsets, alignment, element size, capacity) is computed by `Shape::new`.
/// * Every method must interpret `ptrs[i]` as a pointer to the `i`th field's type (same field
///   order as [`SHAPE`]'s layout list), and do nothing beyond casting the pointers and
///   reading / writing / cloning / dropping / creating references or slices through them.
///   (This also applies to [`CloneFields`] - for its `clone_columns`, "`ptrs[i]`" means both of
///   `ptrs[i]`'s pointers.)
///
/// [`MultiVec`]: super::MultiVec
/// [`multi_vec!`]: super::multi_vec
/// [`Array<T>`]: Fields::Array
/// [`SHAPE`]: Fields::SHAPE
/// [`drop_columns`]: Fields::drop_columns
/// [`Layout`]: std::alloc::Layout
pub unsafe trait Fields: Sized {
    // The view types are bounded `Self: 'v`. Each borrows field data for `'v`, so `Self` must
    // outlive `'v` - a view can't outlast the data it points to.
    // (The slice views on [`SliceFields`] share this bound - same reason, plus one of their own.)

    /// References to each field of one element.
    type Ref<'v>
    where
        Self: 'v;

    /// Mutable references to each field of one element.
    type Mut<'v>
    where
        Self: 'v;

    /// One `T` per field: `[T; N]`, where `N` is the number of fields.
    ///
    /// See the trait docs for why this is an associated type. The sealed `CopyArray<T>` bound
    /// (only `[T; N]` implements it) grants everything generic code does with the array: indexing
    /// to read elements, copying by value (which also lets [`Shape`], storing an `Array<usize>`,
    /// be [`Copy`]), and `CopyArray::from_fn` to construct one.
    type Array<T: Copy>: CopyArray<T>;

    /// The precomputed [`Shape`] for these fields - alignment, element size, field offsets, etc,
    /// all derived from the field types' [`Layout`]s.
    ///
    /// The *only* layout information the implementor provides. The trusted input is the
    /// `[Layout; N]` passed to [`Shape::new`] (which must be the field types' layouts, in
    /// declaration order - see the safety contract). Every derived quantity is computed by
    /// `Shape::new`, so no generic code trusts an impl-supplied offset, size, or capacity.
    ///
    /// [`Layout`]: std::alloc::Layout
    const SHAPE: Shape<Self::Array<usize>>;

    /// Write `self`, splitting its fields across the pointed-to locations.
    ///
    /// Does not read or drop any previous values at the locations. A previously-initialized value
    /// of a type that needs dropping is leaked.
    ///
    /// # SAFETY
    ///
    /// Each `ptrs[i]` must be valid for writes of the `i`th field's type, and aligned for it.
    unsafe fn write(self, ptrs: Self::Array<NonNull<u8>>);

    /// Create an owned element, assembling `Self` from the pointed-to field values.
    ///
    /// The values are moved out of the pointed-to locations (a bitwise read).
    /// After this call, the locations must be treated as uninitialized.
    /// Reading or dropping the values through them again would be a double read / double drop.
    ///
    /// # SAFETY
    ///
    /// * Each `ptrs[i]` must be valid for reads of the `i`th field's type, aligned for it,
    ///   and point to an initialized value of it.
    /// * The pointed-to values must not be used (read or dropped) after this call,
    ///   unless they are first re-initialized.
    unsafe fn create_owned(ptrs: Self::Array<NonNull<u8>>) -> Self;

    /// Create references to each field of one element.
    ///
    /// # SAFETY
    ///
    /// * Each `ptrs[i]` must be valid for reads of the `i`th field's type, aligned for it,
    ///   and point to an initialized value of it.
    /// * The pointed-to values must not be mutated for the duration of lifetime `'v`.
    unsafe fn create_ref<'v>(ptrs: Self::Array<NonNull<u8>>) -> Self::Ref<'v>;

    /// Create mutable references to each field of one element.
    ///
    /// # SAFETY
    ///
    /// * Each `ptrs[i]` must be valid for reads and writes of the `i`th field's type,
    ///   aligned for it, and point to an initialized value of it.
    /// * The pointed-to values must not be accessed through any other pointer for the
    ///   duration of lifetime `'v`.
    unsafe fn create_mut<'v>(ptrs: Self::Array<NonNull<u8>>) -> Self::Mut<'v>;

    /// Drop the first `len` elements of each field's array, in place.
    ///
    /// If an element's `Drop` panics, all remaining elements (in that field's array and in
    /// later fields' arrays) are leaked (not dropped).
    ///
    /// # SAFETY
    ///
    /// * Each `ptrs[i]` must be aligned for the `i`th field's type, and valid for reads
    ///   and writes of `len` consecutive values of it, all initialized. (If `len == 0`,
    ///   a dangling aligned pointer is sufficient.)
    /// * The dropped values must not be used after this call.
    unsafe fn drop_columns(ptrs: Self::Array<NonNull<u8>>, len: usize);
}

/// A [`Fields`] type whose columns can be viewed as slices, indexed by `I`.
///
/// Split from [`Fields`] because the slice views name the *index* type - each field is an
/// `IndexSlice<I, [_]>`. The [`Fields`] implementor is the *element* struct (e.g. `Thing<'a>`),
/// which does not carry the index type or its lifetimes, so a [`Fields`] method could not name
/// them. Taking `I` as a trait parameter lets the impl bind those lifetimes -
/// `unsafe impl<'k, 'a> SliceFields<ThingId<'k>> for Thing<'a>` - where the [`Fields`] impl is
/// only `impl<'a> ... for Thing<'a>`.
///
/// A struct implements `SliceFields<I>` once per index type it is stored under. Each generated
/// table pairs one struct with one index type, so in practice that is exactly once.
///
/// # SAFETY
///
/// Implementations must uphold [`Fields`]' requirements - interpret each `ptrs[i]` as a pointer
/// to the `i`th field's type (same field order as [`SHAPE`]'s layout list), and do nothing beyond
/// casting the pointers and creating slices through them.
///
/// [`MultiVec`]: super::MultiVec
/// [`SHAPE`]: Fields::SHAPE
pub unsafe trait SliceFields<I: Idx>: Fields {
    /// Slices over each field's whole array.
    type Slices<'v>
    where
        Self: 'v;

    /// Mutable slices over each field's whole array.
    type SlicesMut<'v>
    where
        Self: 'v;

    /// Create slices over the first `len` elements of each field's array.
    ///
    /// # SAFETY
    ///
    /// * Each `ptrs[i]` must be aligned for the `i`th field's type, and valid for reads of
    ///   `len` consecutive values of it, all initialized. (If `len == 0`, a dangling
    ///   aligned pointer is sufficient.)
    /// * The pointed-to values must not be mutated for the duration of lifetime `'v`.
    unsafe fn create_slices<'v>(ptrs: Self::Array<NonNull<u8>>, len: usize) -> Self::Slices<'v>;

    /// Create mutable slices over the first `len` elements of each field's array.
    ///
    /// # SAFETY
    ///
    /// Same requirements as [`create_slices`], except the pointed-to values
    /// must not be accessed through any other pointer for the duration of lifetime `'v`.
    ///
    /// [`create_slices`]: Self::create_slices
    unsafe fn create_slices_mut<'v>(
        ptrs: Self::Array<NonNull<u8>>,
        len: usize,
    ) -> Self::SlicesMut<'v>;
}

/// A [`Fields`] type whose stored field values can also be cloned, column by column.
///
/// Implemented by the [`multi_vec!`] macro only for tables declared with `#[derive(Clone)]` -
/// a table without the derive gets no clone machinery at all. [`MultiVec`] is [`Clone`]
/// only where its fields struct implements this trait.
///
/// The `Clone` supertrait is always satisfied. The macro derives `Clone` on the fields struct
/// itself whenever the table derives it. The struct's own `Clone` impl is never invoked
/// (elements only ever exist as scattered field values), but since it is the *derived*,
/// structural impl - a divergent manual impl would conflict with it - field-wise and
/// whole-struct cloning are equivalent.
///
/// # SAFETY
///
/// Implementations must uphold [`Fields`]' requirements (interpret both of `ptrs[i]`'s
/// pointers as pointers to the `i`th field's type, and do nothing beyond casting and
/// cloning through them), and additionally:
///
/// * [`clone_columns`] must, on successful (non-panicking) return, have initialized the
///   first `len` elements of every destination column with valid values of the field
///   types. ([`MultiVec`]'s `Clone` impl relies on this to set the clone's length.)
///
/// [`MultiVec`]: super::MultiVec
/// [`multi_vec!`]: super::multi_vec
/// [`clone_columns`]: CloneFields::clone_columns
pub unsafe trait CloneFields: Fields + Clone {
    /// Clone the first `len` elements of each field's array, from its `src_ptr` column
    /// to its `dst_ptr` column.
    ///
    /// Each field's array is cloned element by element, running the field type's `Clone` impl
    /// (never skipped, even for `Copy` types - see `clone_column` in the `clone` module).
    ///
    /// If an element's `clone` panics, all elements already written to the destination
    /// columns are dropped during unwinding, via drop guards - see `ColumnDropGuard` in
    /// the `clone` module. Nothing is leaked.
    ///
    /// # SAFETY
    ///
    /// * Each `src_and_dst_ptrs[i].src_ptr` must be aligned for the `i`th field's type,
    ///   and valid for reads of `len` consecutive values of it, all initialized.
    /// * Each `src_and_dst_ptrs[i].dst_ptr` must be aligned for the `i`th field's type,
    ///   and valid for writes of `len` consecutive values of it.
    /// * The source and destination ranges must not overlap.
    /// * If `len == 0`, dangling aligned pointers are sufficient.
    unsafe fn clone_columns(src_and_dst_ptrs: Self::Array<SrcAndDstPtrs>, len: usize);
}
