use oxc_index::{Idx, IndexVec};
use oxc_span::Atom;

use crate::{
    symbol::{SymbolId, SymbolTable},
    Semantic,
};

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
#[derive(Debug, Default)]
pub struct Mangler;

type Slot = usize;

impl Mangler {
    /// Mangle the symbol table by computing slots from the scope tree.
    /// A slot is the occurrence index of a binding identifier inside a scope.
    pub fn mangle(semantic: &mut Semantic) {
        // Total number of slots for all scopes
        let mut total_number_of_slots: Slot = 0;

        // All symbols with their assigned slots
        let mut slots: IndexVec<SymbolId, Slot> =
            IndexVec::from_raw(vec![0; semantic.symbol_table.len()]);

        // Keep track of the maximum slot number for each scope
        let mut max_slot_for_scope = vec![0; semantic.scope_tree.len()];

        // Walk the scope tree and compute the slot number for each scope
        for (scope_id, scope) in semantic.scope_tree.descendants() {
            // Skip if the scope is empty
            if scope.bindings().is_empty() {
                continue;
            }

            // The current slot number is continued by the maximum slot from the parent scope
            let parent_max_slot = scope
                .parent_id()
                .map_or(0, |parent_scope_id| max_slot_for_scope[parent_scope_id.index()]);

            let mut slot = parent_max_slot;

            // `bindings` are stored in order, traverse and increment slot
            for symbol_id in scope.bindings().values() {
                slots[*symbol_id] = slot;
                slot += 1;
            }

            max_slot_for_scope[scope_id.index()] = slot;

            if slot > total_number_of_slots {
                total_number_of_slots = slot;
            }
        }

        let frequencies =
            Self::tally_slot_frequencies(&semantic.symbol_table, total_number_of_slots, &slots);

        let unresolved_references = semantic
            .symbol_table
            .unresolved_references
            .values()
            .filter(|name| name.len() < 5)
            .collect::<Vec<_>>();

        // Compute short identifiers by slot frequency
        let mut count = 0;
        for freq in &frequencies {
            let name = loop {
                let name = Atom::base54(count);
                count += 1;
                // Do not mangle keywords and unresolved references
                if !is_keyword(&name) && !unresolved_references.iter().any(|n| **n == name) {
                    break name;
                }
            };
            // All symbols for the same frequency gets the same
            for symbol_id in &freq.symbol_ids {
                semantic.symbol_table.names[*symbol_id] = name.clone();
            }
        }
    }

    fn tally_slot_frequencies(
        symbol_table: &SymbolTable,
        total_number_of_slots: usize,
        slots: &IndexVec<SymbolId, Slot>,
    ) -> Vec<SlotFrequency> {
        let mut frequencies = vec![SlotFrequency::default(); total_number_of_slots];
        for (symbol_id, slot) in slots.iter_enumerated() {
            if !symbol_table.get_flag(symbol_id).is_variable() {
                continue;
            }
            let index = slot.index();
            frequencies[index].slot = *slot;
            frequencies[index].frequency += symbol_table.references[symbol_id].len();
            frequencies[index].symbol_ids.push(symbol_id);
        }
        frequencies.sort_by_key(|x| (std::cmp::Reverse(x.frequency)));
        frequencies
    }
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
