use crate::wtf8::*;

/// A JavaScript string code point, including lone surrogates.
///
/// The value is always in `0..=0x10_FFFF`. Unlike [`char`], this includes the
/// surrogate range `0xD800..=0xDFFF`. A supplementary character is one `JSChar`,
/// even though it occupies two UTF-16 code units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct JSChar(u32);

impl From<char> for JSChar {
    /// Every Unicode scalar value is a JavaScript code point.
    #[inline]
    fn from(value: char) -> Self {
        Self(u32::from(value))
    }
}

impl PartialEq<char> for JSChar {
    /// Code points are equal by value.
    /// A `char` cannot be a lone surrogate, so a lone-surrogate code point never equals any `char`.
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.0 == u32::from(*other)
    }
}

impl JSChar {
    /// Construct a code point, returning `None` if `value > 0x10_FFFF`.
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value <= CODE_POINT_MAX { Some(Self(value)) } else { None }
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
            // SAFETY: `JSChar` is at most 0x10_FFFF, and we excluded surrogates
            Some(unsafe { char::from_u32_unchecked(self.0) })
        }
    }

    /// Construct a code point without checking its range.
    ///
    /// # SAFETY
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
        self.0 >= LEAD_SURROGATE_MIN && self.0 <= TRAIL_SURROGATE_MAX
    }

    #[inline]
    pub(super) const fn is_lead_surrogate(self) -> bool {
        self.0 >= LEAD_SURROGATE_MIN && self.0 <= LEAD_SURROGATE_MAX
    }

    #[inline]
    pub(super) const fn is_trail_surrogate(self) -> bool {
        self.0 >= TRAIL_SURROGATE_MIN && self.0 <= TRAIL_SURROGATE_MAX
    }

    /// Byte length of this code point as stored in a [`JSStr`].
    ///
    /// [`JSStr`]: crate::JSStr
    #[inline]
    pub const fn len_bytes(self) -> usize {
        match self.0 {
            0..=ONE_BYTE_CODE_POINT_MAX => 1,
            TWO_BYTE_CODE_POINT_MIN..=TWO_BYTE_CODE_POINT_MAX => 2,
            THREE_BYTE_CODE_POINT_MIN..=THREE_BYTE_CODE_POINT_MAX => 3,
            _ => 4,
        }
    }

    /// Encode one code point. The caller handles pairing adjacent surrogates.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "each byte is masked or range-checked")]
    pub(super) fn encode(self, buffer: &mut [u8; char::MAX_LEN_UTF8]) -> &[u8] {
        let value = self.0;
        let len = match value {
            0..=ONE_BYTE_CODE_POINT_MAX => {
                buffer[0] = value as u8;
                1
            }
            TWO_BYTE_CODE_POINT_MIN..=TWO_BYTE_CODE_POINT_MAX => {
                buffer[0] = TWO_BYTE_TAG | (value >> 6) as u8;
                buffer[1] = CONT_TAG | (value as u8 & CONT_MASK);
                2
            }
            THREE_BYTE_CODE_POINT_MIN..=THREE_BYTE_CODE_POINT_MAX => {
                buffer[0] = THREE_BYTE_TAG | (value >> 12) as u8;
                buffer[1] = CONT_TAG | ((value >> 6) as u8 & CONT_MASK);
                buffer[2] = CONT_TAG | (value as u8 & CONT_MASK);
                3
            }
            _ => {
                buffer[0] = FOUR_BYTE_TAG | (value >> 18) as u8;
                buffer[1] = CONT_TAG | ((value >> 12) as u8 & CONT_MASK);
                buffer[2] = CONT_TAG | ((value >> 6) as u8 & CONT_MASK);
                buffer[3] = CONT_TAG | (value as u8 & CONT_MASK);
                4
            }
        };
        &buffer[..len]
    }
}

#[cfg(test)]
mod tests {
    use super::JSChar;

    #[test]
    fn len_bytes_matches_storage_width() {
        assert_eq!(JSChar::from('a').len_bytes(), 1);
        assert_eq!(JSChar::from('\u{00E9}').len_bytes(), 2);
        assert_eq!(JSChar::from_u32(0xD800).unwrap().len_bytes(), 3);
        assert_eq!(JSChar::from('\u{1F600}').len_bytes(), 4);
    }

    #[test]
    fn compares_to_char_by_code_point() {
        assert_eq!(JSChar::from('\r'), '\r');
        assert_ne!(JSChar::from('\r'), '\n');
        let lead = JSChar::from_u32(0xD800).unwrap();
        assert_ne!(lead, '\u{FFFD}');
    }
}
