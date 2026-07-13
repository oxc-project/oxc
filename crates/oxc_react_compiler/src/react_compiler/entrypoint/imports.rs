/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
use std::borrow::Cow;

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, ModuleExportName, Program, Statement};
use oxc_str::{Ident, IdentHashSet, format_ident};

use crate::diagnostics::ErrorCategory;
use crate::scope::ScopeResolver;

use oxc_diagnostics::Diagnostics;

use super::suppression::SuppressionRange;
use crate::options::{CompilerTarget, PluginOptions};

/// An import specifier tracked by ProgramContext.
/// Corresponds to NonLocalImportSpecifier in the TS compiler.
#[derive(Debug, Clone, Copy)]
pub struct NonLocalImportSpecifier<'a> {
    pub name: Ident<'a>,
    pub imported: Ident<'a>,
}

/// Context for the program being compiled.
/// Tracks compiled functions, generated names, and import requirements.
/// Equivalent to ProgramContext class in Imports.ts.
pub struct ProgramContext<'a> {
    allocator: &'a Allocator,
    pub source_text: &'a str,
    pub opts: PluginOptions,
    pub react_runtime_module: Cow<'static, str>,
    pub suppressions: Vec<SuppressionRange>,
    pub has_module_scope_opt_out: bool,
    /// Diagnostics (errors/warnings) accumulated during compilation. Fatality is
    /// decided separately by `panicThreshold`.
    pub diagnostics: Diagnostics,
    // Pre-resolved import local names for codegen
    pub instrument_fn_name: Option<Ident<'a>>,
    pub instrument_gating_name: Option<Ident<'a>>,
    pub hook_guard_name: Option<Ident<'a>>,
    pub memo_cache_name: Option<Ident<'a>>,

    // Internal state
    known_referenced_names: IdentHashSet<'a>,
    imports: FxHashMap<String, FxHashMap<String, NonLocalImportSpecifier<'a>>>,
}

impl<'a> ProgramContext<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        opts: PluginOptions,
        suppressions: Vec<SuppressionRange>,
        has_module_scope_opt_out: bool,
    ) -> Self {
        let react_runtime_module = get_react_compiler_runtime_module(&opts.target);
        Self {
            allocator,
            source_text,
            opts,
            react_runtime_module,
            suppressions,
            has_module_scope_opt_out,
            diagnostics: Diagnostics::new(),
            instrument_fn_name: None,
            instrument_gating_name: None,
            hook_guard_name: None,
            memo_cache_name: None,
            known_referenced_names: IdentHashSet::default(),
            imports: FxHashMap::default(),
        }
    }

    /// Initialize known referenced names from scope bindings.
    /// Call this after construction to seed conflict detection with program scope bindings.
    pub fn init_from_scope(&mut self, scope: &ScopeResolver<'_, 'a>) {
        // Register ALL bindings (not just program-scope) so that UID generation
        // avoids name conflicts with any binding in the file. This matches
        // Babel's generateUid() which checks all scopes.
        for symbol_id in scope.symbols() {
            self.known_referenced_names.insert(scope.symbol_ident(symbol_id));
        }
    }

    /// The arena the compiled nodes were allocated in.
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
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
    pub fn new_uid(&mut self, name: &str) -> Ident<'a> {
        if is_hook_name(name) {
            // Don't prefix hooks with underscore, since InferTypes might
            // type HookKind based on callee naming convention.
            let mut uid = Ident::from_str_in(name, &self.allocator);
            let mut i = 0;
            while self.has_reference(&uid) {
                uid = format_ident!(self.allocator, "{name}_{i}");
                i += 1;
            }
            self.known_referenced_names.insert(uid);
            uid
        } else if !self.has_reference(name) {
            let uid = Ident::from_str_in(name, &self.allocator);
            self.known_referenced_names.insert(uid);
            uid
        } else {
            // Generate unique name with underscore prefix (similar to Babel's generateUid).
            // Babel strips leading underscores before prefixing, so:
            //   generateUid("_c") → strips to "c" → generates "_c", "_c2", "_c3", ...
            //   generateUid("foo") → generates "_foo", "_foo2", "_foo3", ...
            let base = name.trim_start_matches('_');
            let mut uid = format_ident!(self.allocator, "_{base}");
            let mut i = 2;
            while self.has_reference(&uid) {
                uid = format_ident!(self.allocator, "_{base}{i}");
                i += 1;
            }
            self.known_referenced_names.insert(uid);
            uid
        }
    }

    /// Reserve the memo cache import's local name (`_c`, `_c2`, ...) before compilation
    /// so codegen can emit it directly. Also avoids names only referenced as globals
    /// (Babel's `generateUid` checks `hasGlobal`); `known_referenced_names` cannot see
    /// globals in functions that have not compiled yet. Mirrors `new_uid("_c")`.
    pub fn reserve_memo_cache_name(&mut self, scope: &ScopeResolver<'_, 'a>) {
        let mut name = Ident::from("_c");
        let mut i = 2;
        while self.has_reference(&name) || scope.has_unresolved_reference(&name) {
            name = format_ident!(self.allocator, "_c{i}");
            i += 1;
        }
        self.known_referenced_names.insert(name);
        self.memo_cache_name = Some(name);
    }

    /// Register the memo cache import (the `c` function from the compiler runtime) under
    /// the local name reserved by [`Self::reserve_memo_cache_name`].
    pub fn add_memo_cache_import(&mut self) {
        let name = self.memo_cache_name.expect("memo cache name reserved in compile_program");
        let binding = NonLocalImportSpecifier { name, imported: Ident::from("c") };
        self.imports
            .entry(self.react_runtime_module.to_string())
            .or_default()
            .insert("c".to_string(), binding);
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
    ) -> NonLocalImportSpecifier<'a> {
        // Check if already imported
        if let Some(module_imports) = self.imports.get(module) {
            if let Some(existing) = module_imports.get(specifier) {
                return *existing;
            }
        }

        let name = self.new_uid(name_hint.unwrap_or(specifier));
        let binding = NonLocalImportSpecifier {
            name,
            imported: Ident::from_str_in(specifier, &self.allocator),
        };

        self.imports.entry(module.to_string()).or_default().insert(specifier.to_string(), binding);

        binding
    }

    /// Register a name as referenced so future uid generation avoids it.
    pub fn add_new_reference(&mut self, name: Ident<'a>) {
        self.known_referenced_names.insert(name);
    }

    /// Get the set of known referenced names for seeding per-function Environment UID generation.
    pub fn known_referenced_names(&self) -> &IdentHashSet<'a> {
        &self.known_referenced_names
    }

    /// Merge UID names generated during a function compilation back into the program context,
    /// so subsequent function compilations avoid collisions.
    pub fn merge_uid_known_names(&mut self, names: &IdentHashSet<'a>) {
        self.known_referenced_names.extend(names.iter().copied());
    }

    /// Check if there are any pending imports to add to the program.
    pub fn has_pending_imports(&self) -> bool {
        !self.imports.is_empty()
    }

    /// Get an immutable view of the generated imports.
    pub fn imports(&self) -> &FxHashMap<String, FxHashMap<String, NonLocalImportSpecifier<'a>>> {
        &self.imports
    }
}

