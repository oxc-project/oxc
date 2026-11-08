use std::{cell::Cell, mem};

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, VisitMut, walk_mut};
use oxc_ecmascript::BoundNames;
use oxc_semantic::{NodeId, ScopeFlags, ScopeId, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_str::{Ident, static_ident};
use oxc_traverse::{BoundIdentifier, MaybeBoundIdentifier};

use super::ObjectRestSpread;
use crate::context::TraverseCtx;

impl<'a> ObjectRestSpread<'a> {
    pub(super) fn lower_parameter_dependencies(
        params: &mut FormalParameters<'a>,
        body: &mut FunctionBody<'a>,
        scope_id: ScopeId,
        r#async: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        if !params.items.iter().any(|param| Self::has_nested_object_rest(&param.pattern)) {
            return false;
        }
        let mut moved_symbols = FxHashSet::default();
        for param in &params.items {
            if Self::has_nested_object_rest(&param.pattern) {
                param.pattern.bound_names(&mut |ident| {
                    moved_symbols.insert(ident.symbol_id());
                });
            }
        }
        let mut references = ParameterReferences { symbols: &moved_symbols, ctx, found: false };
        references.visit_formal_parameters(params);
        if !references.found {
            return false;
        }

        let mut let_parameter = None;
        params.bound_names(&mut |ident| {
            if ident.name == "let" {
                let_parameter = Some(ident.symbol_id());
            }
        });
        if let_parameter.is_some() {
            if ctx.scoping().scope_flags(scope_id).contains_direct_eval() {
                return false;
            }
            let mut with_statement = WithStatementFinder::default();
            with_statement.visit_formal_parameters(params);
            with_statement.visit_function_body(body);
            if with_statement.found {
                return false;
            }
        }

        let mut shadow = BodyShadow { scope_id, ctx, found: false };
        for param in &params.items {
            shadow.visit_formal_parameter(param);
        }
        if let Some(rest) = &params.rest {
            shadow.visit_formal_parameter_rest(rest);
        }
        let mut body_bindings = BodyBindings { scope_id, ctx, symbols: FxHashSet::default() };
        body_bindings.visit_statements(&body.statements);
        let mut copied_parameters = Vec::new();
        params.bound_names(&mut |ident| {
            if body_bindings.symbols.contains(&ident.symbol_id()) {
                copied_parameters.push(BodyParameter {
                    source: MaybeBoundIdentifier::new(ident.name, Some(ident.symbol_id())),
                    binding: BoundIdentifier::from_binding_ident(ident),
                    flags: ctx.scoping().symbol_flags(ident.symbol_id()),
                });
            }
        });
        if params.kind != FormalParameterKind::ArrowFormalParameters
            && let Some(symbol_id) = ctx.scoping().get_binding(scope_id, static_ident!("arguments"))
            && body_bindings.symbols.contains(&symbol_id)
            && !copied_parameters.iter().any(|param| param.binding.symbol_id == symbol_id)
        {
            copied_parameters.push(BodyParameter {
                source: MaybeBoundIdentifier::new(static_ident!("arguments"), None),
                binding: BoundIdentifier::new(static_ident!("arguments"), symbol_id),
                flags: ctx.scoping().symbol_flags(symbol_id),
            });
        }
        let needs_body_scope = shadow.found
            || !copied_parameters.is_empty()
            || ctx.scoping().scope_flags(scope_id).contains_direct_eval();
        if r#async && needs_body_scope {
            return false;
        }
        let mut prefix = ArenaVec::new_in(ctx);
        for param in &mut params.items {
            let temp = ctx.generate_uid("ref", scope_id, SymbolFlags::FunctionScopedVariable);
            let pattern = mem::replace(&mut param.pattern, temp.create_binding_pattern(ctx));
            let value = temp.create_read_expression(ctx);
            let init = if let Some(default) = param.initializer.take() {
                let mut default = default.unbox();
                if let BindingPattern::BindingIdentifier(ident) = &pattern {
                    default = preserve_parameter_name(default, ident.name, ctx);
                }
                param.initializer = Some(ctx.alloc(Expression::new_void_0(SPAN, ctx)));
                let test = Expression::new_binary_expression(
                    SPAN,
                    temp.create_read_expression(ctx),
                    BinaryOperator::StrictEquality,
                    Expression::new_void_0(SPAN, ctx),
                    ctx,
                );
                Expression::new_conditional_expression(SPAN, test, default, value, ctx)
            } else {
                value
            };
            prefix.push(Self::parameter_declaration(pattern, init, ctx));
        }
        if let Some(rest) = &mut params.rest {
            let temp = ctx.generate_uid("ref", scope_id, SymbolFlags::FunctionScopedVariable);
            let pattern = mem::replace(&mut rest.rest.argument, temp.create_binding_pattern(ctx));
            prefix.push(Self::parameter_declaration(
                pattern,
                temp.create_read_expression(ctx),
                ctx,
            ));
        }
        if params.rest.is_none() && params.items.iter().all(|param| param.initializer.is_none()) {
            let unused = ctx.generate_uid("unused", scope_id, SymbolFlags::FunctionScopedVariable);
            let rest = BindingRestElement::new(SPAN, unused.create_binding_pattern(ctx), ctx);
            params.rest = Some(FormalParameterRest::boxed(SPAN, [], rest, None, ctx));
        }
        if needs_body_scope {
            let statements = body.statements.take_in(ctx);
            let body_scope =
                Self::parameter_body_scope(statements, scope_id, &copied_parameters, ctx);
            prefix.push(body_scope);
            body.statements = prefix;
        } else {
            body.statements.splice(0..0, prefix);
        }
        if let Some(symbol_id) = let_parameter {
            let name = ctx.generate_uid_name("let");
            let scope_id = ctx.scoping().symbol_scope_id(symbol_id);
            ctx.scoping_mut().rename_symbol(symbol_id, scope_id, name);
            let mut rename = RenameParameter { symbol_id, name, ctx };
            rename.visit_formal_parameters(params);
            rename.visit_function_body(body);
        }
        true
    }

    fn parameter_declaration(
        pattern: BindingPattern<'a>,
        init: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        pattern.bound_names(&mut |ident| {
            *ctx.scoping_mut().symbol_flags_mut(ident.symbol_id()) =
                SymbolFlags::BlockScopedVariable;
        });
        let decl = VariableDeclarator::new(SPAN, pattern, None, Some(init), false, ctx);
        Statement::VariableDeclaration(VariableDeclaration::boxed(
            SPAN,
            VariableDeclarationKind::Let,
            [decl],
            false,
            ctx,
        ))
    }

    fn parameter_body_scope(
        mut statements: ArenaVec<'a, Statement<'a>>,
        parent_scope_id: ScopeId,
        copied_parameters: &[BodyParameter<'a>],
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let mut flags = ScopeFlags::Function | ScopeFlags::Arrow;
        flags.set(
            ScopeFlags::StrictMode,
            ctx.scoping().scope_flags(parent_scope_id).is_strict_mode(),
        );
        let mut eval = DirectEval::default();
        eval.visit_statements(&statements);
        flags.set(ScopeFlags::DirectEval, eval.found);
        let scope_id = ctx.create_child_scope(parent_scope_id, flags);
        let mut copies = FxHashMap::default();
        let mut items = ArenaVec::new_in(ctx);
        let mut arguments = ArenaVec::new_in(ctx);
        for param in copied_parameters {
            let outer = &param.source;
            let body_symbol_id = param.binding.symbol_id;
            let flags = param.flags;
            let inner = ctx.generate_binding(param.binding.name, scope_id, flags);
            ctx.scoping_mut().clear_symbol_redeclarations(body_symbol_id);
            if outer.symbol_id != Some(body_symbol_id) {
                ctx.scoping_mut().remove_binding(parent_scope_id, param.binding.name);
            }
            arguments.push(Argument::from(outer.create_read_expression(ctx)));
            items.push(FormalParameter::new(
                SPAN,
                [],
                inner.create_binding_pattern(ctx),
                None,
                None,
                false,
                None,
                false,
                false,
                ctx,
            ));
            copies.insert(body_symbol_id, inner.symbol_id);
        }
        BodyScopeMover { parent_scope_id, scope_id, copies: &copies, ctx }
            .visit_statements(&mut statements);
        let params = FormalParameters::boxed(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            items,
            None,
            ctx,
        );
        let body = FunctionBody::boxed(SPAN, [], statements, ctx);
        let callee = Expression::new_arrow_function_expression_with_scope_id_and_pure_and_pife(
            SPAN,
            false,
            None,
            params,
            None,
            ArrowFunctionBody::FunctionBody(body),
            scope_id,
            false,
            false,
            ctx,
        );
        let call = Expression::new_call_expression(SPAN, callee, None, arguments, false, ctx);
        Statement::new_return_statement(SPAN, Some(call), ctx)
    }
}

