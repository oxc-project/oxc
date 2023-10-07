//! Literals

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use bitflags::bitflags;
use num_bigint::BigInt;
use oxc_span::{Atom, Span};
use oxc_syntax::NumberBase;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BooleanLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: bool,
}

impl BooleanLiteral {
    pub fn as_str(&self) -> &'static str {
        if self.value {
            "true"
        } else {
            "false"
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct NullLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
}

impl Hash for NullLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        None::<bool>.hash(state);
    }
}

impl NullLiteral {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct NumberLiteral<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: f64,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub raw: &'a str,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub base: NumberBase,
}

impl<'a> NumberLiteral<'a> {
    pub fn new(span: Span, value: f64, raw: &'a str, base: NumberBase) -> Self {
        Self { span, value, raw, base }
    }

    /// port from [closure compiler](https://github.com/google/closure-compiler/blob/a4c880032fba961f7a6c06ef99daa3641810bfdd/src/com/google/javascript/jscomp/base/JSCompDoubles.java#L113)
    /// <https://262.ecma-international.org/5.1/#sec-9.5>
    #[allow(clippy::cast_possible_truncation)] // for `as i32`
    pub fn ecmascript_to_int32(num: f64) -> i32 {
        // Fast path for most common case. Also covers -0.0
        let int32_value = num as i32;
        if (f64::from(int32_value) - num).abs() < f64::EPSILON {
            return int32_value;
        }

        // NaN, Infinity if not included in our NumberLiteral, so we just skip step 2.

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

impl<'a> Hash for NumberLiteral<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.raw.hash(state);
    }
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct BigintLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::serialize::serialize_bigint"))]
    pub value: BigInt,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct RegExpLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    // valid regex is printed as {}
    // invalid regex is printed as null, which we can't implement yet
    pub value: EmptyObject,
    pub regex: RegExp,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RegExp {
    pub pattern: Atom,
    pub flags: RegExpFlags,
}

impl fmt::Display for RegExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flags)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RegExpFlags: u8 {
        const G = 1 << 0;
        const I = 1 << 1;
        const M = 1 << 2;
        const S = 1 << 3;
        const U = 1 << 4;
        const Y = 1 << 5;
        const D = 1 << 6;
        /// v flag from `https://github.com/tc39/proposal-regexp-set-notation`
        const V = 1 << 7;
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

// TODO: should we implement TryFrom<&str> too?

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

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct EmptyObject;

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type"))]
pub struct StringLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: Atom,
}

impl StringLiteral {
    /// Static Semantics: `IsStringWellFormedUnicode`
    /// test for \uD800-\uDFFF
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
