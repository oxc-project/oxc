//! Symbol liveness: observability that reference counts cannot express, and
//! reachability for recursively referenced function declarations.
//!
//! Reference counting removes an acyclic unused declaration once its last
//! reference disappears, but it cannot remove `function a() { b() }
//! function b() { a() }`: each function keeps the other's count non-zero.
//! This module adds the missing reachability question: can executing code
//! reach a candidate function declaration or function-valued declarator?
//!
//! Four concepts define the analysis:
//!
//! 1. **Candidates** are eligible function declarations and exact
//!    function-valued declarators whose deadness may require graph reachability.
//!    Normalize initially registers every eligible declaration. Function
//!    declarations and registered declarators participate in later analyses.
//!    A live declarator whose body has no graph edge is unregistered and its
//!    scope owner mapping is cleared before normal inlining can relocate it;
//!    declarators that do own graph edges remain registered so later dead-code
//!    elimination can safely recompute their reachability. Published-dead
//!    candidates remain tracked as tombstones after their declarations are
//!    removed.
//!    Other declaration kinds continue to use reference counts. The local
//!    self-recursive declarator check remains as a fallback for exact shapes
//!    created after Normalize.
//! 2. **Ownership** comes from semantic scopes. The nearest registered function
//!    declaration or active declarator initializer is a reference's owner. A
//!    reference from a candidate that is
//!    not implicitly observable is stored as `(owner, target)` and propagates
//!    when the owner becomes live. Implicitly observable owners and references
//!    outside registered functions make their targets live immediately. A
//!    registered non-candidate owner does so only while implicitly observable
//!    or count-live, because a count-dead owner's body never executes.
//! 3. **Implicit observability** is stable metadata for bindings whose runtime
//!    value can be observed without a resolved reference in this AST. A symbol
//!    here may have any number of references; the bit records that an observer
//!    remains even if every reference disappears. The four channels are module
//!    exports, Script-root bindings, Annex B aliases, and `using` disposal. This
//!    metadata protects every count-based removal consumer, not only
//!    declaration removal.
//! 4. **Analysis runs after scoping is flushed.** Every result is derived from
//!    the settled resolved-reference lists, so AST rewrites need no parallel
//!    collection hooks or behind-the-cursor repair log.
//!
//! ## Transform contract
//!
//! Functions are registered once during Normalize and the graph is recomputed
//! only when its settled inputs can change. Peephole transforms must preserve
//! four invariants:
//!
//! 1. **Do not create function declarations.** A new declaration would have no
//!    entry in `function_by_scope` and therefore could not own references or
//!    become a candidate. Removing an existing declaration is supported. Exact
//!    declarators are registered only during Normalize. A declarator that owns
//!    a graph edge cannot be single-use inlined until the graph untracks or
//!    removes it.
//! 2. **Do not move an existing reference across function owners.** Ownership
//!    comes from `Reference::scope_id()` and the registered function-scope
//!    ancestors. A transform that changes the nearest owning function must
//!    instead drop and recreate the reference, so the pass-change gate requests
//!    a new analysis.
//! 3. **Do not create a path to a function already published as dead.** Deadness
//!    is monotonic. Transforms may duplicate an already-live reference, but
//!    must not make an unreachable binding reachable; debug builds assert that
//!    a dead function never becomes live again.
//! 4. **Do not form a direct eval call.** Semantic flags propagate any direct
//!    eval to the root scope, disabling the analysis for the whole program. The
//!    analysis relies on that flag only ever clearing (dropping an eval
//!    re-triggers analysis). An eval created after deadness was published could
//!    reach a removed function. Debug builds assert this never happens.
//!
//! A transform that needs to violate one of these invariants must first extend
//! function registration or the pass-change analysis signal. Keeping this boundary
//! explicit is what lets the analysis run without per-transform collection
//! hooks or a reference-mint log.
//!
//! ES module bindings and CommonJS top-level bindings are local to their module
//! or wrapper. Script root bindings are visible to later scripts, so root
//! function declarations are implicitly observable, not graph candidates;
//! references in their bodies keep local candidates live. Local declarations
//! inside Script functions and strict blocks can still be removed.
//!
//! Treating a count-dead non-candidate owner's body as unreachable requires
//! that a declaration cannot execute without a resolved reference.
//! Registration therefore also excludes sloppy Annex B block functions whose
//! runtime var-alias write is not represented by their block-scoped symbol.
//!
//! ## Current limitations
//!
//! - Only exact function-valued declarators in statement lists (and their
//!   direct classic `for` initializers) participate. Wrapped initializers and
//!   declaration positions without a stable removal site are kept.
//! - Class cycles are kept. Class removal has no stable proof
//!   (`remove_unused_class` extracts static values, and heritage classification
//!   can change between passes).
//! - Any direct eval propagates to the root scope and disables the analysis for
//!   the whole program. There is no per-scope granularity.
//! - Sloppy unhoisted Annex B block functions are always kept, even when
//!   nothing can reach them. Removing them requires modeling the runtime
//!   var-alias write.

