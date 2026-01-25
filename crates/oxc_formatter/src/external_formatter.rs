use std::sync::Arc;

/// Callback function type for formatting embedded code.
/// Takes (tag_name, code) and returns formatted code or an error.
pub type EmbeddedFormatterCallback =
    Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for sorting Tailwind CSS classes.
/// Takes classes and returns the sorted versions.
pub type TailwindCallback = Arc<dyn Fn(Vec<String>) -> Vec<String> + Send + Sync>;

/// External callbacks for JS-side functionality.
///
/// This struct holds all callbacks that delegate to external (typically JS) implementations:
/// - Embedded language formatting (CSS, GraphQL, HTML in template literals)
/// - Tailwind CSS class sorting
#[derive(Default)]
pub struct ExternalCallbacks {
    embedded_formatter: Option<EmbeddedFormatterCallback>,
    tailwind: Option<TailwindCallback>,
}

impl ExternalCallbacks {
    /// Create a new `ExternalCallbacks` with no callbacks set.
    pub fn new() -> Self {
        Self { embedded_formatter: None, tailwind: None }
    }

    /// Set the embedded formatter callback.
    #[must_use]
    pub fn with_embedded_formatter(mut self, callback: Option<EmbeddedFormatterCallback>) -> Self {
        self.embedded_formatter = callback;
        self
    }

    /// Set the Tailwind callback.
    #[must_use]
    pub fn with_tailwind(mut self, callback: Option<TailwindCallback>) -> Self {
        self.tailwind = callback;
        self
    }

    /// Format embedded code with the given tag name.
    ///
    /// # Arguments
    /// * `tag_name` - The template tag (e.g., "css", "gql", "html")
    /// * `code` - The code to format
    ///
    /// # Returns
    /// * `Some(Ok(String))` - The formatted code
    /// * `Some(Err(String))` - An error message if formatting failed
    /// * `None` - No embedded formatter callback is set
    pub fn format_embedded(&self, tag_name: &str, code: &str) -> Option<Result<String, String>> {
        self.embedded_formatter.as_ref().map(|cb| cb(tag_name, code))
    }

    /// Sort Tailwind CSS classes.
    ///
    /// # Arguments
    /// * `classes` - List of class strings to sort
    ///
    /// # Returns
    /// The sorted classes, or the original classes unsorted if no Tailwind callback is set.
    pub fn sort_tailwind_classes(&self, classes: Vec<String>) -> Vec<String> {
        if classes.is_empty() {
            return classes;
        }

        match self.tailwind.as_ref() {
            Some(cb) => cb(classes),
            None => classes,
        }
    }
}
