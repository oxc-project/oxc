use std::rc::Rc;

use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, visit::walk_mut, VisitMut};
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    number::{NumberBase, ToJsInt32, ToJsString},
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
};
use oxc_traverse::TraverseCtx;
use rustc_hash::FxHashMap;

use crate::context::Ctx;

pub struct TypeScriptEnum<'a> {
    ctx: Ctx<'a>,
    enums: FxHashMap<Atom<'a>, FxHashMap<Atom<'a>, ConstantValue>>,
}

impl<'a> TypeScriptEnum<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx), enums: FxHashMap::default() }
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
    pub fn transform_ts_enum(
        &mut self,
        decl: &Box<'a, TSEnumDeclaration<'a>>,
        is_export: bool,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        if decl.modifiers.contains(ModifierKind::Declare) {
            return None;
        }

        let span = decl.span;
        let ident = decl.id.clone();
        let kind = self.ctx.ast.binding_pattern_identifier(ident);
        let id = self.ctx.ast.binding_pattern(kind, None, false);

        // ((Foo) => {
        let params =
            self.ctx.ast.formal_parameter(SPAN, id, None, false, false, self.ctx.ast.new_vec());
        let params = self.ctx.ast.new_vec_single(params);
        let params = self.ctx.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            None,
        );

        // Foo[Foo["X"] = 0] = "X";
        let enum_name = decl.id.name.clone();
        let is_already_declared = self.enums.contains_key(&enum_name);
        let statements = self.transform_ts_enum_members(&decl.members, &enum_name, ctx);
        let body = self.ctx.ast.function_body(decl.span, self.ctx.ast.new_vec(), statements);
        let r#type = FunctionType::FunctionExpression;
        let callee = self.ctx.ast.plain_function(r#type, SPAN, None, params, Some(body));
        let callee = Expression::FunctionExpression(callee);

        let arguments = if is_export && !is_already_declared {
            // }({});
            let object_expr = self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
            self.ctx.ast.new_vec_single(Argument::from(object_expr))
        } else {
            // }(Foo || {});
            let op = LogicalOperator::Or;
            let left = self
                .ctx
                .ast
                .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
            let right = self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
            let expression = self.ctx.ast.logical_expression(SPAN, left, op, right);
            self.ctx.ast.new_vec_single(Argument::from(expression))
        };

        let call_expression = self.ctx.ast.call_expression(SPAN, callee, arguments, false, None);

        if is_already_declared {
            let op = AssignmentOperator::Assign;
            let left = self.ctx.ast.simple_assignment_target_identifier(IdentifierReference::new(
                SPAN,
                enum_name.clone(),
            ));
            let expr = self.ctx.ast.assignment_expression(SPAN, op, left, call_expression);
            return Some(self.ctx.ast.expression_statement(SPAN, expr));
        }

        let kind =
            if is_export { VariableDeclarationKind::Let } else { VariableDeclarationKind::Var };
        let decls = {
            let mut decls = self.ctx.ast.new_vec();

            let binding_identifier = BindingIdentifier::new(SPAN, enum_name.clone());
            let binding_pattern_kind = self.ctx.ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ctx.ast.binding_pattern(binding_pattern_kind, None, false);
            let decl =
                self.ctx.ast.variable_declarator(SPAN, kind, binding, Some(call_expression), false);

            decls.push(decl);
            decls
        };
        let variable_declaration =
            self.ctx.ast.variable_declaration(span, kind, decls, Modifiers::empty());
        let variable_declaration = Declaration::VariableDeclaration(variable_declaration);

        let stmt = if is_export {
            let declaration =
                self.ctx.ast.plain_export_named_declaration_declaration(SPAN, variable_declaration);

            self.ctx.ast.module_declaration(ModuleDeclaration::ExportNamedDeclaration(declaration))
        } else {
            Statement::from(variable_declaration)
        };
        Some(stmt)
    }

    pub fn transform_ts_enum_members(
        &mut self,
        members: &Vec<'a, TSEnumMember<'a>>,
        enum_name: &Atom<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Vec<'a, Statement<'a>> {
        let mut statements = self.ctx.ast.new_vec();
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
                        let mut new_initializer = self.ctx.ast.copy(initializer);

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
                                &self.ctx,
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
                                self.ctx.ast.literal_string_expression(StringLiteral {
                                    span: SPAN,
                                    value: self.ctx.ast.new_atom(&str),
                                })
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
                    let obj = self.ctx.ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, enum_name.clone()),
                    );
                    let expr = self
                        .ctx
                        .ast
                        .literal_string_expression(StringLiteral::new(SPAN, prev_member_name));
                    self.ctx.ast.computed_member_expression(SPAN, obj, expr, false)
                };

                // 1 + Foo["x"]
                let one = self.get_number_literal_expression(1.0);
                self.ctx.ast.binary_expression(SPAN, one, BinaryOperator::Addition, self_ref)
            } else {
                self.get_number_literal_expression(0.0)
            };

            let is_str = init.is_string_literal();

            // Foo["x"] = init
            let member_expr = {
                let obj = self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    enum_name.clone(),
                ));
                let literal = StringLiteral::new(SPAN, member_name.clone());
                let expr = self.ctx.ast.literal_string_expression(literal);

                self.ctx.ast.computed_member(SPAN, obj, expr, false)
            };
            let left = self.ctx.ast.simple_assignment_target_member_expression(member_expr);
            let mut expr =
                self.ctx.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, init);

            // Foo[Foo["x"] = init] = "x"
            if !is_str {
                let member_expr = {
                    let obj = self.ctx.ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, enum_name.clone()),
                    );
                    self.ctx.ast.computed_member(SPAN, obj, expr, false)
                };
                let left = self.ctx.ast.simple_assignment_target_member_expression(member_expr);
                let right = self
                    .ctx
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));
                expr = self.ctx.ast.assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    left,
                    right,
                );
            }

            prev_member_name = Some(member_name.clone());
            statements.push(self.ctx.ast.expression_statement(member.span, expr));
        }

        self.enums.insert(enum_name.clone(), previous_enum_members.clone());

        let enum_ref = self
            .ctx
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
        // return Foo;
        let return_stmt = self.ctx.ast.return_statement(SPAN, Some(enum_ref));
        statements.push(return_stmt);

        statements
    }

    fn get_number_literal_expression(&self, value: f64) -> Expression<'a> {
        self.ctx.ast.literal_number_expression(NumericLiteral {
            span: SPAN,
            value,
            raw: self.ctx.ast.new_str(&value.to_string()),
            base: NumberBase::Decimal,
        })
    }

    fn get_initializer_expr(&self, value: f64) -> Expression<'a> {
        let is_negative = value < 0.0;

        // Infinity
        let expr = if value.is_infinite() {
            let ident = IdentifierReference::new(SPAN, self.ctx.ast.new_atom("Infinity"));
            self.ctx.ast.identifier_reference_expression(ident)
        } else {
            let value = if is_negative { -value } else { value };
            self.get_number_literal_expression(value)
        };

        if is_negative {
            self.ctx.ast.unary_expression(SPAN, UnaryOperator::UnaryNegation, expr)
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
struct IdentifierReferenceRename<'a> {
    enum_name: Atom<'a>,
    ctx: Ctx<'a>,
    previous_enum_members: FxHashMap<Atom<'a>, ConstantValue>,
}

impl IdentifierReferenceRename<'_> {
    fn new<'a>(
        enum_name: Atom<'a>,
        previous_enum_members: FxHashMap<Atom<'a>, ConstantValue>,
        ctx: &Ctx<'a>,
    ) -> IdentifierReferenceRename<'a> {
        IdentifierReferenceRename { enum_name, ctx: Rc::clone(ctx), previous_enum_members }
    }
}

impl<'a> VisitMut<'a> for IdentifierReferenceRename<'a> {
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
                // TODO: shadowed case, e.g. let ident = 1; ident; // ident is not an enum
                // enum_name.identifier
                let ident_reference = IdentifierReference::new(SPAN, self.enum_name.clone());
                let object = self.ctx.ast.identifier_reference_expression(ident_reference);
                let property = self.ctx.ast.identifier_name(SPAN, &ident.name);
                Some(self.ctx.ast.static_member_expression(SPAN, object, property, false))
            }
            _ => None,
        };
        if let Some(new_expr) = new_expr {
            *expr = new_expr;
        } else {
            walk_mut::walk_expression_mut(self, expr);
        }
    }
}
