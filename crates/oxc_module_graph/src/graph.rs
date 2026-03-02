use compact_str::CompactString;
use oxc_index::IndexVec;

use crate::algo::BindingError;
use crate::default::SymbolRefDb;
use crate::module::{ExternalModule, Module, NormalModule};
use crate::types::{ModuleIdx, SymbolRef};
use oxc_syntax::symbol::SymbolId;

/// A concrete, batteries-included module graph.
///
/// Owns both the module storage and the symbol database.
/// Algorithms operate directly on `&ModuleGraph` / `&mut ModuleGraph`
/// instead of through trait bounds.
#[derive(Debug, Default, Clone)]
pub struct ModuleGraph {
    /// All modules (normal and external), indexed by `ModuleIdx`.
    pub modules: IndexVec<ModuleIdx, Module>,
    /// Cross-module symbol database with union-find linking.
    pub symbols: SymbolRefDb,

    /// Entry point module indices.
    entries: Vec<ModuleIdx>,
    /// Optional runtime module index.
    runtime: Option<ModuleIdx>,

    // --- Global link-time results ---
    /// Modules in execution order.
    exec_order: Vec<ModuleIdx>,
    /// Detected circular dependencies.
    cycles: Vec<Vec<ModuleIdx>>,
    /// Binding errors from import matching.
    binding_errors: Vec<BindingError>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Build API ---

    /// Allocate a module index for a module that will be added later.
    pub fn alloc_module_idx(&mut self) -> ModuleIdx {
        let idx = self.modules.next_idx();
        // Push a placeholder — will be replaced by add_normal_module or add_external_module.
        // We use an external module placeholder since it's smaller.
        self.modules.push(Module::External(ExternalModule {
            idx,
            specifier: compact_str::CompactString::default(),
            side_effects: crate::module::SideEffects::True,
            namespace_ref: SymbolRef::new(idx, oxc_syntax::symbol::SymbolId::from_raw_unchecked(0)),
            exec_order: u32::MAX,
        }));
        self.symbols.ensure_modules(idx.index() + 1);
        idx
    }

    /// Add a normal module to the graph.
    ///
    /// The module's `idx` must match an allocated index.
    pub fn add_normal_module(&mut self, module: NormalModule) {
        let idx = module.idx;
        if idx.index() < self.modules.len() {
            self.modules[idx] = Module::Normal(Box::new(module));
        } else {
            // Extend to fit
            while self.modules.len() < idx.index() {
                let placeholder_idx = self.modules.next_idx();
                self.modules.push(Module::External(ExternalModule {
                    idx: placeholder_idx,
                    specifier: compact_str::CompactString::default(),
                    side_effects: crate::module::SideEffects::True,
                    namespace_ref: SymbolRef::new(
                        placeholder_idx,
                        oxc_syntax::symbol::SymbolId::from_raw_unchecked(0),
                    ),
                    exec_order: u32::MAX,
                }));
            }
            debug_assert_eq!(idx, self.modules.next_idx());
            self.modules.push(Module::Normal(Box::new(module)));
        }
        self.symbols.ensure_modules(idx.index() + 1);
    }

    /// Add an external module to the graph.
    ///
    /// The module's `idx` must match an allocated index.
    pub fn add_external_module(&mut self, module: ExternalModule) {
        let idx = module.idx;
        if idx.index() < self.modules.len() {
            self.modules[idx] = Module::External(module);
        } else {
            while self.modules.len() < idx.index() {
                let placeholder_idx = self.modules.next_idx();
                self.modules.push(Module::External(ExternalModule {
                    idx: placeholder_idx,
                    specifier: compact_str::CompactString::default(),
                    side_effects: crate::module::SideEffects::True,
                    namespace_ref: SymbolRef::new(
                        placeholder_idx,
                        oxc_syntax::symbol::SymbolId::from_raw_unchecked(0),
                    ),
                    exec_order: u32::MAX,
                }));
            }
            debug_assert_eq!(idx, self.modules.next_idx());
            self.modules.push(Module::External(module));
        }
        self.symbols.ensure_modules(idx.index() + 1);
    }

    /// Add a symbol to the symbol database.
    pub fn add_symbol(&mut self, module: ModuleIdx, name: impl Into<CompactString>) -> SymbolRef {
        self.symbols.add_symbol(module, name)
    }

    /// Ensure the given module has symbol slots for at least `len` symbols.
    pub fn ensure_module_symbol_capacity(&mut self, module: ModuleIdx, len: usize) {
        self.symbols.ensure_module_symbol_capacity(module, len);
    }

