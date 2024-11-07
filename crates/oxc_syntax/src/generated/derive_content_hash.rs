// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/derives/content_hash.rs`

#![allow(clippy::match_same_arms)]

use std::{hash::Hasher, mem::discriminant};

use oxc_span::hash::ContentHash;

use crate::number::*;

use crate::operator::*;

impl ContentHash for NumberBase {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for BigintBase {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for AssignmentOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for BinaryOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for LogicalOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for UnaryOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}

impl ContentHash for UpdateOperator {
    fn content_hash<H: Hasher>(&self, state: &mut H) {
        ContentHash::content_hash(&discriminant(self), state);
    }
}
