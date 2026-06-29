use std::sync::Arc;

use oxc_allocator::Allocator;
use oxc_formatter_core::{DispatchResult, EmbeddedContext, FormatDispatcher};

use super::formatter::UniqueGroupIdBuilder;

/// Callback function type for formatting embedded code.
/// Takes (tag_name, code) and returns formatted code or an error.
pub type EmbeddedFormatterCallback =
    Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// Child→parent metadata for CSS formatted as an embedded child.
///
/// NOTE: Belongs to the CSS formatter crate once it exists
/// (see `refactor-oxfmt-architecture.md`); lives here until then because
/// both the producer (oxfmt's dispatcher) and the consumer (`embed/css.rs`)
/// can reach this crate.
pub struct CssEmbedMeta {
    /// How many `@prettier-placeholder-N-id` patterns survived formatting.
    pub placeholder_count: usize,
}

/// Child→parent metadata for HTML/Angular formatted as an embedded child.
///
/// NOTE: Belongs to the HTML formatter crate once it exists (same as [`CssEmbedMeta`]).
pub struct HtmlEmbedMeta {
    /// How many `PRETTIER_HTML_PLACEHOLDER_N_0_IN_JS` patterns survived formatting.
    pub placeholder_count: usize,
    /// Whether the parsed HTML has more than one root element.
    /// Used to decide whether to `indent` the template content.
    pub has_multiple_root_elements: Option<bool>,
}

/// Callback function type for sorting Tailwind CSS classes.
/// Takes classes and returns the sorted versions.
pub type TailwindCallback = Arc<dyn Fn(Vec<String>) -> Vec<String> + Send + Sync>;

/// External callbacks for JS-side functionality.
///
/// This struct holds all callbacks that delegate to external implementations:
/// - Embedded language formatting (CSS, GraphQL, HTML in template literals)
///   via the orchestrator-assembled [`FormatDispatcher`]
/// - Tailwind CSS class sorting
#[derive(Default)]
pub struct ExternalCallbacks {
    embedded_formatter: Option<EmbeddedFormatterCallback>,
    dispatcher: Option<FormatDispatcher>,
    tailwind: Option<TailwindCallback>,
}

impl ExternalCallbacks {
    /// Create a new `ExternalCallbacks` with no callbacks set.
    pub fn new() -> Self {
        Self { embedded_formatter: None, dispatcher: None, tailwind: None }
    }

    /// Set the embedded formatter callback.
    #[must_use]
    pub fn with_embedded_formatter(mut self, callback: Option<EmbeddedFormatterCallback>) -> Self {
        self.embedded_formatter = callback;
        self
    }

    /// Set the embedded-language dispatcher (IR path).
    #[must_use]
    pub fn with_dispatcher(mut self, dispatcher: Option<FormatDispatcher>) -> Self {
        self.dispatcher = dispatcher;
        self
    }

    /// Set the Tailwind callback.
    #[must_use]
    pub fn with_tailwind(mut self, callback: Option<TailwindCallback>) -> Self {
        self.tailwind = callback;
        self
    }

    /// Format embedded code with the given language name.
    ///
    /// # Arguments
    /// * `language` - A generic language identifier (e.g., "css", "html", "graphql").
    ///   These are NOT specific to any external formatter.
    ///   The callback implementation is responsible for mapping them to its own parser/language names.
    /// * `code` - The code to format
    ///
    /// # Returns
    /// * `Some(Ok(String))` - The formatted code
    /// * `Some(Err(String))` - An error message if formatting failed
    /// * `None` - No embedded formatter callback is set
    pub fn format_embedded(&self, language: &str, code: &str) -> Option<Result<String, String>> {
        self.embedded_formatter.as_ref().map(|cb| cb(language, code))
    }

    /// Format embedded code through the dispatcher (IR path).
    ///
    /// Builds an [`EmbeddedContext`] from the current formatting state and
    /// invokes the dispatcher with it, so the child formatter shares this
    /// formatter's arena and `GroupId` space (and can recurse further).
    ///
    /// # Arguments
    /// * `allocator` - The arena allocator for allocating strings in `FormatElement::Text`
    /// * `group_id_builder` - Builder for creating unique `GroupId`s
    /// * `language` - A generic language identifier (e.g., "css", "graphql", "html", "angular").
    ///   These are NOT specific to any external formatter.
    ///   The dispatcher implementation is responsible for mapping them to its own parser/language names.
    /// * `texts` - The code texts to format (multiple quasis for GraphQL, single joined text for CSS/HTML)
    ///
    /// # Returns
    /// * `Some(Ok(DispatchResult))` - The formatted IR(s) plus optional child→parent metadata
    /// * `Some(Err(String))` - An error message if formatting failed
    /// * `None` - No dispatcher is set
    pub fn dispatch_embedded<'a>(
        &self,
        allocator: &'a Allocator,
        group_id_builder: &UniqueGroupIdBuilder,
        language: &str,
        texts: &[&str],
    ) -> Option<Result<DispatchResult<'a>, String>> {
        let dispatcher = self.dispatcher.as_ref()?;
        let ctx = EmbeddedContext {
            allocator,
            group_id_builder,
            dispatcher: Some(Arc::clone(dispatcher)),
        };
        Some(dispatcher(&ctx, language, texts, None))
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
