use oxc_formatter::SortOptions;
use oxc_formatter_core::CoreFormatOptions;

use super::{
    super::oxfmtrc::FormatConfig, to_core_options::to_core_options,
    to_oxc_formatter::to_sort_options,
};

/// The artifacts of the validation gate:
/// every value whose derivation can fail, derived exactly once.
///
/// Downstream mapping (`FormatStrategy::from_format_config` and the option mappers) consumes these
/// instead of re-deriving, so it stays infallible.
#[derive(Debug, Clone)]
pub struct ValidatedOptions {
    pub core: CoreFormatOptions,
    pub sort: SortOptions,
}

/// The eager validation gate during config resolution.
/// For `Prettier` kinds, it is the only safety net before values reach Prettier.
///
/// # Errors
/// Returns an error if any option value is invalid.
pub fn validate(config: &FormatConfig) -> Result<ValidatedOptions, String> {
    Ok(ValidatedOptions { core: to_core_options(config)?, sort: to_sort_options(config)? })
}
