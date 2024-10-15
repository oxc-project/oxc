#![allow(clippy::unused_self)]

use std::cell::Cell;

use rustc_hash::FxHashSet;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::SymbolFlags;
use oxc_span::{Atom, GetSpan, Span, SPAN};
use oxc_syntax::{
    operator::AssignmentOperator,
    reference::ReferenceFlags,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{TransformCtx, TypeScriptOptions};

pub struct TypeScriptAnnotations<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,

    // Options
    only_remove_type_imports: bool,

    /// Assignments to be added to the constructor body
    assignments: Vec<Assignment<'a>>,
    has_super_call: bool,

    has_jsx_element: bool,
    has_jsx_fragment: bool,
    jsx_element_import_name: String,
    jsx_fragment_import_name: String,
    type_identifier_names: FxHashSet<Atom<'a>>,
}

impl<'a, 'ctx> TypeScriptAnnotations<'a, 'ctx> {
    pub fn new(options: &TypeScriptOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
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
            ctx,
            only_remove_type_imports: options.only_remove_type_imports,
            has_super_call: false,
            assignments: vec![],
            has_jsx_element: false,
            has_jsx_fragment: false,
            jsx_element_import_name,
            jsx_fragment_import_name,
            type_identifier_names: FxHashSet::default(),
        }
    }
}

