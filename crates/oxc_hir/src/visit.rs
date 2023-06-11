//! Visitor Pattern
//!
//! See:
//! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
//! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)

use oxc_allocator::Vec;
use oxc_span::Span;

#[allow(clippy::wildcard_imports)]
use crate::{hir::*, hir_kind::HirKind};

/// Syntax tree traversal
pub trait Visit<'a>: Sized {
    fn enter_node(&mut self, _kind: HirKind<'a>) {}
    fn leave_node(&mut self, _kind: HirKind<'a>) {}

    fn visit_program(&mut self, program: &'a Program<'a>) {
        let kind = HirKind::Program(program);
        self.enter_node(kind);
        for directive in &program.directives {
            self.visit_directive(directive);
        }
        self.visit_statements(&program.body);
        self.leave_node(kind);
    }

    /* ----------  Statement ---------- */

    fn visit_statements(&mut self, stmts: &'a Vec<'a, Statement<'a>>) {
        for stmt in stmts {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &'a Statement<'a>) {
        self.visit_statement_match(stmt);
    }

    fn visit_statement_match(&mut self, stmt: &'a Statement<'a>) {
        match stmt {
            Statement::BlockStatement(stmt) => self.visit_block_statement(stmt),
            Statement::BreakStatement(stmt) => self.visit_break_statement(stmt),
            Statement::ContinueStatement(stmt) => self.visit_continue_statement(stmt),
            Statement::DebuggerStatement(stmt) => self.visit_debugger_statement(stmt),
            Statement::DoWhileStatement(stmt) => self.visit_do_while_statement(stmt),
            Statement::ExpressionStatement(stmt) => self.visit_expression_statement(stmt),
            Statement::ForInStatement(stmt) => self.visit_for_in_statement(stmt),
            Statement::ForOfStatement(stmt) => self.visit_for_of_statement(stmt),
            Statement::ForStatement(stmt) => self.visit_for_statement(stmt),
            Statement::IfStatement(stmt) => self.visit_if_statement(stmt),
            Statement::LabeledStatement(stmt) => self.visit_labeled_statement(stmt),
            Statement::ReturnStatement(stmt) => self.visit_return_statement(stmt),
            Statement::SwitchStatement(stmt) => self.visit_switch_statement(stmt),
            Statement::ThrowStatement(stmt) => self.visit_throw_statement(stmt),
            Statement::TryStatement(stmt) => self.visit_try_statement(stmt),
            Statement::WhileStatement(stmt) => self.visit_while_statement(stmt),
            Statement::WithStatement(stmt) => self.visit_with_statement(stmt),

            Statement::ModuleDeclaration(decl) => self.visit_module_declaration(decl),
            Statement::Declaration(decl) => self.visit_declaration(decl),
        }
    }

    fn visit_block_statement(&mut self, stmt: &'a BlockStatement<'a>) {
        let kind = HirKind::BlockStatement(stmt);
        self.enter_node(kind);
        self.visit_statements(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_break_statement(&mut self, stmt: &'a BreakStatement) {
        let kind = HirKind::BreakStatement(stmt);
        self.enter_node(kind);
        if let Some(break_target) = &stmt.label {
            self.visit_label_identifier(break_target);
        }
        self.leave_node(kind);
    }

    fn visit_continue_statement(&mut self, stmt: &'a ContinueStatement) {
        let kind = HirKind::ContinueStatement(stmt);
        self.enter_node(kind);
        if let Some(continue_target) = &stmt.label {
            self.visit_label_identifier(continue_target);
        }
        self.leave_node(kind);
    }

    fn visit_debugger_statement(&mut self, stmt: &'a DebuggerStatement) {
        let kind = HirKind::DebuggerStatement(stmt);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &'a DoWhileStatement<'a>) {
        let kind = HirKind::DoWhileStatement(stmt);
        self.enter_node(kind);
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.visit_expression(&stmt.test);
        self.leave_node(kind);
    }

    fn visit_expression_statement(&mut self, stmt: &'a ExpressionStatement<'a>) {
        let kind = HirKind::ExpressionStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.expression);
        self.leave_node(kind);
    }

    fn visit_for_statement(&mut self, stmt: &'a ForStatement<'a>) {
        let kind = HirKind::ForStatement(stmt);
        self.enter_node(kind);
        if let Some(init) = &stmt.init {
            self.visit_for_statement_init(init);
        }
        if let Some(test) = &stmt.test {
            self.visit_expression(test);
        }
        if let Some(update) = &stmt.update {
            self.visit_expression(update);
        }
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.leave_node(kind);
    }

    fn visit_for_statement_init(&mut self, init: &'a ForStatementInit<'a>) {
        let kind = HirKind::ForStatementInit(init);
        self.enter_node(kind);
        match init {
            ForStatementInit::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            ForStatementInit::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_for_in_statement(&mut self, stmt: &'a ForInStatement<'a>) {
        let kind = HirKind::ForInStatement(stmt);
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);
        self.visit_expression(&stmt.right);
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.leave_node(kind);
    }

    fn visit_for_of_statement(&mut self, stmt: &'a ForOfStatement<'a>) {
        let kind = HirKind::ForOfStatement(stmt);
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);
        self.visit_expression(&stmt.right);
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.leave_node(kind);
    }

    fn visit_for_statement_left(&mut self, left: &'a ForStatementLeft<'a>) {
        match left {
            ForStatementLeft::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            ForStatementLeft::AssignmentTarget(target) => self.visit_assignment_target(target),
        }
    }

    fn visit_if_statement(&mut self, stmt: &'a IfStatement<'a>) {
        let kind = HirKind::IfStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.test);
        if let Some(stmt) = &stmt.consequent {
            self.visit_statement(stmt);
        }
        if let Some(alternate) = &stmt.alternate {
            self.visit_statement(alternate);
        }
        self.leave_node(kind);
    }

    fn visit_labeled_statement(&mut self, stmt: &'a LabeledStatement<'a>) {
        let kind = HirKind::LabeledStatement(stmt);
        self.enter_node(kind);
        self.visit_label_identifier(&stmt.label);
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.leave_node(kind);
    }

    fn visit_return_statement(&mut self, stmt: &'a ReturnStatement<'a>) {
        let kind = HirKind::ReturnStatement(stmt);
        self.enter_node(kind);
        if let Some(arg) = &stmt.argument {
            self.visit_expression(arg);
        }
        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &'a SwitchStatement<'a>) {
        let kind = HirKind::SwitchStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.discriminant);
        for case in &stmt.cases {
            self.visit_switch_case(case);
        }
        self.leave_node(kind);
    }

    fn visit_switch_case(&mut self, case: &'a SwitchCase<'a>) {
        let kind = HirKind::SwitchCase(case);
        self.enter_node(kind);
        if let Some(expr) = &case.test {
            self.visit_expression(expr);
        }
        self.visit_statements(&case.consequent);
        self.leave_node(kind);
    }

    fn visit_throw_statement(&mut self, stmt: &'a ThrowStatement<'a>) {
        let kind = HirKind::ThrowStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.argument);
        self.leave_node(kind);
    }

    fn visit_try_statement(&mut self, stmt: &'a TryStatement<'a>) {
        let kind = HirKind::TryStatement(stmt);
        self.enter_node(kind);
        self.visit_block_statement(&stmt.block);
        if let Some(handler) = &stmt.handler {
            self.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &stmt.finalizer {
            self.visit_finally_clause(finalizer);
        }
        self.leave_node(kind);
    }

    fn visit_catch_clause(&mut self, clause: &'a CatchClause<'a>) {
        let kind = HirKind::CatchClause(clause);
        self.enter_node(kind);
        if let Some(param) = &clause.param {
            self.visit_binding_pattern(param);
        }
        self.visit_statements(&clause.body.body);
        self.leave_node(kind);
    }

    fn visit_finally_clause(&mut self, clause: &'a BlockStatement<'a>) {
        let kind = HirKind::FinallyClause(clause);
        self.enter_node(kind);
        self.visit_block_statement(clause);
        self.leave_node(kind);
    }

    fn visit_while_statement(&mut self, stmt: &'a WhileStatement<'a>) {
        let kind = HirKind::WhileStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.test);
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.leave_node(kind);
    }

    fn visit_with_statement(&mut self, stmt: &'a WithStatement<'a>) {
        let kind = HirKind::WithStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.object);
        if let Some(stmt) = &stmt.body {
            self.visit_statement(stmt);
        }
        self.leave_node(kind);
    }

    fn visit_directive(&mut self, directive: &'a Directive<'a>) {
        let kind = HirKind::Directive(directive);
        self.enter_node(kind);
        self.visit_string_literal(&directive.expression);
        self.leave_node(kind);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &'a VariableDeclaration<'a>) {
        let kind = HirKind::VariableDeclaration(decl);
        self.enter_node(kind);
        for declarator in &decl.declarations {
            self.visit_variable_declarator(declarator);
        }
        self.leave_node(kind);
    }

    fn visit_variable_declarator(&mut self, declarator: &'a VariableDeclarator<'a>) {
        let kind = HirKind::VariableDeclarator(declarator);
        self.enter_node(kind);
        self.visit_binding_pattern(&declarator.id);
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.leave_node(kind);
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &'a Function<'a>) {
        let kind = HirKind::Function(func);
        self.enter_node(kind);
        if let Some(ident) = &func.id {
            self.visit_binding_identifier(ident);
        }
        self.visit_formal_parameters(&func.params);
        if let Some(body) = &func.body {
            self.visit_function_body(body);
        }
        self.leave_node(kind);
    }

    fn visit_function_body(&mut self, body: &'a FunctionBody<'a>) {
        let kind = HirKind::FunctionBody(body);
        self.enter_node(kind);
        for directive in &body.directives {
            self.visit_directive(directive);
        }
        self.visit_statements(&body.statements);
        self.leave_node(kind);
    }

    fn visit_formal_parameters(&mut self, params: &'a FormalParameters<'a>) {
        let kind = HirKind::FormalParameters(params);
        self.enter_node(kind);
        for param in &params.items {
            self.visit_formal_parameter(param);
        }
        if let Some(rest) = &params.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_formal_parameter(&mut self, param: &'a FormalParameter<'a>) {
        let kind = HirKind::FormalParameter(param);
        self.enter_node(kind);
        for decorator in &param.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_binding_pattern(&param.pattern);
        self.leave_node(kind);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &'a Decorator<'a>) {
        let kind = HirKind::Decorator(decorator);
        self.enter_node(kind);
        self.visit_expression(&decorator.expression);
        self.leave_node(kind);
    }

    fn visit_class(&mut self, class: &'a Class<'a>) {
        // Class level decorators are transpiled as functions outside of the class taking the class
        // itself as argument. They should be visited before class is entered. E.g., they inherit
        // strict mode from the enclosing scope rather than from class.
        for decorator in &class.decorators {
            self.visit_decorator(decorator);
        }
        let kind = HirKind::Class(class);
        self.enter_node(kind);
        if let Some(id) = &class.id {
            self.visit_binding_identifier(id);
        }
        if let Some(super_class) = &class.super_class {
            self.visit_class_heritage(super_class);
        }
        self.visit_class_body(&class.body);
        self.leave_node(kind);
    }

    fn visit_class_heritage(&mut self, expr: &'a Expression<'a>) {
        let kind = HirKind::ClassHeritage(expr);
        self.enter_node(kind);
        self.visit_expression(expr);
        self.leave_node(kind);
    }

    fn visit_class_body(&mut self, body: &'a ClassBody<'a>) {
        for elem in &body.body {
            self.visit_class_element(elem);
        }
    }

    fn visit_class_element(&mut self, elem: &'a ClassElement<'a>) {
        match elem {
            ClassElement::StaticBlock(block) => self.visit_static_block(block),
            ClassElement::MethodDefinition(def) => self.visit_method_definition(def),
            ClassElement::PropertyDefinition(def) => self.visit_property_definition(def),
            ClassElement::AccessorProperty(_def) => { /* TODO */ }
        }
    }

    fn visit_static_block(&mut self, block: &'a StaticBlock<'a>) {
        let kind = HirKind::StaticBlock(block);
        self.enter_node(kind);
        self.visit_statements(&block.body);
        self.leave_node(kind);
    }

    fn visit_method_definition(&mut self, def: &'a MethodDefinition<'a>) {
        let kind = HirKind::MethodDefinition(def);
        self.enter_node(kind);
        for decorator in &def.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_property_key(&def.key);
        self.visit_function(&def.value);
        self.leave_node(kind);
    }

    fn visit_property_definition(&mut self, def: &'a PropertyDefinition<'a>) {
        let kind = HirKind::PropertyDefinition(def);
        self.enter_node(kind);
        for decorator in &def.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_property_key(&def.key);
        if let Some(value) = &def.value {
            self.visit_expression(value);
        }
        self.leave_node(kind);
    }

    /* ----------  Expression ---------- */

    fn visit_expression(&mut self, expr: &'a Expression<'a>) {
        self.visit_expression_match(expr);
    }

    fn visit_expression_match(&mut self, expr: &'a Expression<'a>) {
        match expr {
            Expression::BigintLiteral(lit) => self.visit_bigint_literal(lit),
            Expression::BooleanLiteral(lit) => self.visit_boolean_literal(lit),
            Expression::NullLiteral(lit) => self.visit_null_literal(lit),
            Expression::NumberLiteral(lit) => self.visit_number_literal(lit),
            Expression::RegExpLiteral(lit) => self.visit_reg_expr_literal(lit),
            Expression::StringLiteral(lit) => self.visit_string_literal(lit),
            Expression::TemplateLiteral(lit) => self.visit_template_literal(lit),

            Expression::Identifier(ident) => self.visit_identifier_reference(ident),
            Expression::MetaProperty(meta) => self.visit_meta_property(meta),

            Expression::ArrayExpression(expr) => self.visit_array_expression(expr),
            Expression::ArrowExpression(expr) => self.visit_arrow_expression(expr),
            Expression::AssignmentExpression(expr) => self.visit_assignment_expression(expr),
            Expression::AwaitExpression(expr) => self.visit_await_expression(expr),
            Expression::BinaryExpression(expr) => self.visit_binary_expression(expr),
            Expression::CallExpression(expr) => self.visit_call_expression(expr),
            Expression::ChainExpression(expr) => self.visit_chain_expression(expr),
            Expression::ClassExpression(expr) => self.visit_class(expr),
            Expression::ConditionalExpression(expr) => self.visit_conditional_expression(expr),
            Expression::FunctionExpression(expr) => self.visit_function(expr),
            Expression::ImportExpression(expr) => self.visit_import_expression(expr),
            Expression::LogicalExpression(expr) => self.visit_logical_expression(expr),
            Expression::MemberExpression(expr) => self.visit_member_expression(expr),
            Expression::NewExpression(expr) => self.visit_new_expression(expr),
            Expression::ObjectExpression(expr) => self.visit_object_expression(expr),
            Expression::PrivateInExpression(expr) => self.visit_private_in_expression(expr),
            Expression::SequenceExpression(expr) => self.visit_sequence_expression(expr),
            Expression::TaggedTemplateExpression(expr) => {
                self.visit_tagged_template_expression(expr);
            }
            Expression::ThisExpression(expr) => self.visit_this_expression(expr),
            Expression::UnaryExpression(expr) => self.visit_unary_expression(expr),
            Expression::UpdateExpression(expr) => self.visit_update_expression(expr),
            Expression::YieldExpression(expr) => self.visit_yield_expression(expr),
            Expression::Super(expr) => self.visit_super(expr),
            Expression::JSXElement(elem) => self.visit_jsx_element(elem),
            Expression::JSXFragment(elem) => self.visit_jsx_fragment(elem),
        }
    }

    fn visit_meta_property(&mut self, meta: &'a MetaProperty) {
        let kind = HirKind::MetaProperty(meta);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_array_expression(&mut self, expr: &'a ArrayExpression<'a>) {
        let kind = HirKind::ArrayExpression(expr);
        self.enter_node(kind);
        for elem in expr.elements.iter() {
            self.visit_array_expression_element(elem);
        }
        self.leave_node(kind);
    }

    fn visit_array_expression_element(&mut self, arg: &'a ArrayExpressionElement<'a>) {
        let kind = HirKind::ArrayExpressionElement(arg);
        self.enter_node(kind);
        match arg {
            ArrayExpressionElement::SpreadElement(spread) => self.visit_spread_element(spread),
            ArrayExpressionElement::Expression(expr) => self.visit_expression(expr),
            ArrayExpressionElement::Elision(span) => self.visit_elision(*span),
        }
        self.leave_node(kind);
    }

    fn visit_argument(&mut self, arg: &'a Argument<'a>) {
        let kind = HirKind::Argument(arg);
        self.enter_node(kind);
        match arg {
            Argument::SpreadElement(spread) => self.visit_spread_element(spread),
            Argument::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_spread_element(&mut self, elem: &'a SpreadElement<'a>) {
        let kind = HirKind::SpreadElement(elem);
        self.enter_node(kind);
        self.visit_expression(&elem.argument);
        self.leave_node(kind);
    }

    fn visit_elision(&mut self, span: Span) {
        let kind = HirKind::Elision(span);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_assignment_expression(&mut self, expr: &'a AssignmentExpression<'a>) {
        let kind = HirKind::AssignmentExpression(expr);
        self.enter_node(kind);
        self.visit_assignment_target(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_arrow_expression(&mut self, expr: &'a ArrowExpression<'a>) {
        let kind = HirKind::ArrowExpression(expr);
        self.enter_node(kind);
        self.visit_formal_parameters(&expr.params);
        self.visit_function_body(&expr.body);
        self.leave_node(kind);
    }

    fn visit_await_expression(&mut self, expr: &'a AwaitExpression<'a>) {
        let kind = HirKind::AwaitExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_binary_expression(&mut self, expr: &'a BinaryExpression<'a>) {
        let kind = HirKind::BinaryExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_call_expression(&mut self, expr: &'a CallExpression<'a>) {
        let kind = HirKind::CallExpression(expr);
        self.enter_node(kind);
        for arg in &expr.arguments {
            self.visit_argument(arg);
        }
        self.visit_expression(&expr.callee);
        self.leave_node(kind);
    }

    fn visit_chain_expression(&mut self, expr: &'a ChainExpression<'a>) {
        self.visit_chain_element(&expr.expression);
    }

    fn visit_chain_element(&mut self, elem: &'a ChainElement<'a>) {
        match elem {
            ChainElement::CallExpression(expr) => self.visit_call_expression(expr),
            ChainElement::MemberExpression(expr) => self.visit_member_expression(expr),
        }
    }

    fn visit_conditional_expression(&mut self, expr: &'a ConditionalExpression<'a>) {
        let kind = HirKind::ConditionalExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.test);
        self.visit_expression(&expr.consequent);
        self.visit_expression(&expr.alternate);
        self.leave_node(kind);
    }

    fn visit_import_expression(&mut self, expr: &'a ImportExpression<'a>) {
        self.visit_expression(&expr.source);
        for arg in &expr.arguments {
            self.visit_expression(arg);
        }
    }

    fn visit_logical_expression(&mut self, expr: &'a LogicalExpression<'a>) {
        let kind = HirKind::LogicalExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_member_expression(&mut self, expr: &'a MemberExpression<'a>) {
        let kind = HirKind::MemberExpression(expr);
        self.enter_node(kind);
        match expr {
            MemberExpression::ComputedMemberExpression(expr) => {
                self.visit_computed_member_expression(expr);
            }
            MemberExpression::StaticMemberExpression(expr) => {
                self.visit_static_member_expression(expr);
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                self.visit_private_field_expression(expr);
            }
        }
        self.leave_node(kind);
    }

    fn visit_computed_member_expression(&mut self, expr: &'a ComputedMemberExpression<'a>) {
        self.visit_expression(&expr.object);
        self.visit_expression(&expr.expression);
    }

    fn visit_static_member_expression(&mut self, expr: &'a StaticMemberExpression<'a>) {
        self.visit_expression(&expr.object);
        self.visit_identifier_name(&expr.property);
    }

    fn visit_private_field_expression(&mut self, expr: &'a PrivateFieldExpression<'a>) {
        self.visit_expression(&expr.object);
        self.visit_private_identifier(&expr.field);
    }

    fn visit_new_expression(&mut self, expr: &'a NewExpression<'a>) {
        let kind = HirKind::NewExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            self.visit_argument(arg);
        }
        self.leave_node(kind);
    }

    fn visit_object_expression(&mut self, expr: &'a ObjectExpression<'a>) {
        let kind = HirKind::ObjectExpression(expr);
        self.enter_node(kind);
        for prop in &expr.properties {
            self.visit_object_property_kind(prop);
        }
        self.leave_node(kind);
    }

    fn visit_object_property_kind(&mut self, prop: &'a ObjectPropertyKind<'a>) {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => self.visit_object_property(prop),
            ObjectPropertyKind::SpreadProperty(elem) => self.visit_spread_element(elem),
        }
    }

    fn visit_object_property(&mut self, prop: &'a ObjectProperty<'a>) {
        let kind = HirKind::ObjectProperty(prop);
        self.enter_node(kind);
        self.visit_property_key(&prop.key);
        self.visit_expression(&prop.value);
        self.leave_node(kind);
    }

    fn visit_property_key(&mut self, key: &'a PropertyKey<'a>) {
        let kind = HirKind::PropertyKey(key);
        self.enter_node(kind);
        match key {
            PropertyKey::Identifier(ident) => self.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => self.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_private_in_expression(&mut self, expr: &'a PrivateInExpression<'a>) {
        self.visit_private_identifier(&expr.left);
        self.visit_expression(&expr.right);
    }

    fn visit_sequence_expression(&mut self, expr: &'a SequenceExpression<'a>) {
        let kind = HirKind::SequenceExpression(expr);
        self.enter_node(kind);
        for expr in &expr.expressions {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_tagged_template_expression(&mut self, expr: &'a TaggedTemplateExpression<'a>) {
        let kind = HirKind::TaggedTemplateExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.tag);
        self.visit_template_literal(&expr.quasi);
        self.leave_node(kind);
    }

    fn visit_this_expression(&mut self, expr: &'a ThisExpression) {
        let kind = HirKind::ThisExpression(expr);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_unary_expression(&mut self, expr: &'a UnaryExpression<'a>) {
        let kind = HirKind::UnaryExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_update_expression(&mut self, expr: &'a UpdateExpression<'a>) {
        let kind = HirKind::UpdateExpression(expr);
        self.enter_node(kind);
        self.visit_simple_assignment_target(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_yield_expression(&mut self, expr: &'a YieldExpression<'a>) {
        let kind = HirKind::YieldExpression(expr);
        self.enter_node(kind);
        if let Some(argument) = &expr.argument {
            self.visit_expression(argument);
        }
        self.leave_node(kind);
    }

    fn visit_super(&mut self, expr: &'a Super) {
        let kind = HirKind::Super(expr);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_assignment_target(&mut self, target: &'a AssignmentTarget<'a>) {
        let kind = HirKind::AssignmentTarget(target);
        self.enter_node(kind);
        match target {
            AssignmentTarget::SimpleAssignmentTarget(target) => {
                self.visit_simple_assignment_target(target);
            }
            AssignmentTarget::AssignmentTargetPattern(pat) => {
                self.visit_assignment_target_pattern(pat);
            }
        }
        self.leave_node(kind);
    }

    fn visit_simple_assignment_target(&mut self, target: &'a SimpleAssignmentTarget<'a>) {
        let kind = HirKind::SimpleAssignmentTarget(target);
        self.enter_node(kind);
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.visit_identifier_reference(ident);
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
                self.visit_member_expression(expr);
            }
        }
        self.leave_node(kind);
    }

    fn visit_assignment_target_pattern(&mut self, pat: &'a AssignmentTargetPattern<'a>) {
        match pat {
            AssignmentTargetPattern::ArrayAssignmentTarget(target) => {
                self.visit_array_assignment_target(target);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(target) => {
                self.visit_object_assignment_target(target);
            }
        }
    }

    fn visit_array_assignment_target(&mut self, target: &'a ArrayAssignmentTarget<'a>) {
        for element in target.elements.iter().flatten() {
            self.visit_assignment_target_maybe_default(element);
        }
        if let Some(target) = &target.rest {
            self.visit_assignment_target(target);
        }
    }

    fn visit_assignment_target_maybe_default(
        &mut self,
        target: &'a AssignmentTargetMaybeDefault<'a>,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTarget(target) => {
                self.visit_assignment_target(target);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                self.visit_assignment_target_with_default(target);
            }
        }
    }

    fn visit_assignment_target_with_default(
        &mut self,
        target: &'a AssignmentTargetWithDefault<'a>,
    ) {
        let kind = HirKind::AssignmentTargetWithDefault(target);
        self.enter_node(kind);
        self.visit_assignment_target(&target.binding);
        self.visit_expression(&target.init);
        self.leave_node(kind);
    }

    fn visit_object_assignment_target(&mut self, target: &'a ObjectAssignmentTarget<'a>) {
        for property in &target.properties {
            self.visit_assignment_target_property(property);
        }
        if let Some(target) = &target.rest {
            self.visit_assignment_target(target);
        }
    }

    fn visit_assignment_target_property(&mut self, property: &'a AssignmentTargetProperty<'a>) {
        match property {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                self.visit_assignment_target_property_identifier(ident);
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                self.visit_assignment_target_property_property(prop);
            }
        }
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        ident: &'a AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.visit_identifier_reference(&ident.binding);
        if let Some(expr) = &ident.init {
            self.visit_expression(expr);
        }
    }

    fn visit_assignment_target_property_property(
        &mut self,
        property: &'a AssignmentTargetPropertyProperty<'a>,
    ) {
        self.visit_property_key(&property.name);
        self.visit_assignment_target_maybe_default(&property.binding);
    }

    /* ----------  Expression ---------- */

    fn visit_jsx_element(&mut self, elem: &'a JSXElement<'a>) {
        self.visit_jsx_opening_element(&elem.opening_element);
        for child in &elem.children {
            self.visit_jsx_child(child);
        }
    }

    fn visit_jsx_opening_element(&mut self, elem: &'a JSXOpeningElement<'a>) {
        let kind = HirKind::JSXOpeningElement(elem);
        self.enter_node(kind);
        self.visit_jsx_element_name(&elem.name);
        for attribute in &elem.attributes {
            self.visit_jsx_attribute_item(attribute);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_element_name(&mut self, name: &'a JSXElementName<'a>) {
        let kind = HirKind::JSXElementName(name);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_jsx_attribute_item(&mut self, item: &'a JSXAttributeItem<'a>) {
        match &item {
            JSXAttributeItem::Attribute(attribute) => self.visit_jsx_attribute(attribute),
            JSXAttributeItem::SpreadAttribute(attribute) => {
                self.visit_jsx_spread_attribute(attribute);
            }
        }
    }

    fn visit_jsx_attribute(&mut self, attribute: &'a JSXAttribute<'a>) {
        if let Some(value) = &attribute.value {
            self.visit_jsx_attribute_value(value);
        }
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &'a JSXSpreadAttribute<'a>) {
        self.visit_expression(&attribute.argument);
    }

    fn visit_jsx_attribute_value(&mut self, value: &'a JSXAttributeValue<'a>) {
        match value {
            JSXAttributeValue::ExpressionContainer(expr) => {
                self.visit_jsx_expression_container(expr);
            }
            JSXAttributeValue::Element(elem) => self.visit_jsx_element(elem),
            JSXAttributeValue::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXAttributeValue::StringLiteral(_) => {}
        }
    }

    fn visit_jsx_expression_container(&mut self, expr: &'a JSXExpressionContainer<'a>) {
        self.visit_jsx_expression(&expr.expression);
    }

    fn visit_jsx_expression(&mut self, expr: &'a JSXExpression<'a>) {
        match expr {
            JSXExpression::Expression(expr) => self.visit_expression(expr),
            JSXExpression::EmptyExpression(_) => {}
        }
    }

    fn visit_jsx_fragment(&mut self, elem: &'a JSXFragment<'a>) {
        for child in &elem.children {
            self.visit_jsx_child(child);
        }
    }

    fn visit_jsx_child(&mut self, child: &'a JSXChild<'a>) {
        match child {
            JSXChild::Element(elem) => self.visit_jsx_element(elem),
            JSXChild::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXChild::ExpressionContainer(expr) => self.visit_jsx_expression_container(expr),
            JSXChild::Spread(expr) => self.visit_jsx_spread_child(expr),
            JSXChild::Text(_) => {}
        }
    }

    fn visit_jsx_spread_child(&mut self, child: &'a JSXSpreadChild<'a>) {
        self.visit_expression(&child.expression);
    }

    /* ----------  Pattern ---------- */

    fn visit_binding_pattern(&mut self, pat: &'a BindingPattern<'a>) {
        match pat {
            BindingPattern::BindingIdentifier(ident) => {
                self.visit_binding_identifier(ident);
            }
            BindingPattern::ObjectPattern(pat) => self.visit_object_pattern(pat),
            BindingPattern::ArrayPattern(pat) => self.visit_array_pattern(pat),
            BindingPattern::AssignmentPattern(pat) => self.visit_assignment_pattern(pat),
            BindingPattern::RestElement(elem) => self.visit_rest_element(elem),
        }
    }

    fn visit_binding_identifier(&mut self, ident: &'a BindingIdentifier) {
        let kind = HirKind::BindingIdentifier(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_object_pattern(&mut self, pat: &'a ObjectPattern<'a>) {
        let kind = HirKind::ObjectPattern(pat);
        self.enter_node(kind);
        for prop in &pat.properties {
            self.visit_binding_property(prop);
        }
        if let Some(rest) = &pat.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_binding_property(&mut self, prop: &'a BindingProperty<'a>) {
        self.visit_property_key(&prop.key);
        self.visit_binding_pattern(&prop.value);
    }

    fn visit_array_pattern(&mut self, pat: &'a ArrayPattern<'a>) {
        let kind = HirKind::ArrayPattern(pat);
        self.enter_node(kind);
        for pat in pat.elements.iter().flatten() {
            self.visit_binding_pattern(pat);
        }
        if let Some(rest) = &pat.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_rest_element(&mut self, pat: &'a RestElement<'a>) {
        let kind = HirKind::RestElement(pat);
        self.enter_node(kind);
        self.visit_binding_pattern(&pat.argument);
        self.leave_node(kind);
    }

    fn visit_assignment_pattern(&mut self, pat: &'a AssignmentPattern<'a>) {
        let kind = HirKind::AssignmentPattern(pat);
        self.enter_node(kind);
        self.visit_binding_pattern(&pat.left);
        self.visit_expression(&pat.right);
        self.leave_node(kind);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, ident: &'a IdentifierReference) {
        let kind = HirKind::IdentifierReference(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_private_identifier(&mut self, ident: &'a PrivateIdentifier) {
        let kind = HirKind::PrivateIdentifier(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_label_identifier(&mut self, ident: &'a LabelIdentifier) {
        let kind = HirKind::LabelIdentifier(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_identifier_name(&mut self, ident: &'a IdentifierName) {
        let kind = HirKind::IdentifierName(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, lit: &'a NumberLiteral<'a>) {
        let kind = HirKind::NumberLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_boolean_literal(&mut self, lit: &'a BooleanLiteral) {
        let kind = HirKind::BooleanLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_null_literal(&mut self, lit: &'a NullLiteral) {
        let kind = HirKind::NullLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_bigint_literal(&mut self, lit: &'a BigintLiteral) {
        let kind = HirKind::BigintLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_string_literal(&mut self, lit: &'a StringLiteral) {
        let kind = HirKind::StringLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_literal(&mut self, lit: &'a TemplateLiteral<'a>) {
        let kind = HirKind::TemplateLiteral(lit);
        self.enter_node(kind);
        for elem in &lit.quasis {
            self.visit_template_element(elem);
        }
        for expr in &lit.expressions {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_reg_expr_literal(&mut self, lit: &'a RegExpLiteral) {
        let kind = HirKind::RegExpLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_element(&mut self, _elem: &'a TemplateElement) {}

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &'a ModuleDeclaration<'a>) {
        let kind = HirKind::ModuleDeclaration(decl);
        self.enter_node(kind);
        match decl {
            ModuleDeclaration::ImportDeclaration(decl) => {
                self.visit_import_declaration(decl);
            }
            ModuleDeclaration::ExportAllDeclaration(decl) => {
                self.visit_export_all_declaration(decl);
            }
            ModuleDeclaration::ExportDefaultDeclaration(decl) => {
                self.visit_export_default_declaration(decl);
            }
            ModuleDeclaration::ExportNamedDeclaration(decl) => {
                self.visit_export_named_declaration(decl);
            }
        }
        self.leave_node(kind);
    }

    fn visit_import_declaration(&mut self, decl: &'a ImportDeclaration<'a>) {
        for specifier in &decl.specifiers {
            self.visit_import_declaration_specifier(specifier);
        }
        // TODO: source
        // TODO: assertions
    }

    fn visit_import_declaration_specifier(&mut self, specifier: &'a ImportDeclarationSpecifier) {
        match &specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                self.visit_import_specifier(specifier);
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                self.visit_import_default_specifier(specifier);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                self.visit_import_name_specifier(specifier);
            }
        }
    }

    fn visit_import_specifier(&mut self, specifier: &'a ImportSpecifier) {
        // TODO: imported
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_import_default_specifier(&mut self, specifier: &'a ImportDefaultSpecifier) {
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_import_name_specifier(&mut self, specifier: &'a ImportNamespaceSpecifier) {
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_export_all_declaration(&mut self, _decl: &'a ExportAllDeclaration<'a>) {}

    fn visit_export_default_declaration(&mut self, decl: &'a ExportDefaultDeclaration<'a>) {
        match &decl.declaration {
            ExportDefaultDeclarationKind::Expression(expr) => self.visit_expression(expr),
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                self.visit_function(func);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => self.visit_class(class),
            ExportDefaultDeclarationKind::TSEnumDeclaration(decl) => self.visit_enum(decl),
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &'a ExportNamedDeclaration<'a>) {
        if let Some(decl) = &decl.declaration {
            self.visit_declaration(decl);
        }
    }

    fn visit_enum_member(&mut self, member: &'a TSEnumMember<'a>) {
        let kind = HirKind::TSEnumMember(member);
        self.enter_node(kind);

        if let Some(initializer) = &member.initializer {
            self.visit_expression(initializer);
        }

        self.leave_node(kind);
    }

    fn visit_enum(&mut self, decl: &'a TSEnumDeclaration<'a>) {
        let kind = HirKind::TSEnumDeclaration(decl);
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        for member in &decl.members {
            self.visit_enum_member(member);
        }
        self.leave_node(kind);
    }

    fn visit_declaration(&mut self, decl: &'a Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            Declaration::FunctionDeclaration(func) => self.visit_function(func),
            Declaration::ClassDeclaration(class) => self.visit_class(class),
            Declaration::TSEnumDeclaration(decl) => self.visit_enum(decl),
        }
    }
}
