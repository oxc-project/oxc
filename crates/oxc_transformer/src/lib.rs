#![warn(clippy::print_stdout)]
#![allow(clippy::wildcard_imports)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
mod context;
mod options;
// Presets: <https://babel.dev/docs/presets>
mod env;
mod es2015;
mod es2016;
mod es2018;
mod es2019;
mod es2020;
mod es2021;
mod react;
mod typescript;

mod helpers {
    pub mod bindings;
    pub mod module_imports;
}

use std::{path::Path, rc::Rc};

use es2016::ES2016;
use es2018::ES2018;
use es2019::ES2019;
use es2020::ES2020;
use es2021::ES2021;
use oxc_allocator::{Allocator, Vec};
use oxc_ast::{ast::*, Trivias};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::{ScopeTree, SymbolTable};
use oxc_span::{SourceType, SPAN};
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};

pub use crate::{
    compiler_assumptions::CompilerAssumptions,
    env::{EnvOptions, Targets},
    es2015::{ArrowFunctionsOptions, ES2015Options},
    options::{BabelOptions, TransformOptions},
    react::{ReactJsxRuntime, ReactOptions, ReactRefreshOptions},
    typescript::TypeScriptOptions,
};
use crate::{
    context::{Ctx, TransformCtx},
    es2015::ES2015,
    react::React,
    typescript::TypeScript,
};

pub struct TransformerReturn {
    pub errors: std::vec::Vec<OxcDiagnostic>,
    pub symbols: SymbolTable,
    pub scopes: ScopeTree,
}

pub struct Transformer<'a> {
    ctx: Ctx<'a>,
    // NOTE: all callbacks must run in order.
    x0_typescript: TypeScript<'a>,
    x1_react: React<'a>,
    x2_es2021: ES2021<'a>,
    x2_es2020: ES2020<'a>,
    x2_es2019: ES2019<'a>,
    x2_es2018: ES2018<'a>,
    x2_es2016: ES2016<'a>,
    x3_es2015: ES2015<'a>,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_path: &Path,
        source_type: SourceType,
        source_text: &'a str,
        trivias: Trivias,
        options: TransformOptions,
    ) -> Self {
        let ctx = Rc::new(TransformCtx::new(
            allocator,
            source_path,
            source_type,
            source_text,
            trivias,
            &options,
        ));
        Self {
            ctx: Rc::clone(&ctx),
            x0_typescript: TypeScript::new(options.typescript, Rc::clone(&ctx)),
            x1_react: React::new(options.react, Rc::clone(&ctx)),
            x2_es2021: ES2021::new(options.es2021, Rc::clone(&ctx)),
            x2_es2020: ES2020::new(options.es2020, Rc::clone(&ctx)),
            x2_es2019: ES2019::new(options.es2019, Rc::clone(&ctx)),
            x2_es2018: ES2018::new(options.es2018, Rc::clone(&ctx)),
            x2_es2016: ES2016::new(options.es2016, Rc::clone(&ctx)),
            x3_es2015: ES2015::new(options.es2015, ctx),
        }
    }

    pub fn build_with_symbols_and_scopes(
        mut self,
        symbols: SymbolTable,
        scopes: ScopeTree,
        program: &mut Program<'a>,
    ) -> TransformerReturn {
        let allocator = self.ctx.ast.allocator;
        let (symbols, scopes) = traverse_mut(&mut self, allocator, program, symbols, scopes);
        TransformerReturn { errors: self.ctx.take_errors(), symbols, scopes }
    }
}

