use std::{iter, ops::Deref};

use fixedbitset::FixedBitSet;
use itertools::Itertools;
use rustc_hash::FxHashSet;

use oxc_allocator::{Allocator, Vec};
use oxc_ast::ast::{Declaration, Program, Statement};
use oxc_index::Idx;
use oxc_semantic::{ScopeTree, Semantic, SemanticBuilder, SymbolId, SymbolTable};
use oxc_span::Atom;

#[derive(Default, Debug, Clone, Copy)]
pub struct MangleOptions {
    /// Also mangle exported variables.
    pub top_level: bool,
    /// Use more readable mangled names
    /// (e.g. `slot_0`, `slot_1`, `slot_2`, ...) for debugging.
    ///
    /// Uses base54 if false.
    pub debug: bool,
}

type Slot = usize;

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
#[derive(Default)]
pub struct Mangler {
    options: MangleOptions,
}

impl Mangler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_options(mut self, options: MangleOptions) -> Self {
        self.options = options;
        self
    }

    /// Mangles the program. The resulting SymbolTable contains the mangled symbols - `program` is not modified.
    /// Pass the symbol table to oxc_codegen to generate the mangled code.
    #[must_use]
    pub fn build(self, program: &Program<'_>) -> SymbolTable {
        let semantic =
            SemanticBuilder::new().with_scope_tree_child_ids(true).build(program).semantic;
        self.build_with_semantic(semantic, program)
    }

    /// # Panics
    ///
    /// Panics if the child_ids does not exist in scope_tree.
    #[must_use]
    pub fn build_with_semantic(self, semantic: Semantic<'_>, program: &Program<'_>) -> SymbolTable {
        if self.options.debug {
            self.build_with_symbols_and_scopes_impl(semantic, program, debug_name)
        } else {
            self.build_with_symbols_and_scopes_impl(semantic, program, base54)
        }
    }

    fn build_with_symbols_and_scopes_impl<
        const CAPACITY: usize,
        G: Fn(usize) -> InlineString<CAPACITY>,
    >(
        self,
        semantic: Semantic<'_>,
        program: &Program<'_>,
        generate_name: G,
    ) -> SymbolTable {
        let (mut symbol_table, scope_tree, ast_nodes) = semantic.into_symbols_scopes_nodes();

        assert!(scope_tree.has_child_ids(), "child_id needs to be generated");

        let (exported_names, exported_symbols) = if self.options.top_level {
            Mangler::collect_exported_symbols(program)
        } else {
            Default::default()
        };

        let allocator = Allocator::default();

        // All symbols with their assigned slots. Keyed by symbol id.
        let mut slots: Vec<'_, Slot> =
            Vec::from_iter_in(iter::repeat_n(0, symbol_table.len()), &allocator);

        // Stores the lived scope ids for each slot. Keyed by slot number.
        let mut slot_liveness: std::vec::Vec<FixedBitSet> = vec![];
        let mut tmp_bindings = std::vec::Vec::with_capacity(100);

        let mut reusable_slots = std::vec::Vec::new();
        // Walk down the scope tree and assign a slot number for each symbol.
        // It is possible to do this in a loop over the symbol list,
        // but walking down the scope tree seems to generate a better code.
        for (scope_id, bindings) in scope_tree.iter_bindings() {
            if bindings.is_empty() {
                continue;
            }

            let mut slot = slot_liveness.len();

            reusable_slots.clear();
            reusable_slots.extend(
                // Slots that are already assigned to other symbols, but does not live in the current scope.
                slot_liveness
                    .iter()
                    .enumerate()
                    .filter(|(_, slot_liveness)| !slot_liveness.contains(scope_id.index()))
                    .map(|(slot, _)| slot)
                    .take(bindings.len()),
            );

            // The number of new slots that needs to be allocated.
            let remaining_count = bindings.len() - reusable_slots.len();
            reusable_slots.extend(slot..slot + remaining_count);

            slot += remaining_count;
            if slot_liveness.len() < slot {
                slot_liveness.resize_with(slot, || FixedBitSet::with_capacity(scope_tree.len()));
            }

            // Sort `bindings` in declaration order.
            tmp_bindings.clear();
            tmp_bindings.extend(bindings.values().copied());
            tmp_bindings.sort_unstable();
            for (&symbol_id, assigned_slot) in
                tmp_bindings.iter().zip(reusable_slots.iter().copied())
            {
                slots[symbol_id.index()] = assigned_slot;

                // If the symbol is declared by `var`, then it can be hoisted to
                // parent, so we need to include the scope where it is declared.
                // (for cases like `function foo() { { var x; let y; } }`)
                let declared_scope_id =
                    ast_nodes.get_node(symbol_table.get_declaration(symbol_id)).scope_id();

                // Calculate the scope ids that this symbol is alive in.
                let lived_scope_ids = symbol_table
                    .get_resolved_references(symbol_id)
                    .map(|reference| ast_nodes.get_node(reference.node_id()).scope_id())
                    .chain([scope_id, declared_scope_id])
                    .flat_map(|used_scope_id| {
                        scope_tree.ancestors(used_scope_id).take_while(|s_id| *s_id != scope_id)
                    });

                // Since the slot is now assigned to this symbol, it is alive in all the scopes that this symbol is alive in.
                slot_liveness[assigned_slot].extend(lived_scope_ids.map(oxc_index::Idx::index));
            }
        }

        let total_number_of_slots = slot_liveness.len();

        let frequencies = self.tally_slot_frequencies(
            &symbol_table,
            &exported_symbols,
            &scope_tree,
            total_number_of_slots,
            &slots,
            &allocator,
        );

        let root_unresolved_references = scope_tree.root_unresolved_references();
        let root_bindings = scope_tree.get_bindings(scope_tree.root_scope_id());

        let mut reserved_names = Vec::with_capacity_in(total_number_of_slots, &allocator);

        let mut count = 0;
        for _ in 0..total_number_of_slots {
            let name = loop {
                let name = generate_name(count);
                count += 1;
                // Do not mangle keywords and unresolved references
                let n = name.as_str();
                if !is_keyword(n)
                    && !is_special_name(n)
                    && !root_unresolved_references.contains_key(n)
                    && !(root_bindings.contains_key(n)
                        && (!self.options.top_level || exported_names.contains(n)))
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
        let mut symbols_renamed_in_this_batch = std::vec::Vec::with_capacity(100);
        let mut slice_of_same_len_strings = std::vec::Vec::with_capacity(100);
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
                    symbol_table.set_name(symbol_id, new_name);
                }
            }
        }

        symbol_table
    }

    fn tally_slot_frequencies<'a>(
        &'a self,
        symbol_table: &SymbolTable,
        exported_symbols: &FxHashSet<SymbolId>,
        scope_tree: &ScopeTree,
        total_number_of_slots: usize,
        slots: &[Slot],
        allocator: &'a Allocator,
    ) -> Vec<'a, SlotFrequency<'a>> {
        let root_scope_id = scope_tree.root_scope_id();
        let mut frequencies = Vec::with_capacity_in(total_number_of_slots, allocator);
        for _ in 0..total_number_of_slots {
            frequencies.push(SlotFrequency::new(allocator));
        }

        for (symbol_id, slot) in slots.iter().copied().enumerate() {
            let symbol_id = SymbolId::from_usize(symbol_id);
            if symbol_table.get_scope_id(symbol_id) == root_scope_id
                && (!self.options.top_level || exported_symbols.contains(&symbol_id))
            {
                continue;
            }
            if is_special_name(symbol_table.get_name(symbol_id)) {
                continue;
            }
            let index = slot;
            frequencies[index].slot = slot;
            frequencies[index].frequency +=
                symbol_table.get_resolved_reference_ids(symbol_id).len();
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
}

fn is_special_name(name: &str) -> bool {
    matches!(name, "exports" | "arguments")
}

#[derive(Debug)]
struct SlotFrequency<'a> {
    pub slot: Slot,
    pub frequency: usize,
    pub symbol_ids: Vec<'a, SymbolId>,
}

