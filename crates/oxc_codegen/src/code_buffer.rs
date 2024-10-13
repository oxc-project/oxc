/// A string builder for constructing source code.
///
///
/// `CodeBuffer` provides safe abstractions over a byte array, allowing for
/// a compact byte-array representation without soundness holes.
///
/// Use one of the various `print_*` methods to add text into a buffer. When you
/// are done, call [`take_source_text`] to extract the final [`String`].
///
/// # Examples
/// ```
/// use oxc_codegen::CodeBuffer;
/// let mut code = CodeBuffer::new();
///
/// // mock settings
/// let is_public = true;
///
/// if is_public {
///     code.print_str("export ")
/// }
/// code.print_str("function foo() {\n");
/// code.print_str("    console.log('Hello, world!');\n");
/// code.print_str("}\n");
///
/// let source = code.take_source_text();
/// ```
///
/// [`take_source_text`]: CodeBuffer::take_source_text
#[derive(Debug, Default, Clone)]
pub struct CodeBuffer {
    /// INVARIANT: `buf` is a valid UTF-8 string.
    buf: Vec<u8>,
}

impl CodeBuffer {
    /// Create a new empty `CodeBuffer`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// // use `code` to build new source text
    /// code.print_str("fn main() { println!(\"Hello, world!\"); }");
    /// let source_text = code.take_source_text();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new, empty `CodeBuffer` with the specified capacity.
    ///
    /// The buffer will be able to hold at least `capacity` bytes without
    /// reallocating. This method is allowed to allocate for more bytes than
    /// `capacity`. If `capacity` is 0, the buffer will not allocate.
    ///
    /// It is important to note that although the returned buffer has the
    /// minimum *capacity* specified, the buffer will have a zero *length*.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Vec::with_capacity(capacity) }
    }

    /// Returns the number of bytes in this buffer.
    ///
    /// This is _not_ the same as the number of characters in the buffer, since
    /// non-ASCII characters require multiple bytes.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if this buffer contains no characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// assert!(code.is_empty());
    ///
    /// code.push_char('c');
    /// assert!(!code.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Reserves capacity for at least `additional` more characters in the given
    /// `CodeBuffer`. The buffer may reserve more space to speculatively avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` _bytes_.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut code = CodeBuffer::default();
    /// code.reserve(10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    /// Peek the `n`th character from the end of the buffer.
    /// When `n` is zero, the last character is returned. Returns [`None`] if
    /// `n` exceeds the length of the buffer.
    ///
    /// ## Examples
    /// ```
    /// # use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("foo");
    ///
    /// assert_eq!(code.peek_nth_back(0), Some('o'));
    /// assert_eq!(code.peek_nth_back(2), Some('f'));
    /// assert_eq!(code.peek_nth_back(3), None);
    /// ```
    #[inline]
    #[must_use = "Peeking is pointless if the peeked char isn't used"]
    pub fn peek_nth_back(&self, n: usize) -> Option<char> {
        // SAFETY: `buf` is a valid UTF-8 string because of invariants upheld by CodeBuffer
        unsafe { std::str::from_utf8_unchecked(&self.buf) }.chars().nth_back(n)
    }

    /// Push a single ASCII character into the buffer
    ///
    /// # Panics
    /// If `ch` is not a valid UTF-8 code point in the ASCII range (`0 - 0x7F`).
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_ascii_byte('f');
    /// code.print_ascii_byte('o');
    /// code.print_ascii_byte('o');
    ///
    /// let source = code.take_source_text();
    /// assert_eq!(source, "foo");
    /// ```
    #[inline]
    pub fn print_ascii_byte(&mut self, b: u8) {
        // NOTE: since this method is inlined, this assertion should get
        // optimized away by the compiler when the value of `b` is known,
        // e.g. when printing a constant.
        assert!(b.is_ascii(), "byte {b} is not ASCII");
        self.buf.push(b);
    }

    /// Print a byte without checking that this buffer still represents a valid
    /// UTF-8 string.
    ///
    /// If you are looking to print a byte you know is valid ASCII, prefer
    /// [`print_ascii_byte`]. If you are not certain, you may use [`print_char`]
    /// as a safe alternative.
    ///
    /// # Safety
    /// The caller must ensure that, after 1 or more sequential calls, this
    /// buffer represents a valid UTF-8 string.
    ///
    /// It is safe for a single call to temporarily result in invalid UTF-8, as
    /// long as UTF-8 integrity is restored before calls to any other `print`
    /// method or [`take_source_text`]. This lets you, for example, print an
    /// 8-byte code point using 4 separate calls to this method.
    ///
    /// If you find yourself in such a scenario, consider using
    /// [`print_unchecked`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// // Safe: 'a' is a valid ASCII character. Its UTF-8 representation only
    /// // requires a single byte.
    /// unsafe { code.print_byte_unsafe(b'a') };
    ///
    /// let not_ascii = '⚓';
    /// let as_bytes = not_ascii.to_string().into_bytes();
    /// // Safe: after this loop completes, `code` returns to a valid state.
    /// for byte in as_bytes {
    ///     unsafe { code.print_byte_unsafe(byte) };
    /// }
    ///
    /// // NOT SAFE: `ch` exceeds the ASCII segment range. `code` is no longer
    /// valid UTF-8
    /// unsafe { code.print_byte_unsafe(0xFF) };
    /// ```
    ///
    /// [`print_ascii_byte`]: CodeBuffer::print_ascii_byte
    /// [`print_char`]: CodeBuffer::print_char
    /// [`take_source_text`]: CodeBuffer::take_source_text
    /// [`print_unchecked`]: CodeBuffer::print_unchecked
    #[inline]
    pub unsafe fn print_byte_unsafe(&mut self, ch: u8) {
        self.buf.push(ch);
    }

    /// Print a single Unicode character into the buffer.
    ///
    /// When pushing multiple characters, consider choosing [`print_str`] over
    /// this method since it's much more efficient. If you really want to insert
    /// only a single character and you're certain it's ASCII, consider using
    /// [`print_ascii_byte`].
    ///
    /// ## Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// code.print_char('f');
    /// code.print_char('o');
    /// code.print_char('o');
    ///
    /// assert_eq!(String::from(code), "foo");
    /// ```
    ///
    /// [`print_str`]: CodeBuffer::print_str
    /// [`print_ascii_byte`]: CodeBuffer::print_ascii_byte
    #[inline]
    pub fn print_char(&mut self, ch: char) {
        let mut b = [0; 4];
        self.buf.extend(ch.encode_utf8(&mut b).as_bytes());
    }

    /// Push a string into this the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("function main() { console.log('Hello, world!') }");
    /// ```
    #[inline]
    pub fn print_str<S: AsRef<str>>(&mut self, s: S) {
        self.buf.extend(s.as_ref().as_bytes());
    }

    /// Push a sequence of ASCII characters into the buffer.
    ///
    /// # Panics
    /// If any byte in the iterator is not valid ASCII.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// code.print_ascii([b'f', b'o', b'o'].into_iter());
    /// assert_eq!(String::from(code), "foo");
    /// ```
    pub fn print_ascii<I>(&mut self, chars: I)
    where
        I: IntoIterator<Item = u8>,
    {
        let iter = chars.into_iter();
        let hint = iter.size_hint();
        self.buf.reserve(hint.1.unwrap_or(hint.0));
        for c in iter {
            self.print_ascii_byte(c);
        }
    }

    /// Print a sequence of bytes without checking that this buffer still
    /// represents a valid UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that, after being called, this buffer represents
    /// a valid UTF-8 string. In practice, this means only two cases are valid:
    ///
    /// 1. Both the buffer and the byte sequence are valid UTF-8,
    /// 2. The buffer became invalid after a call to [`print_byte_unsafe`] and `bytes`
    ///    completes any incomplete code points, returning the buffer to a valid
    ///    state.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// // Indent to a dynamic level. Sound because all elements in this
    /// // iterator are valid 1-byte UTF-8 code points (ASCII).
    /// unsafe {
    ///     code.print_unchecked(std::iter::repeat(b' ').take(4));
    /// }
    /// ```
    ///
    /// [`print_byte_unsafe`]: CodeBuffer::print_byte_unsafe
    #[inline]
    pub(crate) unsafe fn print_unchecked<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        self.buf.extend(bytes);
    }

    /// Convert a `CodeBuffer` into a string of source code, leaving its
    /// internal buffer empty and finalizing the codegen process.
    ///
    /// It is safe to re-use a buffer after calling this method. Its contents
    /// will be emptied out, but all memory resources are retained and in a
    /// valid state. You may use [`String::from`] if you don't intend on
    /// re-using the buffer. It simply calls this method and drops the
    /// `CodeBuffer` afterwards.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("console.log('foo');");
    ///
    /// let source = code.take_source_text();
    /// assert_eq!(source, "console.log('foo');");
    /// assert!(code.is_empty());
    /// ```
    #[must_use]
    pub fn take_source_text(&mut self) -> String {
        use std::mem::take;

        #[cfg(debug_assertions)]
        {
            String::from_utf8(take(&mut self.buf)).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            // SAFETY: `buf` is valid UTF-8 because of invariants upheld by
            // CodeBuffer. If, for some reason, it is not, this is caused by
            // improper use of `unsafe` printing methods.
            unsafe { String::from_utf8_unchecked(take(&mut self.buf)) }
        }
    }
}

