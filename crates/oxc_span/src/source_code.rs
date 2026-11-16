use std::fmt;

use oxc_allocator::Allocator;

/// Source text which is followed in memory by [`SourceCode::PADDING`] NUL bytes.
///
/// The padding lets a lexer read at, and a short distance past, the end of the source text
/// without checking bounds first: a read at the end always lands on a NUL byte, so the end
/// of the source only has to be tested for when a NUL byte is found.
///
/// The padding is not part of the source text. [`SourceCode::as_str`] returns the text alone.
///
/// # Creating a `SourceCode`
///
/// * [`SourceCode::from_str_in`] copies source text into an arena, and adds the padding.
///   This works for any `&str`.
/// * [`SourceCode::from_padded_str`] borrows a string which already ends with the padding,
///   without copying. Use this when you own the buffer which the source text is stored in.
///
/// # Example
///
/// ```
/// use oxc_allocator::Allocator;
/// use oxc_span::SourceCode;
///
/// // Copy into an arena
/// let allocator = Allocator::default();
/// let source = SourceCode::from_str_in("let x = 1;", &allocator);
/// assert_eq!(source.as_str(), "let x = 1;");
///
/// // Borrow an owned buffer which has padding added to it
/// let mut buffer = String::from("let x = 1;");
/// SourceCode::pad_string(&mut buffer);
/// let source = SourceCode::from_padded_str(&buffer).unwrap();
/// assert_eq!(source.as_str(), "let x = 1;");
/// ```
#[derive(Clone, Copy)]
pub struct SourceCode<'a> {
    /// Source text followed by `PADDING` NUL bytes.
    ///
    /// NUL is a valid single-byte UTF-8 character, so text and padding together are a valid `&str`.
    /// Holding them as a single `&str` means reading the padding is in bounds of this reference.
    padded: &'a str,
}

/// Padding which is appended to source text.
const PADDING_STR: &str = match str::from_utf8(&[0; SourceCode::PADDING]) {
    Ok(padding) => padding,
    Err(_) => unreachable!(),
};

impl<'a> SourceCode<'a> {
    /// Number of NUL bytes which follow the source text.
    pub const PADDING: usize = 64;

    /// Create a [`SourceCode`] by copying `source_text` into `allocator`, followed by padding.
    ///
    /// This performs 1 allocation of `source_text.len() + SourceCode::PADDING` bytes.
    ///
    /// # Panics
    ///
    /// Panics if `source_text.len() + SourceCode::PADDING` exceeds `isize::MAX`.
    pub fn from_str_in(source_text: &str, allocator: &'a Allocator) -> Self {
        let padded = allocator.alloc_concat_strs_array([source_text, PADDING_STR]);
        Self { padded }
    }

    /// Create a [`SourceCode`] from a string which already ends with padding, without copying.
    ///
    /// `padded` must be the source text followed by [`SourceCode::PADDING`] NUL bytes.
    /// The source text is everything before those bytes.
    /// [`SourceCode::pad_string`] adds the padding to a `String`.
    ///
    /// Returns `None` if `padded` does not end with [`SourceCode::PADDING`] NUL bytes.
    pub fn from_padded_str(padded: &'a str) -> Option<Self> {
        let source_len = padded.len().checked_sub(Self::PADDING)?;
        let padding = &padded.as_bytes()[source_len..];
        if padding.iter().all(|&byte| byte == 0) { Some(Self { padded }) } else { None }
    }

    /// Append [`SourceCode::PADDING`] NUL bytes to `source_text`,
    /// so it can be passed to [`SourceCode::from_padded_str`].
    ///
    /// This does not reallocate if `source_text` has at least [`SourceCode::PADDING`] bytes
    /// of spare capacity.
    pub fn pad_string(source_text: &mut String) {
        source_text.push_str(PADDING_STR);
    }

    /// Get the source text, without padding.
    #[inline]
    pub fn as_str(self) -> &'a str {
        let source_len = self.len();
        // The byte at `source_len` is the first NUL byte of the padding, which is a UTF-8
        // character boundary, so this slice operation cannot panic
        &self.padded[..source_len]
    }

    /// Get the source text followed by its [`SourceCode::PADDING`] NUL bytes.
    #[inline]
    pub fn as_padded_str(self) -> &'a str {
        self.padded
    }

    /// Get length of the source text in bytes, excluding padding.
    #[inline]
    pub fn len(self) -> usize {
        self.padded.len() - Self::PADDING
    }

    /// Returns `true` if the source text is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for SourceCode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SourceCode").field(&self.as_str()).finish()
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use super::SourceCode;

    #[test]
    fn from_str_in() {
        let allocator = Allocator::default();
        for text in ["", "x", "let x = 1;", "日本語", "a\0b"] {
            let source = SourceCode::from_str_in(text, &allocator);
            assert_eq!(source.as_str(), text);
            assert_eq!(source.len(), text.len());
            assert_eq!(source.is_empty(), text.is_empty());

            let padded = source.as_padded_str();
            assert_eq!(padded.len(), text.len() + SourceCode::PADDING);
            assert!(padded.as_bytes()[text.len()..].iter().all(|&byte| byte == 0));
            assert_eq!(padded.as_ptr(), source.as_str().as_ptr());
        }
    }

    #[test]
    fn from_padded_str() {
        let mut buffer = String::from("let x = 1;");
        SourceCode::pad_string(&mut buffer);
        let source = SourceCode::from_padded_str(&buffer).unwrap();
        assert_eq!(source.as_str(), "let x = 1;");
        // Zero-copy
        assert_eq!(source.as_str().as_ptr(), buffer.as_ptr());
        assert_eq!(source.as_padded_str().len(), buffer.len());
    }

    #[test]
    fn from_padded_str_trailing_nul_in_source() {
        // NUL bytes before the padding are part of the source text
        let mut buffer = String::from("x\0\0");
        SourceCode::pad_string(&mut buffer);
        let source = SourceCode::from_padded_str(&buffer).unwrap();
        assert_eq!(source.as_str(), "x\0\0");
    }

    #[test]
    fn from_padded_str_rejects_missing_padding() {
        assert!(SourceCode::from_padded_str("").is_none());
        assert!(SourceCode::from_padded_str("let x = 1;").is_none());

        // 1 byte too few
        let too_short = "\0".repeat(SourceCode::PADDING - 1);
        assert!(SourceCode::from_padded_str(&too_short).is_none());

        // Non-NUL byte within the padding
        let mut buffer = String::from("x");
        SourceCode::pad_string(&mut buffer);
        buffer.push('y');
        assert!(SourceCode::from_padded_str(&buffer).is_none());
    }

    #[test]
    fn pad_string_does_not_reallocate_with_spare_capacity() {
        let mut buffer = String::with_capacity(3 + SourceCode::PADDING);
        buffer.push_str("abc");
        let ptr = buffer.as_ptr();
        SourceCode::pad_string(&mut buffer);
        assert_eq!(buffer.as_ptr(), ptr);
        assert_eq!(buffer.len(), 3 + SourceCode::PADDING);
    }
}
