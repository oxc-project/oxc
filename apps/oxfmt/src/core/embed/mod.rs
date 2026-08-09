//! Embedded-language formatting orchestration.
//!
//! `oxc_formatter_core` holds the abstract contract
//! (`FormatSession`, `FormatDispatcher`, `DispatchRequest`/`DispatchOutcome`, `TailwindCollector`);
//! this module is its concrete counterpart owned by the orchestrator (Oxfmt).
//!
//! - [`dispatcher`] (every build): the native registry — `ResolvedDispatchConfig`
//!   (lazy per-language options) + `build_dispatcher` with a Rust branch per `NativeLanguage`;
//!   IR integrates into the parent's arena / `GroupId` space
//! - [`fence`] (every build): the JSDoc native-fence string adapter over the registry
//!   (the pure build's whole string-out channel)
//! - [`prettier_fallback`] (napi only): Prettier Doc→IR path for the rest
//! - [`string_channel`] (napi only): the Prettier string paths of the string-out channel
//!   - md/html/angular JSDoc fences + html-in-js fallback
//!   - Standalone string formatting that re-embeds line-by-line

#[cfg(feature = "napi")]
use std::sync::Arc;

#[cfg(feature = "napi")]
use serde_json::Value;

pub mod dispatcher;
pub mod fence;
#[cfg(feature = "napi")]
pub mod prettier_fallback;
#[cfg(feature = "napi")]
pub mod string_channel;

// --- Cross-module callback types ---
//
// These describe the shape of the napi-wrapped callbacks the orchestration builders consume.
// They live here (not in `external_formatter`)
// so the `prettier_fallback` / `string_channel` factories stay independent of the napi boundary.
// `external_formatter` is the producer of these types via its `wrap_*` functions,
// and orchestration is the consumer.

/// Callback function type for formatting files with config.
/// Takes (options, code) and returns formatted code or an error.
/// The `options` Value is owned and includes `parser` and `filepath` set by the caller.
#[cfg(feature = "napi")]
pub type FormatFileWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for formatting embedded code with config.
/// Takes (options, code) and returns formatted code or an error.
/// The `options` Value is owned and includes `parser` set by the caller.
#[cfg(feature = "napi")]
pub type FormatEmbeddedWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for formatting embedded code via Doc IR path (batch).
/// Takes (options, texts) and returns Doc JSON strings (one per text) or an error.
#[cfg(feature = "napi")]
pub type FormatEmbeddedDocWithConfigCallback =
    Arc<dyn Fn(Value, &[&str]) -> Result<Vec<String>, String> + Send + Sync>;

/// Internal callback type for Tailwind processing with config.
/// Takes (options, classes) and returns sorted classes.
/// The `filepath` is included in `options`.
#[cfg(feature = "napi")]
pub type TailwindWithConfigCallback = Arc<dyn Fn(&Value, Vec<String>) -> Vec<String> + Send + Sync>;

// --- Shared language → Prettier parser mapping ---

/// Mapping from language identifiers to Prettier `parser` names.
/// This is the single source of truth for embedded languages that can still
/// reach Prettier; languages fully served by a Rust crate are absent
/// (css/scss/less and graphql/gql — both the dispatcher branches and the
/// string channel format them in Rust before this map, with no fallback).
///
/// Language identifiers come from two sources:
/// - xxx-in-js `(Tagged)TemplateLiteral` (`embed/*.rs`)
/// - JSDoc fenced code blocks (`jsdoc/mdast_serialize/`)
///
/// NOTE: these identifiers happen to overlap with some Prettier parser names,
/// but `oxc_formatter` treats them as generic language names.
/// This function is the only place that maps them to Prettier-specific parsers.
#[cfg(feature = "napi")]
pub fn language_to_prettier_parser(language: &str) -> Option<&'static str> {
    match language {
        "html" => Some("html"),
        "angular" => Some("angular"),
        "markdown" | "md" => Some("markdown"),
        _ => None,
    }
}
