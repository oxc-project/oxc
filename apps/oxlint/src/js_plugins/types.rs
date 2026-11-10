use napi::{Error, Result};
use napi_derive::napi;

/// Get the type of the node spanning `start..end` (UTF-16 offsets) in `file_path`.
///
/// The type comes from the `tsrs-lint` process that runs type-aware rules, so this
/// blocks until that process has linted, then answers.
///
/// Returns the type as JSON (`{ nodeKind, flags, text }`), or `null` if no node spans the range.
///
/// # Errors
/// Throws if type information is unavailable, or the query failed.
#[napi]
#[allow(dead_code, clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn get_type_at_location(file_path: String, start: u32, end: u32) -> Result<Option<String>> {
    let result = oxc_linter::type_query("typeAtLocation", &file_path, start, end)
        .map_err(Error::from_reason)?;
    Ok((!result.is_null()).then(|| result.to_string()))
}
