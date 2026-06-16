use std::iter::{self, repeat_with};

use itertools::Itertools;
use keep_names::collect_name_symbols;
use oxc_index::IndexVec;
use oxc_syntax::class::ClassId;
use rustc_hash::{FxHashMap, FxHashSet};

use base54::base54;
use oxc_allocator::{Allocator, BitSet, HashSet, Vec};
use oxc_ast::ast::{Declaration, Program, Statement};
use oxc_data_structures::inline_string::InlineString;
use oxc_semantic::{AstNodes, Reference, Scoping, Semantic, SemanticBuilder, Stats, SymbolId};
use oxc_span::SourceType;
use oxc_str::{CompactStr, Ident, Str};

pub(crate) mod base54;
mod keep_names;

pub use keep_names::MangleOptionsKeepNames;

#[derive(Default, Debug, Clone, Copy)]
pub struct MangleOptions {
    /// Pass true to mangle names declared in the top level scope.
    ///
    /// Default: `true` for [`ModuleKind::Module`] and [`ModuleKind::CommonJS`]. Otherwise `false`.
    ///
    /// [`ModuleKind::Module`]: oxc_span::ModuleKind::Module
    /// [`ModuleKind::CommonJS`]: oxc_span::ModuleKind::CommonJS
    pub top_level: Option<bool>,

    /// Keep function / class names
    pub keep_names: MangleOptionsKeepNames,

    /// Use more readable mangled names
    /// (e.g. `slot_0`, `slot_1`, `slot_2`, ...) for debugging.
    ///
    /// Uses base54 if false.
    pub debug: bool,
}

impl MangleOptions {
    fn top_level(self, source_type: SourceType) -> bool {
        self.top_level.unwrap_or(source_type.is_module() || source_type.is_commonjs())
    }
}

type Slot = u32;

/// Sentinel for symbols the main assignment pass skipped; repaired below.
/// Safe because `SymbolId` is `NonMaxU32`, so `symbols_len` maxes out at
/// `u32::MAX - 1` and real slot values can never reach `Slot::MAX`.
const SLOT_UNASSIGNED: Slot = Slot::MAX;

/// Enum to handle both owned and borrowed allocators. This is not `Cow` because that type
/// requires `ToOwned`/`Clone`, which is not implemented for `Allocator`. Although this does
/// incur some pointer indirection on each reference to the allocator, it allows the API to be
/// more ergonomic by either accepting an existing allocator, or allowing an internal one to
/// be created and used temporarily automatically.
enum TempAllocator<'t> {
    Owned(Allocator),
    Borrowed(&'t Allocator),
}

impl TempAllocator<'_> {
    /// Get a reference to the allocator, regardless of whether it's owned or borrowed
    fn as_ref(&self) -> &Allocator {
        match self {
            TempAllocator::Owned(allocator) => allocator,
            TempAllocator::Borrowed(allocator) => allocator,
        }
    }
}

pub struct ManglerReturn {
    pub scoping: Scoping,
    /// A vector where each element corresponds to a class in declaration order.
    /// Each element is a mapping from original private member names to their mangled names.
    pub class_private_mappings: IndexVec<ClassId, FxHashMap<String, CompactStr>>,
}

