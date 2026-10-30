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

/// Prettier uses `PRETTIER_HTML_PLACEHOLDER_{index}_{counter}_IN_JS` as placeholders, the counter global.
/// Ours is `PRETTIER_HTML_PLACEHOLDER_{depth}_{index}_IN_JS`, the session's dispatch depth:
/// an html-in-js nested in this one (in its `<script>`) is deeper, so neither takes the other's placeholders,
/// and the same input always gets the same placeholders.
///
/// A child may change its case (CSS lowercases a property name: `${b}: 1px` in a `<style>`),
/// so it is found ASCII case-insensitively in the child's IR.
/// The shape stays Prettier's because angular-in-js still formats through Prettier,
/// which recognizes `/^PRETTIER_HTML_PLACEHOLDER_\d+_\d+_IN_JS$/` (unquoted values, parens around an identifier).
/// TODO: Once Angular formats natively, the name is ours to choose,
/// e.g. one no child changes the case of, and the case-insensitive search can go.
pub const HTML_PLACEHOLDER_PREFIX: &str = "PRETTIER_HTML_PLACEHOLDER_";
pub const HTML_PLACEHOLDER_SUFFIX: &str = "_IN_JS";

/// `name` is an html-in-js placeholder, of any depth.
/// One in a `<script>` stands in for an expression of any precedence (`f((${a + b}) / 2)`),
/// so the parens around it in the source stay (Prettier's `parentheses/identifier.js`).
pub fn is_html_placeholder(name: &str) -> bool {
    name.strip_prefix(HTML_PLACEHOLDER_PREFIX)
        .and_then(|id| id.strip_suffix(HTML_PLACEHOLDER_SUFFIX))
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit() || b == b'_'))
}

/// Parent→child context for HTML/Angular dispatched from a JS template literal (html-in-js).
pub struct HtmlInJsTemplate {
    /// The backticks hug the content (no whitespace on both ends, whitespace sensitivity not `ignore`):
    /// the child indents its content when more than one node prints at the root.
    pub indent_multiple_roots: bool,
    /// The prefix and suffix around the id of the placeholder standing in for a `${}` expression.
    pub placeholder: (&'static str, &'static str),
}

/// What a dispatched JS/TS program is embedded in, for the print rules that depend on it.
///
/// The host translates the parent's fact about itself into this (see [`crate::format_to_ir`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsEmbeddedIn {
    /// A Markdown code block (js-in-markdown):
    /// a program of a single JSX expression statement prints without its semicolon.
    MarkdownCodeBlock,
    /// An HTML event handler attribute's value (`onclick="..."`):
    /// a single expression statement prints without its semicolon, and strings in `'` (the attribute's quote is `"`).
    HtmlEventHandler,
}
