use std::collections::BTreeMap;

use oxc_index::Idx;

use crate::symbol::{SymbolId, SymbolTable};

/// A slot is the occurrence index of a binding identifier inside a scope.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Slot(usize);

impl Idx for Slot {
    fn new(idx: usize) -> Self {
        Self(idx)
    }

    fn index(self) -> usize {
        self.0
    }
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
/// function x(slot0, slot1) {
///     function y(slot2, slot3) {
///         slot0 = 1;
///     }
/// }
/// function z(slot0, slot1, slot3, slot4) {
///     slot0 = 1;
/// }
/// ```
///
/// Occurrences of slots and their corresponding newly assigned short identifiers (mangled names) are:
/// - slot0: 4 - a
/// - slot1: 2 - b
/// - slot3: 2 - c
/// - slot2: 1 - d
/// - slot4: 1 - e
///
/// After swapping out the mangled names, the functions become:
///
/// ```javascript
/// function x(a, b) {
///     function y(d, c) {
///         a = 1;
///     }
/// }
/// function z(a, b, c, e) {
///     a = 1;
/// }
/// ```
#[derive(Debug, Default)]
pub struct Mangler {
    /// The current slot used by semantic builder
    pub(crate) current_slot: Slot,

    /// The maximum slot of all scopes.
    pub(crate) max_slot: Slot,

    pub(crate) slots: BTreeMap<SymbolId, Slot>,
}

#[derive(Debug, Clone)]
pub struct SlotFrequency {
    pub slot: Slot,
    pub frequency: usize,
    pub symbol_ids: Vec<SymbolId>,
}

impl SlotFrequency {
    fn new() -> Self {
        Self { slot: Slot::new(0), frequency: 0, symbol_ids: vec![] }
    }
}

impl Mangler {
    pub fn add_slot(&mut self, symbol_id: SymbolId) {
        self.slots.insert(symbol_id, self.current_slot);
        self.current_slot.increment();
        if self.current_slot > self.max_slot {
            self.max_slot = self.current_slot;
        }
    }

    pub fn decrease_slot(&mut self, n: usize) {
        self.current_slot = Slot::new(self.current_slot.index() - n);
    }

    pub fn tally_slot_frequency(&self, symbol_table: &SymbolTable) -> Vec<SlotFrequency> {
        let mut frequencies = vec![SlotFrequency::new(); self.max_slot.index()];
        for (symbol_id, slot) in &self.slots {
            let index = slot.index();
            frequencies[index].slot = *slot;
            frequencies[index].frequency += symbol_table.references[*symbol_id].len();
            frequencies[index].symbol_ids.push(*symbol_id);
        }
        frequencies.sort_by_key(|x| std::cmp::Reverse(x.frequency));
        frequencies
    }

    #[rustfmt::skip]
    pub(crate) fn is_keyword(s: &str) -> bool {
        let len = s.len();
        if len == 1 {
            return false;
        }
        matches!(s, "as" | "do" | "if" | "in" | "is" | "of" | "any" | "for" | "get"
                | "let" | "new" | "out" | "set" | "try" | "var" | "case" | "else"
                | "enum" | "from" | "meta" | "null" | "this" | "true" | "type"
                | "void" | "with")
    }
}
