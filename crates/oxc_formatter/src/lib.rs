// NOTE: `inline_always`: Intentional on `FormatWith::fmt` / `FormatOnce::fmt` hot-path dispatch
#![allow(clippy::inline_always)]

mod ast_nodes;
#[cfg(feature = "detect_code_removal")]
mod detect_code_removal;
mod external_formatter;
mod formatter;
mod ir_transform;
mod options;
mod parentheses;
mod print;
mod source_text;
mod utils;

use oxc_allocator::Allocator;
use oxc_ast::Comment;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;

// Internal only AST-wrapping IR primitives.
// External call-sites use the text-in `format`, `format_fragment`,
// or the special-purpose AST-in `format_program`.
pub(crate) use crate::ast_nodes::{AstNode, AstNodes};
pub use crate::external_formatter::{
    EmbeddedDocFormatterCallback, EmbeddedDocResult, EmbeddedFormatterCallback, ExternalCallbacks,
    TailwindCallback,
};
// `JsFormatContext` is public solely as the type parameter of the `Formatted`
// returned by `format` / `format_fragment`.
// Its methods are not part of the public contract.
pub use crate::formatter::JsFormatContext;
pub use crate::ir_transform::options::*;
pub use crate::options::*;
#[cfg(feature = "detect_code_removal")]
pub use detect_code_removal::detect_code_removal;
// Re-export the language-agnostic formatting macros from `oxc_formatter_core` so existing
// `crate::write!` / `crate::format_args!` / `crate::best_fitting!`
// call-sites in `oxc_formatter` continue to work without changes.
pub(crate) use oxc_formatter_core::{best_fitting, format_args, write};
// Internal-only re-exports so crate-local `use crate::{Buffer, Format};` continues to work
// without leaking these IR primitives in the public API.
pub(crate) use crate::formatter::{Buffer, Format};

use self::formatter::Formatted;
use self::formatter::prelude::tag::Label;
use crate::print::{FormatFunctionParams, FormatTypeParameters};

/// Usage context a JS/TS fragment is placed in js-in-xxx.
/// Drives context-dependent formatting decisions (e.g. forced parentheses, quote style).
///
/// Currently `format_fragment()` callers pass wrapped source.
/// (Prettier's multiparser wraps the fragment before `textToDoc()`);
/// Each variant documents the expected wrap as an input contract.
/// The JS formatter knows nothing about Prettier/Vue vocabulary.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum FragmentContext {
    /// Function params in a binding-LHS position (e.g. Vue `v-for` left).
    /// Parentheses are forced when there are multiple params or a rest element.
    ///
    /// Input wrap: `function _(PARAMS) {}`
    FunctionParamsAsBindingLhs,
    /// Function params in a plain binding position (e.g. Vue `v-slot`).
    ///
    /// Input wrap: `function _(PARAMS) {}`
    FunctionParamsAsBinding,
    /// Type parameters in a standard declaration position (e.g. Vue `<script generic>`).
    ///
    /// Input wrap: `type T<PARAMS> = any`
    TypeParameters,
}

/// Format an entire JS/TS program from source text, text-in entry point.
///
/// # Errors
/// Returns the first parse error as an [`OxcDiagnostic`].
pub fn format<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    options: JsFormatOptions,
    external_callbacks: Option<ExternalCallbacks>,
) -> Result<Formatted<'a, JsFormatContext<'a>>, OxcDiagnostic> {
    let program = parse(allocator, source_text, source_type)?;
    Ok(format_program(allocator, program, options, external_callbacks))
}

