use std::cell::Cell;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::{
    AstKind, AstType,
    ast::{Expression, Program, Statement},
    builder::AstBuilder,
};
use oxc_ast_visit::Visit;
use oxc_span::{SourceType, Span};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

fn program(allocator: &Allocator) -> Program<'_> {
    let builder = AstBuilder::new(allocator);
    let expression = Expression::new_parenthesized_expression(
        Span::new(0, 7),
        Expression::new_identifier(Span::new(1, 6), "value", &builder),
        &builder,
    );
    Program::new(
        Span::new(0, 8),
        SourceType::mjs(),
        "(value);",
        ArenaVec::new_in(&builder),
        None,
        ArenaVec::new_in(&builder),
        ArenaVec::from_array_in(
            [Statement::new_expression_statement(Span::new(0, 8), expression, &builder)],
            &builder,
        ),
        &builder,
    )
}

#[derive(Default)]
struct Recorder {
    skip: Option<AstType>,
    entered: Vec<AstType>,
    left: Vec<AstType>,
    scopes_entered: usize,
    scopes_left: usize,
}

impl<'a> Visit<'a> for Recorder {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.entered.push(kind.ty());
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        self.left.push(kind.ty());
    }

    fn skip_children(&mut self, kind: AstKind<'a>) -> bool {
        self.skip == Some(kind.ty())
    }

    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes_entered += 1;
    }

    fn leave_scope(&mut self) {
        self.scopes_left += 1;
    }
}

#[test]
fn pruning_preserves_node_callbacks_and_omits_descendants() {
    let allocator = Allocator::default();
    let program = program(&allocator);
    let mut visitor =
        Recorder { skip: Some(AstType::ParenthesizedExpression), ..Recorder::default() };
    visitor.visit_program(&program);
    assert_eq!(
        visitor.entered,
        [AstType::Program, AstType::ExpressionStatement, AstType::ParenthesizedExpression]
    );
    assert_eq!(visitor.left, visitor.entered.iter().rev().copied().collect::<Vec<_>>());
    assert_eq!(visitor.scopes_entered, 1);
    assert_eq!(visitor.scopes_left, 1);
}

#[test]
fn pruning_a_scoped_node_omits_its_scope_callbacks() {
    let allocator = Allocator::default();
    let program = program(&allocator);
    let mut visitor = Recorder { skip: Some(AstType::Program), ..Recorder::default() };
    visitor.visit_program(&program);
    assert_eq!(visitor.entered, [AstType::Program]);
    assert_eq!(visitor.left, [AstType::Program]);
    assert_eq!(visitor.scopes_entered, 0);
    assert_eq!(visitor.scopes_left, 0);
}

#[test]
fn default_traversal_visits_every_node() {
    let allocator = Allocator::default();
    let program = program(&allocator);
    let mut visitor = Recorder::default();
    visitor.visit_program(&program);
    assert_eq!(
        visitor.entered,
        [
            AstType::Program,
            AstType::ExpressionStatement,
            AstType::ParenthesizedExpression,
            AstType::IdentifierReference
        ]
    );
    assert_eq!(visitor.left, visitor.entered.iter().rev().copied().collect::<Vec<_>>());
    assert_eq!(visitor.scopes_entered, 1);
    assert_eq!(visitor.scopes_left, 1);
}
