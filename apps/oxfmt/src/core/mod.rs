mod config;
mod format;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;

#[cfg(feature = "napi")]
pub use config::discover_configs_in_tree;
#[cfg(feature = "napi")]
pub use config::resolve_options_from_value;
pub use config::{
    ConfigResolver, ConfigStore, ResolvedOptions, build_nested_resolver, discover_nested_configs,
    resolve_editorconfig_path, resolve_oxfmtrc_path,
};
pub use format::{FormatResult, SourceFormatter};
pub use support::FormatFileStrategy;

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb,
};
