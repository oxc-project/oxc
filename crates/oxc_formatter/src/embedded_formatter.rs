use std::sync::Arc;

/// Callback function type for formatting embedded code.
/// Takes (tag_name, code) and returns formatted code or an error.
pub type EmbeddedFormatterCallback =
    Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// Formatter for embedded languages in template literals.
///
/// This allows formatting code embedded in template literals like:
/// - CSS in `css\`...\``
/// - GraphQL in `gql\`...\``
/// - HTML in `html\`...\``
#[derive(Clone)]
pub struct EmbeddedFormatter {
    callback: EmbeddedFormatterCallback,
}

/// See <apps/oxfmt/src-js/embedded.ts> for supported tags.
const SUPPORTED_TAGS: &[&str] = &["css", "styled", "gql", "graphql", "html", "md", "markdown"];

impl EmbeddedFormatter {
    /// Create a new embedded formatter with the given callback.
    pub fn new(callback: EmbeddedFormatterCallback) -> Self {
        Self { callback }
    }

    /// Check if the given tag name is supported for embedded formatting.
    /// Supports both simple identifiers (css) and member expressions (styled.div).
    pub fn is_supported_tag(tag_name: &str) -> bool {
        // For member expressions, check if the base identifier is supported
        // e.g., "styled" from "styled.div"
        let base_tag = tag_name.split('.').next().unwrap_or(tag_name);
        SUPPORTED_TAGS.contains(&base_tag)
    }

    /// Format embedded code with the given tag name.
    ///
    /// # Arguments
    /// * `tag_name` - The template tag (e.g., "css", "gql", "html")
    /// * `code` - The code to format
    ///
    /// # Returns
    /// * `Ok(String)` - The formatted code
    /// * `Err(String)` - An error message if formatting failed
    ///
    /// # Errors
    /// Returns an error if the embedded formatter fails to format the code
    pub fn format(&self, tag_name: &str, code: &str) -> Result<String, String> {
        (self.callback)(tag_name, code)
    }
}
