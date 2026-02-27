use std::path::PathBuf;

use compact_str::CompactString;

use super::ModuleIdx;

/// Errors that can occur during module graph construction or import binding.
#[derive(Debug)]
pub enum ModuleGraphError {
    /// Failed to resolve a module specifier to a file path.
    ResolutionFailed { specifier: CompactString, importer: PathBuf, reason: String },
    /// Failed to parse a module.
    ParseError { path: PathBuf, reason: String },
    /// An import could not be matched to any export.
    UnresolvedImport { module: ModuleIdx, import_name: CompactString, specifier: CompactString },
    /// An import is ambiguous (multiple `export *` provide the same name).
    AmbiguousImport { module: ModuleIdx, import_name: CompactString },
    /// Circular dependency detected during import binding.
    CircularDependency { chain: Vec<ModuleIdx> },
}
