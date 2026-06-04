use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression, walk::walk_class, walk::walk_function};
use oxc_semantic::{IsGlobalReference, Scoping};
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};
use rustc_hash::FxHashSet;

/// Whether `call` is a direct call to the global `eval` binding.
#[inline]
pub fn is_direct_eval_call(scoping: &Scoping, call: &CallExpression<'_>) -> bool {
    if call.optional {
        return false;
    }
    let Some(ident) = call.callee.get_identifier_reference() else {
        return false;
    };
    ident.name == "eval" && ident.is_global_reference(scoping)
}

/// Record a direct `eval(...)` call site in `direct_eval_scopes`.
#[inline]
pub fn record_direct_eval_call(
    scoping: &Scoping,
    call: &CallExpression<'_>,
    direct_eval_scopes: &mut FxHashSet<ScopeId>,
) {
    if is_direct_eval_call(scoping, call) {
        let ident = call.callee.get_identifier_reference().unwrap();
        let scope_id = scoping.get_reference(ident.reference_id()).scope_id();
        direct_eval_scopes.insert(scope_id);
    }
}

/// Pre-pass data for unused-declaration removal and `refresh_direct_eval_flags`.
pub struct PrepassData {
    pub direct_eval_scopes: FxHashSet<ScopeId>,
    pub named_declaration_body_scopes: Vec<(SymbolId, ScopeId)>,
}

/// Collect direct-eval scopes and named function/class body scopes in one walk.
pub fn collect_prepass_data<'a>(scoping: &Scoping, program: &Program<'a>) -> PrepassData {
    struct Collector<'a> {
        scoping: &'a Scoping,
        direct_eval_scopes: FxHashSet<ScopeId>,
        named_declaration_body_scopes: Vec<(SymbolId, ScopeId)>,
    }

    impl<'a> Visit<'a> for Collector<'a> {
        fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
            record_direct_eval_call(self.scoping, it, &mut self.direct_eval_scopes);
            walk_call_expression(self, it);
        }

        fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
            if let Some(id) = &it.id
                && let Some(symbol_id) = id.symbol_id.get()
            {
                self.named_declaration_body_scopes.push((symbol_id, it.scope_id()));
            }
            walk_function(self, it, flags);
        }

        fn visit_class(&mut self, it: &Class<'a>) {
            if let Some(id) = &it.id
                && let Some(symbol_id) = id.symbol_id.get()
            {
                self.named_declaration_body_scopes.push((symbol_id, it.scope_id()));
            }
            walk_class(self, it);
        }
    }

    let mut collector = Collector {
        scoping,
        direct_eval_scopes: FxHashSet::default(),
        named_declaration_body_scopes: Vec::new(),
    };
    collector.visit_program(program);
    PrepassData {
        direct_eval_scopes: collector.direct_eval_scopes,
        named_declaration_body_scopes: collector.named_declaration_body_scopes,
    }
}

/// Whether `scope_id` is `ancestor_id` or nested inside it.
#[inline]
pub fn is_scope_descendant_of(scoping: &Scoping, scope_id: ScopeId, ancestor_id: ScopeId) -> bool {
    let mut current = Some(scope_id);
    while let Some(id) = current {
        if id == ancestor_id {
            return true;
        }
        current = scoping.scope_parent_id(id);
    }
    false
}

/// Whether any live direct `eval` outside `body_scope_id` blocks removing the unused
/// declaration `symbol_id`.
///
/// Eval inside another unused function/class body never runs, so it must not block removal
/// of the current unused declaration.
pub fn direct_eval_blocks_unused_declaration_removal(
    body_scope_id: ScopeId,
    symbol_id: SymbolId,
    direct_eval_scopes: &FxHashSet<ScopeId>,
    named_declaration_body_scopes: &[(SymbolId, ScopeId)],
    scoping: &Scoping,
) -> bool {
    direct_eval_scopes.iter().any(|&eval_scope| {
        if is_scope_descendant_of(scoping, eval_scope, body_scope_id) {
            return false;
        }
        !named_declaration_body_scopes.iter().any(|&(other_sym, other_scope)| {
            other_sym != symbol_id
                && scoping.symbol_is_unused(other_sym)
                && is_scope_descendant_of(scoping, eval_scope, other_scope)
        })
    })
}
