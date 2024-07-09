use std::cell::Cell;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, visit::walk_mut, VisitMut};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::{
    number::{NumberBase, ToJsInt32, ToJsString},
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
    reference::ReferenceFlag,
    symbol::SymbolFlags,
};
use oxc_traverse::TraverseCtx;
use rustc_hash::FxHashMap;

use crate::context::Ctx;

pub struct TypeScriptEnum<'a> {
    ctx: Ctx<'a>,
    enums: FxHashMap<Atom<'a>, FxHashMap<Atom<'a>, ConstantValue>>,
}

impl<'a> TypeScriptEnum<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { ctx, enums: FxHashMap::default() }
    }

    pub fn transform_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let new_stmt = match stmt {
            Statement::TSEnumDeclaration(ts_enum_decl) => {
                self.transform_ts_enum(ts_enum_decl, None, ctx)
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(Declaration::TSEnumDeclaration(ts_enum_decl)) = &decl.declaration {
                    self.transform_ts_enum(ts_enum_decl, Some(decl.span), ctx)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(new_stmt) = new_stmt {
            *stmt = new_stmt;
        }
    }

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
        &mut self,
        decl: &TSEnumDeclaration<'a>,
        export_span: Option<Span>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if decl.declare {
            return None;
        }

        let ast = ctx.ast;

        let is_export = export_span.is_some();
        let is_not_top_scope = !ctx.scopes().get_flags(ctx.current_scope_id()).is_top();

        let enum_name = decl.id.name.clone();
        let func_scope_id = decl.scope_id.get().unwrap();
        let param_symbol_id = ctx.symbols_mut().create_symbol(
            decl.id.span,
            enum_name.to_compact_str(),
            SymbolFlags::FunctionScopedVariable,
            func_scope_id,
        );
        let ident = BindingIdentifier {
            span: decl.id.span,
            name: decl.id.name.clone(),
            symbol_id: Cell::new(Some(param_symbol_id)),
        };
        let kind = ast.binding_pattern_kind_from_binding_identifier(ident);
        let id = ast.binding_pattern(kind, Option::<TSTypeAnnotation>::None, false);

        // ((Foo) => {
        let params = ast.formal_parameter(SPAN, id, None, false, false, ast.new_vec());
        let params = ast.new_vec_single(params);
        let params = ast.alloc_formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            Option::<BindingRestElement>::None,
        );

        // Foo[Foo["X"] = 0] = "X";
        let is_already_declared = self.enums.contains_key(&enum_name);
        let statements = self.transform_ts_enum_members(&decl.members, enum_name.clone(), ctx);
        let body = ast.alloc_function_body(decl.span, ast.new_vec(), statements);
        let callee = Expression::FunctionExpression(ctx.alloc(Function {
            r#type: FunctionType::FunctionExpression,
            span: SPAN,
            id: None,
            generator: false,
            r#async: false,
            declare: false,
            this_param: None,
            params,
            body: Some(body),
            type_parameters: None,
            return_type: None,
            scope_id: Cell::new(Some(func_scope_id)),
        }));

        let var_symbol_id = decl.id.symbol_id.get().unwrap();
        let arguments = if (is_export || is_not_top_scope) && !is_already_declared {
            // }({});
            let object_expr = ast.expression_object(SPAN, ast.new_vec(), None);
            ast.new_vec_single(Argument::from(object_expr))
        } else {
            // }(Foo || {});
            let op = LogicalOperator::Or;
            let left = ctx.create_bound_reference_id(
                decl.id.span,
                enum_name.clone(),
                var_symbol_id,
                ReferenceFlag::Read,
            );
            let left = ast.expression_from_identifier_reference(left);
            let right = ast.expression_object(SPAN, ast.new_vec(), None);
            let expression = ast.expression_logical(SPAN, left, op, right);
            ast.new_vec_single(Argument::from(expression))
        };

        let call_expression = ast.expression_call(
            SPAN,
            arguments,
            callee,
            Option::<TSTypeParameterInstantiation>::None,
            false,
        );

        if is_already_declared {
            let op = AssignmentOperator::Assign;
            let left = ctx.create_bound_reference_id(
                decl.id.span,
                enum_name.clone(),
                var_symbol_id,
                ReferenceFlag::Write,
            );
            let left = ast.simple_assignment_target_from_identifier_reference(left);
            let expr = ast.expression_assignment(SPAN, op, left.into(), call_expression);
            return Some(ast.statement_expression(decl.span, expr));
        }

        let kind = if is_export || is_not_top_scope {
            VariableDeclarationKind::Let
        } else {
            VariableDeclarationKind::Var
        };
        let decls = {
            let binding_identifier = decl.id.clone();
            let binding_pattern_kind =
                ast.binding_pattern_kind_from_binding_identifier(binding_identifier);
            let binding =
                ast.binding_pattern(binding_pattern_kind, Option::<TSTypeAnnotation>::None, false);
            let decl = ast.variable_declarator(SPAN, kind, binding, Some(call_expression), false);
            ast.new_vec_single(decl)
        };
        let variable_declaration = ast.declaration_variable(decl.span, kind, decls, false);

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
        &mut self,
        members: &Vec<'a, TSEnumMember<'a>>,
        enum_name: Atom<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Vec<'a, Statement<'a>> {
        // TODO: Set `span` and `references_id` on all `IdentifierReference`s created here

        let ast = ctx.ast;

        let mut statements = ast.new_vec();
        let mut prev_constant_value = Some(ConstantValue::Number(-1.0));
        let mut previous_enum_members = self.enums.entry(enum_name.clone()).or_default().clone();
        let mut prev_member_name: Option<Atom<'a>> = None;

        for member in members {
            let member_name = match &member.id {
                TSEnumMemberName::StaticIdentifier(id) => &id.name,
                TSEnumMemberName::StaticStringLiteral(str) => &str.value,
                #[allow(clippy::unnested_or_patterns)] // Clippy is wrong
                TSEnumMemberName::StaticNumericLiteral(_) | match_expression!(TSEnumMemberName) => {
                    unreachable!()
                }
            };

            let init = if let Some(initializer) = member.initializer.as_ref() {
                let constant_value =
                    self.computed_constant_value(initializer, &previous_enum_members);

                // prev_constant_value = constant_value
                let init = match constant_value {
                    None => {
                        prev_constant_value = None;
                        let mut new_initializer = ast.copy(initializer);

                        // If the initializer is a binding identifier,
                        // and it is not a binding in the current scope and parent scopes,
                        // we need to rename it to the enum name. e.g. `d = c` to `d = A.c`
                        // same behavior in https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L145-L150
                        let has_binding = matches!(
                            &new_initializer,
                            Expression::Identifier(ident) if ctx.scopes().has_binding(ctx.current_scope_id(), &ident.name)
                        );
                        if !has_binding {
                            IdentifierReferenceRename::new(
                                enum_name.clone(),
                                previous_enum_members.clone(),
                                ctx,
                            )
                            .visit_expression(&mut new_initializer);
                        }

                        new_initializer
                    }
                    Some(constant_value) => {
                        previous_enum_members.insert(member_name.clone(), constant_value.clone());
                        match constant_value {
                            ConstantValue::Number(v) => {
                                prev_constant_value = Some(ConstantValue::Number(v));
                                self.get_initializer_expr(v)
                            }
                            ConstantValue::String(str) => {
                                prev_constant_value = None;
                                ast.expression_string_literal(SPAN, str)
                            }
                        }
                    }
                };

                init
            } else if let Some(ref value) = prev_constant_value {
                match value {
                    ConstantValue::Number(value) => {
                        let value = value + 1.0;
                        let constant_value = ConstantValue::Number(value);
                        prev_constant_value = Some(constant_value.clone());
                        previous_enum_members.insert(member_name.clone(), constant_value);
                        self.get_initializer_expr(value)
                    }
                    ConstantValue::String(_) => unreachable!(),
                }
            } else if let Some(prev_member_name) = prev_member_name {
                let self_ref = {
                    let obj = ast.expression_identifier_reference(SPAN, &enum_name);
                    let expr = ctx.ast.expression_string_literal(SPAN, prev_member_name);
                    ast.member_expression_computed(SPAN, obj, expr, false).into()
                };

                // 1 + Foo["x"]
                let one = self.get_number_literal_expression(1.0);
                ast.expression_binary(SPAN, one, BinaryOperator::Addition, self_ref)
            } else {
                self.get_number_literal_expression(0.0)
            };

            let is_str = init.is_string_literal();

            // Foo["x"] = init
            let member_expr = {
                let obj = ast.expression_identifier_reference(SPAN, &enum_name);
                let expr = ast.expression_string_literal(SPAN, member_name);

                ast.member_expression_computed(SPAN, obj, expr, false)
            };
            let left = ast.simple_assignment_target_member_expression(member_expr);
            let mut expr =
                ast.expression_assignment(SPAN, AssignmentOperator::Assign, left.into(), init);

            // Foo[Foo["x"] = init] = "x"
            if !is_str {
                let member_expr = {
                    let obj = ast.expression_identifier_reference(SPAN, &enum_name);
                    ast.member_expression_computed(SPAN, obj, expr, false)
                };
                let left = ast.simple_assignment_target_member_expression(member_expr);
                let right = ast.expression_string_literal(SPAN, member_name);
                expr =
                    ast.expression_assignment(SPAN, AssignmentOperator::Assign, left.into(), right);
            }

            prev_member_name = Some(member_name.clone());
            statements.push(ast.statement_expression(member.span, expr));
        }

        self.enums.insert(enum_name.clone(), previous_enum_members.clone());

        let enum_ref = ast.expression_identifier_reference(SPAN, enum_name);
        // return Foo;
        let return_stmt = ast.statement_return(SPAN, Some(enum_ref));
        statements.push(return_stmt);

        statements
    }

    fn get_number_literal_expression(&self, value: f64) -> Expression<'a> {
        self.ctx.ast.expression_numeric_literal(SPAN, value, value.to_string(), NumberBase::Decimal)
    }

    fn get_initializer_expr(&self, value: f64) -> Expression<'a> {
        let is_negative = value < 0.0;

        // Infinity
        let expr = if value.is_infinite() {
            self.ctx.ast.expression_identifier_reference(SPAN, "Infinity")
        } else {
            let value = if is_negative { -value } else { value };
            self.get_number_literal_expression(value)
        };

        if is_negative {
            self.ctx.ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, expr)
        } else {
            expr
        }
    }
}

