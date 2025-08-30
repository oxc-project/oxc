use oxc_allocator::TakeIn;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue};
use oxc_span::GetSpan;
use rustc_hash::FxHashSet;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_value(decl: &VariableDeclarator<'a>, ctx: &mut Ctx<'a, '_>) {
        let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        let value = if decl.kind.is_var() {
            // Skip constant value inlining for `var` declarations, due to TDZ problems.
            None
        } else {
            decl.init.as_ref().map_or(Some(ConstantValue::Undefined), |e| e.evaluate_value(ctx))
        };
        ctx.init_value(symbol_id, value);
    }

    pub fn take_inlineable_function_declaration(stmt: &mut Statement<'a>, ctx: &mut Ctx<'a, '_>) {
        let Statement::FunctionDeclaration(func) = stmt else { return };
        let Some(id) = &func.id else { return };
        let symbol_id = id.symbol_id();

        let exported = ctx.current_scope_id() == ctx.scoping().root_scope_id()
            && (ctx.source_type().is_script() || {
                ctx.ancestors().any(|ancestor| {
                    ancestor.is_export_named_declaration()
                        || ancestor.is_export_all_declaration()
                        || ancestor.is_export_default_declaration()
                })
            });
        if exported {
            return;
        }

        let mut is_read_symbol_once = false;
        for r in ctx.scoping().get_resolved_references(symbol_id) {
            if r.is_read() {
                if is_read_symbol_once {
                    // Read more than once, cannot inline
                    return;
                }
                is_read_symbol_once = true;
            }
            if r.is_write() {
                // Function is reassigned, cannot inline
                return;
            }
        }
        if !is_read_symbol_once {
            // Never read, will be removed by dead code elimination
            return;
        }

        if !Self::can_safely_inline_function(func, ctx) {
            return;
        }

        let func = func.take_in(ctx.ast.allocator);
        *stmt = ctx.ast.statement_empty(func.span);
        ctx.state.inline_function_declarations.insert(symbol_id, func);
    }

    pub fn inline_identifier_reference(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::Identifier(ident) = expr else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return;
        };
        // Skip if there are write references.
        if symbol_value.write_references_count > 0 {
            return;
        }
        if symbol_value.for_statement_init {
            return;
        }
        let Some(cv) = &symbol_value.initialized_constant else { return };
        if symbol_value.read_references_count == 1
            || match cv {
                ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
                ConstantValue::BigInt(_) => false,
                ConstantValue::String(s) => s.len() <= 3,
                ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
            }
        {
            *expr = ctx.value_to_expr(expr.span(), cv.clone());
            ctx.state.changed = true;
        }
    }

    pub fn inline_function_declaration_reference(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::Identifier(ident) = expr else { return };
        let Some(reference_id) = ident.reference_id.get() else { return };
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
            return;
        };
        let Some(mut func_decl) = ctx.state.inline_function_declarations.remove(&symbol_id) else {
            return;
        };
        let current_scope_id = ctx.current_scope_id();
        ctx.scoping_mut().change_scope_parent_id(func_decl.scope_id(), Some(current_scope_id));
        func_decl.r#type = FunctionType::FunctionExpression;
        if !ctx.options().keep_names.function {
            func_decl.id = None;
        }
        *expr = Expression::FunctionExpression(ctx.alloc(func_decl));
        ctx.state.changed = true;
    }

    /// Check if a function can be safely inlined without variable name conflicts
    fn can_safely_inline_function(function_decl: &Function<'a>, ctx: &Ctx<'a, '_>) -> bool {
        struct VariableCollector<'a, 'ctx> {
            ctx: &'ctx Ctx<'a, 'ctx>,
            external_variables: FxHashSet<String>,
            function_scope_id: oxc_semantic::ScopeId,
        }

        impl<'a> Visit<'a> for VariableCollector<'a, '_> {
            fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
                // Check references inside the function
                if let Some(reference_id) = ident.reference_id.get() {
                    if let Some(symbol_id) =
                        self.ctx.scoping().get_reference(reference_id).symbol_id()
                    {
                        let symbol_scope_id = self.ctx.scoping().symbol_scope_id(symbol_id);

                        // If this reference is to a variable from outside the function
                        if self.is_external_scope(symbol_scope_id) {
                            self.external_variables.insert(ident.name.to_string());
                        }
                    }
                }
            }
        }

        impl VariableCollector<'_, '_> {
            fn is_external_scope(&self, scope_id: oxc_semantic::ScopeId) -> bool {
                // Check if this scope is outside the function being inlined
                // For now, we consider any scope that's not the target scope as external
                self.ctx
                    .scoping()
                    .scope_ancestors(self.function_scope_id)
                    .skip(1)
                    .any(|ancestor_id| ancestor_id == scope_id)
            }
        }

        // For now, we'll be conservative and only allow inlining of simple functions
        // that don't reference external variables, to avoid complex scope analysis
        let mut collector = VariableCollector {
            ctx,
            external_variables: FxHashSet::default(),
            function_scope_id: function_decl.scope_id(),
        };

        collector.visit_function(function_decl, oxc_syntax::scope::ScopeFlags::empty());

        // For now, allow inlining if no external variables are referenced
        // This is conservative but safe
        collector.external_variables.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        CompressOptions,
        tester::{test_options, test_same_options},
    };

    #[test]
    fn r#const() {
        let options = CompressOptions::smallest();
        test_options("const foo = 1; log(foo)", "log(1)", &options);
        test_options("export const foo = 1; log(foo)", "export const foo = 1; log(1)", &options);

        test_options("let foo = 1; log(foo)", "log(1)", &options);
        test_options("export let foo = 1; log(foo)", "export let foo = 1; log(1)", &options);
    }

    #[test]
    fn small_value() {
        let options = CompressOptions::smallest();
        test_options("const foo = 999; log(foo), log(foo)", "log(999), log(999)", &options);
        test_options("const foo = -99; log(foo), log(foo)", "log(-99), log(-99)", &options);
        test_same_options("const foo = 1000; log(foo), log(foo)", &options);
        test_same_options("const foo = -100; log(foo), log(foo)", &options);

        test_same_options("const foo = 0n; log(foo), log(foo)", &options);

        test_options("const foo = 'aaa'; log(foo), log(foo)", "log('aaa'), log('aaa')", &options);
        test_same_options("const foo = 'aaaa'; log(foo), log(foo)", &options);

        test_options("const foo = true; log(foo), log(foo)", "log(!0), log(!0)", &options);
        test_options("const foo = false; log(foo), log(foo)", "log(!1), log(!1)", &options);
        test_options(
            "const foo = undefined; log(foo), log(foo)",
            "log(void 0), log(void 0)",
            &options,
        );
        test_options("const foo = null; log(foo), log(foo)", "log(null), log(null)", &options);

        test_options(
            r#"
            const o = 'o';
            const d = 'd';
            const boolean = false;
            var frag = `<p autocapitalize="${`w${o}r${d}s`}" contenteditable="${boolean}"/>`;
            console.log(frag);
            "#,
            r#"console.log('<p autocapitalize="words" contenteditable="false"/>');"#,
            &options,
        );
    }

    #[test]
    fn function_inlining() {
        let options = CompressOptions::smallest();

        // Simple function used once should be inlined
        test_options(
            "function foo(a,b,c) { return a + b + c; } console.log(foo(1,2,3))",
            "console.log(function(a,b,c){return a + b + c}(1,2,3))",
            &options,
        );

        // Function used multiple times should not be inlined (but may be formatted)
        test_options(
            "function foo(a,b,c) { return a + b + c; } console.log(foo(1,2,3)); console.log(foo(4,5,6))",
            "function foo(a,b,c){return a + b + c}console.log(foo(1,2,3)),console.log(foo(4,5,6))",
            &options,
        );

        // Function that is reassigned should not be inlined (but may be formatted)
        test_options(
            "function foo(a,b,c) { return a + b + c; } foo = bar; console.log(foo(1,2,3))",
            "function foo(a,b,c){return a + b + c}foo = bar,console.log(foo(1,2,3))",
            &options,
        );

        // Function parameter shadowing outer variable is safe to inline
        test_options(
            "var a = 1; function foo(a) { return a + 1; } console.log(foo(2))",
            "console.log(function(a){return a + 1}(2))",
            &options,
        );

        // Function accessing outer scope variable (no parameter shadowing) should not be inlined for safety
        test_same_options(
            "var x = 1; function foo(y) { return x + y; } console.log(foo(2))",
            &options,
        );
    }
}
