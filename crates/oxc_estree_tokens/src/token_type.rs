use oxc_data_structures::assert_unchecked;

/// Maximum length of a `TokenType` string representation.
/// `PrivateIdentifier` and `RegularExpression` are the longest.
const TOKEN_TYPE_MAX_LEN: usize = 17;

/// Token type.
///
/// Just a wrapper around a `&'static str`.
/// Purpose of this type is to inform the compiler that the `&str` has a short length,
/// which allows it to remove bounds checks when concatenating multiple strings.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct TokenType(&'static str);

#[expect(clippy::inline_always)]
impl TokenType {
    /// Create a new [`TokenType`].
    // `#[inline(always)]` so the assertion can be optimized out.
    #[inline(always)]
    pub const fn new(name: &'static str) -> Self {
        assert!(name.len() <= TOKEN_TYPE_MAX_LEN);
        Self(name)
    }

    /// Get the string representation of this [`TokenType`].
    // `#[inline(always)]` as this is a no-op at runtime, and so that compiler can benefit from
    // the guaranteed short length of the string in the caller.
    #[inline(always)]
    pub const fn as_str(&self) -> &'static str {
        let s = self.0;
        // SAFETY: `TokenType` can only be constructed via `TokenType::new`,
        // which ensures `s.len() <= TOKEN_TYPE_MAX_LEN`
        unsafe { assert_unchecked!(s.len() <= TOKEN_TYPE_MAX_LEN) };
        s
    }
}