/// # Name Mangler / Symbol Minification
///
/// ## Example
///
/// ```rust,ignore
/// use oxc_codegen::{Codegen, CodegenOptions};
/// use oxc_ast::ast::Program;
/// use oxc_parser::Parser;
/// use oxc_allocator::Allocator;
/// use oxc_span::SourceType;
/// use oxc_mangler::{MangleOptions, Mangler};
///
/// let allocator = Allocator::default();
/// let source = "const result = 1 + 2;";
/// let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
/// assert!(parsed.diagnostics.is_empty());
///
/// let mangled_symbols = Mangler::new()
///     .with_options(MangleOptions { top_level: true, debug: true })
///     .build(&parsed.program);
///
/// let js = Codegen::new().with_symbol_table(mangled_symbols).build(&parsed.program);
/// // this will be `const a = 1 + 2;` if debug = false
/// assert_eq!(js.code, "const slot_0 = 1 + 2;\n");
/// ```
///
/// ## Implementation
///
/// See:
///   * [esbuild](https://github.com/evanw/esbuild/blob/v0.24.0/docs/architecture.md#symbol-minification)
///
/// This algorithm is based on the implementation of esbuild and additionally implements improved name reuse functionality.
/// It targets for better gzip compression.
///
/// A slot is a placeholder for binding identifiers that shares the same name.
/// Visually, it is the index position for binding identifiers:
///
/// ```javascript
/// function slot0(slot1, slot2, slot3) {
///     slot2 = 1;
/// }
/// function slot1(slot0) {
///     function slot2() {
///         slot0 = 1;
///     }
/// }
/// ```
///
/// The slot number for a new scope starts after the maximum slot of the parent scope.
///
/// Occurrences of slots and their corresponding newly assigned short identifiers are:
/// - slot2: 3 - a
/// - slot0: 2 - b
/// - slot1: 2 - c
/// - slot3: 1 - d
///
/// After swapping out the mangled names:
///
/// ```javascript
/// function b(c, a, d) {
///     a = 1;
/// }
/// function c(b) {
///     function a() {
///         b = 1;
///     }
/// }
/// ```
///
/// ### Name Reuse Calculation
///
/// This improvement was inspired by [evanw/esbuild#2614](https://github.com/evanw/esbuild/pull/2614).
///
/// For better compression, we shadow the variables where possible to reuse the same name.
/// For example, the following code:
/// ```javascript
/// var top_level_a = 0;
/// var top_level_b = 1;
/// function foo() {
///   var foo_a = 1;
///   console.log(top_level_b, foo_a);
/// }
/// function bar() {
///   var bar_a = 1;
///   console.log(top_level_b, bar_a);
/// }
/// console.log(top_level_a, foo(), bar())
/// ```
/// `top_level_a` is declared in the root scope, but is not used in function `foo` and function `bar`.
/// Therefore, we can reuse the same name for `top_level_a` and `foo_a` and `bar_a`.
///
/// To calculate whether the variable name can be reused in the descendant scopes,
/// this mangler introduces a concept of symbol liveness and slot liveness.
/// Symbol liveness is a subtree of the scope tree that contains the declared scope of the symbol and
/// all the scopes that the symbol is used in. It is a subtree, so any scopes that are between the declared scope and the used scope
/// are also included. This is to ensure that the symbol is not shadowed by a different symbol before the use in the descendant scope.
///
/// For the example above, the liveness of each symbols are:
/// - `top_level_a`: root_scope
/// - `top_level_b`: root_scope -> foo, root_scope -> bar
/// - `foo_a`: root_scope -> foo
/// - `bar_a`: root_scope -> bar
/// - `foo`: root_scope
/// - `bar`: root_scope
///
/// Slot liveness is the same as symbol liveness, but it is a subforest (multiple subtrees) of the scope tree that can contain
/// multiple symbol liveness.
///
/// Now that we have the liveness of each symbol, we want to assign symbols to minimal number of slots.
/// This is a graph coloring problem where the node of the graph is the symbol and the edge of the graph indicates whether
/// the symbols has a common alive scope and the color of the node is the slot.
/// This mangler uses a greedy algorithm to assign symbols to slots to achieve that.
/// In other words, it assigns symbols to the first slot that does not live in the liveness of the symbol.
/// For the example above, each symbol is assigned to the following slots:
/// - slot 0: `top_level_a`
/// - slot 1: `top_level_b`, `foo_a`, `bar_a`
/// - slot 2: `foo`
/// - slot 3: `bar`
pub struct Mangler<'t> {
    options: MangleOptions,
    /// Statistics from a prior semantic build of the same program, used to
    /// pre-allocate the semantic data [`Mangler::build`] constructs.
    stats: Option<Stats>,
    /// An allocator meant to be used for temporary allocations during mangling.
    /// It can be cleared after mangling is done, to free up memory for subsequent
    /// files or other operations.
    temp_allocator: TempAllocator<'t>,
}

impl Default for Mangler<'_> {
    fn default() -> Self {
        Self {
            options: MangleOptions::default(),
            stats: None,
            temp_allocator: TempAllocator::Owned(Allocator::default()),
        }
    }
}

