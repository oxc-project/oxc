use oxc_ast::ast::{ObjectExpression, ObjectPropertyKind, PropertyKey};
use oxc_formatter_core::{
    Buffer, Format, FormatContext,
    builders::{block_indent, group, soft_block_indent_with_maybe_space, space, text},
    util::{NumberFormatOptions, format_trimmed_number, is_simple_number},
    write,
};
use oxc_span::GetSpan;
use oxc_syntax::number::ToJsString;

use crate::{
    comments::{
        FormatLeadingComments, FormatSuppressedNode, FormatTrailingInsideComments,
        is_suppressed_before, write_dangling_comments,
    },
    context::JsonFormatContext,
    options::Expand,
};

use crate::separated::{TrailingSeparator, write_separated};

use super::{
    FmtJsonValue, FormatInvalidJson, JsonFormatter, arena_cow_str, format_with,
    literal::FmtJsonString,
};

pub struct FmtJsonObject<'a, 'b> {
    pub object: &'b ObjectExpression<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonObject<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        write!(f, "{");

        if self.object.properties.is_empty() {
            let dangling = f.context().comments().take_before(self.object.span.end);
            if dangling.is_empty() {
                write!(f, "}");
                return;
            }
            let inner = format_with(move |f: &mut JsonFormatter<'_, 'a>| {
                write_dangling_comments(dangling, f);
            });
            write!(f, [block_indent(&inner), "}"]);
            return;
        }

        // Collect property spans up-front for blank-line detection
        let spans: Vec<_> = self.object.properties.iter().map(oxc_span::GetSpan::span).collect();
        let properties = format_with(|f: &mut JsonFormatter<'_, 'a>| {
            write_separated(f, &spans, TrailingSeparator::Disallowed, |i, f| {
                let property = &self.object.properties[i];
                match property {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        if is_suppressed_before(f, prop.span.start) {
                            write!(f, FormatSuppressedNode(prop.span));
                        } else {
                            write!(f, FormatLeadingComments(prop.span));

                            write_object_key(&prop.key, f);
                            write!(f, [":", space()]);
                            FmtJsonValue { expression: &prop.value }.fmt(f);
                        }
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        write!(f, FormatInvalidJson(spread.span));
                    }
                }
            });

            // `properties` is non-empty here, the empty-object early return is above
            let last_end = spans.last().expect("non-empty properties").end;
            write!(
                f,
                FormatTrailingInsideComments {
                    lower_bound: last_end,
                    upper_bound: self.object.span.end,
                }
            );
        });

        // `bracketSpacing` (default: true) puts a single space inside braces when
        // the group fits on one line.
        //
        // `objectWrap` controls the multi-line shape preservation: with `Auto` (preserve),
        // an object whose source has a newline right after `{` stays expanded.
        // With `Never` (collapse), the group decides purely by width.
        // The array side does NOT do this — only objects.
        let options = f.context().options();
        let expand = match options.expand {
            Expand::Auto => {
                f.context().source_text().has_newline_after_opening_brace(self.object.span.start)
            }
            Expand::Never => false,
        };
        write!(
            f,
            [
                group(&soft_block_indent_with_maybe_space(&properties, options.bracket_spacing))
                    .should_expand(expand),
                "}"
            ]
        );
    }
}

/// Emits an object property's key, applying Prettier's `json`-parser conventions.
///
/// Identifier keys (`a:`, `null:`, `true:`) are always quoted: `"a":`/`"null":`/`"true":`.
///
/// Numeric keys are normalized via [`format_trimmed_number`];
/// whether the result is quoted follows Prettier's `shouldQuotePropertyKey` rule
/// (mirrors `language-js/print/property.js:shouldQuotePropertyKey`):
/// a number key is quoted only when its normalized form is:
/// - a "simple number"
/// - AND round-trips through `f64` losslessly.
///
/// Otherwise it's emitted bare.
/// Examples (raw → emitted):
/// - `0`       → `"0":`
/// - `0.1`     → `"0.1":`
/// - `1.0`     → `1.0:`   (round-trip via f64 gives `1`, mismatch → unquoted)
/// - `1.00000` → `1.0:`
/// - `1e2`     → `1e2:`   (not a simple number after normalization)
/// - `0xdecaf` → `0xdecaf:`
fn write_object_key<'a>(key: &PropertyKey<'a>, f: &mut JsonFormatter<'_, 'a>) {
    match key {
        PropertyKey::StringLiteral(lit) => FmtJsonString { lit: lit.as_ref() }.fmt(f),
        PropertyKey::StaticIdentifier(ident) => {
            write!(f, [text("\""), text(ident.name.as_str()), text("\"")]);
        }
        PropertyKey::NumericLiteral(lit) => {
            let raw = lit.raw.as_ref().map_or("", oxc_ast::ast::Str::as_str);
            let printed =
                format_trimmed_number(raw, NumberFormatOptions::keep_one_trailing_decimal_zero());
            let printed_str = arena_cow_str(printed, f);

            if should_quote_numeric_key(printed_str) {
                write!(f, [text("\""), text(printed_str), text("\"")]);
            } else {
                write!(f, text(printed_str));
            }
        }
        _ => write!(f, FormatInvalidJson(key.span())),
    }
}

/// Returns `true` if a normalized numeric key should be wrapped in double quotes.
///
/// Matches Prettier's `shouldQuotePropertyKey` for the `json`/`jsonc` parsers:
/// the key is quoted only when both
/// 1. the printed form is a "simple number" (`\d+` or `\d+\.\d+`), and
/// 2. it equals `String(Number(printed))` — i.e. survives an f64 round-trip without changing shape.
///
/// This keeps `0`/`0.1` quoted while leaving `1e2`/`1.0`/`0xdecaf`/
/// `999999999999999999999999999999`/`0.000...001` (which lose precision or
/// change shape under JS `String(Number)`) unquoted.
fn should_quote_numeric_key(printed: &str) -> bool {
    if !is_simple_number(printed) {
        return false;
    }
    let Ok(v) = printed.parse::<f64>() else { return false };
    v.to_js_string() == printed
}
