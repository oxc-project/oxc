//! Shared trait and types for cross-file type checking coordination.
//!
//! This crate defines `CheckerHost` (the interface between per-file checkers
//! and the project-level coordinator) and `IntrinsicIds` (shared primitive
//! type IDs). It exists as a separate crate so both `oxc_checker` and
//! `oxc_project` can depend on the same trait without circular dependencies.

use oxc_span::CompactStr;
use oxc_syntax::symbol::SymbolId;
use oxc_types::TypeId;

/// Exported binding with separate type-side and value-side types.
///
/// A name can be a type (interface, type alias), a value (variable, function),
/// or both (class, enum, merged interface+var). For example, `RegExp` in
/// lib.d.ts has both `interface RegExp` (type-side, instance shape) and
/// `declare var RegExp: RegExpConstructor` (value-side, constructor).
#[derive(Clone, Copy, Default, Debug)]
pub struct ExportedBinding {
    /// Type namespace: interface, type alias, class instance type, enum union.
    pub type_type: Option<TypeId>,
    /// Value namespace: var annotation, function type, class constructor, enum namespace.
    pub value_type: Option<TypeId>,
}

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
    /// Widening null type — same TypeFlags/name as `null_type` but with
    /// `ObjectFlags::ContainsWideningType`. Used for null literal expressions
    /// when `strictNullChecks` is false; `getWidenedType` maps it to `any`.
    pub null_widening_type: TypeId,
    /// Widening undefined type — same TypeFlags/name as `undefined_type` but
    /// with `ObjectFlags::ContainsWideningType`. Used for undefined/void
    /// expressions when `strictNullChecks` is false.
    pub undefined_widening_type: TypeId,
}

/// Compiler options that affect type-checking behavior.
///
/// Mirrors a subset of TypeScript's `compilerOptions`. All fields have
/// tsc-compatible defaults (non-strict mode). Options from the `strict`
/// family are stored as `Option<bool>`; the checker resolves them against
/// `strict` during construction.
#[derive(Clone, Copy, Debug)]
pub struct CheckerOptions {
    /// Master strict mode flag. When `true`, unset strict-family options
    /// (`strict_null_checks`, `no_implicit_any`, etc.) default to `true`.
    pub strict: bool,

    // -- Strict sub-options: `None` = inherit from `strict` --
    /// Enable strict null checks (affects type narrowing, assignability).
    pub strict_null_checks: Option<bool>,
    /// Enable strict property initialization checks (TS2564).
    pub strict_property_initialization: Option<bool>,
    /// Enable strict function type checks (contravariant parameters).
    pub strict_function_types: Option<bool>,
    /// Report errors for expressions/statements with an implied `any` type.
    pub no_implicit_any: Option<bool>,
    /// Report errors when `this` has an implied `any` type.
    pub no_implicit_this: Option<bool>,

    // -- Tristate options --
    // These have three distinct behaviors in tsc:
    //   `Some(true)` = suppress the diagnostic entirely
    //   `Some(false)` = report as an error
    //   `None` (unset) = report as a suggestion (warning)
    /// Controls unreachable code diagnostics (e.g., TS2695).
    pub allow_unreachable_code: Option<bool>,
    /// Controls unused label diagnostics (e.g., TS1328).
    pub allow_unused_labels: Option<bool>,

    // -- Plain bool options --
    /// Report errors for fallthrough cases in switch statements (TS7029).
    pub no_fallthrough_cases_in_switch: bool,
    /// Report errors when a function doesn't explicitly return in all code paths.
    pub no_implicit_returns: bool,
}

impl Default for CheckerOptions {
    fn default() -> Self {
        Self {
            strict: false,
            strict_null_checks: None,
            strict_property_initialization: None,
            strict_function_types: None,
            no_implicit_any: None,
            no_implicit_this: None,
            allow_unreachable_code: None,
            allow_unused_labels: None,
            no_fallthrough_cases_in_switch: false,
            no_implicit_returns: false,
        }
    }
}

/// Interface for cross-file type resolution.
///
/// Implemented by `Project` (in `oxc_project`), consumed by `Checker`
/// (in `oxc_checker`). Mirrors tsgo's `Program` interface.
pub trait CheckerHost {
    /// Get the shared intrinsic type IDs (any, string, number, etc.).
    fn get_intrinsics(&self) -> IntrinsicIds;

    /// Get a global type by name (type-side, e.g., "Array", "Promise", "String").
    fn get_global_type(&self, name: &str) -> Option<TypeId>;

    /// Get a global value type by name (value-side, e.g., "RegExp" → RegExpConstructor).
    /// Used for resolving unresolved identifiers in expression position.
    fn get_global_value_type(&self, name: &str) -> Option<TypeId> {
        let _ = name;
        None
    }

    /// Resolve a named import from a module specifier.
    /// Returns both type-side and value-side types for the exported name.
    fn resolve_import(
        &self,
        from_file: &str,
        module_specifier: &str,
        export_name: &str,
    ) -> Option<ExportedBinding>;

    /// Look up a type parameter constraint resolved in another file.
    fn get_type_param_constraint(&self, type_id: TypeId) -> Option<TypeId>;

    /// Get the display name for a symbol in a specific file.
    /// The `file_idx` identifies which file's Semantic to look up,
    /// and `symbol_id` indexes into that file's symbol table.
    fn get_symbol_name(&self, file_idx: u16, symbol_id: SymbolId) -> Option<CompactStr>;
}
