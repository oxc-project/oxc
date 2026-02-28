use oxc_index::IndexVec;

use crate::algo::BindingError;
use crate::default::SymbolRefDb;
use crate::module::{ExternalModule, Module, NormalModule};
use crate::types::{ModuleIdx, SymbolRef};

/// A concrete, batteries-included module graph.
///
/// Owns both the module storage and the symbol database.
/// Algorithms operate directly on `&ModuleGraph` / `&mut ModuleGraph`
/// instead of through trait bounds.
#[derive(Debug, Default)]
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
            self.modules[idx] = Module::Normal(module);
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
            self.modules.push(Module::Normal(module));
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
    pub fn add_symbol(&mut self, module: ModuleIdx, name: String) -> SymbolRef {
        self.symbols.add_symbol(module, name)
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

    /// Follow link chains to find the canonical (final) symbol.
    pub fn canonical_ref(&self, sym: SymbolRef) -> SymbolRef {
        self.symbols.canonical_ref_for(sym)
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
    pub fn link(&mut self, config: &crate::hooks::LinkConfig) {
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

        // 3. Dynamic exports
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
}