struct ParameterReferences<'a, 'ctx> {
    symbols: &'ctx FxHashSet<SymbolId>,
    ctx: &'ctx TraverseCtx<'a>,
    found: bool,
}

impl<'a> Visit<'a> for ParameterReferences<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.found |= self
            .ctx
            .scoping()
            .get_reference(ident.reference_id())
            .symbol_id()
            .is_some_and(|id| self.symbols.contains(&id));
    }
    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
    fn visit_ts_type(&mut self, _ty: &TSType<'a>) {}
}

struct BodyShadow<'a, 'ctx> {
    scope_id: ScopeId,
    ctx: &'ctx TraverseCtx<'a>,
    found: bool,
}

impl<'a> Visit<'a> for BodyShadow<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let scoping = self.ctx.scoping();
        if let Some(body_symbol) = scoping.get_binding(self.scope_id, ident.name) {
            let symbol = scoping.get_reference(ident.reference_id()).symbol_id();
            self.found |= symbol != Some(body_symbol)
                && !symbol.is_some_and(|id| {
                    let scope = scoping.symbol_scope_id(id);
                    scope == self.scope_id || scoping.scope_is_descendant_of(scope, self.scope_id)
                });
        }
    }
    fn visit_ts_type(&mut self, _ty: &TSType<'a>) {}
}

