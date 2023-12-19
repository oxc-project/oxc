use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator},
    NumberBase,
};
use rustc_hash::FxHashSet;
use std::{mem, rc::Rc};

use crate::{context::TransformerCtx, utils::is_valid_identifier};

/// Transform TypeScript
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-typescript>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-typescript>
/// * <https://www.typescriptlang.org/tsconfig#verbatimModuleSyntax>
pub struct TypeScript<'a> {
    ast: Rc<AstBuilder<'a>>,
    ctx: TransformerCtx<'a>,
    verbatim_module_syntax: bool,

    export_name_set: FxHashSet<Atom>,
}

impl<'a> TypeScript<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        verbatim_module_syntax: bool,
    ) -> Self {
        Self { ast, ctx, verbatim_module_syntax, export_name_set: FxHashSet::default() }
    }

    /// ```TypeScript
    /// enum Foo {
    ///   X
    /// }
    /// ```
    /// ```JavaScript
    /// var Foo = ((Foo) => {
    ///   const X = 0; Foo[Foo["X"] = X] = "X";
    ///   return Foo;
    /// })(Foo || {});
    /// ```
    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>) {
        let Declaration::TSEnumDeclaration(ts_enum_declaration) = decl else {
            return;
        };

        if ts_enum_declaration.modifiers.contains(ModifierKind::Declare) {
            return;
        }

        let span = ts_enum_declaration.span;
        let ident = ts_enum_declaration.id.clone();
        let kind = self.ast.binding_pattern_identifier(ident);
        let id = self.ast.binding_pattern(kind, None, false);

        let mut params = self.ast.new_vec();

        // ((Foo) => {
        params.push(self.ast.formal_parameter(SPAN, id, None, false, self.ast.new_vec()));

        let params = self.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            None,
        );

        // Foo[Foo["X"] = 0] = "X";
        let enum_name = ts_enum_declaration.id.name.clone();
        let statements =
            self.transform_ts_enum_members(&mut ts_enum_declaration.body.members, &enum_name);
        let body =
            self.ast.function_body(ts_enum_declaration.body.span, self.ast.new_vec(), statements);

        let callee = self.ast.arrow_expression(SPAN, false, false, false, params, body, None, None);

        // })(Foo || {});
        let mut arguments = self.ast.new_vec();
        let op = LogicalOperator::Or;
        let left = self
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
        let right = self.ast.object_expression(SPAN, self.ast.new_vec(), None);
        let expression = self.ast.logical_expression(SPAN, left, op, right);
        arguments.push(Argument::Expression(expression));

        let call_expression = self.ast.call_expression(SPAN, callee, arguments, false, None);

        let kind = VariableDeclarationKind::Var;
        let decls = {
            let mut decls = self.ast.new_vec();

            let binding_identifier = BindingIdentifier::new(SPAN, enum_name.clone());
            let binding_pattern_kind = self.ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ast.binding_pattern(binding_pattern_kind, None, false);
            let decl =
                self.ast.variable_declarator(SPAN, kind, binding, Some(call_expression), false);

            decls.push(decl);
            decls
        };
        let variable_declaration =
            self.ast.variable_declaration(span, kind, decls, Modifiers::empty());

        *decl = Declaration::VariableDeclaration(variable_declaration);
    }

    /// Remove `export` from merged declaration.
    /// We only preserve the first one.
    /// for example:
    /// ```TypeScript
    /// export enum Foo {}
    /// export enum Foo {}
    /// ```
    /// ```JavaScript
    /// export enum Foo {}
    /// enum Foo {}
    /// ```
    pub fn transform_statement(&mut self, stmt: &mut Statement<'a>) {
        let Statement::ModuleDeclaration(module_decl) = stmt else {
            return;
        };

        let ModuleDeclaration::ExportNamedDeclaration(export_decl) = &mut **module_decl else {
            return;
        };

        let ExportNamedDeclaration {
            declaration: Some(declaration),
            source: None,
            export_kind: ImportOrExportKind::Value,
            ..
        } = &mut **export_decl
        else {
            return;
        };

        let id = match &declaration {
            Declaration::TSEnumDeclaration(decl) => decl.id.name.clone(),
            Declaration::TSModuleDeclaration(decl) => {
                let TSModuleDeclarationName::Identifier(id) = &decl.id else {
                    return;
                };

                id.name.clone()
            }
            _ => return,
        };

        if self.export_name_set.insert(id) {
            return;
        }

        *stmt = Statement::Declaration(self.ast.move_declaration(declaration));
    }

    /// * Remove the top level import / export statements that are types
    /// * Adds `export {}` if all import / export statements are removed, this is used to tell
    /// downstream tools that this file is in ESM.
    pub fn transform_program(&self, program: &mut Program<'a>) {
        let mut needs_explicit_esm = false;

        for stmt in program.body.iter_mut() {
            if let Statement::ModuleDeclaration(module_decl) = stmt {
                needs_explicit_esm = true;
                match &mut **module_decl {
                    ModuleDeclaration::ExportNamedDeclaration(decl) => {
                        decl.specifiers.retain(|specifier| specifier.export_kind.is_value());
                    }
                    ModuleDeclaration::ImportDeclaration(decl) if decl.import_kind.is_value() => {
                        if let Some(specifiers) = &mut decl.specifiers {
                            specifiers.retain(|specifier| match specifier {
                                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                    if s.import_kind.is_type() {
                                        return false;
                                    }

                                    if self.verbatim_module_syntax {
                                        return true;
                                    }

                                    self.has_value_references(&s.local.name)
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(s)
                                    if !self.verbatim_module_syntax =>
                                {
                                    self.has_value_references(&s.local.name)
                                }
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s)
                                    if !self.verbatim_module_syntax =>
                                {
                                    self.has_value_references(&s.local.name)
                                }
                                _ => true,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        program.body.retain(|stmt| match stmt {
            Statement::ModuleDeclaration(module_decl) => match &**module_decl {
                ModuleDeclaration::ImportDeclaration(decl) => {
                    if decl.import_kind.is_type() {
                        return false;
                    }

                    if self.verbatim_module_syntax {
                        return true;
                    }

                    !decl.specifiers.as_ref().is_some_and(|specifiers| specifiers.is_empty())
                }
                ModuleDeclaration::ExportNamedDeclaration(decl) => {
                    if decl.export_kind.is_type() {
                        return false;
                    }

                    if self.verbatim_module_syntax {
                        return true;
                    }

                    if decl.declaration.is_none() && decl.specifiers.is_empty() {
                        return false;
                    }

                    true
                }
                _ => true,
            },
            _ => true,
        });

        if needs_explicit_esm
            && !program.body.iter().any(|s| matches!(s, Statement::ModuleDeclaration(_)))
        {
            let empty_export = self.ast.export_named_declaration(
                SPAN,
                None,
                self.ast.new_vec(),
                None,
                ImportOrExportKind::Value,
            );
            let export_decl = ModuleDeclaration::ExportNamedDeclaration(empty_export);
            program.body.push(self.ast.module_declaration(export_decl));
        }
    }

    fn has_value_references(&self, name: &Atom) -> bool {
        let root_scope_id = self.ctx.scopes().root_scope_id();

        self.ctx
            .scopes()
            .get_binding(root_scope_id, name)
            .map(|symbol_id| {
                self.ctx
                    .symbols()
                    .get_resolved_references(symbol_id)
                    .any(|x| x.is_read() || x.is_write())
            })
            .unwrap_or_default()
    }
}

impl<'a> TypeScript<'a> {
    fn transform_ts_enum_members(
        &self,
        members: &mut Vec<'a, TSEnumMember<'a>>,
        enum_name: &Atom,
    ) -> Vec<'a, Statement<'a>> {
        let mut default_init = self.ast.literal_number_expression(NumberLiteral {
            span: SPAN,
            value: 0.0,
            raw: "0",
            base: NumberBase::Decimal,
        });
        let mut statements = self.ast.new_vec();

        for member in members.iter_mut() {
            let (member_name, member_span) = match &member.id {
                TSEnumMemberName::Identifier(id) => (&id.name, id.span),
                TSEnumMemberName::StringLiteral(str) => (&str.value, str.span),
                TSEnumMemberName::ComputedPropertyName(..)
                | TSEnumMemberName::NumberLiteral(..) => unreachable!(),
            };

            let mut init =
                self.ast.move_expression(member.initializer.as_mut().unwrap_or(&mut default_init));

            let is_str = init.is_string_literal();

            let mut self_ref = {
                let obj = self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    enum_name.clone(),
                ));
                let expr = self
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));
                self.ast.computed_member_expression(SPAN, obj, expr, false)
            };

            if is_valid_identifier(member_name, true) {
                let ident = IdentifierReference::new(member_span, member_name.clone());

                self_ref = self.ast.identifier_reference_expression(ident.clone());
                let init = mem::replace(&mut init, self.ast.identifier_reference_expression(ident));

                let kind = VariableDeclarationKind::Const;
                let decls = {
                    let mut decls = self.ast.new_vec();

                    let binding_identifier = BindingIdentifier::new(SPAN, member_name.clone());
                    let binding_pattern_kind =
                        self.ast.binding_pattern_identifier(binding_identifier);
                    let binding = self.ast.binding_pattern(binding_pattern_kind, None, false);
                    let decl = self.ast.variable_declarator(SPAN, kind, binding, Some(init), false);

                    decls.push(decl);
                    decls
                };
                let decl = self.ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());
                let stmt: Statement<'_> =
                    Statement::Declaration(Declaration::VariableDeclaration(decl));

                statements.push(stmt);
            }

            // Foo["x"] = init
            let member_expr = {
                let obj = self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    enum_name.clone(),
                ));
                let expr = self
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));

                self.ast.computed_member(SPAN, obj, expr, false)
            };
            let left = AssignmentTarget::SimpleAssignmentTarget(
                self.ast.simple_assignment_target_member_expression(member_expr),
            );
            let mut expr =
                self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, init);

            // Foo[Foo["x"] = init] = "x"
            if !is_str {
                let member_expr = {
                    let obj = self.ast.identifier_reference_expression(IdentifierReference::new(
                        SPAN,
                        enum_name.clone(),
                    ));
                    self.ast.computed_member(SPAN, obj, expr, false)
                };
                let left = AssignmentTarget::SimpleAssignmentTarget(
                    self.ast.simple_assignment_target_member_expression(member_expr),
                );
                let right = self
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));
                expr =
                    self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);
            }

            statements.push(self.ast.expression_statement(member.span, expr));

            // 1 + Foo["x"]
            default_init = {
                let one = self.ast.literal_number_expression(NumberLiteral {
                    span: SPAN,
                    value: 1.0,
                    raw: "1",
                    base: NumberBase::Decimal,
                });

                self.ast.binary_expression(SPAN, one, BinaryOperator::Addition, self_ref)
            };
        }

        let enum_ref = self
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
        // return Foo;
        let return_stmt = self.ast.return_statement(SPAN, Some(enum_ref));
        statements.push(return_stmt);

        statements
    }
}
