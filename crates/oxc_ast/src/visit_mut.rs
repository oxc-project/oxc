//! Visit Mut Pattern

use oxc_allocator::Vec;
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use crate::{ast::*, AstKind};

/// Syntax tree traversal to mutate an exclusive borrow of a syntax tree in place.
pub trait VisitMut<'a>: Sized {
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

    fn visit_program(&mut self, program: &mut Program<'a>) {
        let kind = AstKind::Program(self.alloc(program));
        self.enter_scope({
            let mut flags = ScopeFlags::Top;
            if program.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        self.enter_node(kind);
        for directive in program.directives.iter_mut() {
            self.visit_directive(directive);
        }
        self.visit_statements(&mut program.body);

        self.leave_node(kind);
        self.leave_scope();
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
        let kind = AstKind::BlockStatement(self.alloc(stmt));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);
        self.visit_statements(&mut stmt.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_break_statement(&mut self, stmt: &mut BreakStatement<'a>) {
        let kind = AstKind::BreakStatement(self.alloc(stmt));
        self.enter_node(kind);
        if let Some(break_target) = &mut stmt.label {
            self.visit_label_identifier(break_target);
        }
        self.leave_node(kind);
    }

    fn visit_continue_statement(&mut self, stmt: &mut ContinueStatement<'a>) {
        let kind = AstKind::ContinueStatement(self.alloc(stmt));
        self.enter_node(kind);
        if let Some(continue_target) = &mut stmt.label {
            self.visit_label_identifier(continue_target);
        }
        self.leave_node(kind);
    }

    fn visit_debugger_statement(&mut self, stmt: &mut DebuggerStatement) {
        let kind = AstKind::DebuggerStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_do_while_statement(&mut self, stmt: &mut DoWhileStatement<'a>) {
        let kind = AstKind::DoWhileStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_statement(&mut stmt.body);
        self.visit_expression(&mut stmt.test);
        self.leave_node(kind);
    }

    fn visit_empty_statement(&mut self, stmt: &mut EmptyStatement) {
        let kind = AstKind::EmptyStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_expression_statement(&mut self, stmt: &mut ExpressionStatement<'a>) {
        let kind = AstKind::ExpressionStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&mut stmt.expression);
        self.leave_node(kind);
    }

    fn visit_for_statement(&mut self, stmt: &mut ForStatement<'a>) {
        let kind = AstKind::ForStatement(self.alloc(stmt));
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ForStatementInit::is_lexical_declaration);
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
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
        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_statement_init(&mut self, init: &mut ForStatementInit<'a>) {
        let kind = AstKind::ForStatementInit(self.alloc(init));
        self.enter_node(kind);
        match init {
            ForStatementInit::VariableDeclaration(decl) => {
                self.visit_variable_declaration(decl);
            }
            ForStatementInit::Expression(expr) => self.visit_expression(expr),
            ForStatementInit::UsingDeclaration(decl) => {
                self.visit_using_declaration(decl);
            }
        }
        self.leave_node(kind);
    }

    fn visit_for_in_statement(&mut self, stmt: &mut ForInStatement<'a>) {
        let kind = AstKind::ForInStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
        self.visit_for_statement_left(&mut stmt.left);
        self.visit_expression(&mut stmt.right);
        self.visit_statement(&mut stmt.body);
        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
    }

    fn visit_for_of_statement(&mut self, stmt: &mut ForOfStatement<'a>) {
        let kind = AstKind::ForOfStatement(self.alloc(stmt));
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        if is_lexical_declaration {
            self.enter_scope(ScopeFlags::empty());
        }
        self.enter_node(kind);
        self.visit_for_statement_left(&mut stmt.left);
        self.visit_expression(&mut stmt.right);
        self.visit_statement(&mut stmt.body);
        self.leave_node(kind);
        if is_lexical_declaration {
            self.leave_scope();
        }
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
        let kind = AstKind::IfStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&mut stmt.test);
        self.visit_statement(&mut stmt.consequent);
        if let Some(alternate) = &mut stmt.alternate {
            self.visit_statement(alternate);
        }
        self.leave_node(kind);
    }

    fn visit_labeled_statement(&mut self, stmt: &mut LabeledStatement<'a>) {
        let kind = AstKind::LabeledStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_label_identifier(&mut stmt.label);
        self.visit_statement(&mut stmt.body);
        self.leave_node(kind);
    }

    fn visit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>) {
        let kind = AstKind::ReturnStatement(self.alloc(stmt));
        self.enter_node(kind);
        if let Some(arg) = &mut stmt.argument {
            self.visit_expression(arg);
        }
        self.leave_node(kind);
    }

    fn visit_switch_statement(&mut self, stmt: &mut SwitchStatement<'a>) {
        let kind = AstKind::SwitchStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&mut stmt.discriminant);
        self.enter_scope(ScopeFlags::empty());
        for case in stmt.cases.iter_mut() {
            self.visit_switch_case(case);
        }
        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_switch_case(&mut self, case: &mut SwitchCase<'a>) {
        let kind = AstKind::SwitchCase(self.alloc(case));
        self.enter_node(kind);
        if let Some(expr) = &mut case.test {
            self.visit_expression(expr);
        }
        self.visit_statements(&mut case.consequent);
        self.leave_node(kind);
    }

    fn visit_throw_statement(&mut self, stmt: &mut ThrowStatement<'a>) {
        let kind = AstKind::ThrowStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&mut stmt.argument);
        self.leave_node(kind);
    }

    fn visit_try_statement(&mut self, stmt: &mut TryStatement<'a>) {
        let kind = AstKind::TryStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_block_statement(&mut stmt.block);
        if let Some(handler) = &mut stmt.handler {
            self.visit_catch_clause(handler);
        }
        if let Some(finalizer) = &mut stmt.finalizer {
            self.visit_finally_clause(finalizer);
        }
        self.leave_node(kind);
    }

    fn visit_catch_clause(&mut self, clause: &mut CatchClause<'a>) {
        let kind = AstKind::CatchClause(self.alloc(clause));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);
        if let Some(param) = &mut clause.param {
            self.visit_binding_pattern(param);
        }
        self.visit_statements(&mut clause.body.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_finally_clause(&mut self, clause: &mut BlockStatement<'a>) {
        let kind = AstKind::FinallyClause(self.alloc(clause));
        self.enter_node(kind);
        self.visit_block_statement(clause);
        self.leave_node(kind);
    }

    fn visit_while_statement(&mut self, stmt: &mut WhileStatement<'a>) {
        let kind = AstKind::WhileStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&mut stmt.test);
        self.visit_statement(&mut stmt.body);
        self.leave_node(kind);
    }

    fn visit_with_statement(&mut self, stmt: &mut WithStatement<'a>) {
        let kind = AstKind::WithStatement(self.alloc(stmt));
        self.enter_node(kind);
        self.visit_expression(&mut stmt.object);
        self.visit_statement(&mut stmt.body);
        self.leave_node(kind);
    }

    fn visit_directive(&mut self, directive: &mut Directive<'a>) {
        let kind = AstKind::Directive(self.alloc(directive));
        self.enter_node(kind);
        self.visit_string_literal(&mut directive.expression);
        self.leave_node(kind);
    }

    /* ----------  Declaration ---------- */

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        let kind = AstKind::VariableDeclaration(self.alloc(decl));
        self.enter_node(kind);
        for declarator in decl.declarations.iter_mut() {
            self.visit_variable_declarator(declarator);
        }
        self.leave_node(kind);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        let kind = AstKind::VariableDeclarator(self.alloc(declarator));
        self.enter_node(kind);
        self.visit_binding_pattern(&mut declarator.id);
        if let Some(init) = &mut declarator.init {
            self.visit_expression(init);
        }
        self.leave_node(kind);
    }

    fn visit_using_declaration(&mut self, declaration: &mut UsingDeclaration<'a>) {
        let kind = AstKind::UsingDeclaration(self.alloc(declaration));
        self.enter_node(kind);
        for decl in declaration.declarations.iter_mut() {
            self.visit_variable_declarator(decl);
        }
        self.leave_node(kind);
    }

    /* ----------  Function ---------- */

    fn visit_function(&mut self, func: &mut Function<'a>, flags: Option<ScopeFlags>) {
        let kind = AstKind::Function(self.alloc(func));
        self.enter_scope({
            let mut flags = flags.unwrap_or(ScopeFlags::empty()) | ScopeFlags::Function;
            if func.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        self.enter_node(kind);
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
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_function_body(&mut self, body: &mut FunctionBody<'a>) {
        let kind = AstKind::FunctionBody(self.alloc(body));
        self.enter_node(kind);
        for directive in body.directives.iter_mut() {
            self.visit_directive(directive);
        }
        self.visit_statements(&mut body.statements);
        self.leave_node(kind);
    }

    fn visit_formal_parameters(&mut self, params: &mut FormalParameters<'a>) {
        let kind = AstKind::FormalParameters(self.alloc(params));
        self.enter_node(kind);
        for param in params.items.iter_mut() {
            self.visit_formal_parameter(param);
        }
        if let Some(rest) = &mut params.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        let kind = AstKind::FormalParameter(self.alloc(param));
        self.enter_node(kind);
        for decorator in param.decorators.iter_mut() {
            self.visit_decorator(decorator);
        }
        self.visit_binding_pattern(&mut param.pattern);
        self.leave_node(kind);
    }

    /* ----------  Class ---------- */

    fn visit_decorator(&mut self, decorator: &mut Decorator<'a>) {
        let kind = AstKind::Decorator(self.alloc(decorator));
        self.enter_node(kind);
        self.visit_expression(&mut decorator.expression);
        self.leave_node(kind);
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        for decorator in class.decorators.iter_mut() {
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
        self.leave_node(kind);
        if is_class_expr {
            self.leave_scope();
        }
    }

    fn visit_class_heritage(&mut self, expr: &mut Expression<'a>) {
        let kind = AstKind::ClassHeritage(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(expr);
        self.leave_node(kind);
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
            ClassElement::TSIndexSignature(_def) => {}
        }
    }

    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        let kind = AstKind::StaticBlock(self.alloc(block));
        self.enter_scope(ScopeFlags::ClassStaticBlock);
        self.enter_node(kind);
        self.visit_statements(&mut block.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        let kind = AstKind::MethodDefinition(self.alloc(def));
        self.enter_node(kind);
        for decorator in def.decorators.iter_mut() {
            self.visit_decorator(decorator);
        }

        let flags = match def.kind {
            MethodDefinitionKind::Get => ScopeFlags::GetAccessor,
            MethodDefinitionKind::Set => ScopeFlags::SetAccessor,
            MethodDefinitionKind::Constructor => ScopeFlags::Constructor,
            MethodDefinitionKind::Method => ScopeFlags::empty(),
        };
        self.visit_property_key(&mut def.key);
        self.visit_function(&mut def.value, Some(flags));
        self.leave_node(kind);
    }

    fn visit_property_definition(&mut self, def: &mut PropertyDefinition<'a>) {
        let kind = AstKind::PropertyDefinition(self.alloc(def));
        self.enter_node(kind);
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
        self.leave_node(kind);
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
            Expression::ClassExpression(expr) => self.visit_class(expr),
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

    fn visit_meta_property(&mut self, meta: &mut MetaProperty<'a>) {
        let kind = AstKind::MetaProperty(self.alloc(meta));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_array_expression(&mut self, expr: &mut ArrayExpression<'a>) {
        let kind = AstKind::ArrayExpression(self.alloc(expr));
        self.enter_node(kind);
        for elem in expr.elements.iter_mut() {
            self.visit_array_expression_element(elem);
        }
        self.leave_node(kind);
    }

    fn visit_array_expression_element(&mut self, arg: &mut ArrayExpressionElement<'a>) {
        let kind = AstKind::ArrayExpressionElement(self.alloc(arg));
        self.enter_node(kind);
        match arg {
            ArrayExpressionElement::SpreadElement(spread) => self.visit_spread_element(spread),
            ArrayExpressionElement::Expression(expr) => self.visit_expression_array_element(expr),
            ArrayExpressionElement::Elision(span) => self.visit_elision(*span),
        }
        self.leave_node(kind);
    }

    fn visit_argument(&mut self, arg: &mut Argument<'a>) {
        let kind = AstKind::Argument(self.alloc(arg));
        self.enter_node(kind);
        match arg {
            Argument::SpreadElement(spread) => self.visit_spread_element(spread),
            Argument::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_spread_element(&mut self, elem: &mut SpreadElement<'a>) {
        let kind = AstKind::SpreadElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_expression(&mut elem.argument);
        self.leave_node(kind);
    }

    fn visit_expression_array_element(&mut self, expr: &mut Expression<'a>) {
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

    fn visit_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        let kind = AstKind::AssignmentExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_assignment_target(&mut expr.left);
        self.visit_expression(&mut expr.right);
        self.leave_node(kind);
    }

    fn visit_arrow_expression(&mut self, expr: &mut ArrowFunctionExpression<'a>) {
        let kind = AstKind::ArrowFunctionExpression(self.alloc(expr));
        self.enter_scope(ScopeFlags::Function | ScopeFlags::Arrow);
        self.enter_node(kind);
        self.visit_formal_parameters(&mut expr.params);
        self.visit_function_body(&mut expr.body);
        if let Some(parameters) = &mut expr.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_await_expression(&mut self, expr: &mut AwaitExpression<'a>) {
        let kind = AstKind::AwaitExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.argument);
        self.leave_node(kind);
    }

    fn visit_binary_expression(&mut self, expr: &mut BinaryExpression<'a>) {
        let kind = AstKind::BinaryExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.left);
        self.visit_expression(&mut expr.right);
        self.leave_node(kind);
    }

    fn visit_call_expression(&mut self, expr: &mut CallExpression<'a>) {
        let kind = AstKind::CallExpression(self.alloc(expr));
        self.enter_node(kind);
        for arg in expr.arguments.iter_mut() {
            self.visit_argument(arg);
        }
        self.visit_expression(&mut expr.callee);
        if let Some(parameters) = &mut expr.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        self.leave_node(kind);
    }

    fn visit_chain_expression(&mut self, expr: &mut ChainExpression<'a>) {
        let kind = AstKind::ChainExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_chain_element(&mut expr.expression);
        self.leave_node(kind);
    }

    fn visit_chain_element(&mut self, elem: &mut ChainElement<'a>) {
        match elem {
            ChainElement::CallExpression(expr) => self.visit_call_expression(expr),
            ChainElement::MemberExpression(expr) => self.visit_member_expression(expr),
        }
    }

    fn visit_conditional_expression(&mut self, expr: &mut ConditionalExpression<'a>) {
        let kind = AstKind::ConditionalExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.test);
        self.visit_expression(&mut expr.consequent);
        self.visit_expression(&mut expr.alternate);
        self.leave_node(kind);
    }

    fn visit_import_expression(&mut self, expr: &mut ImportExpression<'a>) {
        self.visit_expression(&mut expr.source);
        for arg in expr.arguments.iter_mut() {
            self.visit_expression(arg);
        }
    }

    fn visit_logical_expression(&mut self, expr: &mut LogicalExpression<'a>) {
        let kind = AstKind::LogicalExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.left);
        self.visit_expression(&mut expr.right);
        self.leave_node(kind);
    }

    fn visit_member_expression(&mut self, expr: &mut MemberExpression<'a>) {
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
        let kind = AstKind::NewExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.callee);
        if let Some(parameters) = &mut expr.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        for arg in expr.arguments.iter_mut() {
            self.visit_argument(arg);
        }
        self.leave_node(kind);
    }

    fn visit_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(self.alloc(expr));
        self.enter_node(kind);
        for prop in expr.properties.iter_mut() {
            self.visit_object_property_kind(prop);
        }
        self.leave_node(kind);
    }

    fn visit_object_property_kind(&mut self, prop: &mut ObjectPropertyKind<'a>) {
        match prop {
            ObjectPropertyKind::ObjectProperty(prop) => self.visit_object_property(prop),
            ObjectPropertyKind::SpreadProperty(elem) => self.visit_spread_element(elem),
        }
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        let kind = AstKind::ObjectProperty(self.alloc(prop));
        self.enter_node(kind);
        self.visit_property_key(&mut prop.key);
        self.visit_expression(&mut prop.value);
        if let Some(init) = &mut prop.init {
            self.visit_expression(init);
        }
        self.leave_node(kind);
    }

    fn visit_property_key(&mut self, key: &mut PropertyKey<'a>) {
        let kind = AstKind::PropertyKey(self.alloc(key));
        self.enter_node(kind);
        match key {
            PropertyKey::Identifier(ident) => self.visit_identifier_name(ident),
            PropertyKey::PrivateIdentifier(ident) => self.visit_private_identifier(ident),
            PropertyKey::Expression(expr) => self.visit_expression(expr),
        }
        self.leave_node(kind);
    }

    fn visit_parenthesized_expression(&mut self, expr: &mut ParenthesizedExpression<'a>) {
        let kind = AstKind::ParenthesizedExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.expression);
        self.leave_node(kind);
    }

    fn visit_private_in_expression(&mut self, expr: &mut PrivateInExpression<'a>) {
        self.visit_private_identifier(&mut expr.left);
        self.visit_expression(&mut expr.right);
    }

    fn visit_sequence_expression(&mut self, expr: &mut SequenceExpression<'a>) {
        let kind = AstKind::SequenceExpression(self.alloc(expr));
        self.enter_node(kind);
        for expr in expr.expressions.iter_mut() {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_tagged_template_expression(&mut self, expr: &mut TaggedTemplateExpression<'a>) {
        let kind = AstKind::TaggedTemplateExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.tag);
        self.visit_template_literal(&mut expr.quasi);
        self.leave_node(kind);
    }

    fn visit_this_expression(&mut self, expr: &mut ThisExpression) {
        let kind = AstKind::ThisExpression(self.alloc(expr));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>) {
        let kind = AstKind::UnaryExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.argument);
        self.leave_node(kind);
    }

    fn visit_update_expression(&mut self, expr: &mut UpdateExpression<'a>) {
        let kind = AstKind::UpdateExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_simple_assignment_target(&mut expr.argument);
        self.leave_node(kind);
    }

    fn visit_yield_expression(&mut self, expr: &mut YieldExpression<'a>) {
        let kind = AstKind::YieldExpression(self.alloc(expr));
        self.enter_node(kind);
        if let Some(argument) = &mut expr.argument {
            self.visit_expression(argument);
        }
        self.leave_node(kind);
    }

    fn visit_super(&mut self, expr: &mut Super) {
        let kind = AstKind::Super(self.alloc(expr));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_assignment_target(&mut self, target: &mut AssignmentTarget<'a>) {
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

    fn visit_simple_assignment_target(&mut self, target: &mut SimpleAssignmentTarget<'a>) {
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
        self.leave_node(kind);
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
        let kind = AstKind::AssignmentTargetWithDefault(self.alloc(target));
        self.enter_node(kind);
        self.visit_assignment_target(&mut target.binding);
        self.visit_expression(&mut target.init);
        self.leave_node(kind);
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
        let kind = AstKind::JSXElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_jsx_opening_element(&mut elem.opening_element);
        for child in elem.children.iter_mut() {
            self.visit_jsx_child(child);
        }
        if let Some(closing_elem) = &mut elem.closing_element {
            self.visit_jsx_closing_element(closing_elem);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        let kind = AstKind::JSXOpeningElement(self.alloc(elem));
        self.enter_node(kind);

        self.visit_jsx_element_name(&mut elem.name);
        for attribute in elem.attributes.iter_mut() {
            self.visit_jsx_attribute_item(attribute);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_closing_element(&mut self, elem: &mut JSXClosingElement<'a>) {
        let kind = AstKind::JSXClosingElement(self.alloc(elem));
        self.enter_node(kind);
        self.visit_jsx_element_name(&mut elem.name);
        self.leave_node(kind);
    }

    fn visit_jsx_element_name(&mut self, name: &mut JSXElementName<'a>) {
        let kind = AstKind::JSXElementName(self.alloc(name));
        self.enter_node(kind);
        match name {
            JSXElementName::Identifier(ident) => self.visit_jsx_identifier(ident),
            JSXElementName::MemberExpression(expr) => self.visit_jsx_member_expression(expr),
            JSXElementName::NamespacedName(name) => self.visit_jsx_namespaced_name(name),
        }
        self.leave_node(kind);
    }

    fn visit_jsx_identifier(&mut self, ident: &mut JSXIdentifier<'a>) {
        let kind = AstKind::JSXIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_jsx_member_expression(&mut self, expr: &mut JSXMemberExpression<'a>) {
        let kind = AstKind::JSXMemberExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_jsx_member_expression_object(&mut expr.object);
        self.visit_jsx_identifier(&mut expr.property);
        self.leave_node(kind);
    }

    fn visit_jsx_member_expression_object(&mut self, expr: &mut JSXMemberExpressionObject<'a>) {
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

    fn visit_jsx_namespaced_name(&mut self, name: &mut JSXNamespacedName<'a>) {
        let kind = AstKind::JSXNamespacedName(self.alloc(name));
        self.enter_node(kind);
        self.visit_jsx_identifier(&mut name.namespace);
        self.visit_jsx_identifier(&mut name.property);
        self.leave_node(kind);
    }

    fn visit_jsx_attribute_item(&mut self, item: &mut JSXAttributeItem<'a>) {
        let kind = AstKind::JSXAttributeItem(self.alloc(item));
        self.enter_node(kind);
        match item {
            JSXAttributeItem::Attribute(attribute) => self.visit_jsx_attribute(attribute),
            JSXAttributeItem::SpreadAttribute(attribute) => {
                self.visit_jsx_spread_attribute(attribute);
            }
        }
        self.leave_node(kind);
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
        let kind = AstKind::JSXExpressionContainer(self.alloc(expr));
        self.enter_node(kind);
        self.visit_jsx_expression(&mut expr.expression);
        self.leave_node(kind);
    }

    fn visit_jsx_expression(&mut self, expr: &mut JSXExpression<'a>) {
        match expr {
            JSXExpression::Expression(expr) => self.visit_expression(expr),
            JSXExpression::EmptyExpression(_) => {}
        }
    }

    fn visit_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>) {
        let kind = AstKind::JSXFragment(self.alloc(elem));
        self.enter_node(kind);
        for child in elem.children.iter_mut() {
            self.visit_jsx_child(child);
        }
        self.leave_node(kind);
    }

    fn visit_jsx_child(&mut self, child: &mut JSXChild<'a>) {
        match child {
            JSXChild::Element(elem) => self.visit_jsx_element(elem),
            JSXChild::Fragment(elem) => self.visit_jsx_fragment(elem),
            JSXChild::ExpressionContainer(expr) => self.visit_jsx_expression_container(expr),
            JSXChild::Spread(expr) => self.visit_jsx_spread_child(expr),
            JSXChild::Text(expr) => self.visit_jsx_text(expr),
        }
    }

    fn visit_jsx_spread_child(&mut self, child: &mut JSXSpreadChild<'a>) {
        self.visit_expression(&mut child.expression);
    }

    fn visit_jsx_text(&mut self, child: &JSXText<'a>) {
        let kind = AstKind::JSXText(self.alloc(child));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    /*<'a> ----------  Pattern ---------- */

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

    fn visit_binding_identifier(&mut self, ident: &mut BindingIdentifier<'a>) {
        let kind = AstKind::BindingIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_object_pattern(&mut self, pat: &mut ObjectPattern<'a>) {
        let kind = AstKind::ObjectPattern(self.alloc(pat));
        self.enter_node(kind);
        for prop in pat.properties.iter_mut() {
            self.visit_binding_property(prop);
        }
        if let Some(rest) = &mut pat.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_binding_property(&mut self, prop: &mut BindingProperty<'a>) {
        self.visit_property_key(&mut prop.key);
        self.visit_binding_pattern(&mut prop.value);
    }

    fn visit_array_pattern(&mut self, pat: &mut ArrayPattern<'a>) {
        let kind = AstKind::ArrayPattern(self.alloc(pat));
        self.enter_node(kind);
        for pat in pat.elements.iter_mut().flatten() {
            self.visit_binding_pattern(pat);
        }
        if let Some(rest) = &mut pat.rest {
            self.visit_rest_element(rest);
        }
        self.leave_node(kind);
    }

    fn visit_rest_element(&mut self, pat: &mut BindingRestElement<'a>) {
        let kind = AstKind::BindingRestElement(self.alloc(pat));
        self.enter_node(kind);
        self.visit_binding_pattern(&mut pat.argument);
        self.leave_node(kind);
    }

    fn visit_assignment_pattern(&mut self, pat: &mut AssignmentPattern<'a>) {
        let kind = AstKind::AssignmentPattern(self.alloc(pat));
        self.enter_node(kind);
        self.visit_binding_pattern(&mut pat.left);
        self.visit_expression(&mut pat.right);
        self.leave_node(kind);
    }

    /* ----------  Identifier ---------- */

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        let kind = AstKind::IdentifierReference(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_private_identifier(&mut self, ident: &mut PrivateIdentifier<'a>) {
        let kind = AstKind::PrivateIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_label_identifier(&mut self, ident: &mut LabelIdentifier<'a>) {
        let kind = AstKind::LabelIdentifier(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_identifier_name(&mut self, ident: &mut IdentifierName<'a>) {
        let kind = AstKind::IdentifierName(self.alloc(ident));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    /* ----------  Literal ---------- */

    fn visit_number_literal(&mut self, lit: &mut NumericLiteral<'a>) {
        let kind = AstKind::NumericLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_boolean_literal(&mut self, lit: &mut BooleanLiteral) {
        let kind = AstKind::BooleanLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_null_literal(&mut self, lit: &mut NullLiteral) {
        let kind = AstKind::NullLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_bigint_literal(&mut self, lit: &mut BigintLiteral<'a>) {
        let kind = AstKind::BigintLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_string_literal(&mut self, lit: &mut StringLiteral<'a>) {
        let kind = AstKind::StringLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_literal(&mut self, lit: &mut TemplateLiteral<'a>) {
        let kind = AstKind::TemplateLiteral(self.alloc(lit));
        self.enter_node(kind);
        for elem in lit.quasis.iter_mut() {
            self.visit_template_element(elem);
        }
        for expr in lit.expressions.iter_mut() {
            self.visit_expression(expr);
        }
        self.leave_node(kind);
    }

    fn visit_reg_expr_literal(&mut self, lit: &mut RegExpLiteral<'a>) {
        let kind = AstKind::RegExpLiteral(self.alloc(lit));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_template_element(&mut self, _elem: &mut TemplateElement) {}

    /* ----------  Module ---------- */

    fn visit_module_declaration(&mut self, decl: &mut ModuleDeclaration<'a>) {
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
                self.visit_expression(&mut decl.expression);
            }
            ModuleDeclaration::TSNamespaceExportDeclaration(_) => {}
        }
        self.leave_node(kind);
    }

    fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        let kind = AstKind::ImportDeclaration(self.alloc(decl));
        self.enter_node(kind);
        if let Some(specifiers) = &mut decl.specifiers {
            for specifier in specifiers.iter_mut() {
                self.visit_import_declaration_specifier(specifier);
            }
        }
        self.visit_string_literal(&mut decl.source);
        // TODO: assertions
        self.leave_node(kind);
    }

    fn visit_import_declaration_specifier(
        &mut self,
        specifier: &mut ImportDeclarationSpecifier<'a>,
    ) {
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

    fn visit_import_specifier(&mut self, specifier: &mut ImportSpecifier<'a>) {
        let kind = AstKind::ImportSpecifier(self.alloc(specifier));
        self.enter_node(kind);
        // TODO: imported
        self.visit_binding_identifier(&mut specifier.local);
        self.leave_node(kind);
    }

    fn visit_import_default_specifier(&mut self, specifier: &mut ImportDefaultSpecifier<'a>) {
        let kind = AstKind::ImportDefaultSpecifier(self.alloc(specifier));
        self.enter_node(kind);
        self.visit_binding_identifier(&mut specifier.local);
        self.leave_node(kind);
    }

    fn visit_import_name_specifier(&mut self, specifier: &mut ImportNamespaceSpecifier<'a>) {
        let kind = AstKind::ImportNamespaceSpecifier(self.alloc(specifier));
        self.enter_node(kind);
        self.visit_binding_identifier(&mut specifier.local);
        self.leave_node(kind);
    }

    fn visit_export_all_declaration(&mut self, decl: &mut ExportAllDeclaration<'a>) {
        let kind = AstKind::ExportAllDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_string_literal(&mut decl.source);
        self.leave_node(kind);
    }

    fn visit_export_default_declaration(&mut self, decl: &mut ExportDefaultDeclaration<'a>) {
        let kind = AstKind::ExportDefaultDeclaration(self.alloc(decl));
        self.enter_node(kind);
        match &mut decl.declaration {
            ExportDefaultDeclarationKind::Expression(expr) => self.visit_expression(expr),
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                self.visit_function(func, None);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => self.visit_class(class),
            _ => {}
        }
        self.leave_node(kind);
    }

    fn visit_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        let kind = AstKind::ExportNamedDeclaration(self.alloc(decl));
        self.enter_node(kind);
        if let Some(decl) = &mut decl.declaration {
            self.visit_declaration(decl);
        }
        if let Some(source) = &mut decl.source {
            self.visit_string_literal(source);
        }
        self.leave_node(kind);
    }

    fn visit_enum_member(&mut self, member: &mut TSEnumMember<'a>) {
        let kind = AstKind::TSEnumMember(self.alloc(member));
        self.enter_node(kind);
        if let Some(initializer) = &mut member.initializer {
            self.visit_expression(initializer);
        }
        self.leave_node(kind);
    }

    fn visit_enum(&mut self, decl: &mut TSEnumDeclaration<'a>) {
        let kind = AstKind::TSEnumDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&mut decl.id);
        self.enter_scope(ScopeFlags::empty());
        for member in decl.members.iter_mut() {
            self.visit_enum_member(member);
        }
        self.leave_scope();
        self.leave_node(kind);
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
        self.visit_declaration_match(decl);
    }

    fn visit_declaration_match(&mut self, decl: &mut Declaration<'a>) {
        match decl {
            Declaration::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            Declaration::FunctionDeclaration(func) => self.visit_function(func, None),
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
        let kind = AstKind::TSImportEqualsDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&mut decl.id);
        self.visit_ts_module_reference(&mut decl.module_reference);
        self.leave_node(kind);
    }

    fn visit_ts_module_reference(&mut self, reference: &mut TSModuleReference<'a>) {
        match reference {
            TSModuleReference::TypeName(name) => self.visit_ts_type_name(name),
            TSModuleReference::ExternalModuleReference(reference) => {
                self.visit_ts_external_module_reference(reference);
            }
        }
    }

    fn visit_ts_type_name(&mut self, name: &mut TSTypeName<'a>) {
        let kind = AstKind::TSTypeName(self.alloc(name));
        self.enter_node(kind);
        match name {
            TSTypeName::IdentifierReference(ident) => self.visit_identifier_reference(ident),
            TSTypeName::QualifiedName(name) => self.visit_ts_qualified_name(name),
        }
        self.leave_node(kind);
    }

    fn visit_ts_external_module_reference(
        &mut self,
        reference: &mut TSExternalModuleReference<'a>,
    ) {
        let kind = AstKind::TSExternalModuleReference(self.alloc(reference));
        self.enter_node(kind);
        self.visit_string_literal(&mut reference.expression);
        self.leave_node(kind);
    }

    fn visit_ts_qualified_name(&mut self, name: &mut TSQualifiedName<'a>) {
        let kind = AstKind::TSQualifiedName(self.alloc(name));
        self.enter_node(kind);
        self.visit_ts_type_name(&mut name.left);
        self.visit_identifier_name(&mut name.right);
        self.leave_node(kind);
    }

    fn visit_ts_module_declaration(&mut self, decl: &mut TSModuleDeclaration<'a>) {
        let kind = AstKind::TSModuleDeclaration(self.alloc(decl));
        self.enter_node(kind);
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
        self.leave_node(kind);
    }

    fn visit_ts_module_block(&mut self, block: &mut TSModuleBlock<'a>) {
        let kind = AstKind::TSModuleBlock(self.alloc(block));
        self.enter_scope(ScopeFlags::TsModuleBlock);
        self.enter_node(kind);
        self.visit_statements(&mut block.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &mut TSTypeAliasDeclaration<'a>) {
        let kind = AstKind::TSTypeAliasDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&mut decl.id);
        if let Some(parameters) = &mut decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        self.visit_ts_type(&mut decl.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &mut TSInterfaceDeclaration<'a>) {
        let kind = AstKind::TSInterfaceDeclaration(self.alloc(decl));
        self.enter_node(kind);
        self.visit_binding_identifier(&mut decl.id);
        if let Some(parameters) = &mut decl.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        for signature in decl.body.body.iter_mut() {
            self.visit_ts_signature(signature);
        }
        self.leave_node(kind);
    }

    fn visit_ts_as_expression(&mut self, expr: &mut TSAsExpression<'a>) {
        let kind = AstKind::TSAsExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type(&mut expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_satisfies_expression(&mut self, expr: &mut TSSatisfiesExpression<'a>) {
        let kind = AstKind::TSSatisfiesExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type(&mut expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_non_null_expression(&mut self, expr: &mut TSNonNullExpression<'a>) {
        let kind = AstKind::TSNonNullExpression(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.expression);
        self.leave_node(kind);
    }

    fn visit_ts_type_assertion(&mut self, expr: &mut TSTypeAssertion<'a>) {
        let kind = AstKind::TSTypeAssertion(self.alloc(expr));
        self.enter_node(kind);
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type(&mut expr.type_annotation);
        self.leave_node(kind);
    }

    fn visit_ts_instantiation_expression(&mut self, expr: &mut TSInstantiationExpression<'a>) {
        self.visit_expression(&mut expr.expression);
        self.visit_ts_type_parameter_instantiation(&mut expr.type_parameters);
    }

    fn visit_ts_type_annotation(&mut self, annotation: &mut TSTypeAnnotation<'a>) {
        let kind = AstKind::TSTypeAnnotation(self.alloc(annotation));
        self.enter_node(kind);
        self.visit_ts_type(&mut annotation.type_annotation);
        self.leave_node(kind);
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
            TSType::TSTypeQuery(ty) => self.visit_ts_type_query(ty),
            _ => {}
        }
    }

    fn visit_ts_type_literal(&mut self, ty: &mut TSTypeLiteral<'a>) {
        let kind = AstKind::TSTypeLiteral(self.alloc(ty));
        self.enter_node(kind);
        for signature in ty.members.iter_mut() {
            self.visit_ts_signature(signature);
        }
        self.leave_node(kind);
    }

    fn visit_ts_indexed_access_type(&mut self, ty: &mut TSIndexedAccessType<'a>) {
        let kind = AstKind::TSIndexedAccessType(self.alloc(ty));
        self.enter_node(kind);
        self.visit_ts_type(&mut ty.object_type);
        self.visit_ts_type(&mut ty.index_type);
        self.leave_node(kind);
    }

    fn visit_ts_type_predicate(&mut self, ty: &mut TSTypePredicate<'a>) {
        if let Some(annotation) = &mut ty.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
    }

    fn visit_ts_type_operator_type(&mut self, ty: &mut TSTypeOperator<'a>) {
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
            self.visit_ts_type(type_annotation);
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
        let kind = AstKind::TSTypeParameter(self.alloc(ty));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);
        if let Some(constraint) = &mut ty.constraint {
            self.visit_ts_type(constraint);
        }

        if let Some(default) = &mut ty.default {
            self.visit_ts_type(default);
        }
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_ts_type_parameter_instantiation(&mut self, ty: &mut TSTypeParameterInstantiation<'a>) {
        let kind = AstKind::TSTypeParameterInstantiation(self.alloc(ty));
        self.enter_node(kind);
        for ts_parameter in ty.params.iter_mut() {
            self.visit_ts_type(ts_parameter);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_parameter_declaration(&mut self, ty: &mut TSTypeParameterDeclaration<'a>) {
        let kind = AstKind::TSTypeParameterDeclaration(self.alloc(ty));
        self.enter_node(kind);
        for ts_parameter in ty.params.iter_mut() {
            self.visit_ts_type_parameter(ts_parameter);
        }
        self.leave_node(kind);
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

    fn visit_ts_null_keyword(&mut self, ty: &mut TSNullKeyword) {
        let kind = AstKind::TSNullKeyword(self.alloc(ty));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_any_keyword(&mut self, ty: &mut TSAnyKeyword) {
        let kind = AstKind::TSAnyKeyword(self.alloc(ty));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_void_keyword(&mut self, ty: &mut TSVoidKeyword) {
        let kind = AstKind::TSVoidKeyword(self.alloc(ty));
        self.enter_node(kind);
        self.leave_node(kind);
    }

    fn visit_ts_intersection_type(&mut self, ty: &mut TSIntersectionType<'a>) {
        let kind = AstKind::TSIntersectionType(self.alloc(ty));
        self.enter_node(kind);
        for ty in ty.types.iter_mut() {
            self.visit_ts_type(ty);
        }
        self.leave_node(kind);
    }

    fn visit_ts_type_reference(&mut self, ty: &mut TSTypeReference<'a>) {
        let kind = AstKind::TSTypeReference(self.alloc(ty));
        self.enter_node(kind);
        self.visit_ts_type_name(&mut ty.type_name);
        if let Some(parameters) = &mut ty.type_parameters {
            self.visit_ts_type_parameter_instantiation(parameters);
        }
        self.leave_node(kind);
    }

    fn visit_ts_union_type(&mut self, ty: &mut TSUnionType<'a>) {
        let kind = AstKind::TSUnionType(self.alloc(ty));
        self.enter_node(kind);
        for ty in ty.types.iter_mut() {
            self.visit_ts_type(ty);
        }
        self.leave_node(kind);
    }

    fn visit_ts_literal_type(&mut self, ty: &mut TSLiteralType<'a>) {
        let kind = AstKind::TSLiteralType(self.alloc(ty));
        self.enter_node(kind);
        match &mut ty.literal {
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
        let kind = AstKind::TSMethodSignature(self.alloc(signature));
        self.enter_node(kind);
        self.visit_formal_parameters(&mut signature.params);
        if let Some(parameters) = &mut signature.type_parameters {
            self.visit_ts_type_parameter_declaration(parameters);
        }
        if let Some(annotation) = &mut signature.return_type {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
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
        let kind = AstKind::TSPropertySignature(self.alloc(signature));
        self.enter_node(kind);
        self.visit_property_key(&mut signature.key);
        if let Some(annotation) = &mut signature.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
        self.leave_node(kind);
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

    fn visit_ts_type_query(&mut self, ty: &mut TSTypeQuery<'a>) {
        let kind = AstKind::TSTypeQuery(self.alloc(ty));
        self.enter_node(kind);
        self.visit_ts_type_name(&mut ty.expr_name);
        if let Some(type_parameters) = &mut ty.type_parameters {
            self.visit_ts_type_parameter_instantiation(type_parameters);
        }
        self.leave_node(kind);
    }
}
