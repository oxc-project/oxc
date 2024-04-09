use std::fmt::Debug;

use oxc_ast::ast::{BlockStatement, NumericLiteral, Program};
use oxc_ast::AstOwnedKind;
use oxc_span::Span;

use crate::tst::{MutationKind, TransformContext};

pub trait VisitTransform<'a>: Debug {
    fn transform_program(&mut self, program: &mut Program<'a>, context: &mut TransformContext<'a>) {
    }

    fn transform_block_statement(
        &mut self,
        block: &mut BlockStatement<'a>,
        context: &mut TransformContext<'a>,
    ) {
    }

    fn transform_numeric_literal(
        &mut self,
        num: &mut NumericLiteral<'a>,
        context: &mut TransformContext<'a>,
    ) {
    }
}

#[derive(Debug)]
pub struct NumericSeparators;

impl<'a> VisitTransform<'a> for NumericSeparators {
    fn transform_numeric_literal(
        &mut self,
        num: &mut NumericLiteral<'a>,
        context: &mut TransformContext<'a>,
    ) {
        let in_program =
            context.query_ancestors(|node| Some(matches!(node, AstOwnedKind::Program(_))));

        dbg!("in_program", in_program);

        let in_block_statement = context.query_ancestor_paths(|node| {
            if matches!(node.as_node(), AstOwnedKind::BlockStatement(_)) {
                Some(node.id)
            } else {
                None
            }
        });

        dbg!("in_block_statement", in_block_statement);

        // If we're in a block statement, try mutating the parent block
        if let Some(block_id) = in_block_statement {
            context.mutate_node(block_id, |node| {
                if let AstOwnedKind::BlockStatement(inner) = node {
                    inner.span = Span::new(1000, 1000);
                }

                MutationKind::Keep
            });
        }

        // Mutate the node itself
        if num.raw.contains('_') {
            num.raw = context.new_str(num.raw.replace('_', "").as_str());
        }
    }
}
