mod config;
mod format;
pub mod options;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;

pub use config::{
    ConfigResolver, NestedConfigCtx, ResolveOutcome, resolve_editorconfig_path,
    resolve_file_scope_config,
};
// `config_discovery` is consumed only by LSP code paths (napi-gated).
#[cfg(feature = "napi")]
pub use config::config_discovery;
#[cfg(feature = "napi")]
pub use config::{
    JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader, resolve_for_api,
    resolve_for_embedded_js,
};
pub use format::{FormatResult, FormatStrategy, SourceFormatter};
pub use support::classify_file_kind;

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb,
};
