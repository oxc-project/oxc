/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
use rustc_hash::{FxHashMap, FxHashSet};

use crate::react_compiler_diagnostics::{CompilerError, CompilerErrorDetail, ErrorCategory};
use crate::react_compiler_hir::environment::BindingRename;
use crate::scope::ScopeInfo;

use oxc_diagnostics::Diagnostics;

use super::compile_result::{DebugLogEntry, OrderedLogItem};
use super::plugin_options::{CompilerTarget, PluginOptions};
use super::suppression::SuppressionRange;

/// An import specifier tracked by ProgramContext.
/// Corresponds to NonLocalImportSpecifier in the TS compiler.
#[derive(Debug, Clone)]
pub struct NonLocalImportSpecifier {
    pub name: String,
    pub module: String,
    pub imported: String,
}

/// Context for the program being compiled.
/// Tracks compiled functions, generated names, and import requirements.
/// Equivalent to ProgramContext class in Imports.ts.
pub struct ProgramContext {
    pub opts: PluginOptions,
    pub code: Option<String>,
    pub react_runtime_module: String,
    pub suppressions: Vec<SuppressionRange>,
    pub has_module_scope_opt_out: bool,
    /// Diagnostics (errors/warnings) accumulated during compilation. Fatality is
    /// decided separately by `panicThreshold`.
    pub diagnostics: Diagnostics,
    /// Debug-log entries (HIR dumps) emitted when the `debug` feature is enabled.
    pub ordered_log: Vec<OrderedLogItem>,

    // Pre-resolved import local names for codegen
    pub instrument_fn_name: Option<String>,
    pub instrument_gating_name: Option<String>,
    pub hook_guard_name: Option<String>,

    // Variable renames from lowering, to be applied back to the Babel AST
    pub renames: Vec<BindingRename>,

    /// Whether debug logging is enabled (HIR formatting after each pass).
    pub debug_enabled: bool,

    // Internal state
    already_compiled: FxHashSet<u32>,
    known_referenced_names: FxHashSet<String>,
    imports: FxHashMap<String, FxHashMap<String, NonLocalImportSpecifier>>,
}

impl ProgramContext {
    pub fn new(
        opts: PluginOptions,
        code: Option<String>,
        suppressions: Vec<SuppressionRange>,
        has_module_scope_opt_out: bool,
    ) -> Self {
        let react_runtime_module = get_react_compiler_runtime_module(&opts.target);
        let debug_enabled = opts.debug;
        Self {
            opts,
            code,
            react_runtime_module,
            suppressions,
            has_module_scope_opt_out,
            diagnostics: Diagnostics::new(),
            ordered_log: Vec::new(),
            instrument_fn_name: None,
            instrument_gating_name: None,
            hook_guard_name: None,
            renames: Vec::new(),
            debug_enabled,
            already_compiled: FxHashSet::default(),
            known_referenced_names: FxHashSet::default(),
            imports: FxHashMap::default(),
        }
    }

    /// Check if a function at the given start position has already been compiled.
    /// This is a workaround for Babel not consistently respecting skip().
    pub fn is_already_compiled(&self, start: u32) -> bool {
        self.already_compiled.contains(&start)
    }

    /// Mark a function at the given start position as compiled.
    pub fn mark_compiled(&mut self, start: u32) {
        self.already_compiled.insert(start);
    }

    /// Initialize known referenced names from scope bindings.
    /// Call this after construction to seed conflict detection with program scope bindings.
    pub fn init_from_scope(&mut self, scope: &ScopeInfo) {
        // Register ALL bindings (not just program-scope) so that UID generation
        // avoids name conflicts with any binding in the file. This matches
        // Babel's generateUid() which checks all scopes.
        for binding in &scope.bindings {
            self.known_referenced_names.insert(binding.name.clone());
        }
    }

    /// Check if a name conflicts with known references.
    pub fn has_reference(&self, name: &str) -> bool {
        self.known_referenced_names.contains(name)
    }

    /// Generate a unique identifier name that doesn't conflict with existing bindings.
    ///
    /// For hook names (use*), preserves the original name to avoid breaking
    /// hook-name-based type inference. For other names, prefixes with underscore
    /// similar to Babel's generateUid.
    pub fn new_uid(&mut self, name: &str) -> String {
        if is_hook_name(name) {
            // Don't prefix hooks with underscore, since InferTypes might
            // type HookKind based on callee naming convention.
            let mut uid = name.to_string();
            let mut i = 0;
            while self.has_reference(&uid) {
                uid = format!("{}_{}", name, i);
                i += 1;
            }
            self.known_referenced_names.insert(uid.clone());
            uid
        } else if !self.has_reference(name) {
            self.known_referenced_names.insert(name.to_string());
            name.to_string()
        } else {
            // Generate unique name with underscore prefix (similar to Babel's generateUid).
            // Babel strips leading underscores before prefixing, so:
            //   generateUid("_c") → strips to "c" → generates "_c", "_c2", "_c3", ...
            //   generateUid("foo") → generates "_foo", "_foo2", "_foo3", ...
            let base = name.trim_start_matches('_');
            let mut uid = format!("_{}", base);
            let mut i = 2;
            while self.has_reference(&uid) {
                uid = format!("_{}{}", base, i);
                i += 1;
            }
            self.known_referenced_names.insert(uid.clone());
            uid
        }
    }

