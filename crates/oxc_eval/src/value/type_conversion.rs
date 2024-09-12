//! [7.1 Type
//! Conversion](https://262.ecma-international.org/15.0/index.html#sec-type-conversion)

use std::borrow::Cow;

use oxc_ast::BigInt;
use oxc_span::Atom;

use crate::{completion::TypeError, EvalResult, JsFrom, JsInto, TryJsFrom};

use super::{numeric::Number, Numeric, Value};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum PreferredType {
    String,
    Number,
}

impl<'a> Value<'a> {
    /// [7.1.1 ToPrimitive](https://262.ecma-international.org/15.0/index.html#sec-toprimitive)
    ///
    /// > The abstract operation ToPrimitive takes argument `input` (an ECMAScript
    /// > language value) and optional argument `preferredType` (STRING or NUMBER)
    /// > and returns either a normal completion containing an ECMAScript language
    /// > value or a throw completion. It converts its input argument to a
    /// > non-Object type. If an object is capable of converting to more than one
    /// > primitive type, it may use the optional hint preferredType to favour
    /// > that type.
    ///
    /// > Note:
    /// >
    /// > When ToPrimitive is called without a hint, then it generally behaves
    /// > as if the hint were NUMBER. However, objects may over-ride this
    /// > behaviour by defining a @@toPrimitive method. Of the objects defined in
    /// > this specification only Dates (see 21.4.4.45) and Symbol objects (see
    /// > 20.4.3.5) over-ride the default ToPrimitive behaviour. Dates treat the
    /// > absence of a hint as if the hint were STRING.

    pub fn to_primitive(self, _preferred_type: Option<PreferredType>) -> EvalResult<'a> {
        match self {
            // 1. If input is an Object, then:
            Self::Object(_) => {
                // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
                // b. If exoticToPrim is not undefined, then:
                //   i. If preferredType is not present, then:
                //     1. Let hint be "default".
                //   ii. Else if preferredType is STRING, then:
                //     1. Let hint be "string".
                //   iii. Else,
                //     1. Assert: preferredType is NUMBER.
                //     2. Let hint be "number".
                //   iv. Let result be ? Call(exoticToPrim, input, « hint »).
                //   v. If result is not an Object, return result.
                //   vi. Throw a TypeError exception.
                // c. If preferredType is not present, let preferredType be NUMBER
                // d. Return ? OrdinaryToPrimitive(input, preferredType)

                todo!("Value::to_primitive(Value::Object(_)")
            }
            _ => Ok(self),
        }
    }

    /// ### 7.1.2 ToBoolean(`argument`)
    /// > The abstract operation ToBoolean takes argument `argument` (an ECMAScript
    /// > language value) and returns a Boolean. It converts argument to a value
    /// > of type Boolean.
    pub fn to_boolean(&self) -> bool {
        match self {
            Self::Boolean(b) => *b,
            // NOTE: rounding errors could be a problem here
            Self::Number(n) => *n != Number::ZERO && !n.is_nan(),
            Self::BigInt(n) => n != &BigInt::ZERO,
            Self::String(s) => !s.is_empty(),
            Self::Null | Self::Undefined => false,
            Self::Symbol(_) | Self::Object(_) => true,
        }
    }

    /// ### 7.1.3 ToNumeric(`value`)
    /// > The abstract operation ToNumeric takes argument `value` (an ECMAScript
    /// > language value) and returns either a normal completion containing either
    /// > a Number or a BigInt, or a throw completion. It returns `value` converted
    /// > to a Number or a BigInt.
    pub fn to_numeric(self) -> Result<Numeric, TypeError> {
        // TODO: implement this algorithm when to_primitive is ready
        // 1. Let primValue be ? ToPrimitive(value, NUMBER)
        // 2. if primValue is a BigInt, return primValue
        // 3. Return ? ToNumber(primValue)

        match self {
            Self::BigInt(int) => Ok(Numeric::BigInt(int)),
            _ => self.to_number().map(Numeric::Number),
        }
    }

    /// ### 7.1.4 ToNumber(`argument`)
    /// > The abstract operation ToNumber takes argument `argument` (an ECMAScript
    /// > language value) and returns either a normal completion containing a
    /// > Number or a throw completion. It converts `argument` to a value of type
    /// > Number.
    pub fn to_number(self) -> Result<Number, TypeError> {
        match self {
            Self::Number(n) => Ok(n),
            Self::Symbol(_) => type_err("Cannot convert a Symbol to a Number"),
            Self::BigInt(_) => type_err("Cannot convert a BigInt to a Number"),
            Self::Undefined => Ok(Number::NAN),
            Self::Null | Self::Boolean(false) => Ok(Number::ZERO),
            Self::Boolean(true) => Ok(Number::ONE),
            Self::String(s) => Ok(string_to_number(&s)),
            Self::Object(_obj) => {
                // TODO: to_primitive
                Ok(Number::NAN)
            }
        }
    }

    /// ### 7.1.17 ToString(`argument`)
    /// > The abstract operation ToString takes argument `argument` (an ECMAScript
    /// > language value) and returns either a normal completion containing a
    /// > String or a throw completion. It converts `argument` to a value of type
    /// > String.
    pub fn to_string(self) -> Result<Cow<'a, str>, Option<TypeError>> {
        match self {
            // 1. if argument is a String, return argument
            Self::String(s) => Ok(s),
            // 2. if argument is a Symbol, throw a TypeError exception
            Self::Symbol(_) => some_type_err("Cannot convert a Symbol to a String"),
            // 3. if argument is undefined, return "undefined"
            Self::Undefined => Ok(Cow::Borrowed("undefined")),
            // 4. if argument is null, return "null"
            Self::Null => Ok(Cow::Borrowed("null")),
            // 5. if argument is true, return "true"
            // 6. if argument is false, return "false"
            Self::Boolean(b) => Ok(Cow::Borrowed(if b { "true" } else { "false" })),
            // 7. If argument is a Number, return Number::toString(argument, 10).
            Self::Number(n) => Ok(Cow::Owned(n.to_string())),
            // 8. if argument is a BigInt, return BigInt::toString(argument, 10)
            Self::BigInt(n) => Ok(Cow::Owned(n.to_string())),
            // 9. Assert: argument is an Object
            Self::Object(_obj) => {
                // // 10. let primValue be ? ToPrimitive(argument, STRING)
                // let prim = self.to_primitive(None)?;
                // // 11. Assert: primValue is not an Object
                // assert!(!prim.is_object());
                // // 12. Return ? ToString(primValue)
                // prim.to_string()
                todo!("Value::to_string(Self::Object(_obj))")
            }
        }
    }
}