impl<'t> Mangler<'t> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Mangler` using an existing temporary allocator. This is an allocator
    /// that can be reset after mangling and is only used for temporary allocations during
    /// the mangling process. This makes processing multiple files at once much more efficient,
    /// because the same memory can be used for mangling each file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oxc_allocator::Allocator;
    /// use oxc_mangler::Mangler;
    /// use oxc_parser::Parser;
    /// use oxc_span::SourceType;
    ///
    /// let allocator = Allocator::default();
    /// let mut temp_allocator = Allocator::default();
    /// let source = "function myFunction(param) { return param + 1; }";
    ///
    /// let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    /// let mangled_symbols = Mangler::new_with_temp_allocator(&temp_allocator)
    ///     .build(&parsed.program);
    ///
    /// // Reset the allocator to free temporary memory
    /// temp_allocator.reset();
    /// ```
    ///
    /// Processing multiple files:
    ///
    /// ```rust
    /// # use oxc_allocator::Allocator;
    /// # use oxc_mangler::Mangler;
    /// # use oxc_parser::Parser;
    /// # use oxc_span::SourceType;
    /// let allocator = Allocator::default();
    /// let mut temp_allocator = Allocator::default();
    /// let files = ["function foo() {}", "function bar() {}"];
    ///
    /// for source in files {
    ///     let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    ///     let mangled_symbols = Mangler::new_with_temp_allocator(&temp_allocator)
    ///         .build(&parsed.program);
    ///     temp_allocator.reset(); // Free memory between files
    /// }
    /// ```
    #[must_use]
    pub fn new_with_temp_allocator(temp_allocator: &'t Allocator) -> Self {
        Self {
            options: MangleOptions::default(),
            stats: None,
            temp_allocator: TempAllocator::Borrowed(temp_allocator),
        }
    }

    #[must_use]
    pub fn with_options(mut self, options: MangleOptions) -> Self {
        self.options = options;
        self
    }

    /// Provide statistics from a prior semantic analysis of the same program
    /// (see [`Semantic::stats`]) to pre-allocate the semantic data [`Mangler::build`]
    /// constructs, avoiding the full-AST counting pass `SemanticBuilder` otherwise
    /// performs.
    #[must_use]
    pub fn with_stats(mut self, stats: Stats) -> Self {
        self.stats = Some(stats);
        self
    }

    /// Mangles the program. The resulting SymbolTable contains the mangled symbols - `program` is not modified.
    /// Pass the symbol table to oxc_codegen to generate the mangled code.
    #[must_use]
    pub fn build(self, program: &Program<'_>) -> ManglerReturn {
        let mut builder = SemanticBuilder::new().with_build_nodes(true).with_class_table(true);
        if let Some(stats) = self.stats {
            builder = builder.with_stats(stats);
        }
        let mut semantic = builder.build(program).semantic;
        let class_private_mappings = self.build_with_semantic(&mut semantic, program);
        ManglerReturn { scoping: semantic.into_scoping(), class_private_mappings }
    }

    /// # Panics
    ///
    /// Panics if the child_ids does not exist in scope_tree.
    pub fn build_with_semantic(
        self,
        semantic: &mut Semantic<'_>,
        program: &Program<'_>,
    ) -> IndexVec<ClassId, FxHashMap<String, CompactStr>> {
        let class_private_mappings = Self::collect_private_members_from_semantic(semantic);
        if self.options.debug {
            self.build_with_semantic_impl(semantic, program, debug_name);
        } else {
            self.build_with_semantic_impl(semantic, program, base54);
        }
        class_private_mappings
    }

    /// Mangle the program: rewrite local bindings to the shortest legal names.
    ///
    /// Runs as a five-stage pipeline; each stage is one method and feeds the next:
    ///
    /// ```text
    ///  collect constraints → assign slots → tally frequencies → generate names → apply names
    ///    names we must NOT     group bindings   rank slots by how    make that many    give each slot a
    ///    reuse or shadow       that can share   often referenced     short, legal      name and rewrite
    ///    (keywords, globals,   one name into    (hottest first)      names             every reference
    ///    exports, eval, …)     numbered "slots"
    /// ```
    ///
    /// # The slot idea (stages 2–5 hinge on this)
    ///
    /// A *slot* is just an integer. Two bindings get the **same** slot exactly when they can
    /// share one name — their live ranges never overlap. So the problem splits cleanly:
    ///   1. *Which bindings may share a name?* → assign slots (graph-colour the scope tree by
    ///      liveness: a slot is reusable in any scope where it isn't live).
    ///   2. *Which name does each slot get?*  → tally + generate + apply (frequency-ranked
    ///      base54 names, clustered by length).
    ///
    /// # Worked example
    ///
    /// ```js
    /// function C(n) {
    ///   for (var i = 0; i < n; i++) log(i);
    ///   for (var j = 0; j < n; j++) log(j);
    /// }
    /// ```
    /// - **assign slots**: `C`,`log` untouched (root binding / global); `n`→slot 1; `i`→slot 2;
    ///   `j`→slot 3 (each its own slot — same-scope bindings never share today).
    /// - **tally**: count references per slot, hottest first.
    /// - **generate names**: `e, t, n, r, …` (base54), skipping any that collide with a keyword,
    ///   a global like `log`, an export, a kept name, or an eval-visible name.
    /// - **apply names**: shortest-*length* names to the hottest slots; within one length, in
    ///   source order. Here the live slots are 1 char, assigned in declaration order:
    /// ```js
    /// function C(e) {
    ///   for (var t = 0; t < e; t++) log(t);
    ///   for (var n = 0; n < e; n++) log(n);
    /// }
    /// ```
    fn build_with_semantic_impl<const CAPACITY: usize, G: Fn(u32) -> InlineString<CAPACITY, u8>>(
        self,
        semantic: &mut Semantic<'_>,
        program: &Program<'_>,
        generate_name: G,
    ) {
        let (scoping, ast_nodes) = semantic.scoping_mut_and_nodes();
        let allocator = self.temp_allocator.as_ref();

        // ── Phase 1: collect constraints — names we must not reuse or shadow. ──
        let constraints =
            Constraints::collect(allocator, scoping, ast_nodes, program, self.options);
        // ── Phase 2: assign slots — give bindings that can share a name the same slot. ──
        let slots = SlotAssignment::compute(allocator, scoping, ast_nodes, &constraints);
        // ── Phase 3: rank slots by reference frequency (hottest first). ──
        let ranking = SlotRanking::tally(allocator, scoping, &constraints, &slots);
        // ── Phase 4: generate that many short, collision-free names. ──
        let names =
            NameTable::generate(allocator, scoping, &constraints, &ranking, &slots, generate_name);
        // ── Phase 5: give each slot its name and rewrite every reference. ──
        names.apply(allocator, scoping, &ranking);
    }

    /// Collects and generates mangled names for private members using semantic information
    /// Returns a Vec where each element corresponds to a class in declaration order
    fn collect_private_members_from_semantic(
        semantic: &Semantic<'_>,
    ) -> IndexVec<ClassId, FxHashMap<String, CompactStr>> {
        let classes = semantic.classes();

        let private_member_count: IndexVec<ClassId, usize> = classes
            .elements
            .iter()
            .map(|class_elements| {
                class_elements.iter().filter(|element| element.is_private).count()
            })
            .collect();
        let parent_private_member_count: IndexVec<ClassId, usize> = classes
            .declarations
            .iter_enumerated()
            .map(|(class_id, _)| {
                classes
                    .ancestors(class_id)
                    .skip(1)
                    .map(|id| private_member_count[id])
                    .sum::<usize>()
            })
            .collect();

        classes
            .elements
            .iter_enumerated()
            .map(|(class_id, class_elements)| {
                let parent_private_member_count = parent_private_member_count[class_id];
                assert!(
                    u32::try_from(class_elements.len() + parent_private_member_count).is_ok(),
                    "too many class elements"
                );
                class_elements
                    .iter()
                    .filter_map(|element| {
                        if element.is_private { Some(element.name.to_string()) } else { None }
                    })
                    .enumerate()
                    .map(|(i, name)| {
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "checked above with assert"
                        )]
                        let mangled = CompactStr::new(
                            // Avoid reusing the same mangled name in parent classes.
                            // We can improve this by reusing names that are not used in child classes,
                            // but nesting a class inside another class is not common
                            // and that would require liveness analysis.
                            base54((parent_private_member_count + i) as u32).as_str(),
                        );
                        (name, mangled)
                    })
                    .collect::<FxHashMap<_, _>>()
            })
            .collect()
    }
}

