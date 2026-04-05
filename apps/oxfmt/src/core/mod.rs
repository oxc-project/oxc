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
pub use config::all_config_file_names;
pub use config::{ConfigResolver, ResolvedOptions, resolve_editorconfig_path};
#[cfg(feature = "napi")]
pub use config::{extract_external_plugin_specs, resolve_options_from_value};
pub use format::{FormatResult, SourceFormatter};
pub use support::{ExternalPluginSupport, FormatFileStrategy};

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalFormatterCb, JsSortTailwindClassesCb,
};
#[cfg(feature = "napi")]
pub use js_config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader};
