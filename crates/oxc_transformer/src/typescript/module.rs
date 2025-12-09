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
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Note: unused import equals are removed in TypeScript::exit_program before annotations runs
        
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
        // Transform TSImportEqualsDeclaration to variable declarations
        // Note: unused ones will be removed in TypeScript::exit_program before annotations runs
        if let Declaration::TSImportEqualsDeclaration(import_equals) = decl
            && import_equals.import_kind.is_value()
            && let Some(new_decl) = self.transform_ts_import_equals(import_equals, ctx)
        {
            *decl = new_decl;
        }
    }
}

impl<'a> TypeScriptModule<'a, '_> {
    /// Remove unused import equals declarations iteratively.
    /// This must be called BEFORE TypeScriptAnnotations.exit_program removes them.
    pub fn remove_unused_import_equals_from_program(
        &self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        eprintln!("remove_unused_import_equals_from_program: {} stmts", program.body.len());
        for (i, stmt) in program.body.iter().enumerate() {
            let ty = match stmt {
                Statement::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
                _ => "Other",
            };
            eprintln!("  stmt[{}]: {}", i, ty);
        }
        self.remove_unused_import_equals(&mut program.body, ctx);
    }

    /// Remove unused import equals declarations iteratively.
    /// This processes import equals in reverse order (bottom-up) to correctly detect chains.
    fn remove_unused_import_equals(
        &self,
        stmts: &mut oxc_allocator::Vec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // Iterate until no more import equals are removed
        loop {
            let mut removed_any = false;
            
            // Process statements in reverse order (bottom-up)
            for i in (0..stmts.len()).rev() {
                if let Statement::TSImportEqualsDeclaration(import_equals) = &stmts[i] {
                    if !import_equals.import_kind.is_value() {
                        continue;
                    }
                    
                    // Check if this import has no value references
                    let refs: Vec<_> = ctx
                        .scoping()
                        .get_resolved_references(import_equals.id.symbol_id())
                        .collect();
                    
                    eprintln!("Checking import {}: {} refs", import_equals.id.name, refs.len());
                    for (i, r) in refs.iter().enumerate() {
                        eprintln!("  ref[{}]: is_type={} flags={:?}", i, r.is_type(), r.flags());
                    }
                    
                    let has_value_refs = refs.iter().any(|r| !r.is_type());
                    eprintln!("  has_value_refs={}", has_value_refs);
                    
                    if !has_value_refs && !self.only_remove_type_imports {
                        // Mark the reference in module_reference as Type
                        match &import_equals.module_reference {
                            module_reference @ match_ts_type_name!(TSModuleReference) => {
                                if let Some(ident) =
                                    module_reference.to_ts_type_name().get_identifier_reference()
                                {
                                    let reference =
                                        ctx.scoping_mut().get_reference_mut(ident.reference_id());
                                    *reference.flags_mut() = ReferenceFlags::Type;
                                }
                            }
                            TSModuleReference::ExternalModuleReference(_) => {}
                        }
                        
                        // Remove the binding
                        let scope_id = ctx.current_scope_id();
                        ctx.scoping_mut().remove_binding(scope_id, &import_equals.id.name);
                        
                        // Replace with empty statement
                        stmts[i] = ctx.ast.statement_empty(SPAN);
                        removed_any = true;
                    }
                }
            }
            
            // If we didn't remove anything in this pass, we're done
            if !removed_any {
                break;
            }
        }
        
        // Now transform remaining import equals to variable declarations
        for stmt in stmts.iter_mut() {
            if let Statement::TSImportEqualsDeclaration(import_equals) = stmt {
                if import_equals.import_kind.is_value() {
                    let mut import_equals_owned = import_equals.take_in(ctx.ast);
                    if let Some(new_decl) = self.transform_ts_import_equals(&mut import_equals_owned, ctx) {
                        *stmt = Statement::from(new_decl);
                    }
                }
            }
        }
    }

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
            // No value reference, we will remove this declaration
            match &mut decl.module_reference {
                module_reference @ match_ts_type_name!(TSModuleReference) => {
                    if let Some(ident) =
                        module_reference.to_ts_type_name().get_identifier_reference()
                    {
                        let reference = ctx.scoping_mut().get_reference_mut(ident.reference_id());
                        // The binding of TSImportEqualsDeclaration has treated as a type reference,
                        // so an identifier reference that it referenced also should be treated as a type reference.
                        // `import TypeBinding = X.Y.Z`
                        //                       ^ `X` should be treated as a type reference.
                        let flags = reference.flags_mut();
                        debug_assert_eq!(*flags, ReferenceFlags::Read);
                        *flags = ReferenceFlags::Type;
                    }
                }
                TSModuleReference::ExternalModuleReference(_) => {}
            }
            let scope_id = ctx.current_scope_id();
            ctx.scoping_mut().remove_binding(scope_id, &decl.id.name);
            return None;
        }

        let binding_pattern_kind =
            BindingPatternKind::BindingIdentifier(ctx.ast.alloc(decl.id.clone()));
        let binding = ctx.ast.binding_pattern(binding_pattern_kind, NONE, false);
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
            ctx.ast.vec1(ctx.ast.variable_declarator(SPAN, kind, binding, Some(init), false));

        Some(ctx.ast.declaration_variable(SPAN, kind, decls, false))
    }

    #[expect(clippy::only_used_in_recursion)]
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
}
