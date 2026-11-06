// NOTE: `inline_always`: Intentional on `FormatWith::fmt` / `FormatOnce::fmt` hot-path dispatch
#![allow(clippy::inline_always)]

mod ast_nodes;
#[cfg(feature = "detect_code_removal")]
mod detect_code_removal;
mod embed_context;
mod formatter;
mod ir_transform;
mod options;
mod parentheses;
mod print;
mod source_text;
mod utils;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{EmbeddedIr, FormatSession, Formatted, InputKind};
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;

// Internal only AST-wrapping IR primitives.
// External call-sites use the text-in `format`, `format_with_session`, `format_to_ir`,
// or the special-purpose AST-in `format_program`.
pub(crate) use crate::ast_nodes::{AstNode, AstNodes};
pub use crate::embed_context::{
    CssInJsTemplate, ExpressionLayout, ExpressionRoot, HtmlInJsTemplate, JsEmbeddedIn, JsRoot,
    MarkdownInJsTemplate, ProgramLayout, ProgramSemicolons,
};
// `JsFormatContext` is public solely as the type parameter of the returned `Formatted`;
// its methods are not part of the public contract.
pub use crate::formatter::JsFormatContext;
pub use crate::ir_transform::options::*;
pub use crate::options::*;
#[cfg(feature = "detect_code_removal")]
pub use detect_code_removal::detect_code_removal;
// Re-export the language-agnostic formatting macros from `oxc_formatter_core` so existing
// `crate::write!` / `crate::format_args!` / `crate::best_fitting!`
// call-sites in `oxc_formatter` continue to work without changes.
pub(crate) use oxc_formatter_core::{best_fitting, format_args, write};
// Internal-only re-exports so crate-local `use crate::Format;` continues to work
// without leaking these IR primitives in the public API.
pub(crate) use oxc_formatter_core::Format;

use self::formatter::prelude::tag::Label;

/// Format an entire JS/TS program from source text, text-in entry point.
///
/// # Errors
/// Returns the first parse error as an [`OxcDiagnostic`].
/// For now, any parse diagnostic is an error, even when the parser could recover.
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    options: JsFormatOptions,
) -> Result<Formatted<'a, JsFormatContext<'a>>, OxcDiagnostic> {
    // Compatibility wrapper: a service-less `PhysicalFile` session,
    // so embedded languages stay as-is.
    // Hosts that install services (oxfmt) use [`format_with_session`].
    format_with_session(
        &FormatSession::new(allocator, InputKind::PhysicalFile),
        source_text,
        source_type,
        options,
    )
}

/// Like [`format()`], but on a caller-supplied [`FormatSession`]:
/// the root whose session carries the host's `SessionServices`
/// (embedded dispatch, string embedding, Tailwind sorting).
///
/// # Errors
/// Same as [`format()`].
pub fn format_with_session<'a>(
    session: &FormatSession<'a>,
    source_text: &'a str,
    source_type: SourceType,
    options: JsFormatOptions,
) -> Result<Formatted<'a, JsFormatContext<'a>>, OxcDiagnostic> {
    let program = parse(session.allocator(), source_text, source_type)?;
    Ok(format_program_with_session(session, program, options))
}

/// Parse `source_text` and build the IR for embedding into another formatter's document
/// (dispatcher path, e.g. a `js` code fence in Markdown, an `onclick` value, Vue's `{{ }}` and directive values).
///
/// Unlike [`format_with_session`], this:
/// - allocates from the session's shared arena and `GroupId` space, so the IR lives as long as the parent's document
/// - leaves the Tailwind classes unsorted (the parent sorts them in its own batch)
/// - emits neither a BOM nor the trailing newline
///
/// `source_text` is the part of the host's document as written; it is wrapped here to parse as `root` says.
/// `embedded_in` is what the host does with the code, for the rules that keep what it reads of it.
///
/// # Errors
/// Returns the first parse error, an error when the code is not the one `root` names,
/// when a comment would not print (`v-slot="/* c */"` has no params to attach it to),
/// or, for a host that inlines the code ([`JsEmbeddedIn::inlines_code`]), when it ends in a line comment without a line break after it
/// or is a top-level sequence in no parens.
pub fn format_to_ir<'a>(
    session: &FormatSession<'a>,
    source_text: &str,
    source_type: SourceType,
    options: JsFormatOptions,
    root: JsRoot,
    embedded_in: JsEmbeddedIn,
) -> Result<EmbeddedIr<'a>, OxcDiagnostic> {
    let allocator = session.allocator();
    let (prefix, suffix) = print::root_wrapper(root);
    let wrapped = allocator.alloc_concat_strs_array([prefix, source_text, suffix]);
    let program = parse(allocator, wrapped, source_type)?;
    let ends_in_line_comment = print::check_root(
        program,
        prefix.len()..prefix.len() + source_text.len(),
        root,
        embedded_in.inlines_code,
    )?;

    let mut js_context =
        JsFormatContext::new(program.source_text, program.source_type, &program.comments, options);
    js_context.set_in_html_attribute(embedded_in.in_html_attribute);

    let node = AstNode::new(program, AstNodes::Dummy());
    let content = print::FormatRoot { program: &node, root, ends_in_line_comment };
    let (ir, js_context) = formatter::build_ir(
        js_context,
        session,
        oxc_formatter_core::Arguments::new(&[oxc_formatter_core::Argument::new(&content)]),
    );

    // Losing a comment is worse than leaving the code as written
    if !js_context.comments().unprinted_comments().is_empty() {
        return Err(OxcDiagnostic::error("Some comments in the code were not printed"));
    }
    Ok(EmbeddedIr { ir })
}

