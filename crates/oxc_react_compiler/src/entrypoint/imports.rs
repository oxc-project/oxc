/// Import management for the compiler output.
///
/// Port of `Entrypoint/Imports.ts` from the React Compiler.
///
/// Manages imports that the compiler needs to add to the output code,
/// such as the React compiler runtime (`c` function for cache management).
/// Also provides `validateRestrictedImports` for checking blocklisted module imports.
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::Statement;

use crate::{
    compiler_error::{
        CompilerError, CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory,
        SourceLocation,
    },
    entrypoint::options::CompilerReactTarget,
    hir::environment::ExternalFunction,
};

/// Tracks imports that need to be added to the program.
#[derive(Debug, Default)]
pub struct ProgramContext {
    /// Map from module source to the set of import specifiers needed.
    pub imports: FxHashMap<String, Vec<ImportSpecifier>>,
    /// Whether any function was compiled (used for checking if imports are needed).
    pub has_compiled_function: bool,
    /// Set of known referenced names, used to avoid collisions in `new_uid`.
    pub known_referenced_names: FxHashSet<String>,
}

/// A single import specifier.
#[derive(Debug, Clone)]
pub struct ImportSpecifier {
    pub local: String,
    pub imported: String,
}

impl ProgramContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an import for the React compiler runtime cache function.
    pub fn add_cache_import(&mut self, target: &CompilerReactTarget) {
        let (source, specifier) = match target {
            CompilerReactTarget::React17 | CompilerReactTarget::React18 => {
                ("react-compiler-runtime".to_string(), "c".to_string())
            }
            CompilerReactTarget::React19 => ("react/compiler-runtime".to_string(), "c".to_string()),
            CompilerReactTarget::MetaInternal { runtime_module } => {
                (runtime_module.clone(), "c".to_string())
            }
        };

        let specifiers = self.imports.entry(source).or_default();
        if !specifiers.iter().any(|s| s.imported == specifier) {
            specifiers
                .push(ImportSpecifier { local: format!("_{specifier}"), imported: specifier });
        }
    }

    /// Check if any imports have been registered.
    pub fn has_imports(&self) -> bool {
        !self.imports.is_empty()
    }

    /// Generate a unique identifier name, avoiding collisions with known references.
    ///
    /// Port of `ProgramContext.newUid` from `Entrypoint/Imports.ts`.
    ///
    /// Mirrors the upstream behaviour:
    /// * If `name` looks like a hook (`useFoo`, `use1`, etc.), keep the name and
    ///   only append `_0`, `_1`, ... suffixes on collision (so the hook-naming
    ///   convention is preserved for type inference).
    /// * Otherwise, return the unprefixed name when free; on collision fall back
    ///   to Babel's `scope.generateUid(name)` form: `_name`, `_name2`, `_name3`,
    ///   ... (the same algorithm Babel uses, matching upstream import aliases
    ///   like `_makeReadOnly`).
    pub fn new_uid(&mut self, name: &str) -> String {
        if crate::hir::environment::is_hook_name(name) {
            let mut uid = name.to_string();
            let mut i = 0;
            while self.known_referenced_names.contains(&uid) {
                uid = format!("{name}_{i}");
                i += 1;
            }
            self.known_referenced_names.insert(uid.clone());
            return uid;
        }
        if !self.known_referenced_names.contains(name) {
            self.known_referenced_names.insert(name.to_string());
            return name.to_string();
        }
        // Babel's `scope.generateUid` collision path: `_name`, `_name2`, `_name3`, ...
        let mut i = 1u32;
        loop {
            let uid = if i == 1 { format!("_{name}") } else { format!("_{name}{i}") };
            if !self.known_referenced_names.contains(&uid) {
                self.known_referenced_names.insert(uid.clone());
                return uid;
            }
            i += 1;
        }
    }

    /// Add an import specifier for an external function.
    ///
    /// Port of `ProgramContext.addImportSpecifier` from `Entrypoint/Imports.ts`.
    ///
    /// If the same module+specifier pair has already been added, returns the
    /// existing local name. Otherwise, generates a unique local name and
    /// registers the import.
    pub fn add_import_specifier(&mut self, ext_fn: &ExternalFunction) -> String {
        // Check if this module+specifier combination already exists
        if let Some(specifiers) = self.imports.get(&ext_fn.source)
            && let Some(existing) =
                specifiers.iter().find(|s| s.imported == ext_fn.import_specifier_name)
        {
            return existing.local.clone();
        }

        let local = self.new_uid(&ext_fn.import_specifier_name);
        let specifiers = self.imports.entry(ext_fn.source.clone()).or_default();
        specifiers.push(ImportSpecifier {
            local: local.clone(),
            imported: ext_fn.import_specifier_name.clone(),
        });
        local
    }

    /// Record a name as already referenced, so `new_uid` will avoid it.
    pub fn add_reference(&mut self, name: &str) {
        self.known_referenced_names.insert(name.to_string());
    }
}

/// Assert that a global identifier name is NOT shadowed by a local binding
/// in the provided scope (or the program scope).
///
/// Port of `ProgramContext.assertGlobalBinding` from `Entrypoint/Imports.ts`.
///
/// Upstream consults Babel's scope to check `hasReference` / `hasBinding`.
/// We don't have Babel scope information here, so callers pass the relevant
/// set of locally-bound names (e.g. the `unique_identifiers` set produced by
/// `rename_variables`). If `name` is present, returns a `Todo` error matching
/// the upstream error shape.
///
/// # Errors
/// Returns a `Todo` `CompilerError` if `local_bindings` contains `name`.
#[expect(clippy::implicit_hasher)]
pub fn assert_global_binding(
    name: &str,
    local_bindings: &FxHashSet<String>,
    loc: Option<SourceLocation>,
) -> Result<(), CompilerError> {
    if !local_bindings.contains(name) {
        return Ok(());
    }
    let mut error = CompilerError::new();
    error.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
        category: ErrorCategory::Todo,
        reason: "Encountered conflicting global in generated program".to_string(),
        description: Some(format!("Conflict from local binding {name}")),
        loc,
        suggestions: None,
    }));
    Err(error)
}

/// Validate that the program does not import from blocklisted modules.
///
/// Port of `validateRestrictedImports` from `Entrypoint/Imports.ts` lines 21-47.
///
/// If `validate_blocklisted_imports` is `None` or empty, returns `None` (no error).
/// Otherwise, checks all `ImportDeclaration` nodes in the program body against the
/// blocklist and returns a `CompilerError` with category `Todo` for each match.
pub fn validate_restricted_imports(
    body: &[Statement<'_>],
    validate_blocklisted_imports: Option<&[String]>,
) -> Option<CompilerError> {
    let blocklist = match validate_blocklisted_imports {
        Some(list) if !list.is_empty() => list,
        _ => return None,
    };

    let restricted: FxHashSet<&str> = blocklist.iter().map(String::as_str).collect();
    let mut error = CompilerError::new();

    for stmt in body {
        if let Statement::ImportDeclaration(import_decl) = stmt {
            let module_name = import_decl.source.value.as_str();
            if restricted.contains(module_name) {
                error.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
                    category: ErrorCategory::Todo,
                    reason: "Bailing out due to blocklisted import".to_string(),
                    description: Some(format!("Import from module {module_name}")),
                    loc: Some(SourceLocation::Source(import_decl.span)),
                    suggestions: None,
                }));
            }
        }
    }

    if error.has_any_errors() { Some(error) } else { None }
}
