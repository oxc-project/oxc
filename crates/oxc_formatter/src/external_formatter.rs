use std::sync::Arc;

use oxc_allocator::Allocator;

use super::formatter::{FormatElement, group_id::UniqueGroupIdBuilder};

/// Callback function type for formatting embedded code.
/// Takes (tag_name, code) and returns formatted code or an error.
pub type EmbeddedFormatterCallback =
    Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// Result of formatting embedded code via the Doc→IR path.
///
/// The variant depends on the language being formatted:
/// - GraphQL: multiple IRs (one per quasi text)
/// - CSS: single IR with placeholder survival count
/// - HTML(TODO): single IR with placeholder survival count
pub enum EmbeddedDocResult<'a> {
    MultipleDocs(Vec<Vec<FormatElement<'a>>>),
    /// CSS: The count indicates how many `@prettier-placeholder-N-id` patterns survived formatting
    DocWithPlaceholders(Vec<FormatElement<'a>>, usize),
}

/// Callback function type for formatting embedded code via `Doc`.
///
/// Takes (allocator, group_id_builder, language, texts) and returns [`EmbeddedDocResult`].
/// Used for the Doc→IR path (e.g., `JS:printToDoc()` → Doc JSON → `Rust:FormatElement`).
///
/// The `&Allocator` allows the callback to allocate arena strings for `FormatElement::Text`.
/// The `&UniqueGroupIdBuilder` allows the callback to create `GroupId`s for group/conditional constructs.
pub type EmbeddedDocFormatterCallback = Arc<
    dyn for<'a> Fn(
            &'a Allocator,
            &UniqueGroupIdBuilder,
            &str,
            &[&str],
        ) -> Result<EmbeddedDocResult<'a>, String>
        + Send
        + Sync,
>;

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
    embedded_doc_formatter: Option<EmbeddedDocFormatterCallback>,
    tailwind: Option<TailwindCallback>,
}

impl ExternalCallbacks {
    /// Create a new `ExternalCallbacks` with no callbacks set.
    pub fn new() -> Self {
        Self { embedded_formatter: None, embedded_doc_formatter: None, tailwind: None }
    }

    /// Set the embedded formatter callback.
    #[must_use]
    pub fn with_embedded_formatter(mut self, callback: Option<EmbeddedFormatterCallback>) -> Self {
        self.embedded_formatter = callback;
        self
    }

    /// Set the embedded Doc formatter callback (Doc→IR path).
    #[must_use]
    pub fn with_embedded_doc_formatter(
        mut self,
        callback: Option<EmbeddedDocFormatterCallback>,
    ) -> Self {
        self.embedded_doc_formatter = callback;
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

    /// Format embedded code as Doc.
    ///
    /// # Arguments
    /// * `allocator` - The arena allocator for allocating strings in `FormatElement::Text`
    /// * `group_id_builder` - Builder for creating unique `GroupId`s
    /// * `language` - The embedded language (e.g. "tagged-css", "tagged-graphql")
    /// * `texts` - The code texts to format (multiple quasis for GraphQL, single joined text for CSS/HTML)
    ///
    /// # Returns
    /// * `Some(Ok(EmbeddedDocResult))` - The formatted Doc result, which may contain multiple IRs or placeholder counts depending on the language (see [`EmbeddedDocResult`])
    /// * `Some(Err(String))` - An error message if formatting failed
    /// * `None` - No embedded Doc formatter callback is set
    ///
    pub fn format_embedded_doc<'a>(
        &self,
        allocator: &'a Allocator,
        group_id_builder: &UniqueGroupIdBuilder,
        language: &str,
        texts: &[&str],
    ) -> Option<Result<EmbeddedDocResult<'a>, String>> {
        self.embedded_doc_formatter
            .as_ref()
            .map(|cb| cb(allocator, group_id_builder, language, texts))
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
