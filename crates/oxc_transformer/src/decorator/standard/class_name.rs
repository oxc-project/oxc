use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, VisitMut, walk_mut};
use oxc_span::{GetSpan, SPAN};
use oxc_str::Ident;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};
use oxc_traverse::BoundIdentifier;

use crate::{
    common::helper_loader::{Helper, helper_call_expr, helper_load},
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

    /// Create the replacement binding even if the class body never references the name.
    #[must_use]
    pub fn with_replacement(mut self) -> Self {
        self.replacement();
        self
    }

    /// Rewrite references in the class heritage, which is evaluated in the class scope.
    ///
    /// If `only_in_functions`, references evaluated directly by the heritage are kept: they read
    /// the original inner binding, which is still in its TDZ, and so throw as they should. Only
    /// references in functions, which can run after the class decorators, are rewritten.
    pub fn rewrite_expression(mut self, expression: &mut Expression<'a>, only_in_functions: bool) {
        struct Functions<'r, 'a, 'ctx>(&'r mut ClassNameRewriter<'a, 'ctx>);
        impl<'a> VisitMut<'a> for Functions<'_, 'a, '_> {
            fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
                self.0.visit_function(function, flags);
            }
            fn visit_arrow_function_expression(&mut self, arrow: &mut ArrowFunctionExpression<'a>) {
                self.0.visit_arrow_function_expression(arrow);
            }
            fn visit_class(&mut self, class: &mut Class<'a>) {
                self.0.visit_class(class);
            }
        }
        if !only_in_functions {
            self.visit_expression(expression);
            return;
        }
        Functions(&mut self).visit_expression(expression);
    }

    /// Whether `expression` references the class binding `symbol_id`.
    pub fn references(
        expression: &Expression<'a>,
        symbol_id: SymbolId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        struct Finder<'s> {
            symbol_id: SymbolId,
            scoping: &'s oxc_semantic::Scoping,
            found: bool,
        }
        impl<'a> Visit<'a> for Finder<'_> {
            fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
                if self.scoping.get_reference(ident.reference_id()).symbol_id()
                    == Some(self.symbol_id)
                {
                    self.found = true;
                }
            }
        }
        let mut finder = Finder { symbol_id, scoping: ctx.scoping(), found: false };
        finder.visit_expression(expression);
        finder.found
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
        self.take_class_name_reference(ident)
    }

    fn take_class_name_reference(&mut self, ident: &IdentifierReference<'a>) -> bool {
        let reference_id = ident.reference_id();
        if self.ctx.scoping().get_reference(reference_id).symbol_id() != Some(self.symbol_id) {
            return false;
        }
        self.ctx.scoping_mut().delete_resolved_reference(self.symbol_id, reference_id);
        true
    }

    /// `_toSetter(_readOnlyError, ["K"])._` throws when the target is assigned, after its
    /// value and any default initializer have been evaluated. Keep the surrounding pattern
    /// or loop intact so evaluation order and iterator closing remain unchanged.
    fn read_only_target(&mut self, span: Span) -> SimpleAssignmentTarget<'a> {
        let ctx = &mut *self.ctx;
        let error = helper_load(Helper::ReadOnlyError, ctx);
        let name = Expression::new_string_literal(SPAN, self.name, None, ctx);
        let arguments =
            Expression::new_array_expression(SPAN, [ArrayExpressionElement::from(name)], ctx);
        let setter = helper_call_expr(
            Helper::ToSetter,
            ArenaVec::from_array_in([Argument::from(error), Argument::from(arguments)], ctx),
            ctx,
        );
        SimpleAssignmentTarget::new_static_member_expression(
            span,
            setter,
            IdentifierName::new(SPAN, "_", ctx),
            false,
            ctx,
        )
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
    /// Logical assignments retain their short circuit: `K &&= v` becomes
    /// `_K && (v, _readOnlyError("K"))`.
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        let replacement = match expr {
            Expression::AssignmentExpression(assign)
                if assign
                    .left
                    .as_simple_assignment_target()
                    .is_some_and(|target| self.take_class_name_target(target)) =>
            {
                self.visit_expression(&mut assign.right);
                let value = assign.right.take_in(self.ctx);
                if let Some(operator) = assign.operator.to_logical_operator() {
                    let read = self.read();
                    let error = self.read_only_error(value, SPAN);
                    Expression::new_logical_expression(assign.span, read, operator, error, self.ctx)
                } else {
                    let value = match assign.operator.to_binary_operator() {
                        Some(operator) => {
                            let read = self.read();
                            Expression::new_binary_expression(SPAN, read, operator, value, self.ctx)
                        }
                        None => value,
                    };
                    self.read_only_error(value, assign.span)
                }
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

    fn visit_simple_assignment_target(&mut self, target: &mut SimpleAssignmentTarget<'a>) {
        if self.take_class_name_target(target) {
            let span = target.span();
            *target = self.read_only_target(span);
        } else {
            walk_mut::walk_simple_assignment_target(self, target);
        }
    }

    fn visit_assignment_target_property(&mut self, property: &mut AssignmentTargetProperty<'a>) {
        if let AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(shorthand) = property
            && self.take_class_name_reference(&shorthand.binding)
        {
            let span = shorthand.span;
            let key = PropertyKey::new_static_identifier(
                shorthand.binding.span,
                shorthand.binding.name,
                self.ctx,
            );
            let target = AssignmentTarget::from(self.read_only_target(shorthand.binding.span));
            let target = if let Some(mut init) = shorthand.init.take() {
                self.visit_expression(&mut init);
                AssignmentTargetMaybeDefault::new_assignment_target_with_default(
                    span, target, init, self.ctx,
                )
            } else {
                AssignmentTargetMaybeDefault::from(target)
            };
            *property = AssignmentTargetProperty::new_assignment_target_property_property(
                span, key, target, self.ctx,
            );
        } else {
            walk_mut::walk_assignment_target_property(self, property);
        }
    }

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let symbol_id = self.ctx.scoping().get_reference(ident.reference_id()).symbol_id();
        if symbol_id != Some(self.symbol_id) {
            return;
        }
        let Some(replacement) = self.replacement() else { return };
        self.ctx.rebind_identifier_reference(ident, &replacement);
    }
}
