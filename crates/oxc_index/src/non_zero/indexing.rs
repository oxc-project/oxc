use super::{slice::NonZeroIndexSlice, NonZeroIdx};

mod private_slice_index {
    pub trait Sealed {}
}

/// This is the equivalent of the sealed `core::slice::SliceIndex` trait. It
/// cannot be overridden from user, code nor should it normally need use
/// directly (Outside of trait bounds, I guess).
pub trait NonZeroIdxSliceIndex<I: NonZeroIdx, T>: private_slice_index::Sealed {
    type Output: ?Sized;

    fn get(self, slice: &NonZeroIndexSlice<I, [T]>) -> Option<&Self::Output>;
    fn get_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> Option<&mut Self::Output>;

    fn index(self, slice: &NonZeroIndexSlice<I, [T]>) -> &Self::Output;
    fn index_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> &mut Self::Output;
}

// Does this defeat the point of sealing?
impl<I: NonZeroIdx> private_slice_index::Sealed for I {}

impl<I: NonZeroIdx, T> NonZeroIdxSliceIndex<I, T> for I {
    type Output = T;

    #[inline]
    fn get(self, slice: &NonZeroIndexSlice<I, [T]>) -> Option<&Self::Output> {
        slice.raw.get(self.index())
    }
    #[inline]
    fn get_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self.index())
    }

    #[inline]
    fn index(self, slice: &NonZeroIndexSlice<I, [T]>) -> &Self::Output {
        &slice.raw[self.index()]
    }

    #[inline]
    fn index_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> &mut Self::Output {
        &mut slice.raw[self.index()]
    }
}

macro_rules! range_slice {
    ($r:ty) => {
        impl<I: NonZeroIdx, T: Default> NonZeroIdxSliceIndex<I, T> for $r {
            type Output = NonZeroIndexSlice<I, [T]>;

            #[inline]
            fn get(self, slice: &NonZeroIndexSlice<I, [T]>) -> Option<&Self::Output> {
                slice.raw.get(self.into_range()).map(NonZeroIndexSlice::new)
            }
            #[inline]
            fn get_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> Option<&mut Self::Output> {
                slice.raw.get_mut(self.into_range()).map(NonZeroIndexSlice::new_mut)
            }

            #[inline]
            fn index(self, slice: &NonZeroIndexSlice<I, [T]>) -> &Self::Output {
                NonZeroIndexSlice::new(&slice.raw[self.into_range()])
            }
            #[inline]
            fn index_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> &mut Self::Output {
                NonZeroIndexSlice::new_mut(&mut slice.raw[self.into_range()])
            }
        }
    };
}

impl<I: NonZeroIdx> private_slice_index::Sealed for core::ops::Range<I> {}
impl<I: NonZeroIdx> private_slice_index::Sealed for core::ops::RangeFrom<I> {}
impl<I: NonZeroIdx> private_slice_index::Sealed for core::ops::RangeTo<I> {}
impl<I: NonZeroIdx> private_slice_index::Sealed for core::ops::RangeInclusive<I> {}
impl<I: NonZeroIdx> private_slice_index::Sealed for core::ops::RangeToInclusive<I> {}

range_slice!(core::ops::Range<I>);
range_slice!(core::ops::RangeFrom<I>);
range_slice!(core::ops::RangeTo<I>);
range_slice!(core::ops::RangeInclusive<I>);
range_slice!(core::ops::RangeToInclusive<I>);
// range_slice!(core::ops::RangeFull);
impl private_slice_index::Sealed for core::ops::RangeFull {}
impl<I: NonZeroIdx, T> NonZeroIdxSliceIndex<I, T> for core::ops::RangeFull {
    type Output = NonZeroIndexSlice<I, [T]>;

    #[inline]
    fn get(self, slice: &NonZeroIndexSlice<I, [T]>) -> Option<&Self::Output> {
        Some(slice)
    }

    #[inline]
    fn get_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> Option<&mut Self::Output> {
        Some(slice)
    }

    #[inline]
    fn index(self, slice: &NonZeroIndexSlice<I, [T]>) -> &Self::Output {
        slice
    }

    #[inline]
    fn index_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> &mut Self::Output {
        slice
    }
}