use oxc_allocator::{Allocator, ArenaVec, BitSet, GetAllocator};
use oxc_ast::ast::*;
#[cfg(debug_assertions)]
use oxc_ast_visit::{
    VisitJs,
    walk_js::{walk_function, walk_variable_declarator},
};
use oxc_ecmascript::BoundNames;
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_syntax::{reference::ReferenceId, scope::ScopeId, symbol::SymbolId};

use crate::{CompressOptions, CompressOptionsUnused, TraverseCtx, generated::ancestor::Ancestor};

/// Stable program-wide symbol facts plus the optional recursive-function graph.
///
/// This is always present for ES modules because exported-binding implicit
/// observability must protect count-based optimizations even when recursive
/// function removal is disabled. For CommonJS and Script sources it exists
/// whenever unused-declaration removal is enabled, and a `using` declaration
/// creates it lazily even otherwise, because disposal observes its binding
/// without an ordinary reference.
pub struct SymbolLiveness<'a> {
    /// Bindings with a runtime observer independent of resolved references:
    /// module exports, Script roots, Annex B aliases, and `using` disposal.
    implicitly_observable: BitSet<'a>,
    recursive_functions: Option<FunctionGraph<'a>>,
}

impl<'a> SymbolLiveness<'a> {
    /// Return `None` when the configuration needs no symbol liveness: a
    /// non-module source with unused-declaration removal disabled.
    pub fn new_if_enabled(
        source_type: SourceType,
        options: &CompressOptions,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) -> Option<Self> {
        let recursive_functions_enabled = options.unused != CompressOptionsUnused::Keep;
        if !source_type.is_module() && !recursive_functions_enabled {
            return None;
        }
        Some(Self::new(source_type, scoping, allocator))
    }

    /// Seed implicit observability from scoping; the function graph is created
    /// lazily at first registration.
    fn new(source_type: SourceType, scoping: &Scoping, allocator: &'a Allocator) -> Self {
        let symbols_len = scoping.symbols_len();
        let mut implicitly_observable = BitSet::new_in(symbols_len, allocator);
        if source_type.is_script() {
            for &symbol_id in scoping.get_bindings(scoping.root_scope_id()).values() {
                implicitly_observable.set_bit(symbol_id.index());
            }
        }
        Self { implicitly_observable, recursive_functions: None }
    }

    #[inline]
    pub fn is_implicitly_observable(&self, symbol_id: SymbolId) -> bool {
        self.implicitly_observable.contains(symbol_id.index())
    }

    #[inline]
    pub fn function_is_dead(&self, symbol_id: SymbolId) -> bool {
        self.recursive_functions.as_ref().is_some_and(|graph| graph.is_dead(symbol_id))
    }

    #[inline]
    pub fn function_declarator_is_dead(&self, symbol_id: SymbolId, scope_id: ScopeId) -> bool {
        self.recursive_functions
            .as_ref()
            .is_some_and(|graph| graph.is_declarator_dead(symbol_id, scope_id))
    }

    #[inline]
    pub fn is_function_declarator_candidate(&self, symbol_id: SymbolId) -> bool {
        self.recursive_functions
            .as_ref()
            .is_some_and(|graph| graph.is_function_declarator_candidate(symbol_id))
    }

    fn mark_implicitly_observable(&mut self, symbol_id: SymbolId) {
        self.implicitly_observable.set_bit(symbol_id.index());
    }

