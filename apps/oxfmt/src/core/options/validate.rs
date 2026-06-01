use super::{
    super::oxfmtrc::FormatConfig,
    to_core_options::to_core_options,
    to_oxc_formatter::{to_jsdoc, to_sort_imports},
};

/// Validate the entire config (core + JS/TS-specific options)
/// without building any formatter's options.
///
/// This is the eager validation gate during config resolution.
/// For `ExternalFormatter*` kinds it is the only safety net before values reach Prettier.
/// For js-in-xxx path, it must catch JS-specific config errors too, not just core options.
///
/// # Errors
/// Returns an error if any option value is invalid.
pub fn validate(config: &FormatConfig) -> Result<(), String> {
    to_core_options(config)?;
    to_sort_imports(config)?;
    to_jsdoc(config)?;
    Ok(())
}
