use std::{cmp::Ordering, ptr::NonNull as NativeNonNull};

/// Wrapper around `NonNull<T>`, which adds methods `add`, `sub`, `offset_from` and `byte_offset_from`.
/// These methods exist on `std::ptr::NonNull`, and became stable in Rust 1.80.0, but are not yet
/// stable in our MSRV.
///
/// These methods are much cleaner than the workarounds required in older Rust versions,
/// and make code using them easier to understand.
///
/// Once we bump MSRV and these methods are natively supported, this type can be removed.
/// `#[expect(clippy::incompatible_msrv)]` on `non_null_add_is_not_stable` below will trigger
/// a lint warning when that happens.
/// Then this module can be deleted, and all uses of this type can be switched to `std::ptr::NonNull`.
#[derive(Debug)]
pub struct NonNull<T>(NativeNonNull<T>);

#[cfg(clippy)]
#[expect(clippy::incompatible_msrv)]
unsafe fn _non_null_add_is_not_stable(ptr: NativeNonNull<u8>) -> NativeNonNull<u8> {
    ptr.add(1)
}

impl<T> NonNull<T> {
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self(NativeNonNull::new_unchecked(ptr))
    }

    #[inline]
    pub const fn dangling() -> Self {
        Self(NativeNonNull::dangling())
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }

    #[inline]
    pub const fn cast<U>(self) -> NonNull<U> {
        // SAFETY: `self` is non-null, so it's still non-null after casting
        unsafe { NonNull::new_unchecked(self.as_ptr().cast()) }
    }

    #[inline]
    pub const unsafe fn add(self, count: usize) -> Self {
        NonNull(NativeNonNull::new_unchecked(self.as_ptr().add(count)))
    }

    #[inline]
    pub const unsafe fn sub(self, count: usize) -> Self {
        NonNull(NativeNonNull::new_unchecked(self.as_ptr().sub(count)))
    }

    #[inline]
    pub const unsafe fn offset_from(self, origin: Self) -> isize {
        self.as_ptr().offset_from(origin.as_ptr())
    }

    #[inline]
    pub const unsafe fn byte_offset_from(self, origin: Self) -> isize {
        self.as_ptr().byte_offset_from(origin.as_ptr())
    }

    #[inline]
    pub const unsafe fn as_ref<'t>(self) -> &'t T {
        self.0.as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<'t>(mut self) -> &'t mut T {
        self.0.as_mut()
    }
}

impl<T> Copy for NonNull<T> {}

impl<T> Clone for NonNull<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for NonNull<T> {}

impl<T> PartialEq for NonNull<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Ord for NonNull<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}

impl<T> PartialOrd for NonNull<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
