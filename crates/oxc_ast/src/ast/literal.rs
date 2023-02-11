//! Literals

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use num_bigint::BigUint;
use ordered_float::NotNan;
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};

use crate::{Atom, Node};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename = "Literal")]
pub struct BooleanLiteral {
    #[serde(flatten)]
    pub node: Node,
    pub value: bool,
}

impl BooleanLiteral {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        if self.value { "true" } else { "false" }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct NullLiteral {
    pub node: Node,
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

impl Serialize for NullLiteral {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NullLiteral", 4)?;
        state.serialize_field("type", &"Literal")?;
        state.serialize_field("start", &self.node.start)?;
        state.serialize_field("end", &self.node.end)?;
        state.serialize_field("value", &())?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename = "Literal")]
pub struct NumberLiteral<'a> {
    #[serde(flatten)]
    pub node: Node,
    pub value: NotNan<f64>, // using NotNan for `Hash`
    #[serde(skip)]
    pub raw: &'a str,
    #[serde(skip)]
    pub base: NumberBase,
}

impl<'a> NumberLiteral<'a> {
    #[must_use]
    pub const fn new(node: Node, value: f64, raw: &'a str, base: NumberBase) -> Self {
        let value = unsafe { NotNan::new_unchecked(value) };
        Self { node, value, raw, base }
    }
}

impl<'a> Hash for NumberLiteral<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename = "Literal")]
pub struct BigintLiteral {
    #[serde(flatten)]
    pub node: Node,
    #[serde(serialize_with = "crate::serialize::serialize_bigint")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename = "Literal")]
pub struct RegExpLiteral {
    #[serde(flatten)]
    pub node: Node,
    // valid regex is printed as {}
    // invalid regex is printed as null, which we can't implement yet
    pub value: EmptyObject,
    pub regex: RegExp,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct RegExp {
    pub pattern: Atom,
    pub flags: Atom,
}

impl fmt::Display for RegExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flags)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct EmptyObject {}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename = "Literal")]
pub struct StringLiteral {
    #[serde(flatten)]
    pub node: Node,
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

impl<'a> NumberLiteral<'a> {
    #[allow(clippy::inherent_to_string)]
    fn to_string(&self) -> String {
        let mut buffer = ryu_js::Buffer::new();
        buffer.format(*self.value).to_string()
    }

    /// From [boa](https://github.com/boa-dev/boa/blob/52bc15bc2320cd6cbc661a138ae955ceb0c9597a/boa_engine/src/builtins/number/mod.rs#L417)
    /// [spec]: `https://tc39.es/ecma262/#sec-number.prototype.toprecision`
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    #[must_use]
    pub fn to_precision(&self, precision: Option<usize>) -> Option<String> {
        // 1 & 6
        let mut this_num = self.value;
        // 2
        let Some(precision) = precision else {
            return Some(self.to_string());
        };

        // 3
        // let precision = precision.to_integer_or_infinity(context)?;

        // 4
        if !this_num.is_finite() {
            return Some(self.to_string());
        }

        // let precision = match precision {
        // IntegerOrInfinity::Integer(x) if (1..=100).contains(&x) => x as usize,
        // _ => {
        // // 5
        // return context.throw_range_error(
        // "precision must be an integer at least 1 and no greater than 100",
        // );
        // }
        // };
        if !(1..=100).contains(&precision) {
            return None;
        }
        let precision_i32 = precision as i32;

        // 7
        let mut prefix = String::new(); // spec: 's'
        let mut suffix: String; // spec: 'm'
        let mut exponent: i32; // spec: 'e'

        // 8
        if *this_num < 0.0 {
            prefix.push('-');
            this_num = -this_num;
        }

        // 9
        if this_num == 0.0 {
            suffix = "0".repeat(precision);
            exponent = 0;
        // 10
        } else {
            // Due to f64 limitations, this part differs a bit from the spec,
            // but has the same effect. It manipulates the string constructed
            // by `format`: digits with an optional dot between two of them.
            suffix = format!("{this_num:.100}");

            // a: getting an exponent
            exponent = Self::flt_str_to_exp(&suffix);
            // b: getting relevant digits only
            if exponent < 0 {
                suffix = suffix.split_off((1 - exponent) as usize);
            } else if let Some(n) = suffix.find('.') {
                suffix.remove(n);
            }
            // impl: having exactly `precision` digits in `suffix`
            if Self::round_to_precision(&mut suffix, precision) {
                exponent += 1;
            }

            // c: switching to scientific notation
            let great_exp = exponent >= precision_i32;
            if exponent < -6 || great_exp {
                // ii
                if precision > 1 {
                    suffix.insert(1, '.');
                }
                // vi
                suffix.push('e');
                // iii
                if great_exp {
                    suffix.push('+');
                }
                // iv, v
                suffix.push_str(&exponent.to_string());

                return Some(format!("{prefix}{suffix}"));
            }
        }

        // 11
        let e_inc = exponent + 1;
        if e_inc == precision_i32 {
            return Some(format!("{prefix}{suffix}"));
        }

        // 12
        if exponent >= 0 {
            suffix.insert(e_inc as usize, '.');
        // 13
        } else {
            prefix.push('0');
            prefix.push('.');
            prefix.push_str(&"0".repeat(-e_inc as usize));
        }

        // 14
        Some(format!("{prefix}{suffix}"))
    }

