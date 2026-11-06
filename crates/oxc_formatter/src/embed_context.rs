//! JS↔other language pair specific dispatch context types
//! (host-delegated services travel as `SessionServices` on the `FormatSession` instead).
//!
//! With JS as the parent, JS picks the child language and passes the pair's own data.
//! With JS as the child of a parent that cannot pick (a Markdown code block),
//! the host translates the parent's fact about itself into the pair's data ([`JsRoot`], [`JsEmbeddedIn`]).
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
/// so the parens around it in the source stay.
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
    /// The prefix of the placeholder standing in for a `${}` expression,
    /// and the test for a whole one (`is_html_placeholder`).
    pub placeholder: (&'static str, fn(&str) -> bool),
}

/// What an embedded JS/TS part is parsed as, and how its kind prints (see [`crate::format_to_ir`]).
///
/// The JS formatter knows nothing about the host's vocabulary; each variant names its use as an example.
#[derive(Clone, Copy, Debug)]
pub enum JsRoot {
    /// Statements (a `<script>`'s, a Markdown code block's, an event handler attribute's).
    Program { semicolons: ProgramSemicolons, layout: ProgramLayout },
}

/// How a program's top-level statements end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramSemicolons {
    /// As `semi` says (a `<script>`).
    AsOptions,
    /// A lone JSX expression statement prints without its `;` (a Markdown code block shows a JSX snippet).
    LoneJsxOmitted,
    /// A lone expression statement prints without its `;` (an event handler attribute, a function's body).
    LoneExpressionOmitted,
}

/// How a program's statements lay out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramLayout {
    /// No wrapper; the host lays them out (a `<script>`'s block).
    Bare,
    /// On their own lines when they break, in a group the host puts its delimiters around (an attribute's value).
    Indented,
}

/// What an embedded JS/TS part is embedded in: what its host does with the code.
///
/// For the rules that keep what the host reads of it (see [`crate::format_to_ir`]).
/// The default does not quote the code (a `<script>`, a Markdown code block).
#[derive(Clone, Copy, Debug, Default)]
pub struct JsEmbeddedIn {
    /// The code is a `"`-quoted HTML attribute's value: strings print in `'`.
    pub in_html_attribute: bool,
}
