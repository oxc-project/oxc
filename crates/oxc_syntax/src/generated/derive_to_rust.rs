// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/to_rust.rs`

#![allow(clippy::redundant_closure_for_method_calls)]

use crate::number::*;
use crate::operator::*;

impl ::oxc_quote_types::ToRust for NumberBase {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Float => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "NumberBase",
                    variant: "Float",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Decimal => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "NumberBase",
                    variant: "Decimal",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Binary => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "NumberBase",
                    variant: "Binary",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Octal => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "NumberBase",
                    variant: "Octal",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Hex => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "NumberBase",
                    variant: "Hex",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for BigintBase {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Decimal => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BigintBase",
                    variant: "Decimal",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Binary => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BigintBase",
                    variant: "Binary",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Octal => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BigintBase",
                    variant: "Octal",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Hex => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BigintBase",
                    variant: "Hex",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for AssignmentOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Assign => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Assign",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Addition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Addition",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Subtraction => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Subtraction",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Multiplication => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Multiplication",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Division => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Division",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Remainder => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Remainder",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Exponential => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "Exponential",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ShiftLeft => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "ShiftLeft",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ShiftRight => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "ShiftRight",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ShiftRightZeroFill => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "ShiftRightZeroFill",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseOR => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "BitwiseOR",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseXOR => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "BitwiseXOR",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseAnd => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "BitwiseAnd",
                    field: ::std::option::Option::None,
                }))
            }
            Self::LogicalOr => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "LogicalOr",
                    field: ::std::option::Option::None,
                }))
            }
            Self::LogicalAnd => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "LogicalAnd",
                    field: ::std::option::Option::None,
                }))
            }
            Self::LogicalNullish => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "AssignmentOperator",
                    variant: "LogicalNullish",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for BinaryOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Equality => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Equality",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Inequality => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Inequality",
                    field: ::std::option::Option::None,
                }))
            }
            Self::StrictEquality => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "StrictEquality",
                    field: ::std::option::Option::None,
                }))
            }
            Self::StrictInequality => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "StrictInequality",
                    field: ::std::option::Option::None,
                }))
            }
            Self::LessThan => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "LessThan",
                    field: ::std::option::Option::None,
                }))
            }
            Self::LessEqualThan => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "LessEqualThan",
                    field: ::std::option::Option::None,
                }))
            }
            Self::GreaterThan => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "GreaterThan",
                    field: ::std::option::Option::None,
                }))
            }
            Self::GreaterEqualThan => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "GreaterEqualThan",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Addition => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Addition",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Subtraction => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Subtraction",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Multiplication => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Multiplication",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Division => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Division",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Remainder => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Remainder",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Exponential => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Exponential",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ShiftLeft => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "ShiftLeft",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ShiftRight => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "ShiftRight",
                    field: ::std::option::Option::None,
                }))
            }
            Self::ShiftRightZeroFill => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "ShiftRightZeroFill",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseOR => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "BitwiseOR",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseXOR => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "BitwiseXOR",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseAnd => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "BitwiseAnd",
                    field: ::std::option::Option::None,
                }))
            }
            Self::In => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "In",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Instanceof => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "BinaryOperator",
                    variant: "Instanceof",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for LogicalOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Or => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LogicalOperator",
                    variant: "Or",
                    field: ::std::option::Option::None,
                }))
            }
            Self::And => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LogicalOperator",
                    variant: "And",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Coalesce => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "LogicalOperator",
                    variant: "Coalesce",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for UnaryOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::UnaryPlus => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "UnaryPlus",
                    field: ::std::option::Option::None,
                }))
            }
            Self::UnaryNegation => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "UnaryNegation",
                    field: ::std::option::Option::None,
                }))
            }
            Self::LogicalNot => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "LogicalNot",
                    field: ::std::option::Option::None,
                }))
            }
            Self::BitwiseNot => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "BitwiseNot",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Typeof => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "Typeof",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Void => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "Void",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Delete => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UnaryOperator",
                    variant: "Delete",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}

impl ::oxc_quote_types::ToRust for UpdateOperator {
    fn to_rust(&self) -> ::oxc_quote_types::Node {
        match self {
            Self::Increment => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UpdateOperator",
                    variant: "Increment",
                    field: ::std::option::Option::None,
                }))
            }
            Self::Decrement => {
                ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: "UpdateOperator",
                    variant: "Decrement",
                    field: ::std::option::Option::None,
                }))
            }
        }
    }
}