#[derive(Debug, Clone)]
enum ConstantValue {
    Number(f64),
    String(String),
}

impl<'a> TypeScriptEnum<'a> {
    /// Evaluate the expression to a constant value.
    /// Refer to [babel](https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L241C1-L394C2)
    fn computed_constant_value(
        &self,
        expr: &Expression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        self.evaluate(expr, prev_members)
    }

    fn evaluate_ref(
        &self,
        expr: &Expression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        match expr {
            match_member_expression!(Expression) => {
                let expr = expr.to_member_expression();
                let Expression::Identifier(ident) = expr.object() else { return None };
                let members = self.enums.get(&ident.name)?;
                let property = expr.static_property_name()?;
                return members.get(property).cloned();
            }
            Expression::Identifier(ident) => {
                if ident.name == "Infinity" {
                    return Some(ConstantValue::Number(f64::INFINITY));
                } else if ident.name == "NaN" {
                    return Some(ConstantValue::Number(f64::NAN));
                }

                if let Some(value) = prev_members.get(&ident.name) {
                    return Some(value.clone());
                }

                // TODO:
                // This is a bit tricky because we need to find the BindingIdentifier that corresponds to the identifier reference.
                // and then we may to evaluate the initializer of the BindingIdentifier.
                // finally, we can get the value of the identifier and call the `computed_constant_value` function.
                // See https://github.com/babel/babel/blob/610897a9a96c5e344e77ca9665df7613d2f88358/packages/babel-plugin-transform-typescript/src/enum.ts#L327-L329
                None
            }
            _ => None,
        }
    }

