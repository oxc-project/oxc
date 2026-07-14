//! Ground-truth validation for the in-pass liveness collection.
//!
//! [`compute_dead_symbols`] re-derives the dead and pinned sets with an
//! INDEPENDENT standalone walk of the settled tree — its own
//! export/for-head state, only the shared gate predicates from the parent
//! module. [`super::propagate_collected`] asserts every flushed in-pass set
//! against it, so the whole test and conformance corpus diffs the
//! interleaved collection against ground truth. Deliberately a second
//! implementation: sharing traversal code with the hooks would make the
//! oracle blind to exactly the bugs it exists to catch.

use oxc_allocator::{Allocator, BitSet, Vec as ArenaVec};
use oxc_ast::ast::*;
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_class, walk_export_default_declaration,
        walk_export_named_declaration, walk_for_statement_left, walk_function,
        walk_variable_declarator,
    },
};
use oxc_ecmascript::BoundNames;
use oxc_semantic::Scoping;
use oxc_syntax::scope::ScopeFlags;
use oxc_syntax::symbol::SymbolId;

use crate::CompressOptions;

use super::{CANDIDATE_KINDS, collection_enabled, propagate};

/// Compute the dead and expected-pin sets with a standalone walk of a
/// settled tree. Returns `(dead, candidates, pins)`; bits are
/// `SymbolId::index()`.
///
/// Debug-only ground truth: the release pipeline never walks — collection
/// rides the Normalize and peephole traversals — but every
/// [`super::propagate_collected`] validates its flushed sets against this
/// walk of the post-flush tree, so the whole test corpus checks the in-pass
/// collection.
pub fn compute_dead_symbols<'a>(
    program: &Program<'a>,
    scoping: &Scoping,
    options: &CompressOptions,
    allocator: &'a Allocator,
) -> (BitSet<'a>, BitSet<'a>, BitSet<'a>) {
    let empty = || {
        (BitSet::new_in(0, allocator), BitSet::new_in(0, allocator), BitSet::new_in(0, allocator))
    };
    if !collection_enabled(scoping, program.source_type, options) {
        return empty();
    }

    let symbols_len = scoping.symbols_len();
    let mut collector = Collector {
        scoping,
        in_export: false,
        in_for_head: false,
        enclosing_candidate: None,
        candidates: BitSet::new_in(symbols_len, allocator),
        live: BitSet::new_in(symbols_len, allocator),
        pins: BitSet::new_in(symbols_len, allocator),
        roots: ArenaVec::new_in(&allocator),
        edges: ArenaVec::new_in(&allocator),
    };
    collector.visit_program(program);

    let Collector { candidates, mut live, pins, roots, mut edges, .. } = collector;
    if candidates.is_empty() {
        return (BitSet::new_in(0, allocator), BitSet::new_in(0, allocator), pins);
    }

    let mut worklist = roots;
    propagate(&candidates, &mut live, &mut worklist, &mut edges);

    let mut dead = BitSet::new_in(symbols_len, allocator);
    for candidate in candidates.ones() {
        if !live.contains(candidate) {
            dead.set_bit(candidate);
        }
    }
    (dead, candidates, pins)
}

struct Collector<'a, 'b> {
    scoping: &'b Scoping,
    /// The visited node is the `declaration` of an export statement (or a
    /// sibling declarator of one). Cleared for the declaration's subtree so
    /// declarations nested inside an exported function/class stay eligible.
    in_export: bool,
    /// The innermost declarator being visited sits in a for-in/of HEAD —
    /// no removal site handles heads, so their bindings force-root.
    in_for_head: bool,
    /// Innermost candidate whose deferred region we are inside.
    enclosing_candidate: Option<SymbolId>,
    candidates: BitSet<'a>,
    /// Marked at record time for roots, during propagation for edge targets.
    live: BitSet<'a>,
    /// Every symbol a pin position demands (export wrappers, for-in/of
    /// heads, `using`): the expected-pin set the flush asserts
    /// `pinned_next` covers. A missed pin is silent wrong code the dead-set
    /// checks cannot see — the count arm deletes an observable binding
    /// while the dead set stays correct.
    pins: BitSet<'a>,
    /// Deduplicated live-context references to candidate-kind symbols (plus
    /// pinned position-ineligible declarations); the propagation worklist.
    roots: ArenaVec<'a, SymbolId>,
    /// `(innermost enclosing candidate, referenced candidate-kind symbol)`.
    /// May contain duplicates (one entry per reference); propagation dedups
    /// via the `live` bitset.
    edges: ArenaVec<'a, (SymbolId, SymbolId)>,
}

