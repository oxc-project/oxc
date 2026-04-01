//! Shared trait and types for cross-file type checking coordination.
//!
//! This crate defines `CheckerHost` (the interface between per-file checkers
//! and the project-level coordinator) and `IntrinsicIds` (shared primitive
//! type IDs). It exists as a separate crate so both `oxc_checker` and
//! `oxc_project` can depend on the same trait without circular dependencies.

use oxc_span::CompactStr;
use oxc_syntax::symbol::SymbolId;
use oxc_types::TypeId;

/// Pre-allocated intrinsic type IDs.
///
/// All 14 primitive/intrinsic types plus true/false literal types are
/// allocated once in the arena. Both the Checker and Project share
/// these IDs, eliminating duplicate intrinsic types.
#[derive(Clone, Copy)]
pub struct IntrinsicIds {
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    pub number_type: TypeId,
    pub bigint_type: TypeId,
    pub boolean_type: TypeId,
    pub es_symbol_type: TypeId,
    pub void_type: TypeId,
    pub undefined_type: TypeId,
    pub null_type: TypeId,
    pub never_type: TypeId,
    pub non_primitive_type: TypeId,
    pub true_type: TypeId,
    pub false_type: TypeId,
}

/// Interface for cross-file type resolution.
///
/// Implemented by `Project` (in `oxc_project`), consumed by `Checker`
/// (in `oxc_checker`). Mirrors tsgo's `Program` interface.
pub trait CheckerHost {
    /// Get the shared intrinsic type IDs (any, string, number, etc.).
    fn get_intrinsics(&self) -> IntrinsicIds;

    /// Get a global type by name (e.g., "Array", "Promise", "String").
    fn get_global_type(&self, name: &str) -> Option<TypeId>;

    /// Resolve a named import from a module specifier.
    fn resolve_import(
        &self,
        from_file: &str,
        module_specifier: &str,
        export_name: &str,
    ) -> Option<TypeId>;

    /// Look up a type parameter constraint resolved in another file.
    fn get_type_param_constraint(&self, type_id: TypeId) -> Option<TypeId>;

    /// Get the display name for a symbol in a specific file.
    /// The `file_idx` identifies which file's Semantic to look up,
    /// and `symbol_id` indexes into that file's symbol table.
    fn get_symbol_name(&self, file_idx: u16, symbol_id: SymbolId) -> Option<CompactStr>;
}
