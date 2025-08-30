use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use oxc_allocator::{Allocator, Box, CloneIn, Vec};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, VisitMut, walk_mut};
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue};
use oxc_semantic::{ScopeId, Scoping, SymbolId};
use oxc_span::GetSpan;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

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

    pub fn inline_function_declarations(stmts: &mut Vec<Statement<'a>>, ctx: &mut Ctx<'a, '_>) {
        let (func_stmts, non_func_stmts) =
            stmts.iter_mut().partition::<std::vec::Vec<_>, _>(|stmt| {
                matches!(stmt, Statement::FunctionDeclaration(_))
            });
        let func_stmts = func_stmts
            .into_iter()
            .map(|stmt| {
                let Statement::FunctionDeclaration(func) = stmt else { unreachable!() };
                Rc::new(RefCell::new(func))
            })
            .collect::<std::vec::Vec<_>>();

        let inlineable_functions = func_stmts
            .clone()
            .into_iter()
            .filter_map(|func| {
                let func2 = Rc::clone(&func);
                let func2 = func2.borrow();
                let id = func2.id.as_ref().expect("FunctionDeclaration must have id");
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
                    return None;
                }

                let mut is_read_symbol_once = false;
                for r in ctx.scoping().get_resolved_references(symbol_id) {
                    if r.is_read() {
                        if is_read_symbol_once {
                            // Read more than once, cannot inline
                            return None;
                        }
                        is_read_symbol_once = true;
                    }
                    if r.is_write() {
                        // Function is reassigned, cannot inline
                        return None;
                    }
                }
                if !is_read_symbol_once {
                    // Never read, will be removed by dead code elimination
                    return None;
                }

                if !Self::can_safely_inline_function(&func.borrow(), ctx) {
                    return None;
                }

                Some((symbol_id, func))
            })
            .collect::<FxHashMap<_, _>>();

        let mut inliner = FunctionDeclarationInliner::new(inlineable_functions, ctx);
        for stmt in func_stmts {
            let mut func = stmt.borrow_mut();
            let scope_id = func.scope_id();
            inliner.enter_scope(ctx.scoping().scope_flags(scope_id), &Cell::new(Some(scope_id)));
            inliner.visit_function_body(
                func.body.as_mut().expect("FunctionDeclaration must have body"),
            );
        }
        let scope_id = ctx.current_scope_id();
        inliner.enter_scope(ctx.scoping().scope_flags(scope_id), &Cell::new(Some(scope_id)));
        for stmt in non_func_stmts {
            inliner.visit_statement(stmt);
        }
        let (inlined_symbols, parent_scope_id_changes) = inliner.close();
        if inlined_symbols.is_empty() {
            return;
        }

        for (scope_id, new_parent_id) in parent_scope_id_changes {
            ctx.scoping_mut().change_scope_parent_id(scope_id, Some(new_parent_id));
        }

        for stmt in stmts {
            if let Statement::FunctionDeclaration(func) = stmt
                && let Some(id) = &func.id
                && inlined_symbols.contains(&id.symbol_id())
            {
                *stmt = ctx.ast.statement_empty(func.span);
            }
        }
        ctx.state.changed = true;
    }

    /// Check if a function can be safely inlined without variable name conflicts
    fn can_safely_inline_function(function_decl: &Function<'a>, ctx: &Ctx<'a, '_>) -> bool {
        struct VariableCollector<'b> {
            scoping: &'b Scoping,
            reference_external_variables: bool,
            function_scope_id: oxc_semantic::ScopeId,
        }

        impl<'b> VariableCollector<'b> {
            pub fn new(scoping: &'b Scoping, function_scope_id: oxc_semantic::ScopeId) -> Self {
                Self { scoping, reference_external_variables: false, function_scope_id }
            }

            pub fn close(self) -> bool {
                self.reference_external_variables
            }

            fn is_external_scope(&self, scope_id: oxc_semantic::ScopeId) -> bool {
                // Check if this scope is outside the function being inlined
                // For now, we consider any scope that's not the target scope as external
                self.scoping
                    .scope_ancestors(self.function_scope_id)
                    .skip(1)
                    .any(|ancestor_id| ancestor_id == scope_id)
            }
        }

        impl<'a> Visit<'a> for VariableCollector<'_> {
            fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
                // Check references inside the function
                if let Some(reference_id) = ident.reference_id.get() {
                    if let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id() {
                        let symbol_scope_id = self.scoping.symbol_scope_id(symbol_id);

                        if self.is_external_scope(symbol_scope_id) {
                            self.reference_external_variables = true;
                        }
                    }
                }
            }
        }

        // For now, we'll be conservative and only allow inlining of simple functions that don't reference external variables
        let mut collector = VariableCollector::new(ctx.scoping(), function_decl.scope_id());
        collector.visit_function(function_decl, oxc_syntax::scope::ScopeFlags::empty());
        !collector.close()
    }
}

struct FunctionDeclarationInliner<'a, 'b> {
    inlineable_functions: FxHashMap<SymbolId, Rc<RefCell<&'b mut Box<'a, Function<'a>>>>>,
    inlined_symbols: FxHashSet<SymbolId>,
    scoping: &'b Scoping,
    allocator: &'a Allocator,
    current_scope_id: ScopeId,
    parent_scope_id_changes: FxHashMap<ScopeId, ScopeId>,
}

impl<'a, 'b> FunctionDeclarationInliner<'a, 'b> {
    fn new(
        inlineable_functions: FxHashMap<SymbolId, Rc<RefCell<&'b mut Box<'a, Function<'a>>>>>,
        ctx: &'b Ctx<'a, 'b>,
    ) -> Self {
        let inlined_symbols =
            FxHashSet::with_capacity_and_hasher(inlineable_functions.len(), FxBuildHasher);
        Self {
            inlineable_functions,
            inlined_symbols,
            scoping: ctx.scoping(),
            allocator: ctx.ast.allocator,
            current_scope_id: ctx.current_scope_id(),
            parent_scope_id_changes: FxHashMap::default(),
        }
    }

    fn close(self) -> (FxHashSet<SymbolId>, FxHashMap<ScopeId, ScopeId>) {
        (self.inlined_symbols, self.parent_scope_id_changes)
    }
}

impl<'a> VisitMut<'a> for FunctionDeclarationInliner<'a, '_> {
    fn enter_scope(&mut self, _flags: oxc_semantic::ScopeFlags, scope_id: &Cell<Option<ScopeId>>) {
        self.current_scope_id = scope_id.get().unwrap();
    }

    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        let Expression::Identifier(ident) = it else {
            walk_mut::walk_expression(self, it);
            return;
        };

        let Some(reference_id) = ident.reference_id.get() else { return };
        let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id() else {
            return;
        };
        let Some(func_decl) = self.inlineable_functions.get(&symbol_id) else {
            return;
        };
        let Ok(func_decl) = func_decl.try_borrow() else {
            // Self-reference
            return;
        };
        let mut func_decl = func_decl.clone_in_with_semantic_ids(self.allocator);

        let current_scope_id = self.current_scope_id;
        func_decl.r#type = FunctionType::FunctionExpression;
        self.parent_scope_id_changes.insert(func_decl.scope_id(), current_scope_id);
        *it = Expression::FunctionExpression(func_decl);
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
