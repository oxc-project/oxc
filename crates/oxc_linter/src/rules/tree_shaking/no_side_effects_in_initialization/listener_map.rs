use std::cell::RefCell;

use oxc_ast::ast::{
    ArrayExpressionElement, AssignmentTarget, ComputedMemberExpression, Expression,
    IdentifierReference, MemberExpression, PrivateFieldExpression, Program, SimpleAssignmentTarget,
    Statement, StaticMemberExpression,
};
use oxc_semantic::SymbolId;
use rustc_hash::FxHashSet;

use crate::{ast_util::get_symbol_id_of_variable, utils::Value, LintContext};

use super::NoSideEffectsDiagnostic;

pub struct NodeListenerOptions<'a, 'b> {
    checked_mutated_nodes: RefCell<FxHashSet<SymbolId>>,
    ctx: &'b LintContext<'a>,
}

impl<'a, 'b> NodeListenerOptions<'a, 'b> {
    fn insert_mutated_node(&self, symbol_id: SymbolId) -> bool {
        self.checked_mutated_nodes.borrow_mut().insert(symbol_id)
    }
}

impl<'a, 'b> NodeListenerOptions<'a, 'b> {
    pub fn new(ctx: &'b LintContext<'a>) -> Self {
        Self { checked_mutated_nodes: RefCell::new(FxHashSet::default()), ctx }
    }
}

pub trait ListenerMap {
    fn report_effects(&self, _options: &NodeListenerOptions) {}
    fn report_effects_when_assigned(&self, _options: &NodeListenerOptions) {}
    fn report_effects_when_called(&self, _options: &NodeListenerOptions) {}
    fn report_effects_when_mutated(&self, _options: &NodeListenerOptions) {}
    fn get_value_and_report_effects(&self, _options: &NodeListenerOptions) -> Option<Value> {
        None
    }
}

impl<'a> ListenerMap for Program<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.body.iter().for_each(|stmt| stmt.report_effects(options));
    }
}

impl<'a> ListenerMap for Statement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        if let Self::ExpressionStatement(expr_stmt) = self {
            expr_stmt.expression.report_effects(options);
        }
    }
}

impl<'a> ListenerMap for Expression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::ArrayExpression(array_expr) => {
                array_expr.elements.iter().for_each(|el| el.report_effects(options));
            }
            Self::AssignmentExpression(assign_expr) => {
                assign_expr.left.report_effects_when_assigned(options);
                assign_expr.right.report_effects(options);
            }
            Self::Identifier(ident) => {
                ident.report_effects(options);
            }
            _ => {}
        }
    }
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        #[allow(clippy::single_match)]
        match self {
            Self::Identifier(ident) => {
                ident.report_effects_when_mutated(options);
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for AssignmentTarget<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        match self {
            Self::SimpleAssignmentTarget(target) => {
                target.report_effects_when_assigned(options);
            }
            Self::AssignmentTargetPattern(_pattern) => {}
        }
    }
}

impl<'a> ListenerMap for SimpleAssignmentTarget<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => {
                ident.report_effects_when_assigned(options);
            }
            Self::MemberAssignmentTarget(member) => {
                member.report_effects_when_assigned(options);
            }
            _ => {
                // For remain TypeScript AST, just visit its expression
                if let Some(expr) = self.get_expression() {
                    expr.report_effects_when_assigned(options);
                }
            }
        }
    }
}

impl<'a> ListenerMap for IdentifierReference<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        if get_symbol_id_of_variable(self, options.ctx).is_none() {
            options.ctx.diagnostic(NoSideEffectsDiagnostic::Assignment(
                self.name.to_compact_str(),
                self.span,
            ));
        }
    }

    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        if let Some(symbol_id) = get_symbol_id_of_variable(self, ctx) {
            if options.insert_mutated_node(symbol_id) {
                for reference in ctx.symbols().get_resolved_references(symbol_id) {
                    if reference.is_write() {
                        ctx.diagnostic(NoSideEffectsDiagnostic::Mutation(
                            self.name.to_compact_str(),
                            self.span,
                        ));
                    }
                }
            }
        } else {
            ctx.diagnostic(NoSideEffectsDiagnostic::Mutation(
                self.name.to_compact_str(),
                self.span,
            ));
        }
    }
}

impl<'a> ListenerMap for MemberExpression<'a> {
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        match self {
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects(options);
                expr.object.report_effects_when_mutated(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects(options);
                expr.object.report_effects_when_mutated(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects(options);
                expr.object.report_effects_when_mutated(options);
            }
        }
    }
}

impl<'a> ListenerMap for ComputedMemberExpression<'a> {}

impl<'a> ListenerMap for StaticMemberExpression<'a> {}

impl<'a> ListenerMap for PrivateFieldExpression<'a> {}

impl<'a> ListenerMap for ArrayExpressionElement<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::Expression(expr) => expr.report_effects(options),
            Self::SpreadElement(spreed) => {
                spreed.argument.report_effects(options);
            }
            Self::Elision(_) => {}
        }
    }
}