/// Format a pre-wrapped JS/TS-in-xxx fragment from source text.
///
/// The caller passes source already wrapped per the [`FragmentContext`] input contract
/// (Prettier wraps it before calling `textToDoc()`);
/// This function parses it, extracts the target node, and formats it with the context-appropriate rules.
///
/// # Errors
/// Returns the first parse error,
/// or an error when the wrapped source doesn't match the shape the context expects.
pub fn format_fragment<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    options: JsFormatOptions,
    context: FragmentContext,
) -> Result<Formatted<'a, JsFormatContext<'a>>, OxcDiagnostic> {
    let program = parse(allocator, source_text, source_type)?;

    // For now, every fragment context sits inside a double-quoted attribute,
    // so single quotes avoid clashing with the surrounding attribute delimiter.
    //
    // NOTE: Since this options is just the preferred quote (not a forced),
    // this may not be enough for all cases.
    // But it seems fine for almost all cases, so leave it for now.
    let options = JsFormatOptions { quote_style: QuoteStyle::Single, ..options };

    let formatted = match context {
        FragmentContext::FunctionParamsAsBindingLhs | FragmentContext::FunctionParamsAsBinding => {
            let Some(Statement::FunctionDeclaration(func)) = program.body.first() else {
                return Err(OxcDiagnostic::error(
                    "Expected fragment wrapped as `function _(...) {}`",
                ));
            };
            let params = &*func.params;
            // Parens are forced only in the binding-LHS context with multiple params or a rest element.
            let with_parens = matches!(context, FragmentContext::FunctionParamsAsBindingLhs)
                && (1 < params.items.len() || params.rest.is_some());
            let node = AstNode::new(params, AstNodes::Dummy(), allocator);
            let content = FormatFunctionParams::new(&node, with_parens);
            format_node(
                allocator,
                options,
                &content,
                program.source_text,
                source_type,
                &program.comments,
                None,
            )
        }
        FragmentContext::TypeParameters => {
            let Some(Statement::TSTypeAliasDeclaration(decl)) = program.body.first() else {
                return Err(OxcDiagnostic::error(
                    "Expected fragment wrapped as `type T<...> = any`",
                ));
            };
            let Some(type_params) = decl.type_parameters.as_deref() else {
                return Err(OxcDiagnostic::error("Expected type parameters in wrapped fragment"));
            };
            let node = AstNode::new(type_params, AstNodes::Dummy(), allocator);
            let content = FormatTypeParameters::new(&node);
            format_node(
                allocator,
                options,
                &content,
                program.source_text,
                source_type,
                &program.comments,
                None,
            )
        }
    };

    Ok(formatted)
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
    external_callbacks: Option<ExternalCallbacks>,
) -> Formatted<'a, JsFormatContext<'a>> {
    let node = AstNode::new(program, AstNodes::Dummy(), allocator);
    format_node(
        allocator,
        options,
        &node,
        program.source_text,
        program.source_type,
        &program.comments,
        external_callbacks,
    )
}

/// Parse `source_text` the way the formatter requires, for AST-in callers of [`format_program`].
///
/// Applies the formatter's parse options and JSX enabling, exactly as [`format()`] does internally,
/// so a program fed to [`format_program`] matches the text-in path.
/// Use this when you need to control parse-error handling
/// or isolate parsing from formatting (perf/allocation measurement, error-tolerant harnesses).
/// Inspect the returned `ParserReturn` (`errors` / `panicked`) and pass `&ret.program` to [`format_program`].
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
    };
    Parser::new(allocator, source_text, source_type).with_options(options).parse()
}

/// Parse `source_text` and promote the `Program` to the arena lifetime.
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

/// Run the IR formatter over a single `Format` value and return its `Formatted` IR.
///
/// The only site that invokes the IR `formatter::format`.
/// Callers ([`format_program`] / [`format_fragment`]) construct the node (a whole-`Program` wrapper or a fragment),
/// and pass the surrounding `source_text` / `comments`.
fn format_node<'a, F: Format<'a, JsFormatContext<'a>>>(
    allocator: &'a Allocator,
    options: JsFormatOptions,
    node: &F,
    source_text: &'a str,
    source_type: SourceType,
    comments: &'a [Comment],
    external_callbacks: Option<ExternalCallbacks>,
) -> Formatted<'a, JsFormatContext<'a>> {
    let context =
        JsFormatContext::new(source_text, source_type, comments, options, external_callbacks);
    formatter::format(
        context,
        allocator,
        formatter::Arguments::new(&[formatter::Argument::new(node)]),
    )
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
    fn id(&self) -> u64 {
        *self as u64
    }

    fn debug_name(&self) -> &'static str {
        match self {
            Self::MemberChain => "MemberChain",
            Self::ImportDeclaration => "ImportDeclaration",
            Self::Comment => "Comment",
        }
    }
}