impl<'a> Traverse<'a> for Transformer<'a> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_program(program, ctx);
        self.x1_react.transform_program(program, ctx);
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x1_react.transform_program_on_exit(program, ctx);
        self.x0_typescript.transform_program_on_exit(program, ctx);
        self.x3_es2015.exit_program(program, ctx);
    }

    // ALPHASORT

    fn enter_arrow_function_expression(
        &mut self,
        expr: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_arrow_expression(expr);
    }

    fn enter_binding_pattern(&mut self, pat: &mut BindingPattern<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_binding_pattern(pat);
    }

    fn enter_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_call_expression(expr);
        self.x1_react.transform_call_expression(expr, ctx);
    }

    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_class(class);
        self.x3_es2015.enter_class(class, ctx);
    }

    fn exit_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.exit_class(class, ctx);
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_class_body(body);
    }
    fn enter_ts_module_declaration(
        &mut self,
        decl: &mut TSModuleDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_ts_module_declaration(decl);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_expression(expr);
        self.x1_react.transform_expression(expr, ctx);
        self.x2_es2021.enter_expression(expr, ctx);
        self.x2_es2020.enter_expression(expr, ctx);
        self.x2_es2018.enter_expression(expr, ctx);
        self.x2_es2016.enter_expression(expr, ctx);
        self.x3_es2015.enter_expression(expr, ctx);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x1_react.transform_expression_on_exit(expr, ctx);
        self.x3_es2015.exit_expression(expr, ctx);
    }

    fn enter_simple_assignment_target(
        &mut self,
        node: &mut SimpleAssignmentTarget<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_simple_assignment_target(node);
    }

    fn enter_assignment_target(
        &mut self,
        node: &mut AssignmentTarget<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_assignment_target(node);
    }

    fn enter_formal_parameter(
        &mut self,
        param: &mut FormalParameter<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_formal_parameter(param);
    }

    fn enter_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_function(func);
        self.x3_es2015.enter_function(func, ctx);
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_function(func);
        self.x1_react.transform_function_on_exit(func, ctx);
        self.x3_es2015.exit_function(func, ctx);
    }

    fn enter_jsx_element(&mut self, node: &mut JSXElement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_jsx_element(node);
    }

    fn enter_jsx_fragment(&mut self, node: &mut JSXFragment<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_jsx_fragment(node);
    }

    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_jsx_opening_element(elem);
        self.x1_react.transform_jsx_opening_element(elem, ctx);
    }

    fn enter_jsx_element_name(&mut self, elem: &mut JSXElementName<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.enter_jsx_element_name(elem, ctx);
    }

    fn enter_method_definition(
        &mut self,
        def: &mut MethodDefinition<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_method_definition(def);
    }

    fn exit_method_definition(
        &mut self,
        def: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_method_definition_on_exit(def, ctx);
    }

    fn enter_new_expression(&mut self, expr: &mut NewExpression<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_new_expression(expr);
    }

    fn enter_property_definition(
        &mut self,
        def: &mut PropertyDefinition<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_property_definition(def);
    }

    fn enter_accessor_property(
        &mut self,
        node: &mut AccessorProperty<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_accessor_property(node);
    }

    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_statements(stmts);
        self.x1_react.transform_statements(stmts, ctx);
        self.x2_es2021.enter_statements(stmts, ctx);
        self.x2_es2020.enter_statements(stmts, ctx);
        self.x2_es2016.enter_statements(stmts, ctx);
    }

    fn exit_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Some plugins may add new statements to the ArrowFunctionExpression's body,
        // which can cause issues with the `() => x;` case, as it only allows a single statement.
        // To address this, we wrap the last statement in a return statement and set the expression to false.
        // This transforms the arrow function into the form `() => { return x; };`.
        if arrow.expression && arrow.body.statements.len() > 1 {
            let Statement::ExpressionStatement(statement) = arrow.body.statements.pop().unwrap()
            else {
                unreachable!("The last statement in an ArrowFunctionExpression should always be an ExpressionStatement.")
            };
            arrow
                .body
                .statements
                .push(ctx.ast.statement_return(SPAN, Some(statement.unbox().expression)));
            arrow.expression = false;
        }
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_statements_on_exit(stmts, ctx);
        self.x1_react.transform_statements_on_exit(stmts, ctx);
        self.x2_es2021.exit_statements(stmts, ctx);
        self.x2_es2020.exit_statements(stmts, ctx);
        self.x2_es2016.exit_statements(stmts, ctx);
    }

    fn enter_tagged_template_expression(
        &mut self,
        expr: &mut TaggedTemplateExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_tagged_template_expression(expr);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_statement(stmt, ctx);
    }

    fn enter_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_declaration(decl, ctx);
        self.x3_es2015.enter_declaration(decl, ctx);
    }

    fn exit_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.exit_declaration(decl, ctx);
    }

    fn enter_if_statement(&mut self, stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_if_statement(stmt, ctx);
    }

    fn enter_while_statement(&mut self, stmt: &mut WhileStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_while_statement(stmt, ctx);
    }

    fn enter_do_while_statement(
        &mut self,
        stmt: &mut DoWhileStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_do_while_statement(stmt, ctx);
    }

    fn enter_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_for_statement(stmt, ctx);
    }

    fn enter_for_of_statement(&mut self, stmt: &mut ForOfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_for_of_statement(stmt, ctx);
    }

    fn enter_for_in_statement(&mut self, stmt: &mut ForInStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_for_in_statement(stmt, ctx);
    }

    fn enter_catch_clause(&mut self, clause: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x2_es2019.enter_catch_clause(clause, ctx);
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.x3_es2015.enter_variable_declarator(node, ctx);
    }

    fn enter_ts_export_assignment(
        &mut self,
        export_assignment: &mut TSExportAssignment<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_ts_export_assignment(export_assignment);
    }
}
