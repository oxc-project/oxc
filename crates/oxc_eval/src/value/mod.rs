//! Contains the [`Value`] enum, which represents an ECMAScript language value.

mod numeric;
mod object;
mod type_conversion;

use std::{
    borrow::{Borrow, Cow},
    rc::Rc,
};

use object::Object;
use oxc_ast::BigInt;
use oxc_span::Atom;

pub use numeric::{Number, Numeric};

/// ## References
/// - [ECMAScript - 4.3.1 Objects](https://262.ecma-international.org/15.0/index.html#sec-objects)
/// - [V8 - Value](https://github.com/v8/v8/blob/main/include/v8-value.h)
#[derive(Debug, Default, Clone, Hash)]
#[must_use]
pub enum Value<'a> {
    Boolean(bool),
    Number(Number),
    BigInt(BigInt),
    String(Cow<'a, str>),
    Symbol(Atom<'a>),

    // TODO: objects, functions
    Object(Rc<Object>),
    #[default]
    Undefined,
    Null,
}

// impl<'a> From<Value<'a>> for f64 {
//     fn from(val: Value<'a>) -> Self {
//         val.as_number().unwrap_or(f64::NAN)
//     }
// }

impl<'a> Value<'a> {
    /// Returns `true` if this [`Value`] is the `undefined` [`Value`].
    ///
    /// This is equivalent to
    /// ```js
    /// value === undefined
    /// ```
    ///
    /// ## References
    /// - [ECMA-262 `undefined` value](https://262.ecma-international.org/15.0/index.html#sec-undefined-value)
    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(self, Value::Undefined)
    }

    /// Returns `true` if this [`Value`] is the `null` [`Value`].
    ///
    /// This is equivalent to
    /// ```js
    /// value === null
    /// ```
    ///
    /// ## References
    /// - [ECMA-262 - 4.4.16 `null` value](https://262.ecma-international.org/15.0/index.html#sec-null-value)
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns `true` if this [`Value`] is the null or the undefined [`Value`].
    ///
    /// This is equivalent to
    /// ```js
    /// value == null
    /// ```
    #[inline]
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    /// Returns `true` if this [`Value`] is `true`.
    ///
    /// This is not the same as `BooleanValue()`. The latter performs a
    /// conversion to [`Value::Boolean`], i.e.e the result of `Boolean(value)`
    /// in JS, whereas this method checks `value === true`.
    ///
    /// Use [`Value::is_false`] to check for `false`.
    ///
    /// ## References
    /// - [ECMA-262 4.4.18 Boolean type](https://262.ecma-international.org/15.0/index.html#sec-terms-and-definitions-boolean-type)
    #[inline]
    pub fn is_true(&self) -> bool {
        matches!(self, Value::Boolean(true))
    }

    /// Returns `true` if this [`Value`] is `false`.
    ///
    /// This is not the same as `!BooleanValue()`. The latter performs a
    /// conversion to [`Value::Boolean`], i.e.e the result of `!Boolean(value)`
    /// in JS, whereas this method checks `value === false`.
    ///
    ///
    /// Use [`Value::is_true`] to check for `true`.
    ///
    /// ## References
    /// - [ECMA-262 4.4.18 Boolean type](https://262.ecma-international.org/15.0/index.html#sec-terms-and-definitions-boolean-type)
    #[inline]
    pub fn is_false(&self) -> bool {
        matches!(self, Value::Boolean(false))
    }

    /// Returns `true` if this [`Value`] is a a [`symbol`] or [`string`]
    ///
    /// This is equivalent to
    /// ```js
    /// typeof value === 'string' || typeof value === 'symbol'
    /// ```
    ///
    /// [`symbol`]: Value::Symbol
    /// [`string`]: Value::String
    #[inline]
    pub fn is_name(&self) -> bool {
        matches!(self, Value::String(_) | Value::Symbol(_))
    }

    /// Returns `true` if this [`Value`] is an instance of the `String` type.
    ///
    /// This is equivalent to
    /// ```js
    /// typeof value === 'string'
    /// ```
    ///
    /// ## References
    /// - [ECMA262 - 4.4.22 String object](https://262.ecma-international.org/15.0/index.html#sec-string-object)
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns `true` if this [`Value`] is a unique symbol.
    ///
    /// This is equivalent to
    /// ```js
    /// typeof value === 'symbol'
    /// ```
    ///
    /// ## References
    /// - [ECMA262 - 4.4.31 Symbol value](https://262.ecma-international.org/15.0/index.html#sec-symbol-value)
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    /// Returns `true` if this [`Value`] is a function.
    /// > _NOTE: functions are not yet implemented. This method will panic._
    ///
    /// This is equivalent to
    /// ```js
    /// typeof value === 'function'
    /// ```
    ///
    pub fn is_function(&self) -> bool {
        todo!("Function values are not yet implemented")
    }

    /// Returns `true` if this [`Value`] is an object.
    #[inline]
    pub fn is_object(&self) -> bool {
        // TODO: check if V8 also returns `true` for functions, arrays, etc
        matches!(self, Value::Object(_))
    }

    /// Returns `true` if this [`Value`] is a bigint.
    ///
    /// This is equivalent to
    /// ```js
    /// typeof value === 'bigint'
    /// ```
    #[inline]
    pub fn is_big_int(&self) -> bool {
        matches!(self, Value::BigInt(_))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(b) => *b,
            Self::Number(n) => *n != Number::ZERO,
            Self::BigInt(n) => n != &BigInt::ZERO,
            Self::String(s) => !s.is_empty(),
            Self::Symbol(_) | Self::Object(_) => true,
            Self::Undefined | Self::Null => false,
        }
    }

    #[inline]
    pub fn is_falsey(&self) -> bool {
        !self.is_truthy()
    }

    // pub fn as_number(&self) -> Option<f64> {
    //     const fn int_to_float(n: i64) -> Option<f64> {
    //         if n < (f64::MAX as i64) {
    //             Some(n as f64)
    //         } else {
    //             None
    //         }
    //     }

    //     match self {
    //         Self::Undefined => None,
    //         Self::Null => Some(0.0),
    //         Self::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
    //         Self::Number(n) => Some(*n),
    //         Self::BigInt(n) => i64::try_from(n).map(int_to_float).ok().flatten(),
    //         Self::String(s) => s.parse().ok(),
    //         Self::Symbol(_) | Self::Object(_) => None,
    //     }
    // }

    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Value::Undefined => Cow::Borrowed("undefined"),
            Value::Null => Cow::Borrowed("null"),
            Value::Boolean(b) => Cow::Borrowed(if *b { "true" } else { "false" }),
            Value::Number(n) => Cow::Owned(n.to_string()),
            Value::BigInt(n) => Cow::Owned(n.to_string() + "n"),
            Value::String(s) => Cow::Borrowed(s.borrow()),
            Value::Symbol(s) => Cow::Borrowed(s.as_str()),
            Value::Object(o) => Cow::Borrowed(o.name()),
        }
    }
}
