#![allow(clippy::unused_self)]

use std::rc::Rc;

use crate::context::Ctx;
use crate::TypeScriptOptions;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_syntax::operator::AssignmentOperator;
use rustc_hash::FxHashSet;

use super::collector::TypeScriptReferenceCollector;

pub struct TypeScriptAnnotations<'a> {
    #[allow(dead_code)]
    options: Rc<TypeScriptOptions>,
    ctx: Ctx<'a>,
    /// Assignments to be added to the constructor body
    assignments: Vec<'a, Statement<'a>>,
    has_super_call: bool,

    has_jsx_element: bool,
    has_jsx_fragment: bool,
    jsx_element_import_name: String,
    jsx_fragment_import_name: String,
}

impl<'a> TypeScriptAnnotations<'a> {
    pub fn new(options: &Rc<TypeScriptOptions>, ctx: &Ctx<'a>) -> Self {
        let jsx_element_import_name = if options.jsx_pragma.contains('.') {
            options.jsx_pragma.split('.').next().map(String::from).unwrap()
        } else {
            options.jsx_pragma.to_string()
        };

        let jsx_fragment_import_name = if options.jsx_pragma_frag.contains('.') {
            options.jsx_pragma_frag.split('.').next().map(String::from).unwrap()
        } else {
            options.jsx_pragma_frag.to_string()
        };

        Self {
            has_super_call: false,
            assignments: ctx.ast.new_vec(),
            options: Rc::clone(options),
            ctx: Rc::clone(ctx),
            has_jsx_element: false,
            has_jsx_fragment: false,
            jsx_element_import_name,
            jsx_fragment_import_name,
        }
    }

    /// Check if the given name is a JSX pragma or fragment pragma import
    /// and if the file contains JSX elements or fragments
    fn is_jsx_imports(&self, name: &str) -> bool {
        self.has_jsx_element && name == self.jsx_element_import_name
            || self.has_jsx_fragment && name == self.jsx_fragment_import_name
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
    pub fn transform_program_on_exit(
        &self,
        program: &mut Program<'a>,
        references: &TypeScriptReferenceCollector,
    ) {
        let mut type_names = FxHashSet::default();
        let mut module_count = 0;
        let mut removed_count = 0;

        program.body.retain_mut(|stmt| {
            // fix namespace/export-type-only/input.ts
            // The namespace is type only. So if its name appear in the ExportNamedDeclaration, we should remove it.
            if let Statement::TSModuleDeclaration(decl) = stmt {
                type_names.insert(decl.id.name().clone());
                return false;
            }

            let Some(module_decl) = stmt.as_module_declaration_mut() else {
                return true;
            };

            let need_delete = match module_decl {
                ModuleDeclaration::ExportNamedDeclaration(decl) => {
                    decl.specifiers.retain(|specifier| {
                        !(specifier.export_kind.is_type()
                            || type_names.contains(specifier.exported.name()))
                    });

                    decl.export_kind.is_type()
                        || ((decl.declaration.is_none()
                            || decl
                                .declaration
                                .as_ref()
                                .is_some_and(Declaration::is_typescript_syntax))
                            && decl.specifiers.is_empty())
                }
                ModuleDeclaration::ImportDeclaration(decl) => {
                    let is_type = decl.import_kind.is_type();

                    let is_specifiers_empty =
                        decl.specifiers.as_ref().is_some_and(|s| s.is_empty());

                    if let Some(specifiers) = &mut decl.specifiers {
                        specifiers.retain(|specifier| match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                if is_type || s.import_kind.is_type() {
                                    type_names.insert(s.local.name.clone());
                                    return false;
                                }

                                if self.options.only_remove_type_imports {
                                    return true;
                                }

                                references.has_reference(&s.local.name)
                                    || self.is_jsx_imports(&s.local.name)
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                if is_type {
                                    type_names.insert(s.local.name.clone());
                                    return false;
                                }

                                if self.options.only_remove_type_imports {
                                    return true;
                                }
                                references.has_reference(&s.local.name)
                                    || self.is_jsx_imports(&s.local.name)
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                if is_type {
                                    type_names.insert(s.local.name.clone());
                                }

                                if self.options.only_remove_type_imports {
                                    return true;
                                }

                                references.has_reference(&s.local.name)
                                    || self.is_jsx_imports(&s.local.name)
                            }
                        });
                    }

                    decl.import_kind.is_type()
                        || (!self.options.only_remove_type_imports
                            && !is_specifiers_empty
                            && decl
                                .specifiers
                                .as_ref()
                                .is_some_and(|specifiers| specifiers.is_empty()))
                }
                _ => false,
            };

            if need_delete {
                removed_count += 1;
            } else {
                module_count += 1;
            }

            !need_delete
        });

