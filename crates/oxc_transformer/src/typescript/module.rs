use oxc_allocator::TakeIn;
use oxc_ast::{NONE, ast::*};
use oxc_semantic::{Reference, SymbolFlags};
use oxc_span::SPAN;
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::Traverse;

use super::diagnostics;

use crate::{
    context::{TransformCtx, TraverseCtx},
    state::TransformState,
};

pub struct TypeScriptModule<'a, 'ctx> {
    /// <https://babeljs.io/docs/babel-plugin-transform-typescript#onlyremovetypeimports>
    only_remove_type_imports: bool,
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> TypeScriptModule<'a, 'ctx> {
    pub fn new(only_remove_type_imports: bool, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { only_remove_type_imports, ctx }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for TypeScriptModule<'a, '_> {
    #[inline]
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.mark_unused_import_equals_references_as_type(&program.body, ctx);
    }

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // In Babel, it will insert `use strict` in `@babel/transform-modules-commonjs` plugin.
        // Once we have a commonjs plugin, we can consider moving this logic there.
        if self.ctx.module.is_commonjs() {
            let has_use_strict = program.directives.iter().any(Directive::is_use_strict);
            if !has_use_strict {
                program.directives.insert(0, ctx.ast.use_strict_directive());
            }
        }
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::TSExportAssignment(export_assignment) = stmt {
            *stmt = self.transform_ts_export_assignment(export_assignment, ctx);
        }
    }

    fn enter_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Declaration::TSImportEqualsDeclaration(import_equals) = decl
            && import_equals.import_kind.is_value()
            && let Some(new_decl) = self.transform_ts_import_equals(import_equals, ctx)
        {
            *decl = new_decl;
        }
    }
}

