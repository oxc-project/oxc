use oxc_types::TypeId;

/// Interface for cross-file type resolution, defined in the checker
/// but implemented by the project/program layer.
///
/// Mirrors tsgo's `Program` interface in `checker/checker.go`.
///
/// The checker holds an `Option<&dyn CheckerHost>` and calls through it
/// for global type lookups and cross-file import resolution. When no host
/// is provided (standalone single-file checking), the checker falls back
/// to its own local state.
// NOTE: Eager implementation for v1. API designed so Project internals
// can move to lazy loading without changing this trait.
pub trait CheckerHost {
    /// Get a global type by name (e.g., "Array", "Promise", "String").
    ///
    /// Called when the checker encounters a type reference that can't be
    /// resolved via the current file's symbol table.
    fn get_global_type(&self, name: &str) -> Option<TypeId>;

    /// Resolve a named import from a module specifier.
    ///
    /// `from_file`: the file containing the import statement
    /// `module_specifier`: the string in `from '...'`
    /// `export_name`: the imported binding name
    ///
    /// Returns `None` if the module or export can't be resolved.
    fn resolve_import(
        &self,
        from_file: &str,
        module_specifier: &str,
        export_name: &str,
    ) -> Option<TypeId>;
}