    /// `round_to_precision` - used in `to_precision`
    ///
    /// This procedure has two roles:
    /// - If there are enough or more than enough digits in the
    ///   string to show the required precision, the number
    ///   represented by these digits is rounded using string
    ///   manipulation.
    /// - Else, zeroes are appended to the string.
    /// - Additionally, sometimes the exponent was wrongly computed and
    ///   while up-rounding we find that we need an extra digit. When this
    ///   happens, we return true so that the calling context can adjust
    ///   the exponent. The string is kept at an exact length of `precision`.
    ///
    /// When this procedure returns, `digits` is exactly `precision` long.
    ///
    fn round_to_precision(digits: &mut String, precision: usize) -> bool {
        if digits.len() > precision {
            let to_round = digits.split_off(precision);
            let mut digit =
                digits.pop().expect("already checked that length is bigger than precision") as u8;
            if let Some(first) = to_round.chars().next() {
                if first > '4' {
                    digit += 1;
                }
            }

            if digit as char == ':' {
                // ':' is '9' + 1
                // need to propagate the increment backward
                let mut replacement = String::from("0");
                let mut propagated = false;
                for c in digits.chars().rev() {
                    let d = match (c, propagated) {
                        ('0'..='8', false) => (c as u8 + 1) as char,
                        (_, false) => '0',
                        (_, true) => c,
                    };
                    replacement.push(d);
                    if d != '0' {
                        propagated = true;
                    }
                }
                digits.clear();
                let replacement = if propagated {
                    replacement.as_str()
                } else {
                    digits.push('1');
                    &replacement.as_str()[1..]
                };
                for c in replacement.chars().rev() {
                    digits.push(c);
                }
                !propagated
            } else {
                digits.push(digit as char);
                false
            }
        } else {
            digits.push_str(&"0".repeat(precision - digits.len()));
            false
        }
    }

    /// `flt_str_to_exp` - used in `to_precision`
    ///
    /// This function traverses a string representing a number,
    /// returning the floored log10 of this number.
    ///
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    fn flt_str_to_exp(flt: &str) -> i32 {
        let mut non_zero_encountered = false;
        let mut dot_encountered = false;
        for (i, c) in flt.chars().enumerate() {
            if c == '.' {
                if non_zero_encountered {
                    return (i as i32) - 1;
                }
                dot_encountered = true;
            } else if c != '0' {
                if dot_encountered {
                    return 1 - (i as i32);
                }
                non_zero_encountered = true;
            }
        }
        (flt.len() as i32) - 1
    }
}
