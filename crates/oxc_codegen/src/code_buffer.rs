use assert_unchecked::assert_unchecked;

/// A string builder for constructing source code.
///
/// `CodeBuffer` provides safe abstractions over a byte array.
/// Essentially same as `String` but with additional methods.
///
/// Use one of the various `print_*` methods to add text into the buffer.
/// When you are done, call [`into_string`] to extract the final [`String`].
///
/// # Example
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
/// let source = code.into_string();
/// ```
///
/// [`into_string`]: CodeBuffer::into_string
#[derive(Debug, Default, Clone)]
pub struct CodeBuffer {
    /// INVARIANT: `buf` is a valid UTF-8 string.
    buf: Vec<u8>,
}

impl CodeBuffer {
    /// Create a new empty `CodeBuffer`.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// // use `code` to build new source text
    /// code.print_str("fn main() { println!(\"Hello, world!\"); }");
    /// let source_text = code.into_string();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new, empty `CodeBuffer` with the specified capacity.
    ///
    /// The buffer will be able to hold at least `capacity` bytes without reallocating.
    /// This method is allowed to allocate for more bytes than `capacity`.
    /// If `capacity` is 0, the buffer will not allocate.
    ///
    /// It is important to note that although the returned buffer has the
    /// minimum *capacity* specified, the buffer will have a zero *length*.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Vec::with_capacity(capacity) }
    }

    /// Returns the number of bytes in the buffer.
    ///
    /// This is *not* the same as the number of characters in the buffer,
    /// since non-ASCII characters require multiple bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if the buffer contains no characters.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// assert!(code.is_empty());
    ///
    /// code.print_char('c');
    /// assert!(!code.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Reserves capacity for at least `additional` more bytes in the buffer.
    ///
    /// The buffer may reserve more space to speculatively avoid frequent reallocations.
    /// After calling `reserve`, capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::default();
    /// code.reserve(10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    /// Peek the `n`th character from the end of the buffer.
    ///
    /// When `n` is zero, the last character is returned.
    /// Returns [`None`] if `n` exceeds the length of the buffer.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("foo");
    ///
    /// assert_eq!(code.peek_nth_char_back(0), Some('o'));
    /// assert_eq!(code.peek_nth_char_back(2), Some('f'));
    /// assert_eq!(code.peek_nth_char_back(3), None);
    /// ```
    #[inline]
    #[must_use = "Peeking is pointless if the peeked char isn't used"]
    pub fn peek_nth_char_back(&self, n: usize) -> Option<char> {
        // SAFETY: All methods of `CodeBuffer` ensure `buf` is valid UTF-8
        unsafe { std::str::from_utf8_unchecked(&self.buf) }.chars().nth_back(n)
    }

    /// Peek the `n`th byte from the end of the buffer.
    ///
    /// When `n` is zero, the last byte is returned.
    /// Returns [`None`] if `n` exceeds the length of the buffer.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("foo");
    ///
    /// assert_eq!(code.peek_nth_byte_back(0), Some(b'o'));
    /// assert_eq!(code.peek_nth_byte_back(2), Some(b'f'));
    /// assert_eq!(code.peek_nth_byte_back(3), None);
    /// ```
    #[inline]
    #[must_use = "Peeking is pointless if the peeked char isn't used"]
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        let len = self.len();
        if n < len {
            Some(self.buf[len - 1 - n])
        } else {
            None
        }
    }

    /// Peek the last byte from the end of the buffer.
    #[inline]
    pub fn last_byte(&self) -> Option<u8> {
        self.buf.last().copied()
    }

    /// Peek the last char from the end of the buffer.
    #[inline]
    pub fn last_char(&self) -> Option<char> {
        self.peek_nth_char_back(0)
    }

    /// Push a single ASCII byte into the buffer.
    ///
    /// # Panics
    /// Panics if `byte` is not an ASCII byte (`0 - 0x7F`).
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_ascii_byte(b'f');
    /// code.print_ascii_byte(b'o');
    /// code.print_ascii_byte(b'o');
    ///
    /// let source = code.into_string();
    /// assert_eq!(source, "foo");
    /// ```
    #[inline]
    pub fn print_ascii_byte(&mut self, byte: u8) {
        // When this method is inlined, and the value of `byte` is known, this assertion should
        // get optimized away by the compiler. e.g. `code_buffer.print_ascii_byte(b' ')`.
        assert!(byte.is_ascii(), "byte {byte} is not ASCII");

        // SAFETY: `byte` is an ASCII character
        unsafe { self.print_byte_unchecked(byte) }
    }

    /// Push a byte to the buffer, without checking that the buffer still represents a valid
    /// UTF-8 string.
    ///
    /// If you are looking to print a byte you know is valid ASCII, prefer [`print_ascii_byte`].
    /// If you are not certain, you may use [`print_char`] as a safe alternative.
    ///
    /// # SAFETY
    /// The caller must ensure that, after 1 or more sequential calls, the buffer represents
    /// a valid UTF-8 string.
    ///
    /// It is safe for a single call to temporarily result in invalid UTF-8, as long as
    /// UTF-8 integrity is restored before calls to any other `print_*` method or
    /// [`into_string`]. This lets you, for example, print an 4-byte Unicode character
    /// using 4 separate calls to this method. However, consider using [`print_bytes_unchecked`]
    /// instead for that use case.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// // Safe: 'a' is a valid ASCII character. Its UTF-8 representation only
    /// // requires a single byte.
    /// unsafe { code.print_byte_unchecked(b'a') };
    ///
    /// let not_ascii = '⚓';
    /// let bytes = not_ascii.to_string().into_bytes();
    /// // Safe: after this loop completes, `code` returns to a valid state.
    /// for byte in bytes {
    ///     unsafe { code.print_byte_unchecked(byte) };
    /// }
    ///
    /// // NOT SAFE: `ch` exceeds the ASCII segment range. `code` is no longer valid UTF-8
    /// // unsafe { code.print_byte_unchecked(0xFF) };
    /// ```
    ///
    /// [`print_ascii_byte`]: CodeBuffer::print_ascii_byte
    /// [`print_char`]: CodeBuffer::print_char
    /// [`into_string`]: CodeBuffer::into_string
    /// [`print_bytes_unchecked`]: CodeBuffer::print_bytes_unchecked
    #[inline]
    pub unsafe fn print_byte_unchecked(&mut self, byte: u8) {
        // By default, `self.buf.push(byte)` results in quite verbose assembly, because the default
        // branch is for the "buf is full to capacity" case.
        //
        // That's not ideal because growth strategy is doubling, so e.g. when the `Vec` has just grown
        // from 1024 bytes to 2048 bytes, it won't need to grow again until another 1024 bytes have
        // been pushed. "Needs to grow" is a very rare occurrence.
        //
        // So we use `push_slow` to move the complicated logic for the "needs to grow" path out of
        // `print_byte_unchecked`, leaving a fast path for the common "there is sufficient capacity" case.
        // https://godbolt.org/z/Kv8sEoEed
        // https://github.com/oxc-project/oxc/pull/6148#issuecomment-2381635390
        #[cold]
        #[inline(never)]
        fn push_slow(code_buffer: &mut CodeBuffer, byte: u8) {
            let buf = &mut code_buffer.buf;
            // SAFETY: We only call this function below if `buf.len() == buf.capacity()`.
            // This function is not inlined, so we need this assertion to assist compiler to
            // understand this fact.
            unsafe { assert_unchecked!(buf.len() == buf.capacity()) }
            buf.push(byte);
        }

        #[expect(clippy::if_not_else)]
        if self.buf.len() != self.buf.capacity() {
            self.buf.push(byte);
        } else {
            push_slow(self, byte);
        }
    }

    /// Push a single Unicode character into the buffer.
    ///
    /// When pushing multiple characters, consider choosing [`print_str`] over this method
    /// since it's much more efficient. If you really want to insert only a single character
    /// and you're certain it's ASCII, consider using [`print_ascii_byte`].
    ///
    /// # Example
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

    /// Push a string into the buffer.
    ///
    /// # Example
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
    /// Panics if any byte in the iterator is not ASCII.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// code.print_ascii_bytes([b'f', b'o', b'o']);
    /// assert_eq!(String::from(code), "foo");
    /// ```
    pub fn print_ascii_bytes<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        let iter = bytes.into_iter();
        let hint = iter.size_hint();
        self.buf.reserve(hint.1.unwrap_or(hint.0));
        for byte in iter {
            self.print_ascii_byte(byte);
        }
    }

    /// Print a sequence of bytes without checking that the buffer still
    /// represents a valid UTF-8 string.
    ///
    /// # Safety
    ///
    /// The caller must ensure that, after this method call, the buffer represents
    /// a valid UTF-8 string. In practice, this means only two cases are valid:
    ///
    /// 1. Both the buffer and the byte sequence are valid UTF-8,
    /// 2. The buffer became invalid after a call to [`print_byte_unchecked`] and `bytes` completes
    ///    any incomplete Unicode characters, returning the buffer to a valid state.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// // Indent to a dynamic level.
    /// // Sound because all elements in this iterator are ASCII characters.
    /// unsafe {
    ///     code.print_bytes_unchecked(std::iter::repeat(b' ').take(4));
    /// }
    /// ```
    ///
    /// [`print_byte_unchecked`]: CodeBuffer::print_byte_unchecked
    #[inline]
    pub unsafe fn print_bytes_unchecked<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        self.buf.extend(bytes);
    }

    /// Get contents of buffer as a byte slice.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("foo");
    /// assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Consume buffer and return source code as a `String`.
    ///
    /// # Example
    /// ```
    /// use oxc_codegen::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("console.log('foo');");
    ///
    /// let source = code.into_string();
    /// assert_eq!(source, "console.log('foo');");
    /// ```
    #[must_use]
    #[inline]
    pub fn into_string(self) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(self.buf).unwrap()
        } else {
            // SAFETY: All methods of `CodeBuffer` ensure `buf` is valid UTF-8
            unsafe { String::from_utf8_unchecked(self.buf) }
        }
    }
}

