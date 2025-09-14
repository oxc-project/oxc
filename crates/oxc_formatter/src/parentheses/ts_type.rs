use oxc_ast::ast::*;

use super::NeedsParentheses;
use crate::{
    Format,
    formatter::Formatter,
    generated::ast_nodes::{AstNode, AstNodes},
    write::{BinaryLikeExpression, ExpressionLeftSide, should_flatten},
};

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.as_ast_nodes() {
            AstNodes::TSFunctionType(it) => it.needs_parentheses(f),
            AstNodes::TSInferType(it) => it.needs_parentheses(f),
            _ => {
                // TODO: incomplete
                false
            }
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSFunctionType<'a>> {
    #[inline]
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::TSUnionType(_))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSInferType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::TSArrayType(_))
    }
}