    /// Set the name for an existing symbol slot.
    pub fn set_symbol_name(
        &mut self,
        module: ModuleIdx,
        symbol: SymbolId,
        name: impl Into<CompactString>,
    ) {
        self.symbols.set_symbol_name(module, symbol, name);
    }

    /// Initialize or reset a symbol slot to point to itself.
    pub fn init_symbol_self_link(&mut self, module: ModuleIdx, symbol: SymbolId) {
        self.symbols.init_symbol_self_link(module, symbol);
    }

    /// Allocate a new synthetic symbol in a module.
    pub fn alloc_synthetic_symbol(
        &mut self,
        module: ModuleIdx,
        name: impl Into<CompactString>,
    ) -> SymbolRef {
        self.symbols.alloc_synthetic_symbol(module, name)
    }

    /// Initialize all symbols for a module in one call.
    pub fn init_module_symbols(&mut self, module: ModuleIdx, names: &[CompactString]) {
        self.symbols.init_module_symbols(module, names);
    }

    /// Pre-allocate capacity for `additional` more modules.
    ///
    /// Useful when the total module count is known upfront (e.g. during a scan stage)
    /// to avoid repeated resizing.
    pub fn reserve_modules(&mut self, additional: usize) {
        self.modules.reserve(additional);
    }

    /// Set the entry points for the graph.
    pub fn set_entries(&mut self, entries: Vec<ModuleIdx>) {
        self.entries = entries;
    }

    /// Set the runtime module.
    pub fn set_runtime(&mut self, runtime: ModuleIdx) {
        self.runtime = Some(runtime);
    }

    // --- Query API ---

    /// Get a module by index.
    pub fn module(&self, idx: ModuleIdx) -> &Module {
        &self.modules[idx]
    }

    /// Get a module by index (mutable).
    pub fn module_mut(&mut self, idx: ModuleIdx) -> &mut Module {
        &mut self.modules[idx]
    }

    /// Get a normal module by index (returns `None` for external modules).
    pub fn normal_module(&self, idx: ModuleIdx) -> Option<&NormalModule> {
        self.modules.get(idx).and_then(Module::as_normal)
    }

    /// Get a mutable normal module by index.
    pub fn normal_module_mut(&mut self, idx: ModuleIdx) -> Option<&mut NormalModule> {
        self.modules.get_mut(idx).and_then(Module::as_normal_mut)
    }

    /// Get an external module by index.
    pub fn external_module(&self, idx: ModuleIdx) -> Option<&ExternalModule> {
        self.modules.get(idx).and_then(Module::as_external)
    }

    /// Number of modules (normal + external) in the graph.
    pub fn modules_len(&self) -> usize {
        self.modules.len()
    }

    /// Iterate over all normal modules.
    pub fn normal_modules(&self) -> impl Iterator<Item = &NormalModule> {
        self.modules.iter().filter_map(Module::as_normal)
    }

    /// Check if a symbol slot exists in the symbol database.
    pub fn has_symbol(&self, sym: SymbolRef) -> bool {
        self.symbols.has_symbol(sym)
    }

    /// Follow link chains to find the canonical (final) symbol.
    pub fn canonical_ref(&self, sym: SymbolRef) -> SymbolRef {
        self.symbols.canonical_ref_for(sym)
    }

    /// Follow link chains to find the canonical symbol, applying path halving.
    pub fn canonical_ref_mut(&mut self, sym: SymbolRef) -> SymbolRef {
        self.symbols.canonical_ref_for_mut(sym)
    }

    /// Link `from` to resolve to `to`.
    pub fn link_symbols(&mut self, from: SymbolRef, to: SymbolRef) {
        self.symbols.link(from, to);
    }

    /// Get the declared name of a symbol.
    pub fn symbol_name(&self, sym: SymbolRef) -> &str {
        self.symbols.symbol_name(sym)
    }

    /// Get the entry points.
    pub fn entries(&self) -> &[ModuleIdx] {
        &self.entries
    }

    /// Get the runtime module index.
    pub fn runtime(&self) -> Option<ModuleIdx> {
        self.runtime
    }

    // --- Link API ---