impl AsRef<[u8]> for CodeBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<CodeBuffer> for String {
    #[inline]
    fn from(code: CodeBuffer) -> Self {
        code.into_string()
    }
}

#[cfg(test)]
mod test {
    use super::CodeBuffer;

    #[test]
    fn empty() {
        let code = CodeBuffer::default();
        assert!(code.is_empty());
        assert_eq!(code.len(), 0);
        assert_eq!(String::from(code), "");
    }

    #[test]
    fn string_isomorphism() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);
        assert_eq!(code.len(), s.len());
        assert_eq!(String::from(code), s.to_string());
    }

    #[test]
    fn into_string() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);

        let source = code.into_string();
        assert_eq!(source, s);
    }

    #[test]
    #[allow(clippy::byte_char_slices)]
    fn print_ascii_byte() {
        let mut code = CodeBuffer::new();
        code.print_ascii_byte(b'f');
        code.print_ascii_byte(b'o');
        code.print_ascii_byte(b'o');

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[allow(clippy::byte_char_slices)]
    fn print_byte_unchecked() {
        let mut code = CodeBuffer::new();
        // SAFETY: These bytes are all ASCII
        unsafe {
            code.print_byte_unchecked(b'f');
            code.print_byte_unchecked(b'o');
            code.print_byte_unchecked(b'o');
        }

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[allow(clippy::byte_char_slices)]
    fn print_ascii_bytes() {
        let mut code = CodeBuffer::new();
        code.print_ascii_bytes([b'f', b'o', b'o']);

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    fn peek_nth_char_back() {
        let mut code = CodeBuffer::new();
        code.print_str("bar");

        assert_eq!(code.peek_nth_char_back(0), Some('r'));
        assert_eq!(code.peek_nth_char_back(1), Some('a'));
        assert_eq!(code.peek_nth_char_back(2), Some('b'));
        assert_eq!(code.peek_nth_char_back(3), None);
    }

    #[test]
    fn peek_nth_byte_back() {
        let mut code = CodeBuffer::new();
        code.print_str("bar");

        assert_eq!(code.peek_nth_byte_back(0), Some(b'r'));
        assert_eq!(code.peek_nth_byte_back(1), Some(b'a'));
        assert_eq!(code.peek_nth_byte_back(2), Some(b'b'));
        assert_eq!(code.peek_nth_byte_back(3), None);
    }

    #[test]
    fn last_byte() {
        let mut code = CodeBuffer::new();
        assert_eq!(code.last_byte(), None);
        code.print_str("bar");
        assert_eq!(code.last_byte(), Some(b'r'));
    }

    #[test]
    fn last_char() {
        let mut code = CodeBuffer::new();
        assert_eq!(code.last_char(), None);
        code.print_str("bar");
        assert_eq!(code.last_char(), Some('r'));
    }
}
