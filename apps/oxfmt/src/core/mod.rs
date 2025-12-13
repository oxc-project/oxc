mod config;
mod eof;
mod format;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;

pub use config::{load_config, resolve_config_path};
pub use eof::{equals_with_eof_adjustment, write_with_eof_adjustment};
pub use format::{FormatResult, SourceFormatter};
pub use support::FormatFileStrategy;

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatFileCb, JsSetupConfigCb,
};
