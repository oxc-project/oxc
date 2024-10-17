use itertools::Itertools;
use oxc_ast::ast::Program;
use oxc_index::{index_vec, Idx, IndexVec};
use oxc_semantic::{ReferenceId, ScopeTree, SemanticBuilder, SymbolId, SymbolTable};
use oxc_span::CompactStr;

type Slot = usize;

#[derive(Default)]
pub struct MangleOptions {
    pub debug: bool,
}

/// # Name Mangler / Symbol Minification
///
/// See:
///   * [esbuild](https://github.com/evanw/esbuild/blob/main/docs/architecture.md#symbol-minification)
///
/// This algorithm is targeted for better gzip compression.
///
/// Visually, a slot is the index position for binding identifiers:
///
/// ```javascript
/// function slot0(slot2, slot3, slot4) {
///     slot2 = 1;
/// }
/// function slot1(slot2, slot3) {
///     function slot4() {
///         slot2 = 1;
///     }
/// }
/// ```
///
/// The slot number for a new scope starts after the maximum slot of the parent scope.
///
/// Occurrences of slots and their corresponding newly assigned short identifiers are:
/// - slot2: 4 - a
/// - slot3: 2 - b
/// - slot4: 2 - c
/// - slot0: 1 - d
/// - slot1: 1 - e
///
/// After swapping out the mangled names:
///
/// ```javascript
/// function d(a, b, c) {
///     a = 1;
/// }
/// function e(a, b) {
///     function c() {
///         a = 1;
///     }
/// }
/// ```
#[derive(Default)]
pub struct Mangler {
    symbol_table: SymbolTable,

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

    pub fn get_symbol_name(&self, symbol_id: SymbolId) -> &str {
        self.symbol_table.get_name(symbol_id)
    }

    pub fn get_reference_name(&self, reference_id: ReferenceId) -> Option<&str> {
        let symbol_id = self.symbol_table.get_reference(reference_id).symbol_id()?;
        Some(self.symbol_table.get_name(symbol_id))
    }

    #[must_use]
    pub fn build<'a>(mut self, program: &'a Program<'a>) -> Mangler {
        let semantic = SemanticBuilder::new().build(program).semantic;

        // Mangle the symbol table by computing slots from the scope tree.
        // A slot is the occurrence index of a binding identifier inside a scope.
        let (mut symbol_table, scope_tree) = semantic.into_symbol_table_and_scope_tree();

        // Total number of slots for all scopes
        let mut total_number_of_slots: Slot = 0;

        // All symbols with their assigned slots
        let mut slots: IndexVec<SymbolId, Slot> = index_vec![0; symbol_table.len()];

        // Keep track of the maximum slot number for each scope
        let mut max_slot_for_scope = vec![0; scope_tree.len()];

        // Walk the scope tree and compute the slot number for each scope
        for scope_id in scope_tree.descendants_from_root() {
            let bindings = scope_tree.get_bindings(scope_id);

            // The current slot number is continued by the maximum slot from the parent scope
            let parent_max_slot = scope_tree
                .get_parent_id(scope_id)
                .map_or(0, |parent_scope_id| max_slot_for_scope[parent_scope_id.index()]);

            let mut slot = parent_max_slot;

            if !bindings.is_empty() {
                // `bindings` are stored in order, traverse and increment slot
                for symbol_id in bindings.values().copied() {
                    slots[symbol_id] = slot;
                    slot += 1;
                }
            }

            max_slot_for_scope[scope_id.index()] = slot;

            if slot > total_number_of_slots {
                total_number_of_slots = slot;
            }
        }

        let frequencies =
            Self::tally_slot_frequencies(&symbol_table, &scope_tree, total_number_of_slots, &slots);

        let root_unresolved_references = scope_tree.root_unresolved_references();
        let root_bindings = scope_tree.get_bindings(scope_tree.root_scope_id());

        let mut reserved_names = Vec::with_capacity(total_number_of_slots);

        let generate_name = if self.options.debug { debug_name } else { base54 };
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
                    && !root_bindings.contains_key(n)
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
        // 2. "N number of vars are going to be assigned names of the same length"
        for (_, slice_of_same_len_strings_group) in
            &reserved_names.into_iter().chunk_by(CompactStr::len)
        {
            // 1. "The most frequent vars get the shorter names"
            // (freq_iter is sorted by frequency from highest to lowest,
            //  so taking means take the N most frequent symbols remaining)
            let slice_of_same_len_strings = slice_of_same_len_strings_group.collect_vec();
            let mut symbols_renamed_in_this_batch =
                freq_iter.by_ref().take(slice_of_same_len_strings.len()).collect::<Vec<_>>();

            debug_assert!(symbols_renamed_in_this_batch.len() == slice_of_same_len_strings.len());

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
                for symbol_id in &symbol_to_rename.symbol_ids {
                    symbol_table.set_name(*symbol_id, new_name.clone());
                }
            }
        }

        self.symbol_table = symbol_table;
        self
    }

    fn tally_slot_frequencies(
        symbol_table: &SymbolTable,
        scope_tree: &ScopeTree,
        total_number_of_slots: usize,
        slots: &IndexVec<SymbolId, Slot>,
    ) -> Vec<SlotFrequency> {
        let root_scope_id = scope_tree.root_scope_id();
        let mut frequencies = vec![SlotFrequency::default(); total_number_of_slots];
        for (symbol_id, slot) in slots.iter_enumerated() {
            if symbol_table.get_scope_id(symbol_id) == root_scope_id {
                continue;
            }
            if is_special_name(symbol_table.get_name(symbol_id)) {
                continue;
            }
            let index = *slot;
            frequencies[index].slot = *slot;
            frequencies[index].frequency +=
                symbol_table.get_resolved_reference_ids(symbol_id).len();
            frequencies[index].symbol_ids.push(symbol_id);
        }
        frequencies.sort_unstable_by_key(|x| std::cmp::Reverse(x.frequency));
        frequencies
    }
}

fn is_special_name(name: &str) -> bool {
    matches!(name, "exports" | "arguments")
}

#[derive(Debug, Default, Clone)]
struct SlotFrequency {
    pub slot: Slot,
    pub frequency: usize,
    pub symbol_ids: Vec<SymbolId>,
}

#[rustfmt::skip]
fn is_keyword(s: &str) -> bool {
    matches!(s, "as" | "do" | "if" | "in" | "is" | "of" | "any" | "for" | "get"
            | "let" | "new" | "out" | "set" | "try" | "var" | "case" | "else"
            | "enum" | "from" | "meta" | "null" | "this" | "true" | "type"
            | "void" | "with")
}

const BASE54_CHARS: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789";

/// Get the shortest mangled name for a given n.
/// Code adapted from [terser](https://github.com/terser/terser/blob/8b966d687395ab493d2c6286cc9dd38650324c11/lib/scope.js#L1041-L1051)
fn base54(n: usize) -> CompactStr {
    let mut num = n;
    // Base 54 at first because these are the usable first characters in JavaScript identifiers
    // <https://tc39.es/ecma262/#prod-IdentifierStart>
    let base = 54usize;
    let mut ret = String::new();
    ret.push(BASE54_CHARS[num % base] as char);
    num /= base;
    // Base 64 for the rest because after the first character we can also use 0-9 too
    // <https://tc39.es/ecma262/#prod-IdentifierPart>
    let base = 64usize;
    while num > 0 {
        num -= 1;
        ret.push(BASE54_CHARS[num % base] as char);
        num /= base;
    }
    CompactStr::new(&ret)
}

fn debug_name(n: usize) -> CompactStr {
    CompactStr::from(format!("slot_{n}"))
}
