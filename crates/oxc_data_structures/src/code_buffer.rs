//! A string builder for constructing source code.

use std::iter;

use crate::assert_unchecked;

/// Character to use for indentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum IndentChar {
    /// Use tab character for indentation.
    #[default]
    Tab = b'\t',
    /// Use space character for indentation.
    Space = b' ',
}

/// Default indentation width.
pub const DEFAULT_INDENT_WIDTH: usize = 1;

/// A string builder for constructing source code.
///
/// [`CodeBuffer`] provides safe abstractions over a byte array.
/// Essentially same as `String` but with additional methods.
///
/// Use one of the various `print_*` methods to add text into the buffer.
/// When you are done, call [`into_string`] to extract the final [`String`].
///
/// # Example
/// ```
/// # use oxc_data_structures::code_buffer::CodeBuffer;
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
#[derive(Debug, Clone)]
pub struct CodeBuffer {
    /// INVARIANT: `buf` is a valid UTF-8 string.
    buf: Vec<u8>,
    /// Character to use for indentation.
    indent_char: IndentChar,
    /// Number of indent characters per indentation level.
    indent_width: usize,
    /// Small scratch buffer to accumulate bytes before flushing to main buffer.
    /// This reduces the number of reallocations and improves performance.
    scratch: [u8; 64],
    /// Current length of valid data in scratch buffer.
    scratch_len: usize,
}

impl Default for CodeBuffer {
    #[inline]
    fn default() -> Self {
        Self {
            buf: Vec::new(),
            indent_char: IndentChar::default(),
            indent_width: DEFAULT_INDENT_WIDTH,
            scratch: [0; 64],
            scratch_len: 0,
        }
    }
}

impl CodeBuffer {
    /// Create a new [`CodeBuffer`].
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
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

    /// Create a new [`CodeBuffer`] with specified indentation.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::{CodeBuffer, IndentChar};
    /// let mut code = CodeBuffer::with_indent(IndentChar::Space, 4);
    ///
    /// // This will use 4 spaces per indentation level
    /// code.print_indent(2); // prints 8 spaces
    /// ```
    #[inline]
    pub fn with_indent(indent_char: IndentChar, indent_width: usize) -> Self {
        Self { buf: Vec::new(), indent_char, indent_width, scratch: [0; 64], scratch_len: 0 }
    }

