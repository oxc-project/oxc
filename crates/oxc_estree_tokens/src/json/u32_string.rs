//! [`U32String`] type.
//! Used in JSON serialization.

use itoa::Buffer as ItoaBuffer;

use oxc_data_structures::assert_unchecked;

/// Maximum length of a decimal string representation of a `u32`.
const MAX_U32_LEN: usize = "4294967295".len(); // `u32::MAX` (10 bytes string)

/// Wrapper around [`itoa::Buffer`], which asserts that the formatted string is not longer than 10 bytes.
///
/// Purpose of this type is to inform the compiler that the `&str` has a short length,
/// which allows it to remove arithmetic overflow checks when concatenating multiple strings.
#[repr(transparent)]
pub struct U32String(ItoaBuffer);

#[expect(clippy::inline_always)]
impl U32String {
    /// Create a new [`U32String`].
    //
    // `#[inline(always)]` as it's a no-op.
    #[inline(always)]
    pub fn new() -> Self {
        Self(ItoaBuffer::new())
    }

    /// Use this [`U32String`] to format a `u32` as a string.
    /// The returned `&str` is guaranteed to be no longer than 10 bytes.
    //
    // `#[inline(always)]` because just delegates to `itoa`, and so that compiler can benefit from
    // the guaranteed short length of the string in the caller.
    #[inline(always)]
    pub fn format(&mut self, n: u32) -> &str {
        let s = self.0.format(n);
        // SAFETY: A `u32` converted to decimal string cannot have more than 10 digits
        unsafe { assert_unchecked!(s.len() <= MAX_U32_LEN) };
        s
    }
}
