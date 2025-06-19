use oxc_ast::ast::Expression;

use crate::{
    is_global_reference::IsGlobalReference,
    to_primitive::{ToPrimitive, ToPrimitiveResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToNumericResult {
    Number,
    BigInt,
    Undetermined,
}

impl ToNumericResult {
    pub fn is_number(self) -> bool {
        matches!(self, Self::Number)
    }

    pub fn is_bigint(self) -> bool {
        matches!(self, Self::BigInt)
    }
}

/// `ToNumeric`
///
/// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumeric>
pub trait ToNumeric<'a> {
    fn to_numeric(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ToNumericResult;
}

impl<'a> ToNumeric<'a> for Expression<'a> {
    fn to_numeric(&self, is_global_reference: &impl IsGlobalReference<'a>) -> ToNumericResult {
        self.to_primitive(is_global_reference).to_numeric(is_global_reference)
    }
}

impl<'a> ToNumeric<'a> for ToPrimitiveResult {
    fn to_numeric(&self, _is_global_reference: &impl IsGlobalReference<'a>) -> ToNumericResult {
        match self {
            // Symbol throws an error when passed to ToNumber in step 3
            ToPrimitiveResult::Symbol | ToPrimitiveResult::Undetermined => {
                ToNumericResult::Undetermined
            }
            ToPrimitiveResult::BigInt => ToNumericResult::BigInt,
            ToPrimitiveResult::Boolean
            | ToPrimitiveResult::Null
            | ToPrimitiveResult::Number
            | ToPrimitiveResult::String
            | ToPrimitiveResult::Undefined => ToNumericResult::Number,
        }
    }
}
