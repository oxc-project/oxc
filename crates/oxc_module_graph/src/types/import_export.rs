use std::fmt::Debug;
use std::hash::Hash;

use compact_str::CompactString;
use oxc_span::Span;

use super::{ModuleIdx, SymbolRef};

/// The module's export format, determined by usage analysis.
///
/// Initially `None` for modules that don't explicitly declare a format.
/// `determine_module_exports_kind` resolves `None` → `Esm` or `CommonJs`
/// based on how the module is imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportsKind {
    /// ES module format (has `import`/`export` syntax).
    Esm,
    /// CommonJS format (uses `module.exports` / `exports.x`).
    CommonJs,
    /// Not yet determined — resolved by `determine_module_exports_kind`.
    #[default]
    None,
}

impl ExportsKind {
    pub fn is_esm(self) -> bool {
        matches!(self, Self::Esm)
    }

    pub fn is_commonjs(self) -> bool {
        matches!(self, Self::CommonJs)
    }
}

/// Module wrapping strategy for CJS/ESM interop.
///
/// Determined by `determine_module_exports_kind` and `wrap_modules`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapKind {
    /// No wrapping needed.
    #[default]
    None,
    /// CommonJS wrapper: `function require_foo() { ... }`
    Cjs,
    /// ESM wrapper: `function init_foo() { ... }`
    Esm,
}

impl WrapKind {
    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

bitflags::bitflags! {
    /// Metadata flags for an import record.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct ImportRecordMeta: u16 {
        /// This is an `export * from '...'` re-export.
        const IS_EXPORT_STAR = 1 << 0;
        /// The import is inside a try-catch block.
        const IN_TRY_CATCH = 1 << 1;
        /// This is a top-level import (not nested in a function/block).
        const IS_TOP_LEVEL = 1 << 2;
        /// The import is a plain side-effect-only import (`import './foo'`).
        const IS_PLAIN_IMPORT = 1 << 3;
        /// Dynamic import whose exports are never used.
        const PURE_DYNAMIC_IMPORT = 1 << 4;
    }
}

/// A namespace property access alias.
///
/// Represents `namespace_ref.property_name` — used when an import resolves
/// to a namespace member access (e.g., CJS interop: `require_mod().foo`).
#[derive(Debug, Clone)]
pub struct NamespaceAlias {
    /// The property name being accessed on the namespace.
    pub property_name: CompactString,
    /// The namespace symbol being accessed.
    pub namespace_ref: SymbolRef,
}

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
    /// `import.meta.hot.accept('...')` — HMR-only, not a graph edge for
    /// execution order or side-effects propagation.
    HotAccept,
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
    /// Whether this export originated from CommonJS (`exports.foo = 1`).
    ///
    /// This affects star re-export semantics:
    /// - CJS "default" exports are **not** skipped during `export *` propagation
    /// - Ambiguity detection is suppressed when an existing export came from CJS
    pub came_from_cjs: bool,
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
    /// The namespace symbol for this import (e.g., the `*` binding).
    pub namespace_ref: SymbolRef,
    /// Metadata flags for this import record.
    pub meta: ImportRecordMeta,
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
    /// Resolved to a namespace object with a property access alias.
    ///
    /// Used for CJS interop and dynamic-export fallback: the import
    /// becomes `namespace_ref.alias` (e.g., `require("mod").foo`).
    NormalAndNamespace { namespace_ref: S, alias: CompactString },
    /// Ambiguous: multiple `export *` provide the same name.
    Ambiguous { candidates: Vec<S> },
    /// Circular import detected during resolution.
    Cycle,
    /// No matching export found.
    NoMatch,
}