    fn mark_bound_names<'b>(&mut self, node: &impl BoundNames<'b>) {
        node.bound_names(&mut |ident| {
            if let Some(symbol_id) = ident.symbol_id.get() {
                self.mark_implicitly_observable(symbol_id);
            }
        });
    }

    fn register_function(
        &mut self,
        function: &Function<'_>,
        source_type: SourceType,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) {
        let Some(symbol_id) = function.id.as_ref().and_then(|id| id.symbol_id.get()) else {
            return;
        };
        let Some(scope_id) = function.scope_id.get() else { return };

        let binding_scope_id = scoping.symbol_scope_id(symbol_id);

        // Annex B block functions also assign their function object to a
        // var-like binding when the block executes. Semantic hoisting records
        // that alias when possible, but duplicates and TypeScript declarations
        // can retain a block-scoped symbol even though the runtime alias write
        // still occurs. Such a declaration and writes to it can be observed
        // with no resolved reference to its own symbol, so mark it implicitly
        // observable instead of registering it.
        let binding_scope_flags = scoping.scope_flags(binding_scope_id);
        if !function.r#async
            && !function.generator
            && !binding_scope_flags.is_var()
            && !binding_scope_flags.is_strict_mode()
        {
            self.mark_implicitly_observable(symbol_id);
            return;
        }

        if source_type.is_script() && binding_scope_id == scoping.root_scope_id() {
            // Already seeded implicitly observable by `new`; only graph
            // candidacy is skipped here.
            return;
        }

        let graph =
            self.recursive_functions.get_or_insert_with(|| FunctionGraph::new(scoping, allocator));
        graph.register_function_declaration(scope_id, symbol_id);
    }

    fn register_function_declarator(
        &mut self,
        decl: &VariableDeclarator<'_>,
        allocator: &'a Allocator,
        scoping: &Scoping,
    ) {
        let BindingPattern::BindingIdentifier(id) = &decl.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        let Some(scope_id) = function_declarator_scope_id(decl) else { return };
        if self.is_implicitly_observable(symbol_id)
            || scoping.root_scope_flags().contains_direct_eval()
        {
            return;
        }

        let graph =
            self.recursive_functions.get_or_insert_with(|| FunctionGraph::new(scoping, allocator));
        graph.register_function_declarator(scope_id, symbol_id);
    }

    fn analyze(&mut self, scoping: &Scoping) -> bool {
        let Some(graph) = &mut self.recursive_functions else { return false };
        graph.analyze(scoping, &self.implicitly_observable)
    }

    #[cfg(debug_assertions)]
    fn dead_declarations(&self) -> Option<(&BitSet<'a>, Option<&BitSet<'a>>)> {
        self.recursive_functions.as_ref().map(|graph| {
            (&graph.dead, graph.declarators.as_ref().map(|declarators| &declarators.scopes))
        })
    }
}

/// The recursive-function reachability graph and its reused analysis buffers.
struct FunctionGraph<'a> {
    /// Eligible functions the graph still tracks. [`Self::analyze`]
    /// permanently stops tracking a non-dead candidate whose references all
    /// lie outside registered functions, because it no longer needs graph
    /// reachability. Graph-independent removal checks then handle it. Untracked
    /// functions stay in `function_by_scope` and can still own references.
    candidates: BitSet<'a>,
    /// Declarator-specific source metadata. It is allocated only after an
    /// eligible function-valued declarator appears, keeping the longstanding
    /// function-declaration-only path allocation-free.
    declarators: Option<FunctionDeclaratorCandidates<'a>>,
    allocator: &'a Allocator,
    /// Maps each registered function declaration's own scope to its symbol.
    /// Owner lookup walks the current semantic parent chain, so scopes inserted
    /// or reparented after Normalize remain correct.
    function_by_scope: ArenaVec<'a, Option<SymbolId>>,
    /// Function symbols already published as unreachable. Declaration-removal
    /// sites consume membership, but bits persist after AST removal so later
    /// analyses can reject resurrection. Deadness is monotonic.
    dead: BitSet<'a>,
    scratch: GraphScratch<'a>,
}

