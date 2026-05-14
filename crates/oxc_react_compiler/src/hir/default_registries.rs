//! Lazily-initialized default shape and global registries.
//!
//! `Environment::new` previously rebuilt the entire `ShapeRegistry` (~125 entries)
//! and `GlobalRegistry` on every construction. Both registries are deterministic
//! functions of the static built-in definitions, so we compute them once per
//! process and share the cached snapshot via `Arc`.
//!
//! Per-`Environment` customizations (type providers, custom hooks) still apply
//! their mutations after handing out an `Arc` snapshot. Customization passes
//! call `Arc::make_mut` once before mutating, which clones the underlying
//! registry exactly once if the snapshot is shared; otherwise it's a no-op.
//!
//! The `DefaultModuleTypeProvider` modules (`react-hook-form`,
//! `@tanstack/react-table`, `@tanstack/react-virtual`) are deterministic — their
//! type configs depend only on the module name — so they are pre-resolved into
//! the cached snapshot as well, and exposed as a pristine `module_types`
//! `Arc<FxHashMap>` ready for any `Environment` to start from.

use std::sync::{Arc, LazyLock};

use rustc_hash::FxHashMap;

use super::default_module_type_provider::{DefaultModuleTypeProvider, ModuleTypeProvider};
use super::globals::{GlobalRegistry, default_globals, default_shapes, install_type_config};
use super::object_shape::ShapeRegistry;
use super::types::Type;

/// Bundle of pristine cached registries shared by every `Environment`.
///
/// All three components are wrapped in `Arc` so `Environment::new` can hand out
/// references cheaply; mutations clone-on-write via `Arc::make_mut`.
struct DefaultRegistries {
    shapes: Arc<ShapeRegistry>,
    globals: Arc<GlobalRegistry>,
    module_types: Arc<FxHashMap<String, Type>>,
}

static DEFAULT_REGISTRIES: LazyLock<DefaultRegistries> = LazyLock::new(|| {
    let mut shapes = default_shapes();
    let globals = default_globals(&mut shapes);

    // Pre-resolve every module known to `DefaultModuleTypeProvider`. These are
    // global to the provider (not config-gated) and their type configs depend
    // only on the module name, so resolving them once at LazyLock-init time
    // keeps the per-`Environment::new` path on the pure Arc-bump fast path for
    // a vanilla `EnvironmentConfig::default()`.
    let mut module_types: FxHashMap<String, Type> = FxHashMap::default();
    for module_name in DefaultModuleTypeProvider::KNOWN_MODULE_NAMES {
        if let Some(type_config) = DefaultModuleTypeProvider.get_type(module_name) {
            let t = install_type_config(&mut shapes, type_config, module_name);
            module_types.insert((*module_name).to_string(), t);
        }
    }

    DefaultRegistries {
        shapes: Arc::new(shapes),
        globals: Arc::new(globals),
        module_types: Arc::new(module_types),
    }
});

/// Get an `Arc` handle to the pristine default `ShapeRegistry`.
pub fn default_shapes_arc() -> Arc<ShapeRegistry> {
    Arc::clone(&DEFAULT_REGISTRIES.shapes)
}

/// Get an `Arc` handle to the pristine default `GlobalRegistry`.
pub fn default_globals_arc() -> Arc<GlobalRegistry> {
    Arc::clone(&DEFAULT_REGISTRIES.globals)
}

/// Get an `Arc` handle to the pristine default `module_types` map.
///
/// The map contains every module declared in
/// [`DefaultModuleTypeProvider::KNOWN_MODULE_NAMES`], pre-resolved against the
/// cached default `ShapeRegistry`. Config-gated type providers
/// (`enableCustomTypeDefinitionForReanimated`,
/// `enableSharedRuntimeTypeProvider`) extend this map via `Arc::make_mut` in
/// `EnvironmentContext::build`.
pub fn default_module_types_arc() -> Arc<FxHashMap<String, Type>> {
    Arc::clone(&DEFAULT_REGISTRIES.module_types)
}
