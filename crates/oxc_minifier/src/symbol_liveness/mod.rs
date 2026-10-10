//! Symbol liveness: observability that reference counts cannot express, and
//! reachability for recursively referenced function declarations.
//!
//! Reference counting removes an acyclic unused declaration once its last
//! reference disappears, but it cannot remove `function a() { b() }
//! function b() { a() }`: each function keeps the other's count non-zero.
//! This module adds the missing reachability question: can executing code
//! reach a candidate function declaration?
//!
//! Four concepts define the analysis:
//!
//! 1. **Candidates** are eligible function declarations whose deadness may
//!    require graph reachability. Normalize initially registers every eligible
//!    declaration. Analysis permanently untracks a non-dead candidate once no
//!    current reference has a registered function owner; it no longer needs
//!    graph reachability, so ordinary removal checks handle it from then on.
//!    Published-dead candidates remain tracked to enforce monotonicity. Other
//!    declaration kinds continue to use reference counts.
//!    Self-recursive function and arrow expressions in declarator initializers
//!    are handled by a local check at their removal site
//!    (`self_recursive_function_declarator_is_unused`) and need no graph state.
//! 2. **Ownership** comes from semantic scopes. The nearest registered function
//!    declaration is a reference's owner. A reference from a candidate that is
//!    not implicitly observable is stored as `(owner, target)` and propagates
//!    when the owner becomes live. Implicitly observable owners and references
//!    outside registered functions make their targets live immediately. A
//!    registered non-candidate owner does so only while implicitly observable
//!    or count-live, because a count-dead owner's body never executes.
//! 3. **Implicit observability** is stable metadata for bindings whose runtime
//!    value can be observed without a resolved reference in this AST. A symbol
//!    here may have any number of references; the bit records that an observer
//!    remains even if every reference disappears. The five channels are module
//!    exports, Script-root bindings, mapped arguments, Annex B aliases, and
//!    `using` disposal. This metadata protects every count-based removal
//!    consumer, not only declaration removal. The mapped-arguments channel is
//!    the exception to stability because it is derived from lexical
//!    reachability: a parameter of a sloppy function with a simple parameter
//!    list is observable only while that function's `arguments` object is
//!    reachable (see `MappedArguments`). Like the direct-eval flag, it can
//!    only clear.
//! 4. **Analysis runs after scoping is flushed.** Every result is derived from
//!    the settled resolved-reference lists, so AST rewrites need no parallel
//!    collection hooks or behind-the-cursor repair log.
//!
//! ## Transform contract
//!
//! Functions are registered once during Normalize and the graph is recomputed
//! only when its settled inputs can change. Peephole transforms must preserve
//! five invariants:
//!
//! 1. **Do not create function declarations.** A new declaration would have no
//!    entry in `function_by_scope` and therefore could not own references or
//!    become a candidate. Removing an existing declaration is supported.
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
//! 5. **Do not create an `arguments` reference in, or move one into, a function
//!    that had none.** A mapped parameter stops being implicitly observable
//!    once its function can no longer reach its `arguments` object, and that
//!    only ever clears (dropping a reference re-derives it). A reference
//!    appearing later could observe writes that were already removed. Debug
//!    builds assert this never happens.
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
//! - Only function declarations are candidates. Mutual declarator cycles
//!   (`const a = () => b(); const b = () => a();`) and class cycles are kept.
//!   Making them candidates measured zero output bytes on real bundles but
//!   caused most of the wrong-code hazards, and class removal has no stable
//!   proof (`remove_unused_class` extracts static values, and heritage
//!   classification can change between passes).
//! - Any direct eval propagates to the root scope and disables the analysis for
//!   the whole program. There is no per-scope granularity.
//! - Sloppy unhoisted Annex B block functions are always kept, even when
//!   nothing can reach them. Removing them requires modeling the runtime
//!   var-alias write.

use oxc_allocator::{Allocator, ArenaVec, BitSet, GetAllocator};
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitJs, walk_js::walk_function};
use oxc_ecmascript::{BoundNames, IsSimpleParameterList};
use oxc_semantic::Scoping;
use oxc_span::SourceType;
use oxc_syntax::{
    reference::ReferenceId,
    scope::{ScopeFlags, ScopeId},
    symbol::SymbolId,
};

use crate::{CompressOptions, CompressOptionsUnused, TraverseCtx};

/// Stable program-wide symbol facts plus the optional recursive-function graph.
///
/// This is always present for ES modules and Script sources because the
/// implicit observability of exported and Script-root bindings must protect
/// count-based optimizations even when recursive function removal is
/// disabled. For CommonJS sources it exists whenever unused-declaration
/// removal is enabled. Otherwise a `using` declaration or a sloppy function
/// with a simple parameter list creates it lazily, because disposal and
/// mapped arguments observe those bindings without an ordinary reference.
pub struct SymbolLiveness<'a> {
    /// Bindings with a runtime observer independent of resolved references:
    /// module exports, Script roots, mapped arguments, Annex B aliases, and
    /// `using` disposal.
    implicitly_observable: BitSet<'a>,
    /// Created by the first sloppy function with a simple parameter list.
    mapped_arguments: Option<MappedArguments<'a>>,
    recursive_functions: Option<FunctionGraph<'a>>,
}

