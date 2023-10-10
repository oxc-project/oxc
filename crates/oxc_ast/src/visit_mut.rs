//! Visit Mut Pattern

use oxc_allocator::Vec;
use oxc_span::Span;

#[allow(clippy::wildcard_imports)]
use crate::ast::*;

/// Syntax tree traversal to mutate an exclusive borrow of a syntax tree in place.
pub trait VisitMut<'a>: Sized {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        for directive in program.directives.iter_mut() {
            self.visit_directive(directive);
        }
        self.visit_statements(&mut program.body);
    }

    /* ----------  Statement ---------- */

    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        for stmt in stmts.iter_mut() {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.visit_statement_match(stmt);
    }

    fn visit_statement_match(&mut self, stmt: &mut Statement<'a>) {
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

    fn visit_block_statement(&mut self, stmt: &mut BlockStatement<'a>) {
        self.visit_statements(&mut stmt.body);
    }

    fn visit_break_statement(&mut self, stmt: &mut BreakStatement) {
        if let Some(break_target) = &mut stmt.label {
            self.visit_label_identifier(break_target);
        }
    }

    fn visit_continue_statement(&mut self, stmt: &mut ContinueStatement) {
        if let Some(continue_target) = &mut stmt.label {
            self.visit_label_identifier(continue_target);
        }
    }

    fn visit_debugger_statement(&mut self, _stmt: &mut DebuggerStatement) {}

    fn visit_do_while_statement(&mut self, stmt: &mut DoWhileStatement<'a>) {
        self.visit_statement(&mut stmt.body);
        self.visit_expression(&mut stmt.test);
    }

    fn visit_empty_statement(&mut self, _stmt: &mut EmptyStatement) {}

    fn visit_expression_statement(&mut self, stmt: &mut ExpressionStatement<'a>) {
        self.visit_expression(&mut stmt.expression);
    }

    fn visit_for_statement(&mut self, stmt: &mut ForStatement<'a>) {
        if let Some(init) = &mut stmt.init {
            self.visit_for_statement_init(init);
        }
        if let Some(test) = &mut stmt.test {
            self.visit_expression(test);
        }
        if let Some(update) = &mut stmt.update {
            self.visit_expression(update);
        }
        self.visit_statement(&mut stmt.body);
    }

    fn visit_for_statement_init(&mut self, init: &mut ForStatementInit<'a>) {
        match init {
            ForStatementInit::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            ForStatementInit::Expression(expr) => self.visit_expression(expr),
            ForStatementInit::UsingDeclaration(decl) => {
                self.visit_using_declaration(decl);
            }
        }
    }

    fn visit_for_in_statement(&mut self, stmt: &mut ForInStatement<'a>) {
        self.visit_for_statement_left(&mut stmt.left);
        self.visit_expression(&mut stmt.right);
        self.visit_statement(&mut stmt.body);
    }

    fn visit_for_of_statement(&mut self, stmt: &mut ForOfStatement<'a>) {
        self.visit_for_statement_left(&mut stmt.left);
        self.visit_expression(&mut stmt.right);
        self.visit_statement(&mut stmt.body);
    }

    fn visit_for_statement_left(&mut self, left: &mut ForStatementLeft<'a>) {
        match left {
            ForStatementLeft::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            ForStatementLeft::AssignmentTarget(target) => self.visit_assignment_target(target),
            ForStatementLeft::UsingDeclaration(decl) => {
                self.visit_using_declaration(decl);
            }
        }
    }

    fn visit_if_statement(&mut self, stmt: &mut IfStatement<'a>) {
        self.visit_expression(&mut stmt.test);
        self.visit_statement(&mut stmt.consequent);
        if let Some(alternate) = &mut stmt.alternate {
            self.visit_statement(alternate);
        }
    }

    fn visit_labeled_statement(&mut self, stmt: &mut LabeledStatement<'a>) {
        self.visit_label_identifier(&mut stmt.label);
        self.visit_statement(&mut stmt.body);
    }

    fn visit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        if let Some(arg) = &mut stmt.argument {
            self.visit_expression(arg);
        }
    }

    fn visit_switch_statement(&mut self, stmt: &mut SwitchStatement<'a>) {
        self.visit_expression(&mut stmt.discriminant);
        for case in stmt.cases.iter_mut() {
            self.visit_switch_case(case);
        }
    }

    fn visit_switch_case(&mut self, case: &mut SwitchCase<'a>) {
        if let Some(expr) = &mut case.test {
            self.visit_expression(expr);
        }
        self.visit_statements(&mut case.consequent);
    }

    fn visit_throw_statement(&mut self, stmt: &mut ThrowStatement<'a>) {
        self.visit_expression(&mut stmt.argument);
    }

    fn visit_try_statement(&mut self, stmt: &mut TryStatement<'a>) {
        self.visit_block_statement(&mut stmt.block);
        if let Some(handler) = &mut stmt.handler {
            self.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &mut stmt.finalizer {
            self.visit_finally_clause(finalizer);
        }
    }

    fn visit_catch_clause(&mut self, clause: &mut CatchClause<'a>) {
        if let Some(param) = &mut clause.param {
            self.visit_binding_pattern(param);
        }
        self.visit_statements(&mut clause.body.body);
    }

    fn visit_finally_clause(&mut self, clause: &mut BlockStatement<'a>) {
        self.visit_block_statement(clause);
    }

    fn visit_while_statement(&mut self, stmt: &mut WhileStatement<'a>) {
        self.visit_expression(&mut stmt.test);
        self.visit_statement(&mut stmt.body);
    }

    fn visit_with_statement(&mut self, stmt: &mut WithStatement<'a>) {
        self.visit_expression(&mut stmt.object);
        self.visit_statement(&mut stmt.body);
    }

    fn visit_directive(&mut self, directive: &mut Directive) {
        self.visit_string_literal(&mut directive.expression);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        for declarator in decl.declarations.iter_mut() {
            self.visit_variable_declarator(declarator);
        }
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        self.visit_binding_pattern(&mut declarator.id);
        if let Some(init) = &mut declarator.init {
            self.visit_expression(init);
        }
    }

    fn visit_using_declaration(&mut self, declaration: &mut UsingDeclaration<'a>) {
        for decl in declaration.declarations.iter_mut() {
            self.visit_variable_declarator(decl);
        }
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &mut Function<'a>) {
        if let Some(ident) = &mut func.id {
            self.visit_binding_identifier(ident);
        }
        self.visit_formal_parameters(&mut func.params);
        if let Some(body) = &mut func.body {
            self.visit_function_body(body);
        }
        if let Some(parameters) = &mut func.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut func.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_function_body(&mut self, body: &mut FunctionBody<'a>) {
        for directive in body.directives.iter_mut() {
            self.visit_directive(directive);
        }
        self.visit_statements(&mut body.statements);
    }

    fn visit_formal_parameters(&mut self, params: &mut FormalParameters<'a>) {
        for param in params.items.iter_mut() {
            self.visit_formal_parameter(param);
        }
        if let Some(rest) = &mut params.rest {
            self.visit_rest_element(rest);
        }
    }

    fn visit_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        for decorator in param.decorators.iter_mut() {
            self.visit_decorator(decorator);
        }
        self.visit_binding_pattern(&mut param.pattern);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &mut Decorator<'a>) {
        self.visit_expression(&mut decorator.expression);
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        for decorator in class.decorators.iter_mut() {
            self.visit_decorator(decorator);
        }
        if let Some(id) = &mut class.id {
            self.visit_binding_identifier(id);
        }
        if let Some(parameters) = &mut class.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(super_class) = &mut class.super_class {
            self.visit_class_heritage(super_class);
        }
        if let Some(super_parameters) = &mut class.super_type_parameters {
            self.visit_ts_type_parameter_instantiation(super_parameters);
        }
        self.visit_class_body(&mut class.body);
    }

    fn visit_class_heritage(&mut self, expr: &mut Expression<'a>) {
        self.visit_expression(expr);
    }

    fn visit_class_body(&mut self, body: &mut ClassBody<'a>) {
        for elem in body.body.iter_mut() {
            self.visit_class_element(elem);
        }
    }

    fn visit_class_element(&mut self, elem: &mut ClassElement<'a>) {
        match elem {
            ClassElement::StaticBlock(block) => self.visit_static_block(block),
            ClassElement::MethodDefinition(def) => self.visit_method_definition(def),
            ClassElement::PropertyDefinition(def) => self.visit_property_definition(def),
            ClassElement::AccessorProperty(_def) => { /* TODO */ }
            ClassElement::TSAbstractMethodDefinition(def) => {
                self.visit_method_definition(&mut def.method_definition);
            }
            ClassElement::TSAbstractPropertyDefinition(def) => {
                self.visit_property_definition(&mut def.property_definition);
            }
            ClassElement::TSIndexSignature(_def) => {}
        }
    }

    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        self.visit_statements(&mut block.body);
    }

    fn visit_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        for decorator in def.decorators.iter_mut() {
            self.visit_decorator(decorator);
        }
        self.visit_property_key(&mut def.key);
        self.visit_function(&mut def.value);
    }

    fn visit_property_definition(&mut self, def: &mut PropertyDefinition<'a>) {
        for decorator in def.decorators.iter_mut() {
            self.visit_decorator(decorator);
        }
        self.visit_property_key(&mut def.key);
        if let Some(value) = &mut def.value {
            self.visit_expression(value);
        }
        if let Some(annotation) = &mut def.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
    }

    /* ----------  Expression ---------- */

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.visit_expression_match(expr);
    }

    fn visit_expression_match(&mut self, expr: &mut Expression<'a>) {
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
            Expression::TSSatisfiesExpression(expr) => self.visit_ts_satisfies_expression(expr),
            Expression::TSNonNullExpression(expr) => self.visit_ts_non_null_expression(expr),
            Expression::TSTypeAssertion(expr) => self.visit_ts_type_assertion(expr),
            Expression::TSInstantiationExpression(expr) => {
                self.visit_ts_instantiation_expression(expr);
            }
        }
    }

    fn visit_meta_property(&mut self, _meta: &mut MetaProperty) {}

    fn visit_array_expression(&mut self, expr: &mut ArrayExpression<'a>) {
        for elem in expr.elements.iter_mut() {
            self.visit_array_expression_element(elem);
        }
    }

    fn visit_array_expression_element(&mut self, arg: &mut ArrayExpressionElement<'a>) {
        match arg {
            ArrayExpressionElement::SpreadElement(spread) => self.visit_spread_element(spread),
            ArrayExpressionElement::Expression(expr) => self.visit_expression_array_element(expr),
            ArrayExpressionElement::Elision(span) => self.visit_elision(*span),
        }
    }

    fn visit_argument(&mut self, arg: &mut Argument<'a>) {
        match arg {
            Argument::SpreadElement(spread) => self.visit_spread_element(spread),
            Argument::Expression(expr) => self.visit_expression(expr),
        }
    }

    fn visit_spread_element(&mut self, elem: &mut SpreadElement<'a>) {
        self.visit_expression(&mut elem.argument);
    }

    fn visit_expression_array_element(&mut self, expr: &mut Expression<'a>) {
        self.visit_expression(expr);
    }

    fn visit_elision(&mut self, _span: Span) {}

    fn visit_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        self.visit_assignment_target(&mut expr.left);
        self.visit_expression(&mut expr.right);
    }

    fn visit_arrow_expression(&mut self, expr: &mut ArrowExpression<'a>) {
        self.visit_formal_parameters(&mut expr.params);
        self.visit_function_body(&mut expr.body);
        if let Some(parameters) = &mut expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
    }

    fn visit_await_expression(&mut self, expr: &mut AwaitExpression<'a>) {
        self.visit_expression(&mut expr.argument);
    }

    fn visit_binary_expression(&mut self, expr: &mut BinaryExpression<'a>) {
        self.visit_expression(&mut expr.left);
        self.visit_expression(&mut expr.right);
    }

    fn visit_call_expression(&mut self, expr: &mut CallExpression<'a>) {
        for arg in expr.arguments.iter_mut() {
            self.visit_argument(arg);
        }
        self.visit_expression(&mut expr.callee);
        if let Some(parameters) = &mut expr.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
    }

    fn visit_chain_expression(&mut self, expr: &mut ChainExpression<'a>) {
        self.visit_chain_element(&mut expr.expression);
    }

    fn visit_chain_element(&mut self, elem: &mut ChainElement<'a>) {
        match elem {
            ChainElement::CallExpression(expr) => self.visit_call_expression(expr),
            ChainElement::MemberExpression(expr) => self.visit_member_expression(expr),
        }
    }

    fn visit_conditional_expression(&mut self, expr: &mut ConditionalExpression<'a>) {
        self.visit_expression(&mut expr.test);
        self.visit_expression(&mut expr.consequent);
        self.visit_expression(&mut expr.alternate);
    }

    fn visit_import_expression(&mut self, expr: &mut ImportExpression<'a>) {
        self.visit_expression(&mut expr.source);
        for arg in expr.arguments.iter_mut() {
            self.visit_expression(arg);
        }
    }

    fn visit_logical_expression(&mut self, expr: &mut LogicalExpression<'a>) {
        self.visit_expression(&mut expr.left);
        self.visit_expression(&mut expr.right);
    }

    fn visit_member_expression(&mut self, expr: &mut MemberExpression<'a>) {
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
    }

    fn visit_computed_member_expression(&mut self, expr: &mut ComputedMemberExpression<'a>) {
        self.visit_expression(&mut expr.object);
        self.visit_expression(&mut expr.expression);
    }

    fn visit_static_member_expression(&mut self, expr: &mut StaticMemberExpression<'a>) {
        self.visit_expression(&mut expr.object);
        self.visit_identifier_name(&mut expr.property);
    }

    fn visit_private_field_expression(&mut self, expr: &mut PrivateFieldExpression<'a>) {
        self.visit_expression(&mut expr.object);
        self.visit_private_identifier(&mut expr.field);
    }

    fn visit_new_expression(&mut self, expr: &mut NewExpression<'a>) {
        self.visit_expression(&mut expr.callee);
        if let Some(parameters) = &mut expr.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        for arg in expr.arguments.iter_mut() {
            self.visit_argument(arg);
        }
    }

    fn visit_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        for prop in expr.properties.iter_mut() {
            self.visit_object_property_kind(prop);
        }
    }

    fn visit_object_property_kind(&mut self, prop: &mut ObjectPropertyKind<'a>) {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => self.visit_object_property(prop),
            ObjectPropertyKind::SpreadProperty(elem) => self.visit_spread_element(elem),
        }
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        self.visit_property_key(&mut prop.key);
        self.visit_expression(&mut prop.value);
    }

    fn visit_property_key(&mut self, key: &mut PropertyKey<'a>) {
        match key {
            PropertyKey::Identifier(ident) => self.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => self.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => self.visit_expression(expr),
        }
    }

    fn visit_parenthesized_expression(&mut self, expr: &mut ParenthesizedExpression<'a>) {
        self.visit_expression(&mut expr.expression);
    }

    fn visit_private_in_expression(&mut self, expr: &mut PrivateInExpression<'a>) {
        self.visit_private_identifier(&mut expr.left);
        self.visit_expression(&mut expr.right);
    }

    fn visit_sequence_expression(&mut self, expr: &mut SequenceExpression<'a>) {
        for expr in expr.expressions.iter_mut() {
            self.visit_expression(expr);
        }
    }

    fn visit_tagged_template_expression(&mut self, expr: &mut TaggedTemplateExpression<'a>) {
        self.visit_expression(&mut expr.tag);
        self.visit_template_literal(&mut expr.quasi);
    }

    fn visit_this_expression(&mut self, _expr: &mut ThisExpression) {}

    fn visit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>) {
        self.visit_expression(&mut expr.argument);
    }

    fn visit_update_expression(&mut self, expr: &mut UpdateExpression<'a>) {
        self.visit_simple_assignment_target(&mut expr.argument);
    }

    fn visit_yield_expression(&mut self, expr: &mut YieldExpression<'a>) {
        if let Some(argument) = &mut expr.argument {
            self.visit_expression(argument);
        }
    }

    fn visit_super(&mut self, _expr: &mut Super) {}

    fn visit_assignment_target(&mut self, target: &mut AssignmentTarget<'a>) {
        match target {
            AssignmentTarget::SimpleAssignmentTarget(target) => {
                self.visit_simple_assignment_target(target);
            }
            AssignmentTarget::AssignmentTargetPattern(pat) => {
                self.visit_assignment_target_pattern(pat);
            }
        }
    }

    fn visit_simple_assignment_target(&mut self, target: &mut SimpleAssignmentTarget<'a>) {
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                self.visit_identifier_reference(ident);
            }
            SimpleAssignmentTarget::MemberAssignmentTarget(expr) => {
                self.visit_member_expression(expr);
            }
            SimpleAssignmentTarget::TSAsExpression(expr) => {
                self.visit_expression(&mut expr.expression);
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(expr) => {
                self.visit_expression(&mut expr.expression);
            }
            SimpleAssignmentTarget::TSNonNullExpression(expr) => {
                self.visit_expression(&mut expr.expression);
            }
            SimpleAssignmentTarget::TSTypeAssertion(expr) => {
                self.visit_expression(&mut expr.expression);
            }
        }
    }

    fn visit_assignment_target_pattern(&mut self, pat: &mut AssignmentTargetPattern<'a>) {
        match pat {
            AssignmentTargetPattern::ArrayAssignmentTarget(target) => {
                self.visit_array_assignment_target(target);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(target) => {
                self.visit_object_assignment_target(target);
            }
        }
    }

    fn visit_array_assignment_target(&mut self, target: &mut ArrayAssignmentTarget<'a>) {
        for element in target.elements.iter_mut().flatten() {
            self.visit_assignment_target_maybe_default(element);
        }
        if let Some(target) = &mut target.rest {
            self.visit_assignment_target(target);
        }
    }

    fn visit_assignment_target_maybe_default(
        &mut self,
        target: &mut AssignmentTargetMaybeDefault<'a>,
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
        target: &mut AssignmentTargetWithDefault<'a>,
    ) {
        self.visit_assignment_target(&mut target.binding);
        self.visit_expression(&mut target.init);
    }

    fn visit_object_assignment_target(&mut self, target: &mut ObjectAssignmentTarget<'a>) {
        for property in target.properties.iter_mut() {
            self.visit_assignment_target_property(property);
        }
        if let Some(target) = &mut target.rest {
            self.visit_assignment_target(target);
        }
    }

    fn visit_assignment_target_property(&mut self, property: &mut AssignmentTargetProperty<'a>) {
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
        ident: &mut AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.visit_identifier_reference(&mut ident.binding);
        if let Some(expr) = &mut ident.init {
            self.visit_expression(expr);
        }
    }

    fn visit_assignment_target_property_property(
        &mut self,
        property: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        self.visit_property_key(&mut property.name);
        self.visit_assignment_target_maybe_default(&mut property.binding);
    }

    /* ----------  Expression ---------- */

    fn visit_jsx_element(&mut self, elem: &mut JSXElement<'a>) {
        self.visit_jsx_opening_element(&mut elem.opening_element);
        for child in elem.children.iter_mut() {
            self.visit_jsx_child(child);
        }
    }

    fn visit_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        self.visit_jsx_element_name(&mut elem.name);
        for attribute in elem.attributes.iter_mut() {
            self.visit_jsx_attribute_item(attribute);
        }
    }

    fn visit_jsx_element_name(&mut self, __name: &mut JSXElementName<'a>) {}

    fn visit_jsx_attribute_item(&mut self, item: &mut JSXAttributeItem<'a>) {
        match item {
            JSXAttributeItem::Attribute(attribute) => self.visit_jsx_attribute(attribute),
            JSXAttributeItem::SpreadAttribute(attribute) => {
                self.visit_jsx_spread_attribute(attribute);
            }
        }
    }

    fn visit_jsx_attribute(&mut self, attribute: &mut JSXAttribute<'a>) {
        if let Some(value) = &mut attribute.value {
            self.visit_jsx_attribute_value(value);
        }
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &mut JSXSpreadAttribute<'a>) {
        self.visit_expression(&mut attribute.argument);
    }

    fn visit_jsx_attribute_value(&mut self, value: &mut JSXAttributeValue<'a>) {
        match value {
            JSXAttributeValue::ExpressionContainer(expr) => {
                self.visit_jsx_expression_container(expr);
            }
            JSXAttributeValue::Element(elem) => self.visit_jsx_element(elem),
            JSXAttributeValue::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXAttributeValue::StringLiteral(_) => {}
        }
    }

    fn visit_jsx_expression_container(&mut self, expr: &mut JSXExpressionContainer<'a>) {
        self.visit_jsx_expression(&mut expr.expression);
    }

    fn visit_jsx_expression(&mut self, expr: &mut JSXExpression<'a>) {
        match expr {
            JSXExpression::Expression(expr) => self.visit_expression(expr),
            JSXExpression::EmptyExpression(_) => {}
        }
    }

    fn visit_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>) {
        for child in elem.children.iter_mut() {
            self.visit_jsx_child(child);
        }
    }

    fn visit_jsx_child(&mut self, child: &mut JSXChild<'a>) {
        match child {
            JSXChild::Element(elem) => self.visit_jsx_element(elem),
            JSXChild::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXChild::ExpressionContainer(expr) => self.visit_jsx_expression_container(expr),
            JSXChild::Spread(expr) => self.visit_jsx_spread_child(expr),
            JSXChild::Text(_) => {}
        }
    }

    fn visit_jsx_spread_child(&mut self, child: &mut JSXSpreadChild<'a>) {
        self.visit_expression(&mut child.expression);
    }

    /* ----------  Pattern ---------- */

    fn visit_binding_pattern(&mut self, pat: &mut BindingPattern<'a>) {
        match &mut pat.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                self.visit_binding_identifier(ident);
            }
            BindingPatternKind::ObjectPattern(pat) => self.visit_object_pattern(pat),
            BindingPatternKind::ArrayPattern(pat) => self.visit_array_pattern(pat),
            BindingPatternKind::AssignmentPattern(pat) => self.visit_assignment_pattern(pat),
        }
        if let Some(type_annotation) = &mut pat.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    fn visit_binding_identifier(&mut self, __ident: &mut BindingIdentifier) {}

    fn visit_object_pattern(&mut self, pat: &mut ObjectPattern<'a>) {
        for prop in pat.properties.iter_mut() {
            self.visit_binding_property(prop);
        }
    }

    fn visit_binding_property(&mut self, prop: &mut BindingProperty<'a>) {
        self.visit_property_key(&mut prop.key);
        self.visit_binding_pattern(&mut prop.value);
    }

    fn visit_array_pattern(&mut self, pat: &mut ArrayPattern<'a>) {
        for pat in pat.elements.iter_mut().flatten() {
            self.visit_binding_pattern(pat);
        }
        if let Some(rest) = &mut pat.rest {
            self.visit_rest_element(rest);
        }
    }

    fn visit_rest_element(&mut self, pat: &mut RestElement<'a>) {
        self.visit_binding_pattern(&mut pat.argument);
    }

    fn visit_assignment_pattern(&mut self, pat: &mut AssignmentPattern<'a>) {
        self.visit_binding_pattern(&mut pat.left);
        self.visit_expression(&mut pat.right);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, _ident: &mut IdentifierReference) {}

    fn visit_private_identifier(&mut self, _ident: &mut PrivateIdentifier) {}

    fn visit_label_identifier(&mut self, _ident: &mut LabelIdentifier) {}

    fn visit_identifier_name(&mut self, _ident: &mut IdentifierName) {}

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, _lit: &mut NumberLiteral<'a>) {}

    fn visit_boolean_literal(&mut self, _lit: &mut BooleanLiteral) {}

    fn visit_null_literal(&mut self, _lit: &mut NullLiteral) {}

    fn visit_bigint_literal(&mut self, _lit: &mut BigintLiteral) {}

    fn visit_string_literal(&mut self, _lit: &mut StringLiteral) {}

    fn visit_template_literal(&mut self, lit: &mut TemplateLiteral<'a>) {
        for elem in lit.quasis.iter_mut() {
            self.visit_template_element(elem);
        }
        for expr in lit.expressions.iter_mut() {
            self.visit_expression(expr);
        }
    }

    fn visit_reg_expr_literal(&mut self, _lit: &mut RegExpLiteral) {}

    fn visit_template_element(&mut self, _elem: &mut TemplateElement) {}

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &mut ModuleDeclaration<'a>) {
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
            ModuleDeclaration::TSExportAssignment(decl) => {
                self.visit_expression(&mut decl.expression);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(_) => {}
        }
    }

    fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        for specifier in decl.specifiers.iter_mut() {
            self.visit_import_declaration_specifier(specifier);
        }
        // TODO: source
        // TODO: assertions
    }

    fn visit_import_declaration_specifier(&mut self, specifier: &mut ImportDeclarationSpecifier) {
        match specifier {
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

    fn visit_import_specifier(&mut self, specifier: &mut ImportSpecifier) {
        // TODO: imported
        self.visit_binding_identifier(&mut specifier.local);
    }

    fn visit_import_default_specifier(&mut self, specifier: &mut ImportDefaultSpecifier) {
        self.visit_binding_identifier(&mut specifier.local);
    }

    fn visit_import_name_specifier(&mut self, specifier: &mut ImportNamespaceSpecifier) {
        self.visit_binding_identifier(&mut specifier.local);
    }

    fn visit_export_all_declaration(&mut self, _decl: &mut ExportAllDeclaration<'a>) {}

    fn visit_export_default_declaration(&mut self, decl: &mut ExportDefaultDeclaration<'a>) {
        match &mut decl.declaration {
            ExportDefaultDeclarationKind::Expression(expr) => self.visit_expression(expr),
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                self.visit_function(func);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => self.visit_class(class),
            _ => {}
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        if let Some(decl) = &mut decl.declaration {
            self.visit_declaration(decl);
        }
    }

    fn visit_enum_member(&mut self, member: &mut TSEnumMember<'a>) {
        if let Some(initializer) = &mut member.initializer {
            self.visit_expression(initializer);
        }
    }

    fn visit_enum(&mut self, decl: &mut TSEnumDeclaration<'a>) {
        self.visit_binding_identifier(&mut decl.id);
        for member in decl.body.members.iter_mut() {
            self.visit_enum_member(member);
        }
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            Declaration::FunctionDeclaration(func) => self.visit_function(func),
            Declaration::ClassDeclaration(class) => self.visit_class(class),
            Declaration::UsingDeclaration(decl) => self.visit_using_declaration(decl),
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

    fn visit_ts_import_equals_declaration(&mut self, decl: &mut TSImportEqualsDeclaration<'a>) {
        self.visit_binding_identifier(&mut decl.id);
    }

    fn visit_ts_module_declaration(&mut self, decl: &mut TSModuleDeclaration<'a>) {
        match &mut decl.id {
            TSModuleDeclarationName::Identifier(ident) => self.visit_identifier_name(ident),
            TSModuleDeclarationName::StringLiteral(lit) => self.visit_string_literal(lit),
        }
        match &mut decl.body {
            TSModuleDeclarationBody::TSModuleDeclaration(decl) => {
                self.visit_ts_module_declaration(decl);
            }
            TSModuleDeclarationBody::TSModuleBlock(block) => self.visit_ts_module_block(block),
        }
    }

    fn visit_ts_module_block(&mut self, block: &mut TSModuleBlock<'a>) {
        self.visit_statements(&mut block.body);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &mut TSTypeAliasDeclaration<'a>) {
        self.visit_binding_identifier(&mut decl.id);
        if let Some(parameters) = &mut decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type(&mut decl.type_annotation);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &mut TSInterfaceDeclaration<'a>) {
        self.visit_binding_identifier(&mut decl.id);
        if let Some(parameters) = &mut decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        for signature in decl.body.body.iter_mut() {
            self.visit_ts_signature(signature);
        }
    }

    fn visit_ts_as_expression(&mut self, expr: &mut TSAsExpression<'a>) {
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type(&mut expr.type_annotation);
    }

    fn visit_ts_satisfies_expression(&mut self, expr: &mut TSSatisfiesExpression<'a>) {
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type(&mut expr.type_annotation);
    }

    fn visit_ts_non_null_expression(&mut self, expr: &mut TSNonNullExpression<'a>) {
        self.visit_expression(&mut expr.expression);
    }

    fn visit_ts_type_assertion(&mut self, expr: &mut TSTypeAssertion<'a>) {
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type(&mut expr.type_annotation);
    }

    fn visit_ts_instantiation_expression(&mut self, expr: &mut TSInstantiationExpression<'a>) {
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type_parameter_instantiation(&mut expr.type_parameters);
    }

    fn visit_ts_type_annotation(&mut self, annotation: &mut TSTypeAnnotation<'a>) {
        self.visit_ts_type(&mut annotation.type_annotation);
    }

    fn visit_ts_type(&mut self, ty: &mut TSType<'a>) {
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

    fn visit_ts_type_literal(&mut self, ty: &mut TSTypeLiteral<'a>) {
        for signature in ty.members.iter_mut() {
            self.visit_ts_signature(signature);
        }
    }

    fn visit_ts_indexed_access_type(&mut self, ty: &mut TSIndexedAccessType<'a>) {
        self.visit_ts_type(&mut ty.object_type);
        self.visit_ts_type(&mut ty.index_type);
    }

    fn visit_ts_type_predicate(&mut self, ty: &mut TSTypePredicate<'a>) {
        if let Some(annotation) = &mut ty.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_type_operator_type(&mut self, ty: &mut TSTypeOperatorType<'a>) {
        self.visit_ts_type(&mut ty.type_annotation);
    }

    fn visit_ts_tuple_type(&mut self, ty: &mut TSTupleType<'a>) {
        for element in ty.element_types.iter_mut() {
            self.visit_ts_tuple_element(element);
        }
    }

    fn visit_ts_tuple_element(&mut self, ty: &mut TSTupleElement<'a>) {
        match ty {
            TSTupleElement::TSType(ty) => self.visit_ts_type(ty),
            TSTupleElement::TSOptionalType(ty) => self.visit_ts_type(&mut ty.type_annotation),
            TSTupleElement::TSRestType(ty) => self.visit_ts_type(&mut ty.type_annotation),
            TSTupleElement::TSNamedTupleMember(ty) => self.visit_ts_type(&mut ty.element_type),
        };
    }

    fn visit_ts_mapped_type(&mut self, ty: &mut TSMappedType<'a>) {
        self.visit_ts_type_parameter(&mut ty.type_parameter);
        if let Some(name) = &mut ty.name_type {
            self.visit_ts_type(name);
        }
        if let Some(type_annotation) = &mut ty.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    fn visit_ts_function_type(&mut self, ty: &mut TSFunctionType<'a>) {
        self.visit_formal_parameters(&mut ty.params);
        if let Some(parameters) = &mut ty.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type_annotation(&mut ty.return_type);
    }

    fn visit_ts_type_parameter(&mut self, ty: &mut TSTypeParameter<'a>) {
        if let Some(constraint) = &mut ty.constraint {
            self.visit_ts_type(constraint);
        }

        if let Some(default) = &mut ty.default {
            self.visit_ts_type(default);
        }
    }

    fn visit_ts_type_parameter_instantiation(&mut self, ty: &mut TSTypeParameterInstantiation<'a>) {
        for ts_parameter in ty.params.iter_mut() {
            self.visit_ts_type(ts_parameter);
        }
    }

    fn visit_ts_type_parameter_declaration(&mut self, ty: &mut TSTypeParameterDeclaration<'a>) {
        for ts_parameter in ty.params.iter_mut() {
            self.visit_ts_type_parameter(ts_parameter);
        }
    }

    fn visit_ts_constructor_type(&mut self, ty: &mut TSConstructorType<'a>) {
        self.visit_formal_parameters(&mut ty.params);
        if let Some(parameters) = &mut ty.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type_annotation(&mut ty.return_type);
    }

    fn visit_ts_conditional_type(&mut self, ty: &mut TSConditionalType<'a>) {
        self.visit_ts_type(&mut ty.check_type);
        self.visit_ts_type(&mut ty.extends_type);
        self.visit_ts_type(&mut ty.true_type);
        self.visit_ts_type(&mut ty.false_type);
    }

    fn visit_ts_array_type(&mut self, ty: &mut TSArrayType<'a>) {
        self.visit_ts_type(&mut ty.element_type);
    }

    fn visit_ts_type_name(&mut self, name: &mut TSTypeName<'a>) {
        match name {
            TSTypeName::IdentifierReference(ident) => self.visit_identifier_reference(ident),
            TSTypeName::QualifiedName(_) => {}
        }
    }

    fn visit_ts_null_keyword(&mut self, _ty: &mut TSNullKeyword) {}

    fn visit_ts_any_keyword(&mut self, _ty: &mut TSAnyKeyword) {}

    fn visit_ts_void_keyword(&mut self, _ty: &mut TSVoidKeyword) {}

    fn visit_ts_intersection_type(&mut self, ty: &mut TSIntersectionType<'a>) {
        for ty in ty.types.iter_mut() {
            self.visit_ts_type(ty);
        }
    }

    fn visit_ts_type_reference(&mut self, ty: &mut TSTypeReference<'a>) {
        self.visit_ts_type_name(&mut ty.type_name);
        if let Some(parameters) = &mut ty.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
    }

    fn visit_ts_union_type(&mut self, ty: &mut TSUnionType<'a>) {
        for ty in ty.types.iter_mut() {
            self.visit_ts_type(ty);
        }
    }

    fn visit_ts_literal_type(&mut self, ty: &mut TSLiteralType<'a>) {
        match &mut ty.literal {
            TSLiteral::BigintLiteral(lit) => self.visit_bigint_literal(lit),
            TSLiteral::BooleanLiteral(lit) => self.visit_boolean_literal(lit),
            TSLiteral::NullLiteral(lit) => self.visit_null_literal(lit),
            TSLiteral::NumberLiteral(lit) => self.visit_number_literal(lit),
            TSLiteral::RegExpLiteral(lit) => self.visit_reg_expr_literal(lit),
            TSLiteral::StringLiteral(lit) => self.visit_string_literal(lit),
            TSLiteral::TemplateLiteral(lit) => self.visit_template_literal(lit),
            TSLiteral::UnaryExpression(expr) => self.visit_unary_expression(expr),
        }
    }

    fn visit_ts_signature(&mut self, signature: &mut TSSignature<'a>) {
        match signature {
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
        signature: &mut TSConstructSignatureDeclaration<'a>,
    ) {
        self.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_method_signature(&mut self, signature: &mut TSMethodSignature<'a>) {
        self.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_index_signature_name(&mut self, name: &mut TSIndexSignatureName<'a>) {
        self.visit_ts_type_annotation(&mut name.type_annotation);
    }

    fn visit_ts_index_signature(&mut self, signature: &mut TSIndexSignature<'a>) {
        for name in signature.parameters.iter_mut() {
            self.visit_ts_index_signature_name(name);
        }

        self.visit_ts_type_annotation(&mut signature.type_annotation);
    }

    fn visit_ts_property_signature(&mut self, signature: &mut TSPropertySignature<'a>) {
        self.visit_property_key(&mut signature.key);
        if let Some(annotation) = &mut signature.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_call_signature_declaration(
        &mut self,
        signature: &mut TSCallSignatureDeclaration<'a>,
    ) {
        self.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(annotation) = &mut signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }
}