impl<'a> FunctionGraph<'a> {
    fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        let symbols_len = scoping.symbols_len();
        let function_by_scope =
            ArenaVec::from_iter_in(std::iter::repeat_n(None, scoping.scopes_len()), &allocator);
        Self {
            candidates: BitSet::new_in(symbols_len, allocator),
            declarators: None,
            allocator,
            function_by_scope,
            dead: BitSet::new_in(symbols_len, allocator),
            scratch: GraphScratch::new(symbols_len, allocator),
        }
    }

    fn register_function_declaration(&mut self, scope_id: ScopeId, symbol_id: SymbolId) {
        self.function_by_scope[scope_id.index()] = Some(symbol_id);
        self.candidates.set_bit(symbol_id.index());
        if let Some(declarators) = &mut self.declarators {
            declarators.function_declarations.set_bit(symbol_id.index());
        }
    }

    fn register_function_declarator(&mut self, scope_id: ScopeId, symbol_id: SymbolId) {
        if self.declarators.is_none() {
            let mut declarators = FunctionDeclaratorCandidates::new(
                self.candidates.capacity(),
                self.function_by_scope.len(),
                self.allocator,
            );
            for bit in self.candidates.ones() {
                declarators.function_declarations.set_bit(bit);
            }
            self.declarators = Some(declarators);
        }
        self.function_by_scope[scope_id.index()] = Some(symbol_id);
        self.candidates.set_bit(symbol_id.index());
        let declarators = self.declarators.as_mut().unwrap();
        declarators.function_declarators.set_bit(symbol_id.index());
        declarators.scopes.set_bit(scope_id.index());
    }

    #[inline]
    fn is_dead(&self, symbol_id: SymbolId) -> bool {
        self.dead.contains(symbol_id.index())
    }

    #[inline]
    fn is_declarator_dead(&self, symbol_id: SymbolId, scope_id: ScopeId) -> bool {
        self.dead.contains(symbol_id.index())
            && self.declarators.as_ref().is_some_and(|declarators| {
                declarators.function_declarators.contains(symbol_id.index())
                    && declarators.scopes.contains(scope_id.index())
            })
    }

    #[inline]
    fn is_function_declarator_candidate(&self, symbol_id: SymbolId) -> bool {
        self.declarators
            .as_ref()
            .is_some_and(|declarators| declarators.function_declarators.contains(symbol_id.index()))
    }

    /// Returns the nearest registered function whose body contains
    /// `scope_id`. Code there only runs when that function is called. `None`
    /// means the scope is not inside any registered function, so references
    /// there make their target unconditionally live.
    fn owner(&self, scoping: &Scoping, scope_id: ScopeId) -> Option<(SymbolId, ScopeId)> {
        scoping.scope_ancestors(scope_id).find_map(|scope_id| {
            self.function_by_scope
                .get(scope_id.index())
                .copied()
                .flatten()
                .map(|symbol_id| (symbol_id, scope_id))
        })
    }

    /// Rebuild current reachability and report whether this run published any
    /// newly dead function, requiring another peephole pass.
    fn analyze(&mut self, scoping: &Scoping, implicitly_observable: &BitSet<'_>) -> bool {
        if scoping.root_scope_flags().contains_direct_eval() {
            // The minifier may remove direct eval but must never create one,
            // so a graph that already published dead functions cannot get
            // here.
            debug_assert!(self.dead.is_empty(), "direct eval formed after liveness was published");
            // In release builds, stop later passes from consuming more
            // invalidated deadness. Declarations already removed cannot be
            // restored if a transform violated the direct-eval contract.
            self.dead.clear();
            return false;
        }

        self.scratch.reset();
        if let Some(declarators) = &mut self.declarators {
            declarators.reset();
        }

        // Phase 1: classify every current reference to a candidate. Seed
        // unconditional liveness, store candidate-owned dependencies, and
        // identify functions that no longer need graph reachability.
        for bit in self.candidates.ones() {
            let target = SymbolId::from_usize(bit);
            // Implicitly observable candidates (e.g. exported functions) are
            // unconditionally live regardless of their references.
            if implicitly_observable.contains(bit) {
                self.scratch.mark_live(target);
            }
            let mut has_registered_function_owner = false;
            for &reference_id in scoping.get_resolved_reference_ids(target) {
                let reference = scoping.get_reference(reference_id);
                let Some((owner, owner_scope_id)) = self.owner(scoping, reference.scope_id())
                else {
                    self.scratch.mark_live(target);
                    continue;
                };

                has_registered_function_owner = true;
                if implicitly_observable.contains(owner.index()) {
                    // A reference owned by a permanently live function keeps
                    // its target live unconditionally. Avoid storing and
                    // sorting the common implicitly observable owner case; the
                    // result is the same.
                    self.scratch.mark_live(target);
                    continue;
                }
                if self.candidates.contains(owner.index()) {
                    self.scratch.owned_references.push((owner, target));
                    if let Some(declarators) = &mut self.declarators
                        && declarators.scopes.contains(owner_scope_id.index())
                    {
                        declarators.scratch.with_graph_dependencies.set_bit(owner.index());
                    }
                } else if !scoping.symbol_is_unused(owner) {
                    // A registered non-candidate owner keeps the target live
                    // only while its body can still execute.
                    self.scratch.mark_live(target);
                }
            }
            if !has_registered_function_owner
                && !self.dead.contains(bit)
                && self
                    .declarators
                    .as_ref()
                    .is_none_or(|declarators| declarators.function_declarations.contains(bit))
            {
                self.scratch.candidates_to_untrack.set_bit(bit);
            }
        }
        for bit in self.candidates.ones() {
            if self.dead.contains(bit) {
                continue;
            }
            if let Some(declarators) = &mut self.declarators
                && declarators.function_declarators.contains(bit)
                && !declarators.scratch.with_graph_dependencies.contains(bit)
            {
                declarators.scratch.to_untrack.set_bit(bit);
            }
        }

        // Phase 2: untrack owners that no longer need graph reachability.
        // `owned_references` was collected before `candidates_to_untrack` was
        // known. Stored owners are not implicitly observable, so preserve
        // their targets when ordinary reference counts still make the owner
        // executable.
        for index in 0..self.scratch.owned_references.len() {
            let (owner, target) = self.scratch.owned_references[index];
            let owner_leaves_graph = self.owner_leaves_graph_after_untracking(owner);
            let owner_stays_live_by_count = !scoping.symbol_is_unused(owner);
            if owner_leaves_graph && owner_stays_live_by_count {
                self.scratch.mark_live(target);
            }
        }
        self.untrack_sources();

        #[cfg(debug_assertions)]
        for bit in self.dead.ones() {
            assert!(
                self.candidates.contains(bit),
                "dead function symbol {bit} was untracked; dead candidates must remain graph \
                 candidates",
            );
        }

        // Phase 3: propagate from unconditional live seeds through references
        // owned by live candidates.
        self.scratch.propagate_liveness(&self.candidates);

        #[cfg(debug_assertions)]
        for bit in self.dead.ones() {
            assert!(
                !self.scratch.live.contains(bit),
                "function liveness resurrected dead symbol {bit}; transforms must not create a \
                 new path to a previously unreachable binding",
            );
        }

        // Phase 4: publish newly unreachable candidates. Existing dead bits
        // remain as tombstones for the transform contract checks above.
        let mut published_new_dead = false;
        for bit in self.candidates.ones() {
            if !self.scratch.live.contains(bit) && !self.dead.contains(bit) {
                self.dead.set_bit(bit);
                published_new_dead = true;
            }
        }

        published_new_dead
    }

    fn owner_leaves_graph_after_untracking(&self, symbol_id: SymbolId) -> bool {
        let bit = symbol_id.index();
        let Some(declarators) = &self.declarators else {
            return self.scratch.candidates_to_untrack.contains(bit);
        };
        (!declarators.function_declarations.contains(bit)
            || self.scratch.candidates_to_untrack.contains(bit))
            && (!declarators.function_declarators.contains(bit)
                || declarators.scratch.to_untrack.contains(bit))
    }

    /// Apply source-specific untracking using sparse bitset iteration. Function
    /// declaration scopes remain conservative non-candidate owners. Declarator
    /// scopes are cleared in one batched sparse sweep before later transforms
    /// can move their function expressions.
    fn untrack_sources(&mut self) {
        let Some(declarators) = &mut self.declarators else {
            for bit in self.scratch.candidates_to_untrack.ones() {
                self.candidates.unset_bit(bit);
            }
            return;
        };

        for bit in self.scratch.candidates_to_untrack.ones() {
            declarators.function_declarations.unset_bit(bit);
            if !declarators.function_declarators.contains(bit) {
                self.candidates.unset_bit(bit);
            }
        }
        for bit in declarators.scratch.to_untrack.ones() {
            declarators.function_declarators.unset_bit(bit);
            if !declarators.function_declarations.contains(bit) {
                self.candidates.unset_bit(bit);
            }
        }

        for scope_index in declarators.scopes.ones() {
            let Some(symbol_id) = self.function_by_scope[scope_index] else {
                debug_assert!(false, "registered declarator scope has no owner");
                declarators.scratch.scopes_to_untrack.set_bit(scope_index);
                continue;
            };
            if !declarators.function_declarators.contains(symbol_id.index()) {
                declarators.scratch.scopes_to_untrack.set_bit(scope_index);
            }
        }
        for scope_index in declarators.scratch.scopes_to_untrack.ones() {
            self.function_by_scope[scope_index] = None;
            declarators.scopes.unset_bit(scope_index);
        }
    }
}

