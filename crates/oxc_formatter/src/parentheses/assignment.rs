use oxc_ast::{AstKind, ast::*};
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{formatter::Formatter, generated::ast_nodes::AstNode};

use super::NeedsParentheses;

impl<'a> NeedsParentheses<'a> for AstNode<'a, SimpleAssignmentTarget<'a>> {
    fn needs_parentheses(&self, stack: &Formatter<'_, 'a>) -> bool {
        false
    }
}