// boolean

impl From<bool> for Value<'static> {
    #[inline]
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl JsFrom<bool> for Value<'static> {
    #[inline]
    fn from_js(value: bool) -> Self {
        Self::from(value)
    }
}

impl JsFrom<Value<'_>> for bool {
    #[inline]
    fn from_js(value: Value<'_>) -> bool {
        value.to_boolean()
    }
}

// numeric types

impl<N: Into<Number>> From<N> for Value<'static> {
    #[inline]
    fn from(n: N) -> Self {
        Value::Number(n.into())
    }
}
impl<N: JsInto<Number>> JsFrom<N> for Value<'static> {
    #[inline]
    fn from_js(value: N) -> Self {
        Value::Number(value.into_js())
    }
}

impl From<BigInt> for Value<'static> {
    #[inline]
    fn from(n: BigInt) -> Self {
        Value::BigInt(n)
    }
}

impl TryJsFrom<Value<'_>> for Number {
    type Error = TypeError;

    #[inline]
    fn try_from_js(value: Value<'_>) -> Result<Self, Self::Error> {
        value.to_number()
    }
}

impl TryJsFrom<Value<'_>> for Numeric {
    type Error = TypeError;

    #[inline]
    fn try_from_js(value: Value<'_>) -> Result<Self, Self::Error> {
        value.to_numeric()
    }
}

// string types

impl<'a> From<&'a str> for Value<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Value::String(Cow::Borrowed(s))
    }
}

impl<'a> From<Atom<'a>> for Value<'a> {
    #[inline]
    fn from(atom: Atom<'a>) -> Self {
        Value::String(Cow::Borrowed(atom.as_str()))
    }
}

impl From<String> for Value<'static> {
    fn from(value: String) -> Self {
        Value::String(Cow::Owned(value))
    }
}

impl<'a> TryJsFrom<Value<'a>> for Cow<'a, str> {
    type Error = Option<TypeError>;

    #[inline]
    fn try_from_js(value: Value<'a>) -> Result<Self, Self::Error> {
        value.to_string()
    }
}

impl<'a> TryJsFrom<Value<'a>> for String {
    type Error = Option<TypeError>;

    #[inline]
    fn try_from_js(value: Value<'a>) -> Result<Self, Self::Error> {
        value.to_string().map(Cow::into_owned)
    }
}

/// ### 7.1.4.1.1 StringToNumber(`str`)
///
/// > The abstract operation StringToNumber takes argument `str` (a String) and
/// > returns a Number.
///
/// ### Reference
/// <https://262.ecma-international.org/15.0/index.html#sec-stringtonumber>
fn string_to_number(s: &str) -> Number {
    // TODO: follow implementation. This vvvv does not support non-base 10 numbers
    let f: f64 = str::parse(s).unwrap_or(f64::NAN);
    Number::float(f)
}

/// Shorthand for creating a throw completion with a [`TypeError`].
#[inline]
fn type_err<T>(msg: &'static str) -> Result<T, TypeError> {
    Err(TypeError::error(msg))
}

/// Shorthand for creating a throw completion with a [`TypeError`].
#[inline]
fn some_type_err<T>(msg: &'static str) -> Result<T, Option<TypeError>> {
    Err(Some(TypeError::error(msg)))
}
