//! Package.json sorting functionality using the sort-package-json crate.

use oxc_diagnostics::OxcDiagnostic;

/// Sort a package.json file's content.
///
/// Uses the sort-package-json crate to reorder top-level fields
/// according to well-established npm conventions.
///
/// # Arguments
/// * `source_text` - The raw package.json content as a string
///
/// # Returns
/// * `Ok(String)` - The sorted JSON content
/// * `Err(OxcDiagnostic)` - If parsing or sorting fails
pub fn sort_package_json_content(source_text: &str) -> Result<String, OxcDiagnostic> {
    sort_package_json::sort_package_json(source_text).map_err(|err| {
        OxcDiagnostic::error(format!("Failed to parse package.json for sorting: {err}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_package_json_basic() {
        let input = r#"{"version":"1.0.0","name":"test"}"#;
        let result = sort_package_json_content(input).unwrap();
        // Name should come before version
        assert!(result.find("\"name\"").unwrap() < result.find("\"version\"").unwrap());
    }

    #[test]
    fn test_sort_package_json_preserves_data() {
        let input = r#"{"scripts":{"test":"echo"},"name":"test","version":"1.0.0"}"#;
        let result = sort_package_json_content(input).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"version\""));
        assert!(result.contains("\"scripts\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"echo\""));
    }

    #[test]
    fn test_sort_package_json_invalid_json() {
        let input = r#"{"name": invalid}"#;
        let result = sort_package_json_content(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_package_json_empty_object() {
        let input = r#"{}"#;
        let result = sort_package_json_content(input).unwrap();
        assert_eq!(result.trim(), "{}");
    }
}
