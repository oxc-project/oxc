//! [`Shape`]: the precomputed shape of a [`MultiVec`]'s columns.
//!
//! [`MultiVec`]: super::MultiVec

use std::{
    alloc::Layout,
    array,
    ops::Index,
    ptr::{self, NonNull},
};

/// A field's size and column offset. Element of [`Shape::ordered_field_sizes_and_offsets`].
#[derive(Clone, Copy)]
pub struct FieldSizeAndOffset {
    /// Size of the field's type.
    pub size: usize,
    /// The field's column offset - the total size of all fields sorted before it.
    /// The column starts `capacity * offset` bytes into the allocation.
    pub offset: usize,
}

/// The shape of a [`MultiVec`]'s columns: every derived quantity the allocation and
/// addressing maths need, precomputed once from the field types' [`Layout`]s.
///
/// One value exists per fields struct, as the [`Fields::SHAPE`] associated const.
/// `A` is the fields struct's `[usize; N]` array type ([`Fields::Array<usize>`]) -
/// the field count `N` is a property of the implementor which generic code cannot name,
/// so it is threaded through the array *type* instead.
///
/// # "Can't lie"
///
/// The fields are private to this module, and [`new`] - which computes them all, and is
/// the only constructor - takes just the field layouts and derives everything else. So a
/// `Shape` cannot be forged or altered, even by the rest of this crate - every value it holds
/// is `new`'s output for *some* `[Layout; N]`, internally consistent by construction.
///
/// The only trusted input is that layout list, which [`Fields`]' safety contract requires to be
/// the layouts of the actual field types (the one thing only the implementor can know). Nothing
/// generic reads impl-supplied offsets, sizes, alignments, or capacities - they are all computed
/// here.
///
/// The scalar accessors are inherent `const fn`s, so generic code can use them in const
/// context (e.g. [`MultiVec::MAX_CAPACITY`]) - trait methods could not be called there.
///
/// [`MultiVec`]: super::MultiVec
/// [`Fields`]: super::fields::Fields
/// [`Fields::SHAPE`]: super::fields::Fields::SHAPE
/// [`Fields::Array<usize>`]: super::fields::Fields::Array
/// [`new`]: Shape::new
/// [`MultiVec::MAX_CAPACITY`]: super::MultiVec::MAX_CAPACITY
#[derive(Clone, Copy)]
pub struct Shape<A: CopyArray<usize>> {
    /// Maximum alignment of all field types. The allocation's alignment.
    align: usize,
    /// Total size of all fields of one element. The allocation's size is
    /// `capacity * element_size`.
    element_size: usize,
    /// Maximum capacity any allocation can hold, regardless of index type:
    /// `(isize::MAX - (align - 1)) / element_size` (allocations cannot exceed
    /// `isize::MAX` bytes, including the padding to round the size up to alignment -
    /// [`Layout`]'s invariant).
    max_alloc_capacity: usize,
    /// For each field (in declaration order): the total size of all fields sorted
    /// before it (descending alignment order, ties broken by declaration order).
    /// Field `i`'s column starts `capacity * field_offsets[i]` bytes into the allocation.
    field_offsets: A,
    /// Size of each field's type, in declaration order.
    field_sizes: A,
    /// Each field's `(size, offset)`, sorted ascending by column offset (memory order).
    /// `ordered_field_sizes_and_offsets[0].offset` is 0 (the first column),
    /// and offsets ascend from there.
    ordered_field_sizes_and_offsets: A::SameLength<FieldSizeAndOffset>,
}

impl<const N: usize> Shape<[usize; N]> {
    /// Compute the [`Shape`] for the given field layouts (in declaration order).
    ///
    /// Overflow of the total size aborts compilation (this is only called in `const`
    /// context).
    ///
    /// # Panics
    ///
    /// Panics if every field is zero-sized. Such a set would produce a zero-sized
    /// allocation layout, which is undefined behavior to allocate. Evaluated in const
    /// context, the panic is a compile-time error.
    pub const fn new(field_layouts: [Layout; N]) -> Self {
        let mut align = 1;
        let mut element_size = 0;
        let mut i = 0;
        while i < N {
            if field_layouts[i].align() > align {
                align = field_layouts[i].align();
            }
            element_size += field_layouts[i].size();
            i += 1;
        }

        assert!(
            element_size > 0,
            "`MultiVec` does not support field sets consisting only of zero-sized types",
        );

        let max_alloc_capacity = (isize::MAX as usize - (align - 1)) / element_size;

        let mut field_offsets = [0; N];
        let mut field_sizes = [0; N];
        let mut ordered_field_sizes_and_offsets = [FieldSizeAndOffset { size: 0, offset: 0 }; N];
        let mut i = 0;
        while i < N {
            let layout = field_layouts[i];
            field_sizes[i] = layout.size();

            // Sum the sizes of all fields sorted before field `i` - those with greater
            // alignment, plus those with equal alignment declared earlier - and count
            // them. The count is field `i`'s position in memory order.
            let mut offset = 0;
            let mut fields_before_count = 0;
            let mut j = 0;
            while j < N {
                let other_layout = field_layouts[j];
                if other_layout.align() > layout.align()
                    || (other_layout.align() == layout.align() && j < i)
                {
                    offset += other_layout.size();
                    fields_before_count += 1;
                }
                j += 1;
            }

            field_offsets[i] = offset;

            // "Sorted before" is a strict total order over the fields, so every field has a
            // distinct `fields_before_count` - each lands in its own slot, and the array holds
            // every field exactly once, sorted by ascending offset.
            ordered_field_sizes_and_offsets[fields_before_count] =
                FieldSizeAndOffset { size: layout.size(), offset };

            i += 1;
        }

        Self {
            align,
            element_size,
            max_alloc_capacity,
            field_offsets,
            field_sizes,
            ordered_field_sizes_and_offsets,
        }
    }
}

