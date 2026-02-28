use oxc_index::IndexVec;
use oxc_syntax::symbol::SymbolId;

use crate::types::{ModuleIdx, SymbolRef};

/// Per-module symbol table: stores name + link for each symbol.
#[derive(Debug, Default)]
struct ModuleSymbols {
    /// Symbol name (indexed by SymbolId).
    names: IndexVec<SymbolId, String>,
    /// Link target for union-find. Points to self initially.
    links: IndexVec<SymbolId, SymbolRef>,
}

/// Symbol database — implements union-find across modules.
#[derive(Debug, Default)]
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

    /// Add a symbol to a module and return its SymbolRef.
    pub fn add_symbol(&mut self, module: ModuleIdx, name: String) -> SymbolRef {
        let symbols = &mut self.modules[module];
        let symbol_id = symbols.names.push(name);
        let sym_ref = SymbolRef::new(module, symbol_id);
        symbols.links.push(sym_ref);
        sym_ref
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

    /// Link `from` to resolve to `to`.
    pub fn link(&mut self, from: SymbolRef, to: SymbolRef) {
        self.modules[from.owner].links[from.symbol] = to;
    }

    /// Get the declared name of a symbol.
    pub fn symbol_name(&self, symbol: SymbolRef) -> &str {
        &self.modules[symbol.owner].names[symbol.symbol]
    }

    /// Get the owning module of a symbol.
    pub fn symbol_owner(symbol: SymbolRef) -> ModuleIdx {
        symbol.owner
    }
}
