use std::sync::Arc;

use oxc_formatter::{SortAttributesOptions, SortImportsOptions};
use oxc_formatter_core::CoreFormatOptions;

use super::{
    super::oxfmtrc::FormatConfig, to_core_options::to_core_options,
    to_oxc_formatter::to_sort_imports, to_sort_attributes::to_sort_attributes,
};

/// The artifacts of the validation gate:
/// every value whose derivation can fail, derived exactly once.
///
/// Downstream mapping (the format step and the option mappers) consumes these
/// instead of re-deriving, so it stays infallible.
#[derive(Debug)]
pub struct ValidatedOptions {
    pub core: CoreFormatOptions,
    pub sort_imports: Option<SortImportsOptions>,
    pub jsx_attributes: Option<Arc<SortAttributesOptions>>,
    pub html_attributes: Option<Arc<oxc_formatter_htmlish::SortAttributesOptions>>,
}

/// The eager validation gate during config resolution.
/// For the `Prettier` strategy, it is the only safety net before values reach Prettier.
///
/// This runs every fallible conversion and returns the derived artifacts
/// (enumerated options are already rejected at deserialize time,
/// everything else is pure field translation).
///
/// # Errors
/// Returns an error if any option value is invalid.
pub fn validate(config: &FormatConfig) -> Result<ValidatedOptions, String> {
    let (jsx_attributes, html_attributes) = to_sort_attributes(config)?;
    Ok(ValidatedOptions {
        core: to_core_options(config)?,
        sort_imports: to_sort_imports(config)?,
        jsx_attributes,
        html_attributes,
    })
}
