#![allow(clippy::unused_self)]

use std::rc::Rc;

use crate::context::Ctx;
use crate::TypeScriptOptions;

use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_syntax::operator::AssignmentOperator;
use rustc_hash::FxHashSet;

pub struct TypeScriptAnnotations<'a> {
    #[allow(dead_code)]
    options: Rc<TypeScriptOptions>,
    ctx: Ctx<'a>,

    global_types: FxHashSet<String>,
}

impl<'a> TypeScriptAnnotations<'a> {
    pub fn new(options: &Rc<TypeScriptOptions>, ctx: &Ctx<'a>) -> Self {
        Self {
            options: Rc::clone(options),
            ctx: Rc::clone(ctx),
            global_types: FxHashSet::default(),
        }
    }

    pub fn is_global_type(&self) -> bool {
        self.global_types.contains("TODO")
    }

    // Convert `export = expr` into `module.exports = expr`
    fn create_module_exports(&self, exp: &TSExportAssignment<'a>) -> Statement<'a> {
        let ast = &self.ctx.ast;

        ast.expression_statement(
            SPAN,
            ast.assignment_expression(
                SPAN,
                AssignmentOperator::Assign,
                ast.simple_assignment_target_member_expression(ast.static_member(
                    SPAN,
                    ast.identifier_reference_expression(ast.identifier_reference(SPAN, "module")),
                    ast.identifier_name(SPAN, "exports"),
                    false,
                )),
                ast.copy(&exp.expression),
            ),
        )
    }

    // Creates `this.name = name`
    fn create_this_property_assignment(&self, name: &Atom<'a>) -> Statement<'a> {
        let ast = &self.ctx.ast;

        ast.expression_statement(
            SPAN,
            ast.assignment_expression(
                SPAN,
                AssignmentOperator::Assign,
                ast.simple_assignment_target_member_expression(ast.static_member(
                    SPAN,
                    ast.this_expression(SPAN),
                    ast.identifier_name(SPAN, name),
                    false,
                )),
                ast.identifier_reference_expression(ast.identifier_reference(SPAN, name)),
            ),
        )
    }

    // Remove type only imports/exports
    pub fn transform_program_on_exit(&self, program: &mut Program<'a>) {
        let mut module_count = 0;

        let body =
            self.ctx.ast.move_statement_vec(&mut program.body).into_iter().filter_map(|stmt| {
                // If an import/export declaration, remove all that are type-only
                if let Statement::ModuleDeclaration(decl) = &stmt {
                    let keep = match &**decl {
                        ModuleDeclaration::ImportDeclaration(inner) => !inner.import_kind.is_type(),
                        ModuleDeclaration::ExportAllDeclaration(inner) => {
                            !inner.is_typescript_syntax()
                        }
                        ModuleDeclaration::ExportNamedDeclaration(inner) => {
                            !(inner.is_typescript_syntax()
                                || inner.specifiers.is_empty()
                                || inner.specifiers.iter().all(|spec| spec.export_kind.is_type())
                                || self.is_global_type())
                        }
                        ModuleDeclaration::ExportDefaultDeclaration(inner) => {
                            !inner.is_typescript_syntax()
                        }
                        ModuleDeclaration::TSNamespaceExportDeclaration(_) => false,

                        // Replace with `module.exports = expr`
                        ModuleDeclaration::TSExportAssignment(exp) => {
                            return Some(self.create_module_exports(exp));
                        }
                    };

                    if keep {
                        module_count += 1;
                    } else {
                        return None;
                    }
                }

                Some(stmt)
            });

        program.body = self.ctx.ast.new_vec_from_iter(body);

        // Determine if we still have import/export statements, otherwise we
        // need to inject an empty statement (`export {}`) so that the file is
        // still considered a module
        if module_count == 0 && self.ctx.semantic.source_type().is_module() {
            // FIXME
            // program.body.push(self.ctx.ast.module_declaration(
            // ModuleDeclaration::ExportNamedDeclaration(
            // self.ctx.ast.plain_export_named_declaration(SPAN, self.ctx.ast.new_vec(), None),
            // ),
            // ));
        }
    }

    pub fn transform_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        // Remove type only specifiers
        decl.specifiers.retain(|spec| !spec.export_kind.is_type());
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        *expr = self.ctx.ast.copy(expr.get_inner_expression());
    }

    pub fn transform_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        // Collects parameter properties so that we can add an assignment
        // for each of them in the constructor body.
        if def.kind == MethodDefinitionKind::Constructor {
            let mut assigns = self.ctx.ast.new_vec();

            for param in &def.value.params.items {
                if param.pattern.type_annotation.is_none() {
                    continue;
                }

                if let Some(id) = param.pattern.get_identifier() {
                    assigns.push(self.create_this_property_assignment(id));
                }
            }

            if !assigns.is_empty() {
                def.value
                    .body
                    .get_or_insert(self.ctx.ast.function_body(
                        SPAN,
                        self.ctx.ast.new_vec(),
                        self.ctx.ast.new_vec(),
                    ))
                    .statements
                    .extend(assigns);
            }
        }
    }
}
