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

use std::sync::{Arc, LazyLock};

use super::globals::{GlobalRegistry, default_globals, default_shapes};
use super::object_shape::ShapeRegistry;

static DEFAULT_REGISTRIES: LazyLock<(Arc<ShapeRegistry>, Arc<GlobalRegistry>)> =
    LazyLock::new(|| {
        let mut shapes = default_shapes();
        let globals = default_globals(&mut shapes);
        (Arc::new(shapes), Arc::new(globals))
    });

/// Get an `Arc` handle to the pristine default `ShapeRegistry`.
pub fn default_shapes_arc() -> Arc<ShapeRegistry> {
    Arc::clone(&DEFAULT_REGISTRIES.0)
}

/// Get an `Arc` handle to the pristine default `GlobalRegistry`.
pub fn default_globals_arc() -> Arc<GlobalRegistry> {
    Arc::clone(&DEFAULT_REGISTRIES.1)
}