fn is_special_name(name: &str) -> bool {
    matches!(name, "arguments")
}

#[derive(Debug)]
struct SlotFrequency<'a> {
    pub slot: Slot,
    pub frequency: usize,
    pub symbol_ids: Vec<'a, SymbolId>,
}

impl<'t> SlotFrequency<'t> {
    fn new(temp_allocator: &'t Allocator) -> Self {
        Self { slot: 0, frequency: 0, symbol_ids: Vec::new_in(temp_allocator) }
    }
}

// ─────────────────────────── Pipeline stages ───────────────────────────
//
// Mangling runs as five stages, each a typed value feeding the next:
//   Constraints → SlotAssignment → SlotRanking → NameTable → (applied)
// `Mangler::build_with_semantic_impl` orchestrates them. `'a` is the temporary
// arena lifetime; `'s` is the borrow of `Scoping` (some name sets point into it).

/// Phase 1 output — names and symbols mangling must not rename or shadow.
struct Constraints<'a, 's> {
    /// Whether top-level (module / CommonJS) bindings may be mangled at all.
    top_level: bool,
    /// Names of top-level exports — kept when `top_level` so importers still resolve.
    exported_names: HashSet<'a, Str<'a>>,
    exported_symbols: Option<BitSet<'a>>,
    /// Names preserved by the `keep_names` option (function / class names).
    keep_name_names: FxHashSet<&'s str>,
    keep_name_symbols: Option<BitSet<'a>>,
}

