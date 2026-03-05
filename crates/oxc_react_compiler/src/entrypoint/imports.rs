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
    /// Appends incrementing numeric suffixes (`_0`, `_1`, ...) if the base name
    /// is already in use.
    pub fn new_uid(&mut self, name: &str) -> String {
        let mut uid = name.to_string();
        let mut i = 0;
        while self.known_referenced_names.contains(&uid) {
            uid = format!("{name}_{i}");
            i += 1;
        }
        self.known_referenced_names.insert(uid.clone());
        uid
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
