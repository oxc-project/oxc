/// Import management for the compiler output.
///
/// Port of `Entrypoint/Imports.ts` from the React Compiler.
///
/// Manages imports that the compiler needs to add to the output code,
/// such as the React compiler runtime (`c` function for cache management).
use rustc_hash::FxHashMap;

use crate::entrypoint::options::CompilerReactTarget;

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
            specifiers.push(ImportSpecifier { local: format!("_{specifier}"), imported: specifier });
        }
    }

    /// Check if any imports have been registered.
    pub fn has_imports(&self) -> bool {
        !self.imports.is_empty()
    }
}
