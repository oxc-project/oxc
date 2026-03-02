use compact_str::CompactString;
use oxc_index::IndexVec;
use oxc_syntax::symbol::SymbolId;

use crate::types::{ModuleIdx, SymbolRef};

/// Symbol database — implements union-find across modules.
///
/// Hot data (links) is stored separately from cold data (names) for cache
/// locality: the `canonical_ref_for` hot loop only touches `module_links`.
#[derive(Debug, Default, Clone)]
pub struct SymbolRefDb {
    /// HOT: union-find link targets per module (read on every `canonical_ref_for`).
    module_links: IndexVec<ModuleIdx, IndexVec<SymbolId, SymbolRef>>,
    /// COLD: symbol names per module (diagnostics/codegen only).
    module_names: IndexVec<ModuleIdx, IndexVec<SymbolId, CompactString>>,
}

impl SymbolRefDb {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure the database has capacity for at least `module_count` modules.
    pub fn ensure_modules(&mut self, module_count: usize) {
        while self.module_links.len() < module_count {
            self.module_links.push(IndexVec::default());
            self.module_names.push(IndexVec::default());
        }
    }

    /// Ensure the per-module symbol storage can address at least `len` symbols.
    ///
    /// New slots are initialized with empty names and self-links so callers can
    /// populate parser-produced `SymbolId`s directly without remapping.
    pub fn ensure_module_symbol_capacity(&mut self, module: ModuleIdx, len: usize) {
        self.ensure_modules(module.index() + 1);

        let links = &mut self.module_links[module];
        let names = &mut self.module_names[module];
        while links.len() < len {
            let symbol_id = SymbolId::from_usize(links.len());
            links.push(SymbolRef::new(module, symbol_id));
            names.push(CompactString::default());
        }
    }

    /// Set the declared name of an existing symbol slot.
    ///
    /// The slot is created if needed.
    pub fn set_symbol_name(
        &mut self,
        module: ModuleIdx,
        symbol: SymbolId,
        name: impl Into<CompactString>,
    ) {
        self.ensure_module_symbol_capacity(module, symbol.index() + 1);
        self.module_names[module][symbol] = name.into();
    }

    /// Initialize or reset a symbol slot to link to itself.
    ///
    /// The slot is created if needed.
    pub fn init_symbol_self_link(&mut self, module: ModuleIdx, symbol: SymbolId) {
        self.ensure_module_symbol_capacity(module, symbol.index() + 1);
        self.module_links[module][symbol] = SymbolRef::new(module, symbol);
    }

    /// Add a symbol to a module and return its SymbolRef.
    pub fn add_symbol(&mut self, module: ModuleIdx, name: impl Into<CompactString>) -> SymbolRef {
        self.ensure_modules(module.index() + 1);
        let links = &mut self.module_links[module];
        let names = &mut self.module_names[module];
        let symbol_id = names.push(name.into());
        let sym_ref = SymbolRef::new(module, symbol_id);
        links.push(sym_ref);
        sym_ref
    }

    /// Allocate a new synthetic symbol at the end of a module's symbol table.
    pub fn alloc_synthetic_symbol(
        &mut self,
        module: ModuleIdx,
        name: impl Into<CompactString>,
    ) -> SymbolRef {
        self.add_symbol(module, name)
    }

    /// Initialize all symbols for a module in one call.
    ///
    /// Replaces N calls to `set_symbol_name` + `init_symbol_self_link` with a
    /// single allocation. All symbols are self-linked.
    pub fn init_module_symbols(&mut self, module: ModuleIdx, names: &[CompactString]) {
        self.ensure_modules(module.index() + 1);
        let links = &mut self.module_links[module];
        let name_vec = &mut self.module_names[module];

        // Pre-allocate
        links.reserve(names.len().saturating_sub(links.len()));
        name_vec.reserve(names.len().saturating_sub(name_vec.len()));

        for (i, name) in names.iter().enumerate() {
            let symbol_id = SymbolId::from_usize(i);
            let sym_ref = SymbolRef::new(module, symbol_id);
            if i < links.len() {
                links[symbol_id] = sym_ref;
                name_vec[symbol_id].clone_from(name);
            } else {
                links.push(sym_ref);
                name_vec.push(name.clone());
            }
        }
    }

    /// Check if a symbol slot exists for the given symbol ref.
    pub fn has_symbol(&self, symbol: SymbolRef) -> bool {
        symbol.owner.index() < self.module_links.len()
            && symbol.symbol.index() < self.module_links[symbol.owner].len()
    }

    /// Follow link chains to find the canonical (final) symbol.
    pub fn canonical_ref_for(&self, symbol: SymbolRef) -> SymbolRef {
        let mut current = symbol;
        loop {
            let next = self.module_links[current.owner][current.symbol];
            if next == current {
                return current;
            }
            current = next;
        }
    }

    /// Follow link chains to find the canonical symbol, applying path halving.
    ///
    /// # Panics
    ///
    /// Panics if a cycle is detected (more than 10 000 steps).
    pub fn canonical_ref_for_mut(&mut self, symbol: SymbolRef) -> SymbolRef {
        let mut current = symbol;
        let mut steps = 0u32;
        loop {
            let next = self.module_links[current.owner][current.symbol];
            if next == current {
                return current;
            }
            steps += 1;
            assert!(
                steps <= 10000,
                "canonical_ref_for_mut: cycle detected starting from ({}, {}), stuck at ({}, {})",
                symbol.owner.index(),
                symbol.symbol.index(),
                current.owner.index(),
                current.symbol.index(),
            );

            let next_next = self.module_links[next.owner][next.symbol];
            if next_next != next {
                self.module_links[current.owner][current.symbol] = next_next;
            }
            current = next;
        }
    }

    /// Link `from` to resolve to `to`.
    pub fn link(&mut self, from: SymbolRef, to: SymbolRef) {
        // Early exit: no-op if linking to self or already directly linked.
        if from == to {
            return;
        }
        if self.module_links[from.owner][from.symbol] == to {
            return;
        }

        let from_root = self.canonical_ref_for_mut(from);
        let to_root = self.canonical_ref_for_mut(to);
        if from_root != to_root {
            self.module_links[from_root.owner][from_root.symbol] = to_root;
        }
    }

    /// Flatten all union-find chains so every symbol points directly to its root.
    ///
    /// After calling this, all `canonical_ref_for()` calls are O(1) — one read,
    /// link == self means root. This enables lock-free parallel reads in the
    /// generate stage.
    pub fn flatten_all_chains(&mut self) {
        for module_idx in 0..self.module_links.len() {
            let module = ModuleIdx::from_usize(module_idx);
            for sym_idx in 0..self.module_links[module].len() {
                let sym = SymbolRef::new(module, SymbolId::from_usize(sym_idx));
                let root = self.canonical_ref_for_mut(sym);
                // Point directly to root (full compression, not just path halving)
                self.module_links[module][SymbolId::from_usize(sym_idx)] = root;
            }
        }
    }

    /// Get the declared name of a symbol.
    pub fn symbol_name(&self, symbol: SymbolRef) -> &str {
        &self.module_names[symbol.owner][symbol.symbol]
    }

    /// Get the owning module of a symbol.
    pub fn symbol_owner(symbol: SymbolRef) -> ModuleIdx {
        symbol.owner
    }
}
