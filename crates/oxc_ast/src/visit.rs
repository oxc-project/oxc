//! AST Visitor Pattern.
//! See:
//! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
//! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)

use oxc_allocator::Vec;

#[allow(clippy::wildcard_imports)]
use crate::{ast::*, ast_kind::AstKind};

pub trait Visit<'a>: Sized {
    fn enter_node(&mut self, _kind: AstKind<'a>) {}
    fn leave_node(&mut self, _kind: AstKind<'a>) {}

    fn visit_program(&mut self, program: &'a Program<'a>) {
        let kind = AstKind::Program(program);
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
            Statement::EmptyStatement(stmt) => self.visit_empty_statement(stmt),
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
        let kind = AstKind::BlockStatement(stmt);
        self.enter_node(kind);
        self.visit_statements(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_break_statement(&mut self, stmt: &'a BreakStatement) {
        let kind = AstKind::BreakStatement(stmt);
        self.enter_node(kind);
        if let Some(break_target) = &stmt.label {
            self.visit_label_identifier(break_target);
        }
        self.leave_node(kind);
    }

    fn visit_continue_statement(&mut self, stmt: &'a ContinueStatement) {
        let kind = AstKind::ContinueStatement(stmt);
        self.enter_node(kind);
        if let Some(continue_target) = &stmt.label {
            self.visit_label_identifier(continue_target);
        }
        self.leave_node(kind);
    }

    fn visit_debugger_statement(&mut self, stmt: &'a DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(stmt);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &'a DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(stmt);
        self.enter_node(kind);
        self.visit_statement(&stmt.body);
        self.visit_expression(&stmt.test);
        self.leave_node(kind);
    }

    fn visit_empty_statement(&mut self, stmt: &'a EmptyStatement) {
        let kind = AstKind::EmptyStatement(stmt);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_expression_statement(&mut self, stmt: &'a ExpressionStatement<'a>) {
        let kind = AstKind::ExpressionStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.expression);
        self.leave_node(kind);
    }

    fn visit_for_statement(&mut self, stmt: &'a ForStatement<'a>) {
        let kind = AstKind::ForStatement(stmt);
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
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_for_statement_init(&mut self, init: &'a ForStatementInit<'a>) {
        let kind = AstKind::ForStatementInit(init);
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
        let kind = AstKind::ForInStatement(stmt);
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);
        self.visit_expression(&stmt.right);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_for_of_statement(&mut self, stmt: &'a ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(stmt);
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);
        self.visit_expression(&stmt.right);
        self.visit_statement(&stmt.body);
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
        let kind = AstKind::IfStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.test);
        self.visit_statement(&stmt.consequent);
        if let Some(alternate) = &stmt.alternate {
            self.visit_statement(alternate);
        }
        self.leave_node(kind);
    }

    fn visit_labeled_statement(&mut self, stmt: &'a LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(stmt);
        self.enter_node(kind);
        self.visit_label_identifier(&stmt.label);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_return_statement(&mut self, stmt: &'a ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(stmt);
        self.enter_node(kind);
        if let Some(arg) = &stmt.argument {
            self.visit_expression(arg);
        }
        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &'a SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.discriminant);
        for case in &stmt.cases {
            self.visit_switch_case(case);
        }
        self.leave_node(kind);
    }

    fn visit_switch_case(&mut self, case: &'a SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(case);
        self.enter_node(kind);
        if let Some(expr) = &case.test {
            self.visit_expression(expr);
        }
        self.visit_statements(&case.consequent);
        self.leave_node(kind);
    }

    fn visit_throw_statement(&mut self, stmt: &'a ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.argument);
        self.leave_node(kind);
    }

    fn visit_try_statement(&mut self, stmt: &'a TryStatement<'a>) {
        let kind = AstKind::TryStatement(stmt);
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
        let kind = AstKind::CatchClause(clause);
        self.enter_node(kind);
        if let Some(param) = &clause.param {
            self.visit_pattern(param);
        }
        self.visit_statements(&clause.body.body);
        self.leave_node(kind);
    }

    fn visit_finally_clause(&mut self, clause: &'a BlockStatement<'a>) {
        let kind = AstKind::FinallyClause(clause);
        self.enter_node(kind);
        self.visit_block_statement(clause);
        self.leave_node(kind);
    }

    fn visit_while_statement(&mut self, stmt: &'a WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.test);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_with_statement(&mut self, stmt: &'a WithStatement<'a>) {
        let kind = AstKind::WithStatement(stmt);
        self.enter_node(kind);
        self.visit_expression(&stmt.object);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_directive(&mut self, directive: &'a Directive<'a>) {
        let kind = AstKind::Directive(directive);
        self.enter_node(kind);
        self.visit_string_literal(&directive.expression);
        self.leave_node(kind);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &'a VariableDeclaration<'a>) {
        let kind = AstKind::VariableDeclaration(decl);
        self.enter_node(kind);
        for declarator in &decl.declarations {
            self.visit_variable_declarator(declarator);
        }
        self.leave_node(kind);
    }

    fn visit_variable_declarator(&mut self, declarator: &'a VariableDeclarator<'a>) {
        let kind = AstKind::VariableDeclarator(declarator);
        self.enter_node(kind);
        self.visit_pattern(&declarator.id);
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.leave_node(kind);
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &'a Function<'a>) {
        let kind = AstKind::Function(func);
        self.enter_node(kind);
        if let Some(ident) = &func.id {
            self.visit_binding_identifier(ident);
        }
        self.visit_formal_parameters(&func.params);
        if let Some(body) = &func.body {
            self.visit_function_body(body);
        }
        if let Some(parameters) = &func.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &func.return_type {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
    }

    fn visit_function_body(&mut self, body: &'a FunctionBody<'a>) {
        let kind = AstKind::FunctionBody(body);
        self.enter_node(kind);
        for directive in &body.directives {
            self.visit_directive(directive);
        }
        self.visit_statements(&body.statements);
        self.leave_node(kind);
    }

    fn visit_formal_parameters(&mut self, params: &'a FormalParameters<'a>) {
        let kind = AstKind::FormalParameters(params);
        self.enter_node(kind);
        for param in &params.items {
            self.visit_formal_parameter(param);
        }
        self.leave_node(kind);
    }

    fn visit_formal_parameter(&mut self, param: &'a FormalParameter<'a>) {
        let kind = AstKind::FormalParameter(param);
        self.enter_node(kind);
        for decorator in &param.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_pattern(&param.pattern);
        self.leave_node(kind);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &'a Decorator<'a>) {
        let kind = AstKind::Decorator(decorator);
        self.enter_node(kind);
        self.visit_expression(&decorator.expression);
        self.leave_node(kind);
    }

    fn visit_class(&mut self, class: &'a Class<'a>) {
        let kind = AstKind::Class(class);
        self.enter_node(kind);
        for decorator in &class.decorators {
            self.visit_decorator(decorator);
        }
        if let Some(id) = &class.id {
            self.visit_binding_identifier(id);
        }
        if let Some(parameters) = &class.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(super_class) = &class.super_class {
            self.visit_class_heritage(super_class);
        }
        if let Some(super_parameters) = &class.super_type_parameters {
            self.visit_ts_type_parameter_instantiation(super_parameters);
        }
        self.visit_class_body(&class.body);
        self.leave_node(kind);
    }

    fn visit_class_heritage(&mut self, expr: &'a Expression<'a>) {
        let kind = AstKind::ClassHeritage(expr);
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
            ClassElement::TSAbstractMethodDefinition(def) => {
                self.visit_method_definition(&def.method_definition);
            }
            ClassElement::TSAbstractPropertyDefinition(def) => {
                self.visit_property_definition(&def.property_definition);
            }
            ClassElement::TSIndexSignature(_def) => {}
        }
    }

    fn visit_static_block(&mut self, block: &'a StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(block);
        self.enter_node(kind);
        self.visit_statements(&block.body);
        self.leave_node(kind);
    }

    fn visit_method_definition(&mut self, def: &'a MethodDefinition<'a>) {
        let kind = AstKind::MethodDefinition(def);
        self.enter_node(kind);
        for decorator in &def.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_property_key(&def.key);
        self.visit_function(&def.value);
        self.leave_node(kind);
    }

    fn visit_property_definition(&mut self, def: &'a PropertyDefinition<'a>) {
        let kind = AstKind::PropertyDefinition(def);
        self.enter_node(kind);
        for decorator in &def.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_property_key(&def.key);
        if let Some(value) = &def.value {
            self.visit_expression(value);
        }
        if let Some(annotation) = &def.type_annotation {
            self.visit_ts_type_annotation(annotation);
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
            Expression::ArrowFunctionExpression(expr) => self.visit_arrow_expression(expr),
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
            Expression::ParenthesizedExpression(expr) => {
                self.visit_parenthesized_expression(expr);
            }
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

            Expression::TSAsExpression(expr) => self.visit_ts_as_expression(expr),
            Expression::TSNonNullExpression(expr) => self.visit_ts_non_null_expression(expr),
            Expression::TSTypeAssertion(expr) => self.visit_ts_type_assertion(expr),
            Expression::TSInstantiationExpression(expr) => {
                self.visit_ts_instantiation_expression(expr);
            }
        }
    }

    fn visit_meta_property(&mut self, meta: &'a MetaProperty) {
        let kind = AstKind::MetaProperty(meta);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_array_expression(&mut self, expr: &'a ArrayExpression<'a>) {
        let kind = AstKind::ArrayExpression(expr);
        self.enter_node(kind);
        for elem in expr.elements.iter().flatten() {
            self.visit_argument(elem);
        }
        self.leave_node(kind);
    }

    fn visit_argument(&mut self, arg: &'a Argument<'a>) {
        let kind = AstKind::Argument(arg);
        self.enter_node(kind);
        match arg {
            Argument::SpreadElement(spread) => self.visit_spread_element(spread),
            Argument::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_spread_element(&mut self, elem: &'a SpreadElement<'a>) {
        let kind = AstKind::SpreadElement(elem);
        self.enter_node(kind);
        self.visit_expression(&elem.argument);
        self.leave_node(kind);
    }

    fn visit_assignment_expression(&mut self, expr: &'a AssignmentExpression<'a>) {
        let kind = AstKind::AssignmentExpression(expr);
        self.enter_node(kind);
        self.visit_assignment_target(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_arrow_expression(&mut self, expr: &'a ArrowExpression<'a>) {
        let kind = AstKind::ArrowExpression(expr);
        self.enter_node(kind);
        self.visit_formal_parameters(&expr.params);
        self.visit_function_body(&expr.body);
        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.leave_node(kind);
    }

    fn visit_await_expression(&mut self, expr: &'a AwaitExpression<'a>) {
        let kind = AstKind::AwaitExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_binary_expression(&mut self, expr: &'a BinaryExpression<'a>) {
        let kind = AstKind::BinaryExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_call_expression(&mut self, expr: &'a CallExpression<'a>) {
        let kind = AstKind::CallExpression(expr);
        self.enter_node(kind);
        for arg in &expr.arguments {
            self.visit_argument(arg);
        }
        self.visit_expression(&expr.callee);
        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
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
        let kind = AstKind::ConditionalExpression(expr);
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
        let kind = AstKind::LogicalExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_member_expression(&mut self, expr: &'a MemberExpression<'a>) {
        let kind = AstKind::MemberExpression(expr);
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
        let kind = AstKind::NewExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.callee);
        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        for arg in &expr.arguments {
            self.visit_argument(arg);
        }
        self.leave_node(kind);
    }

    fn visit_object_expression(&mut self, expr: &'a ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(expr);
        self.enter_node(kind);
        for prop in &expr.properties {
            self.visit_object_property(prop);
        }
        self.leave_node(kind);
    }

    fn enter_object_expression(&mut self, _expr: &'a ObjectExpression<'a>) {}

    fn visit_object_property(&mut self, prop: &'a ObjectProperty<'a>) {
        match prop {
            ObjectProperty::Property(prop) => self.visit_property(prop),
            ObjectProperty::SpreadProperty(elem) => self.visit_spread_element(elem),
        }
    }

    fn visit_property(&mut self, prop: &'a Property<'a>) {
        let kind = AstKind::Property(prop);
        self.enter_node(kind);
        self.visit_property_key(&prop.key);
        self.visit_property_value(&prop.value);
        self.leave_node(kind);
    }

    fn visit_property_key(&mut self, key: &'a PropertyKey<'a>) {
        let kind = AstKind::PropertyKey(key);
        self.enter_node(kind);
        match key {
            PropertyKey::Identifier(ident) => self.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => self.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_property_value(&mut self, value: &'a PropertyValue<'a>) {
        let kind = AstKind::PropertyValue(value);
        self.enter_node(kind);
        match value {
            PropertyValue::Pattern(pat) => self.visit_pattern(pat),
            PropertyValue::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_parenthesized_expression(&mut self, expr: &'a ParenthesizedExpression<'a>) {
        let kind = AstKind::ParenthesizedExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.leave_node(kind);
    }

    fn visit_private_in_expression(&mut self, expr: &'a PrivateInExpression<'a>) {
        self.visit_private_identifier(&expr.left);
        self.visit_expression(&expr.right);
    }

    fn visit_sequence_expression(&mut self, expr: &'a SequenceExpression<'a>) {
        let kind = AstKind::SequenceExpression(expr);
        self.enter_node(kind);
        for expr in &expr.expressions {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_tagged_template_expression(&mut self, expr: &'a TaggedTemplateExpression<'a>) {
        let kind = AstKind::TaggedTemplateExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.tag);
        self.visit_template_literal(&expr.quasi);
        self.leave_node(kind);
    }

    fn visit_this_expression(&mut self, expr: &'a ThisExpression) {
        let kind = AstKind::ThisExpression(expr);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_unary_expression(&mut self, expr: &'a UnaryExpression<'a>) {
        let kind = AstKind::UnaryExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_update_expression(&mut self, expr: &'a UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(expr);
        self.enter_node(kind);
        self.visit_simple_assignment_target(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_yield_expression(&mut self, expr: &'a YieldExpression<'a>) {
        let kind = AstKind::YieldExpression(expr);
        self.enter_node(kind);
        if let Some(argument) = &expr.argument {
            self.visit_expression(argument);
        }
        self.leave_node(kind);
    }

    fn visit_super(&mut self, expr: &'a Super) {
        let kind = AstKind::Super(expr);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_assignment_target(&mut self, target: &'a AssignmentTarget<'a>) {
        let kind = AstKind::AssignmentTarget(target);
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
        let kind = AstKind::SimpleAssignmentTarget(target);
        self.enter_node(kind);
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.visit_identifier_reference(ident);
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
                self.visit_member_expression(expr);
            }
            SimpleAssignmentTarget::TSAsExpression(expr) => {
                self.visit_expression(&expr.expression);
            }
            SimpleAssignmentTarget::TSNonNullExpression(expr) => {
                self.visit_expression(&expr.expression);
            }
            SimpleAssignmentTarget::TSTypeAssertion(expr) => {
                self.visit_expression(&expr.expression);
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
        let kind = AstKind::AssignmentTargetWithDefault(target);
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
        let kind = AstKind::JSXOpeningElement(elem);
        self.enter_node(kind);
        self.visit_jsx_element_name(&elem.name);
        for attribute in &elem.attributes {
            self.visit_jsx_attribute_item(attribute);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_element_name(&mut self, name: &'a JSXElementName<'a>) {
        let kind = AstKind::JSXElementName(name);
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

    fn visit_pattern(&mut self, pat: &'a BindingPattern<'a>) {
        match &pat.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                self.visit_binding_identifier(ident);
            }
            BindingPatternKind::ObjectPattern(pat) => self.visit_object_pattern(pat),
            BindingPatternKind::ArrayPattern(pat) => self.visit_array_pattern(pat),
            BindingPatternKind::RestElement(pat) => self.visit_rest_element(pat),
            BindingPatternKind::AssignmentPattern(pat) => self.visit_assignment_pattern(pat),
        }
        if let Some(type_annotation) = &pat.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    fn visit_binding_identifier(&mut self, ident: &'a BindingIdentifier) {
        let kind = AstKind::BindingIdentifier(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_object_pattern(&mut self, pat: &'a ObjectPattern<'a>) {
        let kind = AstKind::ObjectPattern(pat);
        self.enter_node(kind);
        for prop in &pat.properties {
            self.visit_object_pattern_property(prop);
        }
        self.leave_node(kind);
    }

    fn visit_object_pattern_property(&mut self, prop: &'a ObjectPatternProperty<'a>) {
        match prop {
            ObjectPatternProperty::Property(prop) => self.visit_property(prop),
            ObjectPatternProperty::RestElement(prop) => self.visit_rest_element(prop),
        }
    }

    fn visit_array_pattern(&mut self, pat: &'a ArrayPattern<'a>) {
        let kind = AstKind::ArrayPattern(pat);
        self.enter_node(kind);
        for pat in pat.elements.iter().flatten() {
            self.visit_pattern(pat);
        }
        self.leave_node(kind);
    }

    fn visit_rest_element(&mut self, pat: &'a RestElement<'a>) {
        let kind = AstKind::RestElement(pat);
        self.enter_node(kind);
        self.visit_pattern(&pat.argument);
        self.leave_node(kind);
    }

    fn visit_assignment_pattern(&mut self, pat: &'a AssignmentPattern<'a>) {
        let kind = AstKind::AssignmentPattern(pat);
        self.enter_node(kind);
        self.visit_pattern(&pat.left);
        self.visit_expression(&pat.right);
        self.leave_node(kind);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, ident: &'a IdentifierReference) {
        let kind = AstKind::IdentifierReference(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_private_identifier(&mut self, ident: &'a PrivateIdentifier) {
        let kind = AstKind::PrivateIdentifier(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_label_identifier(&mut self, ident: &'a LabelIdentifier) {
        let kind = AstKind::LabelIdentifier(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_identifier_name(&mut self, ident: &'a IdentifierName) {
        let kind = AstKind::IdentifierName(ident);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, lit: &'a NumberLiteral<'a>) {
        let kind = AstKind::NumberLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_boolean_literal(&mut self, lit: &'a BooleanLiteral) {
        let kind = AstKind::BooleanLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_null_literal(&mut self, lit: &'a NullLiteral) {
        let kind = AstKind::NullLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_bigint_literal(&mut self, lit: &'a BigintLiteral) {
        let kind = AstKind::BigintLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_string_literal(&mut self, lit: &'a StringLiteral) {
        let kind = AstKind::StringLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_literal(&mut self, lit: &'a TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(lit);
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
        let kind = AstKind::RegExpLiteral(lit);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_element(&mut self, _elem: &'a TemplateElement) {}

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &'a ModuleDeclaration<'a>) {
        let kind = AstKind::ModuleDeclaration(decl);
        self.enter_node(kind);
        match &decl.kind {
            ModuleDeclarationKind::ImportDeclaration(decl) => {
                self.visit_import_declaration(decl);
            }
            ModuleDeclarationKind::ExportAllDeclaration(decl) => {
                self.visit_export_all_declaration(decl);
            }
            ModuleDeclarationKind::ExportDefaultDeclaration(decl) => {
                self.visit_export_default_declaration(decl);
            }
            ModuleDeclarationKind::ExportNamedDeclaration(decl) => {
                self.visit_export_named_declaration(decl);
            }
            ModuleDeclarationKind::TSExportAssignment(decl) => {
                self.visit_expression(&decl.expression);
            }
            ModuleDeclarationKind::TSNamespaceExportDeclaration(_) => {}
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
            _ => {}
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &'a ExportNamedDeclaration<'a>) {
        if let Some(decl) = &decl.declaration {
            self.visit_declaration(decl);
        }
    }

    fn visit_enum_member(&mut self, member: &'a TSEnumMember<'a>) {
        let kind = AstKind::TSEnumMember(member);
        self.enter_node(kind);

        if let Some(initializer) = &member.initializer {
            self.visit_expression(initializer);
        }

        self.leave_node(kind);
    }

    fn visit_enum(&mut self, decl: &'a TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(decl);
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
            Declaration::TSModuleDeclaration(module) => {
                self.visit_ts_module_declaration(module);
            }
            Declaration::TSTypeAliasDeclaration(decl) => {
                self.visit_ts_type_alias_declaration(decl);
            }
            Declaration::TSEnumDeclaration(decl) => self.visit_enum(decl),
            Declaration::TSImportEqualsDeclaration(decl) => {
                self.visit_ts_import_equals_declaration(decl);
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                self.visit_ts_interface_declaration(decl);
            }
        }
    }

    fn visit_ts_import_equals_declaration(&mut self, decl: &'a TSImportEqualsDeclaration<'a>) {
        let kind = AstKind::TSImportEqualsDeclaration(decl);
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        self.leave_node(kind);
    }

    fn visit_ts_module_declaration(&mut self, decl: &'a TSModuleDeclaration<'a>) {
        let kind = AstKind::TSModuleDeclaration(decl);
        self.enter_node(kind);
        match &decl.id {
            TSModuleDeclarationName::Identifier(ident) => self.visit_identifier_name(ident),
            TSModuleDeclarationName::StringLiteral(lit) => self.visit_string_literal(lit),
        }
        match &decl.body {
            TSModuleDeclarationBody::TSModuleDeclaration(decl) => {
                self.visit_ts_module_declaration(decl);
            }
            TSModuleDeclarationBody::TSModuleBlock(block) => self.visit_ts_module_block(block),
        }
        self.leave_node(kind);
    }

    fn visit_ts_module_block(&mut self, block: &'a TSModuleBlock<'a>) {
        let kind = AstKind::TSModuleBlock(block);
        self.enter_node(kind);
        self.visit_statements(&block.body);
        self.leave_node(kind);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &'a TSTypeAliasDeclaration<'a>) {
        let kind = AstKind::TSTypeAliasDeclaration(decl);
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        if let Some(parameters) = &decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type(&decl.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &'a TSInterfaceDeclaration<'a>) {
        let kind = AstKind::TSInterfaceDeclaration(decl);
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        if let Some(parameters) = &decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        for signature in &decl.body.body {
            self.visit_ts_signature(signature);
        }
        self.leave_node(kind);
    }

    fn visit_ts_as_expression(&mut self, expr: &'a TSAsExpression<'a>) {
        let kind = AstKind::TSAsExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.visit_ts_type(&expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_non_null_expression(&mut self, expr: &'a TSNonNullExpression<'a>) {
        let kind = AstKind::TSNonNullExpression(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.leave_node(kind);
    }

    fn visit_ts_type_assertion(&mut self, expr: &'a TSTypeAssertion<'a>) {
        let kind = AstKind::TSTypeAssertion(expr);
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.visit_ts_type(&expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_instantiation_expression(&mut self, expr: &'a TSInstantiationExpression<'a>) {
        self.visit_expression(&expr.expression);
        self.visit_ts_type_parameter_instantiation(&expr.type_parameters);
    }

    fn visit_ts_type_annotation(&mut self, annotation: &'a TSTypeAnnotation<'a>) {
        let kind = AstKind::TSTypeAnnotation(annotation);
        self.enter_node(kind);
        self.visit_ts_type(&annotation.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_type(&mut self, ty: &'a TSType<'a>) {
        match ty {
            TSType::TSAnyKeyword(ty) => self.visit_ts_any_keyword(ty),
            TSType::TSNullKeyword(ty) => self.visit_ts_null_keyword(ty),
            TSType::TSVoidKeyword(ty) => self.visit_ts_void_keyword(ty),
            TSType::TSIntersectionType(ty) => self.visit_ts_intersection_type(ty),
            TSType::TSTypeReference(ty) => self.visit_ts_type_reference(ty),
            TSType::TSUnionType(ty) => self.visit_ts_union_type(ty),
            TSType::TSLiteralType(ty) => self.visit_ts_literal_type(ty),
            TSType::TSArrayType(ty) => self.visit_ts_array_type(ty),
            TSType::TSConditionalType(ty) => self.visit_ts_conditional_type(ty),
            TSType::TSConstructorType(ty) => self.visit_ts_constructor_type(ty),
            TSType::TSFunctionType(ty) => self.visit_ts_function_type(ty),
            TSType::TSMappedType(ty) => self.visit_ts_mapped_type(ty),
            TSType::TSTupleType(ty) => self.visit_ts_tuple_type(ty),
            TSType::TSTypeOperatorType(ty) => self.visit_ts_type_operator_type(ty),
            TSType::TSTypePredicate(ty) => self.visit_ts_type_predicate(ty),
            TSType::TSTypeLiteral(ty) => self.visit_ts_type_literal(ty),
            TSType::TSIndexedAccessType(ty) => self.visit_ts_indexed_access_type(ty),
            _ => {}
        }
    }

    fn visit_ts_type_literal(&mut self, ty: &'a TSTypeLiteral<'a>) {
        let kind = AstKind::TSTypeLiteral(ty);
        self.enter_node(kind);
        for signature in &ty.members {
            self.visit_ts_signature(signature);
        }
        self.leave_node(kind);
    }

    fn visit_ts_indexed_access_type(&mut self, ty: &'a TSIndexedAccessType<'a>) {
        let kind = AstKind::TSIndexedAccessType(ty);
        self.enter_node(kind);
        self.visit_ts_type(&ty.object_type);
        self.visit_ts_type(&ty.index_type);
        self.leave_node(kind);
    }

    fn visit_ts_type_predicate(&mut self, ty: &'a TSTypePredicate<'a>) {
        if let Some(annotation) = &ty.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_type_operator_type(&mut self, ty: &'a TSTypeOperatorType<'a>) {
        self.visit_ts_type(&ty.type_annotation);
    }

    fn visit_ts_tuple_type(&mut self, ty: &'a TSTupleType<'a>) {
        for element in &ty.element_types {
            self.visit_ts_tuple_element(element);
        }
    }

    fn visit_ts_tuple_element(&mut self, ty: &'a TSTupleElement<'a>) {
        match ty {
            TSTupleElement::TSType(ty) => self.visit_ts_type(ty),
            TSTupleElement::TSOptionalType(ty) => self.visit_ts_type(&ty.type_annotation),
            TSTupleElement::TSRestType(ty) => self.visit_ts_type(&ty.type_annotation),
            TSTupleElement::TSNamedTupleMember(ty) => self.visit_ts_type(&ty.element_type),
        };
    }

    fn visit_ts_mapped_type(&mut self, ty: &'a TSMappedType<'a>) {
        self.visit_ts_type_parameter(&ty.type_parameter);
        if let Some(name) = &ty.name_type {
            self.visit_ts_type(name);
        }
        self.visit_ts_type(&ty.type_annotation);
    }

    fn visit_ts_function_type(&mut self, ty: &'a TSFunctionType<'a>) {
        self.visit_formal_parameters(&ty.params);
        if let Some(parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type_annotation(&ty.return_type);
    }

    fn visit_ts_type_parameter(&mut self, ty: &'a TSTypeParameter<'a>) {
        let kind = AstKind::TSTypeParameter(ty);
        self.enter_node(kind);
        if let Some(constraint) = &ty.constraint {
            self.visit_ts_type(constraint);
        }

        if let Some(default) = &ty.default {
            self.visit_ts_type(default);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_parameter_instantiation(&mut self, ty: &'a TSTypeParameterInstantiation<'a>) {
        let kind = AstKind::TSTypeParameterInstantiation(ty);
        self.enter_node(kind);
        for ts_parameter in &ty.params {
            self.visit_ts_type(ts_parameter);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_parameter_declaration(&mut self, ty: &'a TSTypeParameterDeclaration<'a>) {
        let kind = AstKind::TSTypeParameterDeclaration(ty);
        self.enter_node(kind);
        for ts_parameter in &ty.params {
            self.visit_ts_type_parameter(ts_parameter);
        }
        self.leave_node(kind);
    }

    fn visit_ts_constructor_type(&mut self, ty: &'a TSConstructorType<'a>) {
        self.visit_formal_parameters(&ty.params);
        if let Some(parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type_annotation(&ty.return_type);
    }

    fn visit_ts_conditional_type(&mut self, ty: &'a TSConditionalType<'a>) {
        self.visit_ts_type(&ty.check_type);
        self.visit_ts_type(&ty.extends_type);
        self.visit_ts_type(&ty.true_type);
        self.visit_ts_type(&ty.false_type);
    }

    fn visit_ts_array_type(&mut self, ty: &'a TSArrayType<'a>) {
        self.visit_ts_type(&ty.element_type);
    }

    fn visit_ts_type_name(&mut self, name: &'a TSTypeName<'a>) {
        match &name {
            TSTypeName::IdentifierName(ident) => self.visit_identifier_name(ident),
            TSTypeName::QualifiedName(_) => {}
        }
    }

    fn visit_ts_null_keyword(&mut self, ty: &'a TSNullKeyword) {
        let kind = AstKind::TSNullKeyword(ty);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_any_keyword(&mut self, ty: &'a TSAnyKeyword) {
        let kind = AstKind::TSAnyKeyword(ty);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_void_keyword(&mut self, ty: &'a TSVoidKeyword) {
        let kind = AstKind::TSVoidKeyword(ty);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_intersection_type(&mut self, ty: &'a TSIntersectionType<'a>) {
        let kind = AstKind::TSIntersectionType(ty);
        self.enter_node(kind);
        for ty in &ty.types {
            self.visit_ts_type(ty);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_reference(&mut self, ty: &'a TSTypeReference<'a>) {
        let kind = AstKind::TSTypeReference(ty);
        self.enter_node(kind);
        self.visit_ts_type_name(&ty.type_name);
        if let Some(parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        self.leave_node(kind);
    }

    fn visit_ts_union_type(&mut self, ty: &'a TSUnionType<'a>) {
        let kind = AstKind::TSUnionType(ty);
        self.enter_node(kind);
        for ty in &ty.types {
            self.visit_ts_type(ty);
        }
        self.leave_node(kind);
    }

    fn visit_ts_literal_type(&mut self, ty: &'a TSLiteralType<'a>) {
        let kind = AstKind::TSLiteralType(ty);
        self.enter_node(kind);
        match &ty.literal {
            TSLiteral::BigintLiteral(lit) => self.visit_bigint_literal(lit),
            TSLiteral::BooleanLiteral(lit) => self.visit_boolean_literal(lit),
            TSLiteral::NullLiteral(lit) => self.visit_null_literal(lit),
            TSLiteral::NumberLiteral(lit) => self.visit_number_literal(lit),
            TSLiteral::RegExpLiteral(lit) => self.visit_reg_expr_literal(lit),
            TSLiteral::StringLiteral(lit) => self.visit_string_literal(lit),
            TSLiteral::TemplateLiteral(lit) => self.visit_template_literal(lit),
            TSLiteral::UnaryExpression(expr) => self.visit_unary_expression(expr),
        }
        self.leave_node(kind);
    }

    #[allow(clippy::single_match)]
    fn visit_ts_signature(&mut self, signature: &'a TSSignature<'a>) {
        match &signature {
            TSSignature::TSPropertySignature(sig) => self.visit_ts_property_signature(sig),
            TSSignature::TSCallSignatureDeclaration(sig) => {
                self.visit_ts_call_signature_declaration(sig);
            }
            TSSignature::TSIndexSignature(sig) => self.visit_ts_index_signature(sig),
            TSSignature::TSMethodSignature(sig) => self.visit_ts_method_signature(sig),
            TSSignature::TSConstructSignatureDeclaration(sig) => {
                self.visit_ts_construct_signature_declaration(sig);
            }
        }
    }

    fn visit_ts_construct_signature_declaration(
        &mut self,
        signature: &'a TSConstructSignatureDeclaration<'a>,
    ) {
        self.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_method_signature(&mut self, signature: &'a TSMethodSignature<'a>) {
        let kind = AstKind::TSMethodSignature(signature);
        self.enter_node(kind);
        self.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
    }

    fn visit_ts_index_signature_name(&mut self, name: &'a TSIndexSignatureName<'a>) {
        self.visit_ts_type_annotation(&name.type_annotation);
    }

    fn visit_ts_index_signature(&mut self, signature: &'a TSIndexSignature<'a>) {
        for name in &signature.parameters {
            self.visit_ts_index_signature_name(name);
        }

        self.visit_ts_type_annotation(&signature.type_annotation);
    }

    fn visit_ts_property_signature(&mut self, signature: &'a TSPropertySignature<'a>) {
        let kind = AstKind::TSPropertySignature(signature);
        self.enter_node(kind);
        self.visit_property_key(&signature.key);
        if let Some(annotation) = &signature.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
    }

    fn visit_ts_call_signature_declaration(
        &mut self,
        signature: &'a TSCallSignatureDeclaration<'a>,
    ) {
        self.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(annotation) = &signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }
}