/// Check for blocklisted import modules.
/// Returns diagnostics if any blocklisted imports are found.
pub fn validate_restricted_imports(
    program: &Program,
    blocklisted: &Option<Vec<String>>,
) -> Option<Diagnostics> {
    let blocklisted = match blocklisted {
        Some(b) if !b.is_empty() => b,
        _ => return None,
    };
    let restricted: FxHashSet<&str> = blocklisted.iter().map(|s| s.as_str()).collect();
    let mut diagnostics = Diagnostics::new();

    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt {
            if restricted.contains(import.source.value.as_str()) {
                diagnostics.push(
                    ErrorCategory::Todo
                        .diagnostic("Bailing out due to blocklisted import")
                        .with_help(format!("Import from module {}", import.source.value)),
                );
            }
        }
    }

    if diagnostics.is_empty() { None } else { Some(diagnostics) }
}

/// Whether the program already imports the `c` memo-cache helper from `module_name`
/// — i.e. the file has already been compiled and must be skipped.
pub fn has_memo_cache_function_import(program: &Program, module_name: &str) -> bool {
    for stmt in &program.body {
        if let Statement::ImportDeclaration(import) = stmt
            && import.source.value == module_name
            && import.import_kind.is_value()
            && let Some(specifiers) = &import.specifiers
        {
            for specifier in specifiers {
                if let ImportDeclarationSpecifier::ImportSpecifier(data) = specifier
                    && data.import_kind.is_value()
                {
                    let imported_name = match &data.imported {
                        ModuleExportName::IdentifierName(id) => Some(id.name.as_str()),
                        ModuleExportName::IdentifierReference(id) => Some(id.name.as_str()),
                        ModuleExportName::StringLiteral(s) => Some(s.value.as_str()),
                    };
                    if imported_name == Some("c") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if a name follows the React hook naming convention (use[A-Z0-9]...).
fn is_hook_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 4
        && bytes[0] == b'u'
        && bytes[1] == b's'
        && bytes[2] == b'e'
        && bytes.get(3).is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// Get the runtime module name based on the compiler target.
pub fn get_react_compiler_runtime_module(target: &CompilerTarget) -> Cow<'static, str> {
    match target {
        CompilerTarget::Version(v) if v == "19" => Cow::Borrowed("react/compiler-runtime"),
        CompilerTarget::Version(v) if v == "17" || v == "18" => {
            Cow::Borrowed("react-compiler-runtime")
        }
        CompilerTarget::MetaInternal { runtime_module, .. } => Cow::Owned(runtime_module.clone()),
        // Default to React 19 runtime for unrecognized versions
        CompilerTarget::Version(_) => Cow::Borrowed("react/compiler-runtime"),
    }
}
