/// Default module type provider.
///
/// Port of `HIR/DefaultModuleTypeProvider.ts` from the React Compiler.
///
/// Provides default type information for known npm modules and their exports.
/// This allows the compiler to infer types for commonly used libraries
/// without requiring explicit type annotations.
use crate::hir::types::Type;

/// A module type provider that can return type information for module exports.
pub trait ModuleTypeProvider {
    /// Get the type for a specific export from a module.
    fn get_type(&self, module: &str, export: &str) -> Option<Type>;
}

/// The default module type provider — returns None for all lookups.
/// Custom providers can be configured via EnvironmentConfig.
pub struct DefaultModuleTypeProvider;

impl ModuleTypeProvider for DefaultModuleTypeProvider {
    fn get_type(&self, _module: &str, _export: &str) -> Option<Type> {
        // The full implementation would have built-in type info for:
        // - react (useState, useRef, useEffect, etc.)
        // - react-dom
        // - Other commonly used libraries
        // These are currently handled via the Globals registry instead.
        None
    }
}
