#![allow(clippy::unused_self)]

use oxc_allocator::{Allocator, Box, Vec};
use oxc_ast::ast;
use oxc_hir::{hir, HirBuilder};
use oxc_semantic::{
    Reference, ReferenceFlag, ReferenceId, ScopeFlags, Semantic, SemanticBuilder, SymbolFlags,
    SymbolId,
};
use oxc_span::{Atom, GetSpan, SourceType, Span};

// <https://github.com/rust-lang/rust/blob/master/compiler/rustc_data_structures/src/stack.rs>
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn ensure_sufficient_stack<R, F: FnOnce() -> R>(f: F) -> R {
    const RED_ZONE: usize = 100 * 1024; // 100k
    const STACK_PER_RECURSION: usize = 1024 * 1024; // 1MB
    stacker::maybe_grow(RED_ZONE, STACK_PER_RECURSION, f)
}
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn ensure_sufficient_stack<R, F: FnOnce() -> R>(f: F) -> R {
    f()
}

pub struct AstLowerReturn<'a> {
    pub program: hir::Program<'a>,
    pub semantic: Semantic<'a>,
}

pub struct AstLower<'a> {
    hir: HirBuilder<'a>,
    semantic: SemanticBuilder<'a>,
}

impl<'a> AstLower<'a> {
    pub fn enter_binding_identifier(
        &mut self,
        span: Span,
        name: &Atom,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        self.semantic.declare_symbol(span, name, includes, excludes)
    }

    pub fn enter_identifier_reference(
        &mut self,
        span: Span,
        name: &Atom,
        reference_flag: ReferenceFlag,
    ) -> ReferenceId {
        let reference =
            Reference::new(span, name.clone(), self.semantic.current_node_id, reference_flag);
        self.semantic.declare_reference(reference)
    }

    pub fn enter_block_statement(&mut self) {
        self.semantic.enter_scope(ScopeFlags::empty());
    }

    pub fn leave_block_statement(&mut self) {
        self.semantic.leave_scope();
    }

    pub fn enter_function_scope(&mut self) {
        self.semantic.enter_scope(ScopeFlags::Function);
    }

    pub fn leave_function_scope(&mut self) {
        self.semantic.leave_scope();
    }

    pub fn enter_static_block(&mut self) {
        self.semantic.enter_scope(ScopeFlags::ClassStaticBlock);
    }

    pub fn leave_static_block(&mut self) {
        self.semantic.leave_scope();
    }

    pub fn enter_catch_clause(&mut self) {
        self.semantic.enter_scope(ScopeFlags::empty());
    }

    pub fn leave_catch_clause(&mut self) {
        self.semantic.leave_scope();
    }

    // ForStatement : for ( LexicalDeclaration Expressionopt ; Expressionopt ) Statement
    //   1. Let oldEnv be the running execution context's LexicalEnvironment.
    //   2. Let loopEnv be NewDeclarativeEnvironment(oldEnv).
    pub fn enter_for_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.semantic.enter_scope(ScopeFlags::empty());
        }
    }

    pub fn leave_for_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.semantic.leave_scope();
        }
    }

    pub fn enter_for_in_of_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.semantic.enter_scope(ScopeFlags::empty());
        }
    }

    pub fn leave_for_in_of_statement(&mut self, is_lexical_declaration: bool) {
        if is_lexical_declaration {
            self.semantic.leave_scope();
        }
    }
}

