use crate::constant_evaluation::ValueType;

use super::traits::BuiltinValue;

pub struct Infinity;
impl BuiltinValue for Infinity {
    fn value_type(&self) -> ValueType {
        ValueType::Number
    }
}

pub struct NaN;
impl BuiltinValue for NaN {
    fn value_type(&self) -> ValueType {
        ValueType::Number
    }
}

pub struct Undefined;
impl BuiltinValue for Undefined {
    fn value_type(&self) -> ValueType {
        ValueType::Undefined
    }
}