/// Source-specific metadata required only when exact function-valued
/// declarators participate in the graph.
struct FunctionDeclaratorCandidates<'a> {
    /// Candidate symbols with at least one active function declaration source.
    function_declarations: BitSet<'a>,
    /// Candidate symbols whose graph ownership originates from an exact
    /// function-valued variable declarator.
    function_declarators: BitSet<'a>,
    /// Function scopes owned by still-active declarator candidates. This
    /// makes graph deadness specific to an initializer site rather than every
    /// declaration that happens to share the same binding symbol.
    scopes: BitSet<'a>,
    scratch: FunctionDeclaratorScratch<'a>,
}

impl<'a> FunctionDeclaratorCandidates<'a> {
    fn new(symbols_len: usize, scopes_len: usize, allocator: &'a Allocator) -> Self {
        Self {
            function_declarations: BitSet::new_in(symbols_len, allocator),
            function_declarators: BitSet::new_in(symbols_len, allocator),
            scopes: BitSet::new_in(scopes_len, allocator),
            scratch: FunctionDeclaratorScratch::new(symbols_len, scopes_len, allocator),
        }
    }

    fn reset(&mut self) {
        self.scratch.reset();
    }
}

struct FunctionDeclaratorScratch<'a> {
    /// Declarators whose bodies have references to another candidate, so their
    /// scopes must remain graph owners.
    with_graph_dependencies: BitSet<'a>,
    /// Declarator sources whose scopes no longer own graph edges and can be
    /// unregistered before their expressions are inlined.
    to_untrack: BitSet<'a>,
    /// Declarator scope owners to clear after source untracking is settled.
    scopes_to_untrack: BitSet<'a>,
}