impl<'a> SlotFrequency<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self { slot: 0, frequency: 0, symbol_ids: Vec::new_in(allocator) }
    }
}

#[rustfmt::skip]
fn is_keyword(s: &str) -> bool {
    matches!(s, "as" | "do" | "if" | "in" | "is" | "of" | "any" | "for" | "get"
            | "let" | "new" | "out" | "set" | "try" | "var" | "case" | "else"
            | "enum" | "from" | "meta" | "null" | "this" | "true" | "type"
            | "void" | "with")
}

#[repr(C, align(64))]
struct Aligned64([u8; 64]);

/// The characters are in frequency order, so that the characters with higher frequency are used first.
///
/// This idea was inspired by nanoid. <https://github.com/ai/nanoid/blob/5.0.9/url-alphabet/index.js>
///
/// This list was generated by the following steps:
/// 1. Generate a source code with replacing all manglable variable names with `$` (assuming `$` is the least used character).
///    You can do this by passing the following `blank` function to the `generate_name` parameter of [Mangler::build_with_symbols_and_scopes_impl].
///    ```no_run
///    fn blank(_: usize) -> InlineString<12> {
///        let mut str = InlineString::new();
///        unsafe { str.push_unchecked(b"$"[0]); }
///        str
///    }
///    ```
/// 2. Run the following command in `target/minifier/default` to check generate the list:
///    ```shell
///    find . -type f -exec cat {} + | `# concat all files in that directory` \
///      tr -d '\n' | fold -w1 | `# separate each characters in to each line` \
///      grep -E '[abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789]' | `# filter the character` \
///      sort | uniq -c | `# count each characters` \
///      sort -nr | awk '{print $2}' | tr -d '\n' `# format output`
///    ```
///    The result I got is `etnriaoscludfpmhg_10vy2436b8x579SCwTEDOkAjMNPFILRzBVHUWGKqJYXZQ`.
/// 3. Add `$` at the end and then move all numbers to the end of the list.
const BASE54_CHARS: Aligned64 =
    Aligned64(*b"etnriaoscludfpmhg_vybxSCwTEDOkAjMNPFILRzBVHUWGKqJYXZQ$1024368579");

