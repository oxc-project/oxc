use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression, walk::walk_statement};
use oxc_semantic::{IsGlobalReference, Scoping};
use oxc_syntax::scope::ScopeId;
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
    pub unused_declaration_body_scopes: FxHashSet<ScopeId>,
}

fn try_record_unused_declaration_body(
    scopes: &mut FxHashSet<ScopeId>,
    scoping: &Scoping,
    id: Option<&BindingIdentifier<'_>>,
    scope_id: ScopeId,
) {
    if let Some(id) = id
        && let Some(symbol_id) = id.symbol_id.get()
        && scoping.symbol_is_unused(symbol_id)
    {
        scopes.insert(scope_id);
    }
}

/// Collect direct-eval scopes and unused named declaration body scopes in one walk.
///
/// Only `function` / `class` declarations (including exported) are recorded — not named
/// function/class expressions, whose bodies may still run when the expression is evaluated.
pub fn collect_prepass_data<'a>(scoping: &Scoping, program: &Program<'a>) -> PrepassData {
    struct Collector<'a> {
        scoping: &'a Scoping,
        direct_eval_scopes: FxHashSet<ScopeId>,
        unused_declaration_body_scopes: FxHashSet<ScopeId>,
    }

    impl<'a> Visit<'a> for Collector<'a> {
        fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
            record_direct_eval_call(self.scoping, it, &mut self.direct_eval_scopes);
            walk_call_expression(self, it);
        }

        fn visit_statement(&mut self, stmt: &Statement<'a>) {
            match stmt {
                Statement::FunctionDeclaration(f) => {
                    try_record_unused_declaration_body(
                        &mut self.unused_declaration_body_scopes,
                        self.scoping,
                        f.id.as_ref(),
                        f.scope_id(),
                    );
                }
                Statement::ClassDeclaration(c) => {
                    try_record_unused_declaration_body(
                        &mut self.unused_declaration_body_scopes,
                        self.scoping,
                        c.id.as_ref(),
                        c.scope_id(),
                    );
                }
                Statement::ExportNamedDeclaration(exp) => {
                    if let Some(decl) = &exp.declaration {
                        match decl {
                            Declaration::FunctionDeclaration(f) => {
                                try_record_unused_declaration_body(
                                    &mut self.unused_declaration_body_scopes,
                                    self.scoping,
                                    f.id.as_ref(),
                                    f.scope_id(),
                                );
                            }
                            Declaration::ClassDeclaration(c) => {
                                try_record_unused_declaration_body(
                                    &mut self.unused_declaration_body_scopes,
                                    self.scoping,
                                    c.id.as_ref(),
                                    c.scope_id(),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                Statement::ExportDefaultDeclaration(exp) => match &exp.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                        try_record_unused_declaration_body(
                            &mut self.unused_declaration_body_scopes,
                            self.scoping,
                            f.id.as_ref(),
                            f.scope_id(),
                        );
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(c) => {
                        try_record_unused_declaration_body(
                            &mut self.unused_declaration_body_scopes,
                            self.scoping,
                            c.id.as_ref(),
                            c.scope_id(),
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
            walk_statement(self, stmt);
        }
    }

    let mut collector = Collector {
        scoping,
        direct_eval_scopes: FxHashSet::default(),
        unused_declaration_body_scopes: FxHashSet::default(),
    };
    collector.visit_program(program);
    PrepassData {
        direct_eval_scopes: collector.direct_eval_scopes,
        unused_declaration_body_scopes: collector.unused_declaration_body_scopes,
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
/// declaration at `body_scope_id`.
///
/// Eval inside another unused function/class **declaration** body never runs, so it must
/// not block removal of the current unused declaration.
pub fn direct_eval_blocks_unused_declaration_removal(
    body_scope_id: ScopeId,
    direct_eval_scopes: &FxHashSet<ScopeId>,
    unused_declaration_body_scopes: &FxHashSet<ScopeId>,
    scoping: &Scoping,
) -> bool {
    direct_eval_scopes.iter().any(|&eval_scope| {
        if is_scope_descendant_of(scoping, eval_scope, body_scope_id) {
            return false;
        }
        !unused_declaration_body_scopes.iter().any(|&other_scope| {
            other_scope != body_scope_id && is_scope_descendant_of(scoping, eval_scope, other_scope)
        })
    })
}
