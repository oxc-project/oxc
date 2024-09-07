//! Literals

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
};

use oxc_regular_expression::ast::Pattern;
use oxc_span::{hash::ContentHash, Atom, Span};
use oxc_syntax::number::NumberBase;

use crate::ast::*;

impl BooleanLiteral {
    pub fn new(span: Span, value: bool) -> Self {
        Self { span, value }
    }

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

impl ContentHash for NullLiteral {
    #[inline]
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&Option::<bool>::None, state);
    }
}

impl NullLiteral {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

impl fmt::Display for NullLiteral {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "null".fmt(f)
    }
}

impl<'a> NumericLiteral<'a> {
    pub fn new(span: Span, value: f64, raw: &'a str, base: NumberBase) -> Self {
        Self { span, value, raw, base }
    }

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
}

impl<'a> ContentHash for NumericLiteral<'a> {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&self.base, state);
        ContentHash::content_hash(&self.raw, state);
    }
}

impl<'a> fmt::Display for NumericLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<'a> BigIntLiteral<'a> {
    pub fn is_zero(&self) -> bool {
        self.raw == "0n"
    }
}

impl<'a> fmt::Display for BigIntLiteral<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.raw.fmt(f)
    }
}

impl<'a> fmt::Display for RegExp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flags)
    }
}

impl<'a> RegExpPattern<'a> {
    pub fn len(&self) -> usize {
        match self {
            Self::Raw(it) | Self::Invalid(it) => it.len(),
            Self::Pattern(it) => it.span.size() as usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

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

    pub fn as_pattern(&self) -> Option<&Pattern<'a>> {
        if let Self::Pattern(it) = self {
            Some(it.as_ref())
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for RegExpPattern<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(it) | Self::Invalid(it) => write!(f, "{it}"),
            Self::Pattern(it) => it.fmt(f),
        }
    }
}

impl<'a> StringLiteral<'a> {
    pub fn new(span: Span, value: Atom<'a>) -> Self {
        Self { span, value }
    }

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

impl<'a> AsRef<str> for StringLiteral<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl<'a> fmt::Display for StringLiteral<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
