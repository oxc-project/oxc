use std::mem;

use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator},
    NumberBase,
};

use crate::utils::is_valid_identifier;

use super::TypeScript;

impl<'a> TypeScript<'a> {
    /// ```TypeScript
    /// enum Foo {
    ///   X
    /// }
    /// ```
    /// ```JavaScript
    /// var Foo = ((Foo) => {
    ///   const X = 0; Foo[Foo["X"] = X] = "X";
    ///   return Foo;
    /// })(Foo || {});
    /// ```
    pub fn transform_ts_enum(
        &self,
        decl: &mut Box<'a, TSEnumDeclaration<'a>>,
    ) -> Option<Declaration<'a>> {
        if decl.modifiers.contains(ModifierKind::Declare) {
            return None;
        }

        let span = decl.span;
        let ident = decl.id.clone();
        let kind = self.ctx.ast.binding_pattern_identifier(ident);
        let id = self.ctx.ast.binding_pattern(kind, None, false);

        let mut params = self.ctx.ast.new_vec();

        // ((Foo) => {
        params.push(self.ctx.ast.formal_parameter(
            SPAN,
            id,
            None,
            false,
            false,
            self.ctx.ast.new_vec(),
        ));

        let params = self.ctx.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            None,
        );

        // Foo[Foo["X"] = 0] = "X";
        let enum_name = decl.id.name.clone();
        let statements = self.transform_ts_enum_members(&mut decl.members, &enum_name);
        let body = self.ctx.ast.function_body(decl.span, self.ctx.ast.new_vec(), statements);

        let callee =
            self.ctx.ast.arrow_function_expression(SPAN, false, false, params, body, None, None);

        // })(Foo || {});
        let mut arguments = self.ctx.ast.new_vec();
        let op = LogicalOperator::Or;
        let left = self
            .ctx
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
        let right = self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
        let expression = self.ctx.ast.logical_expression(SPAN, left, op, right);
        arguments.push(Argument::Expression(expression));

        let call_expression = self.ctx.ast.call_expression(SPAN, callee, arguments, false, None);

        let kind = VariableDeclarationKind::Var;
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

        Some(Declaration::VariableDeclaration(variable_declaration))
    }

    pub fn transform_ts_enum_members(
        &self,
        members: &mut Vec<'a, TSEnumMember<'a>>,
        enum_name: &Atom<'a>,
    ) -> Vec<'a, Statement<'a>> {
        let mut default_init = self.ctx.ast.literal_number_expression(NumericLiteral {
            span: SPAN,
            value: 0.0,
            raw: "0",
            base: NumberBase::Decimal,
        });
        let mut statements = self.ctx.ast.new_vec();

        for member in members.iter_mut() {
            let (member_name, member_span) = match &member.id {
                TSEnumMemberName::Identifier(id) => (&id.name, id.span),
                TSEnumMemberName::StringLiteral(str) => (&str.value, str.span),
                TSEnumMemberName::ComputedPropertyName(..)
                | TSEnumMemberName::NumericLiteral(..) => unreachable!(),
            };

            let mut init = self
                .ctx
                .ast
                .move_expression(member.initializer.as_mut().unwrap_or(&mut default_init));

            let is_str = init.is_string_literal();

            let mut self_ref = {
                let obj = self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    enum_name.clone(),
                ));
                let expr = self
                    .ctx
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));
                self.ctx.ast.computed_member_expression(SPAN, obj, expr, false)
            };

            if is_valid_identifier(member_name, true) {
                let ident = IdentifierReference::new(member_span, member_name.clone());

                self_ref = self.ctx.ast.identifier_reference_expression(ident.clone());
                let init =
                    mem::replace(&mut init, self.ctx.ast.identifier_reference_expression(ident));

                let kind = VariableDeclarationKind::Const;
                let decls = {
                    let mut decls = self.ctx.ast.new_vec();

                    let binding_identifier = BindingIdentifier::new(SPAN, member_name.clone());
                    let binding_pattern_kind =
                        self.ctx.ast.binding_pattern_identifier(binding_identifier);
                    let binding = self.ctx.ast.binding_pattern(binding_pattern_kind, None, false);
                    let decl =
                        self.ctx.ast.variable_declarator(SPAN, kind, binding, Some(init), false);

                    decls.push(decl);
                    decls
                };
                let decl = self.ctx.ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());
                let stmt: Statement<'_> =
                    Statement::Declaration(Declaration::VariableDeclaration(decl));

                statements.push(stmt);
            }

            // Foo["x"] = init
            let member_expr = {
                let obj = self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    enum_name.clone(),
                ));
                let expr = self
                    .ctx
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));

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

            statements.push(self.ctx.ast.expression_statement(member.span, expr));

            // 1 + Foo["x"]
            default_init = {
                let one = self.ctx.ast.literal_number_expression(NumericLiteral {
                    span: SPAN,
                    value: 1.0,
                    raw: "1",
                    base: NumberBase::Decimal,
                });

                self.ctx.ast.binary_expression(SPAN, one, BinaryOperator::Addition, self_ref)
            };
        }

        let enum_ref = self
            .ctx
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
        // return Foo;
        let return_stmt = self.ctx.ast.return_statement(SPAN, Some(enum_ref));
        statements.push(return_stmt);

        statements
    }
}