impl Collector<'_, '_> {
    /// Mark a symbol live and enqueue it for propagation; deduplicated at
    /// record time via the `live` bitset.
    fn mark_live_root(&mut self, symbol_id: SymbolId) {
        if !self.live.contains(symbol_id.index()) {
            self.live.set_bit(symbol_id.index());
            self.roots.push(symbol_id);
        }
    }

    /// Root AND record an expected pin — mirrors the release collection's
    /// `pin_live_root` for the pin positions this walk re-derives.
    fn pin_live_root(&mut self, symbol_id: SymbolId) {
        self.mark_live_root(symbol_id);
        self.pins.set_bit(symbol_id.index());
    }

    /// Shared visitor skeleton for the three declaration kinds: clear
    /// `in_export` for the subtree, track the innermost enclosing candidate
    /// across the walk, restore both.
    fn walk_declaration(&mut self, candidate: Option<SymbolId>, walk: impl FnOnce(&mut Self)) {
        let saved_export = std::mem::replace(&mut self.in_export, false);
        let saved_candidate = self.enclosing_candidate;
        if let Some(symbol_id) = candidate {
            self.candidates.set_bit(symbol_id.index());
            self.enclosing_candidate = Some(symbol_id);
        }
        walk(self);
        self.enclosing_candidate = saved_candidate;
        self.in_export = saved_export;
    }
}

impl<'a> Visit<'a> for Collector<'_, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if let Some(reference_id) = it.reference_id.get()
            && let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id()
            && self.scoping.symbol_flags(symbol_id).intersects(CANDIDATE_KINDS)
        {
            match self.enclosing_candidate {
                None => self.mark_live_root(symbol_id),
                Some(from) => self.edges.push((from, symbol_id)),
            }
        }
    }

    fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
        let saved = std::mem::replace(&mut self.in_export, true);
        walk_export_named_declaration(self, it);
        self.in_export = saved;
    }

    fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
        let saved = std::mem::replace(&mut self.in_export, true);
        walk_export_default_declaration(self, it);
        self.in_export = saved;
    }

    fn visit_for_statement_left(&mut self, it: &ForStatementLeft<'a>) {
        let saved = std::mem::replace(&mut self.in_for_head, true);
        walk_for_statement_left(self, it);
        self.in_for_head = saved;
    }

    /// An arrow bounds the export flag exactly like the other function
    /// forms do (via [`Self::walk_declaration`]): in
    /// `export default (w) => { function f() {} }`, `f` is an ordinary
    /// candidate, not the exported declaration. Arrows are the one
    /// function form `visit_function` never sees, and they can never BE a
    /// declaration, so only the flag needs clearing — the enclosing
    /// candidate stays (an arrow inside a candidate's body is part of its
    /// deferred region).
    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        let saved = std::mem::replace(&mut self.in_export, false);
        walk_arrow_function_expression(self, it);
        self.in_export = saved;
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        let mut candidate = None;
        if it.is_declaration()
            && let Some(symbol_id) = it.id.as_ref().and_then(|id| id.symbol_id.get())
        {
            if self.in_export {
                self.pin_live_root(symbol_id);
            } else {
                candidate = Some(symbol_id);
            }
        }
        self.walk_declaration(candidate, |v| walk_function(v, it, flags));
    }

    fn visit_class(&mut self, it: &Class<'a>) {
        // Classes are never candidates; only the export-observability pin
        // applies (mirrors `collect_enter_class`).
        if it.is_declaration()
            && let Some(symbol_id) = it.id.as_ref().and_then(|id| id.symbol_id.get())
            && self.in_export
        {
            self.pin_live_root(symbol_id);
        }
        self.walk_declaration(None, |v| walk_class(v, it));
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        // Declarators are never candidates; mirror the stable pins of
        // `collect_enter_variable_declarator`. For-in/of heads root but are
        // NOT demanded as expected pins: the head merge in
        // `minimize_statements` (`var a; for (a in b)` -> `for (var a in b)`)
        // legitimately creates head declarators mid-pass that the enter-time
        // collection never saw, and a merged head is init-less with no other
        // declaration site left to consult — the release pin for
        // enter-time heads is defensive, not load-bearing.
        if it.kind.is_using() || self.in_export {
            // Expect a pin for every symbol the site binds, destructuring
            // included.
            it.id.bound_names(&mut |ident| {
                if let Some(symbol_id) = ident.symbol_id.get() {
                    self.pin_live_root(symbol_id);
                }
            });
        } else if self.in_for_head {
            it.id.bound_names(&mut |ident| {
                if let Some(symbol_id) = ident.symbol_id.get() {
                    self.mark_live_root(symbol_id);
                }
            });
        }
        self.walk_declaration(None, |v| walk_variable_declarator(v, it));
    }
}