/// Phase 2 output — each symbol's slot, plus the names a direct `eval` can see.
struct SlotAssignment<'a, 's> {
    /// `slots[symbol] == slot`, or `SLOT_UNASSIGNED` for symbols that keep their name.
    slots: Vec<'a, Slot>,
    total_slots: usize,
    /// Names of bindings in direct-`eval` scopes — they keep their names, nothing may shadow them.
    eval_reserved_names: FxHashSet<&'s str>,
}

/// Phase 3 output — slots ranked by reference count, hottest first.
struct SlotRanking<'a> {
    frequencies: Vec<'a, SlotFrequency<'a>>,
}

/// Phase 4 output — the short names to hand out, shortest first.
struct NameTable<'a, const CAPACITY: usize> {
    names: Vec<'a, InlineString<CAPACITY, u8>>,
}

impl<'a, 's> Constraints<'a, 's> {
    /// Phase 1: gather everything the later phases must avoid renaming or shadowing.
    fn collect(
        allocator: &'a Allocator,
        scoping: &'s Scoping,
        ast_nodes: &AstNodes,
        program: &'a Program<'a>,
        options: MangleOptions,
    ) -> Self {
        let top_level = options.top_level(program.source_type);
        let (exported_names, exported_symbols) = if top_level && program.source_type.is_module() {
            collect_exported_symbols(program, allocator, scoping.symbols_len())
        } else {
            (HashSet::new_in(allocator), None)
        };
        let (keep_name_names, keep_name_symbols) =
            collect_keep_name_symbols(options.keep_names, allocator, scoping, ast_nodes);
        Self { top_level, exported_names, exported_symbols, keep_name_names, keep_name_symbols }
    }
}