impl<'a> TypeScriptModule<'a, '_> {
    /// Transform `export = expression` to `module.exports = expression`.
    fn transform_ts_export_assignment(
        &self,
        export_assignment: &mut TSExportAssignment<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        if self.ctx.module.is_esm() {
            self.ctx.error(diagnostics::export_assignment_cannot_bed_used_in_esm(
                export_assignment.span,
            ));
        }

        // module.exports
        let module_exports = {
            let reference_id =
                ctx.create_reference_in_current_scope("module", ReferenceFlags::Read);
            let reference =
                ctx.ast.alloc_identifier_reference_with_reference_id(SPAN, "module", reference_id);
            let object = Expression::Identifier(reference);
            let property = ctx.ast.identifier_name(SPAN, "exports");
            ctx.ast.member_expression_static(SPAN, object, property, false)
        };

        let left = AssignmentTarget::from(SimpleAssignmentTarget::from(module_exports));
        let right = export_assignment.expression.take_in(ctx.ast);
        let assignment_expr =
            ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, right);
        ctx.ast.statement_expression(SPAN, assignment_expr)
    }

    /// Transform TSImportEqualsDeclaration to a VariableDeclaration.
    ///
    /// ```TypeScript
    /// import module = require('module');
    /// import AliasModule = LongNameModule;
    ///
    /// ```JavaScript
    /// const module = require('module');
    /// const AliasModule = LongNameModule;
    /// ```
    fn transform_ts_import_equals(
        &self,
        decl: &mut TSImportEqualsDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Declaration<'a>> {
        if !self.only_remove_type_imports
            && !ctx.parent().is_export_named_declaration()
            && ctx.scoping().get_resolved_references(decl.id.symbol_id()).all(Reference::is_type)
        {
            // No value reference, we will remove this declaration in `TypeScriptAnnotations`
            let scope_id = ctx.current_scope_id();
            ctx.scoping_mut().remove_binding(scope_id, &decl.id.name);
            return None;
        }

        let binding = BindingPattern::BindingIdentifier(ctx.ast.alloc(decl.id.clone()));
        let decl_span = decl.span;

        let flags = ctx.scoping_mut().symbol_flags_mut(decl.id.symbol_id());
        flags.remove(SymbolFlags::Import);

        let (kind, init) = match &mut decl.module_reference {
            type_name @ match_ts_type_name!(TSModuleReference) => {
                flags.insert(SymbolFlags::FunctionScopedVariable);

                (
                    VariableDeclarationKind::Var,
                    self.transform_ts_type_name(&mut *type_name.to_ts_type_name_mut(), ctx),
                )
            }
            TSModuleReference::ExternalModuleReference(reference) => {
                flags.insert(SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable);

                if self.ctx.module.is_esm() {
                    self.ctx.error(diagnostics::import_equals_cannot_be_used_in_esm(decl_span));
                }

                let require_symbol_id =
                    ctx.scoping().find_binding(ctx.current_scope_id(), "require");
                let callee = ctx.create_ident_expr(
                    SPAN,
                    Atom::from("require"),
                    require_symbol_id,
                    ReferenceFlags::Read,
                );
                let arguments =
                    ctx.ast.vec1(Argument::StringLiteral(ctx.alloc(reference.expression.clone())));
                (
                    VariableDeclarationKind::Const,
                    ctx.ast.expression_call(SPAN, callee, NONE, arguments, false),
                )
            }
        };
        let decls =
            ctx.ast.vec1(ctx.ast.variable_declarator(SPAN, kind, binding, NONE, Some(init), false));

        Some(ctx.ast.declaration_variable(SPAN, kind, decls, false))
    }

    #[expect(clippy::self_only_used_in_recursion)]
    fn transform_ts_type_name(
        &self,
        type_name: &mut TSTypeName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                let ident = ident.clone();
                let reference = ctx.scoping_mut().get_reference_mut(ident.reference_id());
                *reference.flags_mut() = ReferenceFlags::Read;
                Expression::Identifier(ctx.alloc(ident))
            }
            TSTypeName::QualifiedName(qualified_name) => ctx
                .ast
                .member_expression_static(
                    SPAN,
                    self.transform_ts_type_name(&mut qualified_name.left, ctx),
                    qualified_name.right.clone(),
                    false,
                )
                .into(),
            TSTypeName::ThisExpression(e) => ctx.ast.expression_this(e.span),
        }
    }

    /// Mark identifiers in unused `import foo = bar.baz` module references as type references.
    ///
    /// When an `import foo = bar.baz` declaration is unused (no value references to `foo`),
    /// it will be removed by `TypeScriptAnnotations`. However, the reference to `bar` was marked
    /// as a Read reference during building Semantic. If we don't change it to a Type reference, the binding
    /// `bar` (which might come from another `import bar = ...`) would appear to have a value
    /// reference and wouldn't be removed.
    ///
    /// We process in reverse order to handle chains correctly:
    /// ```ts
    /// import x = foo.x  // reference to foo
    /// import y = x.y    // reference to x
    /// import z = y.z    // reference to y (unused)
    /// ```
    /// Processing `z` first marks `y`'s reference as Type, then `y` marks `x`, then `x` marks `foo`.
    ///
    /// Note: `TSImportEqualsDeclaration` can appear at the top-level and inside namespaces.
    /// We only need to process top-level `Program` here because `TypeScriptNamespace`
    /// has special handling that already covers the case for import equals inside namespaces.
    pub fn mark_unused_import_equals_references_as_type(
        &self,
        stmts: &[Statement<'a>],
        ctx: &mut TraverseCtx<'a>,
    ) {
        // When `only_remove_type_imports` is true, we don't remove unused value imports,
        // so we don't need to change reference flags.
        if self.only_remove_type_imports {
            return;
        }

        for stmt in stmts.iter().rev() {
            if let Statement::TSImportEqualsDeclaration(import_equals) = stmt {
                Self::mark_module_reference_as_type_if_binding_unused(import_equals, ctx);
            }
        }
    }

    /// If the binding of a `TSImportEqualsDeclaration` has no value references,
    /// mark the identifier reference in its module reference as a type reference.
    ///
    /// This is needed because when `import foo = bar.baz` is unused and will be removed,
    /// the reference to `bar` should not count as a value reference.
    fn mark_module_reference_as_type_if_binding_unused(
        decl: &TSImportEqualsDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // If the binding has any value reference, don't change the module reference flag.
        // We only change it when all references are type (meaning the binding is unused as a value).
        if !ctx.scoping().get_resolved_references(decl.id.symbol_id()).all(Reference::is_type) {
            return;
        }

        // For `import foo = bar.baz`, change the reference to `bar` from Read to Type.
        // For `import foo = require('module')`, there's no identifier reference to change.
        if let module_reference @ match_ts_type_name!(TSModuleReference) = &decl.module_reference
            && let Some(ident) = module_reference.to_ts_type_name().get_identifier_reference()
        {
            let reference = ctx.scoping_mut().get_reference_mut(ident.reference_id());
            // The binding of TSImportEqualsDeclaration will be treated as unused
            // because there is no value reference, so it will be removed.
            // Therefore its module reference should also be treated as a type reference.
            // Example: `import Unused = X.Y.Z`
            //                           ^ `X` was marked as Read reference but should be Type.
            let flags = reference.flags_mut();
            debug_assert!(flags.is_read());
            *flags = ReferenceFlags::Type;
        }
    }
}
