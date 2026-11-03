use std::ptr;

/// Array of integers which `Vec128::read_low_unaligned` can read.
///
/// # SAFETY
///
/// Implementing type must be 8 bytes, and all of those bytes must be initialized.
pub unsafe trait IntArray64 {
    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for unaligned reads of 8 bytes.
    #[expect(clippy::inline_always, reason = "no-op at runtime")]
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        ptr::from_ref(self).cast::<u8>()
    }
}

/// Array of integers which `Vec128::read_unaligned` can read.
///
/// # SAFETY
///
/// Implementing type must be 16 bytes, and all of those bytes must be initialized.
pub unsafe trait IntArray128 {
    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for unaligned reads of 16 bytes.
    #[expect(clippy::inline_always, reason = "no-op at runtime")]
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        ptr::from_ref(self).cast::<u8>()
    }
}

/// Array of integers which `Vec256::read_unaligned` can read.
///
/// # SAFETY
///
/// Implementing type must be 32 bytes, and all of those bytes must be initialized.
pub unsafe trait IntArray256 {
    /// Get a `*const u8` pointing to start of the array.
    ///
    /// This pointer is guaranteed to be valid for unaligned reads of 32 bytes.
    #[expect(clippy::inline_always, reason = "no-op at runtime")]
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        ptr::from_ref(self).cast::<u8>()
    }
}

/// Implement `$trait` for an array of each integer type listed, totaling `$bytes` bytes.
///
/// `$bytes` must be a multiple of the size of every integer type listed.
macro_rules! impl_int_arrays {
    ($trait:ident, $bytes:literal => $($int:ty),+) => {
        $(
            // SAFETY: Array of `$int`s which totals `$bytes` bytes. Integers have no padding.
            unsafe impl $trait for [$int; $bytes / size_of::<$int>()] {}
        )+
    };
}

impl_int_arrays!(IntArray64, 8 => u8, i8, u16, i16, u32, i32, u64, i64);
impl_int_arrays!(IntArray128, 16 => u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
impl_int_arrays!(IntArray256, 32 => u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
