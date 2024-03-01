//! Visitor Pattern
//!
//! See:
//! * [visitor pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)
//! * [rustc visitor](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/visit.rs)

use oxc_allocator::Vec;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::{ast::*, ast_kind::AstKind};

/// Syntax tree traversal
pub trait Visit<'a>: Sized {
    fn enter_node(&mut self, _kind: AstKind<'a>) {}
    fn leave_node(&mut self, _kind: AstKind<'a>) {}

    fn enter_scope(&mut self, _flags: ScopeFlags) {}
    fn leave_scope(&mut self) {}

    fn alloc<T>(&self, t: &T) -> &'a T {
        // SAFETY:
        // This should be safe as long as `src` is an reference from the allocator.
        // But honestly, I'm not really sure if this is safe.
        unsafe { std::mem::transmute(t) }
    }

    fn visit_program(&mut self, program: &Program<'a>) {
        let kind = AstKind::Program(self.alloc(program));
        self.enter_scope({
            let mut flags = ScopeFlags::Top;
            if program.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        self.enter_node(kind);
        for directive in &program.directives {
            self.visit_directive(directive);
        }
        self.visit_statements(&program.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    /* ----------  Statement ---------- */

    fn visit_statements(&mut self, stmts: &Vec<'a, Statement<'a>>) {
        for stmt in stmts {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        self.visit_statement_match(stmt);
    }

    fn visit_statement_match(&mut self, stmt: &Statement<'a>) {
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

    fn visit_block_statement(&mut self, stmt: &BlockStatement<'a>) {
        let kind = AstKind::BlockStatement(self.alloc(stmt));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);
        self.visit_statements(&stmt.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_break_statement(&mut self, stmt: &BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(self.alloc(stmt));
        self.enter_node(kind);
        if let Some(break_target) = &stmt.label {
            self.visit_label_identifier(break_target);
        }
        self.leave_node(kind);
    }

    fn visit_continue_statement(&mut self, stmt: &ContinueStatement<'a>) {
        let kind = AstKind::ContinueStatement(self.alloc(stmt));
        self.enter_node(kind);
        if let Some(continue_target) = &stmt.label {
            self.visit_label_identifier(continue_target);
        }
        self.leave_node(kind);
    }

    fn visit_debugger_statement(&mut self, stmt: &DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_statement(&stmt.body);
        self.visit_expression(&stmt.test);
        self.leave_node(kind);
    }

    fn visit_empty_statement(&mut self, stmt: &EmptyStatement) {
        let kind = AstKind::EmptyStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        let kind = AstKind::ExpressionStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.expression);
        self.leave_node(kind);
    }

    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        let kind = AstKind::ForStatement(self.alloc(stmt));
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration);
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
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
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_statement_init(&mut self, init: &ForStatementInit<'a>) {
        let kind = AstKind::ForStatementInit(self.alloc(init));
        self.enter_node(kind);
        match init {
            ForStatementInit::UsingDeclaration(decl) => {
                self.visit_using_declaration(decl);
            }
            ForStatementInit::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            ForStatementInit::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);
        self.visit_expression(&stmt.right);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
        self.visit_for_statement_left(&stmt.left);
        self.visit_expression(&stmt.right);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_statement_left(&mut self, left: &ForStatementLeft<'a>) {
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

    fn visit_if_statement(&mut self, stmt: &IfStatement<'a>) {
        let kind = AstKind::IfStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.test);
        self.visit_statement(&stmt.consequent);
        if let Some(alternate) = &stmt.alternate {
            self.visit_statement(alternate);
        }
        self.leave_node(kind);
    }

    fn visit_labeled_statement(&mut self, stmt: &LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_label_identifier(&stmt.label);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(self.alloc(stmt));
        self.enter_node(kind);
        if let Some(arg) = &stmt.argument {
            self.visit_expression(arg);
        }
        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.discriminant);
        self.enter_scope(ScopeFlags::empty());
        for case in &stmt.cases {
            self.visit_switch_case(case);
        }
        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_switch_case(&mut self, case: &SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(self.alloc(case));
        self.enter_node(kind);
        if let Some(expr) = &case.test {
            self.visit_expression(expr);
        }
        self.visit_statements(&case.consequent);
        self.leave_node(kind);
    }

    fn visit_throw_statement(&mut self, stmt: &ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.argument);
        self.leave_node(kind);
    }

    fn visit_try_statement(&mut self, stmt: &TryStatement<'a>) {
        let kind = AstKind::TryStatement(self.alloc(stmt));
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

    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        let kind = AstKind::CatchClause(self.alloc(clause));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);
        if let Some(param) = &clause.param {
            self.visit_binding_pattern(param);
        }
        self.visit_statements(&clause.body.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_finally_clause(&mut self, clause: &BlockStatement<'a>) {
        let kind = AstKind::FinallyClause(self.alloc(clause));
        self.enter_node(kind);
        self.visit_block_statement(clause);
        self.leave_node(kind);
    }

    fn visit_while_statement(&mut self, stmt: &WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.test);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_with_statement(&mut self, stmt: &WithStatement<'a>) {
        let kind = AstKind::WithStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&stmt.object);
        self.visit_statement(&stmt.body);
        self.leave_node(kind);
    }

    fn visit_directive(&mut self, directive: &Directive<'a>) {
        let kind = AstKind::Directive(self.alloc(directive));
        self.enter_node(kind);
        self.visit_string_literal(&directive.expression);
        self.leave_node(kind);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        let kind = AstKind::VariableDeclaration(self.alloc(decl));
        self.enter_node(kind);
        for declarator in &decl.declarations {
            self.visit_variable_declarator(declarator);
        }
        self.leave_node(kind);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let kind = AstKind::VariableDeclarator(self.alloc(declarator));
        self.enter_node(kind);
        self.visit_binding_pattern(&declarator.id);
        if let Some(init) = &declarator.init {
            self.visit_expression(init);
        }
        self.leave_node(kind);
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &Function<'a>, flags: Option<ScopeFlags>) {
        let kind = AstKind::Function(self.alloc(func));
        self.enter_scope({
            let mut flags = flags.unwrap_or(ScopeFlags::empty()) | ScopeFlags::Function;
            if func.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
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
        self.leave_scope();
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        let kind = AstKind::FunctionBody(self.alloc(body));
        self.enter_node(kind);
        for directive in &body.directives {
            self.visit_directive(directive);
        }
        self.visit_statements(&body.statements);
        self.leave_node(kind);
    }

    fn visit_formal_parameters(&mut self, params: &FormalParameters<'a>) {
        let kind = AstKind::FormalParameters(self.alloc(params));
        self.enter_node(kind);
        for param in &params.items {
            self.visit_formal_parameter(param);
        }
        if let Some(rest) = &params.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_formal_parameter(&mut self, param: &FormalParameter<'a>) {
        let kind = AstKind::FormalParameter(self.alloc(param));
        self.enter_node(kind);
        for decorator in &param.decorators {
            self.visit_decorator(decorator);
        }
        self.visit_binding_pattern(&param.pattern);
        self.leave_node(kind);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &Decorator<'a>) {
        let kind = AstKind::Decorator(self.alloc(decorator));
        self.enter_node(kind);
        self.visit_expression(&decorator.expression);
        self.leave_node(kind);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        // Class level decorators are transpiled as functions outside of the class taking the class
        // itself as argument. They should be visited before class is entered. E.g., they inherit
        // strict mode from the enclosing scope rather than from class.
        for decorator in &class.decorators {
            self.visit_decorator(decorator);
        }
        let kind = AstKind::Class(self.alloc(class));

        // FIXME(don): Should we enter a scope when visiting class declarations?
        let is_class_expr = class.r#type == ClassType::ClassExpression;
        if is_class_expr {
            // Class expressions create a temporary scope with the class name as its only variable
            // E.g., `let c = class A { foo() { console.log(A) } }`
            self.enter_scope(ScopeFlags::empty());
        }

        self.enter_node(kind);

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
        if is_class_expr {
            self.leave_scope();
        }
    }

    fn visit_class_heritage(&mut self, expr: &Expression<'a>) {
        let kind = AstKind::ClassHeritage(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(expr);
        self.leave_node(kind);
    }

    fn visit_class_body(&mut self, body: &ClassBody<'a>) {
        let kind = AstKind::ClassBody(self.alloc(body));
        self.enter_node(kind);
        for elem in &body.body {
            self.visit_class_element(elem);
        }
        self.leave_node(kind);
    }

    fn visit_class_element(&mut self, elem: &ClassElement<'a>) {
        match elem {
            ClassElement::StaticBlock(block) => self.visit_static_block(block),
            ClassElement::MethodDefinition(def) => self.visit_method_definition(def),
            ClassElement::PropertyDefinition(def) => self.visit_property_definition(def),
            ClassElement::AccessorProperty(_def) => { /* TODO */ }
            ClassElement::TSIndexSignature(_def) => {}
        }
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(self.alloc(block));
        self.enter_scope(ScopeFlags::ClassStaticBlock);
        self.enter_node(kind);
        self.visit_statements(&block.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_method_definition(&mut self, def: &MethodDefinition<'a>) {
        let kind = AstKind::MethodDefinition(self.alloc(def));
        self.enter_node(kind);
        for decorator in &def.decorators {
            self.visit_decorator(decorator);
        }
        let flags = match def.kind {
            MethodDefinitionKind::Get => ScopeFlags::GetAccessor,
            MethodDefinitionKind::Set => ScopeFlags::SetAccessor,
            MethodDefinitionKind::Constructor => ScopeFlags::Constructor,
            MethodDefinitionKind::Method => ScopeFlags::empty(),
        };
        self.visit_property_key(&def.key);
        self.visit_function(&def.value, Some(flags));
        self.leave_node(kind);
    }

    fn visit_property_definition(&mut self, def: &PropertyDefinition<'a>) {
        let kind = AstKind::PropertyDefinition(self.alloc(def));
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

    fn visit_using_declaration(&mut self, decl: &UsingDeclaration<'a>) {
        let kind = AstKind::UsingDeclaration(self.alloc(decl));
        self.enter_node(kind);
        for decl in &decl.declarations {
            self.visit_variable_declarator(decl);
        }
        self.leave_node(kind);
    }

    /* ----------  Expression ---------- */

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        self.visit_expression_match(expr);
    }

    fn visit_expression_match(&mut self, expr: &Expression<'a>) {
        match expr {
            Expression::BigintLiteral(lit) => self.visit_bigint_literal(lit),
            Expression::BooleanLiteral(lit) => self.visit_boolean_literal(lit),
            Expression::NullLiteral(lit) => self.visit_null_literal(lit),
            Expression::NumericLiteral(lit) => self.visit_number_literal(lit),
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
            Expression::ClassExpression(expr) => {
                debug_assert_eq!(expr.r#type, ClassType::ClassExpression);
                self.visit_class(expr);
            }
            Expression::ConditionalExpression(expr) => self.visit_conditional_expression(expr),
            Expression::FunctionExpression(expr) => self.visit_function(expr, None),
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

    fn visit_meta_property(&mut self, meta: &MetaProperty<'a>) {
        let kind = AstKind::MetaProperty(self.alloc(meta));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_array_expression(&mut self, expr: &ArrayExpression<'a>) {
        let kind = AstKind::ArrayExpression(self.alloc(expr));
        self.enter_node(kind);
        for elem in &expr.elements {
            self.visit_array_expression_element(elem);
        }
        self.leave_node(kind);
    }

    fn visit_array_expression_element(&mut self, arg: &ArrayExpressionElement<'a>) {
        let kind = AstKind::ArrayExpressionElement(self.alloc(arg));
        self.enter_node(kind);
        match arg {
            ArrayExpressionElement::SpreadElement(spread) => self.visit_spread_element(spread),
            ArrayExpressionElement::Expression(expr) => self.visit_expression_array_element(expr),
            ArrayExpressionElement::Elision(span) => self.visit_elision(*span),
        }
        self.leave_node(kind);
    }

    fn visit_argument(&mut self, arg: &Argument<'a>) {
        let kind = AstKind::Argument(self.alloc(arg));
        self.enter_node(kind);
        match arg {
            Argument::SpreadElement(spread) => self.visit_spread_element(spread),
            Argument::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_spread_element(&mut self, elem: &SpreadElement<'a>) {
        let kind = AstKind::SpreadElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_expression(&elem.argument);
        self.leave_node(kind);
    }

    fn visit_expression_array_element(&mut self, expr: &Expression<'a>) {
        let kind = AstKind::ExpressionArrayElement(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(expr);
        self.leave_node(kind);
    }

    fn visit_elision(&mut self, span: Span) {
        let kind = AstKind::Elision(span);
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        let kind = AstKind::AssignmentExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_assignment_target(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_arrow_expression(&mut self, expr: &ArrowFunctionExpression<'a>) {
        let kind = AstKind::ArrowFunctionExpression(self.alloc(expr));
        self.enter_scope(ScopeFlags::Function | ScopeFlags::Arrow);
        self.enter_node(kind);
        self.visit_formal_parameters(&expr.params);
        self.visit_function_body(&expr.body);
        if let Some(parameters) = &expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_await_expression(&mut self, expr: &AwaitExpression<'a>) {
        let kind = AstKind::AwaitExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        let kind = AstKind::BinaryExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        let kind = AstKind::CallExpression(self.alloc(expr));
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

    fn visit_chain_expression(&mut self, expr: &ChainExpression<'a>) {
        let kind = AstKind::ChainExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_chain_element(&expr.expression);
        self.leave_node(kind);
    }

    fn visit_chain_element(&mut self, elem: &ChainElement<'a>) {
        match elem {
            ChainElement::CallExpression(expr) => self.visit_call_expression(expr),
            ChainElement::MemberExpression(expr) => self.visit_member_expression(expr),
        }
    }

    fn visit_conditional_expression(&mut self, expr: &ConditionalExpression<'a>) {
        let kind = AstKind::ConditionalExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.test);
        self.visit_expression(&expr.consequent);
        self.visit_expression(&expr.alternate);
        self.leave_node(kind);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        let kind = AstKind::ImportExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.source);
        for arg in &expr.arguments {
            self.visit_expression(arg);
        }
        self.leave_node(kind);
    }

    fn visit_logical_expression(&mut self, expr: &LogicalExpression<'a>) {
        let kind = AstKind::LogicalExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_member_expression(&mut self, expr: &MemberExpression<'a>) {
        let kind = AstKind::MemberExpression(self.alloc(expr));
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

    fn visit_computed_member_expression(&mut self, expr: &ComputedMemberExpression<'a>) {
        self.visit_expression(&expr.object);
        self.visit_expression(&expr.expression);
    }

    fn visit_static_member_expression(&mut self, expr: &StaticMemberExpression<'a>) {
        self.visit_expression(&expr.object);
        self.visit_identifier_name(&expr.property);
    }

    fn visit_private_field_expression(&mut self, expr: &PrivateFieldExpression<'a>) {
        self.visit_expression(&expr.object);
        self.visit_private_identifier(&expr.field);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        let kind = AstKind::NewExpression(self.alloc(expr));
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

    fn visit_object_expression(&mut self, expr: &ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(self.alloc(expr));
        self.enter_node(kind);
        for prop in &expr.properties {
            self.visit_object_property_kind(prop);
        }
        self.leave_node(kind);
    }

    fn visit_object_property_kind(&mut self, prop: &ObjectPropertyKind<'a>) {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => self.visit_object_property(prop),
            ObjectPropertyKind::SpreadProperty(elem) => self.visit_spread_element(elem),
        }
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        let kind = AstKind::ObjectProperty(self.alloc(prop));
        self.enter_node(kind);
        self.visit_property_key(&prop.key);
        self.visit_expression(&prop.value);
        self.leave_node(kind);
    }

    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
        let kind = AstKind::PropertyKey(self.alloc(key));
        self.enter_node(kind);
        match key {
            PropertyKey::Identifier(ident) => self.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => self.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_parenthesized_expression(&mut self, expr: &ParenthesizedExpression<'a>) {
        let kind = AstKind::ParenthesizedExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.leave_node(kind);
    }

    fn visit_private_in_expression(&mut self, expr: &PrivateInExpression<'a>) {
        let kind = AstKind::PrivateInExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_private_identifier(&expr.left);
        self.visit_expression(&expr.right);
        self.leave_node(kind);
    }

    fn visit_sequence_expression(&mut self, expr: &SequenceExpression<'a>) {
        let kind = AstKind::SequenceExpression(self.alloc(expr));
        self.enter_node(kind);
        for expr in &expr.expressions {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_tagged_template_expression(&mut self, expr: &TaggedTemplateExpression<'a>) {
        let kind = AstKind::TaggedTemplateExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.tag);
        self.visit_template_literal(&expr.quasi);
        self.leave_node(kind);
    }

    fn visit_this_expression(&mut self, expr: &ThisExpression) {
        let kind = AstKind::ThisExpression(self.alloc(expr));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_unary_expression(&mut self, expr: &UnaryExpression<'a>) {
        let kind = AstKind::UnaryExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_update_expression(&mut self, expr: &UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_simple_assignment_target(&expr.argument);
        self.leave_node(kind);
    }

    fn visit_yield_expression(&mut self, expr: &YieldExpression<'a>) {
        let kind = AstKind::YieldExpression(self.alloc(expr));
        self.enter_node(kind);
        if let Some(argument) = &expr.argument {
            self.visit_expression(argument);
        }
        self.leave_node(kind);
    }

    fn visit_super(&mut self, expr: &Super) {
        let kind = AstKind::Super(self.alloc(expr));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        let kind = AstKind::AssignmentTarget(self.alloc(target));
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

    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        let kind = AstKind::SimpleAssignmentTarget(self.alloc(target));
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
            SimpleAssignmentTarget::TSSatisfiesExpression(expr) => {
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

    fn visit_assignment_target_pattern(&mut self, pat: &AssignmentTargetPattern<'a>) {
        match pat {
            AssignmentTargetPattern::ArrayAssignmentTarget(target) => {
                self.visit_array_assignment_target(target);
            }
            AssignmentTargetPattern::ObjectAssignmentTarget(target) => {
                self.visit_object_assignment_target(target);
            }
        }
    }

    fn visit_array_assignment_target(&mut self, target: &ArrayAssignmentTarget<'a>) {
        for element in target.elements.iter().flatten() {
            self.visit_assignment_target_maybe_default(element);
        }
        if let Some(target) = &target.rest {
            self.visit_assignment_target(target);
        }
    }

    fn visit_assignment_target_maybe_default(&mut self, target: &AssignmentTargetMaybeDefault<'a>) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTarget(target) => {
                self.visit_assignment_target(target);
            }
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                self.visit_assignment_target_with_default(target);
            }
        }
    }

    fn visit_assignment_target_with_default(&mut self, target: &AssignmentTargetWithDefault<'a>) {
        let kind = AstKind::AssignmentTargetWithDefault(self.alloc(target));
        self.enter_node(kind);
        self.visit_assignment_target(&target.binding);
        self.visit_expression(&target.init);
        self.leave_node(kind);
    }

    fn visit_object_assignment_target(&mut self, target: &ObjectAssignmentTarget<'a>) {
        for property in &target.properties {
            self.visit_assignment_target_property(property);
        }
        if let Some(target) = &target.rest {
            self.visit_assignment_target(target);
        }
    }

    fn visit_assignment_target_property(&mut self, property: &AssignmentTargetProperty<'a>) {
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
        ident: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.visit_identifier_reference(&ident.binding);
        if let Some(expr) = &ident.init {
            self.visit_expression(expr);
        }
    }

    fn visit_assignment_target_property_property(
        &mut self,
        property: &AssignmentTargetPropertyProperty<'a>,
    ) {
        self.visit_property_key(&property.name);
        self.visit_assignment_target_maybe_default(&property.binding);
    }

    /* ----------  Expression ---------- */

    fn visit_jsx_element(&mut self, elem: &JSXElement<'a>) {
        let kind = AstKind::JSXElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_jsx_opening_element(&elem.opening_element);
        for child in &elem.children {
            self.visit_jsx_child(child);
        }
        if let Some(closing_elem) = &elem.closing_element {
            self.visit_jsx_closing_element(closing_elem);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_opening_element(&mut self, elem: &JSXOpeningElement<'a>) {
        let kind = AstKind::JSXOpeningElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_jsx_element_name(&elem.name);
        for attribute in &elem.attributes {
            self.visit_jsx_attribute_item(attribute);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_closing_element(&mut self, elem: &JSXClosingElement<'a>) {
        let kind = AstKind::JSXClosingElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_jsx_element_name(&elem.name);
        self.leave_node(kind);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        let kind = AstKind::JSXElementName(self.alloc(name));
        self.enter_node(kind);
        match name {
            JSXElementName::Identifier(ident) => self.visit_jsx_identifier(ident),
            JSXElementName::NamespacedName(expr) => self.visit_jsx_namespaced_name(expr),
            JSXElementName::MemberExpression(expr) => self.visit_jsx_member_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_jsx_identifier(&mut self, ident: &JSXIdentifier<'a>) {
        let kind = AstKind::JSXIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_jsx_member_expression(&mut self, expr: &JSXMemberExpression<'a>) {
        let kind = AstKind::JSXMemberExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_jsx_member_expression_object(&expr.object);
        self.visit_jsx_identifier(&expr.property);
        self.leave_node(kind);
    }

    fn visit_jsx_member_expression_object(&mut self, expr: &JSXMemberExpressionObject<'a>) {
        let kind = AstKind::JSXMemberExpressionObject(self.alloc(expr));
        self.enter_node(kind);
        match expr {
            JSXMemberExpressionObject::Identifier(ident) => self.visit_jsx_identifier(ident),
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.visit_jsx_member_expression(expr);
            }
        }
        self.leave_node(kind);
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        let kind = AstKind::JSXNamespacedName(self.alloc(name));
        self.enter_node(kind);
        self.visit_jsx_identifier(&name.namespace);
        self.visit_jsx_identifier(&name.property);
        self.leave_node(kind);
    }

    fn visit_jsx_attribute_item(&mut self, item: &JSXAttributeItem<'a>) {
        let kind = AstKind::JSXAttributeItem(self.alloc(item));
        self.enter_node(kind);
        match &item {
            JSXAttributeItem::Attribute(attribute) => self.visit_jsx_attribute(attribute),
            JSXAttributeItem::SpreadAttribute(attribute) => {
                self.visit_jsx_spread_attribute(attribute);
            }
        }
        self.leave_node(kind);
    }

    fn visit_jsx_attribute(&mut self, attribute: &JSXAttribute<'a>) {
        if let Some(value) = &attribute.value {
            self.visit_jsx_attribute_value(value);
        }
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &JSXSpreadAttribute<'a>) {
        self.visit_expression(&attribute.argument);
    }

    fn visit_jsx_attribute_value(&mut self, value: &JSXAttributeValue<'a>) {
        match value {
            JSXAttributeValue::ExpressionContainer(expr) => {
                self.visit_jsx_expression_container(expr);
            }
            JSXAttributeValue::Element(elem) => self.visit_jsx_element(elem),
            JSXAttributeValue::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXAttributeValue::StringLiteral(lit) => self.visit_string_literal(lit),
        }
    }

    fn visit_jsx_expression_container(&mut self, expr: &JSXExpressionContainer<'a>) {
        let kind = AstKind::JSXExpressionContainer(self.alloc(expr));
        self.enter_node(kind);
        self.visit_jsx_expression(&expr.expression);
        self.leave_node(kind);
    }

    fn visit_jsx_expression(&mut self, expr: &JSXExpression<'a>) {
        match expr {
            JSXExpression::Expression(expr) => self.visit_expression(expr),
            JSXExpression::EmptyExpression(_) => {}
        }
    }

    fn visit_jsx_fragment(&mut self, elem: &JSXFragment<'a>) {
        let kind = AstKind::JSXFragment(self.alloc(elem));
        self.enter_node(kind);
        for child in &elem.children {
            self.visit_jsx_child(child);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_child(&mut self, child: &JSXChild<'a>) {
        match child {
            JSXChild::Element(elem) => self.visit_jsx_element(elem),
            JSXChild::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXChild::ExpressionContainer(expr) => self.visit_jsx_expression_container(expr),
            JSXChild::Spread(expr) => self.visit_jsx_spread_child(expr),
            JSXChild::Text(expr) => self.visit_jsx_text(expr),
        }
    }

    fn visit_jsx_spread_child(&mut self, child: &JSXSpreadChild<'a>) {
        self.visit_expression(&child.expression);
    }

    fn visit_jsx_text(&mut self, child: &JSXText<'a>) {
        let kind = AstKind::JSXText(self.alloc(child));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    /* ----------  Pattern ---------- */

    fn visit_binding_pattern(&mut self, pat: &BindingPattern<'a>) {
        match &pat.kind {
            BindingPatternKind::BindingIdentifier(ident) => {
                self.visit_binding_identifier(ident);
            }
            BindingPatternKind::ObjectPattern(pat) => self.visit_object_pattern(pat),
            BindingPatternKind::ArrayPattern(pat) => self.visit_array_pattern(pat),
            BindingPatternKind::AssignmentPattern(pat) => self.visit_assignment_pattern(pat),
        }
        if let Some(type_annotation) = &pat.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        let kind = AstKind::BindingIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_object_pattern(&mut self, pat: &ObjectPattern<'a>) {
        let kind = AstKind::ObjectPattern(self.alloc(pat));
        self.enter_node(kind);
        for prop in &pat.properties {
            self.visit_binding_property(prop);
        }
        if let Some(rest) = &pat.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_binding_property(&mut self, prop: &BindingProperty<'a>) {
        self.visit_property_key(&prop.key);
        self.visit_binding_pattern(&prop.value);
    }

    fn visit_array_pattern(&mut self, pat: &ArrayPattern<'a>) {
        let kind = AstKind::ArrayPattern(self.alloc(pat));
        self.enter_node(kind);
        for pat in pat.elements.iter().flatten() {
            self.visit_binding_pattern(pat);
        }
        if let Some(rest) = &pat.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_rest_element(&mut self, pat: &BindingRestElement<'a>) {
        let kind = AstKind::BindingRestElement(self.alloc(pat));
        self.enter_node(kind);
        self.visit_binding_pattern(&pat.argument);
        self.leave_node(kind);
    }

    fn visit_assignment_pattern(&mut self, pat: &AssignmentPattern<'a>) {
        let kind = AstKind::AssignmentPattern(self.alloc(pat));
        self.enter_node(kind);
        self.visit_binding_pattern(&pat.left);
        self.visit_expression(&pat.right);
        self.leave_node(kind);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        let kind = AstKind::IdentifierReference(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_private_identifier(&mut self, ident: &PrivateIdentifier<'a>) {
        let kind = AstKind::PrivateIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_label_identifier(&mut self, ident: &LabelIdentifier<'a>) {
        let kind = AstKind::LabelIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_identifier_name(&mut self, ident: &IdentifierName<'a>) {
        let kind = AstKind::IdentifierName(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, lit: &NumericLiteral<'a>) {
        let kind = AstKind::NumericLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_boolean_literal(&mut self, lit: &BooleanLiteral) {
        let kind = AstKind::BooleanLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_null_literal(&mut self, lit: &NullLiteral) {
        let kind = AstKind::NullLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_bigint_literal(&mut self, lit: &BigintLiteral<'a>) {
        let kind = AstKind::BigintLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_string_literal(&mut self, lit: &StringLiteral<'a>) {
        let kind = AstKind::StringLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(self.alloc(lit));
        self.enter_node(kind);
        for elem in &lit.quasis {
            self.visit_template_element(elem);
        }
        for expr in &lit.expressions {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_reg_expr_literal(&mut self, lit: &RegExpLiteral<'a>) {
        let kind = AstKind::RegExpLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_element(&mut self, _elem: &TemplateElement) {}

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &ModuleDeclaration<'a>) {
        let kind = AstKind::ModuleDeclaration(self.alloc(decl));
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
            ModuleDeclaration::TSExportAssignment(decl) => {
                self.visit_expression(&decl.expression);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(_) => {}
        }
        self.leave_node(kind);
    }

    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let kind = AstKind::ImportDeclaration(self.alloc(decl));
        self.enter_node(kind);
        if let Some(specifiers) = &decl.specifiers {
            for specifier in specifiers {
                self.visit_import_declaration_specifier(specifier);
            }
        }
        self.visit_string_literal(&decl.source);
        // TODO: assertions
        self.leave_node(kind);
    }

    fn visit_import_declaration_specifier(&mut self, specifier: &ImportDeclarationSpecifier<'a>) {
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

    fn visit_import_specifier(&mut self, specifier: &ImportSpecifier<'a>) {
        let kind = AstKind::ImportSpecifier(self.alloc(specifier));
        self.enter_node(kind);
        // TODO: imported
        self.visit_binding_identifier(&specifier.local);
        self.leave_node(kind);
    }

    fn visit_import_default_specifier(&mut self, specifier: &ImportDefaultSpecifier<'a>) {
        let kind = AstKind::ImportDefaultSpecifier(self.alloc(specifier));
        self.enter_node(kind);
        self.visit_binding_identifier(&specifier.local);
        self.leave_node(kind);
    }

    fn visit_import_name_specifier(&mut self, specifier: &ImportNamespaceSpecifier<'a>) {
        let kind = AstKind::ImportNamespaceSpecifier(self.alloc(specifier));
        self.enter_node(kind);
        self.visit_binding_identifier(&specifier.local);
        self.leave_node(kind);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let kind = AstKind::ExportAllDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_string_literal(&decl.source);
        self.leave_node(kind);
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        let kind = AstKind::ExportDefaultDeclaration(self.alloc(decl));
        self.enter_node(kind);
        match &decl.declaration {
            ExportDefaultDeclarationKind::Expression(expr) => self.visit_expression(expr),
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                self.visit_function(func, None);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => self.visit_class(class),
            _ => {}
        }
        self.leave_node(kind);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        let kind = AstKind::ExportNamedDeclaration(self.alloc(decl));
        self.enter_node(kind);
        if let Some(decl) = &decl.declaration {
            self.visit_declaration(decl);
        }
        if let Some(ref source) = decl.source {
            self.visit_string_literal(source);
        }
        self.leave_node(kind);
    }

    fn visit_enum_member(&mut self, member: &TSEnumMember<'a>) {
        let kind = AstKind::TSEnumMember(self.alloc(member));
        self.enter_node(kind);

        if let Some(initializer) = &member.initializer {
            self.visit_expression(initializer);
        }

        self.leave_node(kind);
    }

    fn visit_enum(&mut self, decl: &TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        self.enter_scope(ScopeFlags::empty());
        for member in &decl.members {
            self.visit_enum_member(member);
        }
        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_declaration(&mut self, decl: &Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            Declaration::FunctionDeclaration(func) => self.visit_function(func, None),
            Declaration::ClassDeclaration(class) => {
                debug_assert_eq!(class.r#type, ClassType::ClassDeclaration);
                self.visit_class(class);
            }
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

    fn visit_ts_import_equals_declaration(&mut self, decl: &TSImportEqualsDeclaration<'a>) {
        let kind = AstKind::TSImportEqualsDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        self.visit_ts_module_reference(&decl.module_reference);
        self.leave_node(kind);
    }

    fn visit_ts_module_reference(&mut self, reference: &TSModuleReference<'a>) {
        match reference {
            TSModuleReference::TypeName(name) => self.visit_ts_type_name(name),
            TSModuleReference::ExternalModuleReference(reference) => {
                self.visit_ts_external_module_reference(reference);
            }
        }
    }

    fn visit_ts_type_name(&mut self, name: &TSTypeName<'a>) {
        let kind = AstKind::TSTypeName(self.alloc(name));
        self.enter_node(kind);
        match &name {
            TSTypeName::IdentifierReference(ident) => self.visit_identifier_reference(ident),
            TSTypeName::QualifiedName(name) => self.visit_ts_qualified_name(name),
        }
        self.leave_node(kind);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        let kind = AstKind::TSExternalModuleReference(self.alloc(reference));
        self.enter_node(kind);
        self.visit_string_literal(&reference.expression);
        self.leave_node(kind);
    }

    fn visit_ts_qualified_name(&mut self, name: &TSQualifiedName<'a>) {
        let kind = AstKind::TSQualifiedName(self.alloc(name));
        self.enter_node(kind);
        self.visit_ts_type_name(&name.left);
        self.visit_identifier_name(&name.right);
        self.leave_node(kind);
    }

    fn visit_ts_module_declaration(&mut self, decl: &TSModuleDeclaration<'a>) {
        let kind = AstKind::TSModuleDeclaration(self.alloc(decl));
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

    fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
        let kind = AstKind::TSModuleBlock(self.alloc(block));
        self.enter_scope(ScopeFlags::TsModuleBlock);
        self.enter_node(kind);
        self.visit_statements(&block.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        let kind = AstKind::TSTypeAliasDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&decl.id);
        if let Some(parameters) = &decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type(&decl.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        let kind = AstKind::TSInterfaceDeclaration(self.alloc(decl));
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

    fn visit_ts_as_expression(&mut self, expr: &TSAsExpression<'a>) {
        let kind = AstKind::TSAsExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.visit_ts_type(&expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_satisfies_expression(&mut self, expr: &TSSatisfiesExpression<'a>) {
        let kind = AstKind::TSSatisfiesExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.visit_ts_type(&expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_non_null_expression(&mut self, expr: &TSNonNullExpression<'a>) {
        let kind = AstKind::TSNonNullExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.leave_node(kind);
    }

    fn visit_ts_type_assertion(&mut self, expr: &TSTypeAssertion<'a>) {
        let kind = AstKind::TSTypeAssertion(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&expr.expression);
        self.visit_ts_type(&expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_instantiation_expression(&mut self, expr: &TSInstantiationExpression<'a>) {
        self.visit_expression(&expr.expression);
        self.visit_ts_type_parameter_instantiation(&expr.type_parameters);
    }

    fn visit_ts_type_annotation(&mut self, annotation: &TSTypeAnnotation<'a>) {
        let kind = AstKind::TSTypeAnnotation(self.alloc(annotation));
        self.enter_node(kind);
        self.visit_ts_type(&annotation.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_type(&mut self, ty: &TSType<'a>) {
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
            TSType::TSTypeQuery(ty) => self.visit_ts_type_query(ty),
            _ => {}
        }
    }

    fn visit_ts_type_literal(&mut self, ty: &TSTypeLiteral<'a>) {
        let kind = AstKind::TSTypeLiteral(self.alloc(ty));
        self.enter_node(kind);
        for signature in &ty.members {
            self.visit_ts_signature(signature);
        }
        self.leave_node(kind);
    }

    fn visit_ts_indexed_access_type(&mut self, ty: &TSIndexedAccessType<'a>) {
        let kind = AstKind::TSIndexedAccessType(self.alloc(ty));
        self.enter_node(kind);
        self.visit_ts_type(&ty.object_type);
        self.visit_ts_type(&ty.index_type);
        self.leave_node(kind);
    }

    fn visit_ts_type_predicate(&mut self, ty: &TSTypePredicate<'a>) {
        if let Some(annotation) = &ty.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_type_operator_type(&mut self, ty: &TSTypeOperator<'a>) {
        self.visit_ts_type(&ty.type_annotation);
    }

    fn visit_ts_tuple_type(&mut self, ty: &TSTupleType<'a>) {
        for element in &ty.element_types {
            self.visit_ts_tuple_element(element);
        }
    }

    fn visit_ts_tuple_element(&mut self, ty: &TSTupleElement<'a>) {
        match ty {
            TSTupleElement::TSType(ty) => self.visit_ts_type(ty),
            TSTupleElement::TSOptionalType(ty) => self.visit_ts_type(&ty.type_annotation),
            TSTupleElement::TSRestType(ty) => self.visit_ts_type(&ty.type_annotation),
            TSTupleElement::TSNamedTupleMember(ty) => self.visit_ts_type(&ty.element_type),
        };
    }

    fn visit_ts_mapped_type(&mut self, ty: &TSMappedType<'a>) {
        self.visit_ts_type_parameter(&ty.type_parameter);
        if let Some(name) = &ty.name_type {
            self.visit_ts_type(name);
        }
        if let Some(type_annotation) = &ty.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
    }

    fn visit_ts_function_type(&mut self, ty: &TSFunctionType<'a>) {
        self.visit_formal_parameters(&ty.params);
        if let Some(parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type_annotation(&ty.return_type);
    }

    fn visit_ts_type_parameter(&mut self, ty: &TSTypeParameter<'a>) {
        let kind = AstKind::TSTypeParameter(self.alloc(ty));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);
        if let Some(constraint) = &ty.constraint {
            self.visit_ts_type(constraint);
        }

        if let Some(default) = &ty.default {
            self.visit_ts_type(default);
        }
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_ts_type_parameter_instantiation(&mut self, ty: &TSTypeParameterInstantiation<'a>) {
        let kind = AstKind::TSTypeParameterInstantiation(self.alloc(ty));
        self.enter_node(kind);
        for ts_parameter in &ty.params {
            self.visit_ts_type(ts_parameter);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_parameter_declaration(&mut self, ty: &TSTypeParameterDeclaration<'a>) {
        let kind = AstKind::TSTypeParameterDeclaration(self.alloc(ty));
        self.enter_node(kind);
        for ts_parameter in &ty.params {
            self.visit_ts_type_parameter(ts_parameter);
        }
        self.leave_node(kind);
    }

    fn visit_ts_constructor_type(&mut self, ty: &TSConstructorType<'a>) {
        self.visit_formal_parameters(&ty.params);
        if let Some(parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type_annotation(&ty.return_type);
    }

    fn visit_ts_conditional_type(&mut self, ty: &TSConditionalType<'a>) {
        self.visit_ts_type(&ty.check_type);
        self.visit_ts_type(&ty.extends_type);
        self.visit_ts_type(&ty.true_type);
        self.visit_ts_type(&ty.false_type);
    }

    fn visit_ts_array_type(&mut self, ty: &TSArrayType<'a>) {
        self.visit_ts_type(&ty.element_type);
    }

    fn visit_ts_null_keyword(&mut self, ty: &TSNullKeyword) {
        let kind = AstKind::TSNullKeyword(self.alloc(ty));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_any_keyword(&mut self, ty: &TSAnyKeyword) {
        let kind = AstKind::TSAnyKeyword(self.alloc(ty));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_void_keyword(&mut self, ty: &TSVoidKeyword) {
        let kind = AstKind::TSVoidKeyword(self.alloc(ty));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_intersection_type(&mut self, ty: &TSIntersectionType<'a>) {
        let kind = AstKind::TSIntersectionType(self.alloc(ty));
        self.enter_node(kind);
        for ty in &ty.types {
            self.visit_ts_type(ty);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_reference(&mut self, ty: &TSTypeReference<'a>) {
        let kind = AstKind::TSTypeReference(self.alloc(ty));
        self.enter_node(kind);
        self.visit_ts_type_name(&ty.type_name);
        if let Some(parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        self.leave_node(kind);
    }

    fn visit_ts_union_type(&mut self, ty: &TSUnionType<'a>) {
        let kind = AstKind::TSUnionType(self.alloc(ty));
        self.enter_node(kind);
        for ty in &ty.types {
            self.visit_ts_type(ty);
        }
        self.leave_node(kind);
    }

    fn visit_ts_literal_type(&mut self, ty: &TSLiteralType<'a>) {
        let kind = AstKind::TSLiteralType(self.alloc(ty));
        self.enter_node(kind);
        match &ty.literal {
            TSLiteral::BigintLiteral(lit) => self.visit_bigint_literal(lit),
            TSLiteral::BooleanLiteral(lit) => self.visit_boolean_literal(lit),
            TSLiteral::NullLiteral(lit) => self.visit_null_literal(lit),
            TSLiteral::NumericLiteral(lit) => self.visit_number_literal(lit),
            TSLiteral::RegExpLiteral(lit) => self.visit_reg_expr_literal(lit),
            TSLiteral::StringLiteral(lit) => self.visit_string_literal(lit),
            TSLiteral::TemplateLiteral(lit) => self.visit_template_literal(lit),
            TSLiteral::UnaryExpression(expr) => self.visit_unary_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_ts_signature(&mut self, signature: &TSSignature<'a>) {
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
        signature: &TSConstructSignatureDeclaration<'a>,
    ) {
        self.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_method_signature(&mut self, signature: &TSMethodSignature<'a>) {
        let kind = AstKind::TSMethodSignature(self.alloc(signature));
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

    fn visit_ts_index_signature_name(&mut self, name: &TSIndexSignatureName<'a>) {
        self.visit_ts_type_annotation(&name.type_annotation);
    }

    fn visit_ts_index_signature(&mut self, signature: &TSIndexSignature<'a>) {
        for name in &signature.parameters {
            self.visit_ts_index_signature_name(name);
        }

        self.visit_ts_type_annotation(&signature.type_annotation);
    }

    fn visit_ts_property_signature(&mut self, signature: &TSPropertySignature<'a>) {
        let kind = AstKind::TSPropertySignature(self.alloc(signature));
        self.enter_node(kind);
        self.visit_property_key(&signature.key);
        if let Some(annotation) = &signature.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
    }

    fn visit_ts_call_signature_declaration(&mut self, signature: &TSCallSignatureDeclaration<'a>) {
        self.visit_formal_parameters(&signature.params);
        if let Some(parameters) = &signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }

        if let Some(annotation) = &signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        let kind = AstKind::TSTypeQuery(self.alloc(ty));
        self.enter_node(kind);
        self.visit_ts_type_name(&ty.expr_name);
        if let Some(type_parameters) = &ty.type_parameters {
            self.visit_ts_type_parameter_instantiation(type_parameters);
        }
        self.leave_node(kind);
    }
}
