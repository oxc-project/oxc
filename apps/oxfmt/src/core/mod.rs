mod config;
mod format;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;
#[cfg(feature = "napi")]
mod js_config;

#[cfg(feature = "napi")]
pub use config::resolve_options_from_value;
pub use config::{ConfigResolver, ResolvedOptions, config_discovery, resolve_editorconfig_path};
pub use format::{FormatResult, SourceFormatter};
pub use support::{FormatStrategy, FormatStrategyBuilder};

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb,
};
#[cfg(feature = "napi")]
pub use js_config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader};
