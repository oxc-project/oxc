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

/// An html-in-js placeholder ([`oxc_formatter_core::Placeholder`]) is `oxfmt_html_{depth}_{index}_`,
/// `depth` the session's dispatch depth: an html-in-js nested in this one (in its `<script>`) is deeper,
/// so neither takes the other's placeholders, and the same input always gets the same placeholders.
/// A name in every child (a tag's, an attribute's, JS's, CSS's), lowercase as CSS prints a property name.
pub const HTML_PLACEHOLDER_PREFIX: &str = "oxfmt_html_";
pub const HTML_PLACEHOLDER_SUFFIX: &str = "_";

/// `name` is an html-in-js placeholder, of any depth.
/// One in a `<script>` stands in for an expression of any precedence (`f((${a + b}) / 2)`),
/// so the parens around it in the source stay.
pub fn is_html_placeholder(name: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    name.strip_circumfix(HTML_PLACEHOLDER_PREFIX, HTML_PLACEHOLDER_SUFFIX)
        .and_then(|id| id.split_once('_'))
        .is_some_and(|(depth, index)| digits(depth) && digits(index))
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
    /// Under `semi: false`, `;` still ends each top-level statement but the last, and a lone one, an expression's too:
    /// without it, the host reads the code otherwise
    /// (Vue calls `foo`, where `foo;` runs nothing; vue-tsc types `a = 1`'s value, where `a = 1;` has none).
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
///
/// Each field but `layout` is one fact of the host, independent of the others:
/// where it ends the code, what it puts around it, how it reads it.
/// Any combination is one a host can have; a host's use only names its own.
#[derive(Clone, Copy, Debug)]
pub struct ExpressionRoot {
    pub layout: ExpressionLayout,
    /// The expression holds Tailwind classes (Vue `:class`): its strings sort as in a JSX `className`.
    pub tailwind_classes: bool,
    /// The host ends the code at the first `}}` (Vue's `{{ }}`): an object that would print one prints in parens,
    /// before a template literal substitution's `}` and, under `bracketSpacing: false`, an object's
    /// (but for the known gap, `crates/oxc_formatter/AGENTS.md` "js-in-xxx").
    pub ends_at_double_brace: bool,
    /// The host prints an object's `}` right after the code (an Angular pipe's last argument, the last value of an object):
    /// with `ends_at_double_brace`, an object ending the code prints in parens under `bracketSpacing: false`, as one ending an object's last value does.
    pub ends_before_brace: bool,
    /// The root starts a statement: the host reads code holding a `;` as statements (Vue's `@click`), and its bodies may print one.
    /// A leading `{`, `function`, `class` or `let[` (an object pattern's assignment too) prints in parens, as in an expression statement.
    pub reads_as_statements: bool,
    /// The host puts the code in parens (Vue's `v-if`, an event handler's `$event => (…)`):
    /// a top-level sequence reads as one value there, and prints in parens as anywhere.
    pub in_parens: bool,
    /// The last `host_suffix` bytes of the text are the host's, not code (an Angular alias, ` as x`):
    /// they print as written right after the root, inside its layout (`{⏎  a: 1,⏎} as x` when it hugs),
    /// and the other fields' "after the code" is after them. Left as written when the code ends in a line comment.
    pub host_suffix: usize,
    /// The host prints a `)` right after the code, in parens of its own (an Angular conditional before a pipe, prettier#5389):
    /// a conditional at the root breaks before it, as a member's object does (`(a⏎  ? b⏎  : c⏎) | p`).
    /// Unlike [`Self::in_parens`], whose parens the host reads the code in and does not print.
    pub closes_paren: bool,
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
    /// The host reads the code with a parser of its own, for a subset of JS (Angular's expressions):
    /// code holding syntax outside it, which the host reads otherwise or not at all, is left as written (`delete(x)` calls `delete`),
    /// and no trailing comma prints in a call's arguments or an arrow's params, which it rejects (`f(a,)`).
    pub template_expressions_only: bool,
}
