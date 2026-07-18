use super::PeepholeOptimizations;
use crate::{CompressOptionsUnused, TraverseCtx};
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{DetermineValueType, ValueType};
use oxc_syntax::symbol::SymbolId;

impl<'a> PeepholeOptimizations {
    pub(super) fn can_remove_unused_declarators(ctx: &TraverseCtx<'a>) -> bool {
        ctx.state.options.unused != CompressOptionsUnused::Keep
            && !Self::is_script_root_scope(ctx)
            && !ctx.scoping().root_scope_flags().contains_direct_eval()
    }

    /// Count-based unusedness for declaration removal and IIFE folding. The
    /// assignment, member-write, and single-use-substitution consumers instead
    /// pair `symbol_is_implicitly_observable` with their own count thresholds,
    /// because some runtime semantics can observe a binding independently of
    /// resolved references.
    pub(super) fn symbol_is_unused_by_count(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        !ctx.state.symbol_is_implicitly_observable(symbol_id)
            && ctx.scoping().symbol_is_unused(symbol_id)
    }

    /// Function declarations additionally consume graph deadness, allowing
    /// self- and mutually-recursive cycles to be removed.
    fn function_has_no_live_references(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        Self::symbol_is_unused_by_count(symbol_id, ctx) || ctx.state.function_is_dead(symbol_id)
    }

    /// Return `true` when an exact function-valued initializer contains every
    /// reference to its own binding. Creating a function or arrow has no side
    /// effects, and without a reference from outside that function it can
    /// never be called.
    ///
    /// This check deliberately runs at the declarator removal site instead of
    /// registering declarators in the recursive-function graph. That keeps
    /// mutual declarator cycles unsupported, but also means statement
    /// relocation cannot leave a dead candidate in a non-removable AST slot.
    /// For example, `const f = () => f()` is removable, while adding `use(f)`
    /// supplies an outside reference and keeps it.
    fn self_recursive_function_declarator_is_unused(
        decl: &VariableDeclarator<'a>,
        symbol_id: SymbolId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let Some(function_scope_id) = decl.init.as_ref().and_then(|init| match init {
            Expression::FunctionExpression(function) => function.scope_id.get(),
            Expression::ArrowFunctionExpression(arrow) => arrow.scope_id.get(),
            _ => None,
        }) else {
            return false;
        };

        // Covers exports, Script-root bindings, Annex B aliases, and `using`.
        if ctx.state.symbol_is_implicitly_observable(symbol_id) {
            return false;
        }

        ctx.scoping().get_resolved_references(symbol_id).all(|reference| {
            ctx.scoping()
                .scope_ancestors(reference.scope_id())
                .any(|scope_id| scope_id == function_scope_id)
        })
    }

    fn is_sync_iterator_expr(expr: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        match expr {
            Expression::ArrayExpression(_)
            | Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_) => true,
            Expression::Identifier(ident) => {
                ident.name == "arguments"
                    && ctx.is_global_reference(ident)
                    // arguments can be reassigned in non-strict mode
                    && ctx.current_scope_flags().is_strict_mode()
                    // check if any scope in a chain is a non-arrow function
                    && ctx.ancestor_scopes().any(|scope| {
                        let scope_flags = ctx.scoping().scope_flags(scope);
                        scope_flags.is_function() && !scope_flags.is_arrow()
                    })
            }
            _ => false,
        }
    }

    pub fn should_remove_unused_declarator(
        decl: &VariableDeclarator<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        if !Self::can_remove_unused_declarators(ctx) {
            return false;
        }
        // Unsafe to remove `using`, unable to statically determine usage of [Symbol.dispose].
        if decl.kind.is_using() {
            return false;
        }
        match &decl.id {
            BindingPattern::BindingIdentifier(ident) => {
                if let Some(symbol_id) = ident.symbol_id.get() {
                    return Self::symbol_is_unused_by_count(symbol_id, ctx)
                        || Self::self_recursive_function_declarator_is_unused(
                            decl, symbol_id, ctx,
                        );
                }
                false
            }
            BindingPattern::ArrayPattern(ident) => {
                ident.is_empty()
                    && decl.init.as_ref().is_some_and(|expr| Self::is_sync_iterator_expr(expr, ctx))
            }
            BindingPattern::ObjectPattern(ident) => {
                ident.is_empty()
                    && decl.init.as_ref().is_some_and(|expr| {
                        !matches!(
                            expr.value_type(ctx),
                            ValueType::Null | ValueType::Undefined | ValueType::Undetermined
                        )
                    })
            }
            BindingPattern::AssignmentPattern(_) => false,
        }
    }

    /// Filter unused declarators out of a `KeepVar`-synthesized `var`
    /// statement. Both callers pass `KeepVar` output: a TRANSIENT statement
    /// that is not (yet) part of the live tree, whose declarators never
    /// carry initializers (`KeepVar` hoists names only).
    ///
    /// Because the statement is transient and init-less, removing a
    /// declarator discards no references and must NOT record a mutation or
    /// route through the `drop_*` helpers: if the caller then skips
    /// installation (e.g. `try_fold_if`'s already-canonical slot), nothing
    /// in the live AST changed this pass, and a spurious mutation spins the
    /// fixed-point loop past its iteration guard (bluebird.js, monitor-oxc).
    /// Installing — or declining to install — the filtered statement is the
    /// caller's mutation event.
    pub fn remove_unused_variable_declaration(
        mut stmt: Statement<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        let Statement::VariableDeclaration(var_decl) = &mut stmt else { return Some(stmt) };
        if !Self::can_remove_unused_declarators(ctx) {
            return Some(stmt);
        }
        var_decl.declarations.retain(|decl| {
            debug_assert!(
                decl.init.is_none(),
                "callers must pass KeepVar output (init-less declarators); a declarator \
                 with an init would need a `drop_*` walk for its references"
            );
            !Self::should_remove_unused_declarator(decl, ctx)
        });
        if var_decl.declarations.is_empty() {
            return None;
        }
        Some(stmt)
    }

    pub fn remove_unused_function_declaration(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::FunctionDeclaration(f) = stmt else { return };
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }
        let Some(id) = &f.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        if Self::is_script_root_scope(ctx) || ctx.current_scope_flags().contains_direct_eval() {
            return;
        }
        if !Self::function_has_no_live_references(symbol_id, ctx) {
            return;
        }
        let new_stmt = Statement::new_empty_statement(f.span, ctx);
        ctx.replace_statement(stmt, new_stmt);
    }

    pub fn remove_unused_class_declaration(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ClassDeclaration(c) = stmt else { return };
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }
        let Some(id) = &c.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        if Self::is_script_root_scope(ctx) || ctx.current_scope_flags().contains_direct_eval() {
            return;
        }
        if !Self::symbol_is_unused_by_count(symbol_id, ctx) {
            return;
        }
        if let Some(changed) = Self::remove_unused_class(c, ctx).map(|exprs| {
            if exprs.is_empty() {
                Statement::new_empty_statement(c.span, ctx)
            } else {
                let expr = Expression::new_sequence_expression(c.span, exprs, ctx);
                Statement::new_expression_statement(c.span, expr, ctx)
            }
        }) {
            ctx.replace_statement(stmt, changed);
        }
    }

    /// Whether bindings in the current scope are visible to later scripts.
    pub fn is_script_root_scope(ctx: &TraverseCtx<'a>) -> bool {
        ctx.scoping.current_scope_id() == ctx.scoping().root_scope_id()
            && ctx.source_type().is_script()
    }

    /// Remove unused specifiers from import declarations.
    ///
    /// Since we don't know if an import has side effects, we convert imports
    /// with all unused specifiers to side-effect-only imports (`import 'x'`)
    /// rather than removing them entirely.
    ///
    /// ## Example
    ///
    /// Input:
    /// ```js
    /// import a from 'a'
    /// import { b } from 'b'
    ///
    /// if (false) {
    ///   console.log(b)
    /// }
    /// ```
    ///
    /// Output:
    /// ```js
    /// import 'a'
    /// import 'b'
    /// ```
    pub fn remove_unused_import_specifiers(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.options().treeshake.invalid_import_side_effects
            || ctx.state.options.unused == CompressOptionsUnused::Keep
        {
            return;
        }

        if ctx.scoping().root_scope_flags().contains_direct_eval() {
            return;
        }

        debug_assert!(!ctx.source_type().is_script(), "imports are not allowed in script mode");

        let Statement::ImportDeclaration(import_decl) = stmt else { return };

        if let Some(phase) = import_decl.phase {
            let (ImportPhase::Defer | ImportPhase::Source) = phase;
            if ctx.scoping().symbol_is_unused(
                import_decl.specifiers.as_ref().unwrap().first().unwrap().local().symbol_id(),
            ) {
                let new_stmt = Statement::new_empty_statement(import_decl.span, ctx);
                ctx.replace_statement(stmt, new_stmt);
            }

            return;
        }

        let Some(specifiers) = &mut import_decl.specifiers else {
            return;
        };

        let original_len = specifiers.len();

        specifiers.retain(|specifier| {
            let local = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => &s.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => &s.local,
            };

            let symbol_id = local.symbol_id();
            !ctx.scoping().symbol_is_unused(symbol_id)
        });

        if specifiers.len() != original_len {
            ctx.notice_change();
        }

        if specifiers.is_empty() {
            import_decl.specifiers = None;
            ctx.notice_change();
        }
    }
}
