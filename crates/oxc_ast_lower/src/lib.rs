#![allow(dead_code, unused_variables, clippy::todo)]

use oxc_allocator::{Allocator, Box, Vec};
use oxc_ast::ast;
use oxc_hir::{hir, hir_builder::HirBuilder};

pub struct AstLower<'a> {
    hir: HirBuilder<'a>,
}

impl<'a> AstLower<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self { hir: HirBuilder::new(allocator) }
    }

    #[must_use]
    pub fn build(mut self, program: &ast::Program<'a>) -> hir::Program<'a> {
        self.lower_program(program)
    }

    #[must_use]
    pub fn lower_vec<T, R, F>(&mut self, items: &Vec<'a, T>, cb: F) -> Vec<'a, R>
    where
        F: Fn(&mut Self, &T) -> R,
    {
        let mut vec = self.hir.new_vec_with_capacity(items.len());
        for item in items {
            vec.push(cb(self, item));
        }
        vec
    }

    fn lower_program(&mut self, program: &ast::Program<'a>) -> hir::Program<'a> {
        let directives = self.lower_vec(&program.directives, Self::lower_directive);
        let statements = self.lower_vec(&program.body, Self::lower_statement);
        self.hir.program(program.span, directives, statements)
    }

    fn lower_directive(&mut self, directive: &ast::Directive<'a>) -> hir::Directive<'a> {
        let expression = self.lower_string_literal(&directive.expression);
        self.hir.directive(directive.span, expression, directive.directive)
    }

    fn lower_statement(&mut self, statement: &ast::Statement<'a>) -> hir::Statement<'a> {
        match statement {
            ast::Statement::BlockStatement(stmt) => self.lower_block_statement(stmt),
            ast::Statement::BreakStatement(stmt) => self.lower_break_statement(stmt),
            ast::Statement::ContinueStatement(stmt) => self.lower_continue_statement(stmt),
            ast::Statement::DebuggerStatement(stmt) => self.lower_debugger_statement(stmt),
            ast::Statement::DoWhileStatement(stmt) => self.lower_do_while_statement(stmt),
            ast::Statement::EmptyStatement(stmt) => self.lower_empty_statement(stmt),
            ast::Statement::ExpressionStatement(stmt) => self.lower_expression_statement(stmt),
            ast::Statement::ForInStatement(stmt) => self.lower_for_in_statement(stmt),
            ast::Statement::ForOfStatement(stmt) => self.lower_for_of_statement(stmt),
            ast::Statement::ForStatement(stmt) => self.lower_for_statement(stmt),
            ast::Statement::IfStatement(stmt) => self.lower_if_statement(stmt),
            ast::Statement::LabeledStatement(stmt) => self.lower_labeled_statement(stmt),
            ast::Statement::ReturnStatement(stmt) => self.lower_return_statement(stmt),
            ast::Statement::SwitchStatement(stmt) => self.lower_switch_statement(stmt),
            ast::Statement::ThrowStatement(stmt) => self.lower_throw_statement(stmt),
            ast::Statement::TryStatement(stmt) => self.lower_try_statement(stmt),
            ast::Statement::WhileStatement(stmt) => self.lower_while_statement(stmt),
            ast::Statement::WithStatement(stmt) => self.lower_with_statement(stmt),
            ast::Statement::ModuleDeclaration(_decl) => todo!(),
            ast::Statement::Declaration(_decl) => todo!(),
        }
    }

    fn lower_block(&mut self, stmt: &ast::BlockStatement<'a>) -> Box<'a, hir::BlockStatement<'a>> {
        let body = self.lower_vec(&stmt.body, Self::lower_statement);
        self.hir.block(stmt.span, body)
    }

    fn lower_block_statement(&mut self, stmt: &ast::BlockStatement<'a>) -> hir::Statement<'a> {
        let body = self.lower_vec(&stmt.body, Self::lower_statement);
        self.hir.block_statement(stmt.span, body)
    }

    fn lower_break_statement(&mut self, stmt: &ast::BreakStatement) -> hir::Statement<'a> {
        let label = stmt.label.as_ref().map(|ident| self.lower_label_identifier(ident));
        self.hir.break_statement(stmt.span, label)
    }

    fn lower_continue_statement(&mut self, stmt: &ast::ContinueStatement) -> hir::Statement<'a> {
        let label = stmt.label.as_ref().map(|ident| self.lower_label_identifier(ident));
        self.hir.continue_statement(stmt.span, label)
    }

    fn lower_debugger_statement(&mut self, stmt: &ast::DebuggerStatement) -> hir::Statement<'a> {
        self.hir.debugger_statement(stmt.span)
    }

    fn lower_do_while_statement(&mut self, stmt: &ast::DoWhileStatement<'a>) -> hir::Statement<'a> {
        let body = self.lower_statement(&stmt.body);
        let test = self.lower_expression(&stmt.test);
        self.hir.do_while_statement(stmt.span, body, test)
    }

    fn lower_empty_statement(&mut self, stmt: &ast::EmptyStatement) -> hir::Statement<'a> {
        self.hir.empty_statement(stmt.span)
    }

    fn lower_expression_statement(
        &mut self,
        stmt: &ast::ExpressionStatement<'a>,
    ) -> hir::Statement<'a> {
        let expression = self.lower_expression(&stmt.expression);
        self.hir.expression_statement(stmt.span, expression)
    }

    fn lower_for_statement(&mut self, stmt: &ast::ForStatement<'a>) -> hir::Statement<'a> {
        todo!()
    }

    fn lower_for_statement_init(
        &mut self,
        init: &ast::ForStatementInit<'a>,
    ) -> hir::ForStatementInit<'a> {
        todo!()
    }

    fn lower_for_in_statement(&mut self, stmt: &ast::ForInStatement<'a>) -> hir::Statement<'a> {
        todo!()
    }

    fn lower_for_of_statement(&mut self, stmt: &ast::ForOfStatement<'a>) -> hir::Statement<'a> {
        todo!()
    }

    fn lower_for_statement_left(
        &mut self,
        left: &ast::ForStatementLeft<'a>,
    ) -> hir::ForStatementLeft<'a> {
        todo!()
    }

    fn lower_if_statement(&mut self, stmt: &ast::IfStatement<'a>) -> hir::Statement<'a> {
        let test = self.lower_expression(&stmt.test);
        let consequent = self.lower_statement(&stmt.consequent);
        let alternate = stmt.alternate.as_ref().map(|stmt| self.lower_statement(stmt));
        self.hir.if_statement(stmt.span, test, consequent, alternate)
    }

    fn lower_labeled_statement(&mut self, stmt: &ast::LabeledStatement<'a>) -> hir::Statement<'a> {
        let label = self.lower_label_identifier(&stmt.label);
        let body = self.lower_statement(&stmt.body);
        self.hir.labeled_statement(stmt.span, label, body)
    }

    fn lower_return_statement(&mut self, stmt: &ast::ReturnStatement<'a>) -> hir::Statement<'a> {
        let argument = stmt.argument.as_ref().map(|expr| self.lower_expression(expr));
        self.hir.return_statement(stmt.span, argument)
    }

    fn lower_switch_statement(&mut self, stmt: &ast::SwitchStatement<'a>) -> hir::Statement<'a> {
        let discriminant = self.lower_expression(&stmt.discriminant);
        let cases = self.lower_vec(&stmt.cases, Self::lower_switch_case);
        self.hir.switch_statement(stmt.span, discriminant, cases)
    }

    fn lower_switch_case(&mut self, case: &ast::SwitchCase<'a>) -> hir::SwitchCase<'a> {
        let test = case.test.as_ref().map(|expr| self.lower_expression(expr));
        let consequent = self.lower_vec(&case.consequent, Self::lower_statement);
        self.hir.switch_case(case.span, test, consequent)
    }

    fn lower_throw_statement(&mut self, stmt: &ast::ThrowStatement<'a>) -> hir::Statement<'a> {
        let argument = self.lower_expression(&stmt.argument);
        self.hir.throw_statement(stmt.span, argument)
    }

    fn lower_try_statement(&mut self, stmt: &ast::TryStatement<'a>) -> hir::Statement<'a> {
        let block = self.lower_block(&stmt.block);
        let handler = stmt.handler.as_ref().map(|clause| self.lower_catch_clause(clause));
        let finalizer = stmt.finalizer.as_ref().map(|stmt| self.lower_block(stmt));
        self.hir.try_statement(stmt.span, block, handler, finalizer)
    }

    fn lower_catch_clause(
        &mut self,
        clause: &ast::CatchClause<'a>,
    ) -> Box<'a, hir::CatchClause<'a>> {
        let body = self.lower_block(&clause.body);
        let param = clause.param.as_ref().map(|pat| self.lower_pattern(pat));
        self.hir.catch_clause(clause.span, param, body)
    }

    fn lower_while_statement(&mut self, stmt: &ast::WhileStatement<'a>) -> hir::Statement<'a> {
        todo!()
    }

    fn lower_with_statement(&mut self, stmt: &ast::WithStatement<'a>) -> hir::Statement<'a> {
        todo!()
    }

    fn lower_expression(&mut self, expr: &ast::Expression<'a>) -> hir::Expression<'a> {
        match expr {
            ast::Expression::BigintLiteral(lit) => {
                let lit = self.lower_bigint_literal(lit);
                self.hir.literal_bigint_expression(lit)
            }
            ast::Expression::BooleanLiteral(lit) => {
                let lit = self.lower_boolean_literal(lit);
                self.hir.literal_boolean_expression(lit)
            }
            ast::Expression::NullLiteral(lit) => {
                let lit = self.lower_null_literal(lit);
                self.hir.literal_null_expression(lit)
            }
            ast::Expression::NumberLiteral(lit) => {
                let lit = self.lower_number_literal(lit);
                self.hir.literal_number_expression(lit)
            }
            ast::Expression::RegExpLiteral(lit) => {
                let lit = self.lower_reg_expr_literal(lit);
                self.hir.literal_regexp_expression(lit)
            }
            ast::Expression::StringLiteral(lit) => {
                let lit = self.lower_string_literal(lit);
                self.hir.literal_string_expression(lit)
            }
            ast::Expression::TemplateLiteral(lit) => {
                let lit = self.lower_template_literal(lit);
                self.hir.literal_template_expression(lit)
            }
            ast::Expression::Identifier(ident) => {
                let lit = self.lower_identifier_reference(ident);
                self.hir.identifier_reference_expression(lit)
            }
            ast::Expression::MetaProperty(meta) => self.lower_meta_property(meta),
            ast::Expression::ArrayExpression(expr) => self.lower_array_expression(expr),
            ast::Expression::ArrowFunctionExpression(expr) => self.lower_arrow_expression(expr),
            ast::Expression::AssignmentExpression(expr) => self.lower_assignment_expression(expr),
            ast::Expression::AwaitExpression(expr) => self.lower_await_expression(expr),
            ast::Expression::BinaryExpression(expr) => self.lower_binary_expression(expr),
            ast::Expression::CallExpression(expr) => self.lower_call_expression(expr),
            ast::Expression::ChainExpression(expr) => self.lower_chain_expression(expr),
            ast::Expression::ClassExpression(expr) => self.lower_class_expression(expr),
            ast::Expression::ConditionalExpression(expr) => self.lower_conditional_expression(expr),
            ast::Expression::FunctionExpression(expr) => self.lower_function_expression(expr),
            ast::Expression::ImportExpression(expr) => self.lower_import_expression(expr),
            ast::Expression::LogicalExpression(expr) => self.lower_logical_expression(expr),
            ast::Expression::MemberExpression(expr) => self.lower_member_expression(expr),
            ast::Expression::NewExpression(expr) => self.lower_new_expression(expr),
            ast::Expression::ObjectExpression(expr) => self.lower_object_expression(expr),
            ast::Expression::ParenthesizedExpression(expr) => {
                self.lower_parenthesized_expression(expr)
            }
            ast::Expression::PrivateInExpression(expr) => self.lower_private_in_expression(expr),
            ast::Expression::SequenceExpression(expr) => self.lower_sequence_expression(expr),
            ast::Expression::TaggedTemplateExpression(expr) => {
                self.lower_tagged_template_expression(expr)
            }
            ast::Expression::ThisExpression(expr) => self.lower_this_expression(expr),
            ast::Expression::UnaryExpression(expr) => self.lower_unary_expression(expr),
            ast::Expression::UpdateExpression(expr) => self.lower_update_expression(expr),
            ast::Expression::YieldExpression(expr) => self.lower_yield_expression(expr),
            ast::Expression::Super(expr) => self.lower_super(expr),
            ast::Expression::JSXElement(elem) => todo!(),
            ast::Expression::JSXFragment(elem) => todo!(),

            ast::Expression::TSAsExpression(expr) => todo!(),
            ast::Expression::TSSatisfiesExpression(expr) => todo!(),
            ast::Expression::TSNonNullExpression(expr) => todo!(),
            ast::Expression::TSTypeAssertion(expr) => todo!(),
            ast::Expression::TSInstantiationExpression(expr) => todo!(),
        }
    }

    fn lower_meta_property(&mut self, _meta: &ast::MetaProperty) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_array_expression(&mut self, expr: &ast::ArrayExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_array_expression_element(
        &mut self,
        elem: &ast::ArrayExpressionElement<'a>,
    ) -> hir::ArrayExpressionElement<'a> {
        todo!()
    }

    fn lower_argument(&mut self, arg: &ast::Argument<'a>) -> hir::Argument<'a> {
        todo!()
    }

    fn lower_spread_element(&mut self, elem: &ast::SpreadElement<'a>) -> hir::SpreadElement<'a> {
        todo!()
    }

    fn lower_assignment_expression(
        &mut self,
        expr: &ast::AssignmentExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_arrow_expression(&mut self, expr: &ast::ArrowExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_await_expression(&mut self, expr: &ast::AwaitExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_binary_expression(&mut self, expr: &ast::BinaryExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_call_expression(&mut self, expr: &ast::CallExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_chain_expression(&mut self, expr: &ast::ChainExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_chain_element(&mut self, elem: &ast::ChainElement<'a>) -> hir::ChainElement<'a> {
        todo!()
    }

    fn lower_class_expression(&mut self, class: &ast::Class<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_conditional_expression(
        &mut self,
        expr: &ast::ConditionalExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_function_expression(&mut self, func: &ast::Function<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_import_expression(&mut self, expr: &ast::ImportExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_logical_expression(
        &mut self,
        expr: &ast::LogicalExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_member_expression(&mut self, expr: &ast::MemberExpression<'a>) -> hir::Expression<'a> {
        match expr {
            ast::MemberExpression::ComputedMemberExpression(expr) => {
                todo!()
            }
            ast::MemberExpression::StaticMemberExpression(expr) => {
                todo!()
            }
            ast::MemberExpression::PrivateFieldExpression(expr) => {
                todo!()
            }
        }
    }

    fn lower_computed_member_expression(
        &mut self,
        expr: &ast::ComputedMemberExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_static_member_expression(
        &mut self,
        expr: &ast::StaticMemberExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_private_field_expression(
        &mut self,
        expr: &ast::PrivateFieldExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_new_expression(&mut self, expr: &ast::NewExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_object_expression(&mut self, expr: &ast::ObjectExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_object_property(&mut self, prop: &ast::ObjectProperty<'a>) -> hir::ObjectProperty<'a> {
        todo!()
    }

    fn lower_property(&mut self, prop: &ast::Property<'a>) -> hir::Property<'a> {
        todo!()
    }

    fn lower_property_key(&mut self, key: &ast::PropertyKey<'a>) -> hir::PropertyKey<'a> {
        todo!()
    }

    fn lower_property_value(&mut self, value: &ast::PropertyValue<'a>) -> hir::PropertyValue<'a> {
        todo!()
    }

    fn lower_parenthesized_expression(
        &mut self,
        expr: &ast::ParenthesizedExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_private_in_expression(
        &mut self,
        expr: &ast::PrivateInExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_sequence_expression(
        &mut self,
        expr: &ast::SequenceExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_tagged_template_expression(
        &mut self,
        expr: &ast::TaggedTemplateExpression<'a>,
    ) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_this_expression(&mut self, _expr: &ast::ThisExpression) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_unary_expression(&mut self, expr: &ast::UnaryExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_update_expression(&mut self, expr: &ast::UpdateExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_yield_expression(&mut self, expr: &ast::YieldExpression<'a>) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_super(&mut self, _expr: &ast::Super) -> hir::Expression<'a> {
        todo!()
    }

    fn lower_assignment_target(
        &mut self,
        target: &ast::AssignmentTarget<'a>,
    ) -> hir::AssignmentTarget<'a> {
        todo!()
    }

    fn lower_simple_assignment_target(
        &mut self,
        target: &ast::SimpleAssignmentTarget<'a>,
    ) -> hir::SimpleAssignmentTarget<'a> {
        todo!()
    }

    fn lower_assignment_target_pattern(
        &mut self,
        pat: &ast::AssignmentTargetPattern<'a>,
    ) -> hir::AssignmentPattern<'a> {
        todo!()
    }

    fn lower_array_assignment_target(
        &mut self,
        target: &ast::ArrayAssignmentTarget<'a>,
    ) -> hir::ArrayAssignmentTarget<'a> {
        todo!()
    }

    fn lower_assignment_target_maybe_default(
        &mut self,
        target: &ast::AssignmentTargetMaybeDefault<'a>,
    ) -> hir::AssignmentTargetMaybeDefault<'a> {
        todo!()
    }

    fn lower_assignment_target_with_default(
        &mut self,
        target: &ast::AssignmentTargetWithDefault<'a>,
    ) -> hir::AssignmentTargetWithDefault<'a> {
        todo!()
    }

    fn lower_object_assignment_target(
        &mut self,
        target: &ast::ObjectAssignmentTarget<'a>,
    ) -> hir::ObjectAssignmentTarget<'a> {
        todo!()
    }

    fn lower_assignment_target_property(
        &mut self,
        property: &ast::AssignmentTargetProperty<'a>,
    ) -> hir::AssignmentTargetProperty<'a> {
        todo!()
    }

    fn lower_assignment_target_property_identifier(
        &mut self,
        ident: &ast::AssignmentTargetPropertyIdentifier<'a>,
    ) -> hir::AssignmentTargetPropertyIdentifier<'a> {
        todo!()
    }

    fn lower_assignment_target_property_property(
        &mut self,
        property: &ast::AssignmentTargetPropertyProperty<'a>,
    ) -> hir::AssignmentTargetPropertyProperty<'a> {
        todo!()
    }

    fn lower_jsx_element(&mut self, elem: &ast::JSXElement<'a>) {
        todo!()
    }

    fn lower_jsx_opening_element(&mut self, elem: &ast::JSXOpeningElement<'a>) {
        todo!()
    }

    fn lower_jsx_element_name(&mut self, __name: &ast::JSXElementName<'a>) {
        todo!()
    }

    fn lower_jsx_attribute_item(&mut self, item: &ast::JSXAttributeItem<'a>) {
        todo!()
    }

    fn lower_jsx_attribute(&mut self, attribute: &ast::JSXAttribute<'a>) {
        todo!()
    }

    fn lower_jsx_spread_attribute(&mut self, attribute: &ast::JSXSpreadAttribute<'a>) {
        todo!()
    }

    fn lower_jsx_attribute_value(&mut self, value: &ast::JSXAttributeValue<'a>) {
        todo!()
    }

    fn lower_jsx_expression_container(&mut self, expr: &ast::JSXExpressionContainer<'a>) {
        todo!()
    }

    fn lower_jsx_expression(&mut self, expr: &ast::JSXExpression<'a>) {
        todo!()
    }

    fn lower_jsx_fragment(&mut self, elem: &ast::JSXFragment<'a>) {
        todo!()
    }

    fn lower_jsx_child(&mut self, child: &ast::JSXChild<'a>) {
        todo!()
    }

    fn lower_jsx_spread_child(&mut self, child: &ast::JSXSpreadChild<'a>) {
        todo!()
    }

    /* ----------  Pattern ---------- */

    fn lower_pattern(&mut self, pat: &ast::BindingPattern<'a>) -> hir::BindingPattern<'a> {
        todo!()
    }

    fn lower_object_pattern(&mut self, pat: &ast::ObjectPattern<'a>) -> hir::ObjectPattern<'a> {
        todo!()
    }

    fn lower_object_pattern_property(
        &mut self,
        prop: &ast::ObjectPatternProperty<'a>,
    ) -> hir::ObjectPatternProperty<'a> {
        todo!()
    }

    fn lower_array_pattern(&mut self, pat: &ast::ArrayPattern<'a>) -> hir::ArrayPattern<'a> {
        todo!()
    }

    fn lower_rest_element(&mut self, pat: &ast::RestElement<'a>) -> hir::RestElement<'a> {
        todo!()
    }

    fn lower_assignment_pattern(
        &mut self,
        pat: &ast::AssignmentPattern<'a>,
    ) -> hir::AssignmentPattern<'a> {
        todo!()
    }

    /* ----------  Identifier ---------- */

    fn lower_identifier_reference(
        &mut self,
        _ident: &ast::IdentifierReference,
    ) -> hir::IdentifierReference {
        todo!()
    }

    fn lower_private_identifier(
        &mut self,
        _ident: &ast::PrivateIdentifier,
    ) -> hir::PrivateIdentifier {
        todo!()
    }

    fn lower_label_identifier(&mut self, ident: &ast::LabelIdentifier) -> hir::LabelIdentifier {
        self.hir.label_identifier(ident.span, ident.name.clone())
    }

    fn lower_identifier_name(&mut self, _ident: &ast::IdentifierName) -> hir::IdentifierName {
        todo!()
    }

    fn lower_binding_identifier(
        &mut self,
        _ident: &ast::BindingIdentifier,
    ) -> hir::BindingIdentifier {
        todo!()
    }

    /* ----------  Literal ---------- */

    fn lower_number_literal(&mut self, _lit: &ast::NumberLiteral<'a>) -> hir::NumberLiteral<'a> {
        todo!()
    }

    fn lower_boolean_literal(&mut self, _lit: &ast::BooleanLiteral) -> hir::BooleanLiteral {
        todo!()
    }

    fn lower_null_literal(&mut self, _lit: &ast::NullLiteral) -> hir::NullLiteral {
        todo!()
    }

    fn lower_bigint_literal(&mut self, _lit: &ast::BigintLiteral) -> hir::BigintLiteral {
        todo!()
    }

    fn lower_string_literal(&mut self, _lit: &ast::StringLiteral) -> hir::StringLiteral {
        todo!()
    }

    fn lower_template_literal(
        &mut self,
        lit: &ast::TemplateLiteral<'a>,
    ) -> hir::TemplateLiteral<'a> {
        todo!()
    }

    fn lower_reg_expr_literal(&mut self, _lit: &ast::RegExpLiteral) -> hir::RegExpLiteral {
        todo!()
    }

    fn lower_template_element(&mut self, _elem: &ast::TemplateElement) -> hir::TemplateElement {
        todo!()
    }

    /* ----------  Module ---------- */

    fn lower_module_declaration(
        &mut self,
        decl: &ast::ModuleDeclaration<'a>,
    ) -> hir::ModuleDeclaration<'a> {
        todo!()
    }

    fn lower_import_declaration(
        &mut self,
        decl: &ast::ImportDeclaration<'a>,
    ) -> hir::ImportDeclaration<'a> {
        todo!()
    }

    fn lower_import_declaration_specifier(
        &mut self,
        specifier: &ast::ImportDeclarationSpecifier,
    ) -> hir::ImportDeclarationSpecifier {
        todo!()
    }

    fn lower_import_specifier(&mut self, specifier: &ast::ImportSpecifier) -> hir::ImportSpecifier {
        todo!()
    }

    fn lower_import_default_specifier(
        &mut self,
        specifier: &ast::ImportDefaultSpecifier,
    ) -> hir::ImportDefaultSpecifier {
        todo!()
    }

    fn lower_import_name_specifier(
        &mut self,
        specifier: &ast::ImportNamespaceSpecifier,
    ) -> hir::ImportNamespaceSpecifier {
        todo!()
    }

    fn lower_export_all_declaration(
        &mut self,
        _decl: &ast::ExportAllDeclaration<'a>,
    ) -> hir::ExportAllDeclaration<'a> {
        todo!()
    }

    fn lower_export_default_declaration(
        &mut self,
        decl: &ast::ExportDefaultDeclaration<'a>,
    ) -> hir::ExportDefaultDeclaration<'a> {
        todo!()
    }

    fn lower_export_named_declaration(&mut self, decl: &ast::ExportNamedDeclaration<'a>) {
        todo!()
    }

    fn lower_declaration(&mut self, decl: &ast::Declaration<'a>) -> hir::Declaration<'a> {
        todo!()
    }
}
