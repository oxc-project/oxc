use std::{fmt, marker::PhantomData, ops::Deref, ptr::NonNull};

/// A `&'a [T]` stored as a pointer and a `u32` length.
///
/// It takes 12 bytes and is aligned to 4 bytes, where a `&[T]` takes 16 bytes.
/// This leaves room for the tag of [`crate::FormatElement`], which keeps the element at 16 bytes.
#[repr(C, packed(4))]
pub struct ThinSlice<'a, T> {
    ptr: NonNull<T>,
    len: u32,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> ThinSlice<'a, T> {
    /// # Panics
    /// If `slice` is longer than `u32::MAX`.
    #[inline]
    pub const fn new(slice: &'a [T]) -> Self {
        assert!(slice.len() <= u32::MAX as usize, "slice longer than `u32::MAX`");
        #[expect(clippy::cast_possible_truncation)]
        Self { ptr: NonNull::from_ref(slice).cast(), len: slice.len() as u32, _marker: PhantomData }
    }

    #[inline]
    pub fn get(self) -> &'a [T] {
        // SAFETY: `ptr` and `len` come from a `&'a [T]` in `new`
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }
}

impl<T> Clone for ThinSlice<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ThinSlice<'_, T> {}

impl<T> Deref for ThinSlice<'_, T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        (*self).get()
    }
}

impl<T: PartialEq> PartialEq for ThinSlice<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for ThinSlice<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for ThinSlice<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// A `&'a str` stored as a pointer and a `u32` length, see `ThinSlice`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ThinStr<'a>(ThinSlice<'a, u8>);

impl<'a> ThinStr<'a> {
    #[inline]
    pub(crate) const fn new(text: &'a str) -> Self {
        Self(ThinSlice::new(text.as_bytes()))
    }

    #[inline]
    pub fn as_str(self) -> &'a str {
        // SAFETY: The bytes come from a `&'a str` in `new`
        unsafe { std::str::from_utf8_unchecked(self.0.get()) }
    }
}

impl Deref for ThinStr<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        (*self).as_str()
    }
}

impl fmt::Debug for ThinStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
