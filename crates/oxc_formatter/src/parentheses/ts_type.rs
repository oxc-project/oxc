use oxc_ast::ast::*;
use oxc_span::GetSpan;

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
            AstNodes::TSConstructorType(it) => it.needs_parentheses(f),
            AstNodes::TSUnionType(it) => it.needs_parentheses(f),
            AstNodes::TSIntersectionType(it) => it.needs_parentheses(f),
            AstNodes::TSConditionalType(it) => it.needs_parentheses(f),
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
        match self.parent {
            AstNodes::TSConditionalType(ty) => {
                ty.extends_type().span() == self.span() || ty.check_type().span() == self.span()
            }
            AstNodes::TSUnionType(_) | AstNodes::TSIntersectionType(_) => true,
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSInferType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::TSArrayType(_))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSConstructorType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.parent {
            AstNodes::TSConditionalType(ty) => {
                ty.extends_type().span() == self.span() || ty.check_type().span() == self.span()
            }
            AstNodes::TSUnionType(_) | AstNodes::TSIntersectionType(_) => true,
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSUnionType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::TSArrayType(_))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSIntersectionType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        matches!(self.parent, AstNodes::TSArrayType(_))
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, TSConditionalType<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.parent {
            AstNodes::TSConditionalType(ty) => {
                ty.extends_type().span() == self.span() || ty.check_type().span() == self.span()
            }
            AstNodes::TSUnionType(_) | AstNodes::TSIntersectionType(_) => true,
            _ => false,
        }
    }
}
