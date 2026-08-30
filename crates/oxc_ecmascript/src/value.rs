use std::borrow::Cow;

use num_bigint::BigInt;
use num_traits::Zero;

use crate::{GlobalContext, ToBoolean, ToJsString, ToNumber, ValueType};

#[derive(Debug, PartialEq, Clone)]
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

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::BigInt(_) => ValueType::BigInt,
            Self::String(_) => ValueType::String,
            Self::Boolean(_) => ValueType::Boolean,
            Self::Undefined => ValueType::Undefined,
            Self::Null => ValueType::Null,
        }
    }

    /// <https://tc39.es/ecma262/#sec-isstrictlyequal>
    ///
    /// The derived `PartialEq` already has the spec semantics: `NaN` is not equal to
    /// itself, `+0` equals `-0`, and values of different types are never equal.
    pub fn is_strictly_equal(&self, other: &Self) -> bool {
        self == other
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

    pub fn into_bigint(self) -> Option<BigInt> {
        match self {
            Self::BigInt(s) => Some(s),
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
    fn to_js_string(&self, _ctx: &impl GlobalContext<'a>) -> Option<Cow<'a, str>> {
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
    fn to_number(&self, _ctx: &impl GlobalContext<'a>) -> Option<f64> {
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

impl<'a> ToBoolean<'a> for ConstantValue<'a> {
    fn to_boolean(&self, _ctx: &impl GlobalContext<'a>) -> Option<bool> {
        match self {
            Self::Number(n) => Some(!n.is_nan() && *n != 0.0),
            Self::BigInt(n) => Some(*n != BigInt::zero()),
            Self::String(s) => Some(!s.as_ref().is_empty()),
            Self::Boolean(b) => Some(*b),
            Self::Null | Self::Undefined => Some(false),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_strictly_equal() {
        use ConstantValue::{BigInt as Big, Boolean, Null, Number, String, Undefined};

        assert!(Number(1.0).is_strictly_equal(&Number(1.0)));
        assert!(Number(0.0).is_strictly_equal(&Number(-0.0)));
        assert!(!Number(f64::NAN).is_strictly_equal(&Number(f64::NAN)));
        assert!(!Number(1.0).is_strictly_equal(&Number(2.0)));

        assert!(Big(1.into()).is_strictly_equal(&Big(1.into())));
        assert!(!Big(1.into()).is_strictly_equal(&Big(2.into())));

        assert!(String("a".into()).is_strictly_equal(&String(Cow::Owned("a".to_string()))));
        assert!(!String("a".into()).is_strictly_equal(&String("b".into())));

        assert!(Boolean(true).is_strictly_equal(&Boolean(true)));
        assert!(!Boolean(true).is_strictly_equal(&Boolean(false)));

        assert!(Undefined.is_strictly_equal(&Undefined));
        assert!(Null.is_strictly_equal(&Null));

        // Values of different types are never strictly equal.
        assert!(!Number(1.0).is_strictly_equal(&String("1".into())));
        assert!(!Number(1.0).is_strictly_equal(&Boolean(true)));
        assert!(!Number(1.0).is_strictly_equal(&Big(1.into())));
        assert!(!Number(0.0).is_strictly_equal(&Null));
        assert!(!Undefined.is_strictly_equal(&Null));
        assert!(!String("".into()).is_strictly_equal(&Boolean(false)));
    }
}
