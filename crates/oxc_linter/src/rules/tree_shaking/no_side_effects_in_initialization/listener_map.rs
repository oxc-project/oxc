use std::cell::{Cell, RefCell};

use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, ArrowFunctionExpression, AssignmentTarget,
        BindingPattern, BindingPatternKind, CallExpression, ComputedMemberExpression, Declaration,
        Expression, FormalParameter, Function, IdentifierReference, MemberExpression,
        ModuleDeclaration, NewExpression, ParenthesizedExpression, PrivateFieldExpression, Program,
        SimpleAssignmentTarget, Statement, StaticMemberExpression, ThisExpression,
        VariableDeclarator,
    },
    AstKind,
};
use oxc_semantic::{AstNode, SymbolId};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    ast_util::{get_declaration_of_variable, get_symbol_id_of_variable},
    utils::{get_write_expr, has_pure_notation, no_effects, Value},
    LintContext,
};

use super::NoSideEffectsDiagnostic;
pub struct NodeListenerOptions<'a, 'b> {
    checked_mutated_nodes: RefCell<FxHashSet<SymbolId>>,
    ctx: &'b LintContext<'a>,
    has_valid_this: Cell<bool>,
    called_with_new: Cell<bool>,
}

impl<'a, 'b> NodeListenerOptions<'a, 'b> {
    fn insert_mutated_node(&self, symbol_id: SymbolId) -> bool {
        self.checked_mutated_nodes.borrow_mut().insert(symbol_id)
    }
}

impl<'a, 'b> NodeListenerOptions<'a, 'b> {
    pub fn new(ctx: &'b LintContext<'a>) -> Self {
        Self {
            checked_mutated_nodes: RefCell::new(FxHashSet::default()),
            ctx,
            has_valid_this: Cell::new(false),
            called_with_new: Cell::new(false),
        }
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
        match self {
            Self::ExpressionStatement(expr_stmt) => {
                expr_stmt.expression.report_effects(options);
            }
            Self::BreakStatement(_) | Self::ContinueStatement(_) | Self::EmptyStatement(_) => {
                no_effects();
            }
            Self::Declaration(decl) => {
                decl.report_effects(options);
            }
            Self::ReturnStatement(stmt) => {
                if let Some(arg) = &stmt.argument {
                    arg.report_effects(options);
                }
            }
            Self::ModuleDeclaration(decl) => {
                if matches!(
                    decl.0,
                    ModuleDeclaration::ExportAllDeclaration(_)
                        | ModuleDeclaration::ImportDeclaration(_)
                ) {
                    no_effects();
                }
            }
            Self::TryStatement(stmt) => {
                stmt.block.body.iter().for_each(|stmt| stmt.report_effects(options));
                stmt.handler.iter().for_each(|handler| {
                    handler.body.body.iter().for_each(|stmt| stmt.report_effects(options));
                });
                stmt.finalizer.iter().for_each(|finalizer| {
                    finalizer.body.iter().for_each(|stmt| stmt.report_effects(options));
                });
            }
            _ => {}
        }
    }
}

// we don't need implement all AstNode
// it's same as `reportSideEffectsInDefinitionWhenCalled` in eslint-plugin-tree-shaking
// <https://github.com/lukastaegert/eslint-plugin-tree-shaking/blob/463fa1f0bef7caa2b231a38b9c3557051f506c92/src/rules/no-side-effects-in-initialization.ts#L1070-L1080>
impl<'a> ListenerMap for AstNode<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        #[allow(clippy::single_match)]
        match self.kind() {
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    init.report_effects_when_called(options);
                }
            }
            AstKind::FormalParameter(param) => {
                options.ctx.diagnostic(NoSideEffectsDiagnostic::CallParameter(param.span));
            }
            AstKind::BindingRestElement(rest) => {
                let start = rest.span.start + 3;
                let end = rest.span.end;
                options
                    .ctx
                    .diagnostic(NoSideEffectsDiagnostic::CallParameter(Span::new(start, end)));
            }
            AstKind::Function(function) => {
                let old_val = options.has_valid_this.get();
                options.has_valid_this.set(options.called_with_new.get());
                function.report_effects_when_called(options);
                options.has_valid_this.set(old_val);
            }
            _ => {}
        }
    }
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        #[allow(clippy::single_match)]
        match self.kind() {
            AstKind::VariableDeclarator(decl) => {
                if let Some(init) = &decl.init {
                    init.report_effects_when_mutated(options);
                }
            }
            AstKind::FormalParameter(param) => {
                options.ctx.diagnostic(NoSideEffectsDiagnostic::MutateParameter(param.span));
            }
            AstKind::BindingRestElement(rest) => {
                let start = rest.span.start + 3;
                let end = rest.span.end;
                options
                    .ctx
                    .diagnostic(NoSideEffectsDiagnostic::MutateParameter(Span::new(start, end)));
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for Declaration<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        #[allow(clippy::single_match)]
        match self {
            Self::VariableDeclaration(decl) => {
                decl.declarations.iter().for_each(|decl| decl.report_effects(options));
            }
            _ => {}
        }
    }
}

