mod annotations;
mod diagnostics;
mod r#enum;
mod module;
mod namespace;
mod options;

use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::TraverseCtx;

pub use self::options::TypeScriptOptions;
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
}

impl<'a> TypeScript<'a> {
    pub fn new(options: TypeScriptOptions, ctx: Ctx<'a>) -> Self {
        let options = Rc::new(options.update_with_comments(&ctx));

        Self {
            annotations: TypeScriptAnnotations::new(Rc::clone(&options), Rc::clone(&ctx)),
            r#enum: TypeScriptEnum::new(Rc::clone(&ctx)),
            options,
            ctx,
        }
    }
}

// Transforms
impl<'a> TypeScript<'a> {
    pub fn transform_program(&self, program: &mut Program<'a>, ctx: &mut TraverseCtx) {
        if self.ctx.source_type.is_typescript_definition() {
            // Output empty file for TS definitions
            program.directives.clear();
            program.hashbang = None;
            program.body.clear();
        } else {
            self.transform_program_for_namespace(program, ctx);
        }
    }

    pub fn transform_program_on_exit(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.annotations.transform_program_on_exit(program, ctx);
    }

    pub fn transform_arrow_expression(&mut self, expr: &mut ArrowFunctionExpression<'a>) {
        self.annotations.transform_arrow_expression(expr);
    }

    pub fn transform_binding_pattern(&mut self, pat: &mut BindingPattern<'a>) {
        self.annotations.transform_binding_pattern(pat);
    }

    pub fn transform_call_expression(&mut self, expr: &mut CallExpression<'a>) {
        self.annotations.transform_call_expression(expr);
    }

    pub fn transform_class(&mut self, class: &mut Class<'a>) {
        self.annotations.transform_class(class);
    }

    pub fn transform_class_body(&mut self, body: &mut ClassBody<'a>) {
        self.annotations.transform_class_body(body);
    }

    pub fn transform_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        self.annotations.transform_export_named_declaration(decl);
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        self.annotations.transform_expression(expr);
    }

    pub fn transform_simple_assignment_target(&mut self, target: &mut SimpleAssignmentTarget<'a>) {
        self.annotations.transform_simple_assignment_target(target);
    }

    pub fn transform_assignment_target(&mut self, target: &mut AssignmentTarget<'a>) {
        self.annotations.transform_assignment_target(target);
    }

    pub fn transform_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        self.annotations.transform_formal_parameter(param);
    }

    pub fn transform_function(&mut self, func: &mut Function<'a>) {
        self.annotations.transform_function(func);
    }

    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        self.annotations.transform_jsx_opening_element(elem);
    }

    pub fn transform_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        self.annotations.transform_method_definition(def);
    }

    pub fn transform_method_definition_on_exit(&mut self, def: &mut MethodDefinition<'a>) {
        self.annotations.transform_method_definition_on_exit(def);
    }

    pub fn transform_new_expression(&mut self, expr: &mut NewExpression<'a>) {
        self.annotations.transform_new_expression(expr);
    }

    pub fn transform_property_definition(&mut self, def: &mut PropertyDefinition<'a>) {
        self.annotations.transform_property_definition(def);
    }

    pub fn transform_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.annotations.transform_statements(stmts);
    }

    pub fn transform_statements_on_exit(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.annotations.transform_statements_on_exit(stmts);
    }

    pub fn transform_statement(&mut self, stmt: &mut Statement<'a>, ctx: &TraverseCtx<'a>) {
        self.r#enum.transform_statement(stmt, ctx);
    }

    pub fn transform_if_statement(&mut self, stmt: &mut IfStatement<'a>) {
        self.annotations.transform_if_statement(stmt);
    }

    pub fn transform_while_statement(&mut self, stmt: &mut WhileStatement<'a>) {
        self.annotations.transform_while_statement(stmt);
    }

    pub fn transform_do_while_statement(&mut self, stmt: &mut DoWhileStatement<'a>) {
        self.annotations.transform_do_while_statement(stmt);
    }

    pub fn transform_for_statement(&mut self, stmt: &mut ForStatement<'a>) {
        self.annotations.transform_for_statement(stmt);
    }

    pub fn transform_tagged_template_expression(
        &mut self,
        expr: &mut TaggedTemplateExpression<'a>,
    ) {
        self.annotations.transform_tagged_template_expression(expr);
    }

    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        match decl {
            Declaration::TSImportEqualsDeclaration(ts_import_equals)
                if ts_import_equals.import_kind.is_value() =>
            {
                *decl = self.transform_ts_import_equals(ts_import_equals, ctx);
            }
            _ => {}
        }
    }

    pub fn transform_module_declaration(&mut self, module_decl: &mut ModuleDeclaration<'a>) {
        if let ModuleDeclaration::TSExportAssignment(ts_export_assignment) = &mut *module_decl {
            self.transform_ts_export_assignment(ts_export_assignment);
        }
    }

    pub fn transform_jsx_element(&mut self, elem: &mut JSXElement<'a>) {
        self.annotations.transform_jsx_element(elem);
    }

    pub fn transform_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>) {
        self.annotations.transform_jsx_fragment(elem);
    }
}