impl<'a, 'ctx> Traverse<'a> for TypeScriptAnnotations<'a, 'ctx> {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let mut no_modules_remaining = true;
        let mut some_modules_deleted = false;

        program.body.retain_mut(|stmt| {
            let need_retain = match stmt {
                Statement::ExportNamedDeclaration(decl) => {
                    if decl.export_kind.is_type() {
                        false
                    } else {
                        decl.specifiers.retain(|specifier| {
                            !(specifier.export_kind.is_type()
                                || self.type_identifier_names.contains(&specifier.exported.name())
                                || {
                                    if let ModuleExportName::IdentifierReference(ident) =
                                        &specifier.local
                                    {
                                        ident.reference_id.get().is_some_and(|id| {
                                            ctx.symbols().get_reference(id).is_type()
                                        })
                                    } else {
                                        false
                                    }
                                })
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
                    } else if self.only_remove_type_imports {
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
        if no_modules_remaining && some_modules_deleted && self.ctx.module_imports.is_empty() {
            let export_decl = ModuleDeclaration::ExportNamedDeclaration(
                ctx.ast.plain_export_named_declaration(SPAN, ctx.ast.vec(), None),
            );
            program.body.push(ctx.ast.statement_module_declaration(export_decl));
        }
    }

    fn enter_arrow_function_expression(
        &mut self,
        expr: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        expr.type_parameters = None;
        expr.return_type = None;
    }

    fn enter_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        decl.definite = false;
    }

    fn enter_binding_pattern(&mut self, pat: &mut BindingPattern<'a>, _ctx: &mut TraverseCtx<'a>) {
        pat.type_annotation = None;

        if pat.kind.is_binding_identifier() {
            pat.optional = false;
        }
    }

    fn enter_call_expression(&mut self, expr: &mut CallExpression<'a>, _ctx: &mut TraverseCtx<'a>) {
        expr.type_parameters = None;
    }

    fn enter_class(&mut self, class: &mut Class<'a>, _ctx: &mut TraverseCtx<'a>) {
        class.type_parameters = None;
        class.super_type_parameters = None;
        class.implements = None;
        class.r#abstract = false;
    }

    fn enter_class_body(&mut self, body: &mut ClassBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        // Remove type only members
        body.body.retain(|elem| match elem {
            ClassElement::MethodDefinition(method) => {
                matches!(method.r#type, MethodDefinitionType::MethodDefinition)
                    && !method.value.is_typescript_syntax()
            }
            ClassElement::PropertyDefinition(prop) => {
                if prop.declare {
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

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if expr.is_typescript_syntax() {
            let inner_expr = expr.get_inner_expression_mut();
            *expr = ctx.ast.move_expression(inner_expr);
        }
    }

    fn enter_simple_assignment_target(
        &mut self,
        target: &mut SimpleAssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(expr) = target.get_expression_mut() {
            match expr.get_inner_expression_mut() {
                // `foo!++` to `foo++`
                inner_expr @ Expression::Identifier(_) => {
                    let inner_expr = ctx.ast.move_expression(inner_expr);
                    let Expression::Identifier(ident) = inner_expr else {
                        unreachable!();
                    };
                    *target = SimpleAssignmentTarget::AssignmentTargetIdentifier(ident);
                }
                // `foo.bar!++` to `foo.bar++`
                inner_expr @ match_member_expression!(Expression) => {
                    let inner_expr = ctx.ast.move_expression(inner_expr);
                    let member_expr = inner_expr.into_member_expression();
                    *target = SimpleAssignmentTarget::from(member_expr);
                }
                _ => {
                    // This should be never hit until more syntax is added to the JavaScript/TypeScrips
                    self.ctx.error(OxcDiagnostic::error("Cannot strip out typescript syntax if SimpleAssignmentTarget is not an IdentifierReference or MemberExpression"));
                }
            }
        }
    }

    fn enter_assignment_target(
        &mut self,
        target: &mut AssignmentTarget<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(expr) = target.get_expression_mut() {
            let inner_expr = expr.get_inner_expression_mut();
            if inner_expr.is_member_expression() {
                let inner_expr = ctx.ast.move_expression(inner_expr);
                let member_expr = inner_expr.into_member_expression();
                *target = AssignmentTarget::from(member_expr);
            }
        }
    }

    fn enter_formal_parameter(
        &mut self,
        param: &mut FormalParameter<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        param.accessibility = None;
    }

    fn exit_function(&mut self, func: &mut Function<'a>, _ctx: &mut TraverseCtx<'a>) {
        func.this_param = None;
        func.type_parameters = None;
        func.return_type = None;
    }

    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        elem.type_parameters = None;
    }

    fn enter_method_definition(
        &mut self,
        def: &mut MethodDefinition<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // Collects parameter properties so that we can add an assignment
        // for each of them in the constructor body.
        if def.kind == MethodDefinitionKind::Constructor {
            for param in def.value.params.items.as_mut_slice() {
                if param.accessibility.is_some() || param.readonly || param.r#override {
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

    fn exit_method_definition(
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
                        ctx.ast.alloc_function_body(SPAN, ctx.ast.vec(), ctx.ast.vec())
                    })
                    .statements
                    .splice(
                        0..0,
                        self.assignments
                            .drain(..)
                            .map(|assignment| assignment.create_this_property_assignment(ctx)),
                    );
            }
        }
    }

    fn enter_new_expression(&mut self, expr: &mut NewExpression<'a>, _ctx: &mut TraverseCtx<'a>) {
        expr.type_parameters = None;
    }

    fn enter_property_definition(
        &mut self,
        def: &mut PropertyDefinition<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
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

    fn enter_accessor_property(
        &mut self,
        def: &mut AccessorProperty<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        def.accessibility = None;
        def.definite = false;
        def.type_annotation = None;
    }

    fn enter_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
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

    fn exit_statements(
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
                let mut new_stmts = ctx.ast.vec();
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
    fn enter_if_statement(&mut self, stmt: &mut IfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
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

        Self::replace_with_empty_block_if_ts(&mut stmt.consequent, ctx.current_scope_id(), ctx);

        if stmt.alternate.as_ref().is_some_and(Statement::is_typescript_syntax) {
            stmt.alternate = None;
        }
    }

    fn enter_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::replace_for_statement_body_with_empty_block_if_ts(
            &mut stmt.body,
            &stmt.scope_id,
            ctx,
        );
    }

    fn enter_for_in_statement(&mut self, stmt: &mut ForInStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::replace_for_statement_body_with_empty_block_if_ts(
            &mut stmt.body,
            &stmt.scope_id,
            ctx,
        );
    }

    fn enter_for_of_statement(&mut self, stmt: &mut ForOfStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::replace_for_statement_body_with_empty_block_if_ts(
            &mut stmt.body,
            &stmt.scope_id,
            ctx,
        );
    }

    fn enter_while_statement(&mut self, stmt: &mut WhileStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::replace_with_empty_block_if_ts(&mut stmt.body, ctx.current_scope_id(), ctx);
    }

    fn enter_do_while_statement(
        &mut self,
        stmt: &mut DoWhileStatement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::replace_with_empty_block_if_ts(&mut stmt.body, ctx.current_scope_id(), ctx);
    }

    fn enter_tagged_template_expression(
        &mut self,
        expr: &mut TaggedTemplateExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        expr.type_parameters = None;
    }

    fn enter_jsx_element(&mut self, _elem: &mut JSXElement<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.has_jsx_element = true;
    }

    fn enter_jsx_fragment(&mut self, _elem: &mut JSXFragment<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.has_jsx_fragment = true;
    }

    fn enter_ts_module_declaration(
        &mut self,
        decl: &mut TSModuleDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // NB: Namespace transform happens in `enter_program` visitor, and replaces retained
        // namespaces with functions. This visitor is called after, by which time any remaining
        // namespaces need to be deleted.
        self.type_identifier_names.insert(decl.id.name().clone());
    }
}

impl<'a, 'ctx> TypeScriptAnnotations<'a, 'ctx> {
    /// Check if the given name is a JSX pragma or fragment pragma import
    /// and if the file contains JSX elements or fragments
    fn is_jsx_imports(&self, name: &str) -> bool {
        self.has_jsx_element && name == self.jsx_element_import_name
            || self.has_jsx_fragment && name == self.jsx_fragment_import_name
    }

    fn create_block_with_statement(
        stmt: Statement<'a>,
        span: Span,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let scope_id = ctx.insert_scope_below_statement(&stmt, ScopeFlags::empty());
        let block = BlockStatement::new_with_scope_id(span, ctx.ast.vec1(stmt), scope_id);
        block.scope_id.set(Some(scope_id));
        Statement::BlockStatement(ctx.ast.alloc(block))
    }

    fn replace_for_statement_body_with_empty_block_if_ts(
        body: &mut Statement<'a>,
        scope_id: &Cell<Option<ScopeId>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let scope_id = scope_id.get().unwrap_or(ctx.current_scope_id());
        Self::replace_with_empty_block_if_ts(body, scope_id, ctx);
    }

    fn replace_with_empty_block_if_ts(
        stmt: &mut Statement<'a>,
        parent_scope_id: ScopeId,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if stmt.is_typescript_syntax() {
            let scope_id = ctx.create_child_scope(parent_scope_id, ScopeFlags::empty());
            let block = BlockStatement::new_with_scope_id(stmt.span(), ctx.ast.vec(), scope_id);
            *stmt = Statement::BlockStatement(ctx.ast.alloc(block));
        }
    }

    pub fn has_value_reference(&self, name: &str, ctx: &TraverseCtx<'a>) -> bool {
        if let Some(symbol_id) = ctx.scopes().get_root_binding(name) {
            // `import T from 'mod'; const T = 1;` The T has a value redeclaration
            // `import T from 'mod'; type T = number;` The T has a type redeclaration
            // If the symbol is still a value symbol after SymbolFlags::Import is removed, then it's a value redeclaration.
            // That means the import is shadowed, and we can safely remove the import.
            let has_value_redeclaration =
                (ctx.symbols().get_flags(symbol_id) - SymbolFlags::Import).is_value();
            if has_value_redeclaration {
                return false;
            }
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
        let reference_id = ctx.create_bound_reference(self.symbol_id, ReferenceFlags::Read);
        let id = IdentifierReference::new_with_reference_id(
            self.span,
            self.name.clone(),
            Some(reference_id),
        );

        ctx.ast.statement_expression(
            SPAN,
            ctx.ast.expression_assignment(
                SPAN,
                AssignmentOperator::Assign,
                ctx.ast
                    .simple_assignment_target_member_expression(ctx.ast.member_expression_static(
                        SPAN,
                        ctx.ast.expression_this(SPAN),
                        ctx.ast.identifier_name(self.span, &self.name),
                        false,
                    ))
                    .into(),
                ctx.ast.expression_from_identifier_reference(id),
            ),
        )
    }
}
