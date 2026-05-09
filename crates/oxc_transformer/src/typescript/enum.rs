use std::cell::Cell;

use oxc_allocator::{TakeIn, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::{ScopeFlags, ScopeId};
use oxc_span::{SPAN, Span};
use oxc_str::Ident;
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

        let inlined = match expr {
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
                    ctx.ast.expression_string_literal(SPAN, ctx.ast.str(&s), None)
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

        let ast = ctx.ast;

        let is_export = export_span.is_some();
        let is_not_top_scope = !ctx.scoping().scope_flags(ctx.current_scope_id()).is_top();

        let enum_name: Ident = decl.id.name;
        let func_scope_id = decl.body.scope_id();
        let param_binding =
            ctx.generate_binding(enum_name, func_scope_id, SymbolFlags::FunctionScopedVariable);

        let id = param_binding.create_binding_pattern(ctx);

        // ((Foo) => {
        let params =
            ast.formal_parameter(SPAN, ast.vec(), id, NONE, NONE, false, None, false, false);
        let params = ast.vec1(params);
        let params = ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            NONE,
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
        let span = decl.span;
        let body = ast.alloc_function_body(span, ast.vec(), statements);
        let callee = ctx.ast.expression_function_with_scope_id_and_pure_and_pife(
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
        );

        let enum_symbol_id = decl.id.symbol_id();

        // Foo[Foo["X"] = 0] = "X";
        let redeclarations = ctx.scoping().symbol_redeclarations(enum_symbol_id);
        let is_already_declared =
            redeclarations.first().map_or_else(|| false, |rd| rd.span != decl.id.span);

        let arguments = if (is_export || is_not_top_scope) && !is_already_declared {
            // }({});
            let object_expr = ast.expression_object(SPAN, ast.vec());
            ast.vec1(Argument::from(object_expr))
        } else {
            // }(Foo || {});
            let op = LogicalOperator::Or;
            let left = ctx.create_bound_ident_expr(
                decl.id.span,
                enum_name,
                enum_symbol_id,
                ReferenceFlags::Read,
            );
            let right = ast.expression_object(SPAN, ast.vec());
            let expression = ast.expression_logical(span, left, op, right);
            ast.vec1(Argument::from(expression))
        };

        let call_expression = ast.expression_call_with_pure(
            span,
            callee,
            NONE,
            arguments,
            false,
            !has_potential_side_effect,
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
            let expr = ast.expression_assignment(span, op, left, call_expression);
            return Some(ast.statement_expression(span, expr));
        }

        let kind = if is_export || is_not_top_scope {
            VariableDeclarationKind::Let
        } else {
            VariableDeclarationKind::Var
        };
        let decls = {
            let binding_identifier = decl.id.clone();
            let binding = BindingPattern::BindingIdentifier(ctx.alloc(binding_identifier));
            let decl =
                ast.variable_declarator(span, kind, binding, NONE, Some(call_expression), false);
            ast.vec1(decl)
        };
        let variable_declaration = ast.declaration_variable(span, kind, decls, false);

        let stmt = if let Some(export_span) = export_span {
            let declaration = ctx
                .ast
                .plain_export_named_declaration_declaration(export_span, variable_declaration);
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
        let ast = ctx.ast;

        let mut statements = ast.vec();

        // If enum number has no initializer, its value will be the previous member value + 1,
        // if it's the first member, it will be `0`.
        // It used to keep track of the previous constant number.
        let mut prev_constant_number = Some(-1.0);

        let mut prev_member_name = None;

        for member in members.take_in(ctx.ast) {
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

                        IdentifierReferenceRename::new(param_binding.name, enum_scope_id, ctx)
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
                            ast.expression_string_literal(SPAN, ctx.ast.str(&s), None)
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
                    let expr = ctx.ast.expression_string_literal(SPAN, prev_member_name, None);
                    ast.member_expression_computed(SPAN, obj, expr, false).into()
                };

                // 1 + Foo["x"]
                let one = Self::get_number_literal_expression(1.0, ctx);
                ast.expression_binary(SPAN, one, BinaryOperator::Addition, self_ref)
            } else {
                Self::get_number_literal_expression(0.0, ctx)
            };

            let is_str = init.is_string_literal();

            // Foo["x"] = init
            let member_expr = {
                let obj = param_binding.create_read_expression(ctx);
                let expr = ast.expression_string_literal(SPAN, member_name, None);

                ast.member_expression_computed(SPAN, obj, expr, false)
            };
            let left = SimpleAssignmentTarget::from(member_expr);
            let mut expr = ast.expression_assignment(
                member_span,
                AssignmentOperator::Assign,
                left.into(),
                init,
            );

            // Foo[Foo["x"] = init] = "x"
            if !is_str {
                let member_expr = {
                    let obj = param_binding.create_read_expression(ctx);
                    ast.member_expression_computed(SPAN, obj, expr, false)
                };
                let left = SimpleAssignmentTarget::from(member_expr);
                let right = ast.expression_string_literal(SPAN, member_name, None);
                expr = ast.expression_assignment(
                    member_span,
                    AssignmentOperator::Assign,
                    left.into(),
                    right,
                );
            }

            prev_member_name = Some(member_name);
            statements.push(ast.statement_expression(member_span, expr));
        }

        let enum_ref = param_binding.create_read_expression(ctx);
        // return Foo;
        let return_stmt = ast.statement_return(SPAN, Some(enum_ref));
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
    /// Const enums are always removed when `optimize_const_enums` is set.
    /// Regular enums are removed only if all references were inlined away by `enter_expression`.
    fn can_remove_enum(&self, decl: &TSEnumDeclaration<'a>, ctx: &TraverseCtx<'a>) -> bool {
        self.may_remove_enum(decl, ctx)
            && (decl.r#const
                || ctx.scoping().get_resolved_reference_ids(decl.id.symbol_id()).is_empty())
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
        ctx.ast.expression_numeric_literal(SPAN, value, None, NumberBase::Decimal)
    }

    fn get_initializer_expr(value: f64, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let is_negative = value < 0.0;

        // Infinity
        let expr = if value.is_infinite() {
            let infinity = ctx.ast.ident("Infinity");
            let infinity_symbol_id = ctx.scoping().find_binding(ctx.current_scope_id(), infinity);
            ctx.create_ident_expr(SPAN, infinity, infinity_symbol_id, ReferenceFlags::Read)
        } else {
            let value = if is_negative { -value } else { value };
            Self::get_number_literal_expression(value, ctx)
        };

        if is_negative {
            ctx.ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, expr)
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
    enum_name: Ident<'a>,
    enum_scope_id: ScopeId,
    scope_stack: NonEmptyStack<ScopeId>,
    ctx: &'ctx TraverseCtx<'a>,
}

impl<'a, 'ctx> IdentifierReferenceRename<'a, 'ctx> {
    fn new(enum_name: Ident<'a>, enum_scope_id: ScopeId, ctx: &'ctx TraverseCtx<'a>) -> Self {
        IdentifierReferenceRename {
            enum_name,
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
                scoping.get_binding(parent_scope, self.enum_name.as_str().into())
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
                let object = self.ctx.ast.expression_identifier(SPAN, self.enum_name);
                let property = self.ctx.ast.identifier_name(SPAN, ident.name);
                *expr = self.ctx.ast.member_expression_static(SPAN, object, property, false).into();
            }
            _ => {
                walk_mut::walk_expression(self, expr);
            }
        }
    }
}
