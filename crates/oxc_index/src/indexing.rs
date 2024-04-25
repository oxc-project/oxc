use crate::{Idx, IndexSlice};

mod private_slice_index {
    pub trait Sealed {}
}

/// This is the equivalent of the sealed `core::slice::SliceIndex` trait. It
/// cannot be overridden from user, code nor should it normally need use
/// directly (Outside of trait bounds, I guess).
pub trait IdxSliceIndex<I: Idx, T>: private_slice_index::Sealed {
    type Output: ?Sized;

    fn get(self, slice: &IndexSlice<I, [T]>) -> Option<&Self::Output>;
    fn get_mut(self, slice: &mut IndexSlice<I, [T]>) -> Option<&mut Self::Output>;

    fn index(self, slice: &IndexSlice<I, [T]>) -> &Self::Output;
    fn index_mut(self, slice: &mut IndexSlice<I, [T]>) -> &mut Self::Output;
}

// Does this defeat the point of sealing?
impl<I: Idx> private_slice_index::Sealed for I {}

impl<I: Idx, T> IdxSliceIndex<I, T> for I {
    type Output = T;

    #[inline]
    fn get(self, slice: &IndexSlice<I, [T]>) -> Option<&Self::Output> {
        slice.raw.get(self.index())
    }
    #[inline]
    fn get_mut(self, slice: &mut IndexSlice<I, [T]>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self.index())
    }

    #[inline]
    fn index(self, slice: &IndexSlice<I, [T]>) -> &Self::Output {
        &slice.raw[self.index()]
    }

    #[inline]
    fn index_mut(self, slice: &mut IndexSlice<I, [T]>) -> &mut Self::Output {
        &mut slice.raw[self.index()]
    }
}

macro_rules! range_slice {
    ($r:ty) => {
        impl<I: Idx, T> IdxSliceIndex<I, T> for $r {
            type Output = IndexSlice<I, [T]>;

            #[inline]
            fn get(self, slice: &IndexSlice<I, [T]>) -> Option<&Self::Output> {
                slice.raw.get(self.into_range()).map(IndexSlice::new)
            }
            #[inline]
            fn get_mut(self, slice: &mut IndexSlice<I, [T]>) -> Option<&mut Self::Output> {
                slice
                    .raw
                    .get_mut(self.into_range())
                    .map(IndexSlice::new_mut)
            }

            #[inline]
            fn index(self, slice: &IndexSlice<I, [T]>) -> &Self::Output {
                IndexSlice::new(&slice.raw[self.into_range()])
            }
            #[inline]
            fn index_mut(self, slice: &mut IndexSlice<I, [T]>) -> &mut Self::Output {
                IndexSlice::new_mut(&mut slice.raw[self.into_range()])
            }
        }
    };
}

impl<I: Idx> private_slice_index::Sealed for core::ops::Range<I> {}
impl<I: Idx> private_slice_index::Sealed for core::ops::RangeFrom<I> {}
impl<I: Idx> private_slice_index::Sealed for core::ops::RangeTo<I> {}
impl<I: Idx> private_slice_index::Sealed for core::ops::RangeInclusive<I> {}
impl<I: Idx> private_slice_index::Sealed for core::ops::RangeToInclusive<I> {}

range_slice!(core::ops::Range<I>);
range_slice!(core::ops::RangeFrom<I>);
range_slice!(core::ops::RangeTo<I>);
range_slice!(core::ops::RangeInclusive<I>);
range_slice!(core::ops::RangeToInclusive<I>);
// range_slice!(core::ops::RangeFull);
impl private_slice_index::Sealed for core::ops::RangeFull {}
impl<I: Idx, T> IdxSliceIndex<I, T> for core::ops::RangeFull {
    type Output = IndexSlice<I, [T]>;

