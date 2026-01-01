use std::cell::RefCell;

use oxc_ecmascript::constant_evaluation::ConstantValue;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_data_structures::stack::NonEmptyStack;
use oxc_semantic::{ReferenceId, Scoping};
use oxc_span::{Atom, SourceType};
use oxc_syntax::symbol::SymbolId;

use crate::{CompressOptions, symbol_value::SymbolValues};

pub struct MinifierState<'a> {
    pub source_type: SourceType,

    pub options: CompressOptions,

    /// The return value of function declarations that are pure
    pub pure_functions: FxHashMap<SymbolId, Option<ConstantValue<'a>>>,

    pub symbol_values: SymbolValues<'a>,

    /// Private member usage for classes
    pub class_symbols_stack: ClassSymbolsStack<'a>,

    pub changed: bool,

    /// Maps each reference directly to its innermost containing function/class symbol.
    /// `None` means the reference is at the top level (not inside any function/class body).
    /// This is pre-computed during AST traversal to avoid walking up the scope chain
    /// during analysis.
    pub reference_owning_symbol: FxHashMap<ReferenceId, Option<SymbolId>>,

    /// Set of symbols that are named function expression inner bindings.
    /// For `var outer = function inner() { ... }`, `inner` is in this set.
    /// These symbols are declared in their own scope and require conservative treatment.
    pub named_function_expr_symbols: FxHashSet<SymbolId>,

    /// Memoization cache for `is_symbol_effectively_unused()` results.
    /// This avoids redundant computation when checking circular dependencies.
    /// Cleared between minifier iterations.
    /// Uses `RefCell` for interior mutability to allow caching during analysis
    /// without requiring `&mut self`.
    pub unused_symbol_cache: RefCell<FxHashMap<SymbolId, bool>>,
}

impl MinifierState<'_> {
    pub fn new(source_type: SourceType, options: CompressOptions) -> Self {
        Self {
            source_type,
            options,
            pure_functions: FxHashMap::default(),
            symbol_values: SymbolValues::default(),
            class_symbols_stack: ClassSymbolsStack::new(),
            changed: false,
            reference_owning_symbol: FxHashMap::default(),
            named_function_expr_symbols: FxHashSet::default(),
            unused_symbol_cache: RefCell::new(FxHashMap::default()),
        }
    }

    /// Check if a symbol is effectively unused.
    ///
    /// A symbol is effectively unused if all its references are inside the bodies
    /// of symbols that are also effectively unused (including self-references).
    ///
    /// This handles circular dependencies where symbols only reference each other
    /// but are never called from live code.
    ///
    /// Note: This should only be called for symbols that have at least one reference.
    /// Symbols with zero references should be handled by `symbol_is_unused`.
    pub fn is_symbol_effectively_unused(
        &self,
        symbol_id: SymbolId,
        scoping: &Scoping,
        visited: &mut FxHashSet<SymbolId>,
    ) -> bool {
        // Check cache first - but only if we're not in the middle of cycle detection.
        // During cycle detection, cached results may be stale (computed with different
        // assumptions about which symbols are in the cycle).
        if visited.is_empty()
            && let Some(&cached_result) = self.unused_symbol_cache.borrow().get(&symbol_id)
        {
            return cached_result;
        }

        // Already checking this symbol - we're in a cycle, treat as unused for now.
        // If the cycle has no external entry point, all symbols in it are unused.
        if !visited.insert(symbol_id) {
            return true;
        }

        let result = self.is_symbol_effectively_unused_inner(symbol_id, scoping, visited);

        // Only cache results when we're at the top level (visited becomes empty after
        // removing this symbol). Results computed during recursion may be incorrect
        // due to cycle detection heuristics.
        visited.remove(&symbol_id);
        if visited.is_empty() {
            self.unused_symbol_cache.borrow_mut().insert(symbol_id, result);
        }

        result
    }

    /// Inner implementation of `is_symbol_effectively_unused` without caching logic.
    fn is_symbol_effectively_unused_inner(
        &self,
        symbol_id: SymbolId,
        scoping: &Scoping,
        visited: &mut FxHashSet<SymbolId>,
    ) -> bool {
        let reference_ids = scoping.get_resolved_reference_ids(symbol_id);

        // If truly no references, this symbol might still be "live" (e.g., exported).
        // We can't determine that here, so be conservative and say it's NOT unused.
        // The caller should check `symbol_is_unused` first for the zero-reference case.
        if reference_ids.is_empty() {
            // This is a leaf in our recursion - a symbol with no references.
            // If we got here via recursion, it means this symbol is being checked
            // as a potential container for another reference. A symbol with no
            // references at top level might be exported, so we should NOT consider
            // it unused (which would make references inside it "dead").
            return false;
        }

        for &reference_id in reference_ids {
            // Look up the pre-computed owning symbol for this reference
            let Some(owning_symbol_opt) = self.reference_owning_symbol.get(&reference_id) else {
                // If we don't have owning symbol info, be conservative
                return false;
            };

            let Some(containing_symbol) = *owning_symbol_opt else {
                // Reference is at top level (not inside any function/class body)
                // This is a live reference.
                return false;
            };

            if containing_symbol == symbol_id {
                // Self-reference - this is fine, continue to next reference
                continue;
            }

            // Check if the containing symbol is a named function expression's inner binding.
            // For named function expressions like `var outer = function inner() { ... }`,
            // the `inner` symbol is declared in the function's own scope, not the parent.
            // This inner binding typically only has self-references and appears unused,
            // but the function may still be live via the outer variable binding.
            // We must be conservative here and treat such containing symbols as LIVE.
            if self.named_function_expr_symbols.contains(&containing_symbol) {
                return false;
            }

            // Check if the containing symbol is also effectively unused
            if !self.is_symbol_effectively_unused(containing_symbol, scoping, visited) {
                // Reference is inside a LIVE function/class - symbol is used
                return false;
            }
            // This reference is in dead code, continue to next reference
        }

        true
    }
}

/// Stack to track class symbol information
pub struct ClassSymbolsStack<'a> {
    stack: NonEmptyStack<FxHashSet<Atom<'a>>>,
}

impl<'a> ClassSymbolsStack<'a> {
    pub fn new() -> Self {
        Self { stack: NonEmptyStack::new(FxHashSet::default()) }
    }

    /// Check if the stack is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.stack.is_exhausted()
    }

    /// Enter a new class scope
    pub fn push_class_scope(&mut self) {
        self.stack.push(FxHashSet::default());
    }

    /// Exit the current class scope
    pub fn pop_class_scope(&mut self, declared_private_symbols: impl Iterator<Item = Atom<'a>>) {
        let mut used_private_symbols = self.stack.pop();
        declared_private_symbols.for_each(|name| {
            used_private_symbols.remove(&name);
        });
        // if the symbol was not declared in this class, that is declared in the class outside the current class
        self.stack.last_mut().extend(used_private_symbols);
    }

    /// Add a private member to the current class scope
    pub fn push_private_member_to_current_class(&mut self, name: Atom<'a>) {
        self.stack.last_mut().insert(name);
    }

    /// Check if a private member is used in the current class scope
    pub fn is_private_member_used_in_current_class(&self, name: &Atom<'a>) -> bool {
        self.stack.last().contains(name)
    }
}
