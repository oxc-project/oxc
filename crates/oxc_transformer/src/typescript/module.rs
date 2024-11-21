use oxc_ast::{ast::*, NONE};
use oxc_span::{CompactStr, SPAN};
use oxc_syntax::reference::ReferenceFlags;
use oxc_traverse::{Traverse, TraverseCtx};

use super::diagnostics;
use crate::TransformCtx;

pub struct TypeScriptModule<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> TypeScriptModule<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for TypeScriptModule<'a, 'ctx> {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // In Babel, it will insert `use strict` in `@babel/transform-modules-commonjs` plugin.
        // Once we have a commonjs plugin, we can consider moving this logic there.
        if self.ctx.module.is_commonjs() {
            let has_use_strict = program.directives.iter().any(Directive::is_use_strict);
            if !has_use_strict {
                let use_strict = ctx.ast.string_literal(SPAN, "use strict", None);
                program.directives.insert(0, ctx.ast.directive(SPAN, use_strict, "use strict"));
            }
        }
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::TSExportAssignment(export_assignment) = stmt {
            *stmt = self.transform_ts_export_assignment(export_assignment, ctx);
        }
    }

    fn enter_declaration(&mut self, decl: &mut Declaration<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Declaration::TSImportEqualsDeclaration(import_equals) = decl {
            if import_equals.import_kind.is_value() {
                *decl = self.transform_ts_import_equals(import_equals, ctx);
            }
        }
    }
}

impl<'a, 'ctx> TypeScriptModule<'a, 'ctx> {
    /// Transform `export = expression` to `module.exports = expression`.
    fn transform_ts_export_assignment(
        &mut self,
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
            let reference_id = ctx
                .create_reference_in_current_scope(CompactStr::new("module"), ReferenceFlags::Read);
            let reference =
                ctx.ast.alloc_identifier_reference_with_reference_id(SPAN, "module", reference_id);
            let object = Expression::Identifier(reference);
            let property = ctx.ast.identifier_name(SPAN, "exports");
            ctx.ast.member_expression_static(SPAN, object, property, false)
        };

        let left = AssignmentTarget::from(SimpleAssignmentTarget::from(module_exports));
        let right = ctx.ast.move_expression(&mut export_assignment.expression);
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
    ) -> Declaration<'a> {
        let binding_pattern_kind =
            ctx.ast.binding_pattern_kind_binding_identifier(SPAN, &decl.id.name);
        let binding = ctx.ast.binding_pattern(binding_pattern_kind, NONE, false);
        let decl_span = decl.span;

        let (kind, init) = match &mut decl.module_reference {
            type_name @ match_ts_type_name!(TSModuleReference) => (
                VariableDeclarationKind::Var,
                self.transform_ts_type_name(&mut *type_name.to_ts_type_name_mut(), ctx),
            ),
            TSModuleReference::ExternalModuleReference(reference) => {
                if self.ctx.module.is_esm() {
                    self.ctx.error(diagnostics::import_equals_cannot_be_used_in_esm(decl_span));
                }

                let callee = ctx.ast.expression_identifier_reference(SPAN, "require");
                let arguments =
                    ctx.ast.vec1(Argument::StringLiteral(ctx.alloc(reference.expression.clone())));
                (
                    VariableDeclarationKind::Const,
                    ctx.ast.expression_call(SPAN, callee, NONE, arguments, false),
                )
            }
        };
        let decls =
            ctx.ast.vec1(ctx.ast.variable_declarator(SPAN, kind, binding, Some(init), false));

        ctx.ast.declaration_variable(SPAN, kind, decls, false)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn transform_ts_type_name(
        &self,
        type_name: &mut TSTypeName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                let ident = ident.clone();
                let reference = ctx.symbols_mut().get_reference_mut(ident.reference_id());
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
        }
    }
}
