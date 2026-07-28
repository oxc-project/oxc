use super::{
    super::oxfmtrc::FormatConfig, to_core_options::to_core_options,
    to_oxc_formatter::to_sort_imports,
};

/// This is the eager validation gate during config resolution.
/// For `ExternalFormatter*` kinds, it is the only safety net before values reach Prettier.
///
/// This lists every fallible conversion rather than building any formatter's options.
/// (Enumerated options are already rejected at deserialize time.)
///
/// # Errors
/// Returns an error if any option value is invalid.
pub fn validate(config: &FormatConfig) -> Result<(), String> {
    to_core_options(config)?;
    to_sort_imports(config)?;
    Ok(())
}