impl<'a> ListenerMap for VariableDeclarator<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.id.report_effects(options);

        if let Some(init) = &self.init {
            init.report_effects(options);
        }
    }
}

impl<'a> ListenerMap for BindingPattern<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match &self.kind {
            BindingPatternKind::BindingIdentifier(_) => {}
            BindingPatternKind::ArrayPattern(array) => {
                array.elements.iter().for_each(|el| {
                    if let Some(el) = el {
                        el.report_effects(options);
                    }
                });
            }
            BindingPatternKind::ObjectPattern(object) => {
                object.properties.iter().for_each(|prop| {
                    prop.value.report_effects(options);
                });
            }
            BindingPatternKind::AssignmentPattern(assign_p) => {
                assign_p.left.report_effects(options);
                assign_p.right.report_effects(options);
            }
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
            Self::CallExpression(call_expr) => {
                call_expr.report_effects(options);
            }
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects(options);
            }
            Self::NewExpression(expr) => {
                expr.report_effects(options);
            }
            Self::AwaitExpression(expr) => {
                expr.argument.report_effects(options);
            }
            Self::ArrowFunctionExpression(_)
            | Self::FunctionExpression(_)
            | Self::Identifier(_)
            | Self::MetaProperty(_)
            | Self::Super(_)
            | Self::ThisExpression(_) => no_effects(),
            _ => {}
        }
    }
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        #[allow(clippy::single_match)]
        match self {
            Self::Identifier(ident) => {
                ident.report_effects_when_mutated(options);
            }
            Self::ArrowFunctionExpression(_) | Self::ObjectExpression(_) => no_effects(),
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects_when_mutated(options);
            }
            Self::CallExpression(expr) => {
                expr.report_effects_when_mutated(options);
            }
            Self::ThisExpression(expr) => {
                expr.report_effects_when_mutated(options);
            }
            _ => {
                // Default behavior
                options.ctx.diagnostic(NoSideEffectsDiagnostic::Mutate(self.span()));
            }
        }
    }
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        match self {
            Self::CallExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::Identifier(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::FunctionExpression(expr) => {
                let old_val = options.has_valid_this.get();
                options.has_valid_this.set(options.called_with_new.get());
                expr.report_effects_when_called(options);
                options.has_valid_this.set(old_val);
            }
            Self::ArrowFunctionExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            Self::ParenthesizedExpression(expr) => {
                expr.report_effects_when_called(options);
            }
            _ => {
                // Default behavior
                options.ctx.diagnostic(NoSideEffectsDiagnostic::Call(self.span()));
            }
        }
    }
}

// which kind of Expression defines `report_effects_when_called` method.
fn defined_custom_report_effects_when_called(expr: &Expression) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::ArrowFunctionExpression(_)
            | Expression::CallExpression(_)
            | Expression::ClassExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::Identifier(_)
            | Expression::MemberExpression(_)
    )
}

impl ListenerMap for ThisExpression {
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        if !options.has_valid_this.get() {
            options.ctx.diagnostic(NoSideEffectsDiagnostic::MutateOfThis(self.span));
        }
    }
}

impl<'a> ListenerMap for NewExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        if has_pure_notation(self.span, options.ctx) {
            return;
        }
        self.arguments.iter().for_each(|arg| arg.report_effects(options));
        let old_val = options.called_with_new.get();
        options.called_with_new.set(true);
        self.callee.report_effects_when_called(options);
        options.called_with_new.set(old_val);
    }
}

