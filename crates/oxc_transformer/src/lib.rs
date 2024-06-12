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
mod react;
mod typescript;

mod helpers {
    pub mod bindings;
    pub mod module_imports;
}

use std::{path::Path, rc::Rc};

use es2015::ES2015;
use oxc_allocator::{Allocator, Vec};
use oxc_ast::{ast::*, AstBuilder, Trivias};
use oxc_diagnostics::Error;
use oxc_span::SourceType;
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};

pub use crate::{
    compiler_assumptions::CompilerAssumptions,
    env::EnvOptions,
    es2015::{ArrowFunctionsOptions, ES2015Options},
    options::BabelOptions,
    options::TransformOptions,
    react::{ReactJsxRuntime, ReactOptions},
    typescript::TypeScriptOptions,
};

use crate::{
    context::{Ctx, TransformCtx},
    react::React,
    typescript::TypeScript,
};

pub struct Transformer<'a> {
    ctx: Ctx<'a>,
    // NOTE: all callbacks must run in order.
    x0_typescript: TypeScript<'a>,
    x1_react: React<'a>,
    x3_es2015: ES2015<'a>,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_path: &Path,
        source_type: SourceType,
        source_text: &'a str,
        trivias: Rc<Trivias>,
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
            x3_es2015: ES2015::new(options.es2015, ctx),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &mut Program<'a>) -> Result<(), std::vec::Vec<Error>> {
        let TransformCtx { ast: AstBuilder { allocator }, source_text, source_type, .. } =
            *self.ctx;
        traverse_mut(&mut self, program, source_text, source_type, allocator);

        let errors = self.ctx.take_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> Traverse<'a> for Transformer<'a> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_program(program, ctx);
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x1_react.transform_program_on_exit(program);
        self.x0_typescript.transform_program_on_exit(program, ctx);
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

    fn enter_class(&mut self, class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_class(class);
        self.x3_es2015.transform_class(class);
    }

    fn exit_class(&mut self, class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.transform_class_on_exit(class);
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_class_body(body);
    }

    fn enter_export_named_declaration(
        &mut self,
        decl: &mut ExportNamedDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_export_named_declaration(decl);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_expression(expr);
        self.x1_react.transform_expression(expr, ctx);
        self.x3_es2015.transform_expression(expr);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.transform_expression_on_exit(expr);
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

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_function(func);
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

    fn enter_jsx_element_name(
        &mut self,
        elem: &mut JSXElementName<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x3_es2015.transform_jsx_element_name(elem);
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
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_method_definition_on_exit(def);
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

    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.enter_statements(stmts);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_statements_on_exit(stmts);
        self.x3_es2015.exit_statements(stmts);
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
        self.x3_es2015.transform_declaration(decl);
    }

    fn exit_declaration(&mut self, decl: &mut Declaration<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x3_es2015.transform_declaration_on_exit(decl);
    }

    fn enter_if_statement(&mut self, stmt: &mut IfStatement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_if_statement(stmt);
    }
    fn enter_while_statement(&mut self, stmt: &mut WhileStatement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_while_statement(stmt);
    }
    fn enter_do_while_statement(
        &mut self,
        stmt: &mut DoWhileStatement<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_do_while_statement(stmt);
    }
    fn enter_for_statement(&mut self, stmt: &mut ForStatement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.x0_typescript.transform_for_statement(stmt);
    }

    fn enter_module_declaration(
        &mut self,
        decl: &mut ModuleDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_module_declaration(decl);
    }
}
