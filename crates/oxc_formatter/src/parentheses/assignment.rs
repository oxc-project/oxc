use oxc_ast::{AstKind, ast::*};
use oxc_data_structures::stack;
use oxc_span::GetSpan;
use oxc_syntax::{
    operator,
    precedence::{GetPrecedence, Precedence},
};

use crate::formatter::parent_stack::ParentStack;

use super::NeedsParentheses;

impl<'a> NeedsParentheses<'a> for SimpleAssignmentTarget<'a> {
    fn needs_parentheses(&self, stack: &ParentStack<'a>) -> bool {
        false
    }
}
