//! Literals

// NB: `#[visited_node]` attribute on AST nodes does not do anything to the code in this file.
// It is purely a marker for codegen used in `oxc_traverse`. See docs in that crate.

// Silence erroneous warnings from Rust Analyser for `#[derive(Tsify)]`
#![allow(non_snake_case)]

use crate::ast::*;

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use oxc_span::{Atom, Span};
use oxc_syntax::number::NumberBase;

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

impl<'a> NumericLiteral<'a> {
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

impl<'a> Hash for NumericLiteral<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.raw.hash(state);
    }
}

impl<'a> BigIntLiteral<'a> {
    pub fn is_zero(&self) -> bool {
        self.raw == "0n"
    }
}

impl<'a> fmt::Display for RegExp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flags)
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

impl<'a> StringLiteral<'a> {
    pub fn new(span: Span, value: Atom<'a>) -> Self {
        Self { span, value }
    }

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
