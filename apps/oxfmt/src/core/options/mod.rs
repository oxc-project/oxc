//! Converters from typed [`super::oxfmtrc::FormatConfig`] to each downstream consumer's options shape.
//!
//! - [`to_oxc_formatter()`]: `oxc_formatter::JsFormatOptions` for JS/TS formatting
//! - [`to_oxc_formatter_json()`]: `oxc_formatter_json::JsonFormatOptions` for JSON formatting
//!   - [`to_sort_package_json()`]: its companion `sort_package_json::SortOptions`
//!     for `package.json`'s sorting pre-process
//! - [`to_oxc_formatter_css()`]: `oxc_formatter_css::CssFormatOptions` for CSS/SCSS/Less formatting
//! - [`to_oxc_formatter_graphql()`]: `oxc_formatter_graphql::GraphqlFormatOptions` for GraphQL formatting
//! - [`to_oxc_formatter_yaml()`]: `oxc_formatter_yaml::YamlFormatOptions` for YAML formatting
//! - [`to_oxc_toml()`]: `oxc_toml::Options` for TOML formatting
//! - `to_prettier`(NAPI-only): Prettier-compatible JSON, plus `inject_*` helpers for
//!   layering in `parser` / `filepath` / plugin payloads at the format step
//! - [`validate()`]: validate a config without building any formatter's options

mod to_core_options;
mod to_oxc_formatter;
mod to_oxc_formatter_css;
mod to_oxc_formatter_graphql;
mod to_oxc_formatter_json;
mod to_oxc_formatter_yaml;
mod to_oxc_toml;
#[cfg(feature = "napi")]
mod to_prettier;
mod validate;

pub use to_oxc_formatter::to_oxc_formatter;
pub use to_oxc_formatter_css::to_oxc_formatter_css;
pub use to_oxc_formatter_graphql::to_oxc_formatter_graphql;
pub use to_oxc_formatter_json::{to_oxc_formatter_json, to_sort_package_json};
pub use to_oxc_formatter_yaml::to_oxc_formatter_yaml;
pub use to_oxc_toml::to_oxc_toml;
#[cfg(feature = "napi")]
pub use to_prettier::{
    inject_filepath, inject_oxfmt_plugin_payload, inject_parser, inject_svelte_plugin_payload,
    inject_tailwind_plugin_payload, to_prettier,
};
pub use validate::validate;