/// Format an already-parsed program, special-purpose AST-in entry point.
///
/// Most callers want [`format()`] (text-in).
/// This skips parsing and is meant for cases that already hold a `Program`.
/// e.g. perf/allocation measurement that isolates formatting from parsing, or error-tolerant harnesses.
///
/// The `program` MUST be parsed via [`parse_for_format`] (formatter parse options + JSX enabling),
/// the formatter may panic on the wrong parse options.
/// No parse happens here, so there is no error to return.
pub fn format_program<'a>(
    allocator: &'a Allocator,
    program: &'a Program<'a>,
    options: JsFormatOptions,
) -> Formatted<'a, JsFormatContext<'a>> {
    format_program_with_session(
        &FormatSession::new(allocator, InputKind::PhysicalFile),
        program,
        options,
    )
}

/// Shared AST-in funnel for [`format_with_session`] / [`format_program`].
fn format_program_with_session<'a>(
    session: &FormatSession<'a>,
    program: &'a Program<'a>,
    options: JsFormatOptions,
) -> Formatted<'a, JsFormatContext<'a>> {
    let node = AstNode::new(program, AstNodes::Dummy());
    // The only site that invokes the IR `formatter::format`
    let context =
        JsFormatContext::new(program.source_text, program.source_type, &program.comments, options);
    formatter::format(
        context,
        session,
        oxc_formatter_core::Arguments::new(&[oxc_formatter_core::Argument::new(&node)]),
    )
}

/// Parse `source_text` the way the formatter requires, for AST-in callers of [`format_program`].
///
/// Applies the formatter's parse options and JSX enabling, exactly as [`format()`] does internally,
/// so a program fed to [`format_program`] matches the text-in path.
/// Use this when you need to control parse-error handling
/// or isolate parsing from formatting (perf/allocation measurement, error-tolerant harnesses).
/// Inspect the returned `ParserReturn` (`errors` / `fatal_error`) and pass `&ret.program` to [`format_program`].
pub fn parse_for_format<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    // Always enable JSX for JavaScript source types (no syntax conflict)
    let source_type =
        if source_type.is_javascript() { source_type.with_jsx(true) } else { source_type };

    let options = ParseOptions {
        parse_regular_expression: false, // the formatter doesn't need regexes parsed
        allow_return_outside_function: true, // accept all syntax the formatter may be handed
        allow_v8_intrinsics: true,
        preserve_parens: false, // MUST be false: the formatter panics otherwise
        // The formatter does not use `Ident` hashes, but `detect_code_removal` runs semantic
        // analysis on this AST, and semantic requires hashed `Ident`s.
        enable_ident_hashes: cfg!(feature = "detect_code_removal"),
    };
    Parser::new(allocator, source_text, source_type).with_options(options).parse()
}

/// Parse `source_text` and promote the `Program` to the arena lifetime.
///
/// NOTE: Reject ANY parse diagnostic, not only `fatal_error`: we format valid code only, by design.
/// A recovered AST may be an unfaithful "fix" of the source
/// (e.g. invalid modifiers are reported but not all of them are representable),
/// so formatting it can silently rewrite what the user wrote.
/// Prettier instead formats invalid inputs through parser recovery, which also hides the error from the user;
fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Result<&'a Program<'a>, OxcDiagnostic> {
    let ret = parse_for_format(allocator, source_text, source_type);
    if let Some(err) = ret.diagnostics.into_iter().next() {
        return Err(err);
    }
    Ok(allocator.alloc(ret.program))
}

// ---

#[derive(Copy, Clone, Debug)]
pub(crate) enum JsLabels {
    MemberChain,
    /// For `ir_transform/sort_imports`
    ImportDeclaration,
    /// For `ir_transform/sort_imports`
    /// Wraps a single emitted comment so the transform can identify it without
    /// inspecting element shape (which varies by comment kind / `jsdoc` formatting).
    /// Also suppresses internal line breaks for multi-line block comments.
    Comment,
}

impl Label for JsLabels {
    fn id(&self) -> u32 {
        *self as u32
    }

    fn debug_name(&self) -> &'static str {
        match self {
            Self::MemberChain => "MemberChain",
            Self::ImportDeclaration => "ImportDeclaration",
            Self::Comment => "Comment",
        }
    }
}
