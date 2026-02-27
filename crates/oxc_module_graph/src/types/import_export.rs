use std::fmt::Debug;
use std::hash::Hash;

use compact_str::CompactString;
use oxc_span::Span;

use super::{ModuleIdx, SymbolRef};

oxc_index::define_index_type! {
    /// Index into a module's import records.
    pub struct ImportRecordIdx = u32;
}

/// The kind of import statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportKind {
    /// `import ... from '...'` or `import '...'`
    Static,
    /// `import('...')`
    Dynamic,
    /// `require('...')`
    Require,
}

/// A named import consumed by a module.
///
/// Represents `import { foo as bar } from './mod'` where:
/// - `imported_name` = "foo"
/// - `local_symbol` = symbol for "bar"
#[derive(Debug, Clone)]
pub struct NamedImport {
    /// The name under which the binding is exported by the target module.
    /// "default" for default imports, "*" for namespace imports.
    pub imported_name: CompactString,
    /// The local symbol that binds the imported value.
    pub local_symbol: SymbolRef,
    /// Index into the module's import records.
    pub record_idx: ImportRecordIdx,
    /// Whether this is a `type` import.
    pub is_type: bool,
}

/// A locally declared export.
///
/// Represents `export { foo }` or `export const foo = ...`
#[derive(Debug, Clone)]
pub struct LocalExport {
    /// The name this binding is exported as.
    pub exported_name: CompactString,
    /// The local symbol being exported.
    pub local_symbol: SymbolRef,
}

/// A resolved export: the final symbol that an export name maps to.
///
/// Generic over the symbol reference type so both oxc's `SymbolRef`
/// and Rolldown's `SymbolRef` can be used.
#[derive(Debug, Clone)]
pub struct ResolvedExport<S: Copy + Eq + Hash + Debug = SymbolRef> {
    /// The symbol that this export resolves to.
    pub symbol_ref: S,
    /// If this export is potentially ambiguous (multiple `export *` provide it),
    /// these are the other candidate symbols.
    pub potentially_ambiguous: Option<Vec<S>>,
}

/// An import record after module resolution.
#[derive(Debug, Clone)]
pub struct ResolvedImportRecord {
    /// The specifier string (e.g., "./foo", "react").
    pub specifier: CompactString,
    /// The resolved target module index, if resolution succeeded.
    pub resolved_module: Option<ModuleIdx>,
    /// The kind of import.
    pub kind: ImportKind,
}

/// An edge in the module dependency graph.
#[derive(Debug, Clone)]
pub struct ImportEdge {
    /// The specifier string.
    pub specifier: CompactString,
    /// The target module index.
    pub target: ModuleIdx,
    /// Whether this is a type-only import.
    pub is_type: bool,
}

/// A star export entry: `export * from './foo'`
#[derive(Debug, Clone)]
pub struct StarExportEntry {
    /// The specifier of the module being re-exported.
    pub module_request: CompactString,
    /// The resolved module index, if available.
    pub resolved_module: Option<ModuleIdx>,
    /// Span of the export statement.
    pub span: Span,
}

/// An indirect export entry: `export { x } from './foo'` or `export * as ns from './foo'`
#[derive(Debug, Clone)]
pub struct IndirectExportEntry {
    /// The name as exported from this module.
    pub exported_name: CompactString,
    /// The name as imported from the target module.
    pub imported_name: CompactString,
    /// The specifier of the target module.
    pub module_request: CompactString,
    /// The resolved module index, if available.
    pub resolved_module: Option<ModuleIdx>,
    /// Span of the export statement.
    pub span: Span,
}

/// Result of matching an import to an export.
#[derive(Debug, Clone)]
pub enum MatchImportKind<S: Copy + Eq + Hash + Debug = SymbolRef> {
    /// Successfully resolved to a single symbol.
    Normal { symbol_ref: S },
    /// Resolved to a namespace object.
    Namespace { namespace_ref: S },
    /// Ambiguous: multiple `export *` provide the same name.
    Ambiguous { candidates: Vec<S> },
    /// Circular import detected during resolution.
    Cycle,
    /// No matching export found.
    NoMatch,
}
