use std::fmt::Debug;

use oxc_ast::ast::{BlockStatement, NumericLiteral, Program};
use oxc_ast::AstOwnedKind;

use crate::tst::TstContext;

pub trait VisitTransform<'a>: Debug {
    fn transform_program(&mut self, program: &mut Program<'a>, context: &mut TstContext<'a>) {}

    fn transform_block_statement(
        &mut self,
        block: &mut BlockStatement<'a>,
        context: &mut TstContext<'a>,
    ) {
    }

    fn transform_numeric_literal(
        &mut self,
        num: &mut NumericLiteral<'a>,
        context: &mut TstContext<'a>,
    ) {
    }
}

#[derive(Debug)]
pub struct NumericSeparators;

impl<'a> VisitTransform<'a> for NumericSeparators {
    fn transform_numeric_literal(
        &mut self,
        num: &mut NumericLiteral<'a>,
        context: &mut TstContext<'a>,
    ) {
        let in_program = context.check_ancestor(|node| matches!(node, AstOwnedKind::Program(_)));

        dbg!("in_program", in_program);

        let in_block_statement =
            context.check_ancestor(|node| matches!(node, AstOwnedKind::BlockStatement(_)));

        dbg!("in_block_statement", in_block_statement);

        if num.raw.contains('_') {
            num.raw = context.new_str(num.raw.replace('_', "").as_str());
        }
    }
}
