use oxc_allocator::{Box, Vec};
use oxc_ast::{ast::*, AstBuilder};
use oxc_semantic::SymbolFlags;
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator},
    NumberBase,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{mem, rc::Rc};

mod options;

pub use self::options::TypescriptOptions;
use crate::{context::TransformerCtx, utils::is_valid_identifier, TransformOptions};

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
    export_name_set: FxHashSet<Atom<'a>>,
    options: TypescriptOptions,
    namespace_arg_names: FxHashMap<Atom<'a>, usize>,
}

impl<'a> TypeScript<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        verbatim_module_syntax: bool,
        options: &TransformOptions,
    ) -> Self {
        Self {
            ast,
            ctx,
            verbatim_module_syntax,
            export_name_set: FxHashSet::default(),
            options: options.typescript.clone().unwrap_or_default(),
            namespace_arg_names: FxHashMap::default(),
        }
    }

    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>) {
        match decl {
            Declaration::TSImportEqualsDeclaration(ts_import_equals)
                if ts_import_equals.import_kind.is_value() =>
            {
                *decl = self.transform_ts_import_equals(ts_import_equals);
            }
            Declaration::TSEnumDeclaration(ts_enum_declaration) => {
                if let Some(expr) = self.transform_ts_enum(ts_enum_declaration) {
                    *decl = expr;
                }
            }
            _ => {}
        }
    }

    pub fn transform_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        self.insert_let_decl_for_ts_module_block(stmts);
    }

    pub fn transform_statement(&mut self, stmt: &mut Statement<'a>) {
        let new_stmt = match stmt {
            Statement::ModuleDeclaration(module_decl) => {
                if let ModuleDeclaration::ExportNamedDeclaration(export_decl) = &mut **module_decl {
                    self.transform_export_named_declaration(export_decl).or_else(|| {
                        export_decl.declaration.as_mut().and_then(|decl| {
                            if decl.modifiers().is_some_and(Modifiers::is_contains_declare) {
                                None
                            } else {
                                match decl {
                                    Declaration::TSModuleDeclaration(ts_module_decl) => {
                                        Some(self.transform_ts_module_block(ts_module_decl))
                                    }
                                    _ => None,
                                }
                            }
                        })
                    })
                } else {
                    None
                }
            }
            Statement::Declaration(Declaration::TSModuleDeclaration(ts_module_decl)) => {
                if ts_module_decl.modifiers.is_contains_declare() {
                    None
                } else {
                    Some(self.transform_ts_module_block(ts_module_decl))
                }
            }
            _ => None,
        };

        if let Some(new_stmt) = new_stmt {
            *stmt = new_stmt;
        }
    }

    /// * Remove the top level import / export statements that are types
    /// * Adds `export {}` if all import / export statements are removed, this is used to tell
    /// downstream tools that this file is in ESM.
    pub fn transform_program(&mut self, program: &mut Program<'a>) {
        let mut export_type_names = FxHashSet::default();

        // // Collect export names
        program.body.iter().for_each(|stmt| {
            if let Statement::ModuleDeclaration(module_decl) = stmt {
                if let ModuleDeclaration::ExportNamedDeclaration(decl) = &**module_decl {
                    decl.specifiers.iter().for_each(|specifier| {
                        let name = specifier.exported.name();
                        if self.is_import_binding_only(name)
                            && (decl.export_kind.is_type() || specifier.export_kind.is_type())
                        {
                            export_type_names.insert(name.clone());
                        }
                    });
                }
            }
        });

        let mut import_type_names = FxHashSet::default();
        let mut delete_indexes = vec![];
        let mut module_declaration_len = 0;

        for (index, stmt) in program.body.iter_mut().enumerate() {
            if let Statement::ModuleDeclaration(module_decl) = stmt {
                module_declaration_len += 1;
                match &mut **module_decl {
                    ModuleDeclaration::ExportNamedDeclaration(decl) => {
                        decl.specifiers.retain(|specifier| {
                            !(specifier.export_kind.is_type()
                                || import_type_names.contains(specifier.exported.name()))
                        });

                        if decl.export_kind.is_type()
                            || self.verbatim_module_syntax
                            || ((decl.declaration.is_none()
                                || decl.declaration.as_ref().is_some_and(|d| {
                                    d.modifiers().is_some_and(|modifiers| {
                                        modifiers.contains(ModifierKind::Declare)
                                    }) || matches!(
                                        d,
                                        Declaration::TSInterfaceDeclaration(_)
                                            | Declaration::TSTypeAliasDeclaration(_)
                                    )
                                }))
                                && decl.specifiers.is_empty())
                        {
                            delete_indexes.push(index);
                        }
                    }
                    ModuleDeclaration::ImportDeclaration(decl) => {
                        let is_type = decl.import_kind.is_type();

                        let is_specifiers_empty =
                            decl.specifiers.as_ref().is_some_and(|s| s.is_empty());

                        if let Some(specifiers) = &mut decl.specifiers {
                            specifiers.retain(|specifier| match specifier {
                                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                    if is_type || s.import_kind.is_type() {
                                        import_type_names.insert(s.local.name.clone());
                                        return false;
                                    }

                                    if self.verbatim_module_syntax
                                        || self.options.only_remove_type_imports
                                    {
                                        return true;
                                    }

                                    if export_type_names.contains(&s.local.name) {
                                        return false;
                                    }

                                    self.has_value_references(&s.local.name)
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(s)
                                    if !self.verbatim_module_syntax =>
                                {
                                    if is_type {
                                        import_type_names.insert(s.local.name.clone());
                                        return false;
                                    }

                                    if self.options.only_remove_type_imports {
                                        return true;
                                    }
                                    self.has_value_references(&s.local.name)
                                }
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s)
                                    if !self.verbatim_module_syntax =>
                                {
                                    if is_type {
                                        import_type_names.insert(s.local.name.clone());
                                    }

                                    if self.options.only_remove_type_imports {
                                        return true;
                                    }

                                    self.has_value_references(&s.local.name)
                                }
                                _ => true,
                            });
                        }

                        if decl.import_kind.is_type()
                            || (!self.options.only_remove_type_imports
                                && !is_specifiers_empty
                                && decl
                                    .specifiers
                                    .as_ref()
                                    .is_some_and(|specifiers| specifiers.is_empty()))
                        {
                            delete_indexes.push(index);
                        }
                    }
                    _ => {}
                }
            }
        }

        let delete_indexes_len = delete_indexes.len();

        // remove empty imports/exports
        for index in delete_indexes.into_iter().rev() {
            program.body.remove(index);
        }

        // explicit esm
        if module_declaration_len > 0 && module_declaration_len == delete_indexes_len {
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

    /// ```ts
    /// import foo from "foo"; // is import binding only
    /// import bar from "bar"; // SymbolFlags::ImportBinding | SymbolFlags::BlockScopedVariable
    /// let bar = "xx";
    /// ```
    fn is_import_binding_only(&self, name: &Atom) -> bool {
        let root_scope_id = self.ctx.scopes().root_scope_id();

        self.ctx.scopes().get_binding(root_scope_id, name).is_some_and(|symbol_id| {
            let flag = self.ctx.symbols().get_flag(symbol_id);
            flag.is_import_binding()
                && !flag.intersects(
                    SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable,
                )
        })
    }

    fn has_value_references(&self, name: &Atom) -> bool {
        let root_scope_id = self.ctx.scopes().root_scope_id();

        self.ctx
            .scopes()
            .get_binding(root_scope_id, name)
            .map(|symbol_id| {
                self.ctx.symbols().get_flag(symbol_id).is_export()
                    || self.ctx.symbols().get_resolved_references(symbol_id).any(|x| !x.is_type())
            })
            .unwrap_or_default()
    }
}

