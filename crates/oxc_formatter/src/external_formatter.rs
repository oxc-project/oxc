use std::sync::Arc;

use super::formatter::format_element::{LineMode, PrintMode};

/// Callback function type for formatting embedded code.
/// Takes (tag_name, code) and returns formatted code or an error.
pub type EmbeddedFormatterCallback =
    Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for formatting embedded code via Doc in batch.
///
/// Takes (tag_name, texts) and returns one `Vec<EmbeddedIR>` per input.
/// Used for the Doc→IR path (e.g., `printToDoc` → Doc JSON → `EmbeddedIR`).
///
/// For GraphQL, each quasi is a separate text (`texts.len() == quasis.len()`).
/// For CSS/HTML, quasis are joined with placeholders into a single text (`texts.len() == 1`).
pub type EmbeddedDocFormatterCallback =
    Arc<dyn Fn(&str, &[&str]) -> Result<Vec<Vec<EmbeddedIR>>, String> + Send + Sync>;

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

    /// Format embedded code as Doc in batch.
    ///
    /// Takes multiple texts and returns one `Vec<EmbeddedIR>` per input text.
    /// The caller is responsible for interleaving the results with JS expressions.
    ///
    /// # Arguments
    /// * `tag_name` - The template tag (e.g., "css", "gql", "html")
    /// * `texts` - The code texts to format (multiple quasis for GraphQL, single joined text for CSS/HTML)
    ///
    /// # Returns
    /// * `Some(Ok(Vec<Vec<EmbeddedIR>>))` - The formatted code as a vector of `EmbeddedIR` for each input text
    /// * `Some(Err(String))` - An error message if formatting failed
    /// * `None` - No embedded formatter callback is set
    pub fn format_embedded_doc(
        &self,
        tag_name: &str,
        texts: &[&str],
    ) -> Option<Result<Vec<Vec<EmbeddedIR>>, String>> {
        self.embedded_doc_formatter.as_ref().map(|cb| cb(tag_name, texts))
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

// ---

/// Owned intermediate IR for embedded language formatting.
///
/// This type bridges the callback boundary between `apps/oxfmt` (or other callers) and `oxc_formatter`.
/// Unlike `FormatElement<'a>`, it has no lifetime parameter and owns all its data,
/// so it can be returned from `Arc<dyn Fn>` callbacks.
///
/// The `oxc_formatter` side converts `EmbeddedIR` → `FormatElement<'a>` using the allocator.
#[derive(Debug, Clone)]
pub enum EmbeddedIR {
    Space,
    HardSpace,
    Line(LineMode),
    ExpandParent,
    /// Owned string (unlike `FormatElement::Text` which borrows from the arena).
    Text(String),
    LineSuffixBoundary,
    // --- Tag equivalents (all fields pub, no lifetime) ---
    StartIndent,
    EndIndent,
    /// Positive integer only. Converted to `Tag::StartAlign(Align(NonZeroU8))`.
    StartAlign(u8),
    EndAlign,
    /// - `to_root: false` → `DedentMode::Level`
    /// - `to_root: true` → `DedentMode::Root`
    StartDedent {
        to_root: bool,
    },
    EndDedent {
        to_root: bool,
    },
    /// `id` is a numeric group ID (mapped to `GroupId` via `HashMap<u32, GroupId>`).
    StartGroup {
        id: Option<u32>,
        should_break: bool,
    },
    EndGroup,
    /// `mode` = Break or Flat, `group_id` references a group by numeric ID.
    StartConditionalContent {
        mode: PrintMode,
        group_id: Option<u32>,
    },
    EndConditionalContent,
    /// GroupId is mandatory (matches `Tag::StartIndentIfGroupBreaks(GroupId)`).
    StartIndentIfGroupBreaks(u32),
    EndIndentIfGroupBreaks(u32),
    StartFill,
    EndFill,
    StartEntry,
    EndEntry,
    StartLineSuffix,
    EndLineSuffix,
}
