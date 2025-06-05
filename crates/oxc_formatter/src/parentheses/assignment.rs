use oxc_ast::{AstKind, ast::*};
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::{
    formatter::{Formatter, parent_stack::ParentStack},
    generated::ast_nodes::AstNode,
};

use super::NeedsParentheses;

impl<'a, 'b> NeedsParentheses<'a> for AstNode<'a, 'b, SimpleAssignmentTarget<'a>> {
    fn needs_parentheses(&self, stack: &Formatter<'_, 'a>) -> bool {
        false
    }
}