impl<'a> FunctionDeclaratorScratch<'a> {
    fn new(symbols_len: usize, scopes_len: usize, allocator: &'a Allocator) -> Self {
        Self {
            with_graph_dependencies: BitSet::new_in(symbols_len, allocator),
            to_untrack: BitSet::new_in(symbols_len, allocator),
            scopes_to_untrack: BitSet::new_in(scopes_len, allocator),
        }
    }

    fn reset(&mut self) {
        self.with_graph_dependencies.clear();
        self.to_untrack.clear();
        self.scopes_to_untrack.clear();
    }
}

struct GraphScratch<'a> {
    /// All functions proven live during the current analysis.
    live: BitSet<'a>,
    /// Live functions whose owned references have not been propagated yet.
    live_worklist: ArenaVec<'a, SymbolId>,
    /// `(owner, target)` for each reference to `target` inside `owner`'s body.
    owned_references: ArenaVec<'a, (SymbolId, SymbolId)>,
    /// Candidates that no longer need graph reachability and will stop being
    /// tracked after reference collection.
    candidates_to_untrack: BitSet<'a>,
}

impl<'a> GraphScratch<'a> {
    fn new(symbols_len: usize, allocator: &'a Allocator) -> Self {
        Self {
            live: BitSet::new_in(symbols_len, allocator),
            live_worklist: ArenaVec::new_in(&allocator),
            owned_references: ArenaVec::new_in(&allocator),
            candidates_to_untrack: BitSet::new_in(symbols_len, allocator),
        }
    }

    fn reset(&mut self) {
        self.live.clear();
        self.live_worklist.clear();
        self.owned_references.clear();
        self.candidates_to_untrack.clear();
    }

    fn mark_live(&mut self, symbol_id: SymbolId) {
        let bit = symbol_id.index();
        if !self.live.contains(bit) {
            self.live.set_bit(bit);
            self.live_worklist.push(symbol_id);
        }
    }

    /// Marks the direct targets referenced by `owner` as live.
    /// Requires `owned_references` to be sorted by owner.
    fn mark_targets_live(&mut self, owner: SymbolId) {
        let mut index = self
            .owned_references
            .partition_point(|&(reference_owner, _)| reference_owner.index() < owner.index());
        while let Some((reference_owner, target)) = self.owned_references.get(index).copied() {
            if reference_owner != owner {
                break;
            }
            self.mark_live(target);
            index += 1;
        }
    }

    fn propagate_liveness(&mut self, candidates: &BitSet<'_>) {
        // References owned by functions no longer in the graph cannot
        // propagate. Targets of count-live owners were seeded before this call.
        self.owned_references.retain(|(owner, _)| candidates.contains(owner.index()));
        // Group references by owner for `mark_targets_live`.
        self.owned_references.sort_unstable_by_key(|&(owner, _)| owner.index());
        while let Some(owner) = self.live_worklist.pop() {
            if !candidates.contains(owner.index()) {
                // The owner may have been seeded before it was untracked; its
                // targets were already handled by graph-independent liveness.
                continue;
            }
            self.mark_targets_live(owner);
        }
    }
}

/// Normalize hook: register a function declaration as a potential graph
/// candidate.
pub fn register_function(function: &Function<'_>, ctx: &mut TraverseCtx<'_>) {
    if !function.is_declaration() {
        return;
    }
    if ctx.options().unused == CompressOptionsUnused::Keep {
        return;
    }
    let source_type = ctx.source_type();
    let allocator = ctx.allocator();
    let TraverseCtx { state, scoping, .. } = ctx;
    if let Some(liveness) = state.symbols.liveness_mut() {
        liveness.register_function(function, source_type, scoping.scoping(), allocator);
    }
}