impl<'a> ListenerMap for ParenthesizedExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.expression.report_effects(options);
    }
    fn report_effects_when_assigned(&self, options: &NodeListenerOptions) {
        self.expression.report_effects_when_assigned(options);
    }
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.expression.report_effects_when_called(options);
    }
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        self.expression.report_effects_when_mutated(options);
    }
    fn get_value_and_report_effects(&self, options: &NodeListenerOptions) -> Option<Value> {
        self.expression.get_value_and_report_effects(options)
    }
}

impl<'a> ListenerMap for ArrowFunctionExpression<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.params.items.iter().for_each(|param| param.report_effects(options));
        self.body.statements.iter().for_each(|stmt| stmt.report_effects(options));
    }
}

impl<'a> ListenerMap for Function<'a> {
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        self.params.items.iter().for_each(|param| param.report_effects(options));

        if let Some(body) = &self.body {
            body.statements.iter().for_each(|stmt| stmt.report_effects(options));
        }
    }
}

impl<'a> ListenerMap for FormalParameter<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.pattern.report_effects(options);
    }
}

impl<'a> ListenerMap for CallExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.arguments.iter().for_each(|arg| arg.report_effects(options));
        if defined_custom_report_effects_when_called(&self.callee) {
            let old_value = options.called_with_new.get();
            options.called_with_new.set(false);
            self.callee.report_effects_when_called(options);
            options.called_with_new.set(old_value);
        } else {
            // TODO: Not work now
            options.ctx.diagnostic(NoSideEffectsDiagnostic::Call(self.callee.span()));
        }
    }
    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        if let Expression::Identifier(ident) = &self.callee {
            if let Some(node) = get_declaration_of_variable(ident, ctx) {
                let Some(parent) = ctx.nodes().parent_kind(node.id()) else {
                    return;
                };
                // TODO: `isLocalVariableAWhitelistedModule`
                if matches!(parent, AstKind::ImportDeclaration(_)) {
                    return;
                }
                options.ctx.diagnostic(NoSideEffectsDiagnostic::CallReturnValue(self.span));
            } else {
                options.ctx.diagnostic(NoSideEffectsDiagnostic::CallReturnValue(self.span));
            }
        }
    }
    fn report_effects_when_mutated(&self, options: &NodeListenerOptions) {
        options.ctx.diagnostic(NoSideEffectsDiagnostic::MutateFunctionReturnValue(self.span));
    }
}

impl<'a> ListenerMap for Argument<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::Expression(expr) => expr.report_effects(options),
            Self::SpreadElement(spread) => {
                spread.argument.report_effects(options);
            }
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

    fn report_effects_when_called(&self, options: &NodeListenerOptions) {
        let ctx = options.ctx;
        if let Some(symbol_id) = get_symbol_id_of_variable(self, ctx) {
            let symbol_table = ctx.semantic().symbols();
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    let node_id = reference.node_id();
                    if let Some(expr) = get_write_expr(node_id, ctx) {
                        expr.report_effects_when_called(options);
                    }
                }
            }
            let symbol_table = ctx.semantic().symbols();
            let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
            node.report_effects_when_called(options);
        } else {
            ctx.diagnostic(NoSideEffectsDiagnostic::CallGlobal(
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
                        let node_id = reference.node_id();
                        if let Some(expr) = get_write_expr(node_id, ctx) {
                            expr.report_effects_when_mutated(options);
                        }
                    }
                }

                let symbol_table = ctx.semantic().symbols();
                let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));
                node.report_effects_when_mutated(options);
            }
        } else {
            ctx.diagnostic(NoSideEffectsDiagnostic::MutateWithName(
                self.name.to_compact_str(),
                self.span,
            ));
        }
    }
}

impl<'a> ListenerMap for MemberExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        match self {
            Self::ComputedMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::StaticMemberExpression(expr) => {
                expr.report_effects(options);
            }
            Self::PrivateFieldExpression(expr) => {
                expr.report_effects(options);
            }
        }
    }
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

impl<'a> ListenerMap for ComputedMemberExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.expression.report_effects(options);
        self.object.report_effects(options);
    }
}

impl<'a> ListenerMap for StaticMemberExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.object.report_effects(options);
    }
}

impl<'a> ListenerMap for PrivateFieldExpression<'a> {
    fn report_effects(&self, options: &NodeListenerOptions) {
        self.object.report_effects(options);
    }
}

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
