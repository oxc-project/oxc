mod annotations;
mod diagnostics;
mod r#enum;
mod module;
mod namespace;
mod options;
mod rewrite_extensions;

use std::rc::Rc;

use module::TypeScriptModule;
use namespace::TypeScriptNamespace;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use rewrite_extensions::TypeScriptRewriteExtensions;

pub use self::options::{RewriteExtensionsMode, TypeScriptOptions};
use self::{annotations::TypeScriptAnnotations, r#enum::TypeScriptEnum};
use crate::context::Ctx;

/// [Preset TypeScript](https://babeljs.io/docs/babel-preset-typescript)
///
/// This preset includes the following plugins:
///
/// * [transform-typescript](https://babeljs.io/docs/babel-plugin-transform-typescript)
///
/// This plugin adds support for the types syntax used by the TypeScript programming language.
/// However, this plugin does not add the ability to type-check the JavaScript passed to it.
/// For that, you will need to install and set up TypeScript.
///
/// Note that although the TypeScript compiler tsc actively supports certain JavaScript proposals such as optional chaining (?.),
/// nullish coalescing (??) and class properties (this.#x), this preset does not include these features
/// because they are not the types syntax available in TypeScript only.
/// We recommend using preset-env with preset-typescript if you want to transpile these features.
///
/// This plugin is included in `preset-typescript`.
///
/// ## Example
///
/// In:  `const x: number = 0;`
/// Out: `const x = 0;`
#[allow(unused)]
pub struct TypeScript<'a> {
    options: Rc<TypeScriptOptions>,
    ctx: Ctx<'a>,

    annotations: TypeScriptAnnotations<'a>,
    r#enum: TypeScriptEnum<'a>,
    namespace: TypeScriptNamespace<'a>,
    module: TypeScriptModule<'a>,
    rewrite_extensions: TypeScriptRewriteExtensions,
}

impl<'a> TypeScript<'a> {
    pub fn new(options: TypeScriptOptions, ctx: Ctx<'a>) -> Self {
        let options = Rc::new(options.update_with_comments(&ctx));

        Self {
            annotations: TypeScriptAnnotations::new(Rc::clone(&options), Rc::clone(&ctx)),
            r#enum: TypeScriptEnum::new(Rc::clone(&ctx)),
            rewrite_extensions: TypeScriptRewriteExtensions::new(
                options.rewrite_import_extensions.clone().unwrap_or_default(),
            ),
            namespace: TypeScriptNamespace::new(Rc::clone(&options), Rc::clone(&ctx)),
            module: TypeScriptModule::new(Rc::clone(&ctx)),
            options,
            ctx,
        }
    }
}

impl<'a> Traverse<'a> for TypeScript<'a> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.ctx.source_type.is_typescript_definition() {
            // Output empty file for TS definitions
            program.directives.clear();
            program.hashbang = None;
            program.body.clear();
        } else {
            self.namespace.enter_program(program, ctx);
        }
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.exit_program(program, ctx);
    }

    fn enter_arrow_function_expression(
        &mut self,
        expr: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_arrow_function_expression(expr, ctx);
    }

    fn enter_binding_pattern(&mut self, pat: &mut BindingPattern<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_binding_pattern(pat, ctx);
    }

    fn enter_call_expression(&mut self, expr: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_call_expression(expr, ctx);
    }

    fn enter_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_class(class, ctx);
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_class_body(body, ctx);
    }

    fn enter_ts_module_declaration(
        &mut self,
        decl: &mut TSModuleDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_ts_module_declaration(decl, ctx);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_expression(expr, ctx);
    }

    fn enter_simple_assignment_target(
        &mut self,
        target: &mut SimpleAssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_simple_assignment_target(target, ctx);
    }

    fn enter_assignment_target(
        &mut self,
        target: &mut AssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_assignment_target(target, ctx);
    }

    fn enter_formal_parameter(
        &mut self,
        param: &mut FormalParameter<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_formal_parameter(param, ctx);
    }

    fn exit_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.exit_function(func, ctx);
    }

    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_jsx_opening_element(elem, ctx);
    }

    fn enter_method_definition(
        &mut self,
        def: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_method_definition(def, ctx);
    }

    fn exit_method_definition(
        &mut self,
        def: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.exit_method_definition(def, ctx);
    }

    fn enter_new_expression(&mut self, expr: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_new_expression(expr, ctx);
    }

    fn enter_property_definition(
        &mut self,
        def: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_property_definition(def, ctx);
    }

    fn enter_accessor_property(
        &mut self,
        def: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_accessor_property(def, ctx);
    }

    fn enter_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_statements(stmts, ctx);
    }

    fn exit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.exit_statements(stmts, ctx);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.r#enum.enter_statement(stmt, ctx);
    }

    fn enter_if_statement(&mut self, stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_if_statement(stmt, ctx);
    }

    fn enter_while_statement(&mut self, stmt: &mut WhileStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_while_statement(stmt, ctx);
    }

    fn enter_do_while_statement(
        &mut self,
        stmt: &mut DoWhileStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_do_while_statement(stmt, ctx);
    }

    fn enter_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_for_statement(stmt, ctx);
    }

    fn enter_for_in_statement(&mut self, stmt: &mut ForInStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_for_in_statement(stmt, ctx);
    }

    fn enter_for_of_statement(&mut self, stmt: &mut ForOfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_for_of_statement(stmt, ctx);
    }

    fn enter_tagged_template_expression(
        &mut self,
        expr: &mut TaggedTemplateExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.enter_tagged_template_expression(expr, ctx);
    }

    fn enter_jsx_element(&mut self, elem: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_jsx_element(elem, ctx);
    }

    fn enter_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a>) {
        self.annotations.enter_jsx_fragment(elem, ctx);
    }

    fn enter_declaration(&mut self, node: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        self.module.enter_declaration(node, ctx);
    }

    fn enter_import_declaration(
        &mut self,
        node: &mut ImportDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.rewrite_import_extensions.is_some() {
            self.rewrite_extensions.enter_import_declaration(node, ctx);
        }
    }

    fn enter_export_all_declaration(
        &mut self,
        node: &mut ExportAllDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.rewrite_import_extensions.is_some() {
            self.rewrite_extensions.enter_export_all_declaration(node, ctx);
        }
    }

    fn enter_export_named_declaration(
        &mut self,
        node: &mut ExportNamedDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.rewrite_import_extensions.is_some() {
            self.rewrite_extensions.enter_export_named_declaration(node, ctx);
        }
    }

    fn enter_ts_export_assignment(
        &mut self,
        node: &mut TSExportAssignment<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.module.enter_ts_export_assignment(node, ctx);
    }
}
