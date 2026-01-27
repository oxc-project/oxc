mod config;
mod format;
pub mod oxfmtrc;
mod support;
pub mod utils;

#[cfg(feature = "napi")]
mod external_formatter;

pub use config::{
    ConfigResolver, ResolvedOptions, resolve_editorconfig_path, resolve_oxfmtrc_path,
};
pub use format::{FormatResult, SourceFormatter};
pub use support::FormatFileStrategy;

#[cfg(feature = "napi")]
pub use external_formatter::{
    ExternalFormatter, JsFormatEmbeddedCb, JsFormatFileCb, JsInitExternalFormatterCb,
    JsSortTailwindClassesCb, wrap_format_embedded_only, wrap_sort_tailwind_for_doc,
};
