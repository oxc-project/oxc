mod config;
mod format;
pub mod options;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;
#[cfg(feature = "napi")]
mod js_config;

pub use config::{ConfigResolver, config_discovery, resolve_editorconfig_path};
#[cfg(feature = "napi")]
pub use config::{resolve_for_api, resolve_for_embedded_js};
pub use format::{FormatResult, FormatStrategy, SourceFormatter};
pub use support::classify_file_kind;

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb,
};
#[cfg(feature = "napi")]
pub use js_config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader};
