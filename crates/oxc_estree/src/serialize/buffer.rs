/// Buffer to store serialized JSON.
pub struct Buffer {
    bytes: Vec<u8>,
}

impl Buffer {
    /// Create new [`Buffer`].
    pub(super) fn new() -> Self {
        Self { bytes: vec![] }
    }

    /// Push a single ASCII byte to the buffer.
    ///
    /// # Panics
    /// Panics if `byte` is not ASCII.
    #[inline(always)]
    pub(super) fn push_ascii_byte(&mut self, byte: u8) {
        assert!(byte.is_ascii());
        // SAFETY: We just checked this byte is ASCII
        unsafe { self.push_ascii_byte_unchecked(byte) };
    }

    /// Push a single ASCII byte to the buffer, without checking that it's ASCII.
    ///
    /// # SAFETY
    /// `byte` must be ASCII.
    #[inline(always)]
    pub(super) unsafe fn push_ascii_byte_unchecked(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    /// Push a raw string to the buffer.
    pub(super) fn push_str(&mut self, s: &str) {
        self.bytes.extend_from_slice(s.as_bytes());
    }

    /// Push a slice of bytes to the buffer.
    ///
    /// # SAFETY
    /// `bytes` must comprise a valid UTF-8 string.
    pub(super) unsafe fn push_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    /// Consume [`Buffer`] and convert buffer to string.
    pub(super) fn into_string(self) -> String {
        // SAFETY: None of `Serializer`'s safe methods allow
        // adding invalid byte sequences to `self.bytes`
        unsafe { String::from_utf8_unchecked(self.bytes) }
    }
}
