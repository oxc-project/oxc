//! Literals

use std::{borrow::Cow, fmt};

use oxc_allocator::CloneIn;
use oxc_regular_expression::ast::Pattern;
use oxc_span::cmp::ContentEq;

use crate::ast::*;

impl BooleanLiteral {
    /// `"true"` or `"false"` depending on this boolean's value.
    pub fn as_str(&self) -> &'static str {
        if self.value {
            "true"
        } else {
            "false"
        }
    }
}

impl fmt::Display for BooleanLiteral {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for NullLiteral {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "null".fmt(f)
    }
}

impl NumericLiteral<'_> {
    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/base/JSCompDoubles.java#L113)
    ///
    /// <https://262.ecma-international.org/5.1/#sec-9.5>
    #[allow(clippy::cast_possible_truncation)] // for `as i32`
    pub fn ecmascript_to_int32(num: f64) -> i32 {
        // Fast path for most common case. Also covers -0.0
        let int32_value = num as i32;
        if (f64::from(int32_value) - num).abs() < f64::EPSILON {
            return int32_value;
        }

        // NaN, Infinity if not included in our NumericLiteral, so we just serde(skip) step 2.

        // step 3
        let pos_int = num.signum() * num.abs().floor();

        // step 4
        let int32bit = pos_int % 2f64.powi(32);

        // step5
        if int32bit >= 2f64.powi(31) {
            (int32bit - 2f64.powi(32)) as i32
        } else {
            int32bit as i32
        }
    }

    /// Return raw source code for `NumericLiteral`.
    /// If `raw` is `None` (node is generated, not parsed from source), fallback to formatting `value`.
    pub fn raw_str(&self) -> Cow<str> {
        match self.raw.as_ref() {
            Some(raw) => Cow::Borrowed(raw),
            None => Cow::Owned(format!("{}", self.value)),
        }
    }
}

impl ContentEq for NumericLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        // Note: `f64::content_eq` uses `==` equality.
        // `f64::NAN != f64::NAN` and `0.0 == -0.0`, so we follow the same here.
        ContentEq::content_eq(&self.value, &other.value)
    }
}

impl fmt::Display for NumericLiteral<'_> {
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
                };
            }
        }
        true
    }
}

impl ContentEq for StringLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.value, &other.value)
    }
}

impl AsRef<str> for StringLiteral<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl fmt::Display for StringLiteral<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl BigIntLiteral<'_> {
    /// Is this BigInt literal zero? (`0n`).
    pub fn is_zero(&self) -> bool {
        self.raw == "0n"
    }
}

impl ContentEq for BigIntLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.raw, &other.raw)
    }
}

impl fmt::Display for BigIntLiteral<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl ContentEq for RegExpLiteral<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        ContentEq::content_eq(&self.regex, &other.regex)
    }
}

impl fmt::Display for RegExp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flags)
    }
}

impl<'a> RegExpPattern<'a> {
    /// Returns the number of characters in the pattern.
    pub fn len(&self) -> usize {
        match self {
            Self::Raw(it) | Self::Invalid(it) => it.len(),
            Self::Pattern(it) => it.span.size() as usize,
        }
    }

    /// Returns `true` if the pattern is empty (i.e. has a
    /// [len](RegExpPattern::len) of `0`).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the string as this regular expression would appear in source code.
    pub fn source_text(&self, source_text: &'a str) -> Cow<str> {
        match self {
            Self::Raw(raw) | Self::Invalid(raw) => Cow::Borrowed(raw),
            Self::Pattern(pat) if pat.span.is_unspanned() => Cow::Owned(pat.to_string()),
            Self::Pattern(pat) => Cow::Borrowed(pat.span.source_text(source_text)),
        }
    }

    /// # Panics
    /// If `self` is anything but `RegExpPattern::Pattern`.
    pub fn require_pattern(&self) -> &Pattern<'a> {
        if let Some(it) = self.as_pattern() {
            it
        } else {
            unreachable!(
                "Required `{}` to be `{}`",
                stringify!(RegExpPattern),
                stringify!(Pattern)
            );
        }
    }

    /// Flatten this regular expression into a compiled [`Pattern`], returning
    /// [`None`] if the pattern is invalid or not parsed.
    pub fn as_pattern(&self) -> Option<&Pattern<'a>> {
        if let Self::Pattern(it) = self {
            Some(it.as_ref())
        } else {
            None
        }
    }
}

impl ContentEq for RegExpPattern<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        let self_str = match self {
            Self::Raw(s) | Self::Invalid(s) => *s,
            Self::Pattern(p) => &p.to_string(),
        };

        let other_str = match other {
            Self::Raw(s) | Self::Invalid(s) => *s,
            Self::Pattern(p) => &p.to_string(),
        };

        self_str == other_str
    }
}

impl fmt::Display for RegExpPattern<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(it) | Self::Invalid(it) => write!(f, "{it}"),
            Self::Pattern(it) => it.fmt(f),
        }
    }
}

impl ContentEq for RegExpFlags {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'alloc> CloneIn<'alloc> for RegExpFlags {
    type Cloned = Self;

    fn clone_in(&self, _: &'alloc oxc_allocator::Allocator) -> Self::Cloned {
        *self
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

impl fmt::Display for RegExpFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.contains(Self::G) {
            write!(f, "g")?;
        }
        if self.contains(Self::I) {
            write!(f, "i")?;
        }
        if self.contains(Self::M) {
            write!(f, "m")?;
        }
        if self.contains(Self::S) {
            write!(f, "s")?;
        }
        if self.contains(Self::U) {
            write!(f, "u")?;
        }
        if self.contains(Self::Y) {
            write!(f, "y")?;
        }
        if self.contains(Self::D) {
            write!(f, "d")?;
        }
        if self.contains(Self::V) {
            write!(f, "v")?;
        }
        Ok(())
    }
}