    /// Add the memo cache import (the `c` function from the compiler runtime).
    pub fn add_memo_cache_import(&mut self) -> NonLocalImportSpecifier {
        let module = self.react_runtime_module.clone();
        self.add_import_specifier(&module, "c", Some("_c"))
    }

    /// Add an import specifier, reusing an existing one if it was already added.
    ///
    /// If `name_hint` is provided, it will be used as the basis for the local
    /// name; otherwise `specifier` is used.
    pub fn add_import_specifier(
        &mut self,
        module: &str,
        specifier: &str,
        name_hint: Option<&str>,
    ) -> NonLocalImportSpecifier {
        // Check if already imported
        if let Some(module_imports) = self.imports.get(module) {
            if let Some(existing) = module_imports.get(specifier) {
                return existing.clone();
            }
        }

        let name = self.new_uid(name_hint.unwrap_or(specifier));
        let binding = NonLocalImportSpecifier {
            name,
            module: module.to_string(),
            imported: specifier.to_string(),
        };

        self.imports
            .entry(module.to_string())
            .or_default()
            .insert(specifier.to_string(), binding.clone());

        binding
    }

    /// Register a name as referenced so future uid generation avoids it.
    pub fn add_new_reference(&mut self, name: String) {
        self.known_referenced_names.insert(name);
    }

    /// Get the set of known referenced names for seeding per-function Environment UID generation.
    pub fn known_referenced_names(&self) -> &FxHashSet<String> {
        &self.known_referenced_names
    }

    /// Merge UID names generated during a function compilation back into the program context,
    /// so subsequent function compilations avoid collisions.
    pub fn merge_uid_known_names(&mut self, names: &FxHashSet<String>) {
        self.known_referenced_names.extend(names.iter().cloned());
    }

    /// Log a debug entry (for debugLogIRs support).
    pub fn log_debug(&mut self, entry: DebugLogEntry) {
        self.ordered_log.push(OrderedLogItem::Debug { entry });
    }

    /// Check if there are any pending imports to add to the program.
    pub fn has_pending_imports(&self) -> bool {
        !self.imports.is_empty()
    }

    /// Get an immutable view of the generated imports.
    pub fn imports(&self) -> &FxHashMap<String, FxHashMap<String, NonLocalImportSpecifier>> {
        &self.imports
    }
}

/// Check for blocklisted import modules.
/// Returns a CompilerError if any blocklisted imports are found.
pub fn validate_restricted_imports(
    program: &oxc_ast::ast::Program,
    blocklisted: &Option<Vec<String>>,
) -> Option<CompilerError> {
    let blocklisted = match blocklisted {
        Some(b) if !b.is_empty() => b,
        _ => return None,
    };
    let restricted: FxHashSet<&str> = blocklisted.iter().map(|s| s.as_str()).collect();
    let mut error = CompilerError::new();

    for stmt in &program.body {
        if let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt {
            if restricted.contains(import.source.value.as_str()) {
                let detail = CompilerErrorDetail::new(
                    ErrorCategory::Todo,
                    "Bailing out due to blocklisted import",
                )
                .with_description(format!("Import from module {}", import.source.value));
                error.push_error_detail(detail);
            }
        }
    }

    if error.has_any_errors() { Some(error) } else { None }
}

/// Check if a name follows the React hook naming convention (use[A-Z0-9]...).
fn is_hook_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 4
        && bytes[0] == b'u'
        && bytes[1] == b's'
        && bytes[2] == b'e'
        && bytes.get(3).map_or(false, |c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// Get the runtime module name based on the compiler target.
pub fn get_react_compiler_runtime_module(target: &CompilerTarget) -> String {
    match target {
        CompilerTarget::Version(v) if v == "19" => "react/compiler-runtime".to_string(),
        CompilerTarget::Version(v) if v == "17" || v == "18" => {
            "react-compiler-runtime".to_string()
        }
        CompilerTarget::MetaInternal { runtime_module, .. } => runtime_module.clone(),
        // Default to React 19 runtime for unrecognized versions
        CompilerTarget::Version(_) => "react/compiler-runtime".to_string(),
    }
}