/// Get the shortest mangled name for a given n.
/// Code adapted from [terser](https://github.com/terser/terser/blob/8b966d687395ab493d2c6286cc9dd38650324c11/lib/scope.js#L1041-L1051)
//
// Maximum length of string is 11 (`ZrN6rN6rN6r` for `u64::MAX`), but set `CAPACITY` as 12,
// so the total size of `InlineString` is 16, including the `len` field.
// Then initializing the `InlineString` is a single `xmm` set, and with luck it'll sit in a register
// throughout this function.
#[expect(clippy::items_after_statements)]
fn base54(n: usize) -> InlineString<12> {
    let mut str = InlineString::new();

    let mut num = n;

    // Base 54 at first because these are the usable first characters in JavaScript identifiers
    // <https://tc39.es/ecma262/#prod-IdentifierStart>
    const FIRST_BASE: usize = 54;
    let byte = BASE54_CHARS.0[num % FIRST_BASE];
    // SAFETY: All `BASE54_CHARS` are ASCII. This is first byte we push, so can't be out of bounds.
    unsafe { str.push_unchecked(byte) };
    num /= FIRST_BASE;

    // Base 64 for the rest because after the first character we can also use 0-9 too
    // <https://tc39.es/ecma262/#prod-IdentifierPart>
    const REST_BASE: usize = 64;
    while num > 0 {
        num -= 1;
        let byte = BASE54_CHARS.0[num % REST_BASE];
        // SAFETY: All `BASE54_CHARS` are ASCII.
        // String for `u64::MAX` is `ZrN6rN6rN6r` (11 bytes), so cannot push more than `CAPACITY` (12).
        unsafe { str.push_unchecked(byte) };
        num /= REST_BASE;
    }

    str
}

// Maximum length of string is 25 (`slot_18446744073709551615` for `u64::MAX`)
// but set `CAPACITY` as 28 so the total size of `InlineString` is 32, including the `len` field.
fn debug_name(n: usize) -> InlineString<28> {
    InlineString::from_str(&format!("slot_{n}"))
}

/// Short inline string.
///
/// `CAPACITY` determines the maximum length of the string.
#[repr(align(16))]
struct InlineString<const CAPACITY: usize> {
    len: u32,
    bytes: [u8; CAPACITY],
}

impl<const CAPACITY: usize> InlineString<CAPACITY> {
    /// Create empty [`InlineString`].
    #[inline]
    fn new() -> Self {
        const { assert!(CAPACITY <= u32::MAX as usize) };

        Self { bytes: [0; CAPACITY], len: 0 }
    }

    /// Create [`InlineString`] from `&str`.
    ///
    /// # Panics
    /// Panics if `s.len() > CAPACITY`.
    fn from_str(s: &str) -> Self {
        let mut bytes = [0; CAPACITY];
        let slice = &mut bytes[..s.len()];
        slice.copy_from_slice(s.as_bytes());
        Self { bytes, len: u32::try_from(s.len()).unwrap() }
    }

    /// Push a byte to the string.
    ///
    /// # SAFETY
    /// * Must not push more than `CAPACITY` bytes.
    /// * `byte` must be < 128 (an ASCII character).
    #[inline]
    unsafe fn push_unchecked(&mut self, byte: u8) {
        debug_assert!((self.len as usize) < CAPACITY);
        debug_assert!(byte.is_ascii());

        *self.bytes.get_unchecked_mut(self.len as usize) = byte;
        self.len += 1;
    }

    /// Get length of string as `u32`.
    #[inline]
    fn len(&self) -> u32 {
        self.len
    }

    /// Get string as `&str` slice.
    #[inline]
    fn as_str(&self) -> &str {
        // SAFETY: If safety conditions of `push_unchecked` have been upheld,
        // slice cannot be out of bounds, and contents of that slice is a valid UTF-8 string
        unsafe {
            let slice = self.bytes.get_unchecked(..self.len as usize);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl<const CAPACITY: usize> Deref for InlineString<CAPACITY> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod test {
    use super::base54;

    #[test]
    fn test_base54() {
        assert_eq!(&*base54(0), "a");
        assert_eq!(&*base54(25), "z");
        assert_eq!(&*base54(26), "A");
        assert_eq!(&*base54(51), "Z");
        assert_eq!(&*base54(52), "$");
        assert_eq!(&*base54(53), "_");
        assert_eq!(&*base54(54), "aa");
        assert_eq!(&*base54(55), "ab");

        if cfg!(target_pointer_width = "64") {
            assert_eq!(&*base54(usize::MAX), "ZrN6rN6rN6r");
        }

        if cfg!(target_pointer_width = "32") {
            assert_eq!(&*base54(usize::MAX), "vUdzUd");
        }
    }
}
