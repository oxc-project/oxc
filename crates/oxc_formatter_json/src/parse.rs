use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_ast::{
    Comment,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::options::JsonVariant;

/// Result of [`parse_json`].
/// All borrows are arena-lifetime,
/// so the parser's `Program` does not need to be kept alive by the caller.
pub struct ParsedJson<'a> {
    /// `None` when `source` contains only comments and whitespace.
    pub expression: Option<&'a Expression<'a>>,
    /// Sorted comments. Spans are in [`Self::wrapped_source`] coordinates.
    pub comments: &'a [Comment],
    /// Either:
    /// - `"(" + source + "\n)"` (normal path)
    /// - or the original `source` (comments-only fallback).
    ///
    /// All AST and comment spans index into this.
    pub wrapped_source: &'a str,
    /// Byte offset within `wrapped_source` where the user's original source begins.
    /// `1` on the normal wrapped path (skip the leading `(`), `0` on the bare fallback.
    /// Used to map AST spans back to user-visible line/column for diagnostics.
    pub source_offset: u32,
}

/// Parse a JSON document into a single arena-resident expression.
///
/// # Errors
/// Returns an [`OxcDiagnostic`] if:
/// - `source` has syntax errors,
/// - or when `variant` rejects the comments present in `source`
///   - see [`validate_comments_for_variant`])
pub fn parse_json<'a>(
    allocator: &'a Allocator,
    source: &str,
    variant: JsonVariant,
) -> Result<ParsedJson<'a>, OxcDiagnostic> {
    // JSON object literals like `{"a":1}` are syntax errors when parsed as a JS program
    // (the leading `{` starts a `BlockStatement`),
    // so we wrap the source in `(...)` to force expression context.
    // `preserve_parens: false` keeps the wrapping `ParenthesizedExpression` out of the AST.
    // The trailing `\n` before `)` prevents a closing paren
    // from being swallowed by a trailing line comment in `source`.
    let wrapped_source: &'a str = allocator.alloc_concat_strs_array(["(", source, "\n)"]);

    let options = ParseOptions { preserve_parens: false, ..ParseOptions::default() };
    let ret =
        Parser::new(allocator, wrapped_source, SourceType::default()).with_options(options).parse();

    // If the wrapped parse fails,
    // we retry without the wrap and accept the result only when it contains no statements.
    // i.e. `source` is comments / whitespace only.
    // This lets comment-only JSON files round-trip without changing the normal path's cost.
    if !ret.errors.is_empty() || ret.panicked {
        if let Some(parsed) = try_parse_comments_only(allocator, source, options) {
            validate_comments_for_variant(variant, parsed.comments, false)?;
            return Ok(parsed);
        }

        if let Some(err) = ret.errors.into_iter().next() {
            return Err(err);
        }
        return Err(OxcDiagnostic::error("Failed to parse JSON source"));
    }

    let mut program = ret.program;

    validate_comments_for_variant(variant, &program.comments, true)?;

    // `Vec::into_arena_slice` consumes the stack-resident `Vec` header and
    // exposes its arena-resident storage as `&'a [_]`.
    // This is needed so neither the returned expression reference
    // nor the comments slice borrow from the local `program`.
    let comments =
        std::mem::replace(&mut program.comments, ArenaVec::new_in(allocator)).into_arena_slice();
    let body: &'a [Statement<'a>] =
        std::mem::replace(&mut program.body, ArenaVec::new_in(allocator)).into_arena_slice();

    // The wrap source guarantees exactly one top-level `ExpressionStatement`
    let stmt = body.first().ok_or_else(|| OxcDiagnostic::error("Empty JSON source"))?;
    let Statement::ExpressionStatement(expr_stmt) = stmt else {
        return Err(OxcDiagnostic::error("Expected a single expression at the top level"));
    };

    Ok(ParsedJson {
        expression: Some(&expr_stmt.expression),
        comments,
        wrapped_source,
        source_offset: 1,
    })
}

/// Fallback path for the wrapped-parse failure:
/// try parsing `source` as-is and accept it only when there are no statements.
/// (i.e. comments / whitespace only)
fn try_parse_comments_only<'a>(
    allocator: &'a Allocator,
    source: &str,
    options: ParseOptions,
) -> Option<ParsedJson<'a>> {
    // `Parser::new` ties `source_text` to the arena lifetime;
    // Copy `source` into the arena so the resulting comment spans index into a string that outlives `ret`.
    let bare_source: &'a str = allocator.alloc_str(source);

    let ret =
        Parser::new(allocator, bare_source, SourceType::default()).with_options(options).parse();
    if !ret.errors.is_empty() || ret.panicked || !ret.program.body.is_empty() {
        return None;
    }

    let mut program = ret.program;
    let comments =
        std::mem::replace(&mut program.comments, ArenaVec::new_in(allocator)).into_arena_slice();

    Some(ParsedJson { expression: None, comments, wrapped_source: bare_source, source_offset: 0 })
}

/// Reject comments according to Prettier's per-variant rules.
/// - `Json` and `Json5`: comments are allowed,
///   but a source consisting of only comments (no value expression) is an error
/// - `Jsonc`: comments are always allowed, including comments-only sources
/// - `JsonStringify`: comments are never allowed
fn validate_comments_for_variant(
    variant: JsonVariant,
    comments: &[Comment],
    has_expression: bool,
) -> Result<(), OxcDiagnostic> {
    if comments.is_empty() {
        return Ok(());
    }

    match variant {
        JsonVariant::Jsonc => Ok(()),
        JsonVariant::Json | JsonVariant::Json5 => {
            if has_expression {
                Ok(())
            } else {
                Err(OxcDiagnostic::error("The input is empty or contains only comments"))
            }
        }
        JsonVariant::JsonStringify => {
            if has_expression {
                Err(OxcDiagnostic::error("Comment is not allowed in JSON"))
            } else {
                Err(OxcDiagnostic::error("The input is empty or contains only comments"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use super::parse_json;
    use crate::options::JsonVariant::{Json, Json5, JsonStringify, Jsonc};

    #[test]
    fn variant_comment_acceptance() {
        // (src, variant, should_succeed)
        let cases = [
            // empty input is accepted by all variants
            ("", Json, true),
            ("", Jsonc, true),
            ("", Json5, true),
            ("", JsonStringify, true),
            ("   \n", Json, true),
            ("   \n", Jsonc, true),
            ("   \n", Json5, true),
            ("   \n", JsonStringify, true),
            // comments-only is accepted only by Jsonc
            ("// hi\n", Json, false),
            ("// hi\n", Jsonc, true),
            ("// hi\n", Json5, false),
            ("// hi\n", JsonStringify, false),
            // value + comments is rejected only by JsonStringify
            (r#"{"a":1}//c"#, Json, true),
            (r#"{"a":1}//c"#, Jsonc, true),
            (r#"{"a":1}//c"#, Json5, true),
            (r#"{"a":1}//c"#, JsonStringify, false),
        ];

        for (src, variant, should_succeed) in cases {
            let allocator = Allocator::default();
            let result = parse_json(&allocator, src, variant);
            assert_eq!(result.is_ok(), should_succeed, "src={src:?} variant={variant:?}");
        }
    }
}