    fn evaluate(
        &self,
        expr: &Expression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        match expr {
            Expression::Identifier(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_) => self.evaluate_ref(expr, prev_members),
            Expression::BinaryExpression(expr) => self.eval_binary_expression(expr, prev_members),
            Expression::UnaryExpression(expr) => self.eval_unary_expression(expr, prev_members),
            Expression::NumericLiteral(lit) => Some(ConstantValue::Number(lit.value)),
            Expression::StringLiteral(lit) => Some(ConstantValue::String(lit.value.to_string())),
            Expression::TemplateLiteral(lit) => {
                let mut value = String::new();
                for part in &lit.quasis {
                    value.push_str(&part.value.raw);
                }
                Some(ConstantValue::String(value))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.evaluate(&expr.expression, prev_members)
            }
            _ => None,
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
    fn eval_binary_expression(
        &self,
        expr: &BinaryExpression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        let left = self.evaluate(&expr.left, prev_members)?;
        let right = self.evaluate(&expr.right, prev_members)?;

        if matches!(expr.operator, BinaryOperator::Addition)
            && (matches!(left, ConstantValue::String(_))
                || matches!(right, ConstantValue::String(_)))
        {
            let left_string = match left {
                ConstantValue::String(str) => str,
                ConstantValue::Number(v) => v.to_js_string(),
            };

            let right_string = match right {
                ConstantValue::String(str) => str,
                ConstantValue::Number(v) => v.to_js_string(),
            };

            return Some(ConstantValue::String(format!("{left_string}{right_string}")));
        }

        let left = match left {
            ConstantValue::Number(v) => v,
            ConstantValue::String(_) => return None,
        };

        let right = match right {
            ConstantValue::Number(v) => v,
            ConstantValue::String(_) => return None,
        };

        match expr.operator {
            BinaryOperator::ShiftRight => Some(ConstantValue::Number(f64::from(
                left.to_js_int_32().wrapping_shr(right.to_js_int_32() as u32),
            ))),
            BinaryOperator::ShiftRightZeroFill => Some(ConstantValue::Number(f64::from(
                (left.to_js_int_32() as u32).wrapping_shr(right.to_js_int_32() as u32),
            ))),
            BinaryOperator::ShiftLeft => Some(ConstantValue::Number(f64::from(
                left.to_js_int_32().wrapping_shl(right.to_js_int_32() as u32),
            ))),
            BinaryOperator::BitwiseXOR => {
                Some(ConstantValue::Number(f64::from(left.to_js_int_32() ^ right.to_js_int_32())))
            }
            BinaryOperator::BitwiseOR => {
                Some(ConstantValue::Number(f64::from(left.to_js_int_32() | right.to_js_int_32())))
            }
            BinaryOperator::BitwiseAnd => {
                Some(ConstantValue::Number(f64::from(left.to_js_int_32() & right.to_js_int_32())))
            }
            BinaryOperator::Multiplication => Some(ConstantValue::Number(left * right)),
            BinaryOperator::Division => Some(ConstantValue::Number(left / right)),
            BinaryOperator::Addition => Some(ConstantValue::Number(left + right)),
            BinaryOperator::Subtraction => Some(ConstantValue::Number(left - right)),
            BinaryOperator::Remainder => Some(ConstantValue::Number(left % right)),
            BinaryOperator::Exponential => Some(ConstantValue::Number(left.powf(right))),
            _ => None,
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn eval_unary_expression(
        &self,
        expr: &UnaryExpression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        let value = self.evaluate(&expr.argument, prev_members)?;

        let value = match value {
            ConstantValue::Number(value) => value,
            ConstantValue::String(_) => {
                let value = if expr.operator == UnaryOperator::UnaryNegation {
                    ConstantValue::Number(f64::NAN)
                } else if expr.operator == UnaryOperator::BitwiseNot {
                    ConstantValue::Number(-1.0)
                } else {
                    value
                };
                return Some(value);
            }
        };

        match expr.operator {
            UnaryOperator::UnaryPlus => Some(ConstantValue::Number(value)),
            UnaryOperator::UnaryNegation => Some(ConstantValue::Number(-value)),
            UnaryOperator::BitwiseNot => {
                Some(ConstantValue::Number(f64::from(!value.to_js_int_32())))
            }
            _ => None,
        }
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
struct IdentifierReferenceRename<'a, 'b> {
    enum_name: Atom<'a>,
    ctx: &'b TraverseCtx<'a>,
    previous_enum_members: FxHashMap<Atom<'a>, ConstantValue>,
}

impl<'a, 'b> IdentifierReferenceRename<'a, 'b> {
    fn new(
        enum_name: Atom<'a>,
        previous_enum_members: FxHashMap<Atom<'a>, ConstantValue>,
        ctx: &'b TraverseCtx<'a>,
    ) -> Self {
        IdentifierReferenceRename { enum_name, ctx, previous_enum_members }
    }
}

impl<'a, 'b> VisitMut<'a> for IdentifierReferenceRename<'a, 'b> {
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        let new_expr = match expr {
            match_member_expression!(Expression) => {
                // handle a.toString() -> A.a.toString()
                let expr = expr.to_member_expression();
                if let Expression::Identifier(ident) = expr.object() {
                    if !self.previous_enum_members.contains_key(&ident.name) {
                        return;
                    }
                };
                None
            }
            Expression::Identifier(ident) => {
                // If the identifier is binding in current/parent scopes,
                // and it is not a member of the enum,
                // we don't need to rename it.
                // `var c = 1; enum A { a = c }` -> `var c = 1; enum A { a = c }
                if !self.previous_enum_members.contains_key(&ident.name)
                    && self.ctx.scopes().has_binding(self.ctx.current_scope_id(), &ident.name)
                {
                    return;
                }

                // TODO: shadowed case, e.g. let ident = 1; ident; // ident is not an enum
                // enum_name.identifier
                let object = self.ctx.ast.expression_identifier_reference(SPAN, &self.enum_name);
                let property = self.ctx.ast.identifier_name(SPAN, &ident.name);
                Some(self.ctx.ast.member_expression_static(SPAN, object, property, false).into())
            }
            _ => None,
        };
        if let Some(new_expr) = new_expr {
            *expr = new_expr;
        } else {
            walk_mut::walk_expression(self, expr);
        }
    }
}