impl<'a, 's> SlotAssignment<'a, 's> {
    /// Phase 2: assign every manglable binding a *slot*, reusing one slot across scopes whose
    /// live ranges don't overlap (greedy graph-colouring of the scope tree by liveness).
    ///
    /// A slot is reusable in a scope when it isn't live there. We walk the scope tree top-down;
    /// each scope's bindings take the slots that aren't live in it, allocating fresh slots only
    /// when none are free. `slot_liveness[slot]` records the scopes a slot passes through live so
    /// descendant scopes can tell what's free. Invariant on the result: two symbols sharing a slot
    /// never have overlapping live ranges, so giving them one name is always safe.
    fn compute(
        allocator: &'a Allocator,
        scoping: &'s Scoping,
        ast_nodes: &AstNodes,
        constraints: &Constraints,
    ) -> Self {
        let keep_name_symbols = constraints.keep_name_symbols.as_ref();
        // Names of bindings in direct-`eval` scopes — collected here, reserved in Phase 4.
        // TODO: eval reservation is conservative — ideally we'd reserve names per-slot.
        let mut eval_reserved_names: FxHashSet<&'s str> = FxHashSet::default();

        // All symbols with their assigned slots. Keyed by symbol id.
        let mut slots =
            Vec::from_iter_in(iter::repeat_n(SLOT_UNASSIGNED, scoping.symbols_len()), allocator);
        // Stores the lived scope ids for each slot. Keyed by slot number. Symbol count is the
        // upper bound on slots.
        let mut slot_liveness: Vec<BitSet> =
            Vec::with_capacity_in(scoping.symbols_len(), allocator);
        let mut tmp_bindings = Vec::with_capacity_in(100, allocator);
        let mut reusable_slots = Vec::new_in(allocator);
        // Pre-computed BitSet for ancestor membership tests - reused across iterations
        let mut ancestor_set = BitSet::new_in(scoping.scopes_len(), allocator);

        // Walk down the scope tree and assign a slot number for each symbol. Doing it as a scope
        // walk (rather than a flat symbol loop) generates better code.
        for (scope_id, bindings) in scoping.iter_bindings() {
            if bindings.is_empty() {
                continue;
            }
            // Scopes with direct eval: collect binding names as reserved (they can be
            // accessed by eval at runtime) and skip slot assignment (keep original names).
            if scoping.scope_flags(scope_id).contains_direct_eval() {
                for (name, _) in bindings {
                    eval_reserved_names.insert(name.as_str());
                }
                continue;
            }

            // Sort `bindings` in declaration order.
            tmp_bindings.clear();
            tmp_bindings.extend(bindings.values().copied().filter(|binding| {
                !keep_name_symbols.is_some_and(|keep| keep.has_bit(binding.index()))
            }));
            if tmp_bindings.is_empty() {
                continue;
            }
            tmp_bindings.sort_unstable();

            let mut slot = slot_liveness.len();

            reusable_slots.clear();
            reusable_slots.extend(
                // Slots already assigned to other symbols, but not live in the current scope.
                slot_liveness
                    .iter()
                    .enumerate()
                    .filter(|(_, slot_liveness)| !slot_liveness.has_bit(scope_id.index()))
                    .map(
                        // `slot_liveness` is an arena `Vec`, so its indexes cannot exceed `u32::MAX`
                        #[expect(clippy::cast_possible_truncation)]
                        |(slot, _)| slot as Slot,
                    )
                    .take(tmp_bindings.len()),
            );

            // The number of new slots that needs to be allocated.
            let remaining_count = tmp_bindings.len() - reusable_slots.len();
            // There cannot be more slots than there are symbols, and `SymbolId` is a `u32`,
            // so truncation is not possible here
            #[expect(clippy::cast_possible_truncation)]
            reusable_slots.extend((slot as Slot)..(slot + remaining_count) as Slot);

            slot += remaining_count;
            if slot_liveness.len() < slot {
                slot_liveness.extend(
                    iter::repeat_with(|| BitSet::new_in(scoping.scopes_len(), allocator))
                        .take(remaining_count),
                );
            }

            // Pre-compute the set of ancestors from root to scope_id (exclusive) for O(1) tests.
            ancestor_set.clear();
            for ancestor_id in scoping.scope_ancestors(scope_id).skip(1) {
                ancestor_set.set_bit(ancestor_id.index());
            }

            let scope_id_index = scope_id.index();
            for (&symbol_id, &assigned_slot) in tmp_bindings.iter().zip(&reusable_slots) {
                slots[symbol_id.index()] = assigned_slot;

                // `var` is hoisted, so include the scope where it is declared
                // (for cases like `function foo() { { var x; let y; } }`).
                let declared_scope_id =
                    ast_nodes.get_node(scoping.symbol_declaration(symbol_id)).scope_id();
                let redeclared_scope_ids = scoping
                    .symbol_redeclarations(symbol_id)
                    .iter()
                    .map(|r| ast_nodes.get_node(r.declaration).scope_id());
                let referenced_scope_ids =
                    scoping.get_resolved_references(symbol_id).map(Reference::scope_id);

                // The scopes this symbol is alive in: for each use, walk up to (but not including)
                // `scope_id`, marking the descendant scopes it passes through.
                let slot_liveness_bitset = &mut slot_liveness[assigned_slot as usize];
                for used_scope_id in referenced_scope_ids
                    .chain(redeclared_scope_ids)
                    .chain([scope_id, declared_scope_id])
                {
                    for ancestor_id in scoping.scope_ancestors(used_scope_id) {
                        let ancestor_index = ancestor_id.index();
                        // Stop when we reach scope_id or any of its ancestors
                        if ancestor_index == scope_id_index || ancestor_set.has_bit(ancestor_index)
                        {
                            break;
                        }
                        if slot_liveness_bitset.has_bit(ancestor_index) {
                            debug_assert!(
                                scoping.scope_ancestors(ancestor_id).skip(1).all(|a| {
                                    let idx = a.index();
                                    slot_liveness_bitset.has_bit(idx)
                                        || idx == scope_id_index
                                        || ancestor_set.has_bit(idx)
                                }),
                                "Invariant violated: ancestor chain should be fully marked live"
                            );
                            break;
                        }
                        slot_liveness_bitset.set_bit(ancestor_index);
                    }
                }
            }

            // Repair an orphaned named-fn-expr name: a same-named body declaration
            // (`var foo`, parameter `foo`) overwrites the fn-expr's binding-map entry,
            // so the fn-expr symbol never appears in `bindings` and the main pass leaves
            // its slot at `SLOT_UNASSIGNED`. Copy the shadower's slot so both render with
            // the same mangled name — safe because every body reference resolves to the
            // shadower, not the orphan. Only function expressions can host this orphaning;
            // a function declaration's name lives in the parent scope and is unaffected.
            if scoping.scope_flags(scope_id).is_function()
                && let Some(func) = ast_nodes.kind(scoping.get_node_id(scope_id)).as_function()
                && func.is_expression()
                && let Some(id) = &func.id
                && let Some(&shadower) = bindings.get(&id.name)
                && shadower != id.symbol_id()
                && slots[shadower.index()] != SLOT_UNASSIGNED
            {
                slots[id.symbol_id().index()] = slots[shadower.index()];
            }
        }

        let total_slots = slot_liveness.len();
        Self { slots, total_slots, eval_reserved_names }
    }
}