struct BodyBindings<'a, 'ctx> {
    scope_id: ScopeId,
    ctx: &'ctx TraverseCtx<'a>,
    symbols: FxHashSet<SymbolId>,
}

impl<'a> Visit<'a> for BodyBindings<'a, '_> {
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if self.ctx.scoping().symbol_scope_id(ident.symbol_id()) == self.scope_id {
            self.symbols.insert(ident.symbol_id());
        }
    }
}

struct BodyScopeMover<'a, 'ctx> {
    parent_scope_id: ScopeId,
    scope_id: ScopeId,
    copies: &'ctx FxHashMap<SymbolId, SymbolId>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a> VisitMut<'a> for BodyScopeMover<'a, '_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope: &Cell<Option<ScopeId>>) {
        let scope = scope.get().unwrap();
        if self.ctx.scoping().scope_parent_id(scope) == Some(self.parent_scope_id) {
            self.ctx.scoping_mut().change_scope_parent_id(scope, Some(self.scope_id));
        }
    }

    fn visit_binding_identifier(&mut self, ident: &mut BindingIdentifier<'a>) {
        let id = ident.symbol_id();
        if let Some(&new_id) = self.copies.get(&id) {
            ident.symbol_id.set(Some(new_id));
            self.ctx.scoping_mut().add_symbol_redeclaration(
                new_id,
                SymbolFlags::FunctionScopedVariable,
                NodeId::DUMMY,
                ident.span,
            );
        } else if self.ctx.scoping().symbol_scope_id(id) == self.parent_scope_id {
            self.ctx.scoping_mut().move_binding_by_symbol_id(
                self.parent_scope_id,
                self.scope_id,
                id,
            );
        }
    }
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let reference_id = ident.reference_id();
        if let Some(old_id) = self.ctx.scoping().get_reference(reference_id).symbol_id()
            && let Some(&new_id) = self.copies.get(&old_id)
        {
            let scoping = self.ctx.scoping_mut();
            scoping.delete_resolved_reference(old_id, reference_id);
            scoping.add_resolved_reference(new_id, reference_id);
            scoping.get_reference_mut(reference_id).set_symbol_id(new_id);
        }
    }
}

#[derive(Default)]
struct DirectEval {
    found: bool,
}
impl<'a> Visit<'a> for DirectEval {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.found |= !call.optional && call.callee.is_specific_id("eval");
        oxc_ast_visit::walk::walk_call_expression(self, call);
    }
}