impl AsRef<[u8]> for CodeBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.buf
    }
}
impl From<CodeBuffer> for String {
    #[inline]
    fn from(printer: CodeBuffer) -> Self {
        if cfg!(debug_assertions) {
            String::from_utf8(printer.buf).unwrap()
        } else {
            // SAFETY: `buf` is valid UTF-8 because of invariants upheld by `CodeBuffer`
            unsafe { String::from_utf8_unchecked(printer.buf) }
        }
    }
}

#[cfg(test)]
mod test {
    use super::CodeBuffer;

    #[test]
    fn test_empty() {
        let code = CodeBuffer::default();
        assert!(code.is_empty());
        assert_eq!(code.len(), 0);
        assert_eq!(String::from(code), "");
    }

    #[test]
    fn test_string_isomorphism() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);
        assert_eq!(code.len(), s.len());
        assert_eq!(String::from(code), s.to_string());
    }

    #[test]
    fn test_into_source_string() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);

        let source = code.take_source_text();
        assert_eq!(source, s);

        // buffer has been emptied
        assert!(code.is_empty());
        assert_eq!(code.len(), 0);
        let empty_slice: &[u8] = &[];
        assert_eq!(code.as_ref(), empty_slice);
        assert_eq!(String::from(code), "");
    }

    #[test]
    #[allow(clippy::byte_char_slices)]
    fn test_print_byte_unsafe() {
        let mut code = CodeBuffer::new();
        code.print_ascii_byte(b'f');
        code.print_ascii_byte(b'o');
        code.print_ascii_byte(b'o');

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_ref(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    fn test_peek() {
        let mut code = CodeBuffer::new();
        code.print_str("foo");

        assert_eq!(code.peek_nth_back(0), Some('o'));
        assert_eq!(code.peek_nth_back(2), Some('f'));
        assert_eq!(code.peek_nth_back(3), None);
    }
}