/// Normalize hook: register an exact function-valued declarator that has a
/// guaranteed removal site. This intentionally excludes declarations in bare
/// statement slots and loop heads whose binding is part of the loop's runtime
/// semantics.
pub fn register_function_declarator(decl: &VariableDeclarator<'_>, ctx: &mut TraverseCtx<'_>) {
    if ctx.options().unused == CompressOptionsUnused::Keep {
        return;
    }
    let Ancestor::VariableDeclarationDeclarations(declaration) = ctx.parent() else {
        unreachable!("variable declarator must have a variable declaration parent");
    };
    if declaration.kind().is_using() || !declarator_has_removal_site(ctx) {
        return;
    }

    let allocator = ctx.allocator();
    let TraverseCtx { state, scoping, .. } = ctx;
    if let Some(liveness) = state.symbols.liveness_mut() {
        liveness.register_function_declarator(decl, allocator, scoping.scoping());
    }
}

/// Return the scope that owns references in an exact function-valued
/// declarator initializer.
pub fn function_declarator_scope_id(decl: &VariableDeclarator<'_>) -> Option<ScopeId> {
    decl.init.as_ref().and_then(|init| match init {
        Expression::FunctionExpression(function) => function.scope_id.get(),
        Expression::ArrowFunctionExpression(arrow) => arrow.scope_id.get(),
        _ => None,
    })
}
fn declarator_has_removal_site(ctx: &TraverseCtx<'_>) -> bool {
    let parent_is_statement_list = |ancestor: Ancestor<'_, '_>| {
        matches!(
            ancestor,
            Ancestor::ProgramBody(_)
                | Ancestor::FunctionBodyStatements(_)
                | Ancestor::BlockStatementBody(_)
                | Ancestor::SwitchCaseConsequent(_)
                | Ancestor::StaticBlockBody(_)
                | Ancestor::TSModuleBlockBody(_)
        )
    };

    match ctx.ancestor(1) {
        ancestor if parent_is_statement_list(ancestor) => true,
        Ancestor::ForStatementInit(_) => parent_is_statement_list(ctx.ancestor(2)),
        _ => false,
    }
}

/// Normalize hook: record resource bindings observed later by explicit
/// resource management. Disposal reads the bound value through runtime state,
/// not through a resolved identifier reference in the AST.
pub fn register_using_declaration(
    declaration: &VariableDeclaration<'_>,
    ctx: &mut TraverseCtx<'_>,
) {
    if !declaration.kind.is_using() {
        return;
    }

    let source_type = ctx.source_type();
    let allocator = ctx.allocator();
    let TraverseCtx { state, scoping, .. } = ctx;
    let liveness = state
        .symbols
        .ensure_liveness(|| SymbolLiveness::new(source_type, scoping.scoping(), allocator));
    liveness.mark_bound_names(declaration);
}

/// Normalize hook: record runtime bindings exposed by an exported declaration.
pub fn register_export_declaration(declaration: &ExportDeclaration<'_>, ctx: &mut TraverseCtx<'_>) {
    let TraverseCtx { state, .. } = ctx;
    let Some(liveness) = state.symbols.liveness_mut() else { return };
    if !declaration.export_kind().is_type() {
        liveness.mark_bound_names(&declaration.declaration);
    }
}

/// Normalize hook: record runtime bindings exposed by a local named export.
pub fn register_named_export(declaration: &ExportNamedDeclaration<'_>, ctx: &mut TraverseCtx<'_>) {
    if declaration.export_kind.is_type() {
        return;
    }

    let TraverseCtx { state, scoping, .. } = ctx;
    let Some(liveness) = state.symbols.liveness_mut() else { return };
    for specifier in &declaration.specifiers {
        if specifier.export_kind.is_type() {
            continue;
        }
        let ModuleExportName::IdentifierReference(local) = &specifier.local else { continue };
        let Some(reference_id) = local.reference_id.get() else { continue };
        let reference = scoping.scoping().get_reference(reference_id);
        if reference.flags().is_type_only() {
            continue;
        }
        if let Some(symbol_id) = reference.symbol_id() {
            liveness.mark_implicitly_observable(symbol_id);
        }
    }
}

/// Normalize hook: record the local binding of a named default function or
/// class declaration. `export default identifier` is intentionally excluded:
/// it exports the evaluated value, not subsequent writes to that local binding.
pub fn register_default_export(
    declaration: &ExportDefaultDeclaration<'_>,
    ctx: &mut TraverseCtx<'_>,
) {
    let symbol_id = match &declaration.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            function.id.as_ref().and_then(|id| id.symbol_id.get())
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            class.id.as_ref().and_then(|id| id.symbol_id.get())
        }
        _ => None,
    };
    if let Some(symbol_id) = symbol_id
        && let Some(liveness) = ctx.state.symbols.liveness_mut()
    {
        liveness.mark_implicitly_observable(symbol_id);
    }
}

