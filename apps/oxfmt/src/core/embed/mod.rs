//! Embedded-language formatting orchestration.
//!
//! `oxc_formatter_core` holds the abstract contract
//! (`FormatSession`, `FormatDispatcher`, `DispatchRequest`/`DispatchResponse`, `TailwindCollector`);
//! this module is its concrete counterpart owned by the orchestrator (Oxfmt).
//!
//! - [`dispatcher`] (every build):
//!   the routing table (`route`: one function answering "which formatter serves this language?"),
//!   `ResolvedDispatchConfig` (lazy per-language options) + `build_dispatcher` with a Rust branch per `NativeLanguage`;
//!   IR integrates into the parent's arena / `GroupId` space
//! - [`services`] (every build): the root `SessionServices` assembly (`for_root`, one definition per build)
//! - [`jsdoc_fence`] (every build): the JSDoc native-fence string adapter over the registry
//! - [`prettier_doc`] (napi only): Prettier Doc→IR path for the `Route::Prettier` set
//! - [`prettier_string`] (napi only): the Prettier string paths of the string-out channel
//!   (md/html/angular JSDoc fences; results re-embed line-by-line)

#[cfg(feature = "napi")]
use std::sync::Arc;

#[cfg(feature = "napi")]
use serde_json::Value;

pub mod dispatcher;
pub mod jsdoc_fence;
#[cfg(feature = "napi")]
pub mod prettier_doc;
#[cfg(feature = "napi")]
pub mod prettier_string;
pub mod services;

// --- Cross-module callback types ---
//
// These describe the shape of the napi-wrapped callbacks the orchestration builders consume.
// NOTE: They live here (not in `external_services`),
// so the `prettier_doc` / `prettier_string` factories stay independent of the napi boundary.
// `external_services` is the producer of these types via its `wrap_*` functions, and orchestration is the consumer.

/// Callback function type for formatting embedded code with config.
/// Takes (options, code) and returns formatted code or an error.
/// The `options` Value is owned and includes `parser` set by the caller.
#[cfg(feature = "napi")]
pub type FormatEmbeddedWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for formatting embedded code via the Doc IR path.
/// Takes (options, text) and returns a Doc JSON string or an error.
#[cfg(feature = "napi")]
pub type FormatEmbeddedDocWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Internal callback type for Tailwind processing with config.
/// Takes (options, classes) and returns sorted classes.
/// The `filepath` is included in `options`.
#[cfg(feature = "napi")]
pub type TailwindWithConfigCallback = Arc<dyn Fn(&Value, Vec<String>) -> Vec<String> + Send + Sync>;