    #[inline]
    fn get(self, slice: &IndexSlice<I, [T]>) -> Option<&Self::Output> {
        Some(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut IndexSlice<I, [T]>) -> Option<&mut Self::Output> {
        Some(slice)
    }

    #[inline]
    fn index(self, slice: &IndexSlice<I, [T]>) -> &Self::Output {
        slice
    }

    #[inline]
    fn index_mut(self, slice: &mut IndexSlice<I, [T]>) -> &mut Self::Output {
        slice
    }
}

impl private_slice_index::Sealed for usize {}
// As an ergonomic concession, implement this for `usize` as well, it's too painful without
impl<I: Idx, T> IdxSliceIndex<I, T> for usize {
    type Output = T;

    #[inline]
    fn get(self, slice: &IndexSlice<I, [T]>) -> Option<&Self::Output> {
        slice.raw.get(self)
    }
    #[inline]
    fn get_mut(self, slice: &mut IndexSlice<I, [T]>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self)
    }

    #[inline]
    fn index(self, slice: &IndexSlice<I, [T]>) -> &Self::Output {
        &slice.raw[self]
    }
    #[inline]
    fn index_mut(self, slice: &mut IndexSlice<I, [T]>) -> &mut Self::Output {
        &mut slice.raw[self]
    }
}

/// This trait to function in API signatures where `Vec<T>` or `[T]` use `R:
/// RangeBounds<usize>`. There are blanket implementations for the basic range
/// types in `core::ops` for all Idx types. e.g. `Range<I: Idx>`, `RangeFrom<I:
/// Idx>`, `RangeTo<I: Idx>`, etc all implement it.
///
/// IMO it's unfortunate that this needs to be present in the API, but it
/// doesn't hurt that much.
pub trait IdxRangeBounds<I>: private_range_bounds::Sealed
where
    I: Idx,
{
    type Range: core::ops::RangeBounds<usize>;
    fn into_range(self) -> Self::Range;
}

mod private_range_bounds {
    pub trait Sealed {}
}

impl<I: Idx> private_range_bounds::Sealed for core::ops::Range<I> {}
impl<I: Idx> private_range_bounds::Sealed for core::ops::RangeFrom<I> {}
impl<I: Idx> private_range_bounds::Sealed for core::ops::RangeTo<I> {}
impl<I: Idx> private_range_bounds::Sealed for core::ops::RangeInclusive<I> {}
impl<I: Idx> private_range_bounds::Sealed for core::ops::RangeToInclusive<I> {}
impl private_range_bounds::Sealed for core::ops::RangeFull {}

impl<I: Idx> IdxRangeBounds<I> for core::ops::Range<I> {
    type Range = core::ops::Range<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        self.start.index()..self.end.index()
    }
}

impl<I: Idx> IdxRangeBounds<I> for core::ops::RangeFrom<I> {
    type Range = core::ops::RangeFrom<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        self.start.index()..
    }
}

impl<I: Idx> IdxRangeBounds<I> for core::ops::RangeFull {
    type Range = core::ops::RangeFull;
    #[inline]
    fn into_range(self) -> Self::Range {
        self
    }
}

impl<I: Idx> IdxRangeBounds<I> for core::ops::RangeTo<I> {
    type Range = core::ops::RangeTo<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        ..self.end.index()
    }
}

impl<I: Idx> IdxRangeBounds<I> for core::ops::RangeInclusive<I> {
    type Range = core::ops::RangeInclusive<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        self.start().index()..=self.end().index()
    }
}

impl<I: Idx> IdxRangeBounds<I> for core::ops::RangeToInclusive<I> {
    type Range = core::ops::RangeToInclusive<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        ..=self.end.index()
    }
}

impl<I, R, T> core::ops::Index<R> for IndexSlice<I, [T]>
where
    I: Idx,
    R: IdxSliceIndex<I, T>,
{
    type Output = R::Output;
    #[inline]
    fn index(&self, index: R) -> &R::Output {
        index.index(self)
    }
}

impl<I, R, T> core::ops::IndexMut<R> for IndexSlice<I, [T]>
where
    I: Idx,
    R: IdxSliceIndex<I, T>,
{
    #[inline]
    fn index_mut(&mut self, index: R) -> &mut R::Output {
        index.index_mut(self)
    }
}
