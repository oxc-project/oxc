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
mod es2015;
mod react;
mod typescript;

mod helpers {
    pub mod module_imports;
}

use std::{path::Path, rc::Rc};

use es2015::ES2015;
use oxc_allocator::{Allocator, Vec};
use oxc_ast::{ast::*, Trivias};
use oxc_diagnostics::Error;
use oxc_span::SourceType;
// use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};

pub use crate::{
    compiler_assumptions::CompilerAssumptions, es2015::ES2015Options, options::TransformOptions,
    react::ReactOptions, typescript::TypeScriptOptions,
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
        trivias: &'a Trivias,
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
            x0_typescript: TypeScript::new(options.typescript, &ctx),
            x1_react: React::new(options.react, &ctx),
            x3_es2015: ES2015::new(options.es2015, &ctx),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &mut Program<'a>) -> Result<(), std::vec::Vec<Error>> {
        let allocator = self.ctx.ast.allocator;
        traverse_mut(&mut self, program, allocator);

        let errors = self.ctx.take_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> Traverse<'a> for Transformer<'a> {
    fn enter_program(&mut self, program: &mut Program<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_program(program);
    }

    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &TraverseCtx<'a>) {
        self.x1_react.transform_program_on_exit(program);
        self.x0_typescript.transform_program_on_exit(program);
    }

    // ALPHASORT

    fn enter_arrow_function_expression(
        &mut self,
        expr: &mut ArrowFunctionExpression<'a>,
        _ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_arrow_expression(expr);
    }

    fn enter_binding_pattern(&mut self, pat: &mut BindingPattern<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_binding_pattern(pat);
    }

    fn enter_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_call_expression(expr);
        self.x1_react.transform_call_expression(expr, ctx);
    }

    fn enter_class(&mut self, class: &mut Class<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_class(class);
        self.x3_es2015.transform_class(class);
    }

    fn exit_class(&mut self, class: &mut Class<'a>, _ctx: &TraverseCtx<'a>) {
        self.x3_es2015.transform_class_on_exit(class);
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_class_body(body);
    }

    fn enter_export_named_declaration(
        &mut self,
        decl: &mut ExportNamedDeclaration<'a>,
        _ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_export_named_declaration(decl);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_expression(expr);
        self.x1_react.transform_expression(expr);
        self.x3_es2015.transform_expression(expr);
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &TraverseCtx<'a>) {
        self.x3_es2015.transform_expression_on_exit(expr);
    }

    fn enter_formal_parameter(&mut self, param: &mut FormalParameter<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_formal_parameter(param);
    }

    fn enter_function(&mut self, func: &mut Function<'a>, _ctx: &TraverseCtx<'a>) {
        // TODO: Scope flags
        // Was a function param: flags: Option<ScopeFlags>,
        let flags = None;
        self.x0_typescript.transform_function(func, flags);
    }

    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        _ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_jsx_opening_element(elem);
        self.x1_react.transform_jsx_opening_element(elem);
        self.x3_es2015.transform_jsx_opening_element(elem);
    }

    fn enter_method_definition(&mut self, def: &mut MethodDefinition<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_method_definition(def);
    }

    fn exit_method_definition(&mut self, def: &mut MethodDefinition<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_method_definition_on_exit(def);
    }

    fn enter_new_expression(&mut self, expr: &mut NewExpression<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_new_expression(expr);
    }

    fn enter_property_definition(
        &mut self,
        def: &mut PropertyDefinition<'a>,
        _ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_property_definition(def);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_statements_on_exit(stmts);
        self.x3_es2015.transform_statements_on_exit(stmts);
    }

    fn enter_tagged_template_expression(
        &mut self,
        expr: &mut TaggedTemplateExpression<'a>,
        _ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_tagged_template_expression(expr);
    }

    fn enter_identifier_reference(
        &mut self,
        ident: &mut IdentifierReference<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_identifier_reference(ident, ctx);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_statement(stmt);
    }

    fn enter_declaration(&mut self, decl: &mut Declaration<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_declaration(decl);
        self.x3_es2015.transform_declaration(decl);
    }

    fn exit_declaration(&mut self, decl: &mut Declaration<'a>, _ctx: &TraverseCtx<'a>) {
        self.x3_es2015.transform_declaration_on_exit(decl);
    }

    fn enter_if_statement(&mut self, stmt: &mut IfStatement<'a>, _ctx: &TraverseCtx<'a>) {
        self.x0_typescript.transform_if_statement(stmt);
    }

    fn enter_module_declaration(
        &mut self,
        decl: &mut ModuleDeclaration<'a>,
        _ctx: &TraverseCtx<'a>,
    ) {
        self.x0_typescript.transform_module_declaration(decl);
    }
}
