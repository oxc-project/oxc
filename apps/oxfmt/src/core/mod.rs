mod config;
pub mod embed;
mod format;
mod global_ignore;
pub mod options;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_services;

pub use config::{ConfigResolver, ConfigScopes, ResolveOutcome, config_discovery};
#[cfg(feature = "napi")]
pub use config::{JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader, resolve_for_api};
pub use format::{FormatResult, FormatStrategy, SourceFormatter};
pub use global_ignore::{build_global_ignore_matchers, is_ignored, resolve_ignore_paths};
pub use support::classify_file_kind;

#[cfg(feature = "napi")]
pub use external_services::{
    ExternalServices, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalServicesCb, JsSortTailwindClassesCb,
};
