mod config;
pub mod embed;
mod format;
pub mod options;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_services;

pub use config::{
    ConfigResolver, NestedConfigCtx, ResolveOutcome, config_discovery, resolve_editorconfig_path,
    resolve_file_scope_config,
};
#[cfg(feature = "napi")]
pub use config::{
    EmbeddedCallbackResolved, JsConfigLoaderCb, JsLoadJsConfigCb, create_js_config_loader,
    resolve_for_api, resolve_for_embedded_js,
};
pub use format::{FormatResult, FormatStrategy, SourceFormatter};
pub use support::classify_file_kind;

#[cfg(feature = "napi")]
pub use external_services::{
    ExternalServices, JsFormatEmbeddedCb, JsFormatEmbeddedDocCb, JsFormatFileCb,
    JsInitExternalServicesCb, JsSortTailwindClassesCb,
};