impl private_slice_index::Sealed for usize {}
// As an ergonomic concession, implement this for `usize` as well, it's too painful without
impl<I: NonZeroIdx, T> NonZeroIdxSliceIndex<I, T> for usize {
    type Output = T;

    #[inline]
    fn get(self, slice: &NonZeroIndexSlice<I, [T]>) -> Option<&Self::Output> {
        slice.raw.get(self)
    }
    #[inline]
    fn get_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> Option<&mut Self::Output> {
        slice.raw.get_mut(self)
    }

    #[inline]
    fn index(self, slice: &NonZeroIndexSlice<I, [T]>) -> &Self::Output {
        &slice.raw[self]
    }
    #[inline]
    fn index_mut(self, slice: &mut NonZeroIndexSlice<I, [T]>) -> &mut Self::Output {
        &mut slice.raw[self]
    }
}

/// This trait to function in API signatures where `Vec<T>` or `[T]` use `R:
/// RangeBounds<usize>`. There are blanket implementations for the basic range
/// types in `core::ops` for all NonZeroIdx types. e.g. `Range<I: NonZeroIdx>`, `RangeFrom<I:
/// NonZeroIdx>`, `RangeTo<I: NonZeroIdx>`, etc all implement it.
///
/// IMO it's unfortunate that this needs to be present in the API, but it
/// doesn't hurt that much.
pub trait NonZeroIdxRangeBounds<I>: private_range_bounds::Sealed
where
    I: NonZeroIdx,
{
    type Range: core::ops::RangeBounds<usize>;
    fn into_range(self) -> Self::Range;
}

mod private_range_bounds {
    pub trait Sealed {}
}

impl<I: NonZeroIdx> private_range_bounds::Sealed for core::ops::Range<I> {}
impl<I: NonZeroIdx> private_range_bounds::Sealed for core::ops::RangeFrom<I> {}
impl<I: NonZeroIdx> private_range_bounds::Sealed for core::ops::RangeTo<I> {}
impl<I: NonZeroIdx> private_range_bounds::Sealed for core::ops::RangeInclusive<I> {}
impl<I: NonZeroIdx> private_range_bounds::Sealed for core::ops::RangeToInclusive<I> {}
impl private_range_bounds::Sealed for core::ops::RangeFull {}

impl<I: NonZeroIdx> NonZeroIdxRangeBounds<I> for core::ops::Range<I> {
    type Range = core::ops::Range<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        self.start.index()..self.end.index()
    }
}

impl<I: NonZeroIdx> NonZeroIdxRangeBounds<I> for core::ops::RangeFrom<I> {
    type Range = core::ops::RangeFrom<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        self.start.index()..
    }
}

impl<I: NonZeroIdx> NonZeroIdxRangeBounds<I> for core::ops::RangeFull {
    type Range = core::ops::RangeFull;
    #[inline]
    fn into_range(self) -> Self::Range {
        self
    }
}

impl<I: NonZeroIdx> NonZeroIdxRangeBounds<I> for core::ops::RangeTo<I> {
    type Range = core::ops::RangeTo<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        ..self.end.index()
    }
}

impl<I: NonZeroIdx> NonZeroIdxRangeBounds<I> for core::ops::RangeInclusive<I> {
    type Range = core::ops::RangeInclusive<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        self.start().index()..=self.end().index()
    }
}

impl<I: NonZeroIdx> NonZeroIdxRangeBounds<I> for core::ops::RangeToInclusive<I> {
    type Range = core::ops::RangeToInclusive<usize>;
    #[inline]
    fn into_range(self) -> Self::Range {
        ..=self.end.index()
    }
}

impl<I, R, T> core::ops::Index<R> for NonZeroIndexSlice<I, [T]>
where
    I: NonZeroIdx,
    R: NonZeroIdxSliceIndex<I, T>,
{
    type Output = R::Output;
    #[inline]
    fn index(&self, index: R) -> &R::Output {
        index.index(self)
    }
}

impl<I, R, T> core::ops::IndexMut<R> for NonZeroIndexSlice<I, [T]>
where
    I: NonZeroIdx,
    R: NonZeroIdxSliceIndex<I, T>,
{
    #[inline]
    fn index_mut(&mut self, index: R) -> &mut R::Output {
        index.index_mut(self)
    }
}