impl<'a> AstLower<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, source_type: SourceType) -> Self {
        Self {
            hir: HirBuilder::new(allocator),
            semantic: SemanticBuilder::new(source_text, source_type),
        }
    }

    pub fn build(mut self, program: &ast::Program<'a>) -> AstLowerReturn<'a> {
        let program = self.lower_program(program);
        let semantic = self.semantic.build2();
        AstLowerReturn { program, semantic }
    }

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

    pub fn lower_statements(
        &mut self,
        stmts: &Vec<'a, ast::Statement<'a>>,
    ) -> Vec<'a, hir::Statement<'a>> {
        let mut vec = self.hir.new_vec_with_capacity(stmts.len());
        for stmt in stmts {
            if let Some(stmt) = self.lower_statement(stmt) {
                vec.push(stmt);
            }
        }
        vec
    }

    fn lower_program(&mut self, program: &ast::Program<'a>) -> hir::Program<'a> {
        let directives = self.lower_vec(&program.directives, Self::lower_directive);
        let hashbang = program.hashbang.as_ref().map(|hashbang| self.lower_hasbang(hashbang));
        let statements = self.lower_statements(&program.body);
        self.hir.program(program.span, directives, hashbang, statements)
    }

    fn lower_hasbang(&mut self, hashbang: &ast::Hashbang<'a>) -> hir::Hashbang<'a> {
        self.hir.hashbang(hashbang.span, hashbang.value)
    }

    fn lower_directive(&mut self, directive: &ast::Directive<'a>) -> hir::Directive<'a> {
        let expression = self.lower_string_literal(&directive.expression);
        self.hir.directive(directive.span, expression, directive.directive)
    }

    fn lower_statement(&mut self, statement: &ast::Statement<'a>) -> Option<hir::Statement<'a>> {
        match statement {
            ast::Statement::BlockStatement(stmt) => {
                let block = self.lower_block_statement(stmt);
                Some(hir::Statement::BlockStatement(block))
            }
            ast::Statement::BreakStatement(stmt) => Some(self.lower_break_statement(stmt)),
            ast::Statement::ContinueStatement(stmt) => Some(self.lower_continue_statement(stmt)),
            ast::Statement::DebuggerStatement(stmt) => Some(self.lower_debugger_statement(stmt)),
            ast::Statement::DoWhileStatement(stmt) => Some(self.lower_do_while_statement(stmt)),
            ast::Statement::EmptyStatement(_) => None,
            ast::Statement::ExpressionStatement(stmt) => {
                Some(self.lower_expression_statement(stmt))
            }
            ast::Statement::ForInStatement(stmt) => Some(self.lower_for_in_statement(stmt)),
            ast::Statement::ForOfStatement(stmt) => Some(self.lower_for_of_statement(stmt)),
            ast::Statement::ForStatement(stmt) => Some(self.lower_for_statement(stmt)),
            ast::Statement::IfStatement(stmt) => Some(self.lower_if_statement(stmt)),
            ast::Statement::LabeledStatement(stmt) => Some(self.lower_labeled_statement(stmt)),
            ast::Statement::ReturnStatement(stmt) => Some(self.lower_return_statement(stmt)),
            ast::Statement::SwitchStatement(stmt) => Some(self.lower_switch_statement(stmt)),
            ast::Statement::ThrowStatement(stmt) => Some(self.lower_throw_statement(stmt)),
            ast::Statement::TryStatement(stmt) => Some(self.lower_try_statement(stmt)),
            ast::Statement::WhileStatement(stmt) => Some(self.lower_while_statement(stmt)),
            ast::Statement::WithStatement(stmt) => Some(self.lower_with_statement(stmt)),
            ast::Statement::ModuleDeclaration(decl) => self.lower_module_declaration(decl),
            ast::Statement::Declaration(decl) => {
                self.lower_declaration(decl).map(hir::Statement::Declaration)
            }
        }
    }

    fn lower_block_statement(
        &mut self,
        stmt: &ast::BlockStatement<'a>,
    ) -> Box<'a, hir::BlockStatement<'a>> {
        self.enter_block_statement();
        let body = self.lower_statements(&stmt.body);
        self.leave_block_statement();
        self.hir.block(stmt.span, body)
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

    fn lower_expression_statement(
        &mut self,
        stmt: &ast::ExpressionStatement<'a>,
    ) -> hir::Statement<'a> {
        let expression = self.lower_expression(&stmt.expression);
        self.hir.expression_statement(stmt.span, expression)
    }

    fn lower_for_statement(&mut self, stmt: &ast::ForStatement<'a>) -> hir::Statement<'a> {
        let is_lexical_declaration =
            stmt.init.as_ref().is_some_and(ast::ForStatementInit::is_lexical_declaration);
        self.enter_for_statement(is_lexical_declaration);
        let init = stmt.init.as_ref().map(|init| self.lower_for_statement_init(init));
        let test = stmt.test.as_ref().map(|expr| self.lower_expression(expr));
        let update = stmt.update.as_ref().map(|expr| self.lower_expression(expr));
        let body = self.lower_statement(&stmt.body);
        self.leave_for_statement(is_lexical_declaration);
        self.hir.for_statement(stmt.span, init, test, update, body)
    }

    fn lower_for_statement_init(
        &mut self,
        init: &ast::ForStatementInit<'a>,
    ) -> hir::ForStatementInit<'a> {
        match init {
            ast::ForStatementInit::VariableDeclaration(decl) => {
                hir::ForStatementInit::VariableDeclaration(self.lower_variable_declaration(decl))
            }
            ast::ForStatementInit::Expression(expr) => {
                hir::ForStatementInit::Expression(self.lower_expression(expr))
            }
        }
    }

    fn lower_for_in_statement(&mut self, stmt: &ast::ForInStatement<'a>) -> hir::Statement<'a> {
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        self.enter_for_in_of_statement(is_lexical_declaration);
        let left = self.lower_for_statement_left(&stmt.left);
        let right = self.lower_expression(&stmt.right);
        let body = self.lower_statement(&stmt.body);
        self.leave_for_in_of_statement(is_lexical_declaration);
        self.hir.for_in_statement(stmt.span, left, right, body)
    }

    fn lower_for_of_statement(&mut self, stmt: &ast::ForOfStatement<'a>) -> hir::Statement<'a> {
        let is_lexical_declaration = stmt.left.is_lexical_declaration();
        self.enter_for_in_of_statement(is_lexical_declaration);
        let left = self.lower_for_statement_left(&stmt.left);
        let right = self.lower_expression(&stmt.right);
        let body = self.lower_statement(&stmt.body);
        self.leave_for_in_of_statement(is_lexical_declaration);
        self.hir.for_of_statement(stmt.span, stmt.r#await, left, right, body)
    }

    fn lower_for_statement_left(
        &mut self,
        left: &ast::ForStatementLeft<'a>,
    ) -> hir::ForStatementLeft<'a> {
        match left {
            ast::ForStatementLeft::VariableDeclaration(decl) => {
                hir::ForStatementLeft::VariableDeclaration(self.lower_variable_declaration(decl))
            }
            ast::ForStatementLeft::AssignmentTarget(target) => {
                hir::ForStatementLeft::AssignmentTarget(self.lower_assignment_target(target))
            }
        }
    }

    fn lower_if_statement(&mut self, stmt: &ast::IfStatement<'a>) -> hir::Statement<'a> {
        let test = self.lower_expression(&stmt.test);
        let consequent = self.lower_statement(&stmt.consequent);
        let alternate = stmt.alternate.as_ref().and_then(|stmt| self.lower_statement(stmt));
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
        let consequent = self.lower_statements(&case.consequent);
        self.hir.switch_case(case.span, test, consequent)
    }

    fn lower_throw_statement(&mut self, stmt: &ast::ThrowStatement<'a>) -> hir::Statement<'a> {
        let argument = self.lower_expression(&stmt.argument);
        self.hir.throw_statement(stmt.span, argument)
    }

    fn lower_try_statement(&mut self, stmt: &ast::TryStatement<'a>) -> hir::Statement<'a> {
        let block = self.lower_block_statement(&stmt.block);
        let handler = stmt.handler.as_ref().map(|clause| self.lower_catch_clause(clause));
        let finalizer = stmt.finalizer.as_ref().map(|stmt| self.lower_block_statement(stmt));
        self.hir.try_statement(stmt.span, block, handler, finalizer)
    }

    fn lower_catch_clause(
        &mut self,
        clause: &ast::CatchClause<'a>,
    ) -> Box<'a, hir::CatchClause<'a>> {
        self.enter_catch_clause();
        let param = clause.param.as_ref().map(|pat| {
            self.lower_binding_pattern(
                pat,
                SymbolFlags::CatchVariable | SymbolFlags::BlockScopedVariable,
                SymbolFlags::BlockScopedVariableExcludes,
            )
        });
        let body = self.lower_statements(&clause.body.body);
        let body = self.hir.block(clause.body.span, body);
        self.leave_catch_clause();
        self.hir.catch_clause(clause.span, param, body)
    }

    fn lower_while_statement(&mut self, stmt: &ast::WhileStatement<'a>) -> hir::Statement<'a> {
        let test = self.lower_expression(&stmt.test);
        let body = self.lower_statement(&stmt.body);
        self.hir.while_statement(stmt.span, test, body)
    }

    fn lower_with_statement(&mut self, stmt: &ast::WithStatement<'a>) -> hir::Statement<'a> {
        let object = self.lower_expression(&stmt.object);
        let body = self.lower_statement(&stmt.body);
        self.hir.with_statement(stmt.span, object, body)
    }

    fn lower_expression(&mut self, expr: &ast::Expression<'a>) -> hir::Expression<'a> {
        ensure_sufficient_stack(|| {
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
                    let lit = self.lower_identifier_reference(ident, ReferenceFlag::Read);
                    self.hir.identifier_reference_expression(lit)
                }
                ast::Expression::MetaProperty(meta) => self.lower_meta_property(meta),
                ast::Expression::ArrayExpression(expr) => self.lower_array_expression(expr),
                ast::Expression::ArrowExpression(expr) => self.lower_arrow_expression(expr),
                ast::Expression::AssignmentExpression(expr) => {
                    self.lower_assignment_expression(expr)
                }
                ast::Expression::AwaitExpression(expr) => self.lower_await_expression(expr),
                ast::Expression::BinaryExpression(expr) => self.lower_binary_expression(expr),
                ast::Expression::CallExpression(expr) => self.lower_call_expression(expr),
                ast::Expression::ChainExpression(expr) => self.lower_chain_expression(expr),
                ast::Expression::ClassExpression(expr) => self.lower_class_expression(expr),
                ast::Expression::ConditionalExpression(expr) => {
                    self.lower_conditional_expression(expr)
                }
                ast::Expression::FunctionExpression(expr) => self.lower_function_expression(expr),
                ast::Expression::ImportExpression(expr) => self.lower_import_expression(expr),
                ast::Expression::LogicalExpression(expr) => self.lower_logical_expression(expr),
                ast::Expression::MemberExpression(expr) => self.lower_member_expression(expr),
                ast::Expression::NewExpression(expr) => self.lower_new_expression(expr),
                ast::Expression::ObjectExpression(expr) => self.lower_object_expression(expr),
                ast::Expression::ParenthesizedExpression(expr) => {
                    self.lower_parenthesized_expression(expr)
                }
                ast::Expression::PrivateInExpression(expr) => {
                    self.lower_private_in_expression(expr)
                }
                ast::Expression::SequenceExpression(expr) => self.lower_sequence_expression(expr),
                ast::Expression::TaggedTemplateExpression(expr) => {
                    self.lower_tagged_template_expression(expr)
                }
                ast::Expression::ThisExpression(expr) => self.lower_this_expression(expr),
                ast::Expression::UnaryExpression(expr) => self.lower_unary_expression(expr),
                ast::Expression::UpdateExpression(expr) => self.lower_update_expression(expr),
                ast::Expression::YieldExpression(expr) => self.lower_yield_expression(expr),
                ast::Expression::Super(expr) => self.lower_super(expr),
                ast::Expression::JSXElement(elem) => {
                    // TODO: implement JSX
                    let ident = self.lower_identifier_reference(
                        &ast::IdentifierReference { span: elem.span, name: "undefined".into() },
                        ReferenceFlag::Read,
                    );
                    self.hir.identifier_reference_expression(ident)
                }
                ast::Expression::JSXFragment(elem) => {
                    // TODO: implement JSX
                    let ident = self.lower_identifier_reference(
                        &ast::IdentifierReference { span: elem.span, name: "undefined".into() },
                        ReferenceFlag::Read,
                    );
                    self.hir.identifier_reference_expression(ident)
                }
                // Syntax trimmed for the following expressions
                ast::Expression::TSAsExpression(expr) => self.lower_expression(&expr.expression),
                ast::Expression::TSSatisfiesExpression(expr) => {
                    self.lower_expression(&expr.expression)
                }
                ast::Expression::TSNonNullExpression(expr) => {
                    self.lower_expression(&expr.expression)
                }
                ast::Expression::TSTypeAssertion(expr) => self.lower_expression(&expr.expression),
                ast::Expression::TSInstantiationExpression(expr) => {
                    self.lower_expression(&expr.expression)
                }
            }
        })
    }

    fn lower_meta_property(&mut self, prop: &ast::MetaProperty) -> hir::Expression<'a> {
        let meta = self.lower_identifier_name(&prop.meta);
        let property = self.lower_identifier_name(&prop.property);
        self.hir.meta_property(prop.span, meta, property)
    }

    fn lower_array_expression(&mut self, expr: &ast::ArrayExpression<'a>) -> hir::Expression<'a> {
        let elements = self.lower_vec(&expr.elements, Self::lower_array_expression_element);
        self.hir.array_expression(expr.span, elements, expr.trailing_comma)
    }

    fn lower_array_expression_element(
        &mut self,
        elem: &ast::ArrayExpressionElement<'a>,
    ) -> hir::ArrayExpressionElement<'a> {
        match elem {
            ast::ArrayExpressionElement::SpreadElement(elem) => {
                let elem = self.lower_spread_element(elem);
                hir::ArrayExpressionElement::SpreadElement(elem)
            }
            ast::ArrayExpressionElement::Expression(expr) => {
                let expr = self.lower_expression(expr);
                hir::ArrayExpressionElement::Expression(expr)
            }
            ast::ArrayExpressionElement::Elision(span) => {
                hir::ArrayExpressionElement::Elision(*span)
            }
        }
    }

    fn lower_argument(&mut self, arg: &ast::Argument<'a>) -> hir::Argument<'a> {
        match arg {
            ast::Argument::SpreadElement(elem) => {
                let spread_element = self.lower_spread_element(elem);
                hir::Argument::SpreadElement(spread_element)
            }
            ast::Argument::Expression(expr) => {
                hir::Argument::Expression(self.lower_expression(expr))
            }
        }
    }

    fn lower_spread_element(
        &mut self,
        elem: &ast::SpreadElement<'a>,
    ) -> Box<'a, hir::SpreadElement<'a>> {
        let argument = self.lower_expression(&elem.argument);
        self.hir.spread_element(elem.span, argument)
    }

    fn lower_assignment_expression(
        &mut self,
        expr: &ast::AssignmentExpression<'a>,
    ) -> hir::Expression<'a> {
        let left = self.lower_assignment_target(&expr.left);
        let right = self.lower_expression(&expr.right);
        self.hir.assignment_expression(expr.span, expr.operator, left, right)
    }

    fn lower_arrow_expression(&mut self, expr: &ast::ArrowExpression<'a>) -> hir::Expression<'a> {
        self.enter_function_scope();
        let params = self.lower_formal_parameters(&expr.params);
        let body = self.lower_function_body(&expr.body);
        self.leave_function_scope();
        self.hir.arrow_expression(
            expr.span,
            expr.expression,
            expr.generator,
            expr.r#async,
            params,
            body,
        )
    }

    fn lower_await_expression(&mut self, expr: &ast::AwaitExpression<'a>) -> hir::Expression<'a> {
        let argument = self.lower_expression(&expr.argument);
        self.hir.await_expression(expr.span, argument)
    }

    fn lower_binary_expression(&mut self, expr: &ast::BinaryExpression<'a>) -> hir::Expression<'a> {
        let left = self.lower_expression(&expr.left);
        let right = self.lower_expression(&expr.right);
        self.hir.binary_expression(expr.span, left, expr.operator, right)
    }

    fn lower_call_expression(&mut self, expr: &ast::CallExpression<'a>) -> hir::Expression<'a> {
        let callee = self.lower_expression(&expr.callee);
        let arguments = self.lower_vec(&expr.arguments, Self::lower_argument);
        self.hir.call_expression(expr.span, callee, arguments, expr.optional)
    }

    fn lower_chain_expression(&mut self, expr: &ast::ChainExpression<'a>) -> hir::Expression<'a> {
        let expression = match &expr.expression {
            ast::ChainElement::CallExpression(call_expr) => {
                let hir::Expression::CallExpression(call_expr) =
                    self.lower_call_expression(call_expr)
                else {
                    unreachable!()
                };
                hir::ChainElement::CallExpression(call_expr)
            }
            ast::ChainElement::MemberExpression(member_expr) => {
                let hir::Expression::MemberExpression(member_expr) =
                    self.lower_member_expression(member_expr)
                else {
                    unreachable!()
                };
                hir::ChainElement::MemberExpression(member_expr)
            }
        };
        self.hir.chain_expression(expr.span, expression)
    }

    fn lower_class_expression(&mut self, class: &ast::Class<'a>) -> hir::Expression<'a> {
        let class = self.lower_class(class);
        self.hir.class_expression(class)
    }

    fn lower_conditional_expression(
        &mut self,
        expr: &ast::ConditionalExpression<'a>,
    ) -> hir::Expression<'a> {
        let test = self.lower_expression(&expr.test);
        let consequent = self.lower_expression(&expr.consequent);
        let alternate = self.lower_expression(&expr.alternate);
        self.hir.conditional_expression(expr.span, test, consequent, alternate)
    }

    fn lower_function_expression(&mut self, func: &ast::Function<'a>) -> hir::Expression<'a> {
        let func = self.lower_function(func);
        self.hir.function_expression(func)
    }

    fn lower_import_expression(&mut self, expr: &ast::ImportExpression<'a>) -> hir::Expression<'a> {
        let source = self.lower_expression(&expr.source);
        let arguments = self.lower_vec(&expr.arguments, Self::lower_expression);
        self.hir.import_expression(expr.span, source, arguments)
    }

    fn lower_logical_expression(
        &mut self,
        expr: &ast::LogicalExpression<'a>,
    ) -> hir::Expression<'a> {
        let left = self.lower_expression(&expr.left);
        let right = self.lower_expression(&expr.right);
        self.hir.logical_expression(expr.span, left, expr.operator, right)
    }

    fn lower_member_expr(&mut self, expr: &ast::MemberExpression<'a>) -> hir::MemberExpression<'a> {
        match expr {
            ast::MemberExpression::ComputedMemberExpression(expr) => {
                self.lower_computed_member_expression(expr)
            }
            ast::MemberExpression::StaticMemberExpression(expr) => {
                self.lower_static_member_expression(expr)
            }
            ast::MemberExpression::PrivateFieldExpression(expr) => {
                self.lower_private_field_expression(expr)
            }
        }
    }

    fn lower_member_expression(&mut self, expr: &ast::MemberExpression<'a>) -> hir::Expression<'a> {
        let member_expr = self.lower_member_expr(expr);
        self.hir.member_expression(member_expr)
    }

    fn lower_computed_member_expression(
        &mut self,
        expr: &ast::ComputedMemberExpression<'a>,
    ) -> hir::MemberExpression<'a> {
        let object = self.lower_expression(&expr.object);
        let expression = self.lower_expression(&expr.expression);
        self.hir.computed_member_expression(expr.span, object, expression, expr.optional)
    }

    fn lower_static_member_expression(
        &mut self,
        expr: &ast::StaticMemberExpression<'a>,
    ) -> hir::MemberExpression<'a> {
        let object = self.lower_expression(&expr.object);
        let property = self.lower_identifier_name(&expr.property);
        self.hir.static_member_expression(expr.span, object, property, expr.optional)
    }

    fn lower_private_field_expression(
        &mut self,
        expr: &ast::PrivateFieldExpression<'a>,
    ) -> hir::MemberExpression<'a> {
        let object = self.lower_expression(&expr.object);
        let field = self.lower_private_identifier(&expr.field);
        self.hir.private_field_expression(expr.span, object, field, expr.optional)
    }

    fn lower_new_expression(&mut self, expr: &ast::NewExpression<'a>) -> hir::Expression<'a> {
        let callee = self.lower_expression(&expr.callee);
        let arguments = self.lower_vec(&expr.arguments, Self::lower_argument);
        self.hir.new_expression(expr.span, callee, arguments)
    }

    fn lower_object_expression(&mut self, expr: &ast::ObjectExpression<'a>) -> hir::Expression<'a> {
        let properties = self.lower_vec(&expr.properties, Self::lower_object_property_kind);
        self.hir.object_expression(expr.span, properties, expr.trailing_comma)
    }

    fn lower_parenthesized_expression(
        &mut self,
        expr: &ast::ParenthesizedExpression<'a>,
    ) -> hir::Expression<'a> {
        self.lower_expression(&expr.expression)
    }

    fn lower_object_property_kind(
        &mut self,
        prop: &ast::ObjectPropertyKind<'a>,
    ) -> hir::ObjectPropertyKind<'a> {
        match prop {
            ast::ObjectPropertyKind::ObjectProperty(property) => {
                let property = self.lower_object_property(property);
                hir::ObjectPropertyKind::ObjectProperty(property)
            }
            ast::ObjectPropertyKind::SpreadProperty(spread_element) => {
                let spread_element = self.lower_spread_element(spread_element);
                hir::ObjectPropertyKind::SpreadProperty(spread_element)
            }
        }
    }

    fn lower_object_property(
        &mut self,
        prop: &ast::ObjectProperty<'a>,
    ) -> Box<'a, hir::ObjectProperty<'a>> {
        let kind = match prop.kind {
            ast::PropertyKind::Init => hir::PropertyKind::Init,
            ast::PropertyKind::Get => hir::PropertyKind::Get,
            ast::PropertyKind::Set => hir::PropertyKind::Set,
        };
        let key = self.lower_property_key(&prop.key);
        let value = self.lower_expression(&prop.value);
        self.hir.object_property(
            prop.span,
            kind,
            key,
            value,
            prop.method,
            prop.shorthand,
            prop.computed,
        )
    }

    fn lower_property_key(&mut self, key: &ast::PropertyKey<'a>) -> hir::PropertyKey<'a> {
        match key {
            ast::PropertyKey::Identifier(ident) => {
                let ident = self.lower_identifier_name(ident);
                self.hir.property_key_identifier(ident)
            }
            ast::PropertyKey::PrivateIdentifier(ident) => {
                let ident = self.lower_private_identifier(ident);
                self.hir.property_key_private_identifier(ident)
            }
            ast::PropertyKey::Expression(expr) => {
                hir::PropertyKey::Expression(self.lower_expression(expr))
            }
        }
    }

    fn lower_private_in_expression(
        &mut self,
        expr: &ast::PrivateInExpression<'a>,
    ) -> hir::Expression<'a> {
        let left = self.lower_private_identifier(&expr.left);
        let right = self.lower_expression(&expr.right);
        self.hir.private_in_expression(expr.span, left, right)
    }

    fn lower_sequence_expression(
        &mut self,
        expr: &ast::SequenceExpression<'a>,
    ) -> hir::Expression<'a> {
        let expressions = self.lower_vec(&expr.expressions, Self::lower_expression);
        self.hir.sequence_expression(expr.span, expressions)
    }

    fn lower_tagged_template_expression(
        &mut self,
        expr: &ast::TaggedTemplateExpression<'a>,
    ) -> hir::Expression<'a> {
        let tag = self.lower_expression(&expr.tag);
        let quasi = self.lower_template_literal(&expr.quasi);
        self.hir.tagged_template_expression(expr.span, tag, quasi)
    }

    fn lower_this_expression(&mut self, expr: &ast::ThisExpression) -> hir::Expression<'a> {
        self.hir.this_expression(expr.span)
    }

    fn lower_unary_expression(&mut self, expr: &ast::UnaryExpression<'a>) -> hir::Expression<'a> {
        let argument = self.lower_expression(&expr.argument);
        self.hir.unary_expression(expr.span, expr.operator, argument)
    }

    fn lower_update_expression(&mut self, expr: &ast::UpdateExpression<'a>) -> hir::Expression<'a> {
        let argument = self.lower_simple_assignment_target(&expr.argument);
        self.hir.update_expression(expr.span, expr.operator, expr.prefix, argument)
    }

    fn lower_yield_expression(&mut self, expr: &ast::YieldExpression<'a>) -> hir::Expression<'a> {
        let argument = expr.argument.as_ref().map(|expr| self.lower_expression(expr));
        self.hir.yield_expression(expr.span, expr.delegate, argument)
    }

    fn lower_super(&mut self, expr: &ast::Super) -> hir::Expression<'a> {
        self.hir.super_expression(expr.span)
    }

    fn lower_assignment_target(
        &mut self,
        target: &ast::AssignmentTarget<'a>,
    ) -> hir::AssignmentTarget<'a> {
        match target {
            ast::AssignmentTarget::SimpleAssignmentTarget(target) => {
                hir::AssignmentTarget::SimpleAssignmentTarget(
                    self.lower_simple_assignment_target(target),
                )
            }
            ast::AssignmentTarget::AssignmentTargetPattern(target) => {
                hir::AssignmentTarget::AssignmentTargetPattern(
                    self.lower_assignment_target_pattern(target),
                )
            }
        }
    }

    fn lower_simple_assignment_target(
        &mut self,
        target: &ast::SimpleAssignmentTarget<'a>,
    ) -> hir::SimpleAssignmentTarget<'a> {
        match target {
            ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let ident = self.lower_identifier_reference(ident, ReferenceFlag::Write);
                self.hir.assignment_target_identifier(ident)
            }
            ast::SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) => {
                let member_expr = self.lower_member_expr(member_expr);
                self.hir.member_assignment_target(member_expr)
            }
            ast::SimpleAssignmentTarget::TSAsExpression(expr) => {
                self.lower_assignment_target_expression(&expr.expression)
            }
            ast::SimpleAssignmentTarget::TSSatisfiesExpression(expr) => {
                self.lower_assignment_target_expression(&expr.expression)
            }
            ast::SimpleAssignmentTarget::TSNonNullExpression(expr) => {
                self.lower_assignment_target_expression(&expr.expression)
            }
            ast::SimpleAssignmentTarget::TSTypeAssertion(expr) => {
                self.lower_assignment_target_expression(&expr.expression)
            }
        }
    }

    fn lower_assignment_target_expression(
        &mut self,
        expr: &ast::Expression<'a>,
    ) -> hir::SimpleAssignmentTarget<'a> {
        match expr {
            ast::Expression::Identifier(ident) => {
                let ident = self.lower_identifier_reference(ident, ReferenceFlag::Write);
                self.hir.assignment_target_identifier(ident)
            }
            ast::Expression::MemberExpression(member_expr) => {
                let member_expr = self.lower_member_expr(member_expr);
                self.hir.member_assignment_target(member_expr)
            }
            expr => {
                // return undefined because this is invalid syntax
                let ident = self.lower_identifier_reference(
                    &ast::IdentifierReference { span: expr.span(), name: "undefined".into() },
                    ReferenceFlag::Write,
                );
                self.hir.assignment_target_identifier(ident)
            }
        }
    }

    fn lower_assignment_target_pattern(
        &mut self,
        pat: &ast::AssignmentTargetPattern<'a>,
    ) -> hir::AssignmentTargetPattern<'a> {
        match pat {
            ast::AssignmentTargetPattern::ArrayAssignmentTarget(target) => {
                let target = self.lower_array_assignment_target(target);
                hir::AssignmentTargetPattern::ArrayAssignmentTarget(target)
            }
            ast::AssignmentTargetPattern::ObjectAssignmentTarget(target) => {
                let target = self.lower_object_assignment_target(target);
                hir::AssignmentTargetPattern::ObjectAssignmentTarget(target)
            }
        }
    }

    fn lower_array_assignment_target(
        &mut self,
        target: &ast::ArrayAssignmentTarget<'a>,
    ) -> Box<'a, hir::ArrayAssignmentTarget<'a>> {
        let mut elements = self.hir.new_vec_with_capacity(target.elements.len());
        for elem in &target.elements {
            let elem = elem.as_ref().map(|elem| self.lower_assignment_target_maybe_default(elem));
            elements.push(elem);
        }
        let rest = target.rest.as_ref().map(|target| self.lower_assignment_target(target));
        self.hir.array_assignment_target(target.span, elements, rest, target.trailing_comma)
    }

    fn lower_object_assignment_target(
        &mut self,
        target: &ast::ObjectAssignmentTarget<'a>,
    ) -> Box<'a, hir::ObjectAssignmentTarget<'a>> {
        let properties = self.lower_vec(&target.properties, Self::lower_assignment_target_property);
        let rest = target.rest.as_ref().map(|target| self.lower_assignment_target(target));
        self.hir.object_assignment_target(target.span, properties, rest)
    }

    fn lower_assignment_target_maybe_default(
        &mut self,
        target: &ast::AssignmentTargetMaybeDefault<'a>,
    ) -> hir::AssignmentTargetMaybeDefault<'a> {
        match target {
            ast::AssignmentTargetMaybeDefault::AssignmentTarget(target) => {
                let target = self.lower_assignment_target(target);
                hir::AssignmentTargetMaybeDefault::AssignmentTarget(target)
            }
            ast::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
                let target = self.lower_assignment_target_with_default(target);
                hir::AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target)
            }
        }
    }

    fn lower_assignment_target_with_default(
        &mut self,
        target: &ast::AssignmentTargetWithDefault<'a>,
    ) -> Box<'a, hir::AssignmentTargetWithDefault<'a>> {
        let binding = self.lower_assignment_target(&target.binding);
        let init = self.lower_expression(&target.init);
        self.hir.assignment_target_with_default(target.span, binding, init)
    }

    fn lower_assignment_target_property(
        &mut self,
        property: &ast::AssignmentTargetProperty<'a>,
    ) -> hir::AssignmentTargetProperty<'a> {
        match property {
            ast::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                let ident = self.lower_assignment_target_property_identifier(ident);
                hir::AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident)
            }
            ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                let prop = self.lower_assignment_target_property_property(prop);
                hir::AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop)
            }
        }
    }

    fn lower_assignment_target_property_identifier(
        &mut self,
        ident: &ast::AssignmentTargetPropertyIdentifier<'a>,
    ) -> Box<'a, hir::AssignmentTargetPropertyIdentifier<'a>> {
        let binding = self.lower_identifier_reference(&ident.binding, ReferenceFlag::Write);
        let init = ident.init.as_ref().map(|expr| self.lower_expression(expr));
        self.hir.assignment_target_property_identifier(ident.span, binding, init)
    }

    fn lower_assignment_target_property_property(
        &mut self,
        property: &ast::AssignmentTargetPropertyProperty<'a>,
    ) -> Box<'a, hir::AssignmentTargetPropertyProperty<'a>> {
        let name = self.lower_property_key(&property.name);
        let binding = self.lower_assignment_target_maybe_default(&property.binding);
        self.hir.assignment_target_property_property(property.span, name, binding)
    }

    // fn lower_jsx_element(&mut self, elem: &ast::JSXElement<'a>) {
    // todo!()
    // }

    // fn lower_jsx_opening_element(&mut self, elem: &ast::JSXOpeningElement<'a>) {
    // todo!()
    // }

    // fn lower_jsx_element_name(&mut self, __name: &ast::JSXElementName<'a>) {
    // todo!()
    // }

    // fn lower_jsx_attribute_item(&mut self, item: &ast::JSXAttributeItem<'a>) {
    // todo!()
    // }

    // fn lower_jsx_attribute(&mut self, attribute: &ast::JSXAttribute<'a>) {
    // todo!()
    // }

    // fn lower_jsx_spread_attribute(&mut self, attribute: &ast::JSXSpreadAttribute<'a>) {
    // todo!()
    // }

    // fn lower_jsx_attribute_value(&mut self, value: &ast::JSXAttributeValue<'a>) {
    // todo!()
    // }

    // fn lower_jsx_expression_container(&mut self, expr: &ast::JSXExpressionContainer<'a>) {
    // todo!()
    // }

    // fn lower_jsx_expression(&mut self, expr: &ast::JSXExpression<'a>) {
    // todo!()
    // }

    // fn lower_jsx_fragment(&mut self, elem: &ast::JSXFragment<'a>) {
    // todo!()
    // }

    // fn lower_jsx_child(&mut self, child: &ast::JSXChild<'a>) {
    // todo!()
    // }

    // fn lower_jsx_spread_child(&mut self, child: &ast::JSXSpreadChild<'a>) {
    // todo!()
    // }

    /* ----------  Pattern ---------- */

    fn lower_binding_pattern(
        &mut self,
        pat: &ast::BindingPattern<'a>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> hir::BindingPattern<'a> {
        match &pat.kind {
            ast::BindingPatternKind::BindingIdentifier(ident) => {
                let ident = self.lower_binding_identifier(ident, includes, excludes);
                self.hir.binding_identifier_pattern(ident)
            }
            ast::BindingPatternKind::ObjectPattern(pat) => {
                self.lower_object_pattern(pat, includes, excludes)
            }
            ast::BindingPatternKind::ArrayPattern(pat) => {
                self.lower_array_pattern(pat, includes, excludes)
            }
            ast::BindingPatternKind::AssignmentPattern(pat) => {
                self.lower_assignment_pattern(pat, includes, excludes)
            }
        }
    }

    fn lower_object_pattern(
        &mut self,
        pat: &ast::ObjectPattern<'a>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> hir::BindingPattern<'a> {
        let properties = self.lower_vec(&pat.properties, |p, prop| {
            p.lower_binding_property(prop, includes, excludes)
        });
        let rest = pat.rest.as_ref().map(|rest| self.lower_rest_element(rest, includes, excludes));
        self.hir.object_pattern(pat.span, properties, rest)
    }

    fn lower_binding_property(
        &mut self,
        prop: &ast::BindingProperty<'a>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> hir::BindingProperty<'a> {
        let key = self.lower_property_key(&prop.key);
        let value = self.lower_binding_pattern(&prop.value, includes, excludes);
        self.hir.binding_property(prop.span, key, value, prop.shorthand, prop.computed)
    }

    fn lower_array_pattern(
        &mut self,
        pat: &ast::ArrayPattern<'a>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> hir::BindingPattern<'a> {
        let mut elements = self.hir.new_vec_with_capacity(pat.elements.len());
        for elem in &pat.elements {
            let elem = elem.as_ref().map(|pat| self.lower_binding_pattern(pat, includes, excludes));
            elements.push(elem);
        }
        let rest = pat.rest.as_ref().map(|rest| self.lower_rest_element(rest, includes, excludes));
        self.hir.array_pattern(pat.span, elements, rest)
    }

    fn lower_rest_element(
        &mut self,
        pat: &ast::RestElement<'a>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> Box<'a, hir::RestElement<'a>> {
        let argument = self.lower_binding_pattern(&pat.argument, includes, excludes);
        self.hir.rest_element(pat.span, argument)
    }

    fn lower_assignment_pattern(
        &mut self,
        pat: &ast::AssignmentPattern<'a>,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> hir::BindingPattern<'a> {
        let left = self.lower_binding_pattern(&pat.left, includes, excludes);
        let right = self.lower_expression(&pat.right);
        self.hir.assignment_pattern(pat.span, left, right)
    }

    /* ----------  Identifier ---------- */

    fn lower_identifier_reference(
        &mut self,
        ident: &ast::IdentifierReference,
        reference_flag: ReferenceFlag,
    ) -> hir::IdentifierReference {
        let reference_id = self.enter_identifier_reference(ident.span, &ident.name, reference_flag);
        self.hir.identifier_reference(ident.span, ident.name.clone(), reference_id, reference_flag)
    }

    fn lower_private_identifier(
        &mut self,
        ident: &ast::PrivateIdentifier,
    ) -> hir::PrivateIdentifier {
        self.hir.private_identifier(ident.span, ident.name.clone())
    }

    fn lower_label_identifier(&mut self, ident: &ast::LabelIdentifier) -> hir::LabelIdentifier {
        self.hir.label_identifier(ident.span, ident.name.clone())
    }

    fn lower_identifier_name(&mut self, ident: &ast::IdentifierName) -> hir::IdentifierName {
        self.hir.identifier_name(ident.span, ident.name.clone())
    }

    fn lower_binding_identifier(
        &mut self,
        ident: &ast::BindingIdentifier,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> hir::BindingIdentifier {
        let symbol_id = self.enter_binding_identifier(ident.span, &ident.name, includes, excludes);
        self.hir.binding_identifier(ident.span, ident.name.clone(), symbol_id)
    }

    /* ----------  Literal ---------- */

    fn lower_number_literal(&mut self, lit: &ast::NumberLiteral<'a>) -> hir::NumberLiteral<'a> {
        self.hir.number_literal(lit.span, lit.value, lit.raw, lit.base)
    }

    fn lower_boolean_literal(&mut self, lit: &ast::BooleanLiteral) -> hir::BooleanLiteral {
        self.hir.boolean_literal(lit.span, lit.value)
    }

    fn lower_null_literal(&mut self, lit: &ast::NullLiteral) -> hir::NullLiteral {
        self.hir.null_literal(lit.span)
    }

    fn lower_bigint_literal(&mut self, lit: &ast::BigintLiteral) -> hir::BigintLiteral {
        self.hir.bigint_literal(lit.span, lit.value.clone())
    }

    fn lower_string_literal(&mut self, lit: &ast::StringLiteral) -> hir::StringLiteral {
        self.hir.string_literal(lit.span, lit.value.clone())
    }

    fn lower_template_literal(
        &mut self,
        lit: &ast::TemplateLiteral<'a>,
    ) -> hir::TemplateLiteral<'a> {
        let quasis = self.lower_vec(&lit.quasis, Self::lower_template_element);
        let expressions = self.lower_vec(&lit.expressions, Self::lower_expression);
        self.hir.template_literal(lit.span, quasis, expressions)
    }

    fn lower_reg_expr_literal(&mut self, lit: &ast::RegExpLiteral) -> hir::RegExpLiteral {
        let flags = hir::RegExpFlags::from_bits(lit.regex.flags.bits()).unwrap();
        self.hir.reg_exp_literal(lit.span, lit.regex.pattern.clone(), flags)
    }

    fn lower_template_element(&mut self, elem: &ast::TemplateElement) -> hir::TemplateElement {
        let value = self.lower_template_element_value(&elem.value);
        self.hir.template_element(elem.span, elem.tail, value)
    }

    fn lower_template_element_value(
        &mut self,
        elem: &ast::TemplateElementValue,
    ) -> hir::TemplateElementValue {
        self.hir.template_element_value(elem.raw.clone(), elem.cooked.clone())
    }

    /* ----------  Module ---------- */

    fn lower_module_declaration(
        &mut self,
        decl: &ast::ModuleDeclaration<'a>,
    ) -> Option<hir::Statement<'a>> {
        let decl = match decl {
            ast::ModuleDeclaration::ImportDeclaration(decl) => {
                let decl = self.lower_import_declaration(decl);
                hir::ModuleDeclaration::ImportDeclaration(decl)
            }
            ast::ModuleDeclaration::ExportAllDeclaration(decl) => {
                let decl = self.lower_export_all_declaration(decl);
                hir::ModuleDeclaration::ExportAllDeclaration(decl)
            }
            ast::ModuleDeclaration::ExportDefaultDeclaration(decl) => {
                let decl = self.lower_export_default_declaration(decl)?;
                hir::ModuleDeclaration::ExportDefaultDeclaration(decl)
            }
            ast::ModuleDeclaration::ExportNamedDeclaration(decl) => {
                let decl = self.lower_export_named_declaration(decl);
                hir::ModuleDeclaration::ExportNamedDeclaration(decl)
            }
            ast::ModuleDeclaration::TSExportAssignment(_)
            | ast::ModuleDeclaration::TSNamespaceExportDeclaration(_) => {
                return None;
            }
        };
        Some(self.hir.module_declaration(decl))
    }

    fn lower_import_declaration(
        &mut self,
        decl: &ast::ImportDeclaration<'a>,
    ) -> Box<'a, hir::ImportDeclaration<'a>> {
        let specifiers = self.lower_vec(&decl.specifiers, Self::lower_import_declaration_specifier);
        let source = self.lower_string_literal(&decl.source);
        let assertions = decl
            .assertions
            .as_ref()
            .map(|attributes| self.lower_vec(attributes, Self::lower_import_attribute));
        let import_kind = match decl.import_kind {
            ast::ImportOrExportKind::Value => hir::ImportOrExportKind::Value,
            ast::ImportOrExportKind::Type => hir::ImportOrExportKind::Type,
        };
        self.hir.import_declaration(decl.span, specifiers, source, assertions, import_kind)
    }

    fn lower_import_attribute(&mut self, attribute: &ast::ImportAttribute) -> hir::ImportAttribute {
        let key = match &attribute.key {
            ast::ImportAttributeKey::Identifier(ident) => {
                let ident = self.lower_identifier_name(ident);
                hir::ImportAttributeKey::Identifier(ident)
            }
            ast::ImportAttributeKey::StringLiteral(lit) => {
                let lit = self.lower_string_literal(lit);
                hir::ImportAttributeKey::StringLiteral(lit)
            }
        };
        let value = self.lower_string_literal(&attribute.value);
        self.hir.import_attribute(attribute.span, key, value)
    }

    fn lower_import_declaration_specifier(
        &mut self,
        specifier: &ast::ImportDeclarationSpecifier,
    ) -> hir::ImportDeclarationSpecifier {
        match specifier {
            ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                let specifier = self.lower_import_specifier(specifier);
                hir::ImportDeclarationSpecifier::ImportSpecifier(specifier)
            }
            ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                let specifier = self.lower_import_default_specifier(specifier);
                hir::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier)
            }
            ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                let specifier = self.lower_import_name_specifier(specifier);
                hir::ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier)
            }
        }
    }

    fn lower_module_export_name(&mut self, name: &ast::ModuleExportName) -> hir::ModuleExportName {
        match name {
            ast::ModuleExportName::Identifier(ident) => {
                let ident = self.lower_identifier_name(ident);
                hir::ModuleExportName::Identifier(ident)
            }
            ast::ModuleExportName::StringLiteral(lit) => {
                let lit = self.lower_string_literal(lit);
                hir::ModuleExportName::StringLiteral(lit)
            }
        }
    }

    fn lower_import_specifier(&mut self, specifier: &ast::ImportSpecifier) -> hir::ImportSpecifier {
        let imported = self.lower_module_export_name(&specifier.imported);
        let local = self.lower_binding_identifier(
            &specifier.local,
            SymbolFlags::Import,
            SymbolFlags::empty(),
        );
        self.hir.import_specifier(specifier.span, imported, local)
    }
    fn lower_import_export_type_or_value(
        &mut self,
        import_export_kind: ast::ImportOrExportKind,
    ) -> hir::ImportOrExportKind {
        match import_export_kind {
            ast::ImportOrExportKind::Value => hir::ImportOrExportKind::Value,
            ast::ImportOrExportKind::Type => hir::ImportOrExportKind::Type,
        }
    }

    fn lower_import_default_specifier(
        &mut self,
        specifier: &ast::ImportDefaultSpecifier,
    ) -> hir::ImportDefaultSpecifier {
        let local = self.lower_binding_identifier(
            &specifier.local,
            SymbolFlags::Import,
            SymbolFlags::empty(),
        );
        self.hir.import_default_specifier(specifier.span, local)
    }

    fn lower_import_name_specifier(
        &mut self,
        specifier: &ast::ImportNamespaceSpecifier,
    ) -> hir::ImportNamespaceSpecifier {
        let local = self.lower_binding_identifier(
            &specifier.local,
            SymbolFlags::Import,
            SymbolFlags::empty(),
        );
        self.hir.import_namespace_specifier(specifier.span, local)
    }

    fn lower_export_all_declaration(
        &mut self,
        decl: &ast::ExportAllDeclaration<'a>,
    ) -> Box<'a, hir::ExportAllDeclaration<'a>> {
        let exported = decl.exported.as_ref().map(|name| self.lower_module_export_name(name));
        let source = self.lower_string_literal(&decl.source);
        let assertions = decl
            .assertions
            .as_ref()
            .map(|attributes| self.lower_vec(attributes, Self::lower_import_attribute));
        let export_kind = self.lower_import_export_type_or_value(decl.export_kind);
        self.hir.export_all_declaration(decl.span, exported, source, assertions, export_kind)
    }

    fn lower_export_default_declaration(
        &mut self,
        decl: &ast::ExportDefaultDeclaration<'a>,
    ) -> Option<Box<'a, hir::ExportDefaultDeclaration<'a>>> {
        let declaration = match &decl.declaration {
            ast::ExportDefaultDeclarationKind::Expression(expr) => {
                let expr = self.lower_expression(expr);
                hir::ExportDefaultDeclarationKind::Expression(expr)
            }
            ast::ExportDefaultDeclarationKind::FunctionDeclaration(decl) => {
                let decl = self.lower_function(decl);
                hir::ExportDefaultDeclarationKind::FunctionDeclaration(decl)
            }
            ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                let class = self.lower_class(class);
                hir::ExportDefaultDeclarationKind::ClassDeclaration(class)
            }
            ast::ExportDefaultDeclarationKind::TSEnumDeclaration(decl) => {
                let decl = self.lower_ts_enum_declaration(decl)?;
                hir::ExportDefaultDeclarationKind::TSEnumDeclaration(decl)
            }
            ast::ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => return None,
        };
        let exported = self.lower_module_export_name(&decl.exported);
        Some(self.hir.export_default_declaration(decl.span, declaration, exported))
    }

    fn lower_export_named_declaration(
        &mut self,
        decl: &ast::ExportNamedDeclaration<'a>,
    ) -> Box<'a, hir::ExportNamedDeclaration<'a>> {
        let declaration = decl.declaration.as_ref().and_then(|decl| self.lower_declaration(decl));
        let specifiers = self.lower_vec(&decl.specifiers, Self::lower_export_specifier);
        let source = decl.source.as_ref().map(|source| self.lower_string_literal(source));
        let export_kind = match decl.export_kind {
            ast::ImportOrExportKind::Value => hir::ImportOrExportKind::Value,
            ast::ImportOrExportKind::Type => hir::ImportOrExportKind::Type,
        };
        self.hir.export_named_declaration(decl.span, declaration, specifiers, source, export_kind)
    }

    fn lower_export_specifier(&mut self, specifier: &ast::ExportSpecifier) -> hir::ExportSpecifier {
        let local = self.lower_module_export_name(&specifier.local);
        let exported = self.lower_module_export_name(&specifier.exported);
        let export_kind = self.lower_import_export_type_or_value(specifier.export_kind);
        self.hir.export_specifier(specifier.span, local, exported, export_kind)
    }

    fn lower_declaration(&mut self, decl: &ast::Declaration<'a>) -> Option<hir::Declaration<'a>> {
        match decl {
            ast::Declaration::VariableDeclaration(decl) => {
                Some(hir::Declaration::VariableDeclaration(self.lower_variable_declaration(decl)))
            }
            ast::Declaration::FunctionDeclaration(func) => {
                let func = self.lower_function(func);
                Some(hir::Declaration::FunctionDeclaration(func))
            }
            ast::Declaration::ClassDeclaration(class) => {
                let class = self.lower_class(class);
                Some(hir::Declaration::ClassDeclaration(class))
            }
            ast::Declaration::TSEnumDeclaration(decl) => {
                let decl = self.lower_ts_enum_declaration(decl)?;
                Some(hir::Declaration::TSEnumDeclaration(decl))
            }
            _ => None,
        }
    }

    fn lower_variable_declaration(
        &mut self,
        decl: &ast::VariableDeclaration<'a>,
    ) -> Box<'a, hir::VariableDeclaration<'a>> {
        let kind = match decl.kind {
            ast::VariableDeclarationKind::Var => hir::VariableDeclarationKind::Var,
            ast::VariableDeclarationKind::Const => hir::VariableDeclarationKind::Const,
            ast::VariableDeclarationKind::Let => hir::VariableDeclarationKind::Let,
        };
        let declarations = self.lower_vec(&decl.declarations, Self::lower_variable_declarator);
        self.hir.variable_declaration(decl.span, kind, declarations)
    }

    fn lower_variable_declarator(
        &mut self,
        decl: &ast::VariableDeclarator<'a>,
    ) -> hir::VariableDeclarator<'a> {
        let kind = match decl.kind {
            ast::VariableDeclarationKind::Var => hir::VariableDeclarationKind::Var,
            ast::VariableDeclarationKind::Const => hir::VariableDeclarationKind::Const,
            ast::VariableDeclarationKind::Let => hir::VariableDeclarationKind::Let,
        };

        let (includes, excludes) = if decl.kind.is_lexical() {
            (SymbolFlags::BlockScopedVariable, SymbolFlags::BlockScopedVariableExcludes)
        } else {
            (SymbolFlags::FunctionScopedVariable, SymbolFlags::FunctionScopedVariableExcludes)
        };

        let id = self.lower_binding_pattern(&decl.id, includes, excludes);
        let init = decl.init.as_ref().map(|expr| self.lower_expression(expr));
        self.hir.variable_declarator(decl.span, kind, id, init, decl.definite)
    }

    fn lower_function(&mut self, func: &ast::Function<'a>) -> Box<'a, hir::Function<'a>> {
        let r#type = match func.r#type {
            ast::FunctionType::FunctionDeclaration => hir::FunctionType::FunctionDeclaration,
            ast::FunctionType::FunctionExpression => hir::FunctionType::FunctionExpression,
            ast::FunctionType::TSDeclareFunction => hir::FunctionType::TSDeclareFunction,
        };
        let (includes, excludes) = if func.r#type == ast::FunctionType::FunctionDeclaration {
            (SymbolFlags::FunctionScopedVariable, SymbolFlags::FunctionScopedVariableExcludes)
        } else {
            (SymbolFlags::empty(), SymbolFlags::empty())
        };
        let includes = includes | SymbolFlags::Function;
        let id =
            func.id.as_ref().map(|ident| self.lower_binding_identifier(ident, includes, excludes));
        self.enter_function_scope();
        let params = self.lower_formal_parameters(&func.params);
        let body = func.body.as_ref().map(|body| self.lower_function_body(body));
        self.leave_function_scope();
        self.hir.function(
            r#type,
            func.span,
            id,
            func.expression,
            func.generator,
            func.r#async,
            params,
            body,
        )
    }

    fn lower_function_body(
        &mut self,
        body: &ast::FunctionBody<'a>,
    ) -> Box<'a, hir::FunctionBody<'a>> {
        let directives = self.lower_vec(&body.directives, Self::lower_directive);
        let statements = self.lower_statements(&body.statements);
        self.hir.function_body(body.span, directives, statements)
    }

    fn lower_formal_parameters(
        &mut self,
        params: &ast::FormalParameters<'a>,
    ) -> Box<'a, hir::FormalParameters<'a>> {
        let kind = match params.kind {
            ast::FormalParameterKind::FormalParameter => hir::FormalParameterKind::FormalParameter,
            ast::FormalParameterKind::UniqueFormalParameters => {
                hir::FormalParameterKind::UniqueFormalParameters
            }
            ast::FormalParameterKind::ArrowFormalParameters => {
                hir::FormalParameterKind::ArrowFormalParameters
            }
            ast::FormalParameterKind::Signature => hir::FormalParameterKind::Signature,
        };
        let items = self.lower_vec(&params.items, Self::lower_formal_parameter);
        let rest = params.rest.as_ref().map(|rest| {
            self.lower_rest_element(
                rest,
                SymbolFlags::FunctionScopedVariable,
                SymbolFlags::FunctionScopedVariableExcludes,
            )
        });
        self.hir.formal_parameters(params.span, kind, items, rest)
    }

    fn lower_formal_parameter(
        &mut self,
        param: &ast::FormalParameter<'a>,
    ) -> hir::FormalParameter<'a> {
        let pattern = self.lower_binding_pattern(
            &param.pattern,
            SymbolFlags::FunctionScopedVariable,
            SymbolFlags::FunctionScopedVariableExcludes,
        );
        let decorators = self.lower_vec(&param.decorators, Self::lower_decorator);
        self.hir.formal_parameter(param.span, pattern, decorators)
    }

    fn lower_class(&mut self, class: &ast::Class<'a>) -> Box<'a, hir::Class<'a>> {
        let r#type = match class.r#type {
            ast::ClassType::ClassDeclaration => hir::ClassType::ClassDeclaration,
            ast::ClassType::ClassExpression => hir::ClassType::ClassExpression,
        };
        let (includes, excludes) = if class.r#type == ast::ClassType::ClassDeclaration {
            (SymbolFlags::Class | SymbolFlags::BlockScopedVariable, SymbolFlags::ClassExcludes)
        } else {
            (SymbolFlags::empty(), SymbolFlags::empty())
        };
        let id =
            class.id.as_ref().map(|ident| self.lower_binding_identifier(ident, includes, excludes));
        let super_class = class.super_class.as_ref().map(|expr| self.lower_expression(expr));
        let body = self.lower_class_body(&class.body);
        let decorators = self.lower_vec(&class.decorators, Self::lower_decorator);
        self.hir.class(r#type, class.span, id, super_class, body, decorators)
    }

    fn lower_class_body(&mut self, class_body: &ast::ClassBody<'a>) -> Box<'a, hir::ClassBody<'a>> {
        let mut body = self.hir.new_vec_with_capacity(class_body.body.len());
        for elem in &class_body.body {
            if let Some(elem) = self.lower_class_element(elem) {
                body.push(elem);
            }
        }
        self.hir.class_body(class_body.span, body)
    }

    fn lower_class_element(
        &mut self,
        elem: &ast::ClassElement<'a>,
    ) -> Option<hir::ClassElement<'a>> {
        match elem {
            ast::ClassElement::StaticBlock(block) => {
                let block = self.lower_static_block(block);
                Some(hir::ClassElement::StaticBlock(block))
            }
            ast::ClassElement::MethodDefinition(def) => {
                let def = self.lower_method_definition(def);
                Some(hir::ClassElement::MethodDefinition(def))
            }
            ast::ClassElement::PropertyDefinition(def) => {
                let def = self.lower_property_definition(def);
                Some(hir::ClassElement::PropertyDefinition(def))
            }
            ast::ClassElement::AccessorProperty(prop) => {
                let prop = self.lower_accessor_property(prop);
                Some(hir::ClassElement::AccessorProperty(prop))
            }
            ast::ClassElement::TSAbstractMethodDefinition(_)
            | ast::ClassElement::TSAbstractPropertyDefinition(_)
            | ast::ClassElement::TSIndexSignature(_) => None,
        }
    }

    fn lower_static_block(
        &mut self,
        block: &ast::StaticBlock<'a>,
    ) -> Box<'a, hir::StaticBlock<'a>> {
        self.enter_static_block();
        let body = self.lower_statements(&block.body);
        self.leave_static_block();
        self.hir.static_block(block.span, body)
    }

    fn lower_method_definition(
        &mut self,
        def: &ast::MethodDefinition<'a>,
    ) -> Box<'a, hir::MethodDefinition<'a>> {
        let key = self.lower_property_key(&def.key);
        let value = self.lower_function(&def.value);
        let kind = match def.kind {
            ast::MethodDefinitionKind::Constructor => hir::MethodDefinitionKind::Constructor,
            ast::MethodDefinitionKind::Method => hir::MethodDefinitionKind::Method,
            ast::MethodDefinitionKind::Get => hir::MethodDefinitionKind::Get,
            ast::MethodDefinitionKind::Set => hir::MethodDefinitionKind::Set,
        };
        let decorators = self.lower_vec(&def.decorators, Self::lower_decorator);
        self.hir.method_definition(
            def.span,
            key,
            value,
            kind,
            def.computed,
            def.r#static,
            def.r#override,
            def.optional,
            decorators,
        )
    }

    fn lower_property_definition(
        &mut self,
        def: &ast::PropertyDefinition<'a>,
    ) -> Box<'a, hir::PropertyDefinition<'a>> {
        let key = self.lower_property_key(&def.key);
        let value = def.value.as_ref().map(|expr| self.lower_expression(expr));
        let decorators = self.lower_vec(&def.decorators, Self::lower_decorator);
        self.hir.property_definition(
            def.span,
            key,
            value,
            def.computed,
            def.r#static,
            def.declare,
            def.r#override,
            def.optional,
            def.definite,
            def.readonly,
            decorators,
        )
    }

    fn lower_accessor_property(
        &mut self,
        def: &ast::AccessorProperty<'a>,
    ) -> Box<'a, hir::AccessorProperty<'a>> {
        let key = self.lower_property_key(&def.key);
        let value = def.value.as_ref().map(|expr| self.lower_expression(expr));
        self.hir.accessor_property(def.span, key, value, def.computed, def.r#static)
    }

    fn lower_ts_enum_declaration(
        &mut self,
        _decl: &ast::TSEnumDeclaration<'a>,
    ) -> Option<Box<'a, hir::TSEnumDeclaration<'a>>> {
        None
    }

    fn lower_decorator(&mut self, decorator: &ast::Decorator<'a>) -> hir::Decorator<'a> {
        let expression = self.lower_expression(&decorator.expression);
        self.hir.decorator(decorator.span, expression)
    }
}