    /// Create a new [`CodeBuffer`] with the specified capacity.
    ///
    /// The buffer will be able to hold at least `capacity` bytes without reallocating.
    /// This method is allowed to allocate for more bytes than `capacity`.
    /// If `capacity` is 0, the buffer will not allocate.
    ///
    /// It is important to note that although the returned buffer has the
    /// minimum *capacity* specified, the buffer will have a zero *length*.
    ///
    /// # Panics
    /// Panics if `capacity` exceeds `isize::MAX` bytes.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            indent_char: IndentChar::default(),
            indent_width: DEFAULT_INDENT_WIDTH,
            scratch: [0; 64],
            scratch_len: 0,
        }
    }

    /// Create a new [`CodeBuffer`] with the specified capacity and indentation.
    ///
    /// The buffer will be able to hold at least `capacity` bytes without reallocating.
    /// This method is allowed to allocate for more bytes than `capacity`.
    /// If `capacity` is 0, the buffer will not allocate.
    ///
    /// It is important to note that although the returned buffer has the
    /// minimum *capacity* specified, the buffer will have a zero *length*.
    ///
    /// # Panics
    /// Panics if `capacity` exceeds `isize::MAX` bytes.
    #[inline]
    pub fn with_capacity_and_indent(
        capacity: usize,
        indent_char: IndentChar,
        indent_width: usize,
    ) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            indent_char,
            indent_width,
            scratch: [0; 64],
            scratch_len: 0,
        }
    }

    /// Flush the scratch buffer to the main buffer.
    #[inline]
    fn flush_scratch(&mut self) {
        if self.scratch_len > 0 {
            self.buf.extend_from_slice(&self.scratch[..self.scratch_len]);
            self.scratch_len = 0;
        }
    }

    /// Returns the number of bytes in the buffer.
    ///
    /// This is *not* the same as the number of characters in the buffer,
    /// since non-ASCII characters require multiple bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len() + self.scratch_len
    }

    /// Returns the capacity of the buffer in bytes.
    ///
    /// This is *not* the same as capacity in characters,
    /// since non-ASCII characters require multiple bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Returns `true` if the buffer contains no characters.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// assert!(code.is_empty());
    ///
    /// code.print_char('c');
    /// assert!(!code.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty() && self.scratch_len == 0
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
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::default();
    /// code.reserve(10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.flush_scratch();
        self.buf.reserve(additional);
    }

    /// Peek the `n`th character from the end of the buffer.
    ///
    /// When `n` is zero, the last character is returned.
    /// Returns [`None`] if `n` exceeds the length of the buffer.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("foo");
    ///
    /// assert_eq!(code.peek_nth_char_back(0), Some('o'));
    /// assert_eq!(code.peek_nth_char_back(2), Some('f'));
    /// assert_eq!(code.peek_nth_char_back(3), None);
    /// ```
    #[inline]
    #[must_use = "Peeking is pointless if the peeked char isn't used"]
    #[expect(clippy::missing_panics_doc)] // No panic possible in release mode
    pub fn peek_nth_char_back(&self, n: usize) -> Option<char> {
        // We need to combine buf and scratch to peek correctly
        // Create a temporary combined view without modifying self
        if self.scratch_len == 0 {
            // No scratch buffer content, just use main buffer
            let s = if cfg!(debug_assertions) {
                std::str::from_utf8(&self.buf).unwrap()
            } else {
                // SAFETY: All safe methods of `CodeBuffer` ensure `buf` is valid UTF-8
                unsafe { std::str::from_utf8_unchecked(&self.buf) }
            };
            s.chars().nth_back(n)
        } else {
            // Need to combine buf and scratch for accurate peeking
            // This is less efficient but maintains const-correctness
            let mut combined = Vec::with_capacity(self.buf.len() + self.scratch_len);
            combined.extend_from_slice(&self.buf);
            combined.extend_from_slice(&self.scratch[..self.scratch_len]);

            let s = if cfg!(debug_assertions) {
                std::str::from_utf8(&combined).unwrap()
            } else {
                // SAFETY: All safe methods of `CodeBuffer` ensure both parts are valid UTF-8
                unsafe { std::str::from_utf8_unchecked(&combined) }
            };
            s.chars().nth_back(n)
        }
    }

    /// Peek the `n`th byte from the end of the buffer.
    ///
    /// When `n` is zero, the last byte is returned.
    /// Returns [`None`] if `n` exceeds the length of the buffer.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
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
        let total_len = self.len();
        if n >= total_len {
            return None;
        }

        // Check if we need to look in scratch buffer
        if n < self.scratch_len {
            Some(self.scratch[self.scratch_len - 1 - n])
        } else {
            // Look in main buffer
            let buf_idx = n - self.scratch_len;
            Some(self.buf[self.buf.len() - 1 - buf_idx])
        }
    }

    /// Peek the last byte from the end of the buffer.
    #[inline]
    pub fn last_byte(&self) -> Option<u8> {
        if self.scratch_len > 0 {
            Some(self.scratch[self.scratch_len - 1])
        } else {
            self.buf.last().copied()
        }
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
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
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
    /// If you are looking to print a byte that is statically provable to be valid ASCII,
    /// prefer safe method [`print_ascii_byte`]. The assertion that byte is ASCII in that function
    /// will be elided be elided by compiler if it's trivially provable
    /// e.g. `buffer.print_ascii_byte(b'x')`. So the safe method costs nothing extra in that case.
    ///
    /// If you are not certain, you may use [`print_char`] as a safe alternative.
    ///
    /// # SAFETY
    /// The caller must ensure that, after 1 or more sequential calls, the buffer represents
    /// a valid UTF-8 string.
    ///
    /// It is OK for a single call to temporarily result in the buffer containsing invalid UTF-8, as long
    /// as UTF-8 integrity is restored before calls to any other `print_*` method or [`into_string`].
    /// This lets you, for example, print an 4-byte Unicode character using 4 separate calls to this method.
    /// However, consider using [`print_bytes_unchecked`] instead for that use case.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
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
        // Try to add to scratch buffer first
        if self.scratch_len < self.scratch.len() {
            self.scratch[self.scratch_len] = byte;
            self.scratch_len += 1;
        } else {
            // Scratch buffer is full, flush it and write directly to main buffer
            self.flush_scratch();

            // Use the optimized push logic for direct writes to main buffer
            #[cold]
            #[inline(never)]
            fn push_slow(buf: &mut Vec<u8>, byte: u8) {
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
                push_slow(&mut self.buf, byte);
            }
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
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
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
        let bytes = ch.encode_utf8(&mut b).as_bytes();

        // Try to fit in scratch buffer if it's small enough
        if bytes.len() + self.scratch_len <= self.scratch.len() {
            self.scratch[self.scratch_len..self.scratch_len + bytes.len()].copy_from_slice(bytes);
            self.scratch_len += bytes.len();
        } else {
            // Doesn't fit in scratch, flush and write directly
            self.flush_scratch();
            self.buf.extend_from_slice(bytes);
        }
    }

    /// Push a string into the buffer.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("function main() { console.log('Hello, world!') }");
    /// ```
    #[inline]
    pub fn print_str<S: AsRef<str>>(&mut self, s: S) {
        let bytes = s.as_ref().as_bytes();

        // Try to fit in scratch buffer if it's small enough
        if bytes.len() + self.scratch_len <= self.scratch.len() {
            self.scratch[self.scratch_len..self.scratch_len + bytes.len()].copy_from_slice(bytes);
            self.scratch_len += bytes.len();
        } else {
            // Too large for scratch, flush scratch and write directly
            self.flush_scratch();
            self.buf.extend_from_slice(bytes);
        }
    }

    /// Push a sequence of ASCII characters into the buffer.
    ///
    /// # Panics
    /// Panics if any byte in the iterator is not ASCII.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
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
        let estimated_size = hint.1.unwrap_or(hint.0);

        // Reserve capacity if needed
        if estimated_size > self.scratch.len() - self.scratch_len {
            self.flush_scratch();
            self.buf.reserve(estimated_size);
        }

        for byte in iter {
            self.print_ascii_byte(byte);
        }
    }

    /// Print a slice of bytes, without checking that the buffer still contains a valid UTF-8 string.
    ///
    /// # SAFETY
    ///
    /// The caller must ensure that, after 1 or more sequential calls, the buffer represents
    /// a valid UTF-8 string.
    ///
    /// It is OK for a single call to temporarily result in the buffer containing invalid UTF-8, as long
    /// as UTF-8 integrity is restored before calls to any other `print_*` method or [`into_string`].
    ///
    /// This requirement is easily satisfied if buffer contained valid UTF-8, and the `bytes` slice
    /// also contains a valid UTF-8 string.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// // SAFETY: All bytes in this slice are ASCII
    /// unsafe {
    ///     code.print_bytes_unchecked("abcd".as_bytes());
    /// }
    /// ```
    ///
    /// [`into_string`]: CodeBuffer::into_string
    #[inline]
    pub unsafe fn print_bytes_unchecked(&mut self, bytes: &[u8]) {
        // Try to fit in scratch buffer if it's small enough
        if bytes.len() + self.scratch_len <= self.scratch.len() {
            self.scratch[self.scratch_len..self.scratch_len + bytes.len()].copy_from_slice(bytes);
            self.scratch_len += bytes.len();
        } else {
            // Too large for scratch, flush scratch and write directly
            self.flush_scratch();
            self.buf.extend_from_slice(bytes);
        }
    }

    /// Print a sequence of bytes, without checking that the buffer still contains a valid UTF-8 string.
    ///
    /// # SAFETY
    ///
    /// The caller must ensure that, after 1 or more sequential calls, the buffer represents
    /// a valid UTF-8 string.
    ///
    /// It is OK for a single call to temporarily result in the buffer containing invalid UTF-8, as long
    /// as UTF-8 integrity is restored before calls to any other `print_*` method or [`into_string`].
    ///
    /// This requirement is easily satisfied if buffer contained valid UTF-8, and the `bytes` iterator
    /// also yields a valid UTF-8 string.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    ///
    /// // SAFETY: All values yielded by this iterator are ASCII bytes
    /// unsafe {
    ///     code.print_bytes_iter_unchecked(std::iter::repeat_n(b' ', 20));
    /// }
    /// ```
    ///
    /// [`into_string`]: CodeBuffer::into_string
    #[inline]
    pub unsafe fn print_bytes_iter_unchecked<I: IntoIterator<Item = u8>>(&mut self, bytes: I) {
        // For iterators, we need to flush scratch first since we don't know the size
        self.flush_scratch();
        self.buf.extend(bytes);
    }

    /// Print `depth` levels of indentation into the buffer.
    ///
    /// Uses the configured indentation character and width.
    /// Prints `depth * indent_width` indent characters.
    ///
    /// Optimized on assumption that more than 32 characters of indentation is rare.
    ///
    /// Fast path is to write 32 bytes of tabs/spaces in a single load + store,
    /// but only advance `len` by the actual number of bytes. This avoids a `memset` function call.
    ///
    /// Take alternative slow path if either:
    ///   1. Total characters to print > 32.
    ///   2. Less than 32 bytes spare capacity in buffer (needs to grow).
    ///
    /// Both of these cases should be rare.
    ///
    /// We write 32 bytes because both tabs and spaces are supported.
    /// When using spaces for indentation, it'll usually be 2 or 4 spaces per indent level,
    /// and with 4 it only takes indent level of 5 to exceed 16 bytes. That'd be fairly common.
    /// So we take the cost of 1 more SIMD XMM write to avoid hitting the cold path in such cases.
    ///
    /// <https://godbolt.org/z/P9x87q7nd>
    #[inline]
    pub fn print_indent(&mut self, depth: usize) {
        /// Size of chunks to write indent in.
        /// 16 is largest register size (XMM) available on all x86_64 targets,
        /// so writing 32 bytes takes 2 x XMM writes.
        const CHUNK_SIZE: usize = 32;

        let bytes = depth * self.indent_width;

        // Try to fit in scratch buffer first
        if bytes + self.scratch_len <= self.scratch.len() {
            // Fill scratch with indent characters
            for i in 0..bytes {
                self.scratch[self.scratch_len + i] = self.indent_char as u8;
            }
            self.scratch_len += bytes;
            return;
        }

        // Doesn't fit in scratch, flush and write directly
        self.flush_scratch();

        #[cold]
        #[inline(never)]
        fn write_slow(code_buffer: &mut CodeBuffer, bytes: usize) {
            code_buffer.buf.extend(iter::repeat_n(code_buffer.indent_char as u8, bytes));
        }

        let len = self.buf.len();
        let spare_capacity = self.capacity() - len;
        if bytes > CHUNK_SIZE || spare_capacity < CHUNK_SIZE {
            write_slow(self, bytes);
            return;
        }

        // Write 32 bytes of the indent character into buffer.
        // On x86_64, this is 5 SIMD instructions (32 byte copy).
        // SAFETY: We checked there are at least 32 bytes spare capacity.
        unsafe {
            let ptr = self.buf.as_mut_ptr().add(len).cast::<[u8; CHUNK_SIZE]>();
            ptr.write([self.indent_char as u8; CHUNK_SIZE]);
        }

        // Update length of buffer.
        // SAFETY: We checked there's at least 32 bytes spare capacity, and `bytes <= 32`,
        // so `len + bytes` cannot exceed capacity.
        // `len` cannot exceed `isize::MAX`, so `len + bytes` cannot wrap around.
        unsafe { self.buf.set_len(len + bytes) };
    }

    /// Get contents of buffer as a byte slice.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("foo");
    /// assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
    /// ```
    #[inline]
    pub fn as_bytes(&mut self) -> &[u8] {
        self.flush_scratch();
        &self.buf
    }

    /// Consume buffer and return source code as a `String`.
    ///
    /// # Example
    /// ```
    /// # use oxc_data_structures::code_buffer::CodeBuffer;
    /// let mut code = CodeBuffer::new();
    /// code.print_str("console.log('foo');");
    ///
    /// let source = code.into_string();
    /// assert_eq!(source, "console.log('foo');");
    /// ```
    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    #[inline]
    pub fn into_string(mut self) -> String {
        self.flush_scratch();
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
        // Can't flush scratch buffer with immutable reference, so we assert it's empty.
        // This is safe because as_ref is typically called after all writing is done.
        debug_assert_eq!(self.scratch_len, 0, "as_ref called with non-empty scratch buffer");
        &self.buf
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
    use super::{CodeBuffer, IndentChar};

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
    #[expect(clippy::byte_char_slices)]
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
    #[expect(clippy::byte_char_slices)]
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
    #[expect(clippy::byte_char_slices)]
    fn print_bytes_unchecked() {
        let mut code = CodeBuffer::new();
        // SAFETY: These bytes are all ASCII
        unsafe { code.print_bytes_unchecked(b"foo") };

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
    fn print_bytes_iter_unchecked() {
        let mut code = CodeBuffer::new();
        // SAFETY: These bytes are all ASCII
        unsafe { code.print_bytes_iter_unchecked([b'f', b'o', b'o']) };

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
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

    #[test]
    fn test_cached_indent_tabs() {
        let mut code = CodeBuffer::with_indent(IndentChar::Tab, 1);
        code.print_indent(2);
        assert_eq!(code.into_string(), "\t\t");
    }

    #[test]
    fn test_cached_indent_spaces_width_2() {
        let mut code = CodeBuffer::with_indent(IndentChar::Space, 2);
        code.print_indent(2);
        assert_eq!(code.into_string(), "    "); // 2 levels * 2 spaces = 4 spaces
    }

    #[test]
    fn test_cached_indent_spaces_width_4() {
        let mut code = CodeBuffer::with_indent(IndentChar::Space, 4);
        code.print_indent(2);
        assert_eq!(code.into_string(), "        "); // 2 levels * 4 spaces = 8 spaces
    }
}
