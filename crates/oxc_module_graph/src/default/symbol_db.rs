use oxc_index::IndexVec;
use oxc_syntax::symbol::SymbolId;

use crate::types::{ModuleIdx, SymbolRef};

/// Per-module symbol table: stores name + link for each symbol.
#[derive(Debug, Default, Clone)]
struct ModuleSymbols {
    /// Symbol name (indexed by SymbolId).
    names: IndexVec<SymbolId, String>,
    /// Link target for union-find. Points to self initially.
    links: IndexVec<SymbolId, SymbolRef>,
}

/// Symbol database — implements union-find across modules.
#[derive(Debug, Default, Clone)]
pub struct SymbolRefDb {
    modules: IndexVec<ModuleIdx, ModuleSymbols>,
}

impl SymbolRefDb {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure the database has capacity for at least `module_count` modules.
    pub fn ensure_modules(&mut self, module_count: usize) {
        while self.modules.len() < module_count {
            self.modules.push(ModuleSymbols::default());
        }
    }

    /// Ensure the per-module symbol storage can address at least `len` symbols.
    ///
    /// New slots are initialized with empty names and self-links so callers can
    /// populate parser-produced `SymbolId`s directly without remapping.
    pub fn ensure_module_symbol_capacity(&mut self, module: ModuleIdx, len: usize) {
        self.ensure_modules(module.index() + 1);

        let symbols = &mut self.modules[module];
        while symbols.names.len() < len {
            let symbol_id = SymbolId::from_usize(symbols.names.len());
            symbols.names.push(String::default());
            symbols.links.push(SymbolRef::new(module, symbol_id));
        }
    }

    /// Set the declared name of an existing symbol slot.
    ///
    /// The slot is created if needed.
    pub fn set_symbol_name(&mut self, module: ModuleIdx, symbol: SymbolId, name: String) {
        self.ensure_module_symbol_capacity(module, symbol.index() + 1);
        self.modules[module].names[symbol] = name;
    }

    /// Initialize or reset a symbol slot to link to itself.
    ///
    /// The slot is created if needed.
    pub fn init_symbol_self_link(&mut self, module: ModuleIdx, symbol: SymbolId) {
        self.ensure_module_symbol_capacity(module, symbol.index() + 1);
        self.modules[module].links[symbol] = SymbolRef::new(module, symbol);
    }

    /// Add a symbol to a module and return its SymbolRef.
    pub fn add_symbol(&mut self, module: ModuleIdx, name: String) -> SymbolRef {
        self.ensure_modules(module.index() + 1);
        let symbols = &mut self.modules[module];
        let symbol_id = symbols.names.push(name);
        let sym_ref = SymbolRef::new(module, symbol_id);
        symbols.links.push(sym_ref);
        sym_ref
    }

    /// Allocate a new synthetic symbol at the end of a module's symbol table.
    pub fn alloc_synthetic_symbol(&mut self, module: ModuleIdx, name: String) -> SymbolRef {
        self.add_symbol(module, name)
    }

    /// Check if a symbol slot exists for the given symbol ref.
    pub fn has_symbol(&self, symbol: SymbolRef) -> bool {
        symbol.owner.index() < self.modules.len()
            && symbol.symbol.index() < self.modules[symbol.owner].links.len()
    }

    /// Follow link chains to find the canonical (final) symbol.
    pub fn canonical_ref_for(&self, symbol: SymbolRef) -> SymbolRef {
        let mut current = symbol;
        loop {
            let next = self.modules[current.owner].links[current.symbol];
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
            let next = self.modules[current.owner].links[current.symbol];
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

            let next_next = self.modules[next.owner].links[next.symbol];
            if next_next != next {
                self.modules[current.owner].links[current.symbol] = next_next;
            }
            current = next;
        }
    }

    /// Link `from` to resolve to `to`.
    pub fn link(&mut self, from: SymbolRef, to: SymbolRef) {
        let from_root = self.canonical_ref_for_mut(from);
        let to_root = self.canonical_ref_for_mut(to);
        if from_root != to_root {
            self.modules[from_root.owner].links[from_root.symbol] = to_root;
        }
    }

    /// Get the declared name of a symbol.
    pub fn symbol_name(&self, symbol: SymbolRef) -> &str {
        &self.modules[symbol.owner].names[symbol.symbol]
    }

    /// Flatten all union-find chains using path halving.
    ///
    /// The immutable `canonical_ref_for` cannot do path compression, so chains
    /// built during linking may be multi-hop. This method walks every symbol
    /// once via `canonical_ref_for_mut` (which applies path halving), so that
    /// all subsequent immutable lookups resolve in a single read.
    ///
    /// Call this after all `link()` calls are complete and before the generate
    /// stage's hot loops that call `canonical_ref_for` thousands of times.
    pub fn flatten_all_chains(&mut self) {
        for module_idx_raw in 0..self.modules.len() {
            let module_idx = ModuleIdx::from_usize(module_idx_raw);
            let num_symbols = self.modules[module_idx].links.len();
            for symbol_idx in 0..num_symbols {
                let sym = SymbolRef::new(module_idx, SymbolId::from_usize(symbol_idx));
                self.canonical_ref_for_mut(sym);
            }
        }
    }

    /// Get the owning module of a symbol.
    pub fn symbol_owner(symbol: SymbolRef) -> ModuleIdx {
        symbol.owner
    }
}
