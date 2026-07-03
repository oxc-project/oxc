use std::cell::Cell;

use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::{ast::*, builder::NONE};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::{ScopeFlags, ScopeId};
use oxc_span::{SPAN, Span};
use oxc_str::{Ident, static_ident};
use oxc_syntax::{
    constant_value::ConstantValue,
    number::NumberBase,
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
    reference::{ReferenceFlags, ReferenceId},
    symbol::SymbolFlags,
};
use oxc_traverse::{BoundIdentifier, Traverse};

use crate::{context::TraverseCtx, state::TransformState};

pub struct TypeScriptEnum {
    optimize_const_enums: bool,
    optimize_enums: bool,
}

impl TypeScriptEnum {
    pub fn new(optimize_const_enums: bool, optimize_enums: bool) -> Self {
        Self { optimize_const_enums, optimize_enums }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for TypeScriptEnum {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::TSEnumDeclaration(decl) => {
                // Defer removable enums — they'll be handled in exit_statements
                // after enter_expression has inlined member accesses and deleted references.
                if self.may_remove_enum(decl, ctx) {
                    return;
                }
                if let Some(new_stmt) = Self::transform_ts_enum(decl, None, ctx) {
                    *stmt = new_stmt;
                }
            }
            Statement::ExportNamedDeclaration(export_decl) => {
                let span = export_decl.span;
                if let Some(Declaration::TSEnumDeclaration(decl)) = &mut export_decl.declaration
                    && let Some(new_stmt) = Self::transform_ts_enum(decl, Some(span), ctx)
                {
                    *stmt = new_stmt;
                }
            }
            _ => {}
        }
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.optimize_const_enums && !self.optimize_enums {
            return;
        }

        let parent_scope_id = ctx.current_scope_id();
        let mut has_removable_enum = false;

        // Transform or remove deferred enum declarations.
        for stmt in stmts.iter_mut() {
            let Statement::TSEnumDeclaration(decl) = stmt else { continue };
            if self.can_remove_enum(decl, ctx) {
                has_removable_enum = true;
                continue;
            }
            // Not removable after all (still has value references) — transform now.
            if let Some(new_stmt) = Self::transform_ts_enum(decl, None, ctx) {
                *stmt = new_stmt;
            }
        }

        if !has_removable_enum {
            return;
        }

        let mut names_to_remove = Vec::new();
        stmts.retain(|stmt| {
            if let Statement::TSEnumDeclaration(decl) = stmt {
                names_to_remove.push(decl.id.name);
                return false;
            }
            true
        });

        for name in names_to_remove {
            ctx.scoping_mut().remove_binding(parent_scope_id, name);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !self.optimize_const_enums && !self.optimize_enums {
            return;
        }

        // Peek through TS-only wrappers and parens so `E.X as T`, `E.X satisfies T`,
        // `E.X!`, `<T>E.X`, `E.X` (with `preserveParens`) all inline. `annotations.rs`
        // strips these wrappers, but only after this hook returns — by then the outer
        // node has been replaced and `enter_expression` is not re-invoked on it.
        let inlined = match expr.get_inner_expression_mut() {
            Expression::StaticMemberExpression(member_expr) => {
                self.try_inline_enum_member(member_expr, ctx)
            }
            Expression::ComputedMemberExpression(member_expr) => {
                self.try_inline_computed_enum_member(member_expr, ctx)
            }
            // `c_num?.x` / `c_num?.['x']`: inline before es2020 lowers the chain.
            // Otherwise lowering produces `c_num === null || c_num === void 0 ? void 0 : c_num.x`,
            // and only the inner `c_num.x` reference gets deleted by the inline pass — leaving
            // the test-condition references dangling once the enum declaration is removed.
            Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
                ChainElement::StaticMemberExpression(member_expr) if member_expr.optional => {
                    self.try_inline_enum_member(member_expr, ctx)
                }
                ChainElement::ComputedMemberExpression(member_expr) if member_expr.optional => {
                    self.try_inline_computed_enum_member(member_expr, ctx)
                }
                _ => return,
            },
            _ => return,
        };

        if let Some((value, ref_id)) = inlined {
            ctx.scoping_mut().delete_reference(ref_id);
            *expr = match value {
                ConstantValue::Number(n) => Self::get_initializer_expr(n, ctx),
                ConstantValue::String(s) => {
                    Expression::new_string_literal(SPAN, Str::from_str_in(&s, ctx), None, ctx)
                }
            };
        }
    }
}