impl<'a> TypeScript<'a> {
    fn transform_ts_enum_members(
        &self,
        members: &mut Vec<'a, TSEnumMember<'a>>,
        enum_name: &Atom<'a>,
    ) -> Vec<'a, Statement<'a>> {
        let mut default_init = self.ast.literal_number_expression(NumericLiteral {
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
                | TSEnumMemberName::NumericLiteral(..) => unreachable!(),
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
            let left = self.ast.simple_assignment_target_member_expression(member_expr);
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
                let left = self.ast.simple_assignment_target_member_expression(member_expr);
                let right = self
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));
                expr =
                    self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);
            }

            statements.push(self.ast.expression_statement(member.span, expr));

            // 1 + Foo["x"]
            default_init = {
                let one = self.ast.literal_number_expression(NumericLiteral {
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

    fn transform_ts_type_name(&self, type_name: &mut TSTypeName<'a>) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(reference) => self.ast.identifier_reference_expression(
                IdentifierReference::new(SPAN, reference.name.clone()),
            ),
            TSTypeName::QualifiedName(qualified_name) => self.ast.static_member_expression(
                SPAN,
                self.transform_ts_type_name(&mut qualified_name.left),
                qualified_name.right.clone(),
                false,
            ),
        }
    }

    /// ```TypeScript
    /// import b = babel;
    /// import AliasModule = LongNameModule;
    ///
    /// ```JavaScript
    /// var b = babel;
    /// var AliasModule = LongNameModule;
    /// ```
    fn transform_ts_import_equals(
        &self,
        decl: &mut Box<'a, TSImportEqualsDeclaration<'a>>,
    ) -> Declaration<'a> {
        let kind = VariableDeclarationKind::Var;
        let decls = {
            let binding_identifier = BindingIdentifier::new(SPAN, decl.id.name.clone());
            let binding_pattern_kind = self.ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ast.binding_pattern(binding_pattern_kind, None, false);

            let init = match &mut decl.module_reference.0 {
                TSModuleReference::TypeName(type_name) => self.transform_ts_type_name(type_name),
                TSModuleReference::ExternalModuleReference(reference) => {
                    let callee = self.ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, "require".into()),
                    );
                    let arguments = self.ast.new_vec_single(Argument::Expression(
                        self.ast.literal_string_expression(reference.expression.clone()),
                    ));
                    self.ast.call_expression(SPAN, callee, arguments, false, None)
                }
            };
            self.ast.new_vec_single(self.ast.variable_declarator(
                SPAN,
                kind,
                binding,
                Some(init),
                false,
            ))
        };
        let variable_declaration =
            self.ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());

        Declaration::VariableDeclaration(variable_declaration)
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
    fn transform_ts_enum(
        &self,
        decl: &mut Box<'a, TSEnumDeclaration<'a>>,
    ) -> Option<Declaration<'a>> {
        if decl.modifiers.contains(ModifierKind::Declare) {
            return None;
        }

        let span = decl.span;
        let ident = decl.id.clone();
        let kind = self.ast.binding_pattern_identifier(ident);
        let id = self.ast.binding_pattern(kind, None, false);

        let mut params = self.ast.new_vec();

        // ((Foo) => {
        params.push(self.ast.formal_parameter(SPAN, id, None, false, false, self.ast.new_vec()));

        let params = self.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            None,
        );

        // Foo[Foo["X"] = 0] = "X";
        let enum_name = decl.id.name.clone();
        let statements = self.transform_ts_enum_members(&mut decl.members, &enum_name);
        let body = self.ast.function_body(decl.span, self.ast.new_vec(), statements);

        let callee =
            self.ast.arrow_function_expression(SPAN, false, false, params, body, None, None);

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

        Some(Declaration::VariableDeclaration(variable_declaration))
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
    fn transform_export_named_declaration(
        &mut self,
        decl: &mut Box<'_, ExportNamedDeclaration<'a>>,
    ) -> Option<Statement<'a>> {
        let ExportNamedDeclaration {
            declaration: Some(declaration),
            source: None,
            export_kind: ImportOrExportKind::Value,
            ..
        } = &mut **decl
        else {
            return None;
        };

        let id = match &declaration {
            Declaration::TSEnumDeclaration(decl) => decl.id.name.clone(),
            Declaration::TSModuleDeclaration(decl) => {
                let TSModuleDeclarationName::Identifier(id) = &decl.id else {
                    return None;
                };

                id.name.clone()
            }
            _ => return None,
        };

        if self.export_name_set.insert(id) {
            return None;
        }

        Some(Statement::Declaration(self.ast.move_declaration(declaration)))
    }

    /// Insert let declaration for ts module block
    fn insert_let_decl_for_ts_module_block(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        let mut insert_var_decl = vec![];

        for (index, stmt) in stmts.iter().enumerate() {
            match stmt {
                Statement::Declaration(Declaration::TSModuleDeclaration(decl)) => {
                    if !decl.modifiers.is_contains_declare() {
                        insert_var_decl.push((index, decl.id.name().clone(), false));
                    }
                }
                Statement::ModuleDeclaration(module_decl) => {
                    if let ModuleDeclaration::ExportNamedDeclaration(decl) = &**module_decl {
                        if let Some(Declaration::TSModuleDeclaration(decl)) = &decl.declaration {
                            if !decl.modifiers.is_contains_declare() {
                                insert_var_decl.push((index, decl.id.name().clone(), true));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for (index, name, is_export) in insert_var_decl.into_iter().rev() {
            let kind = VariableDeclarationKind::Let;
            let decls = {
                let binding_identifier = BindingIdentifier::new(SPAN, name.clone());
                let binding_pattern_kind = self.ast.binding_pattern_identifier(binding_identifier);
                let binding = self.ast.binding_pattern(binding_pattern_kind, None, false);
                let decl = self.ast.variable_declarator(SPAN, kind, binding, None, false);
                self.ast.new_vec_single(decl)
            };
            let variable_declaration =
                self.ast.variable_declaration(SPAN, kind, decls, Modifiers::empty());

            let decl = Declaration::VariableDeclaration(variable_declaration);

            let stmt = if is_export {
                self.ast.module_declaration(ModuleDeclaration::ExportNamedDeclaration(
                    self.ast.export_named_declaration(
                        SPAN,
                        Some(decl),
                        self.ast.new_vec(),
                        None,
                        ImportOrExportKind::Value,
                    ),
                ))
            } else {
                Statement::Declaration(decl)
            };
            stmts.insert(index, stmt);
        }
    }

    fn get_namespace_arg_name(&mut self, name: &Atom<'a>) -> Atom<'a> {
        let count = self.namespace_arg_names.entry(name.clone()).or_insert(0);
        *count += 1;
        format!("_{name}{}", if *count > 1 { count.to_string() } else { String::new() }).into()
    }

    /// ```TypeScript
    /// // transform ts module block
    /// namespace Foo {
    /// }
    /// // to
    /// let Foo; // this line added in `insert_let_decl_for_ts_module_block`
    /// (function (_Foo) {
    /// })(Foo || (Foo = {}));
    /// ```
    fn transform_ts_module_block(
        &mut self,
        block: &mut Box<'a, TSModuleDeclaration<'a>>,
    ) -> Statement<'a> {
        let body_statements = match &mut block.body {
            TSModuleDeclarationBody::TSModuleDeclaration(decl) => {
                let transformed_module_block = self.transform_ts_module_block(decl);
                self.ast.new_vec_single(transformed_module_block)
            }
            TSModuleDeclarationBody::TSModuleBlock(ts_module_block) => {
                self.ast.move_statement_vec(&mut ts_module_block.body)
            }
        };

        let name = block.id.name();

        let callee = {
            let body = self.ast.function_body(SPAN, self.ast.new_vec(), body_statements);
            let arg_name = self.get_namespace_arg_name(name);
            let params = self.ast.formal_parameters(
                SPAN,
                FormalParameterKind::FormalParameter,
                self.ast.new_vec_single(self.ast.formal_parameter(
                    SPAN,
                    self.ast.binding_pattern(
                        self.ast.binding_pattern_identifier(BindingIdentifier::new(SPAN, arg_name)),
                        None,
                        false,
                    ),
                    None,
                    false,
                    false,
                    self.ast.new_vec(),
                )),
                None,
            );
            let function = self.ast.function(
                FunctionType::FunctionExpression,
                SPAN,
                None,
                false,
                false,
                None,
                params,
                Some(body),
                None,
                None,
                Modifiers::empty(),
            );
            let function_expr = self.ast.function_expression(function);
            self.ast.parenthesized_expression(SPAN, function_expr)
        };

        let arguments = {
            let right = {
                let left = self.ast.simple_assignment_target_identifier(IdentifierReference::new(
                    SPAN,
                    name.clone(),
                ));
                let right = self.ast.object_expression(SPAN, self.ast.new_vec(), None);
                self.ast.parenthesized_expression(
                    SPAN,
                    self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right),
                )
            };
            self.ast.new_vec_single(Argument::Expression(
                self.ast.logical_expression(
                    SPAN,
                    self.ast.identifier_reference_expression(IdentifierReference::new(
                        SPAN,
                        name.clone(),
                    )),
                    LogicalOperator::Or,
                    right,
                ),
            ))
        };
        let expr = self.ast.call_expression(SPAN, callee, arguments, false, None);
        self.ast.expression_statement(SPAN, expr)
    }

    /// Transform constructor method
    /// ```typescript
    ///
    /// constructor(public x) {
    ///   super();
    /// }
    /// // to
    /// constructor(x) {
    ///   super();
    ///   this.x = x;
    /// }
    /// ```
    pub fn transform_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        if !def.kind.is_constructor() {
            return;
        }

        let mut params_name = vec![];
        def.value.params.items.iter().for_each(|param| {
            if !param.accessibility.is_some_and(|a| matches!(a, TSAccessibility::Public)) {
                return;
            }
            match &param.pattern.kind {
                BindingPatternKind::BindingIdentifier(ident) => {
                    params_name.push(ident.name.clone());
                }
                BindingPatternKind::AssignmentPattern(pattern) => {
                    if let BindingPatternKind::BindingIdentifier(ident) = &pattern.left.kind {
                        params_name.push(ident.name.clone());
                    }
                }
                _ => {}
            }
        });

        let Some(body) = &mut def.value.body else {
            return;
        };

        for name in params_name {
            // TODO: We should push it before the super call
            body.statements.push(self.ast.expression_statement(
                SPAN,
                self.ast.assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    self.ast.simple_assignment_target_member_expression(self.ast.static_member(
                        SPAN,
                        self.ast.this_expression(SPAN),
                        IdentifierName::new(SPAN, name.clone()),
                        false,
                    )),
                    self.ast.identifier_reference_expression(IdentifierReference::new(SPAN, name)),
                ),
            ));
        }
    }
}