/// Parameters aliased by the `arguments` object of their function, and the
/// functions whose `arguments` object is still reachable.
///
/// In a sloppy function with a simple parameter list, `arguments[i]` reads and
/// writes parameter `i`. Only code holding that function's own `arguments`
/// object can use the alias: an identifier named `arguments` positioned in the
/// function or in one of its nested arrow functions, or a direct eval. A
/// nested non-arrow function has its own object.
///
/// References are matched by name and position, not by resolution. Semantic
/// has no implicit `arguments` binding, so a reference in a nested function
/// can resolve to an outer binding of that name although it reads the nested
/// function's own object at runtime. The non-standard `f.arguments` and
/// `f.caller` are not modeled.
struct MappedArguments<'a> {
    /// Parameters of sloppy functions with simple parameter lists.
    parameters: BitSet<'a>,
    /// Scopes of non-arrow functions with an identifier named `arguments`
    /// positioned in them or in their nested arrow functions.
    arguments_scopes: BitSet<'a>,
    /// Whether any of `parameters` is still implicitly observable. Set by
    /// each derivation and gates the refresh.
    any_observable: bool,
}

impl<'a> MappedArguments<'a> {
    fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        Self {
            parameters: BitSet::new_in(scoping.symbols_len(), allocator),
            // No pass creates a function scope, so this covers every one.
            arguments_scopes: BitSet::new_in(scoping.scopes_len(), allocator),
            any_observable: false,
        }
    }
}

impl<'a> SymbolLiveness<'a> {
    /// Return `None` only for a CommonJS source with unused-declaration
    /// removal disabled. Its fresh state has nothing seeded, so Normalize
    /// creates it when it finds a `using` binding or a parameter exposed
    /// through mapped arguments. A Script source always has the state because
    /// later scripts observe its root bindings regardless of the `unused`
    /// option.
    pub fn new_if_enabled(
        source_type: SourceType,
        options: &CompressOptions,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) -> Option<Self> {
        let recursive_functions_enabled = options.unused != CompressOptionsUnused::Keep;
        if !source_type.is_module() && !source_type.is_script() && !recursive_functions_enabled {
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
        Self { implicitly_observable, mapped_arguments: None, recursive_functions: None }
    }

    #[inline]
    pub fn is_implicitly_observable(&self, symbol_id: SymbolId) -> bool {
        self.implicitly_observable.contains(symbol_id.index())
    }

    #[inline]
    pub fn function_is_dead(&self, symbol_id: SymbolId) -> bool {
        self.recursive_functions.as_ref().is_some_and(|graph| graph.is_dead(symbol_id))
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

    /// Record parameters aliased by `arguments`. They start implicitly
    /// observable and [`Self::derive_mapped_parameters`] can only clear them.
    fn register_mapped_parameters(
        &mut self,
        params: &FormalParameters<'_>,
        scoping: &Scoping,
        allocator: &'a Allocator,
    ) {
        let mapped =
            self.mapped_arguments.get_or_insert_with(|| MappedArguments::new(scoping, allocator));
        params.bound_names(&mut |ident| {
            if let Some(symbol_id) = ident.symbol_id.get() {
                mapped.parameters.set_bit(symbol_id.index());
                self.implicitly_observable.set_bit(symbol_id.index());
            }
        });
    }

    /// Clear the implicit observability of every mapped parameter whose
    /// function can no longer reach its `arguments` object. Returns whether
    /// any parameter was cleared.
    ///
    /// In a valid program no other channel marks a parameter, so clearing the
    /// bit cannot hide another observer.
    fn derive_mapped_parameters(&mut self, scoping: &Scoping) -> bool {
        let Some(mapped) = &mut self.mapped_arguments else { return false };
        let mut any_observable = false;
        let mut cleared = false;
        for bit in mapped.parameters.ones() {
            if !self.implicitly_observable.contains(bit) {
                continue;
            }
            let scope_id = scoping.symbol_scope_id(SymbolId::from_usize(bit));
            if mapped.arguments_scopes.contains(scope_id.index())
                || scoping.scope_flags(scope_id).contains_direct_eval()
            {
                any_observable = true;
            } else {
                self.implicitly_observable.unset_bit(bit);
                cleared = true;
            }
        }
        mapped.any_observable = any_observable;
        cleared
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
        graph.register(scope_id, symbol_id);
    }

    fn analyze(&mut self, scoping: &Scoping) -> bool {
        let Some(graph) = &mut self.recursive_functions else { return false };
        graph.analyze(scoping, &self.implicitly_observable)
    }

    #[cfg(debug_assertions)]
    fn dead_functions(&self) -> Option<&BitSet<'a>> {
        self.recursive_functions.as_ref().map(|graph| &graph.dead)
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
            function_by_scope,
            dead: BitSet::new_in(symbols_len, allocator),
            scratch: GraphScratch::new(symbols_len, allocator),
        }
    }

