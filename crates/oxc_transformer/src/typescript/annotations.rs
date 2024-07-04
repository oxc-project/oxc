#![allow(clippy::unused_self)]

use std::{cell::Cell, rc::Rc};

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, GetSpan, Span, SPAN};
use oxc_syntax::{
    operator::AssignmentOperator, reference::ReferenceFlag, scope::ScopeFlags, symbol::SymbolId,
};
use oxc_traverse::TraverseCtx;
use rustc_hash::FxHashSet;

use crate::{context::Ctx, TypeScriptOptions};

pub struct TypeScriptAnnotations<'a> {
    #[allow(dead_code)]
    options: Rc<TypeScriptOptions>,
    ctx: Ctx<'a>,
    /// Assignments to be added to the constructor body
    assignments: Vec<Assignment<'a>>,
    has_super_call: bool,

    has_jsx_element: bool,
    has_jsx_fragment: bool,
    jsx_element_import_name: String,
    jsx_fragment_import_name: String,
    type_identifier_names: FxHashSet<Atom<'a>>,
}

impl<'a> TypeScriptAnnotations<'a> {
    pub fn new(options: Rc<TypeScriptOptions>, ctx: Ctx<'a>) -> Self {
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
            assignments: vec![],
            options,
            ctx,
            has_jsx_element: false,
            has_jsx_fragment: false,
            jsx_element_import_name,
            jsx_fragment_import_name,
            type_identifier_names: FxHashSet::default(),
        }
    }

    /// Check if the given name is a JSX pragma or fragment pragma import
    /// and if the file contains JSX elements or fragments
    fn is_jsx_imports(&self, name: &str) -> bool {
        self.has_jsx_element && name == self.jsx_element_import_name
            || self.has_jsx_fragment && name == self.jsx_fragment_import_name
    }

    // Remove type only imports/exports
    pub fn transform_program_on_exit(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut no_modules_remaining = true;
        let mut some_modules_deleted = false;

        program.body.retain_mut(|stmt| {
            let need_retain = match stmt {
                Statement::ExportNamedDeclaration(decl) => {
                    if decl.export_kind.is_type() {
                        false
                    } else {
                        decl.specifiers.retain(|specifier| {
                            !specifier.export_kind.is_type()
                                && !self.type_identifier_names.contains(&specifier.exported.name())
                        });

                        !decl.specifiers.is_empty()
                            || decl
                                .declaration
                                .as_ref()
                                .is_some_and(|decl| !decl.is_typescript_syntax())
                    }
                }
                Statement::ExportAllDeclaration(decl) => !decl.export_kind.is_type(),
                Statement::ExportDefaultDeclaration(decl) => !decl.is_typescript_syntax(),
                Statement::ImportDeclaration(decl) => {
                    if decl.import_kind.is_type() {
                        false
                    } else if self.options.only_remove_type_imports {
                        true
                    } else if let Some(specifiers) = &mut decl.specifiers {
                        if specifiers.is_empty() {
                            // import {} from 'mod' -> import 'mod'
                            decl.specifiers = None;
                            true
                        } else {
                            specifiers.retain(|specifier| {
                                let id = match specifier {
                                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                        if s.import_kind.is_type() {
                                            return false;
                                        }
                                        &s.local
                                    }
                                    ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                        &s.local
                                    }
                                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                        &s.local
                                    }
                                };
                                self.has_value_reference(&id.name, ctx)
                            });
                            !specifiers.is_empty()
                        }
                    } else {
                        true
                    }
                }
                Statement::TSExportAssignment(_) | Statement::TSNamespaceExportDeclaration(_) => {
                    false
                }
                _ => return true,
            };

            if need_retain {
                no_modules_remaining = false;
            } else {
                some_modules_deleted = true;
            }

            need_retain
        });

        // Determine if we still have import/export statements, otherwise we
        // need to inject an empty statement (`export {}`) so that the file is
        // still considered a module
        if no_modules_remaining && some_modules_deleted {
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
        class.r#abstract = false;
    }

    pub fn transform_class_body(&mut self, body: &mut ClassBody<'a>) {
        // Remove type only members
        body.body.retain(|elem| match elem {
            ClassElement::MethodDefinition(method) => {
                matches!(method.r#type, MethodDefinitionType::MethodDefinition)
                    && !method.value.is_typescript_syntax()
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
            ClassElement::AccessorProperty(prop) => {
                matches!(prop.r#type, AccessorPropertyType::AccessorProperty)
            }
            ClassElement::TSIndexSignature(_) => false,
            ClassElement::StaticBlock(_) => true,
        });
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        if expr.is_typescript_syntax() {
            *expr = self.ctx.ast.copy(expr.get_inner_expression());
        }
    }

    pub fn transform_simple_assignment_target(&mut self, target: &mut SimpleAssignmentTarget<'a>) {
        if let Some(expr) = target.get_expression() {
            if let Expression::Identifier(ident) = expr.get_inner_expression() {
                let ident = self.ctx.ast.copy(ident);
                *target = SimpleAssignmentTarget::AssignmentTargetIdentifier(ident);
            }
        }
    }

    pub fn transform_assignment_target(&mut self, target: &mut AssignmentTarget<'a>) {
        if let Some(expr) = target.get_expression() {
            if let Some(member_expr) = expr.get_inner_expression().as_member_expression() {
                *target = AssignmentTarget::from(self.ctx.ast.copy(member_expr));
            }
        }
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
            for param in def.value.params.items.as_mut_slice() {
                if param.accessibility.is_some() {
                    if let Some(id) = param.pattern.get_binding_identifier() {
                        self.assignments.push(Assignment {
                            span: id.span,
                            name: id.name.clone(),
                            symbol_id: id.symbol_id.get().unwrap(),
                        });
                    }
                }

                param.readonly = false;
                param.accessibility = None;
                param.r#override = false;
            }
        }

        def.accessibility = None;
        def.optional = false;
        def.r#override = false;
    }

    pub fn transform_method_definition_on_exit(
        &mut self,
        def: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
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
                    .extend(
                        self.assignments
                            .drain(..)
                            .map(|assignment| assignment.create_this_property_assignment(ctx)),
                    );
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

    pub fn transform_statements(&mut self, stmts: &mut ArenaVec<'a, Statement<'a>>) {
        // Remove declare declaration
        stmts.retain(
            |stmt| {
                if let Some(decl) = stmt.as_declaration() {
                    !decl.declare()
                } else {
                    true
                }
            },
        );
    }

    pub fn transform_statements_on_exit(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Remove TS specific statements
        stmts.retain(|stmt| match stmt {
            Statement::ExpressionStatement(s) => !s.expression.is_typescript_syntax(),
            // Any namespaces left after namespace transform are type only, so remove them
            Statement::TSModuleDeclaration(_) => false,
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
                        new_stmts.extend(
                            self.assignments
                                .iter()
                                .map(|assignment| assignment.create_this_property_assignment(ctx)),
                        );
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
    pub fn transform_if_statement(
        &mut self,
        stmt: &mut IfStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if !self.assignments.is_empty() {
            let consequent_span = match &stmt.consequent {
                Statement::ExpressionStatement(expr)
                    if expr.expression.is_super_call_expression() =>
                {
                    Some(expr.span)
                }
                _ => None,
            };
            if let Some(span) = consequent_span {
                let consequent = ctx.ast.move_statement(&mut stmt.consequent);
                stmt.consequent = Self::create_block_with_statement(consequent, span, ctx);
            }

            let alternate_span = match &stmt.alternate {
                Some(Statement::ExpressionStatement(expr))
                    if expr.expression.is_super_call_expression() =>
                {
                    Some(expr.span)
                }
                _ => None,
            };
            if let Some(span) = alternate_span {
                let alternate = stmt.alternate.take().unwrap();
                stmt.alternate = Some(Self::create_block_with_statement(alternate, span, ctx));
            }
        }

        Self::replace_with_empty_block_if_ts(&mut stmt.consequent, ctx);

        if stmt.alternate.as_ref().is_some_and(Statement::is_typescript_syntax) {
            stmt.alternate = None;
        }
    }

    fn create_block_with_statement(
        stmt: Statement<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let scope_id = ctx.insert_scope_below_statement(&stmt, ScopeFlags::empty());
        let block = BlockStatement {
            span,
            body: ctx.ast.new_vec_single(stmt),
            scope_id: Cell::new(Some(scope_id)),
        };
        Statement::BlockStatement(ctx.ast.alloc(block))
    }

    pub fn transform_for_statement(
        &mut self,
        stmt: &mut ForStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::replace_with_empty_block_if_ts(&mut stmt.body, ctx);
    }

    pub fn transform_while_statement(
        &mut self,
        stmt: &mut WhileStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::replace_with_empty_block_if_ts(&mut stmt.body, ctx);
    }

    pub fn transform_do_while_statement(
        &mut self,
        stmt: &mut DoWhileStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::replace_with_empty_block_if_ts(&mut stmt.body, ctx);
    }

    fn replace_with_empty_block_if_ts(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if stmt.is_typescript_syntax() {
            let scope_id = ctx.create_scope_child_of_current(ScopeFlags::empty());
            let block = BlockStatement {
                span: stmt.span(),
                body: ctx.ast.new_vec(),
                scope_id: Cell::new(Some(scope_id)),
            };
            *stmt = Statement::BlockStatement(ctx.ast.alloc(block));
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

    pub fn transform_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        let Some(specifiers) = &decl.specifiers else {
            return;
        };
        let is_type = decl.import_kind.is_type();
        for specifier in specifiers {
            let mut specifier_is_type = is_type;
            let id = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    if s.import_kind.is_type() {
                        specifier_is_type = true;
                    }
                    &s.local
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => &s.local,
            };
            if specifier_is_type {
                self.type_identifier_names.insert(id.name.clone());
            }
        }
    }

    pub fn transform_export_named_declaration(&mut self, decl: &mut ExportNamedDeclaration<'a>) {
        let is_type = decl.export_kind.is_type();
        for specifier in &decl.specifiers {
            if is_type || specifier.export_kind.is_type() {
                self.type_identifier_names.insert(specifier.local.name().clone());
            }
        }
    }

    pub fn transform_ts_module_declaration(&mut self, decl: &mut TSModuleDeclaration<'a>) {
        // NB: Namespace transform happens in `enter_program` visitor, and replaces retained
        // namespaces with functions. This visitor is called after, by which time any remaining
        // namespaces need to be deleted.
        self.type_identifier_names.insert(decl.id.name().clone());
    }

    pub fn has_value_reference(&self, name: &str, ctx: &TraverseCtx<'a>) -> bool {
        if let Some(symbol_id) = ctx.scopes().get_root_binding(name) {
            if ctx
                .symbols()
                .get_resolved_references(symbol_id)
                .any(|reference| !reference.is_type())
            {
                return true;
            }
        }

        self.is_jsx_imports(name)
    }
}

struct Assignment<'a> {
    span: Span,
    name: Atom<'a>,
    symbol_id: SymbolId,
}

impl<'a> Assignment<'a> {
    // Creates `this.name = name`
    fn create_this_property_assignment(&self, ctx: &mut TraverseCtx<'a>) -> Statement<'a> {
        let reference_id = ctx.create_bound_reference(
            self.name.to_compact_str(),
            self.symbol_id,
            ReferenceFlag::Read,
        );
        let id = IdentifierReference::new_read(self.span, self.name.clone(), Some(reference_id));

        ctx.ast.expression_statement(
            SPAN,
            ctx.ast.assignment_expression(
                SPAN,
                AssignmentOperator::Assign,
                ctx.ast.simple_assignment_target_member_expression(ctx.ast.static_member(
                    SPAN,
                    ctx.ast.this_expression(SPAN),
                    IdentifierName::new(self.span, self.name.clone()),
                    false,
                )),
                ctx.ast.identifier_reference_expression(id),
            ),
        )
    }
}