    /// Run the full link pipeline.
    ///
    /// This calls all algorithms in order and stores results in-place on each module.
    /// Consumers that need different ordering (like Rolldown, which runs
    /// `determine_module_exports_kind` between steps) can call individual
    /// algorithm methods instead.
    pub fn link(&mut self, config: &mut crate::hooks::LinkConfig) {
        use crate::algo;

        // 1. Execution order
        let exec_config =
            algo::ExecOrderConfig { include_dynamic_imports: config.include_dynamic_imports };
        let exec = algo::compute_exec_order(self, config);
        self.exec_order = exec.sorted;
        self.cycles = exec.cycles;
        #[expect(clippy::cast_possible_truncation)]
        for (i, &idx) in self.exec_order.iter().enumerate() {
            match &mut self.modules[idx] {
                Module::Normal(m) => m.exec_order = i as u32,
                Module::External(m) => m.exec_order = i as u32,
            }
        }
        // Keep exec_config for later reference (it's on the stack, that's fine)
        let _ = exec_config;

        // 2. TLA propagation
        let tla = algo::compute_tla(self);
        for &idx in &tla {
            if let Module::Normal(m) = &mut self.modules[idx] {
                m.is_tla_or_contains_tla = true;
            }
        }

        // 2b. Determine module exports kind
        let ek_config =
            algo::ExportsKindConfig { dynamic_imports_as_require: false, wrap_cjs_entries: true };
        let ek_result = algo::determine_module_exports_kind(self, &ek_config);
        for (idx, kind) in &ek_result.exports_kind_updates {
            if let Module::Normal(m) = &mut self.modules[*idx] {
                m.exports_kind = *kind;
            }
        }
        for (idx, wrap) in &ek_result.wrap_kind_updates {
            if let Module::Normal(m) = &mut self.modules[*idx] {
                m.wrap_kind = *wrap;
            }
        }

        // 2c. Wrap modules
        let wrap_config = algo::WrapModulesConfig {
            on_demand_wrapping: false,
            strict_execution_order: false,
            skip_symbol_creation: false,
        };
        let wrap_result = algo::wrap_modules(self, &wrap_config);
        for (idx, wrap) in &wrap_result.wrap_kind_updates {
            if let Module::Normal(m) = &mut self.modules[*idx] {
                m.wrap_kind = *wrap;
            }
        }
        for (idx, wrapper) in &wrap_result.wrapper_refs {
            if let Module::Normal(m) = &mut self.modules[*idx] {
                m.wrapper_ref = Some(*wrapper);
            }
        }
        for (&idx, &orig) in &wrap_result.original_wrap_kinds {
            if let Module::Normal(m) = &mut self.modules[idx] {
                m.original_wrap_kind = orig;
            }
        }
        for &idx in &wrap_result.required_by_other_module {
            if let Module::Normal(m) = &mut self.modules[idx] {
                m.required_by_other_module = true;
            }
        }

        // 3. Dynamic exports (runs after exports_kind is finalized)
        let dynamic = algo::compute_has_dynamic_exports(self);
        for &idx in &dynamic {
            if let Module::Normal(m) = &mut self.modules[idx] {
                m.has_dynamic_exports = true;
            }
        }

        // 4. Resolved exports
        let resolved = algo::build_resolved_exports(self);
        for (idx, exports) in resolved {
            if let Module::Normal(m) = &mut self.modules[idx] {
                m.resolved_exports = exports;
            }
        }

        // 5. Match imports (with built-in CJS interop)
        let (errors, links) = algo::match_imports_collect(self, config);
        for (from, to) in links {
            self.symbols.link(from, to);
        }
        self.binding_errors = errors;

        // 6. Side effects propagation
        let se = algo::determine_side_effects(self, config);
        for (idx, has) in se {
            if let Module::Normal(m) = &mut self.modules[idx] {
                m.propagated_side_effects = has;
            }
        }
    }

    /// Flatten all symbol union-find chains so every symbol points directly
    /// to its root. After calling this, all `canonical_ref()` calls are O(1).
    pub fn flatten_symbol_chains(&mut self) {
        self.symbols.flatten_all_chains();
    }

    // --- Post-link queries ---

    /// Get module indices in execution order.
    pub fn exec_order(&self) -> &[ModuleIdx] {
        &self.exec_order
    }

    /// Get detected circular dependencies.
    pub fn cycles(&self) -> &[Vec<ModuleIdx>] {
        &self.cycles
    }

    /// Get binding errors from import matching.
    pub fn binding_errors(&self) -> &[BindingError] {
        &self.binding_errors
    }

    /// Set binding errors (used by algorithms).
    pub(crate) fn set_binding_errors(&mut self, errors: Vec<BindingError>) {
        self.binding_errors = errors;
    }

    /// Set execution order (used by `ExecOrderResult::apply`).
    pub(crate) fn set_exec_order(&mut self, order: Vec<ModuleIdx>) {
        self.exec_order = order;
    }

    /// Set detected cycles (used by `ExecOrderResult::apply`).
    pub(crate) fn set_cycles(&mut self, cycles: Vec<Vec<ModuleIdx>>) {
        self.cycles = cycles;
    }
}