    fn register(&mut self, scope_id: ScopeId, symbol_id: SymbolId) {
        self.function_by_scope[scope_id.index()] = Some(symbol_id);
        self.candidates.set_bit(symbol_id.index());
    }

    #[inline]
    fn is_dead(&self, symbol_id: SymbolId) -> bool {
        self.dead.contains(symbol_id.index())
    }

    /// Returns the nearest registered function whose body contains
    /// `scope_id`. Code there only runs when that function is called. `None`
    /// means the scope is not inside any registered function, so references
    /// there make their target unconditionally live.
    fn owner(&self, scoping: &Scoping, scope_id: ScopeId) -> Option<SymbolId> {
        scoping
            .scope_ancestors(scope_id)
            .find_map(|scope_id| self.function_by_scope.get(scope_id.index()).copied().flatten())
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
                let Some(owner) = self.owner(scoping, reference.scope_id()) else {
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
                } else if !scoping.symbol_is_unused(owner) {
                    // A registered non-candidate owner keeps the target live
                    // only while its body can still execute.
                    self.scratch.mark_live(target);
                }
            }
            if !has_registered_function_owner && !self.dead.contains(bit) {
                self.scratch.candidates_to_untrack.set_bit(bit);
            }
        }

        // Phase 2: untrack owners that no longer need graph reachability.
        // `owned_references` was collected before `candidates_to_untrack` was
        // known. Stored owners are not implicitly observable, so preserve
        // their targets when ordinary reference counts still make the owner
        // executable.
        for index in 0..self.scratch.owned_references.len() {
            let (owner, target) = self.scratch.owned_references[index];
            let owner_leaves_graph = self.scratch.candidates_to_untrack.contains(owner.index());
            let owner_stays_live_by_count = !scoping.symbol_is_unused(owner);
            if owner_leaves_graph && owner_stays_live_by_count {
                self.scratch.mark_live(target);
            }
        }
        // Future analyses treat these functions as registered non-candidate
        // owners.
        for bit in self.scratch.candidates_to_untrack.ones() {
            self.candidates.unset_bit(bit);
        }

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

/// Normalize hook: record parameters exposed through mapped arguments and
/// register function declarations as potential graph candidates.
pub fn register_function(function: &Function<'_>, ctx: &mut TraverseCtx<'_>) {
    let recursive_functions_enabled = ctx.options().unused != CompressOptionsUnused::Keep;
    let source_type = ctx.source_type();
    let allocator = ctx.allocator();
    let TraverseCtx { state, scoping, .. } = ctx;
    // Sloppy functions with simple parameter lists alias their parameters
    // through `arguments`, even when a parameter has no resolved reads.
    // A `var` or function redeclaration must not make its writes removable.
    // Single-use substitution runs even when unused declarations are kept, so
    // the parameters are recorded regardless of the `unused` option.
    if !scoping.scoping().scope_flags(function.scope_id()).is_strict_mode()
        && function.params.is_simple_parameter_list()
    {
        let liveness = state
            .symbols
            .ensure_liveness(|| SymbolLiveness::new(source_type, scoping.scoping(), allocator));
        liveness.register_mapped_parameters(&function.params, scoping.scoping(), allocator);
    }
    if recursive_functions_enabled
        && function.is_declaration()
        && let Some(liveness) = state.symbols.liveness_mut()
    {
        liveness.register_function(function, source_type, scoping.scoping(), allocator);
    }
}

/// Normalize hook: record the function whose `arguments` object the current
/// identifier named `arguments` reaches.
pub fn register_arguments_reference(ctx: &mut TraverseCtx<'_>) {
    let TraverseCtx { state, scoping, .. } = ctx;
    // Functions are registered before their bodies are visited, so absent
    // state means the enclosing function has no mapped parameter.
    let Some(mapped) =
        state.symbols.liveness_mut().and_then(|liveness| liveness.mapped_arguments.as_mut())
    else {
        return;
    };
    let function_scope_id = scoping.ancestor_scopes().find(|&scope_id| {
        let scope_flags = scoping.scoping().scope_flags(scope_id);
        scope_flags.is_function() && !scope_flags.is_arrow()
    });
    if let Some(scope_id) = function_scope_id {
        mapped.arguments_scopes.set_bit(scope_id.index());
    }
}

