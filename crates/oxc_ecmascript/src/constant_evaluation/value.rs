use core::f64;
use std::borrow::Cow;

use num_bigint::BigInt;

use crate::{ToJsString, ToNumber};

#[derive(Debug, PartialEq)]
pub enum ConstantValue<'a> {
    Number(f64),
    BigInt(BigInt),
    String(Cow<'a, str>),
    Boolean(bool),
    Undefined,
    Null,
}

impl<'a> ConstantValue<'a> {
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn into_string(self) -> Option<Cow<'a, str>> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_number(self) -> Option<f64> {
        match self {
            Self::Number(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_boolean(self) -> Option<bool> {
        match self {
            Self::Boolean(s) => Some(s),
            _ => None,
        }
    }
}

impl<'a> ToJsString<'a> for ConstantValue<'a> {
    fn to_js_string(&self) -> Option<Cow<'a, str>> {
        match self {
            Self::Number(n) => {
                use oxc_syntax::number::ToJsString;
                Some(Cow::Owned(n.to_js_string()))
            }
            // https://tc39.es/ecma262/#sec-numeric-types-bigint-tostring
            Self::BigInt(n) => Some(Cow::Owned(n.to_string())),
            Self::String(s) => Some(s.clone()),
            Self::Boolean(b) => Some(Cow::Borrowed(if *b { "true" } else { "false" })),
            Self::Undefined => Some(Cow::Borrowed("undefined")),
            Self::Null => Some(Cow::Borrowed("null")),
        }
    }
}

impl<'a> ToNumber<'a> for ConstantValue<'a> {
    fn to_number(&self) -> Option<f64> {
        use crate::StringToNumber;
        match self {
            Self::Number(n) => Some(*n),
            Self::BigInt(_) => None,
            Self::String(s) => Some(s.as_ref().string_to_number()),
            Self::Boolean(true) => Some(1.0),
            Self::Boolean(false) | Self::Null => Some(0.0),
            Self::Undefined => Some(f64::NAN),
        }
    }
}
