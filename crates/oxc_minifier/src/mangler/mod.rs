use std::collections::BTreeMap;

#[allow(clippy::wildcard_imports)]
use oxc_hir::hir::*;
use oxc_hir::Visit;
use oxc_index::{index_vec, IndexVec};
use oxc_semantic2::{
    reference::ReferenceId,
    symbol::{SymbolId, SymbolTable},
    SemanticBuilder,
};
use oxc_span::{Atom, SourceType, Span};
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolFlags};

type Slot = usize;

pub struct Mangler {
    symbol_table: SymbolTable,
    symbol_map: BTreeMap<Span, SymbolId>,
    reference_map: BTreeMap<Span, ReferenceId>,
}

impl Mangler {
    pub fn get_symbol_name(&self, span: Span) -> Option<&Atom> {
        let symbol_id = self.symbol_map.get(&span)?;
        Some(self.symbol_table.get_name(*symbol_id))
    }

    pub fn get_reference_name(&self, span: Span) -> Option<&Atom> {
        let reference_id = self.reference_map.get(&span)?;
        let symbol_id = self.symbol_table.get_reference(*reference_id).symbol_id?;
        Some(self.symbol_table.get_name(symbol_id))
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
pub struct ManglerBuilder {
    semantic: SemanticBuilder,
    symbol_map: BTreeMap<Span, SymbolId>,
    reference_map: BTreeMap<Span, ReferenceId>,
}

impl<'a> Visit<'a> for ManglerBuilder {
    fn enter_scope(&mut self, flags: ScopeFlags) {
        self.semantic.enter_scope(flags);
    }

    fn leave_scope(&mut self) {
        self.semantic.leave_scope();
    }

    fn visit_binding_identifier(
        &mut self,
        ident: &'a BindingIdentifier,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) {
        let new_symbol_id =
            self.semantic.declare_symbol(ident.span, &ident.name, includes, excludes);
        self.symbol_map.insert(ident.span, new_symbol_id);
    }

    fn visit_identifier_reference(&mut self, ident: &'a IdentifierReference) {
        let new_reference_id = self.semantic.declare_reference(ident.span, &ident.name);
        self.reference_map.insert(ident.span, new_reference_id);
    }
}

impl ManglerBuilder {
    pub fn new(source_type: SourceType) -> Self {
        Self {
            semantic: SemanticBuilder::new(source_type),
            symbol_map: BTreeMap::default(),
            reference_map: BTreeMap::default(),
        }
    }

    #[must_use]
    pub fn build<'a>(mut self, program: &'a Program<'a>) -> Mangler {
        self.visit_program(program);
        self.mangle()
    }

    /// Mangle the symbol table by computing slots from the scope tree.
    /// A slot is the occurrence index of a binding identifier inside a scope.
    pub fn mangle(self) -> Mangler {
        let semantic = self.semantic.build();
        let scope_tree = semantic.scope_tree;
        let mut symbol_table = semantic.symbol_table;

        // Total number of slots for all scopes
        let mut total_number_of_slots: Slot = 0;

        // All symbols with their assigned slots
        let mut slots: IndexVec<SymbolId, Slot> = index_vec![0; symbol_table.len()];

        // Keep track of the maximum slot number for each scope
        let mut max_slot_for_scope = vec![0; scope_tree.len()];

        // Walk the scope tree and compute the slot number for each scope
        for scope_id in scope_tree.descendants() {
            let bindings = scope_tree.get_bindings(scope_id);
            // The current slot number is continued by the maximum slot from the parent scope
            let parent_max_slot = scope_tree
                .get_parent_id(scope_id)
                .map_or(0, |parent_scope_id| max_slot_for_scope[parent_scope_id.index()]);

            let mut slot = parent_max_slot;

            // `bindings` are stored in order, traverse and increment slot
            for symbol_id in bindings.values() {
                slots[*symbol_id] = slot;
                slot += 1;
            }

            max_slot_for_scope[scope_id.index()] = slot;

            if slot > total_number_of_slots {
                total_number_of_slots = slot;
            }
        }

        let frequencies =
            Self::tally_slot_frequencies(&symbol_table, total_number_of_slots, &slots);

        let unresolved_references = scope_tree
            .root_unresolved_references()
            .keys()
            // It is unlike to get a 5 letter mangled identifier, which is a lot of slots.
            // .filter(|name| name.len() < 5)
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
                symbol_table.set_name(*symbol_id, name.clone());
            }
        }

        Mangler { symbol_table, symbol_map: self.symbol_map, reference_map: self.reference_map }
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
            let index = *slot;
            frequencies[index].slot = *slot;
            frequencies[index].frequency += symbol_table.get_resolved_references(symbol_id).len();
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