/// Normalize hook: every function and `arguments` reference has been
/// recorded, so settle which mapped parameters are observable.
pub fn finish_registration(ctx: &mut TraverseCtx<'_>) {
    let TraverseCtx { state, scoping, .. } = ctx;
    if let Some(liveness) = state.symbols.liveness_mut() {
        liveness.derive_mapped_parameters(scoping.scoping());
    }
}

/// Re-derive which mapped parameters are observable after a pass dropped an
/// `arguments` identifier or a direct eval. Returns whether any parameter
/// stopped being observable.
///
/// Direct-eval scope flags must already be refreshed. The walk for live
/// `arguments` identifiers is skipped unless one was dropped, because
/// dropping a direct eval leaves the recorded scopes exact.
pub fn refresh_mapped_parameters(
    program: &Program<'_>,
    ctx: &mut TraverseCtx<'_>,
    arguments_dropped: bool,
) -> bool {
    let TraverseCtx { state, scoping, .. } = ctx;
    let Some(liveness) = state.symbols.liveness_mut() else { return false };
    let Some(mapped) = &mut liveness.mapped_arguments else { return false };
    if !mapped.any_observable {
        return false;
    }
    if arguments_dropped {
        mapped.arguments_scopes.clear();
        LiveArgumentsCollector { scopes: &mut mapped.arguments_scopes, function_scope_id: None }
            .visit_program(program);
    }
    liveness.derive_mapped_parameters(scoping.scoping())
}

/// Collects the scopes of non-arrow functions that still have an identifier
/// named `arguments` positioned in them or in their nested arrow functions.
struct LiveArgumentsCollector<'s, 'a> {
    scopes: &'s mut BitSet<'a>,
    /// Scope of the nearest enclosing non-arrow function.
    function_scope_id: Option<ScopeId>,
}

impl<'a> VisitJs<'a> for LiveArgumentsCollector<'_, '_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let outer_scope_id =
            std::mem::replace(&mut self.function_scope_id, function.scope_id.get());
        walk_function(self, function, flags);
        self.function_scope_id = outer_scope_id;
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if ident.name == "arguments"
            && let Some(scope_id) = self.function_scope_id
        {
            self.scopes.set_bit(scope_id.index());
        }
    }
}

/// Debug contract for the mapped-arguments channel: no identifier named
/// `arguments` may be positioned in a function that has a mapped parameter
/// treated as unobservable. Observability is only re-derived when such an
/// identifier is dropped, so it is sound only while no pass creates one in,
/// or moves one into, a function that had none.
#[cfg(debug_assertions)]
pub fn debug_assert_no_stale_mapped_parameters(program: &Program<'_>, ctx: &TraverseCtx<'_>) {
    let Some(liveness) = ctx.state.symbols.liveness() else { return };
    let Some(mapped) = &liveness.mapped_arguments else { return };
    StaleMappedParameterCheck { liveness, mapped, has_unobservable_parameter: false }
        .visit_program(program);
}

#[cfg(debug_assertions)]
struct StaleMappedParameterCheck<'s, 'a> {
    liveness: &'s SymbolLiveness<'a>,
    mapped: &'s MappedArguments<'a>,
    /// Whether the nearest enclosing non-arrow function has a mapped
    /// parameter that is not implicitly observable.
    has_unobservable_parameter: bool,
}

#[cfg(debug_assertions)]
impl<'a> VisitJs<'a> for StaleMappedParameterCheck<'_, '_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        let mut has_unobservable_parameter = false;
        function.params.bound_names(&mut |ident| {
            if let Some(symbol_id) = ident.symbol_id.get() {
                has_unobservable_parameter |= self.mapped.parameters.contains(symbol_id.index())
                    && !self.liveness.is_implicitly_observable(symbol_id);
            }
        });
        let outer =
            std::mem::replace(&mut self.has_unobservable_parameter, has_unobservable_parameter);
        walk_function(self, function, flags);
        self.has_unobservable_parameter = outer;
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        assert!(
            ident.name != "arguments" || !self.has_unobservable_parameter,
            "stale mapped parameters: an `arguments` identifier is positioned in a function \
             whose parameters are treated as unobservable; a pass created it or moved it there",
        );
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
    if let Some(dead) = ctx.state.symbols.liveness().and_then(SymbolLiveness::dead_functions) {
        debug_assert_dead_function_declarations_removed(program, ctx.scoping(), dead);
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
) {
    if dead.is_empty() {
        return;
    }
    DeadFunctionSweep { scoping, dead }.visit_program(program);
}

#[cfg(debug_assertions)]
struct DeadFunctionSweep<'s, 'd, 'a> {
    scoping: &'s Scoping,
    dead: &'d BitSet<'a>,
}

#[cfg(debug_assertions)]
impl<'a> VisitJs<'a> for DeadFunctionSweep<'_, '_, '_> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
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
}
