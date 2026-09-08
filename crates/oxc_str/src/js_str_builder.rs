use oxc_allocator::{Allocator, ArenaVec};

use crate::{JSChar, JSStr};

/// Build a JavaScript string in an arena, preserving lone surrogates.
///
/// Appends concatenate UTF-16 values. A leading surrogate at the end of one
/// append pairs with a trailing surrogate at the start of the next, including
/// across empty appends. [`into_js_str`](Self::into_js_str) returns canonical WTF-8
/// without copying the completed buffer.
///
/// ```
/// use oxc_allocator::Allocator;
/// use oxc_str::JSStrBuilder;
///
/// let allocator = Allocator::new();
/// let mut builder = JSStrBuilder::new_in(&allocator);
/// builder.push_code_unit(0xD800);
/// builder.push_str("");
/// builder.push_code_unit(0xDC00);
/// assert_eq!(builder.into_js_str().as_str(), Some("𐀀"));
/// ```
pub struct JSStrBuilder<'a> {
    /// Canonical WTF-8, excluding a held final leading surrogate.
    bytes: ArenaVec<'a, u8>,
    /// Always in 0xD800..=0xDBFF when present, with three bytes of spare capacity
    /// in `bytes`, so finishing the string never needs to grow the buffer.
    pending_lead_surrogate: Option<u16>,
    /// Describes only `bytes`, excluding `pending_lead_surrogate`.
    has_lone_surrogate: bool,
}

