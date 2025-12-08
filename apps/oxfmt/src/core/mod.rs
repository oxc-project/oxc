mod format;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;

pub use format::{FormatResult, SourceFormatter};
pub use support::FormatFileSource;

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatFileCb, JsSetupConfigCb,
};
