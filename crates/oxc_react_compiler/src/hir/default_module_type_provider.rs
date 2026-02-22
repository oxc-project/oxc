/// Default module type provider.
///
/// Port of `HIR/DefaultModuleTypeProvider.ts` from the React Compiler.
///
/// Provides default type information for known npm modules and their exports.
/// This allows the compiler to recognize APIs from known libraries that are
/// incompatible with memoization, so it can warn or bail out appropriately.
///
/// Libraries developed before the Rules of React were documented may implement
/// APIs with "interior mutability" that cannot be memoized safely. This module
/// teaches the compiler about specific APIs that are known to be incompatible.
use rustc_hash::FxHashMap;

use crate::hir::{
    hir_types::{Effect, ValueKind},
    type_schema::{BuiltInTypeName, ModuleTypeConfig},
};

/// A module type provider that can return type information for module exports.
pub trait ModuleTypeProvider {
    /// Get the type config for a module by its name.
    fn get_type(&self, module_name: &str) -> Option<ModuleTypeConfig>;
}

/// The default module type provider.
///
/// Returns type information for known npm modules whose APIs are incompatible
/// with memoization. The React team has reached out to the teams who own any
/// API listed here to ensure they are aware of the issue.
pub struct DefaultModuleTypeProvider;

impl ModuleTypeProvider for DefaultModuleTypeProvider {
    fn get_type(&self, module_name: &str) -> Option<ModuleTypeConfig> {
        default_module_type_provider(module_name)
    }
}

/// Returns type configuration for known npm modules, or `None` for unknown modules.
///
/// Currently provides type information for:
/// - `react-hook-form`: `useForm()` returns a `watch()` function that cannot be
///   memoized safely due to interior mutability.
/// - `@tanstack/react-table`: `useReactTable()` returns functions that cannot be
///   memoized safely.
/// - `@tanstack/react-virtual`: `useVirtualizer()` returns functions that cannot be
///   memoized safely.
fn default_module_type_provider(module_name: &str) -> Option<ModuleTypeConfig> {
    match module_name {
        "react-hook-form" => {
            // Only the `watch()` function returned by react-hook-form's `useForm()` API
            // is incompatible with memoization.
            let mut watch_properties = FxHashMap::default();
            watch_properties.insert(
                "watch".to_string(),
                ModuleTypeConfig::Function {
                    positional_params: vec![],
                    rest_param: Some(Effect::Read),
                    callee_effect: Effect::Read,
                    return_type: Box::new(ModuleTypeConfig::TypeReference {
                        name: BuiltInTypeName::Any,
                    }),
                    return_value_kind: ValueKind::Mutable,
                    no_alias: false,
                    mutable_only_if_operands_are_mutable: false,
                    impure: false,
                    canonical_name: None,
                    aliasing: None,
                    known_incompatible: Some(
                        "React Hook Form's `useForm()` API returns a `watch()` function \
                         which cannot be memoized safely."
                            .to_string(),
                    ),
                },
            );

            let mut use_form_return = FxHashMap::default();
            use_form_return.insert(
                "useForm".to_string(),
                ModuleTypeConfig::Hook {
                    positional_params: None,
                    rest_param: None,
                    return_type: Box::new(ModuleTypeConfig::Object {
                        properties: watch_properties,
                    }),
                    return_value_kind: None,
                    no_alias: false,
                    aliasing: None,
                    known_incompatible: None,
                },
            );

            Some(ModuleTypeConfig::Object { properties: use_form_return })
        }
        "@tanstack/react-table" => {
            // Many properties of `useReactTable()`'s return value are incompatible,
            // so we mark the entire hook as incompatible.
            let mut properties = FxHashMap::default();
            properties.insert(
                "useReactTable".to_string(),
                ModuleTypeConfig::Hook {
                    positional_params: Some(vec![]),
                    rest_param: Some(Effect::Read),
                    return_type: Box::new(ModuleTypeConfig::TypeReference {
                        name: BuiltInTypeName::Any,
                    }),
                    return_value_kind: None,
                    no_alias: false,
                    aliasing: None,
                    known_incompatible: Some(
                        "TanStack Table's `useReactTable()` API returns functions that \
                         cannot be memoized safely"
                            .to_string(),
                    ),
                },
            );

            Some(ModuleTypeConfig::Object { properties })
        }
        "@tanstack/react-virtual" => {
            // Many properties of `useVirtualizer()`'s return value are incompatible,
            // so we mark the entire hook as incompatible.
            let mut properties = FxHashMap::default();
            properties.insert(
                "useVirtualizer".to_string(),
                ModuleTypeConfig::Hook {
                    positional_params: Some(vec![]),
                    rest_param: Some(Effect::Read),
                    return_type: Box::new(ModuleTypeConfig::TypeReference {
                        name: BuiltInTypeName::Any,
                    }),
                    return_value_kind: None,
                    no_alias: false,
                    aliasing: None,
                    known_incompatible: Some(
                        "TanStack Virtual's `useVirtualizer()` API returns functions that \
                         cannot be memoized safely"
                            .to_string(),
                    ),
                },
            );

            Some(ModuleTypeConfig::Object { properties })
        }
        _ => None,
    }
}
