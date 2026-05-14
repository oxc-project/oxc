//! Lazily-initialized default shape and global registries.
//!
//! `Environment::new` previously rebuilt the entire `ShapeRegistry` (~125 entries)
//! and `GlobalRegistry` on every construction. Both registries are deterministic
//! functions of the static built-in definitions, so we compute them once per
//! process and clone from the cached snapshot.
//!
//! Per-`Environment` customizations (type providers, custom hooks) still apply
//! their mutations after the clone; the snapshot here is the pristine default
//! state matching what `default_shapes()` + `default_globals()` produced.

use std::sync::LazyLock;

use super::globals::{GlobalRegistry, default_globals, default_shapes};
use super::object_shape::ShapeRegistry;

static DEFAULT_REGISTRIES: LazyLock<(ShapeRegistry, GlobalRegistry)> = LazyLock::new(|| {
    let mut shapes = default_shapes();
    let globals = default_globals(&mut shapes);
    (shapes, globals)
});

/// Clone the pristine default `ShapeRegistry`.
pub fn default_shapes_cloned() -> ShapeRegistry {
    DEFAULT_REGISTRIES.0.clone()
}

/// Clone the pristine default `GlobalRegistry`.
pub fn default_globals_cloned() -> GlobalRegistry {
    DEFAULT_REGISTRIES.1.clone()
}
