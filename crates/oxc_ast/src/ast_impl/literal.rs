//! Literals

use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use oxc_allocator::{Allocator, CloneIn, Dummy};
use oxc_data_structures::inline_string::InlineString;
use oxc_span::ContentEq;

use crate::ast::*;

impl BooleanLiteral {
    /// `"true"` or `"false"` depending on this boolean's value.
    pub fn as_str(&self) -> &'static str {
        if self.value { "true" } else { "false" }
    }
}

impl Display for BooleanLiteral {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Display for NullLiteral {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "null".fmt(f)
    }
}

impl NumericLiteral<'_> {
    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/base/JSCompDoubles.java#L113)
    ///
    /// <https://262.ecma-international.org/5.1/#sec-9.5>
    #[expect(clippy::cast_possible_truncation)] // for `as i32`
    pub fn ecmascript_to_int32(num: f64) -> i32 {
        // Fast path for most common case. Also covers -0.0
        let int32_value = num as i32;
        if (f64::from(int32_value) - num).abs() < f64::EPSILON {
            return int32_value;
        }

        // NaN, Infinity if not included in our NumericLiteral, so we just skip step 2.

        // step 3
        let pos_int = num.signum() * num.abs().floor();

        // step 4
        let int32bit = pos_int % 2f64.powi(32);

        // step5
        if int32bit >= 2f64.powi(31) { (int32bit - 2f64.powi(32)) as i32 } else { int32bit as i32 }
    }

    /// Return raw source code for `NumericLiteral`.
    /// If `raw` is `None` (node is generated, not parsed from source), fallback to formatting `value`.
    pub fn raw_str(&self) -> Cow<'_, str> {
        match self.raw.as_ref() {
            Some(raw) => Cow::Borrowed(raw),
            None => Cow::Owned(format!("{}", self.value)),
        }
    }
}

impl Display for NumericLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We have 2 choices here:
        // 1. Only use the `value` field. or
        // 2. Use `raw` field if it's `Some`, otherwise fallback to using `value` field.
        // For now, we take the 2nd approach, since `NumericLiteral::to_string` is only used in linter,
        // where raw does matter.
        self.raw_str().fmt(f)
    }
}

impl StringLiteral<'_> {
    /// Static Semantics: `IsStringWellFormedUnicode`
    /// test for \uD800-\uDFFF
    ///
    /// See: <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-isstringwellformedunicode>
    pub fn is_string_well_formed_unicode(&self) -> bool {
        let mut chars = self.value.chars();
        while let Some(c) = chars.next() {
            if c == '\\' && chars.next() == Some('u') {
                let hex = &chars.as_str()[..4];
                if let Ok(hex) = u32::from_str_radix(hex, 16) {
                    if (0xd800..=0xdfff).contains(&hex) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl AsRef<str> for StringLiteral<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl Display for StringLiteral<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl BigIntLiteral<'_> {
    /// Is this BigInt literal zero? (`0n`).
    pub fn is_zero(&self) -> bool {
        self.value == "0"
    }

    /// Is this BigInt literal negative? (e.g. `-1n`).
    pub fn is_negative(&self) -> bool {
        self.value.starts_with('-')
    }
}

impl Display for BigIntLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}n", self.value)
    }
}

impl Display for RegExp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern.text, self.flags)
    }
}

impl ContentEq for RegExpFlags {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'alloc> CloneIn<'alloc> for RegExpFlags {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc Allocator) -> Self::Cloned {
        *self
    }
}

impl<'a> Dummy<'a> for RegExpFlags {
    /// Create a dummy [`RegExpFlags`].
    ///
    /// Does not allocate any data into arena.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_: &'a Allocator) -> Self {
        RegExpFlags::empty()
    }
}

impl TryFrom<char> for RegExpFlags {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'g' => Ok(Self::G),
            'i' => Ok(Self::I),
            'm' => Ok(Self::M),
            's' => Ok(Self::S),
            'u' => Ok(Self::U),
            'y' => Ok(Self::Y),
            'd' => Ok(Self::D),
            'v' => Ok(Self::V),
            _ => Err(value),
        }
    }
}

impl TryFrom<u8> for RegExpFlags {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'g' => Ok(Self::G),
            b'i' => Ok(Self::I),
            b'm' => Ok(Self::M),
            b's' => Ok(Self::S),
            b'u' => Ok(Self::U),
            b'y' => Ok(Self::Y),
            b'd' => Ok(Self::D),
            b'v' => Ok(Self::V),
            _ => Err(value),
        }
    }
}

impl Display for RegExpFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_inline_string().as_str().fmt(f)
    }
}

impl RegExpFlags {
    /// Convert [`RegExpFlags`] to an [`InlineString`].
    ///
    /// This performs the same role as `RegExpFlags::to_string`, but does not allocate.
    pub fn to_inline_string(self) -> InlineString<8, usize> {
        let mut str = InlineString::new();

        // In alphabetical order.
        // SAFETY: Capacity of the `InlineString` is 8, and we push a maximum of 8 bytes.
        // All bytes pushed are ASCII.
        unsafe {
            if self.contains(Self::D) {
                str.push_unchecked(b'd');
            }
            if self.contains(Self::G) {
                str.push_unchecked(b'g');
            }
            if self.contains(Self::I) {
                str.push_unchecked(b'i');
            }
            if self.contains(Self::M) {
                str.push_unchecked(b'm');
            }
            if self.contains(Self::S) {
                str.push_unchecked(b's');
            }
            if self.contains(Self::U) {
                str.push_unchecked(b'u');
            }
            if self.contains(Self::V) {
                str.push_unchecked(b'v');
            }
            if self.contains(Self::Y) {
                str.push_unchecked(b'y');
            }
        }

        str
    }
}