impl<'a> JSStrBuilder<'a> {
    /// Construct an empty builder without allocating.
    #[inline]
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            bytes: ArenaVec::new_in(&allocator),
            pending_lead_surrogate: None,
            has_lone_surrogate: false,
        }
    }

    /// Reserve capacity in bytes. Zero capacity does not allocate.
    ///
    /// # Panics
    /// Panics if capacity exceeds `u32::MAX` or `isize::MAX`.
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'a Allocator) -> Self {
        Self {
            bytes: ArenaVec::with_capacity_in(capacity, &allocator),
            pending_lead_surrogate: None,
            has_lone_surrogate: false,
        }
    }

    /// Append UTF-8 text. Empty text preserves a pending leading surrogate.
    #[inline]
    pub fn push_str(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        // Reserve before changing state, so a capacity panic cannot leave a
        // flushed leading surrogate that a later append would fail to pair.
        if self.pending_lead_surrogate.is_some() {
            self.bytes.reserve(value.len() + 3);
            self.flush_pending();
        }
        self.bytes.extend_from_slice_copy(value.as_bytes());
    }

    /// Append a Unicode scalar value.
    #[inline]
    pub fn push(&mut self, value: char) {
        self.push_str(value.encode_utf8(&mut [0; 4]));
    }

    /// Append one JavaScript code point, pairing surrogates at the boundary.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "leading surrogates fit in u16")]
    pub fn push_js_char(&mut self, value: JSChar) {
        let point = value.to_u32();
        if let Some(lead) = self.pending_lead_surrogate
            && (0xDC00..=0xDFFF).contains(&point)
        {
            self.bytes.reserve(4);
            self.append_pair(lead, point as u16);
            self.pending_lead_surrogate = None;
            return;
        }

        let mut buffer = [0; 4];
        let bytes = value.encode(&mut buffer);
        let is_lead = (0xD800..=0xDBFF).contains(&point);
        let additional = self.pending_bytes() + bytes.len();
        self.bytes.reserve(additional);
        self.flush_pending();
        if is_lead {
            self.pending_lead_surrogate = Some(point as u16);
        } else {
            self.bytes.extend_from_slice_copy(bytes);
            self.has_lone_surrogate |= value.is_surrogate();
        }
    }

    /// Append one UTF-16 code unit.
    #[inline]
    pub fn push_code_unit(&mut self, unit: u16) {
        self.push_js_char(JSChar::from_code_unit(unit));
    }

    /// Append potentially ill-formed UTF-16.
    #[inline]
    pub fn push_utf16(&mut self, units: &[u16]) {
        for &unit in units {
            self.push_code_unit(unit);
        }
    }

    /// Append a JavaScript string, repairing a surrogate pair at the boundary.
    ///
    /// UTF-8 inputs are copied without scanning. For inputs with lone surrogates,
    /// only the boundary encodings change; the interior is copied as bytes.
    #[inline]
    pub fn push_js_str(&mut self, value: JSStr<'_>) {
        if let Some(value) = value.as_str() {
            self.push_str(value);
            return;
        }
        self.push_js_str_slow(value);
    }

    fn push_js_str_slow(&mut self, value: JSStr<'_>) {
        if value.is_empty() {
            return;
        }
        let mut bytes = value.as_bytes();
        let additional = bytes.len() + self.pending_bytes();
        self.bytes.reserve(additional);
        let mut trimmed = false;

        if let Some(lead) = self.pending_lead_surrogate
            && let [0xED, second @ 0xB0..=0xBF, third, ..] = bytes
        {
            let trail = 0xD000 | (u16::from(*second & 0x3F) << 6) | u16::from(*third & 0x3F);
            self.append_pair(lead, trail);
            self.pending_lead_surrogate = None;
            bytes = &bytes[3..];
            trimmed = true;
        } else {
            self.flush_pending();
        }

        if let [.., 0xED, second @ 0xA0..=0xAF, third] = bytes {
            self.pending_lead_surrogate =
                Some(0xD000 | (u16::from(*second & 0x3F) << 6) | u16::from(*third & 0x3F));
            bytes = &bytes[..bytes.len() - 3];
            trimmed = true;
        }

        // The input's flag remains valid unless we removed an edge surrogate.
        // A removed edge may have been its only lone surrogate, so inspect the
        // interior in that case. An already-set output flag needs no further scan.
        self.has_lone_surrogate = self.has_lone_surrogate
            || if trimmed {
                bytes.windows(3).any(|bytes| bytes[0] == 0xED && bytes[1] >= 0xA0)
            } else {
                value.has_lone_surrogate()
            };
        self.bytes.extend_from_slice_copy(bytes);
    }

    /// Consume the builder and return a string without copying or rescanning the buffer.
    #[inline]
    pub fn into_js_str(mut self) -> JSStr<'a> {
        // A pending surrogate always has three bytes of capacity reserved for it.
        self.flush_pending();
        let bytes = self.bytes.into_arena_slice();
        // SAFETY: Appends maintain canonical WTF-8 and the exact surrogate flag.
        // The final pending surrogate has been flushed. `ArenaVec<u8>` bounds
        // its length by u32::MAX and isize::MAX; the slice owns the arena lifetime.
        unsafe { JSStr::from_bytes_unchecked(bytes, self.has_lone_surrogate) }
    }

    #[inline]
    fn pending_bytes(&self) -> usize {
        if self.pending_lead_surrogate.is_some() { 3 } else { 0 }
    }

    /// Call only after reserving space and deciding the next append cannot pair.
    #[inline]
    fn flush_pending(&mut self) {
        if let Some(lead) = self.pending_lead_surrogate.take() {
            self.bytes.extend_from_slice_copy(JSChar::from_code_unit(lead).encode(&mut [0; 4]));
            self.has_lone_surrogate = true;
        }
    }

    /// `lead` and `trail` have already been checked at the append boundary.
    #[inline]
    fn append_pair(&mut self, lead: u16, trail: u16) {
        let value = 0x10000 + ((u32::from(lead) - 0xD800) << 10) + (u32::from(trail) - 0xDC00);
        let c = char::from_u32(value).unwrap();
        self.bytes.extend_from_slice_copy(c.encode_utf8(&mut [0; 4]).as_bytes());
    }
}

impl<'a> From<JSStrBuilder<'a>> for JSStr<'a> {
    #[inline]
    fn from(builder: JSStrBuilder<'a>) -> Self {
        builder.into_js_str()
    }
}