impl<'a> TypeScriptEnum {
    /// ```TypeScript
    /// enum Foo {
    ///   X = 1,
    ///   Y
    /// }
    /// ```
    /// ```JavaScript
    /// var Foo = ((Foo) => {
    ///   Foo[Foo["X"] = 1] = "X";
    ///   Foo[Foo["Y"] = 2] = "Y";
    ///   return Foo;
    /// })(Foo || {});
    /// ```
    fn transform_ts_enum(
        decl: &mut TSEnumDeclaration<'a>,
        export_span: Option<Span>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if decl.declare {
            return None;
        }

        // Enum lowering relies on pre-computed member values stored in `Scoping`
        // by `evaluate_enum_members` (only run when the semantic builder is configured
        // with `enum_eval`). Without it, string-valued members are not recognized and
        // the transform emits incorrect reverse mappings (see oxc#21667).
        debug_assert!(
            ctx.scoping().get_enum_body_scopes(decl.id.symbol_id()).is_some(),
            "Transformer requires `Scoping` produced with `SemanticBuilder::with_enum_eval(true)` \
             to correctly transform `enum {}`.",
            decl.id.name,
        );

        let is_export = export_span.is_some();
        let is_not_top_scope = !ctx.scoping().scope_flags(ctx.current_scope_id()).is_top();

        let enum_name: Ident = decl.id.name;
        let func_scope_id = decl.body.scope_id();
        let param_binding =
            ctx.generate_binding(enum_name, func_scope_id, SymbolFlags::FunctionScopedVariable);

        let id = param_binding.create_binding_pattern(ctx);

        // ((Foo) => {
        let params = FormalParameter::new(
            SPAN,
            ArenaVec::new_in(ctx),
            id,
            NONE,
            NONE,
            false,
            None,
            false,
            false,
            ctx,
        );
        let params = ArenaVec::from_value_in(params, ctx);
        let params = FormalParameters::boxed(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            NONE,
            ctx,
        );

        let has_potential_side_effect = decl.body.members.iter().any(|member| {
            matches!(
                member.initializer,
                Some(Expression::NewExpression(_) | Expression::CallExpression(_))
            )
        });

        let statements = Self::transform_ts_enum_members(
            func_scope_id,
            &mut decl.body.members,
            &param_binding,
            ctx,
        );
        let scope_flags =
            ctx.scoping().get_new_scope_flags(ScopeFlags::Function, ctx.current_scope_id());
        *ctx.scoping_mut().scope_flags_mut(func_scope_id) = scope_flags;
        ctx.scoping_mut().retain_scope_binding(func_scope_id, param_binding.symbol_id);

        let span = decl.span;
        let body = FunctionBody::boxed(span, ArenaVec::new_in(ctx), statements, ctx);
        let callee = Expression::new_function_expression_with_scope_id_and_pure_and_pife(
            span,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
            func_scope_id,
            false,
            false,
            ctx,
        );

        let enum_symbol_id = decl.id.symbol_id();

        // Foo[Foo["X"] = 0] = "X";
        let redeclarations = ctx.scoping().symbol_redeclarations(enum_symbol_id);
        let is_already_declared =
            redeclarations.first().map_or_else(|| false, |rd| rd.span != decl.id.span);
        let is_last_redeclaration = redeclarations.last().is_some_and(|rd| rd.span == decl.id.span);

        let arguments = if (is_export || is_not_top_scope) && !is_already_declared {
            // }({});
            let object_arg = Argument::new_object_expression(SPAN, ArenaVec::new_in(ctx), ctx);
            ArenaVec::from_value_in(object_arg, ctx)
        } else {
            // }(Foo || {});
            let op = LogicalOperator::Or;
            let left = ctx.create_bound_ident_expr(
                decl.id.span,
                enum_name,
                enum_symbol_id,
                ReferenceFlags::Read,
            );
            let right = Expression::new_object_expression(SPAN, ArenaVec::new_in(ctx), ctx);
            let argument = Argument::new_logical_expression(span, left, op, right, ctx);
            ArenaVec::from_value_in(argument, ctx)
        };

        let call_expression = Expression::new_call_expression_with_pure(
            span,
            callee,
            NONE,
            arguments,
            false,
            !has_potential_side_effect,
            ctx,
        );

        if is_already_declared {
            let op = AssignmentOperator::Assign;
            let left = ctx.create_bound_ident_reference(
                decl.id.span,
                enum_name,
                enum_symbol_id,
                ReferenceFlags::Write,
            );
            let left = AssignmentTarget::AssignmentTargetIdentifier(ctx.alloc(left));
            let expr = Expression::new_assignment_expression(span, op, left, call_expression, ctx);
            if is_last_redeclaration {
                ctx.scoping_mut().clear_symbol_redeclarations(enum_symbol_id);
            }
            return Some(Statement::new_expression_statement(span, expr, ctx));
        }

        let kind = if is_export || is_not_top_scope {
            VariableDeclarationKind::Let
        } else {
            VariableDeclarationKind::Var
        };
        *ctx.scoping_mut().symbol_flags_mut(enum_symbol_id) = match kind {
            VariableDeclarationKind::Var => SymbolFlags::FunctionScopedVariable,
            VariableDeclarationKind::Let => SymbolFlags::BlockScopedVariable,
            VariableDeclarationKind::Const => SymbolFlags::ConstVariable,
            VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => {
                SymbolFlags::BlockScopedVariable
            }
        };
        let decls = {
            let binding = BindingPattern::new_binding_identifier_with_symbol_id(
                decl.id.span,
                decl.id.name,
                enum_symbol_id,
                ctx,
            );
            let decl = VariableDeclarator::new(
                span,
                kind,
                binding,
                NONE,
                Some(call_expression),
                false,
                ctx,
            );
            ArenaVec::from_value_in(decl, ctx)
        };
        let variable_declaration =
            Declaration::new_variable_declaration(span, kind, decls, false, ctx);

        let stmt = if let Some(export_span) = export_span {
            let declaration = ExportNamedDeclaration::boxed_plain_declaration(
                export_span,
                variable_declaration,
                ctx,
            );
            Statement::ExportNamedDeclaration(declaration)
        } else {
            Statement::from(variable_declaration)
        };
        Some(stmt)
    }

    fn transform_ts_enum_members(
        enum_scope_id: ScopeId,
        members: &mut ArenaVec<'a, TSEnumMember<'a>>,
        param_binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        // Each member pushes exactly one statement, plus a final `return` statement,
        // so the length is known up front — pre-size to avoid growth reallocations.
        let mut statements = ArenaVec::with_capacity_in(members.len() + 1, ctx);

        // If enum number has no initializer, its value will be the previous member value + 1,
        // if it's the first member, it will be `0`.
        // It used to keep track of the previous constant number.
        let mut prev_constant_number = Some(-1.0);

        let mut prev_member_name = None;

        for member in members.take_in(ctx) {
            let member_span = member.span;
            let member_name = member.id.static_name();

            let init = if let Some(mut initializer) = member.initializer {
                // Look up the pre-computed constant value from Scoping
                let constant_value: Option<ConstantValue> = ctx
                    .scoping()
                    .get_binding(enum_scope_id, member_name.as_str().into())
                    .and_then(|sym_id| ctx.scoping().get_enum_member_value(sym_id))
                    .cloned();

                match constant_value {
                    None => {
                        prev_constant_number = None;

                        IdentifierReferenceRename::new(param_binding, enum_scope_id, ctx)
                            .visit_expression(&mut initializer);

                        initializer
                    }
                    Some(constant_value) => match constant_value {
                        ConstantValue::Number(v) => {
                            prev_constant_number = Some(v);
                            Self::get_initializer_expr(v, ctx)
                        }
                        ConstantValue::String(s) => {
                            prev_constant_number = None;
                            Expression::new_string_literal(
                                SPAN,
                                Str::from_str_in(&s, ctx),
                                None,
                                ctx,
                            )
                        }
                    },
                }
                // No initializer, try to infer the value from the previous member.
            } else if let Some(value) = &prev_constant_number {
                let value = value + 1.0;
                prev_constant_number = Some(value);
                Self::get_initializer_expr(value, ctx)
            } else if let Some(prev_member_name) = prev_member_name {
                let self_ref = {
                    let obj = param_binding.create_read_expression(ctx);
                    let expr = Expression::new_string_literal(SPAN, prev_member_name, None, ctx);
                    Expression::new_computed_member_expression(SPAN, obj, expr, false, ctx)
                };

                // 1 + Foo["x"]
                let one = Self::get_number_literal_expression(1.0, ctx);
                Expression::new_binary_expression(
                    SPAN,
                    one,
                    BinaryOperator::Addition,
                    self_ref,
                    ctx,
                )
            } else {
                Self::get_number_literal_expression(0.0, ctx)
            };

            let is_str = init.is_string_literal();

            // Foo["x"] = init
            let member_expr = {
                let obj = param_binding.create_read_expression(ctx);
                let expr = Expression::new_string_literal(SPAN, member_name, None, ctx);

                MemberExpression::new_computed_member_expression(SPAN, obj, expr, false, ctx)
            };
            let left = SimpleAssignmentTarget::from(member_expr);
            let mut expr = Expression::new_assignment_expression(
                member_span,
                AssignmentOperator::Assign,
                left.into(),
                init,
                ctx,
            );

            // Foo[Foo["x"] = init] = "x"
            if !is_str {
                let member_expr = {
                    let obj = param_binding.create_read_expression(ctx);
                    MemberExpression::new_computed_member_expression(SPAN, obj, expr, false, ctx)
                };
                let left = SimpleAssignmentTarget::from(member_expr);
                let right = Expression::new_string_literal(SPAN, member_name, None, ctx);
                expr = Expression::new_assignment_expression(
                    member_span,
                    AssignmentOperator::Assign,
                    left.into(),
                    right,
                    ctx,
                );
            }

            prev_member_name = Some(member_name);
            statements.push(Statement::new_expression_statement(member_span, expr, ctx));
        }

        let enum_ref = param_binding.create_read_expression(ctx);
        // return Foo;
        let return_stmt = Statement::new_return_statement(SPAN, Some(enum_ref), ctx);
        statements.push(return_stmt);

        statements
    }

    /// Check if an enum declaration might be removable (pre-inlining).
    /// Used by `enter_statement` to defer transformation of potential removal candidates.
    fn may_remove_enum(&self, decl: &TSEnumDeclaration<'a>, ctx: &TraverseCtx<'a>) -> bool {
        if decl.declare {
            return false;
        }
        if decl.r#const {
            if !self.optimize_const_enums {
                return false;
            }
        } else if !self.optimize_enums {
            return false;
        }
        Self::all_members_evaluable(decl, ctx)
    }

    /// Check if an enum declaration can be safely removed (post-inlining).
    ///
    /// The decl is removable when no value references (`Read`/`Write`) remain —
    /// `enter_expression` inlines member accesses and deletes those references. Type
    /// references (e.g. from `as E` / `: E`) are kept here and stripped later by
    /// `annotations.rs`, so they don't block removal.
    ///
    /// If a non-inlinable value reference remains (e.g. `export default E`, `E.toString()`),
    /// we emit the IIFE form so the binding still exists at runtime — matching tsc under
    /// `--isolatedModules`. Babel drops the declaration in this case, leaving dangling
    /// references; oxc diverges intentionally to preserve runtime correctness.
    fn can_remove_enum(&self, decl: &TSEnumDeclaration<'a>, ctx: &TraverseCtx<'a>) -> bool {
        if !self.may_remove_enum(decl, ctx) {
            return false;
        }
        ctx.scoping().get_resolved_references(decl.id.symbol_id()).all(|r| !r.is_value())
    }

    /// Check if all members of an enum declaration have known constant values.
    fn all_members_evaluable(decl: &TSEnumDeclaration<'a>, ctx: &TraverseCtx<'a>) -> bool {
        let scope_id = decl.body.scope_id();
        decl.body.members.iter().all(|member| match &member.id {
            TSEnumMemberName::Identifier(ident) => ctx
                .scoping()
                .get_binding(scope_id, ident.name.as_str().into())
                .and_then(|sym_id| ctx.scoping().get_enum_member_value(sym_id))
                .is_some(),
            TSEnumMemberName::String(lit) | TSEnumMemberName::ComputedString(lit) => ctx
                .scoping()
                .get_binding(scope_id, lit.value.as_str().into())
                .and_then(|sym_id| ctx.scoping().get_enum_member_value(sym_id))
                .is_some(),
            TSEnumMemberName::ComputedTemplateString(_) => false,
        })
    }

    fn get_number_literal_expression(value: f64, ctx: &TraverseCtx<'a>) -> Expression<'a> {
        Expression::new_numeric_literal(SPAN, value, None, NumberBase::Decimal, ctx)
    }

    fn get_initializer_expr(value: f64, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let is_negative = value < 0.0;

        // Infinity
        let expr = if value.is_infinite() {
            let infinity = static_ident!("Infinity");
            let infinity_symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), infinity);
            ctx.create_ident_expr(SPAN, infinity, infinity_symbol_id, ReferenceFlags::Read)
        } else {
            let value = if is_negative { -value } else { value };
            Self::get_number_literal_expression(value, ctx)
        };

        if is_negative {
            Expression::new_unary_expression(SPAN, UnaryOperator::UnaryNegation, expr, ctx)
        } else {
            expr
        }
    }

    /// Try to inline `Direction.Up` to its literal value.
    /// Returns the constant and the `ReferenceId` of the enum identifier on the LHS,
    /// so the caller can delete the now-unused reference.
    fn try_inline_enum_member(
        &self,
        expr: &StaticMemberExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<(ConstantValue, ReferenceId)> {
        let Expression::Identifier(ident) = &expr.object else { return None };
        self.resolve_enum_member(ident, expr.property.name.as_str(), ctx)
    }

    /// Try to inline `Foo["%/*"]` to its literal value.
    fn try_inline_computed_enum_member(
        &self,
        expr: &ComputedMemberExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<(ConstantValue, ReferenceId)> {
        let Expression::Identifier(ident) = &expr.object else { return None };
        let Expression::StringLiteral(prop) = &expr.expression else { return None };
        self.resolve_enum_member(ident, prop.value.as_str(), ctx)
    }

    /// Resolve an enum member value by identifier and property name.
    /// Inlines const enums when `optimize_const_enums` is set,
    /// and regular enums when `optimize_enums` is set.
    fn resolve_enum_member(
        &self,
        ident: &IdentifierReference<'a>,
        property_name: &str,
        ctx: &TraverseCtx<'a>,
    ) -> Option<(ConstantValue, ReferenceId)> {
        let ref_id = ident.reference_id.get()?;
        let symbol_id = ctx.scoping().get_reference(ref_id).symbol_id()?;

        let flags = ctx.scoping().symbol_flags(symbol_id);
        let is_const_enum = flags.is_const_enum() && self.optimize_const_enums;
        let is_regular_enum = flags.contains(SymbolFlags::RegularEnum) && self.optimize_enums;
        if !is_const_enum && !is_regular_enum {
            return None;
        }

        let body_scopes = ctx.scoping().get_enum_body_scopes(symbol_id)?;
        for &body_scope_id in body_scopes {
            if let Some(member_symbol_id) =
                ctx.scoping().get_binding(body_scope_id, property_name.into())
                && let Some(value) = ctx.scoping().get_enum_member_value(member_symbol_id)
            {
                return Some((value.clone(), ref_id));
            }
        }
        None
    }
}

/// Rename the identifier references in the enum members to `enum_name.identifier`
/// ```ts
/// enum A {
///    a = 1,
///    b = a.toString(),
///    d = c,
/// }
/// ```
/// will be transformed to
/// ```ts
/// enum A {
///   a = 1,
///   b = A.a.toString(),
///   d = A.c,
/// }
/// ```
struct IdentifierReferenceRename<'a, 'ctx> {
    enum_binding: BoundIdentifier<'a>,
    enum_scope_id: ScopeId,
    scope_stack: NonEmptyStack<ScopeId>,
    ctx: &'ctx mut TraverseCtx<'a>,
}

impl<'a, 'ctx> IdentifierReferenceRename<'a, 'ctx> {
    fn new(
        enum_binding: &BoundIdentifier<'a>,
        enum_scope_id: ScopeId,
        ctx: &'ctx mut TraverseCtx<'a>,
    ) -> Self {
        IdentifierReferenceRename {
            enum_binding: enum_binding.clone(),
            enum_scope_id,
            scope_stack: NonEmptyStack::new(enum_scope_id),
            ctx,
        }
    }
}

impl IdentifierReferenceRename<'_, '_> {
    fn should_reference_enum_member(&self, ident: &IdentifierReference<'_>) -> bool {
        let scoping = self.ctx.scoping();

        // Check if this name is an enum member in the current body scope or any
        // sibling body scope (for merged enums like `enum Foo { A }; enum Foo { B = A }`).
        let is_enum_member = self.is_name_in_enum_scopes(scoping, ident.name.as_str());
        if !is_enum_member {
            return false;
        }

        let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
            // No symbol found, yet the name exists as a binding in the enum scope.
            // It must be referencing a member declared in a previous enum block: `enum Foo { A }; enum Foo { B = A }`
            return true;
        };

        let symbol_scope_id = scoping.symbol_scope_id(symbol_id);
        // Don't need to rename the identifier when it references a nested enum member:
        //
        // ```ts
        // enum OuterEnum {
        //   A = 0,
        //   B = () => {
        //     enum InnerEnum {
        //       A = 0,
        //       B = A,
        //           ^ This references to `InnerEnum.A` should not be renamed
        //     }
        //     return InnerEnum.B;
        //   }
        // }
        // ```
        *self.scope_stack.first() == symbol_scope_id
            // The resolved symbol is declared outside the enum,
            // and we have checked that the name exists as a binding in the enum scope:
            //
            // ```ts
            // const A = 0;
            // enum Foo { A }
            // enum Foo { B = A }
            //                ^ This should be renamed to Foo.A
            // ```
            || !self.scope_stack.contains(&symbol_scope_id)
    }

    /// Check if a name exists as an EnumMember binding in the current enum body scope
    /// or any sibling body scope (for merged enum declarations).
    fn is_name_in_enum_scopes(&self, scoping: &oxc_semantic::Scoping, name: &str) -> bool {
        // First check the current body scope
        if scoping
            .get_binding(self.enum_scope_id, name.into())
            .is_some_and(|sym_id| scoping.symbol_flags(sym_id).is_enum_member())
        {
            return true;
        }

        // Check sibling body scopes of the SAME enum (for merged declarations).
        // Only check the enum with the same name to avoid false positives from
        // other enums in the same scope (e.g., `var x = 10; enum Foo { c = b + x }` where
        // `x` is an outer variable, not enum member, even if another `enum Merge { x }` exists).
        if let Some(parent_scope) = scoping.scope_parent_id(self.enum_scope_id)
            && let Some(enum_sym_id) =
                scoping.get_binding(parent_scope, self.enum_binding.name.as_str().into())
            && let Some(body_scopes) = scoping.get_enum_body_scopes(enum_sym_id)
        {
            for &body_scope in body_scopes {
                if body_scope != self.enum_scope_id
                    && scoping
                        .get_binding(body_scope, name.into())
                        .is_some_and(|s| scoping.symbol_flags(s).is_enum_member())
                {
                    return true;
                }
            }
        }
        false
    }
}

impl<'a> VisitMut<'a> for IdentifierReferenceRename<'a, '_> {
    fn enter_scope(&mut self, _flags: ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.scope_stack.push(scope_id.get().unwrap());
    }

    fn leave_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::Identifier(ident) if self.should_reference_enum_member(ident) => {
                let object = self.enum_binding.create_read_expression(self.ctx);
                let property = IdentifierName::new(SPAN, ident.name, self.ctx);
                *expr = MemberExpression::new_static_member_expression(
                    SPAN, object, property, false, self.ctx,
                )
                .into();
            }
            _ => {
                walk_mut::walk_expression(self, expr);
            }
        }
    }
}
