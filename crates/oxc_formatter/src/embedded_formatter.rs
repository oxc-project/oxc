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

impl EmbeddedFormatter {
    /// Create a new embedded formatter with the given callback.
    pub fn new(callback: EmbeddedFormatterCallback) -> Self {
        Self { callback }
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