/// Whether pruning this pass's dead references can change a graph input.
///
/// Reachability is immediately sensitive to resolved-reference-list changes of
/// candidate target symbols. It also checks whether registered non-candidate
/// owners are count-live. Removing the last reference to such an owner can only
/// make the current result conservatively stale: direct eval disables the
/// analysis, while implicitly observable owners take a separate branch. A
/// later pass removes the count-dead owner; removing its body then drops
/// candidate-target references and triggers recomputation. Fresh references
/// cannot resurrect a published dead function, so additions alone need no
/// recompute. Scope-only rewrites preserve the nearest function owner.
pub fn dead_references_affect_analysis(ctx: &TraverseCtx<'_>) -> bool {
    let Some(liveness) = ctx.state.symbols.liveness() else { return false };
    let Some(graph) = &liveness.recursive_functions else { return false };
    // The analysis is disabled while the root scope contains direct eval, so
    // removed references cannot affect it. The flag may be stale if this pass
    // dropped the last eval (flags refresh after this check), but that drop
    // also sets `direct_eval_dropped`, which forces the recompute anyway.
    if ctx.scoping().root_scope_flags().contains_direct_eval() {
        return false;
    }
    ctx.state.pass_changes.removed_references.ones().any(|bit| {
        ctx.scoping()
            .get_reference(ReferenceId::from_usize(bit))
            .symbol_id()
            .is_some_and(|symbol_id| graph.candidates.contains(symbol_id.index()))
    })
}

/// Assert that the completed pass removed every previously published dead
/// declaration, then optionally analyze settled semantic references and
/// publish newly dead functions. Called only by pass completion after
/// scoping has been flushed.
pub fn analyze<'a>(program: &Program<'a>, ctx: &mut TraverseCtx<'a>, recompute: bool) -> bool {
    #[cfg(not(debug_assertions))]
    let _ = program;

    #[cfg(debug_assertions)]
    if let Some((dead, declarator_scopes)) =
        ctx.state.symbols.liveness().and_then(SymbolLiveness::dead_declarations)
    {
        debug_assert_dead_function_declarations_removed(
            program,
            ctx.scoping(),
            dead,
            declarator_scopes,
        );
    }

    if !recompute {
        return false;
    }

    let TraverseCtx { state, scoping, .. } = ctx;
    state.symbols.liveness_mut().is_some_and(|liveness| liveness.analyze(scoping.scoping()))
}

/// Debug contract for previously published graph deadness: it is consumed only
/// at function-declaration sites, and every such declaration must now be gone.
#[cfg(debug_assertions)]
fn debug_assert_dead_function_declarations_removed(
    program: &Program<'_>,
    scoping: &Scoping,
    dead: &BitSet<'_>,
    declarator_scopes: Option<&BitSet<'_>>,
) {
    if dead.is_empty() {
        return;
    }
    DeadFunctionSweep { scoping, dead, declarator_scopes }.visit_program(program);
}

#[cfg(debug_assertions)]
struct DeadFunctionSweep<'s, 'd, 'a> {
    scoping: &'s Scoping,
    dead: &'d BitSet<'a>,
    declarator_scopes: Option<&'d BitSet<'a>>,
}

#[cfg(debug_assertions)]
impl<'a> VisitJs<'a> for DeadFunctionSweep<'_, '_, '_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if function.is_declaration()
            && let Some(symbol_id) = function.id.as_ref().and_then(|id| id.symbol_id.get())
        {
            assert!(
                !self.dead.contains(symbol_id.index()),
                "dead function `{}` survived the pass after its deadness was published",
                self.scoping.symbol_name(symbol_id),
            );
        }
        walk_function(self, function, flags);
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(id) = &decl.id
            && let Some(symbol_id) = id.symbol_id.get()
            && let Some(scope_id) = function_declarator_scope_id(decl)
        {
            assert!(
                !self.dead.contains(symbol_id.index())
                    || !self.declarator_scopes.is_some_and(
                        |declarator_scopes| declarator_scopes.contains(scope_id.index())
                    ),
                "dead function-valued declarator `{}` survived the pass after its deadness was \
                 published",
                self.scoping.symbol_name(symbol_id),
            );
        }
        walk_variable_declarator(self, decl);
    }
}
