/// A JavaScript string code point, including lone surrogates.
///
/// The value is always in `0..=0x10_FFFF`. Unlike [`char`], this includes the
/// surrogate range `0xD800..=0xDFFF`. A supplementary character is one `JSChar`,
/// even though it occupies two UTF-16 code units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct JSChar(u32);

impl JSChar {
    /// Construct a code point, returning `None` if `value > 0x10_FFFF`.
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value <= 0x10_FFFF { Some(Self(value)) } else { None }
    }

    /// Return the code point's numeric value.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Return a Unicode scalar value, or `None` for a lone surrogate.
    #[inline]
    pub const fn to_char(self) -> Option<char> {
        if self.is_surrogate() {
            None
        } else {
            // SAFETY: `JSChar` is at most 0x10_FFFF, and we excluded surrogates.
            Some(unsafe { char::from_u32_unchecked(self.0) })
        }
    }

    /// Construct a code point without checking its range.
    ///
    /// # Safety
    /// `value` must be at most `0x10_FFFF`.
    #[inline]
    pub(super) const unsafe fn from_u32_unchecked(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    pub(super) const fn from_code_unit(unit: u16) -> Self {
        Self(unit as u32)
    }

    #[inline]
    pub(super) const fn is_surrogate(self) -> bool {
        self.0 >= 0xD800 && self.0 <= 0xDFFF
    }

    /// Encode one code point. The caller handles pairing adjacent surrogates.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "each byte is masked or range-checked")]
    pub(super) fn encode(self, buffer: &mut [u8; 4]) -> &[u8] {
        let value = self.0;
        let len = match value {
            0..=0x7F => {
                buffer[0] = value as u8;
                1
            }
            0x80..=0x7FF => {
                buffer[0] = 0xC0 | (value >> 6) as u8;
                buffer[1] = 0x80 | (value & 0x3F) as u8;
                2
            }
            0x800..=0xFFFF => {
                buffer[0] = 0xE0 | (value >> 12) as u8;
                buffer[1] = 0x80 | ((value >> 6) & 0x3F) as u8;
                buffer[2] = 0x80 | (value & 0x3F) as u8;
                3
            }
            _ => {
                buffer[0] = 0xF0 | (value >> 18) as u8;
                buffer[1] = 0x80 | ((value >> 12) & 0x3F) as u8;
                buffer[2] = 0x80 | ((value >> 6) & 0x3F) as u8;
                buffer[3] = 0x80 | (value & 0x3F) as u8;
                4
            }
        };
        &buffer[..len]
    }
}
