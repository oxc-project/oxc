mod convert_to_dotted_properties;
mod fold_constants;
mod inline;
mod minimize_conditional_expression;
mod minimize_conditions;
mod minimize_expression_in_boolean_context;
mod minimize_for_statement;
mod minimize_if_statement;
mod minimize_logical_expression;
mod minimize_not_expression;
mod minimize_statements;
mod normalize;
mod remove_dead_code;
mod remove_unused_declaration;
mod remove_unused_expression;
mod remove_unused_private_members;
mod replace_known_methods;
mod substitute_alternate_syntax;

use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_semantic::Scoping;
use oxc_syntax::{
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};
use rustc_hash::FxHashSet;

use oxc_allocator::{ArenaVec, BitSet, GetAllocator};
use oxc_ast::ast::*;

use crate::{
    ReusableTraverseCtx, Traverse, TraverseCtx, minifier_traverse::traverse_mut_with_ctx,
    symbol_liveness, traverse_context::as_direct_eval_call,
};

pub use self::normalize::{Normalize, NormalizeOptions};

/// Stateless peephole optimizer. The `dce` flag, the `mutated` signal, and
/// the per-pass `PassDirty` accumulator all live on `MinifierState`.
pub struct PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn run_once(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        traverse_mut_with_ctx(self, program, ctx);
    }

    pub fn commutative_pair<'x, A, F, G, RetF: 'x, RetG: 'x>(
        pair: (&'x A, &'x A),
        check_a: F,
        check_b: G,
    ) -> Option<(RetF, RetG)>
    where
        F: Fn(&'x A) -> Option<RetF>,
        G: Fn(&'x A) -> Option<RetG>,
    {
        match check_a(pair.0) {
            Some(a) => {
                if let Some(b) = check_b(pair.1) {
                    return Some((a, b));
                }
            }
            _ => {
                if let Some(a) = check_a(pair.1)
                    && let Some(b) = check_b(pair.0)
                {
                    return Some((a, b));
                }
            }
        }
        None
    }

    /// A body-level statement is "declarative" if executing it cannot run user
    /// code that observes a subsequent hoisted `var x = <literal>;` as
    /// `undefined`. Module loaders (`import`, `export * from`, `export … from`)
    /// can evaluate foreign modules but only observe our bindings on an actual
    /// cycle — handled at program scope by starting the root prelude unsafe when
    /// the module has loaders (see `enter_program`).
    /// Type-only declarations (`type`, `interface`) are erased and never run.
    fn is_declarative_body_statement(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::EmptyStatement(_)
            | Statement::ImportDeclaration(_)
            | Statement::ExportAllDeclaration(_) => true,
            // `export { foo }`, `export { foo } from './x'`, `export type T = …` —
            // no executable code at the statement itself. The cyclic-eval hazard
            // from a `from` source is gated separately at program scope (see
            // `enter_program`).
            Statement::ExportNamedDeclaration(e) => {
                e.declaration.as_ref().is_none_or(Self::is_declarative_declaration)
            }
            // `export default function() {}` is hoisted; `export default <expr>`
            // or `export default class C extends … {}` runs user code.
            Statement::ExportDefaultDeclaration(e) => {
                matches!(&e.declaration, ExportDefaultDeclarationKind::FunctionDeclaration(_))
            }
            // Bare declarations route through the shared classifier; anything else
            // (blocks, expressions, control flow) can run user code.
            _ => stmt.as_declaration().is_some_and(Self::is_declarative_declaration),
        }
    }

    /// A `Declaration` runs no user code at evaluation: function/type/interface
    /// declarations are inert, and a `var`/`let`/`const` is declarative only when
    /// every declarator is a simple binding with a literal (or no) initializer.
    /// Classes, enums, and TS modules run user code, so they are not declarative.
    fn is_declarative_declaration(decl: &Declaration<'a>) -> bool {
        match decl {
            Declaration::FunctionDeclaration(_)
            | Declaration::TSTypeAliasDeclaration(_)
            | Declaration::TSInterfaceDeclaration(_) => true,
            Declaration::VariableDeclaration(decl) => {
                Self::is_declarative_variable_declaration(decl)
            }
            _ => false,
        }
    }

    /// A `VariableDeclaration` is declarative when every declarator is a simple
    /// `BindingIdentifier` (no destructuring / defaults / computed keys, all of
    /// which can run user code) with either no initializer or a primitive
    /// literal initializer.
    fn is_declarative_variable_declaration(decl: &VariableDeclaration<'a>) -> bool {
        decl.declarations.iter().all(Self::is_declarative_variable_declarator)
    }

    /// Note: only AST `Literal`s qualify. Constant-but-non-literal initializers
    /// (`-1`, `void 0`, `1 + 2`) run no user code either, but conservatively end
    /// the prelude here — a missed optimization, never a correctness risk.
    fn is_declarative_variable_declarator(decl: &VariableDeclarator<'a>) -> bool {
        matches!(decl.id, BindingPattern::BindingIdentifier(_))
            && decl.init.as_ref().is_none_or(Expression::is_literal)
    }

    /// Mark the current function/program body as no longer in its declarative
    /// prelude. No-op if the flag is already set, or if `current_scope_id` is
    /// some inner scope (a block/for/etc.) — those don't end the prelude.
    fn mark_current_body_unsafe(ctx: &mut TraverseCtx<'a>) {
        let &(body_scope, body_unsafe) = ctx.state.body_unsafe_stack.last();
        if !body_unsafe && body_scope == ctx.current_scope_id() {
            ctx.state.body_unsafe_stack.last_mut().1 = true;
        }
    }

    /// Checks if a member expression's base object may be mutated.
    ///
    /// This is used to prevent incorrect transformations like:
    /// `x.y || (x = {}, x.y = 3)` → `x.y ||= (x = {}, 3)`
    ///
    /// The `||=` operator evaluates `x.y` (capturing `x`) before the RHS reassigns `x`,
    /// which would change the semantics.
    pub fn member_object_may_be_mutated(
        assignment_target: &AssignmentTarget<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let object = match assignment_target {
            AssignmentTarget::ComputedMemberExpression(member_expr) => &member_expr.object,
            AssignmentTarget::PrivateFieldExpression(member_expr) => &member_expr.object,
            AssignmentTarget::StaticMemberExpression(member_expr) => &member_expr.object,
            _ => return false,
        };

        Self::is_expression_that_reference_may_change(object, ctx)
    }

    /// Checks if an expression's reference may change due to mutation.
    ///
    /// Returns `true` if the expression references a symbol that may be mutated,
    /// or if the expression is not a simple identifier/this reference.
    pub fn is_expression_that_reference_may_change(
        expr: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::Identifier(id) => {
                if let Some(symbol_id) = ctx.scoping().get_reference(id.reference_id()).symbol_id()
                {
                    Self::is_symbol_mutated(symbol_id, ctx)
                } else {
                    true
                }
            }
            Expression::ThisExpression(_) => false,
            _ => true,
        }
    }

    /// Check if a symbol is mutated, using the O(1) cached `write_references_count`
    /// from `SymbolValue` when available, falling back to the O(num_refs) scan in
    /// `Scoping::symbol_is_mutated` for symbols without cached values.
    ///
    /// Only variable declarators have cached values (populated during
    /// `exit_variable_declarator` → `init_symbol_value`); function declarations
    /// and other binding kinds still take the fallback path.
    fn is_symbol_mutated(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        if let Some(sv) = ctx.state.symbol_values.get_symbol_value(symbol_id) {
            sv.write_references_count > 0
        } else {
            ctx.scoping().symbol_is_mutated(symbol_id)
        }
    }

    /// True if the scope chain from `read_scope` up to (excluding) `body_scope`
    /// crosses a function boundary — i.e. the read is in a closure relative to
    /// `body_scope`. Async/generator/arrow scopes are all `Function`.
    fn read_crosses_function_boundary(
        read_scope: ScopeId,
        body_scope: ScopeId,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let scoping = ctx.scoping();
        scoping
            .scope_ancestors(read_scope)
            .take_while(|&s| s != body_scope)
            .any(|s| scoping.scope_flags(s).is_function())
    }

    /// Refresh `ScopeFlags::DirectEval` from live direct-eval call sites.
    ///
    /// `direct_eval_scopes` lists scopes that still contain a direct `eval(...)` call.
    /// Clears `DirectEval` from every scope, then re-propagates from each scope in the
    /// set up to the root.
    ///
    /// Skipping this leaves `DirectEval` set on scopes whose only eval call was just
    /// DCE'd, which keeps unused-declaration removal conservative until reparse.
    fn refresh_direct_eval_flags(scoping: &mut Scoping, direct_eval_scopes: &FxHashSet<ScopeId>) {
        // Semantic propagates `DirectEval` to the root, so an empty live set plus a
        // clean root means no scope has the flag — nothing to clear or set.
        if direct_eval_scopes.is_empty() && !scoping.root_scope_flags().contains_direct_eval() {
            return;
        }

        for index in 0..scoping.scopes_len() {
            scoping.scope_flags_mut(ScopeId::from_usize(index)).remove(ScopeFlags::DirectEval);
        }

        for &scope_id in direct_eval_scopes {
            let mut ancestor = Some(scope_id);
            while let Some(scope_id) = ancestor {
                let flags = scoping.scope_flags_mut(scope_id);
                // An earlier iteration already flagged this chain; stop walking up.
                if flags.contains_direct_eval() {
                    break;
                }
                flags.insert(ScopeFlags::DirectEval);
                ancestor = scoping.scope_parent_id(scope_id);
            }
        }
    }

    /// Debug-only guard for the incremental scoping refresh: every reference
    /// marked dead in `dead_refs` (see [`crate::state::PassDirty::dead_refs`]) must really
    /// be gone from the live program — pruning a still-live reference is the
    /// unsafe direction that produces incorrect output.
    ///
    /// Walks the live program once per dirty pass in debug builds only, so
    /// the entire unit-test and `cargo coverage -- minifier` corpus doubles
    /// as an over-prune detector at zero release cost.
    #[cfg(debug_assertions)]
    fn debug_assert_no_over_prune(program: &Program<'a>, dead_refs: &BitSet<'_>) {
        struct OverPruneCheck<'b, 'c> {
            dead_refs: &'b BitSet<'c>,
        }
        impl<'a> Visit<'a> for OverPruneCheck<'_, '_> {
            fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
                let Some(reference_id) = it.reference_id.get() else { return };
                let idx = reference_id.index();
                // `contains` is false past capacity — the capacity guard
                // (see `PassDirty::dead_refs`).
                assert!(
                    !self.dead_refs.contains(idx),
                    "incremental scoping over-prune: reference {idx} is marked dead but still \
                     appears in the live program",
                );
            }
        }
        OverPruneCheck { dead_refs }.visit_program(program);
    }

    /// Debug-only converse of [`Self::debug_assert_no_over_prune`], run once
    /// by the `Compressor` driver after the fixed-point loop: every reference
    /// that existed when the loop began and is still in a symbol's
    /// resolved-references list must appear in the live program. A violation
    /// means a site discarded a subtree without routing it through a
    /// `drop_*` / `replace_*` helper (the leak direction: stale references
    /// silently block optimizations), or the caller passed a `scoping`
    /// already inconsistent with `program` (see the precondition on
    /// `Compressor::build_with_scoping`).
    ///
    /// References minted during the loop (`idx >= initial_references_len`)
    /// are exempt: the capacity guard deliberately leaves a same-pass
    /// mint-then-drop unmarked (see `PassDirty::dead_refs`).
    ///
    /// Together with the over-prune guard this closes both failure
    /// directions of the drop-helper convention across the whole unit-test
    /// and conformance corpus, at zero release cost.
    #[cfg(debug_assertions)]
    pub(crate) fn debug_assert_no_under_prune(
        program: &Program<'a>,
        ctx: &TraverseCtx<'a>,
        initial_references_len: usize,
    ) {
        struct LiveRefCollector<'b, 'c> {
            live: &'b mut BitSet<'c>,
        }
        impl<'a> Visit<'a> for LiveRefCollector<'_, '_> {
            fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
                if let Some(reference_id) = it.reference_id.get() {
                    let idx = reference_id.index();
                    if idx < self.live.capacity() {
                        self.live.set_bit(idx);
                    }
                }
            }
        }
        let mut live = BitSet::new_in(initial_references_len, ctx.allocator());
        LiveRefCollector { live: &mut live }.visit_program(program);
        for reference_ids in ctx.scoping().resolved_references() {
            for reference_id in reference_ids {
                let idx = reference_id.index();
                assert!(
                    idx >= initial_references_len || live.has_bit(idx),
                    "incremental scoping under-prune: reference {idx} is still in a symbol's \
                     resolved-references list but its node is gone from the program — a drop \
                     site bypassed the `drop_*` / `replace_*` helpers, or the caller passed a \
                     `scoping` inconsistent with `program`",
                );
            }
        }
    }

    /// Debug-only guard for the gated direct-eval refresh: every live direct
    /// `eval(...)` call must already have `ScopeFlags::DirectEval` on its
    /// reference's recorded scope and every ancestor (the exact postcondition
    /// of [`Self::refresh_direct_eval_flags`]). The gate
    /// (`PassDirty::eval_dropped`) only re-derives flags when an eval call is
    /// *dropped*, so it is sound only while no pass *forms* a new direct eval
    /// call (e.g. by moving `eval` into callee position). Stale-SET flags are
    /// merely conservative and not checked; only the missing direction is
    /// unsafe.
    ///
    /// Locally-bound `eval` callees are exempt: `remove_sequence_expression`
    /// deliberately forms them (`var eval; (0, eval)()` -> `var eval; eval()`
    /// — `should_keep_indirect_access` only protects the *global* `eval`),
    /// banking on a local binding named `eval` not holding the real `eval`.
    /// Under that same assumption the missing flag is inert; a later refresh
    /// may still set it (the name-based collector), which is the allowed
    /// conservative direction.
    ///
    /// Allocation-free by design: asserts inline per call during the walk so
    /// the allocation-tracking task (debug assertions on) sees no sys-allocs.
    /// The walk itself is skipped when the program has no unresolved `eval`
    /// at all: the check only fires on unresolved (global) callees, and every
    /// live unresolved reference's name is a key in
    /// `root_unresolved_references` (populated at build, appended on in-loop
    /// mints, deliberately never pruned in-loop) — a conservative superset
    /// that can never skip a checkable call.
    #[cfg(debug_assertions)]
    fn debug_assert_no_stale_direct_eval(program: &Program<'a>, scoping: &Scoping) {
        struct DirectEvalFlagCheck<'s> {
            scoping: &'s Scoping,
        }
        impl<'a> Visit<'a> for DirectEvalFlagCheck<'_> {
            fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
                if let Some(ident) = as_direct_eval_call(it)
                    && let Some(reference_id) = ident.reference_id.get()
                {
                    let reference = self.scoping.get_reference(reference_id);
                    // No symbol = unresolved = the global `eval` (see above).
                    if reference.symbol_id().is_none() {
                        // Same scope derivation as `LiveDirectEvalCollector` —
                        // producer, consumer, and this check must agree.
                        for scope_id in self.scoping.scope_ancestors(reference.scope_id()) {
                            assert!(
                                self.scoping.scope_flags(scope_id).contains_direct_eval(),
                                "stale direct-eval flags: scope {scope_id:?} is missing \
                                 `ScopeFlags::DirectEval` for a live direct `eval(...)` call — a \
                                 pass formed a new direct eval call without dropping one — see \
                                 `PassDirty::eval_dropped`",
                            );
                        }
                    }
                }
                walk_call_expression(self, it);
            }
        }
        if !scoping.root_unresolved_references().contains_key("eval") {
            return;
        }
        DirectEvalFlagCheck { scoping }.visit_program(program);
    }

    /// Consume the `PassDirty` accumulator: batch-prune the dead resolved
    /// references from scoping, refresh direct-eval flags if an `eval(...)`
    /// call was dropped, and re-initialize the accumulator.
    ///
    /// [`Self::end_pass`] calls this after `Normalize` (so the fixed-point
    /// loop starts against already-pruned scoping and Normalize's drops cost
    /// no extra peephole pass) and after every peephole pass — quiet ones
    /// included, where every step below is a cheap no-op.
    fn flush_pass_dirty(program: &Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let had_dead = !ctx.state.dirty.dead_refs.is_empty();

        // (1) Resolved references — direct consumption, no walk.
        //     Dirty data is built by `replace_*` / `drop_*` helpers as
        //     subtrees are removed and is consumed here in one batch.
        if had_dead {
            // Debug-only guard: every reference we are about to prune must
            // really be gone from the live program (see the helper).
            #[cfg(debug_assertions)]
            Self::debug_assert_no_over_prune(program, &ctx.state.dirty.dead_refs);

            // Disjoint-field borrows: `state.dirty` and `scoping` don't overlap.
            ctx.scoping
                .scoping_mut()
                .retain_resolved_references_excluding(&ctx.state.dirty.dead_refs);
        }

        // (2) Direct-eval — gated full walk only when an eval was dropped.
        if ctx.state.dirty.eval_dropped {
            let scoping = ctx.scoping();
            let mut live = LiveDirectEvalCollector::new(scoping);
            live.visit_program(program);
            let scopes = live.scopes;
            Self::refresh_direct_eval_flags(ctx.scoping_mut(), &scopes);
        }
        // Debug-only converse of the gate: no pass may have FORMED a new
        // direct eval call without dropping one (see the helper).
        #[cfg(debug_assertions)]
        Self::debug_assert_no_stale_direct_eval(program, ctx.scoping());

        // (3) Reset the accumulator for the next pass. `references_len` only
        //     grows (helpers mint, never delete, references), so the bitset
        //     is re-allocated only when refs were minted this pass; otherwise
        //     a memset reuses the warm allocation (a bump arena never
        //     reclaims the old one).
        let refs_len = ctx.scoping().references_len();
        if ctx.state.dirty.dead_refs.capacity() == refs_len {
            if had_dead {
                ctx.state.dirty.dead_refs.clear();
            }
        } else {
            ctx.state.dirty.dead_refs = BitSet::new_in(refs_len, ctx.allocator());
        }
        ctx.state.dirty.eval_dropped = false;
    }

    /// End-of-pass sequence: flush the dirty accumulator into scoping, then
    /// consume the pass's liveness collection — which must observe the
    /// post-flush scoping (its debug ground-truth walk validates against
    /// it), so fusing the pair makes the ordering structural. Returns
    /// whether liveness demands another pass (see
    /// `symbol_liveness::propagate_collected`).
    pub(crate) fn end_pass(program: &Program<'a>, ctx: &mut TraverseCtx<'a>) -> bool {
        Self::flush_pass_dirty(program, ctx);
        symbol_liveness::propagate_collected(ctx)
    }
}

