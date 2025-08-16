use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    formatter::Formatter,
    generated::ast_nodes::{AstNode, AstNodes},
};

use super::NeedsParentheses;

impl<'a> NeedsParentheses<'a> for AstNode<'a, AssignmentTarget<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self.as_ast_nodes() {
            AstNodes::IdentifierReference(ident) => {
                if ident.name == "async" {
                    matches!(self.parent, AstNodes::ForOfStatement(for_of) if !for_of.r#await)
                } else if ident.name == "let" {
                    matches!(self.parent, AstNodes::ForOfStatement(_))
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, AssignmentTargetPattern<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}

impl<'a> NeedsParentheses<'a> for AstNode<'a, SimpleAssignmentTarget<'a>> {
    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        false
    }
}
