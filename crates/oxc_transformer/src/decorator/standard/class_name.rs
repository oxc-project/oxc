use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_span::SPAN;
use oxc_str::Ident;
use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::BoundIdentifier;

use crate::{
    common::helper_loader::{Helper, helper_call_expr},
    context::TraverseCtx,
};

/// Rewrite references to the class's own name inside class body to `replacement`.
///
/// Inside the class body, the class name refers to the inner (immutable) class binding,
/// which a class decorator may replace. Element decorators are moved outside the class body,
/// where the name would refer to a different binding.
///
/// The replacement is a mutable binding, so a write to the class name is replaced by a
/// `_readOnlyError` call, which throws like the write would have.
///
/// `replacement` is created lazily on first reference, if `create_in_scope` is `Some`.
pub struct ClassNameRewriter<'a, 'ctx> {
    symbol_id: SymbolId,
    name: Ident<'a>,
    replacement: Option<BoundIdentifier<'a>>,
    create_in_scope: Option<ScopeId>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> ClassNameRewriter<'a, 'ctx> {
    pub fn new(
        symbol_id: SymbolId,
        name: Ident<'a>,
        replacement: Option<BoundIdentifier<'a>>,
        create_in_scope: Option<ScopeId>,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Self {
        Self { symbol_id, name, replacement, create_in_scope, ctx }
    }

    pub fn rewrite(mut self, body: &mut ClassBody<'a>) -> Option<BoundIdentifier<'a>> {
        self.visit_class_body(body);
        self.replacement
    }

    fn replacement(&mut self) -> Option<BoundIdentifier<'a>> {
        if self.replacement.is_none() {
            let scope_id = self.create_in_scope?;
            let binding =
                self.ctx.generate_uid(&self.name, scope_id, SymbolFlags::BlockScopedVariable);
            self.replacement = Some(binding);
        }
        self.replacement.clone()
    }

    /// Whether `target` is the class name. If so, its reference is deleted.
    fn take_class_name_target(&mut self, target: &SimpleAssignmentTarget<'a>) -> bool {
        let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target else {
            return false;
        };
        let reference_id = ident.reference_id();
        if self.ctx.scoping().get_reference(reference_id).symbol_id() != Some(self.symbol_id) {
            return false;
        }
        self.ctx.scoping_mut().delete_resolved_reference(self.symbol_id, reference_id);
        true
    }

    /// Read of the class name, for the read half of `K += v` or `K++`.
    fn read(&mut self) -> Expression<'a> {
        match self.replacement() {
            Some(replacement) => replacement.create_read_expression(self.ctx),
            // Not reachable: `new` is given either a replacement or a scope to create one in.
            None => Expression::new_identifier(SPAN, self.name, self.ctx),
        }
    }

    /// `(expression, _readOnlyError("K"))`
    fn read_only_error(&mut self, expression: Expression<'a>, span: Span) -> Expression<'a> {
        let ctx = &mut *self.ctx;
        let message = Expression::new_string_literal(SPAN, self.name, None, ctx);
        let error = helper_call_expr(
            Helper::ReadOnlyError,
            ArenaVec::from_value_in(Argument::from(message), ctx),
            ctx,
        );
        Expression::new_sequence_expression(
            span,
            ArenaVec::from_array_in([expression, error], ctx),
            ctx,
        )
    }
}

impl<'a> VisitMut<'a> for ClassNameRewriter<'a, '_> {
    /// `K = v` becomes `(v, _readOnlyError("K"))`, `K += v` becomes `(_K + v, _readOnlyError("K"))`,
    /// and `K++` becomes `(+_K, _readOnlyError("K"))`.
    ///
    /// Writes in destructuring patterns and `for-in`/`for-of` heads are not handled, and still
    /// assign to the replacement.
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        let replacement = match expr {
            Expression::AssignmentExpression(assign)
                if !assign.operator.is_logical()
                    && assign
                        .left
                        .as_simple_assignment_target()
                        .is_some_and(|target| self.take_class_name_target(target)) =>
            {
                self.visit_expression(&mut assign.right);
                let value = assign.right.take_in(self.ctx);
                let value = match assign.operator.to_binary_operator() {
                    Some(operator) => {
                        let read = self.read();
                        Expression::new_binary_expression(SPAN, read, operator, value, self.ctx)
                    }
                    None => value,
                };
                self.read_only_error(value, assign.span)
            }
            Expression::UpdateExpression(update)
                if self.take_class_name_target(&update.argument) =>
            {
                let read = self.read();
                let read = Expression::new_unary_expression(
                    SPAN,
                    UnaryOperator::UnaryPlus,
                    read,
                    self.ctx,
                );
                self.read_only_error(read, update.span)
            }
            _ => return walk_mut::walk_expression(self, expr),
        };
        *expr = replacement;
    }

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let symbol_id = self.ctx.scoping().get_reference(ident.reference_id()).symbol_id();
        if symbol_id != Some(self.symbol_id) {
            return;
        }
        let Some(replacement) = self.replacement() else { return };
        let reference_id = ident.reference_id();
        let flags = self.ctx.scoping().get_reference(reference_id).flags();
        self.ctx.scoping_mut().delete_resolved_reference(self.symbol_id, reference_id);
        *ident = replacement.create_reference(flags, self.ctx);
    }
}