impl<'a> Traverse<'a> for PeepholeOptimizations {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Reset the in-pass liveness collection (runs in dce mode too).
        symbol_liveness::begin_pass(ctx);
        ctx.state.symbol_values.reset();
        // Any module loader (`import`, `export * from`, `export … from`) can, on a
        // cycle, evaluate a foreign module that observes a not-yet-assigned binding
        // our exports close over. So the program root starts its prelude "unsafe"
        // when the body has any loader — bailing every program-scope var inline.
        // Loaders are hoisted, so scan the whole body (an import may follow a
        // leading var); the result never changes across passes.
        let module_has_loaders = program
            .body
            .iter()
            .any(|s| s.as_module_declaration().is_some_and(|m| m.source().is_some()));
        // `enter`/`exit_function_body` are balanced, so the stack is back to its
        // single program-root entry by the next pass; reset it in place rather
        // than reallocating (matching the `reset`/`clear` above).
        *ctx.state.body_unsafe_stack.last_mut() =
            (ctx.scoping().root_scope_id(), module_has_loaders);
        // `PassDirty` is managed by the `Compressor` driver via
        // `flush_pass_dirty`, not reset per traversal.
    }

    // Liveness-collection delegations — keep this set in sync with the
    // identical one in `Normalize` (both traversals collect; the membership
    // is part of the analysis contract, see the `symbol_liveness` module
    // doc). Deliberately no dce gating: collection runs in dce mode too.
    fn enter_identifier_reference(
        &mut self,
        node: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        symbol_liveness::collect_identifier_reference(node, ctx);
    }

    fn enter_function(&mut self, node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        symbol_liveness::collect_enter_function(node, ctx);
    }

    fn exit_function(&mut self, _node: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        symbol_liveness::collect_exit_function(ctx);
    }

    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        symbol_liveness::collect_enter_class(node, ctx);
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        symbol_liveness::collect_enter_variable_declarator(node, ctx);
    }

    fn enter_function_body(&mut self, _body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.body_unsafe_stack.push((ctx.current_scope_id(), false));
    }

    fn exit_function_body(&mut self, _body: &mut FunctionBody<'a>, ctx: &mut TraverseCtx<'a>) {
        ctx.state.body_unsafe_stack.pop();
    }

    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Only check class_symbols_stack in full optimization mode (not DCE mode)
        debug_assert!(ctx.state.dce || ctx.state.class_symbols_stack.is_exhausted());
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::minimize_statements(stmts, ctx);
    }

    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        Self::keep_track_of_pure_functions(stmt, ctx);
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            match stmt {
                Statement::BlockStatement(_) => Self::try_optimize_block(stmt, ctx),
                Statement::IfStatement(_) => Self::try_fold_if(stmt, ctx),
                Statement::ForStatement(_) => Self::try_fold_for(stmt, ctx),
                Statement::TryStatement(_) => Self::try_fold_try(stmt, ctx),
                Statement::LabeledStatement(_) => Self::try_fold_labeled(stmt, ctx),
                Statement::FunctionDeclaration(_) => {
                    Self::remove_unused_function_declaration(stmt, ctx);
                }
                Statement::ClassDeclaration(_) => {
                    Self::remove_unused_class_declaration(stmt, ctx);
                }
                Statement::ExpressionStatement(_) => {
                    Self::try_fold_expression_stmt(stmt, ctx);
                }
                Statement::ImportDeclaration(_) => {
                    Self::remove_unused_import_specifiers(stmt, ctx);
                }
                _ => {}
            }
        } else {
            match stmt {
                Statement::BlockStatement(_) => Self::try_optimize_block(stmt, ctx),
                Statement::IfStatement(s) => {
                    Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
                    Self::try_fold_if(stmt, ctx);
                    if let Statement::IfStatement(if_stmt) = stmt
                        && let Some(folded_stmt) = Self::try_minimize_if(if_stmt, ctx)
                    {
                        ctx.replace_statement(stmt, folded_stmt);
                    }
                }
                Statement::WhileStatement(s) => {
                    Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
                }
                Statement::ForStatement(s) => {
                    if let Some(test) = &mut s.test {
                        Self::minimize_expression_in_boolean_context(test, ctx);
                    }
                    Self::try_fold_for(stmt, ctx);
                }
                Statement::DoWhileStatement(s) => {
                    Self::minimize_expression_in_boolean_context(&mut s.test, ctx);
                }
                Statement::TryStatement(_) => Self::try_fold_try(stmt, ctx),
                Statement::LabeledStatement(_) => Self::try_fold_labeled(stmt, ctx),
                Statement::FunctionDeclaration(f) => {
                    Self::init_function_declaration_symbol_value(f.id.as_ref(), ctx);
                    Self::remove_unused_function_declaration(stmt, ctx);
                }
                Statement::ClassDeclaration(c) => {
                    Self::init_class_declaration_symbol_value(c, ctx);
                    Self::remove_unused_class_declaration(stmt, ctx);
                }
                Statement::ImportDeclaration(_) => Self::remove_unused_import_specifiers(stmt, ctx),
                _ => {}
            }
            Self::try_fold_expression_stmt(stmt, ctx);
        }

        // Maintain the per-body declarative-prelude flag used by
        // `is_hoisted_var_inlineable`.
        if !Self::is_declarative_body_statement(stmt) {
            Self::mark_current_body_unsafe(ctx);
        }
    }

    fn exit_for_statement(&mut self, stmt: &mut ForStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_for_statement(stmt, ctx);
        Self::minimize_for_statement(stmt, ctx);
    }

    fn exit_return_statement(&mut self, stmt: &mut ReturnStatement<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_return_statement(stmt, ctx);
    }

    fn exit_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_variable_declaration(decl, ctx);
    }

    fn exit_variable_declarator(
        &mut self,
        decl: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        Self::init_symbol_value(decl, ctx);
        // Per-declarator update of the body-unsafe flag. Catches multi-declarator
        // statements (`var [x=call()] = '', flag = true;`, possibly produced by
        // join-vars) where an earlier declarator runs user code via a
        // destructuring default or non-literal init — the per-statement check
        // would fire too late for subsequent declarators' `init_symbol_value`.
        if !Self::is_declarative_variable_declarator(decl) {
            Self::mark_current_body_unsafe(ctx);
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Tree-shaking mode: fewer passes than full minify below. Only the ones
        // that remove code, plus the constant folds those removals need. The
        // folds stay on because the removal passes don't evaluate compound
        // conditions themselves: `if ('production' === 'production')` must fold
        // to `true` before the dead branch can be dropped. Passes that only
        // shrink code (`substitute_*`, `minimize_*`) are left out. See the `dce`
        // docs in `state.rs`.
        if ctx.state.dce {
            match expr {
                Expression::TemplateLiteral(t) => {
                    Self::inline_template_literal(t, ctx);
                }
                Expression::ObjectExpression(e) => Self::fold_object_exp(e, ctx),
                Expression::BinaryExpression(_) => {
                    Self::fold_binary_expr(expr, ctx);
                    Self::fold_binary_typeof_comparison(expr, ctx);
                }
                Expression::UnaryExpression(_) => Self::fold_unary_expr(expr, ctx),
                Expression::StaticMemberExpression(_) => {
                    Self::fold_static_member_expr(expr, ctx);
                }
                Expression::ComputedMemberExpression(_) => {
                    Self::fold_computed_member_expr(expr, ctx);
                }
                Expression::LogicalExpression(_) => Self::fold_logical_expr(expr, ctx),
                Expression::ChainExpression(_) => Self::fold_chain_expr(expr, ctx),
                Expression::CallExpression(_) => {
                    Self::fold_call_expression(expr, ctx);
                    Self::substitute_iife_call(expr, ctx);
                    Self::remove_dead_code_call_expression(expr, ctx);
                }
                Expression::ConditionalExpression(_) => {
                    Self::try_fold_conditional_expression(expr, ctx);
                }
                Expression::SequenceExpression(_) => {
                    Self::remove_sequence_expression(expr, ctx);
                }
                Expression::AssignmentExpression(_) => {
                    Self::remove_unused_assignment_expr(expr, ctx);
                }
                _ => {}
            }
        } else {
            match expr {
                Expression::TemplateLiteral(t) => {
                    Self::inline_template_literal(t, ctx);
                    Self::substitute_template_literal(expr, ctx);
                }
                Expression::ObjectExpression(e) => Self::fold_object_exp(e, ctx),
                Expression::BinaryExpression(e) => {
                    Self::substitute_swap_binary_expressions(e);
                    Self::fold_binary_expr(expr, ctx);
                    Self::fold_binary_typeof_comparison(expr, ctx);
                    Self::fold_sequence_expression(expr, ctx);
                    Self::minimize_loose_boolean(expr, ctx);
                    Self::minimize_binary(expr, ctx);
                    Self::substitute_loose_equals_undefined(expr, ctx);
                    Self::substitute_typeof_undefined(expr, ctx);
                    Self::substitute_rotate_binary_expression(expr, ctx);
                }
                Expression::UnaryExpression(_) => {
                    Self::fold_unary_expr(expr, ctx);
                    Self::minimize_unary(expr, ctx);
                    Self::substitute_unary_plus(expr, ctx);
                    Self::fold_sequence_expression(expr, ctx);
                }
                Expression::YieldExpression(_) | Expression::AwaitExpression(_) => {
                    Self::fold_sequence_expression(expr, ctx);
                }
                Expression::StaticMemberExpression(_) => {
                    Self::fold_static_member_expr(expr, ctx);
                    Self::replace_known_property_access(expr, ctx);
                }
                Expression::ComputedMemberExpression(_) => {
                    Self::fold_computed_member_expr(expr, ctx);
                    Self::replace_known_property_access(expr, ctx);
                }
                Expression::LogicalExpression(_) => {
                    Self::fold_logical_expr(expr, ctx);
                    Self::fold_sequence_expression(expr, ctx);
                    Self::minimize_logical_expression(expr, ctx);
                    Self::substitute_is_object_and_not_null(expr, ctx);
                    Self::substitute_rotate_logical_expression(expr, ctx);
                }
                Expression::ChainExpression(_) => {
                    Self::fold_chain_expr(expr, ctx);
                    Self::substitute_chain_expression(expr, ctx);
                }
                Expression::CallExpression(_) => {
                    Self::fold_call_expression(expr, ctx);
                    Self::substitute_iife_call(expr, ctx);
                    Self::remove_dead_code_call_expression(expr, ctx);
                    Self::replace_concat_chain(expr, ctx);
                    Self::replace_known_global_methods(expr, ctx);
                    Self::substitute_simple_function_call(expr, ctx);
                    Self::substitute_object_or_array_constructor(expr, ctx);
                }
                Expression::ConditionalExpression(logical_expr) => {
                    Self::minimize_expression_in_boolean_context(&mut logical_expr.test, ctx);
                    if let Some(changed) = Self::minimize_conditional_expression(logical_expr, ctx)
                    {
                        ctx.replace_expression(expr, changed);
                    }
                    Self::try_fold_conditional_expression(expr, ctx);
                }
                Expression::AssignmentExpression(e) => {
                    Self::minimize_normal_assignment_to_combined_logical_assignment(e, ctx);
                    Self::minimize_normal_assignment_to_combined_assignment(e, ctx);
                    Self::minimize_assignment_to_update_expression(expr, ctx);
                    Self::remove_unused_assignment_expr(expr, ctx);
                }
                Expression::SequenceExpression(_) => Self::remove_sequence_expression(expr, ctx),
                Expression::ArrowFunctionExpression(e) => Self::substitute_arrow_expression(e, ctx),
                Expression::FunctionExpression(e) => Self::try_remove_name_from_functions(e, ctx),
                Expression::ClassExpression(e) => Self::try_remove_name_from_classes(e, ctx),
                Expression::NewExpression(e) => {
                    Self::substitute_typed_array_constructor(e, ctx);
                    Self::substitute_global_new_expression(expr, ctx);
                    Self::substitute_object_or_array_constructor(expr, ctx);
                }
                Expression::BooleanLiteral(_) => Self::substitute_boolean(expr, ctx),
                Expression::ArrayExpression(_) => {
                    Self::try_flatten_array_expression_elements(expr, ctx);
                    Self::substitute_array_expression(expr, ctx);
                }
                Expression::Identifier(_) => Self::inline_identifier_reference(expr, ctx),
                _ => {}
            }
        }
    }

    fn exit_unary_expression(&mut self, expr: &mut UnaryExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        if expr.operator.is_not() {
            Self::minimize_expression_in_boolean_context(&mut expr.argument, ctx);
        }
    }

    fn exit_call_expression(&mut self, e: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !ctx.state.dce {
            Self::substitute_call_expression(e, ctx);
            Self::remove_empty_spread_arguments(&mut e.arguments);
        }
        // Re-evaluate each iteration: peephole folding/inlining may expose a
        // pure-eligible arg shape that `Normalize`'s one-shot pass missed.
        Normalize::set_no_side_effects_to_call_expr(e, ctx);
    }

    fn exit_new_expression(&mut self, e: &mut NewExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if !ctx.state.dce {
            Self::substitute_new_expression(e, ctx);
            Self::remove_empty_spread_arguments(&mut e.arguments);
        }
        Normalize::set_pure_or_no_side_effects_to_new_expr(e, ctx);
    }

    fn exit_object_property(&mut self, prop: &mut ObjectProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_object_property(prop, ctx);
    }

    fn exit_assignment_target_property(
        &mut self,
        node: &mut AssignmentTargetProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_assignment_target_property(node, ctx);
    }

    fn exit_assignment_target_property_property(
        &mut self,
        prop: &mut AssignmentTargetPropertyProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_assignment_target_property_property(prop, ctx);
    }

    fn exit_binding_property(&mut self, prop: &mut BindingProperty<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_binding_property(prop, ctx);
    }

    fn exit_method_definition(
        &mut self,
        prop: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_method_definition(prop, ctx);
    }

    fn exit_property_definition(
        &mut self,
        prop: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_property_definition(prop, ctx);
    }

    fn exit_accessor_property(
        &mut self,
        prop: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_accessor_property(prop, ctx);
    }

    fn exit_member_expression(
        &mut self,
        expr: &mut MemberExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        Self::convert_to_dotted_properties(expr, ctx);
    }

    fn enter_class_body(&mut self, _body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        ctx.state.class_symbols_stack.push_class_scope();
    }

    fn exit_class_body(&mut self, body: &mut ClassBody<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        Self::remove_dead_code_exit_class_body(body, ctx);
        Self::remove_unused_private_members(body, ctx);
        ctx.state.class_symbols_stack.pop_class_scope(Self::get_declared_private_symbols(body));
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.state.dce {
            return;
        }
        Self::substitute_catch_clause(catch, ctx);
    }

    fn exit_private_field_expression(
        &mut self,
        node: &mut PrivateFieldExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        ctx.state.class_symbols_stack.push_private_member_to_current_class(node.field.name.into());
    }

    fn exit_private_in_expression(
        &mut self,
        node: &mut PrivateInExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ctx.state.dce {
            return;
        }
        ctx.state.class_symbols_stack.push_private_member_to_current_class(node.left.name.into());
    }
}

/// Walks the live program to find scopes containing direct `eval(...)` calls.
/// Used by `flush_pass_dirty` only when at least one direct eval call was
/// dropped this pass (gated via `PassDirty::eval_dropped`).
struct LiveDirectEvalCollector<'s> {
    scoping: &'s Scoping,
    scopes: FxHashSet<ScopeId>,
}

impl<'s> LiveDirectEvalCollector<'s> {
    fn new(scoping: &'s Scoping) -> Self {
        Self { scoping, scopes: FxHashSet::default() }
    }
}

impl<'a> Visit<'a> for LiveDirectEvalCollector<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(ident) = as_direct_eval_call(it)
            && let Some(reference_id) = ident.reference_id.get()
        {
            let scope_id = self.scoping.get_reference(reference_id).scope_id();
            self.scopes.insert(scope_id);
        }
        // Recurse — `eval` may be nested in another call's arguments, e.g. `foo(eval('x'))`.
        walk_call_expression(self, it);
    }
}