impl<'a> SlotRanking<'a> {
    /// Phase 3: count references per slot and sort hottest-first, skipping slots whose only
    /// symbols are kept, exported (at top level), eval-visible, or special (`arguments`).
    fn tally(
        allocator: &'a Allocator,
        scoping: &Scoping,
        constraints: &Constraints,
        slots: &SlotAssignment,
    ) -> Self {
        let exported_symbols = constraints.exported_symbols.as_ref();
        let keep_name_symbols = constraints.keep_name_symbols.as_ref();
        let root_scope_id = scoping.root_scope_id();
        let mut frequencies = Vec::from_iter_in(
            repeat_with(|| SlotFrequency::new(allocator)).take(slots.total_slots),
            allocator,
        );

        for (symbol_id, &slot) in slots.slots.iter().enumerate() {
            if slot == SLOT_UNASSIGNED {
                continue;
            }
            let symbol_id = SymbolId::from_usize(symbol_id);
            let symbol_scope_id = scoping.symbol_scope_id(symbol_id);
            if symbol_scope_id == root_scope_id
                && (!constraints.top_level
                    || exported_symbols.is_some_and(|e| e.has_bit(symbol_id.index())))
            {
                continue;
            }
            if scoping.scope_flags(symbol_scope_id).contains_direct_eval() {
                continue;
            }
            if is_special_name(scoping.symbol_name(symbol_id)) {
                continue;
            }
            if keep_name_symbols.is_some_and(|keep| keep.has_bit(symbol_id.index())) {
                continue;
            }
            let index = slot as usize;
            frequencies[index].slot = slot;
            frequencies[index].frequency += scoping.get_resolved_reference_ids(symbol_id).len();
            frequencies[index].symbol_ids.push(symbol_id);
        }

        // Remove slots that have no symbols to rename before sorting.
        frequencies.retain(|x| !x.symbol_ids.is_empty());
        frequencies.sort_unstable_by_key(|x| std::cmp::Reverse(x.frequency));
        Self { frequencies }
    }
}

impl<'a, const CAPACITY: usize> NameTable<'a, CAPACITY> {
    /// Phase 4: produce one short name per slot, in order of increasing length.
    ///
    /// Candidates come from `generate_name(0), generate_name(1), …` (base54: `e, t, n, …, ee,
    /// te, …`); a candidate is *reserved* (skipped) if it would clash with a keyword, a global the
    /// program still references by that name, a kept or eval-visible name, or — at the top level —
    /// an export. So the i-th name is the i-th shortest name nothing else needs.
    fn generate(
        allocator: &'a Allocator,
        scoping: &Scoping,
        constraints: &Constraints,
        ranking: &SlotRanking,
        slots: &SlotAssignment,
        generate_name: impl Fn(u32) -> InlineString<CAPACITY, u8>,
    ) -> Self {
        let root_unresolved_references = scoping.root_unresolved_references();
        let root_bindings = scoping.get_bindings(scoping.root_scope_id());
        let is_reserved = |name: &str| {
            oxc_syntax::keyword::is_reserved_keyword(name)
                || is_special_name(name)
                || root_unresolved_references.contains_key(name)
                || (root_bindings.contains_key(name)
                    && (!constraints.top_level || constraints.exported_names.contains(name)))
                // TODO: only skip the names that are kept in the current scope
                || constraints.keep_name_names.contains(name)
                || slots.eval_reserved_names.contains(name)
        };

        let count = ranking.frequencies.len();
        let mut names = Vec::with_capacity_in(count, allocator);
        let mut candidate = 0;
        for _ in 0..count {
            let name = loop {
                let name = generate_name(candidate);
                candidate += 1;
                if !is_reserved(name.as_str()) {
                    break name;
                }
            };
            names.push(name);
        }
        Self { names }
    }

