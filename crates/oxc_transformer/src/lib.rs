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
use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
    Trivias,
};
use oxc_diagnostics::Error;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;

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
        self.visit_program(program);
        let errors = self.ctx.take_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> VisitMut<'a> for Transformer<'a> {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        self.x0_typescript.transform_program(program);

        walk_mut::walk_program_mut(self, program);

        self.x1_react.transform_program_on_exit(program);
        self.x0_typescript.transform_program_on_exit(program);
    }

    // ALPHASORT

    fn visit_arrow_expression(&mut self, expr: &mut ArrowFunctionExpression<'a>) {
        self.x0_typescript.transform_arrow_expression(expr);

        walk_mut::walk_arrow_expression_mut(self, expr);
    }

    fn visit_binding_pattern(&mut self, pat: &mut BindingPattern<'a>) {
        self.x0_typescript.transform_binding_pattern(pat);

        walk_mut::walk_binding_pattern_mut(self, pat);
    }

    fn visit_call_expression(&mut self, expr: &mut CallExpression<'a>) {
        self.x0_typescript.transform_call_expression(expr);

        walk_mut::walk_call_expression_mut(self, expr);
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        self.x0_typescript.transform_class(class);
        self.x3_es2015.transform_class(class);

        walk_mut::walk_class_mut(self, class);

        self.x3_es2015.transform_class_on_exit(class);
    }

    fn visit_class_body(&mut self, body: &mut ClassBody<'a>) {
        self.x0_typescript.transform_class_body(body);

        walk_mut::walk_class_body_mut(self, body);
    }

    fn visit_export_default_declaration(&mut self, decl: &mut ExportDefaultDeclaration<'a>) {
        self.x1_react.transform_export_default_declaration(decl);

        walk_mut::walk_export_default_declaration_mut(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        self.x0_typescript.transform_export_named_declaration(decl);

        walk_mut::walk_export_named_declaration_mut(self, decl);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.x0_typescript.transform_expression(expr);
        self.x1_react.transform_expression(expr);
        self.x3_es2015.transform_expression(expr);

        walk_mut::walk_expression_mut(self, expr);

        self.x3_es2015.transform_expression_on_exit(expr);
    }

    fn visit_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        self.x0_typescript.transform_formal_parameter(param);

        walk_mut::walk_formal_parameter_mut(self, param);
    }

    fn visit_function(&mut self, func: &mut Function<'a>, flags: Option<ScopeFlags>) {
        self.x0_typescript.transform_function(func, flags);

        walk_mut::walk_function_mut(self, func, flags);
    }

    fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        walk_mut::walk_import_declaration_mut(self, decl);
    }

    fn visit_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        self.x0_typescript.transform_jsx_opening_element(elem);
        self.x1_react.transform_jsx_opening_element(elem);
        self.x3_es2015.transform_jsx_opening_element(elem);
        walk_mut::walk_jsx_opening_element_mut(self, elem);
    }

    fn visit_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        self.x0_typescript.transform_method_definition(def);

        walk_mut::walk_method_definition_mut(self, def);

        self.x0_typescript.transform_method_definition_on_exit(def);
    }

    fn visit_new_expression(&mut self, expr: &mut NewExpression<'a>) {
        self.x0_typescript.transform_new_expression(expr);

        walk_mut::walk_new_expression_mut(self, expr);
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        self.x1_react.transform_object_property(prop);

        walk_mut::walk_object_property_mut(self, prop);
    }

    fn visit_property_definition(&mut self, def: &mut PropertyDefinition<'a>) {
        self.x0_typescript.transform_property_definition(def);

        walk_mut::walk_property_definition_mut(self, def);
    }

    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        walk_mut::walk_statements_mut(self, stmts);

        self.x0_typescript.transform_statements_on_exit(stmts);
        self.x3_es2015.transform_statements_on_exit(stmts);
    }

    fn visit_tagged_template_expression(&mut self, expr: &mut TaggedTemplateExpression<'a>) {
        self.x0_typescript.transform_tagged_template_expression(expr);

        walk_mut::walk_tagged_template_expression_mut(self, expr);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        self.x1_react.transform_variable_declarator(declarator);

        walk_mut::walk_variable_declarator_mut(self, declarator);
    }

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        self.x0_typescript.transform_identifier_reference(ident);
        walk_mut::walk_identifier_reference_mut(self, ident);
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.x0_typescript.transform_statement(stmt);
        walk_mut::walk_statement_mut(self, stmt);
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
        self.x0_typescript.transform_declaration(decl);
        self.x3_es2015.transform_declaration(decl);

        walk_mut::walk_declaration_mut(self, decl);

        self.x3_es2015.transform_declaration_on_exit(decl);
    }

    fn visit_if_statement(&mut self, stmt: &mut IfStatement<'a>) {
        self.x0_typescript.transform_if_statement(stmt);
        walk_mut::walk_if_statement_mut(self, stmt);
    }

    fn visit_module_declaration(&mut self, decl: &mut ModuleDeclaration<'a>) {
        self.x0_typescript.transform_module_declaration(decl);
        walk_mut::walk_module_declaration_mut(self, decl);
    }
}
