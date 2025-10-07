use std::iter::{self, repeat_with};

use itertools::Itertools;
use keep_names::collect_name_symbols;
use oxc_index::IndexVec;
use oxc_syntax::class::ClassId;
use rustc_hash::{FxHashMap, FxHashSet};

use base54::base54;
use oxc_allocator::{Allocator, BitSet, Vec};
use oxc_ast::ast::{Declaration, Program, Statement};
use oxc_data_structures::inline_string::InlineString;
use oxc_semantic::{AstNodes, Scoping, Semantic, SemanticBuilder, SymbolId};
use oxc_span::{Atom, CompactStr};

pub(crate) mod base54;
mod keep_names;

pub use keep_names::MangleOptionsKeepNames;

#[derive(Default, Debug, Clone, Copy)]
pub struct MangleOptions {
    /// Pass true to mangle names declared in the top level scope.
    ///
    /// Default: `false`
    pub top_level: bool,

    /// Keep function / class names
    pub keep_names: MangleOptionsKeepNames,

    /// Use more readable mangled names
    /// (e.g. `slot_0`, `slot_1`, `slot_2`, ...) for debugging.
    ///
    /// Uses base54 if false.
    pub debug: bool,
}

type Slot = u32;

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
/// ```rust
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
/// assert!(parsed.errors.is_empty());
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
    /// An allocator meant to be used for temporary allocations during mangling.
    /// It can be cleared after mangling is done, to free up memory for subsequent
    /// files or other operations.
    temp_allocator: TempAllocator<'t>,
}

