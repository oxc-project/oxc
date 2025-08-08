//! Extension trait for pointers. See [`PointerExt`].

// TODO: Once our MSRV reaches v1.87.0, remove this trait and just use `offset_from_unsigned` directly.
// `#[expect(clippy::incompatible_msrv)]` below will trigger a warning when MSRV is bumped to 1.87.0.

// All methods either delegate to Rust's methods, or boil down to a single instruction
#![expect(clippy::inline_always)]

use std::ptr::NonNull;

/// Extension trait for pointers.
///
/// Rust v1.87.0 introduced `offset_from_unsigned` and `byte_offset_from_unsigned` methods for pointers.
/// <https://doc.rust-lang.org/std/primitive.pointer.html#method.offset_from_unsigned>
///
/// These are implemented as intrinsics, and potentially gives the compiler more information
/// with which to make optimizations, compared to either:
///
/// * `end.offset_from(start) as usize`
/// * `usize::try_from(end.offset_from(start)).unwrap_unchecked()`
///
/// We want to use these methods, but they're not available on our current MSRV.
///
/// This trait provides alternatives `offset_from_usize` and `byte_offset_from_usize`.
///
/// * On Rust v1.87.0+, they use Rust's native methods.
/// * On earlier versions of Rust, they use a fallback.
#[expect(private_bounds)]
pub trait PointerExt: PointerExtImpl {
    /// Calculates the distance between two pointers within the same allocation,
    /// *where it's known that `self` is equal to or greater than `origin`*.
    /// The returned value is in units of `T`: the distance in bytes is divided by `size_of::<T>()`.
    ///
    /// # SAFETY
    ///
    /// * The distance between the pointers must be non-negative (`self >= origin`).
    ///
    /// * *All* the safety conditions of `offset_from` apply to this method as well;
    ///   see it for the full details.
    ///
    /// See <https://doc.rust-lang.org/std/primitive.pointer.html#method.offset_from_unsigned>
    /// for full details.
    #[inline(always)]
    unsafe fn offset_from_usize(self, origin: Self) -> usize {
        // SAFETY: Same constraints as this method
        unsafe { self.offset_from_usize_impl(origin) }
    }

    /// Calculates the distance between two pointers within the same allocation,
    /// *where it's known that `self` is equal to or greater than `origin`*.
    /// The returned value is in units of **bytes**.
    ///
    /// # SAFETY
    ///
    /// * The distance between the pointers must be non-negative (`self >= origin`).
    ///
    /// * *All* the safety conditions of `offset_from` apply to this method as well;
    ///   see it for the full details.
    ///
    /// See <https://doc.rust-lang.org/std/primitive.pointer.html#method.byte_offset_from_unsigned>
    /// for full details.
    #[inline(always)]
    unsafe fn byte_offset_from_usize(self, origin: Self) -> usize {
        // SAFETY: Same constraints as this method
        unsafe { self.byte_offset_from_usize_impl(origin) }
    }
}

impl<T> PointerExt for *const T {}

impl<T> PointerExt for *mut T {}

impl<T> PointerExt for NonNull<T> {}

/// Trait that does the actual work.
///
/// This trait is not `pub`, to prevent [`PointerExt`] being implemented on other types
/// outside this module.
///
/// The other purpose of this trait is to avoid repeating the docs for the methods 12 times.
trait PointerExtImpl: Sized {
    unsafe fn offset_from_usize_impl(self, origin: Self) -> usize;
    unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize;
}

/// Native version - just delegates to Rust's methods.
#[rustversion::since(1.87.0)]
#[expect(clippy::undocumented_unsafe_blocks)]
const _: () = {
    impl<T> PointerExtImpl for *const T {
        #[inline(always)]
        unsafe fn offset_from_usize_impl(self, origin: Self) -> usize {
            unsafe { self.offset_from_unsigned(origin) }
        }

        #[inline(always)]
        unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize {
            unsafe { self.byte_offset_from_unsigned(origin) }
        }
    }

    impl<T> PointerExtImpl for *mut T {
        #[inline(always)]
        unsafe fn offset_from_usize_impl(self, origin: Self) -> usize {
            unsafe { self.offset_from_unsigned(origin) }
        }

        #[inline(always)]
        unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize {
            unsafe { self.byte_offset_from_unsigned(origin) }
        }
    }

    impl<T> PointerExtImpl for NonNull<T> {
        #[inline(always)]
        unsafe fn offset_from_usize_impl(self, origin: Self) -> usize {
            unsafe { self.offset_from_unsigned(origin) }
        }

        #[inline(always)]
        unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize {
            unsafe { self.byte_offset_from_unsigned(origin) }
        }
    }
};

/// Fallback version. This is the best we can do prior to Rust v1.87.0.
#[rustversion::before(1.87.0)]
const _: () = {
    impl<T> PointerExtImpl for *const T {
        #[inline(always)]
        unsafe fn offset_from_usize_impl(self, origin: Self) -> usize {
            // SAFETY: Has same safety requirements as native `offset_from_unsigned` method
            unsafe { usize::try_from(self.offset_from(origin)).unwrap_unchecked() }
        }

        #[inline(always)]
        unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize {
            // SAFETY: Has same safety requirements as native `byte_offset_from_unsigned` method
            unsafe { usize::try_from(self.byte_offset_from(origin)).unwrap_unchecked() }
        }
    }

    impl<T> PointerExtImpl for *mut T {
        #[inline(always)]
        unsafe fn offset_from_usize_impl(self, origin: Self) -> usize {
            // SAFETY: Has same safety requirements as native `offset_from_unsigned` method
            unsafe { usize::try_from(self.offset_from(origin)).unwrap_unchecked() }
        }

        #[inline(always)]
        unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize {
            // SAFETY: Has same safety requirements as native `byte_offset_from_unsigned` method
            unsafe { usize::try_from(self.byte_offset_from(origin)).unwrap_unchecked() }
        }
    }

    impl<T> PointerExtImpl for NonNull<T> {
        #[inline(always)]
        unsafe fn offset_from_usize_impl(self, origin: Self) -> usize {
            // SAFETY: Has same safety requirements as native `offset_from_unsigned` method
            unsafe { usize::try_from(self.offset_from(origin)).unwrap_unchecked() }
        }

        #[inline(always)]
        unsafe fn byte_offset_from_usize_impl(self, origin: Self) -> usize {
            // SAFETY: Has same safety requirements as native `byte_offset_from_unsigned` method
            unsafe { usize::try_from(self.byte_offset_from(origin)).unwrap_unchecked() }
        }
    }
};