impl<A: CopyArray<usize>> Shape<A> {
    /// The allocation's alignment. Maximum alignment of all field types.
    ///
    /// Guaranteed non-zero and a power of two.
    pub const fn align(&self) -> usize {
        self.align
    }

    /// Get a dangling pointer with the same alignment as the allocation.
    #[expect(clippy::missing_panics_doc, reason = "`self.align()` never returns 0")]
    pub const fn dangling_ptr(&self) -> NonNull<u8> {
        NonNull::new(ptr::without_provenance_mut(self.align)).unwrap()
    }

    /// Total size of all fields of one element.
    ///
    /// The allocation's size is `capacity * element_size()`.
    pub const fn element_size(&self) -> usize {
        self.element_size
    }

    /// Maximum capacity any allocation can hold, regardless of index type.
    ///
    /// ([`MultiVec::MAX_CAPACITY`] additionally limits capacity to the index type's range.)
    ///
    /// [`MultiVec::MAX_CAPACITY`]: super::MultiVec::MAX_CAPACITY
    pub const fn max_alloc_capacity(&self) -> usize {
        self.max_alloc_capacity
    }

    /// For each field (in declaration order): the total size of all fields sorted
    /// before it (descending alignment order, ties broken by declaration order).
    ///
    /// Field `i`'s column starts `capacity * field_offsets()[i]` bytes into the allocation.
    ///
    /// Returned by value (the array is [`Copy`]) rather than as a slice, so callers can
    /// hold it in a `const` and index it as a fixed-size array - the hot paths rely on the
    /// indices const-folding (which keeps the iterators auto-vectorizing).
    pub const fn field_offsets(&self) -> A {
        self.field_offsets
    }

    /// Size of each field's type, in declaration order.
    ///
    /// Returned by value, for the same reason as [`field_offsets`].
    ///
    /// [`field_offsets`]: Self::field_offsets
    pub const fn field_sizes(&self) -> A {
        self.field_sizes
    }

    /// Each field's [`FieldSizeAndOffset`], sorted ascending by column offset (memory order).
    ///
    /// Every field appears exactly once. `ordered_field_sizes_and_offsets()[0].offset` is 0
    /// (the first column), and offsets ascend from there.
    ///
    /// Returned by value, for the same reason as [`field_offsets`].
    ///
    /// [`field_offsets`]: Self::field_offsets
    pub const fn ordered_field_sizes_and_offsets(&self) -> A::SameLength<FieldSizeAndOffset> {
        self.ordered_field_sizes_and_offsets
    }
}

/// A fixed-size array `[T; N]` whose element type is [`Copy`].
///
/// Sealed via the private [`Sealed`] supertrait - `[T; N]` is the only implementor (no type
/// outside this module can satisfy `Sealed`, and inside it only the blanket `[T; N]` impl
/// below does). This is what lets [`Fields::Array`] name `[T; N]` through a bound
/// (`type Array<T: Copy>: CopyArray<T>`) without the field count `N` appearing as a const
/// generic parameter, while still guaranteeing the associated type really is an array.
///
/// The supertraits cover everything generic code does with one of these arrays -
/// copy it by value ([`Copy`]), and index it ([`Index`], counted up to [`LEN`]).
///
/// [`from_fn`] constructs one - generic code cannot write an array literal
/// of statically unknown length.
///
/// [`Fields::Array`]: super::fields::Fields::Array
/// [`LEN`]: CopyArray::LEN
/// [`from_fn`]: CopyArray::from_fn
#[expect(private_bounds)]
pub trait CopyArray<T: Copy>: Copy + Index<usize, Output = T> + Sealed {
    /// The array's length, `N`.
    const LEN: usize;

    /// The same array type with element type `U` instead -
    /// `[U; N]`, for the implementor's length `N`.
    type SameLength<U: Copy>: CopyArray<U>;

    /// Create an array whose `i`th element is `f(i)`.
    ///
    /// Exactly [`std::array::from_fn`], for the implementor's length `N`.
    fn from_fn(f: impl FnMut(usize) -> T) -> Self;
}

impl<T: Copy, const N: usize> CopyArray<T> for [T; N] {
    const LEN: usize = N;

    type SameLength<U: Copy> = [U; N];

    #[inline]
    fn from_fn(f: impl FnMut(usize) -> T) -> Self {
        array::from_fn(f)
    }
}

trait Sealed {}

impl<T: Copy, const N: usize> Sealed for [T; N] {}
