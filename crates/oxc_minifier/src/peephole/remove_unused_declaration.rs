use super::PeepholeOptimizations;
use crate::{CompressOptionsUnused, ctx::Ctx};
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{DetermineValueType, ValueType};

impl<'a> PeepholeOptimizations {
    fn can_remove_unused_declarators(ctx: &Ctx<'a, '_>) -> bool {
        ctx.state.options.unused != CompressOptionsUnused::Keep
            && !Self::keep_top_level_var_in_script_mode(ctx)
            && !ctx.scoping().root_scope_flags().contains_direct_eval()
    }

    fn is_sync_iterator_expr(expr: &Expression<'a>, ctx: &Ctx<'a, '_>) -> bool {
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
        ctx: &Ctx<'a, '_>,
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
                    return ctx.scoping().symbol_is_unused(symbol_id);
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

    pub fn remove_unused_variable_declaration(
        mut stmt: Statement<'a>,
        ctx: &Ctx<'a, '_>,
    ) -> Option<Statement<'a>> {
        let Statement::VariableDeclaration(var_decl) = &mut stmt else { return Some(stmt) };
        if !Self::can_remove_unused_declarators(ctx) {
            return Some(stmt);
        }
        var_decl.declarations.retain(|decl| !Self::should_remove_unused_declarator(decl, ctx));
        if var_decl.declarations.is_empty() {
            return None;
        }
        Some(stmt)
    }

    pub fn remove_unused_function_declaration(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::FunctionDeclaration(f) = stmt else { return };
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }
        let Some(id) = &f.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        if Self::keep_top_level_var_in_script_mode(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
        {
            return;
        }
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return;
        }
        *stmt = ctx.ast.statement_empty(f.span);
        ctx.state.changed = true;
    }

    pub fn remove_unused_class_declaration(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::ClassDeclaration(c) = stmt else { return };
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return;
        }
        let Some(id) = &c.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        if Self::keep_top_level_var_in_script_mode(ctx)
            || ctx.current_scope_flags().contains_direct_eval()
        {
            return;
        }
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return;
        }
        if let Some(changed) = Self::remove_unused_class(c, ctx).map(|exprs| {
            if exprs.is_empty() {
                ctx.ast.statement_empty(c.span)
            } else {
                let expr = ctx.ast.expression_sequence(c.span, exprs);
                ctx.ast.statement_expression(c.span, expr)
            }
        }) {
            *stmt = changed;
            ctx.state.changed = true;
        }
    }

    /// Do remove top level vars in script mode.
    pub fn keep_top_level_var_in_script_mode(ctx: &Ctx<'a, '_>) -> bool {
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
    pub fn remove_unused_import_specifiers(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
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
                *stmt = ctx.ast.statement_empty(import_decl.span);
                ctx.state.changed = true;
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
            ctx.state.changed = true;
        }

        if specifiers.is_empty() {
            import_decl.specifiers = None;
            ctx.state.changed = true;
        }
    }
}