        // Determine if we still have import/export statements, otherwise we
        // need to inject an empty statement (`export {}`) so that the file is
        // still considered a module
        if module_count == 0 && removed_count > 0 {
            let export_decl = ModuleDeclaration::ExportNamedDeclaration(
                self.ctx.ast.plain_export_named_declaration(SPAN, self.ctx.ast.new_vec(), None),
            );
            program.body.push(self.ctx.ast.module_declaration(export_decl));
        }
    }

    pub fn transform_arrow_expression(&mut self, expr: &mut ArrowFunctionExpression<'a>) {
        expr.type_parameters = None;
        expr.return_type = None;
    }

    pub fn transform_binding_pattern(&mut self, pat: &mut BindingPattern<'a>) {
        pat.type_annotation = None;

        if pat.kind.is_binding_identifier() {
            pat.optional = false;
        }
    }

    pub fn transform_call_expression(&mut self, expr: &mut CallExpression<'a>) {
        expr.type_parameters = None;
    }

    pub fn transform_class(&mut self, class: &mut Class<'a>) {
        class.type_parameters = None;
        class.super_type_parameters = None;
        class.implements = None;
    }

    pub fn transform_class_body(&mut self, body: &mut ClassBody<'a>) {
        // Remove type only members
        body.body.retain(|elem| match elem {
            ClassElement::MethodDefinition(method) => {
                matches!(method.r#type, MethodDefinitionType::MethodDefinition)
                    || !method.value.is_typescript_syntax()
            }
            ClassElement::PropertyDefinition(prop) => {
                if prop.value.as_ref().is_some_and(Expression::is_typescript_syntax)
                    || prop.declare && prop.decorators.is_empty()
                {
                    false
                } else {
                    matches!(prop.r#type, PropertyDefinitionType::PropertyDefinition)
                }
            }
            ClassElement::TSIndexSignature(_) => false,
            _ => true,
        });
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        *expr = self.ctx.ast.copy(expr.get_inner_expression());
    }

    pub fn transform_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        param.accessibility = None;
    }

    pub fn transform_function(&mut self, func: &mut Function<'a>) {
        func.this_param = None;
        func.type_parameters = None;
        func.return_type = None;
    }

    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        elem.type_parameters = None;
    }

    pub fn transform_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        // Collects parameter properties so that we can add an assignment
        // for each of them in the constructor body.
        if def.kind == MethodDefinitionKind::Constructor {
            for param in &def.value.params.items {
                if !param.is_public() {
                    continue;
                }

                if let Some(id) = param.pattern.get_identifier() {
                    let assignment = self.create_this_property_assignment(id);
                    self.assignments.push(assignment);
                }
            }
        }

        def.accessibility = None;
        def.optional = false;
        def.r#override = false;
    }

    pub fn transform_method_definition_on_exit(&mut self, def: &mut MethodDefinition<'a>) {
        if def.kind == MethodDefinitionKind::Constructor && !self.assignments.is_empty() {
            // When the constructor doesn't have a super call,
            // we simply add assignments to the bottom of the function body
            if self.has_super_call {
                self.assignments.clear();
            } else {
                def.value
                    .body
                    .get_or_insert_with(|| {
                        self.ctx.ast.function_body(
                            SPAN,
                            self.ctx.ast.new_vec(),
                            self.ctx.ast.new_vec(),
                        )
                    })
                    .statements
                    .extend(self.assignments.drain(..));
            }
        }
    }

    pub fn transform_new_expression(&mut self, expr: &mut NewExpression<'a>) {
        expr.type_parameters = None;
    }

    pub fn transform_property_definition(&mut self, def: &mut PropertyDefinition<'a>) {
        assert!(
            !(def.declare && def.value.is_some()),
            "Fields with the 'declare' modifier cannot be initialized here, but only in the constructor"
        );

        assert!(
            !(def.definite && def.value.is_some()),
            "Definitely assigned fields cannot be initialized here, but only in the constructor"
        );

        def.accessibility = None;
        def.declare = false;
        def.definite = false;
        def.r#override = false;
        def.optional = false;
        def.readonly = false;
        def.type_annotation = None;
    }

    pub fn transform_statements_on_exit(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        // Remove TS specific statements
        stmts.retain(|stmt| match stmt {
            Statement::ExpressionStatement(s) => !s.expression.is_typescript_syntax(),
            Statement::TSModuleDeclaration(_) => true,
            match_declaration!(Statement) => !stmt.to_declaration().is_typescript_syntax(),
            // Ignore ModuleDeclaration as it's handled in the program
            _ => true,
        });

        // Add assignments after super calls
        if !self.assignments.is_empty() {
            let has_super_call = stmts.iter().any(|stmt| {
                matches!(stmt, Statement::ExpressionStatement(stmt) if stmt.expression.is_super_call_expression())
            });
            if has_super_call {
                let mut new_stmts = self.ctx.ast.new_vec();
                for stmt in stmts.drain(..) {
                    let is_super_call = matches!(stmt, Statement::ExpressionStatement(ref stmt) if stmt.expression.is_super_call_expression());
                    new_stmts.push(stmt);
                    if is_super_call {
                        new_stmts.extend(self.ctx.ast.copy(&self.assignments));
                    }
                }
                self.has_super_call = true;
                *stmts = new_stmts;
            }
        }
    }

    /// Transform if statement's consequent and alternate to block statements if they are super calls
    /// ```ts
    /// if (true) super() else super();
    /// // to
    /// if (true) { super() } else { super() }
    /// ```
    pub fn transform_if_statement(&mut self, stmt: &mut IfStatement<'a>) {
        if !self.assignments.is_empty() {
            if matches!(&stmt.consequent, Statement::ExpressionStatement(expr) if expr.expression.is_super_call_expression())
            {
                stmt.consequent =
                    self.ctx.ast.block_statement(self.ctx.ast.block(
                        SPAN,
                        self.ctx.ast.new_vec_single(self.ctx.ast.copy(&stmt.consequent)),
                    ));
            }
            if let Some(alternate) = &stmt.alternate {
                if matches!(alternate, Statement::ExpressionStatement(expr) if expr.expression.is_super_call_expression())
                {
                    stmt.alternate =
                        Some(self.ctx.ast.block_statement(self.ctx.ast.block(
                            SPAN,
                            self.ctx.ast.new_vec_single(self.ctx.ast.copy(alternate)),
                        )));
                }
            }
        }

        if stmt.consequent.is_typescript_syntax() {
            stmt.consequent =
                self.ctx.ast.block_statement(self.ctx.ast.block(SPAN, self.ctx.ast.new_vec()));
        }

        if stmt.alternate.as_ref().is_some_and(Statement::is_typescript_syntax) {
            stmt.alternate = None;
        }
    }

    pub fn transform_for_statement(&mut self, stmt: &mut ForStatement<'a>) {
        if stmt.body.is_typescript_syntax() {
            stmt.body =
                self.ctx.ast.block_statement(self.ctx.ast.block(SPAN, self.ctx.ast.new_vec()));
        }
    }

    pub fn transform_while_statement(&mut self, stmt: &mut WhileStatement<'a>) {
        if stmt.body.is_typescript_syntax() {
            stmt.body =
                self.ctx.ast.block_statement(self.ctx.ast.block(SPAN, self.ctx.ast.new_vec()));
        }
    }

    pub fn transform_do_while_statement(&mut self, stmt: &mut DoWhileStatement<'a>) {
        if stmt.body.is_typescript_syntax() {
            stmt.body =
                self.ctx.ast.block_statement(self.ctx.ast.block(SPAN, self.ctx.ast.new_vec()));
        }
    }

    pub fn transform_tagged_template_expression(
        &mut self,
        expr: &mut TaggedTemplateExpression<'a>,
    ) {
        expr.type_parameters = None;
    }

    pub fn transform_jsx_element(&mut self, _elem: &mut JSXElement<'a>) {
        self.has_jsx_element = true;
    }

    pub fn transform_jsx_fragment(&mut self, _elem: &mut JSXFragment<'a>) {
        self.has_jsx_fragment = true;
    }
}