struct BodyParameter<'a> {
    source: MaybeBoundIdentifier<'a>,
    binding: BoundIdentifier<'a>,
    flags: SymbolFlags,
}

fn preserve_parameter_name<'a>(
    value: Expression<'a>,
    name: Ident<'a>,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    if !value.get_inner_expression().is_anonymous_function_definition() {
        return value;
    }
    let key = Expression::new_string_literal(SPAN, name, None, ctx);
    let property = ObjectPropertyKind::new_object_property(
        SPAN,
        PropertyKind::Init,
        PropertyKey::from(key),
        value,
        false,
        false,
        true,
        ctx,
    );
    let object = Expression::new_object_expression(SPAN, [property], ctx);
    Expression::new_static_member_expression(
        SPAN,
        object,
        IdentifierName::new(SPAN, name, ctx),
        false,
        ctx,
    )
}

struct RenameParameter<'a, 'ctx> {
    symbol_id: SymbolId,
    name: Ident<'a>,
    ctx: &'ctx TraverseCtx<'a>,
}

impl<'a> VisitMut<'a> for RenameParameter<'a, '_> {
    fn visit_assignment_pattern(&mut self, pattern: &mut AssignmentPattern<'a>) {
        if let BindingPattern::BindingIdentifier(ident) = &pattern.left
            && ident.symbol_id() == self.symbol_id
        {
            pattern.right =
                preserve_parameter_name(pattern.right.take_in(self.ctx), ident.name, self.ctx);
        }
        walk_mut::walk_assignment_pattern(self, pattern);
    }

    fn visit_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        if (expr.operator.is_assign() || expr.operator.is_logical())
            && let AssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.left
            && self.ctx.scoping().get_reference(ident.reference_id()).symbol_id()
                == Some(self.symbol_id)
        {
            expr.right =
                preserve_parameter_name(expr.right.take_in(self.ctx), ident.name, self.ctx);
        }
        walk_mut::walk_assignment_expression(self, expr);
    }

    fn visit_assignment_target_with_default(
        &mut self,
        target: &mut AssignmentTargetWithDefault<'a>,
    ) {
        if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &target.binding
            && self.ctx.scoping().get_reference(ident.reference_id()).symbol_id()
                == Some(self.symbol_id)
        {
            target.init =
                preserve_parameter_name(target.init.take_in(self.ctx), ident.name, self.ctx);
        }
        walk_mut::walk_assignment_target_with_default(self, target);
    }

    fn visit_assignment_target_property(&mut self, property: &mut AssignmentTargetProperty<'a>) {
        if let AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) = property
            && self.ctx.scoping().get_reference(ident.binding.reference_id()).symbol_id()
                == Some(self.symbol_id)
        {
            let name = ident.binding.name;
            let key =
                PropertyKey::StaticIdentifier(IdentifierName::boxed(ident.span, name, self.ctx));
            let binding = AssignmentTarget::AssignmentTargetIdentifier(
                self.ctx.alloc(ident.binding.take_in(self.ctx)),
            );
            let binding = if let Some(init) = ident.init.take() {
                AssignmentTargetMaybeDefault::new_assignment_target_with_default(
                    ident.span, binding, init, self.ctx,
                )
            } else {
                AssignmentTargetMaybeDefault::from(binding)
            };
            *property = AssignmentTargetProperty::new_assignment_target_property_property(
                ident.span, key, binding, false, self.ctx,
            );
        }
        walk_mut::walk_assignment_target_property(self, property);
    }

    fn visit_binding_identifier(&mut self, ident: &mut BindingIdentifier<'a>) {
        if ident.symbol_id() == self.symbol_id {
            ident.name = self.name;
        }
    }

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        if self.ctx.scoping().get_reference(ident.reference_id()).symbol_id()
            == Some(self.symbol_id)
        {
            ident.name = self.name;
        }
    }
}

#[derive(Default)]
struct WithStatementFinder {
    found: bool,
}

impl<'a> Visit<'a> for WithStatementFinder {
    fn visit_with_statement(&mut self, _stmt: &WithStatement<'a>) {
        self.found = true;
    }
}
