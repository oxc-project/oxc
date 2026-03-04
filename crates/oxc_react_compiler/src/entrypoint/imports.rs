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
};

/// Tracks imports that need to be added to the program.
#[derive(Debug, Default)]
pub struct ProgramContext {
    /// Map from module source to the set of import specifiers needed.
    pub imports: FxHashMap<String, Vec<ImportSpecifier>>,
    /// Whether any function was compiled (used for checking if imports are needed).
    pub has_compiled_function: bool,
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
