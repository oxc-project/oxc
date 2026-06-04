use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_semantic::Scoping;
use oxc_syntax::scope::ScopeId;
use rustc_hash::FxHashSet;

/// Record a direct `eval(...)` call site in `direct_eval_scopes`.
#[inline]
pub fn record_direct_eval_call(
    scoping: &Scoping,
    call: &CallExpression<'_>,
    direct_eval_scopes: &mut FxHashSet<ScopeId>,
) {
    if !call.optional
        && let Some(ident) = call.callee.get_identifier_reference()
        && ident.name == "eval"
    {
        let scope_id = scoping.get_reference(ident.reference_id()).scope_id();
        direct_eval_scopes.insert(scope_id);
    }
}

/// Collect scopes that lexically contain a direct `eval(...)` call site.
///
/// Unlike `ScopeFlags::DirectEval`, this set is not propagated to ancestors:
/// each entry is only the scope where the call appears.
pub fn collect_direct_eval_scopes<'a>(
    scoping: &Scoping,
    program: &Program<'a>,
) -> FxHashSet<ScopeId> {
    struct Collector<'a> {
        scoping: &'a Scoping,
        direct_eval_scopes: FxHashSet<ScopeId>,
    }

    impl<'a> Visit<'a> for Collector<'a> {
        fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
            record_direct_eval_call(self.scoping, it, &mut self.direct_eval_scopes);
            walk_call_expression(self, it);
        }
    }

    let mut collector = Collector { scoping, direct_eval_scopes: FxHashSet::default() };
    collector.visit_program(program);
    collector.direct_eval_scopes
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

/// Whether any live direct `eval` can observe bindings outside `body_scope_id`.
///
/// Direct `eval` in the function/class body subtree does not block removing an
/// unused declaration: that `eval` never runs if the declaration is never referenced.
pub fn direct_eval_outside_scope_body(
    body_scope_id: ScopeId,
    direct_eval_scopes: &FxHashSet<ScopeId>,
    scoping: &Scoping,
) -> bool {
    direct_eval_scopes
        .iter()
        .any(|&eval_scope| !is_scope_descendant_of(scoping, eval_scope, body_scope_id))
}