    /// Phase 5: give each slot its name and rewrite every reference to it.
    ///
    /// Names are bucketed by length and, within a bucket, handed out in source order (slot number
    /// == declaration order). Symbols declared near each other thus get near-identical names,
    /// which gzip compresses better — the trick is from Closure Compiler's `RenameVars`:
    /// <https://github.com/google/closure-compiler/blob/c383a3a1d2fce33b6c778ef76b5a626e07abca41/src/com/google/javascript/jscomp/RenameVars.java#L475-L483>
    fn apply(self, allocator: &'a Allocator, scoping: &mut Scoping, ranking: &SlotRanking) {
        // Yields slots hottest-first as we consume each length bucket.
        let mut freq_iter = ranking.frequencies.iter();
        // Scratch buffers in the temp arena (reused/reset across files via `new_with_temp_allocator`).
        let mut symbols_renamed_in_this_batch = Vec::with_capacity_in(100, allocator);
        let mut slice_of_same_len_strings = Vec::with_capacity_in(100, allocator);
        // Names are generated shortest-first, so each `chunk_by(len)` group is one name length.
        for (_, group) in &self.names.into_iter().chunk_by(InlineString::len) {
            // Take the N hottest remaining slots to receive the N names of this length...
            slice_of_same_len_strings.clear();
            slice_of_same_len_strings.extend(group);
            symbols_renamed_in_this_batch.clear();
            symbols_renamed_in_this_batch
                .extend(freq_iter.by_ref().take(slice_of_same_len_strings.len()));

            debug_assert_eq!(symbols_renamed_in_this_batch.len(), slice_of_same_len_strings.len());

            // ...but hand the names out in source order, so neighbours get similar names.
            symbols_renamed_in_this_batch.sort_unstable_by_key(|a: &&SlotFrequency| a.slot);

            for (symbol_to_rename, new_name) in
                symbols_renamed_in_this_batch.iter().zip(slice_of_same_len_strings.iter())
            {
                // A slot can be shared by several symbols (cross-scope reuse); rename them all.
                for &symbol_id in &symbol_to_rename.symbol_ids {
                    scoping.set_symbol_name(symbol_id, Ident::from(new_name.as_str()));
                }
            }
        }
    }
}

fn collect_exported_symbols<'a>(
    program: &Program<'a>,
    allocator: &'a Allocator,
    symbols_len: usize,
) -> (HashSet<'a, Str<'a>>, Option<BitSet<'a>>) {
    let mut exported_symbols = BitSet::new_in(symbols_len, allocator);
    let mut exported_names = HashSet::new_in(allocator);
    for statement in &program.body {
        let Statement::ExportNamedDeclaration(v) = statement else { continue };
        let Some(decl) = &v.declaration else { continue };
        if let Declaration::VariableDeclaration(decl) = decl {
            for decl in &decl.declarations {
                if let Some(id) = decl.id.get_binding_identifier() {
                    exported_names.insert(id.name.as_arena_str());
                    exported_symbols.set_bit(id.symbol_id().index());
                }
            }
        } else if let Some(id) = decl.id() {
            exported_names.insert(id.name.as_arena_str());
            exported_symbols.set_bit(id.symbol_id().index());
        }
    }
    (exported_names, Some(exported_symbols))
}

fn collect_keep_name_symbols<'alloc, 's>(
    keep_names: MangleOptionsKeepNames,
    allocator: &'alloc Allocator,
    scoping: &'s Scoping,
    nodes: &AstNodes,
) -> (FxHashSet<&'s str>, Option<BitSet<'alloc>>) {
    if !keep_names.function && !keep_names.class {
        return (FxHashSet::default(), None);
    }
    let ids = collect_name_symbols(keep_names, allocator, scoping, nodes);
    (ids.ones().map(|id| scoping.symbol_name(SymbolId::from_usize(id))).collect(), Some(ids))
}

// Maximum length of string is 15 (`slot_4294967295` for `u32::MAX`).
fn debug_name(n: u32) -> InlineString<15, u8> {
    // Using `format!` here allocates a string unnecessarily.
    // But this function is not for use in production, so let's not worry about it.
    // We shouldn't resort to unsafe code, when it's not critical for performance.
    InlineString::from_str(&format!("slot_{n}"))
}
