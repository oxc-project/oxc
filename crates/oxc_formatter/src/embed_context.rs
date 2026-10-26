//! JS↔other language pair specific dispatch context types
//! (host-delegated services travel as `SessionServices` on the `FormatSession` instead).
//!
//! With JS as the parent, JS picks the child language and passes the pair's own data.
//! With JS as the child of a parent that cannot pick (a Markdown code block),
//! the host translates the parent's fact about itself into the pair's data ([`JsEmbeddedIn`]).
//!
//! NOTE: These live here permanently, NOT in other formatter crate:
//! the consumer is this crate's `embed/*.rs`, and `oxc_formatter` must never depend on language crates.
//! What every pair shares (the Tailwind class scope) travels on the `FormatSession` in `oxc_formatter_core` instead.

/// Parent→child parse-mode context for CSS dispatched from a JS template literal (css-in-js).
///
/// Requests SCSS grammar + `${}` placeholder markers + top-level declarations.
/// Travels as `DispatchRequest::parent_context`;
/// its ABSENCE means the child parses as a plain standalone stylesheet
/// (e.g. a JSDoc fence routed through the dispatcher).
pub struct CssInJsTemplate;

/// Parent→child context for Markdown dispatched from a JS template literal (markdown-in-js).
///
/// The child prints its code fences with `~` instead of backticks,
/// which would need escaping in the template.
pub struct MarkdownInJsTemplate;

/// Parent→child context for HTML/Angular dispatched from a JS template literal (html-in-js).
pub struct HtmlInJsTemplate {
    /// The backticks hug the content (no whitespace on both ends, whitespace sensitivity not `ignore`):
    /// the child indents its content when more than one node prints at the root.
    pub indent_multiple_roots: bool,
}

/// What a dispatched JS/TS program is embedded in, for the print rules that depend on it.
///
/// The host translates the parent's fact about itself into this (see [`crate::format_to_ir`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsEmbeddedIn {
    /// A Markdown code block (js-in-markdown):
    /// a program of a single JSX expression statement prints without its semicolon.
    MarkdownCodeBlock,
}
