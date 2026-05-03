//! Converters from typed [`super::oxfmtrc::FormatConfig`] to each downstream
//! consumer's options shape.
//!
//! - [`to_oxc_formatter()`]: `oxc_formatter::FormatOptions` for JS/TS formatting
//! - [`to_toml_formatter()`]: `oxc_toml::Options` for TOML formatting
//! - `to_prettier`: Prettier-compatible JSON, plus `inject_*` helpers for
//!   layering in `parser` / `filepath` / plugin payloads at the format step
//!   (NAPI-only)
//! - `to_package_json`: `sort_package_json::SortOptions` for `package.json`
//!   (NAPI-only)

mod to_oxc_formatter;
mod to_toml_formatter;

#[cfg(feature = "napi")]
mod to_package_json;
#[cfg(feature = "napi")]
mod to_prettier;

pub use to_oxc_formatter::to_oxc_formatter;
pub use to_toml_formatter::to_toml_formatter;

#[cfg(feature = "napi")]
pub use to_package_json::to_package_json;
#[cfg(feature = "napi")]
pub use to_prettier::{
    inject_filepath, inject_oxfmt_plugin_payload, inject_parser, inject_tailwind_plugin_payload,
    to_prettier,
};