impl Default for Mangler<'_> {
    fn default() -> Self {
        Self {
            options: MangleOptions::default(),
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
            temp_allocator: TempAllocator::Borrowed(temp_allocator),
        }
    }

    #[must_use]
    pub fn with_options(mut self, options: MangleOptions) -> Self {
        self.options = options;
        self
    }

    /// Mangles the program. The resulting SymbolTable contains the mangled symbols - `program` is not modified.
    /// Pass the symbol table to oxc_codegen to generate the mangled code.
    #[must_use]
    pub fn build(self, program: &Program<'_>) -> ManglerReturn {
        let mut semantic =
            SemanticBuilder::new().with_scope_tree_child_ids(true).build(program).semantic;
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

    fn build_with_semantic_impl<const CAPACITY: usize, G: Fn(u32) -> InlineString<CAPACITY, u8>>(
        self,
        semantic: &mut Semantic<'_>,
        program: &Program<'_>,
        generate_name: G,
    ) {
        let (scoping, ast_nodes) = semantic.scoping_mut_and_nodes();

        assert!(scoping.has_scope_child_ids(), "child_id needs to be generated");

        // TODO: implement opt-out of direct-eval in a branch of scopes.
        if scoping.root_scope_flags().contains_direct_eval() {
            return;
        }

        let (exported_names, exported_symbols) = if self.options.top_level {
            Mangler::collect_exported_symbols(program)
        } else {
            Default::default()
        };
        let (keep_name_names, keep_name_symbols) =
            Mangler::collect_keep_name_symbols(self.options.keep_names, scoping, ast_nodes);

        let temp_allocator = self.temp_allocator.as_ref();

        // All symbols with their assigned slots. Keyed by symbol id.
        let mut slots = Vec::from_iter_in(iter::repeat_n(0, scoping.symbols_len()), temp_allocator);

        // Stores the lived scope ids for each slot. Keyed by slot number.
        let mut slot_liveness: Vec<BitSet> = Vec::new_in(temp_allocator);
        let mut tmp_bindings = Vec::with_capacity_in(100, temp_allocator);

        let mut reusable_slots = Vec::new_in(temp_allocator);
        // Walk down the scope tree and assign a slot number for each symbol.
        // It is possible to do this in a loop over the symbol list,
        // but walking down the scope tree seems to generate a better code.
        for (scope_id, bindings) in scoping.iter_bindings() {
            if bindings.is_empty() {
                continue;
            }

            // Sort `bindings` in declaration order.
            tmp_bindings.clear();
            tmp_bindings.extend(
                bindings.values().copied().filter(|binding| !keep_name_symbols.contains(binding)),
            );
            if tmp_bindings.is_empty() {
                continue;
            }
            tmp_bindings.sort_unstable();

            let mut slot = slot_liveness.len();

            reusable_slots.clear();
            reusable_slots.extend(
                // Slots that are already assigned to other symbols, but does not live in the current scope.
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
                    iter::repeat_with(|| BitSet::new_in(scoping.scopes_len(), temp_allocator))
                        .take(remaining_count),
                );
            }

            for (&symbol_id, &assigned_slot) in tmp_bindings.iter().zip(&reusable_slots) {
                slots[symbol_id.index()] = assigned_slot;

                // If the symbol is declared by `var`, then it can be hoisted to
                // parent, so we need to include the scope where it is declared.
                // (for cases like `function foo() { { var x; let y; } }`)
                let declared_scope_id =
                    ast_nodes.get_node(scoping.symbol_declaration(symbol_id)).scope_id();

                let redeclared_scope_ids = scoping
                    .symbol_redeclarations(symbol_id)
                    .iter()
                    .map(|r| ast_nodes.get_node(r.declaration).scope_id());

                let referenced_scope_ids = scoping
                    .get_resolved_references(symbol_id)
                    .map(|reference| ast_nodes.get_node(reference.node_id()).scope_id());

                // Calculate the scope ids that this symbol is alive in.
                let lived_scope_ids = referenced_scope_ids
                    .chain(redeclared_scope_ids)
                    .chain([scope_id, declared_scope_id])
                    .flat_map(|used_scope_id| {
                        scoping.scope_ancestors(used_scope_id).take_while(|s_id| *s_id != scope_id)
                    });

                // Since the slot is now assigned to this symbol, it is alive in all the scopes that this symbol is alive in.
                for scope_id in lived_scope_ids {
                    slot_liveness[assigned_slot as usize].set_bit(scope_id.index());
                }
            }
        }

        let total_number_of_slots = slot_liveness.len();

        let frequencies = self.tally_slot_frequencies(
            scoping,
            &exported_symbols,
            &keep_name_symbols,
            total_number_of_slots,
            &slots,
        );

        let root_unresolved_references = scoping.root_unresolved_references();
        let root_bindings = scoping.get_bindings(scoping.root_scope_id());

        let mut reserved_names = Vec::with_capacity_in(total_number_of_slots, temp_allocator);

        let mut count = 0;
        for _ in 0..total_number_of_slots {
            let name = loop {
                let name = generate_name(count);
                count += 1;
                // Do not mangle keywords and unresolved references
                let n = name.as_str();
                if !oxc_syntax::keyword::is_reserved_keyword(n)
                    && !is_special_name(n)
                    && !root_unresolved_references.contains_key(n)
                    && !(root_bindings.contains_key(n)
                        && (!self.options.top_level || exported_names.contains(n)))
                        // TODO: only skip the names that are kept in the current scope
                        && !keep_name_names.contains(n)
                {
                    break name;
                }
            };
            reserved_names.push(name);
        }

        // Group similar symbols for smaller gzipped file
        // <https://github.com/google/closure-compiler/blob/c383a3a1d2fce33b6c778ef76b5a626e07abca41/src/com/google/javascript/jscomp/RenameVars.java#L475-L483>
        // Original Comment:
        // 1) The most frequent vars get the shorter names.
        // 2) If N number of vars are going to be assigned names of the same
        //    length, we assign the N names based on the order at which the vars
        //    first appear in the source. This makes the output somewhat less
        //    random, because symbols declared close together are assigned names
        //    that are quite similar. With this heuristic, the output is more
        //    compressible.
        //    For instance, the output may look like:
        //    var da = "..", ea = "..";
        //    function fa() { .. } function ga() { .. }

        let mut freq_iter = frequencies.iter();
        let mut symbols_renamed_in_this_batch = Vec::with_capacity_in(100, temp_allocator);
        let mut slice_of_same_len_strings = Vec::with_capacity_in(100, temp_allocator);
        // 2. "N number of vars are going to be assigned names of the same length"
        for (_, slice_of_same_len_strings_group) in
            &reserved_names.into_iter().chunk_by(InlineString::len)
        {
            // 1. "The most frequent vars get the shorter names"
            // (freq_iter is sorted by frequency from highest to lowest,
            //  so taking means take the N most frequent symbols remaining)
            slice_of_same_len_strings.clear();
            slice_of_same_len_strings.extend(slice_of_same_len_strings_group);
            symbols_renamed_in_this_batch.clear();
            symbols_renamed_in_this_batch
                .extend(freq_iter.by_ref().take(slice_of_same_len_strings.len()));

            debug_assert_eq!(symbols_renamed_in_this_batch.len(), slice_of_same_len_strings.len());

            // 2. "we assign the N names based on the order at which the vars first appear in the source."
            // sorting by slot enables us to sort by the order at which the vars first appear in the source
            // (this is possible because the slots are discovered currently in a DFS method which is the same order
            //  as variables appear in the source code)
            symbols_renamed_in_this_batch.sort_unstable_by_key(|a| a.slot);

            // here we just zip the iterator of symbols to rename with the iterator of new names for the next for loop
            let symbols_to_rename_with_new_names =
                symbols_renamed_in_this_batch.iter().zip(slice_of_same_len_strings.iter());

            // rename the variables
            for (symbol_to_rename, new_name) in symbols_to_rename_with_new_names {
                for &symbol_id in &symbol_to_rename.symbol_ids {
                    scoping.set_symbol_name(symbol_id, new_name);
                }
            }
        }
    }

    fn tally_slot_frequencies<'a>(
        &'a self,
        scoping: &Scoping,
        exported_symbols: &FxHashSet<SymbolId>,
        keep_name_symbols: &FxHashSet<SymbolId>,
        total_number_of_slots: usize,
        slots: &[Slot],
    ) -> Vec<'a, SlotFrequency<'a>> {
        let root_scope_id = scoping.root_scope_id();
        let temp_allocator = self.temp_allocator.as_ref();
        let mut frequencies = Vec::from_iter_in(
            repeat_with(|| SlotFrequency::new(temp_allocator)).take(total_number_of_slots),
            temp_allocator,
        );

        for (symbol_id, slot) in slots.iter().copied().enumerate() {
            let symbol_id = SymbolId::from_usize(symbol_id);
            if scoping.symbol_scope_id(symbol_id) == root_scope_id
                && (!self.options.top_level || exported_symbols.contains(&symbol_id))
            {
                continue;
            }
            if is_special_name(scoping.symbol_name(symbol_id)) {
                continue;
            }
            if keep_name_symbols.contains(&symbol_id) {
                continue;
            }
            let index = slot as usize;
            frequencies[index].slot = slot;
            frequencies[index].frequency += scoping.get_resolved_reference_ids(symbol_id).len();
            frequencies[index].symbol_ids.push(symbol_id);
        }
        frequencies.sort_unstable_by_key(|x| std::cmp::Reverse(x.frequency));
        frequencies
    }

    fn collect_exported_symbols<'a>(
        program: &Program<'a>,
    ) -> (FxHashSet<Atom<'a>>, FxHashSet<SymbolId>) {
        program
            .body
            .iter()
            .filter_map(|statement| {
                let Statement::ExportNamedDeclaration(v) = statement else { return None };
                v.declaration.as_ref()
            })
            .flat_map(|decl| {
                if let Declaration::VariableDeclaration(decl) = decl {
                    itertools::Either::Left(
                        decl.declarations
                            .iter()
                            .filter_map(|decl| decl.id.get_binding_identifier()),
                    )
                } else {
                    itertools::Either::Right(decl.id().into_iter())
                }
            })
            .map(|id| (id.name, id.symbol_id()))
            .collect()
    }

    fn collect_keep_name_symbols<'a>(
        keep_names: MangleOptionsKeepNames,
        scoping: &'a Scoping,
        nodes: &AstNodes,
    ) -> (FxHashSet<&'a str>, FxHashSet<SymbolId>) {
        let ids = collect_name_symbols(keep_names, scoping, nodes);
        (ids.iter().map(|id| scoping.symbol_name(*id)).collect(), ids)
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
                class_elements
                    .iter()
                    .filter_map(|element| {
                        if element.is_private { Some(element.name.to_string()) } else { None }
                    })
                    .count()
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

// Maximum length of string is 15 (`slot_4294967295` for `u32::MAX`).
fn debug_name(n: u32) -> InlineString<15, u8> {
    // Using `format!` here allocates a string unnecessarily.
    // But this function is not for use in production, so let's not worry about it.
    // We shouldn't resort to unsafe code, when it's not critical for performance.
    InlineString::from_str(&format!("slot_{n}"))
}
