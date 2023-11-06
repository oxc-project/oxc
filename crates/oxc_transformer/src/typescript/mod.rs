use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    identifier::is_identifier_start_all,
    operator::{AssignmentOperator, BinaryOperator},
    NumberBase,
};

use std::{mem, rc::Rc};

use crate::{context::TransformerCtx, utils::CreateVars};

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

    vars: Vec<'a, VariableDeclarator<'a>>,
}

impl<'a> TypeScript<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        verbatim_module_syntax: bool,
    ) -> Self {
        let vars = ast.new_vec();

        Self { ast, ctx, verbatim_module_syntax, vars }
    }

    #[allow(clippy::unused_self)]
    pub fn transform_formal_parameters(&self, params: &mut FormalParameters<'a>) {
        if params.items.get(0).is_some_and(|param| matches!(&param.pattern.kind, BindingPatternKind::BindingIdentifier(ident) if ident.name =="this")) {
            params.items.remove(0);
        }
    }

    /// ```TypeScript
    /// enum Foo {
    ///   X
    /// }
    /// ```
    /// ```JavaScript
    /// var Foo;
    /// ((Foo) => {
    ///   Foo[Foo["X"] = 0] = "X";
    /// })(Foo ||= {});
    /// ```
    pub fn transform_ts_enum_declaration(&mut self, stmt: &mut Statement<'a>) {
        let Statement::Declaration(Declaration::TSEnumDeclaration(ts_enum_declaration)) = stmt
        else {
            return;
        };

        if ts_enum_declaration.modifiers.contains(ModifierKind::Declare) {
            return;
        }

        let span = ts_enum_declaration.span;
        let ident = ts_enum_declaration.id.clone();
        let kind = self.ast.binding_pattern_identifier(ident.clone());
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

        // })(Foo ||= {});
        let mut arguments = self.ast.new_vec();
        let op = AssignmentOperator::LogicalOr;
        let ident_ref = self.create_new_var_with_name(ident.name);
        let left = self.ast.simple_assignment_target_identifier(ident_ref);
        let left = AssignmentTarget::SimpleAssignmentTarget(left);
        let assignment = self.ast.assignment_expression(
            SPAN,
            op,
            left,
            self.ast.object_expression(SPAN, self.ast.new_vec(), None),
        );
        arguments.push(Argument::Expression(assignment));

        let call_expression = self.ast.call_expression(SPAN, callee, arguments, false, None);
        *stmt = self.ast.expression_statement(span, call_expression);
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

            if is_valid_identifier(member_name) {
                let ident = IdentifierReference::new(member_span, member_name.clone());

                self_ref = self.ast.identifier_reference_expression(ident.clone());
                let init = mem::replace(&mut init, self.ast.identifier_reference_expression(ident));

                let kind = VariableDeclarationKind::Const;
                let decls = {
                    let mut decls = self.ctx().ast.new_vec();

                    let binding_identifier = BindingIdentifier::new(SPAN, member_name.clone());
                    let binding_pattern_kind =
                        self.ctx().ast.binding_pattern_identifier(binding_identifier);
                    let binding = self.ctx().ast.binding_pattern(binding_pattern_kind, None, false);
                    let decl =
                        self.ctx().ast.variable_declarator(SPAN, kind, binding, Some(init), false);

                    decls.push(decl);
                    decls
                };
                let decl =
                    self.ctx().ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());
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

        statements
    }
}

impl<'a> CreateVars<'a> for TypeScript<'a> {
    fn ctx(&self) -> &TransformerCtx<'a> {
        &self.ctx
    }

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>> {
        &mut self.vars
    }
}

fn is_valid_identifier(name: &Atom) -> bool {
    let mut chars = name.chars();

    chars.next().map_or(false, is_identifier_start_all) && chars.all(is_identifier_start_all)
}
