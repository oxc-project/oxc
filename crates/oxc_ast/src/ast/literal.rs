//! Literals

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use bitflags::bitflags;
use num_bigint::BigUint;
use ordered_float::NotNan;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::{Atom, Span};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename = "Literal"))]
pub struct BooleanLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: bool,
}

impl BooleanLiteral {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        if self.value { "true" } else { "false" }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct NullLiteral {
    pub span: Span,
}

impl Hash for NullLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        None::<bool>.hash(state);
    }
}

impl PartialEq for NullLiteral {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename = "Literal"))]
pub struct NumberLiteral<'a> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: NotNan<f64>, // using NotNan for `Hash`
    #[cfg_attr(feature = "serde", serde(skip))]
    pub raw: &'a str,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub base: NumberBase,
}

impl<'a> NumberLiteral<'a> {
    #[must_use]
    pub fn new(span: Span, value: f64, raw: &'a str, base: NumberBase) -> Self {
        let value = unsafe { NotNan::new_unchecked(value) };
        Self { span, value, raw, base }
    }
}

impl<'a> Hash for NumberLiteral<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename = "Literal"))]
pub struct BigintLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::serialize::serialize_bigint"))]
    pub value: BigUint,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename = "Literal"))]
pub struct RegExpLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    // valid regex is printed as {}
    // invalid regex is printed as null, which we can't implement yet
    pub value: EmptyObject,
    pub regex: RegExp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct EmptyObject;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize), serde(tag = "type", rename = "Literal"))]
pub struct StringLiteral {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub span: Span,
    pub value: Atom,
}

impl StringLiteral {
    /// Static Semantics: `IsStringWellFormedUnicode`
    /// test for \uD800-\uDFFF
    #[must_use]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberBase {
    Decimal,
    Binary,
    Octal,
    Hex,
}
