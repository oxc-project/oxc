// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_eq.rs`

#![allow(clippy::match_like_matches_macro)]

use oxc_span::cmp::ContentEq;

use crate::number::*;
use crate::operator::*;

impl ContentEq for NumberBase {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for BigintBase {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for AssignmentOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for BinaryOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for LogicalOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for UnaryOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl ContentEq for UpdateOperator {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}
