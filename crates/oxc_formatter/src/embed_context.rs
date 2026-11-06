//! JS↔other language pair specific dispatch context types
//! (host-delegated services travel as `SessionServices` on the `FormatSession` instead).
//!
//! With JS as the parent, JS picks the child language and passes the pair's own data.
//! With JS as the child of a parent that cannot pick (a Markdown code block, a template language),
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
    /// An expression (Vue `{{ a }}`, `:a="b"`).
    Expression(ExpressionRoot),
    /// Function params in a binding position (Vue `v-slot`, `v-for` left), in a group.
    FunctionParams {
        /// More than one, or a rest element, print in parens (Vue `v-for` left: `(item, index)`).
        parens_when_many: bool,
    },
    /// Type parameters without their angle brackets (Vue `<script generic>`), on their own lines when they break.
    TypeParameters,
}

/// How a program's top-level statements end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramSemicolons {
    /// As `semi` says (a `<script>`).
    AsOptions,
    /// A lone JSX expression statement prints without its `;` (a Markdown code block shows a JSX snippet).
    LoneJsxOmitted,
    /// A lone expression statement prints without its `;` (an event handler attribute).
    LoneExpressionOmitted,
    /// `;` tells statements from an expression: the host reads code holding one as statements,
    /// other code as an expression it calls or evaluates (Vue's `@click`).
    ///
    /// A lone expression statement keeps its `;` only when it is a reference or a function, whatever `semi` says:
    /// without it, the host calls it (Vue `foo;` runs nothing, `foo` calls it).
    /// Another one reads the same without it, but for its value, which is no part of what the host reads
    /// (Vue uses a handler's value only to report a promise it rejects).
    /// Under `semi: false`, `;` still separates statements, but after the last, and ends a lone one that is no expression.
    MarksStatements,
}

/// How a program's statements lay out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramLayout {
    /// No wrapper; the host lays them out (a `<script>`'s block).
    Bare,
    /// On their own lines when they break, in a group the host puts its delimiters around (an attribute's value).
    Indented,
}

/// How an expression prints.
#[derive(Clone, Copy, Debug)]
pub struct ExpressionRoot {
    pub layout: ExpressionLayout,
    /// The expression holds Tailwind classes (Vue `:class`): its strings sort as in a JSX `className`.
    pub tailwind_classes: bool,
    /// The host ends the code at the first `}}` (Vue's `{{ }}`): an object that would print one prints in parens,
    /// before a template literal substitution's `}` and, under `bracketSpacing: false`, an object's
    /// (but for the known gap, `crates/oxc_formatter/AGENTS.md` "js-in-xxx").
    pub ends_at_double_brace: bool,
    /// The root starts a statement: the host reads code holding a `;` as statements (Vue's `@click`), and its bodies may print one.
    /// A leading `{`, `function`, `class` or `let[` (an object pattern's assignment too) prints in parens, as in an expression statement.
    pub reads_as_statements: bool,
    /// The host puts the code in parens (Vue's `v-if`, an event handler's `$event => (…)`):
    /// a top-level sequence reads as one value there, and prints in parens as anywhere.
    pub in_parens: bool,
}

/// The layout an expression prints in, inside its host's delimiters.
#[derive(Clone, Copy, Debug)]
pub enum ExpressionLayout {
    /// No wrapper; the host lays it out (Vue's `{{ }}`).
    Bare,
    /// In a group.
    Group,
    /// An object or an array (a string or a template literal too, with `literals`) hugs the delimiters, in a group;
    /// another expression goes on its own indented lines when it breaks.
    Hug { literals: bool },
}

/// What an embedded JS/TS part is embedded in: what its host does with the code.
///
/// For the rules that keep what the host reads of it (see [`crate::format_to_ir`]).
/// The default neither quotes the code nor inlines it (a `<script>`, a Markdown code block).
#[derive(Clone, Copy, Debug, Default)]
pub struct JsEmbeddedIn {
    /// The code is a `"`-quoted HTML attribute's value: strings print in `'`.
    pub in_html_attribute: bool,
    /// The host puts the code in its output as written (Vue makes a render function around it):
    /// a line comment the code ends in must be closed by a line break,
    /// and a top-level sequence (`a, b`), which there reads as more than one value, is left as written
    /// (but in parens, the source's or the host's: [`ExpressionRoot::in_parens`]).
    pub inlines_code: bool,
}
