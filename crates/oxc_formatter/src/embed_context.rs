//! JS↔child language-pair specific dispatch context types
//! (host-delegated services travel as `SessionServices` on the `FormatSession` instead).

/// Parent→child parse-mode context for CSS dispatched from a JS template literal (css-in-js).
///
/// Requests SCSS grammar + `${}` placeholder markers + top-level declarations.
/// Travels as `DispatchRequest::parent_context`;
/// its ABSENCE means the child parses as a plain standalone stylesheet
/// (e.g. a JSDoc fence routed through the dispatcher).
///
/// Defined here for the same reason as [`HtmlEmbedMeta`]:
/// it is JS↔CSS pair-specific, and `oxc_formatter` must never depend on language crates.
pub struct CssInJsTemplate;

/// Child→parent pair context for HTML/Angular formatted as an embedded child.
///
/// NOTE: This lives here permanently, NOT in a future HTML formatter crate:
/// the consumer is this crate's `embed/html.rs` (the JS side of html-in-js),
/// and `oxc_formatter` must never depend on language crates. Cross-language
/// contract fields (placeholder counts, Tailwind classes) are first-class on
/// `DispatchPayload` in `oxc_formatter_core` instead; only what is truly
/// specific to the JS↔HTML pair travels as the `dyn Any` child context.
pub struct HtmlEmbedMeta {
    /// Whether the parsed HTML has more than one root element.
    /// Used to decide whether to `indent` the template content.
    pub has_multiple_root_elements: Option<bool>,
}
