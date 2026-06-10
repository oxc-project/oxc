use oxc_ast::ast::{
    NumericLiteral, ObjectExpression, ObjectPropertyKind, PropertyKey, StringLiteral,
};
use oxc_formatter_core::{
    Buffer, Format, FormatContext,
    builders::{block_indent, group, soft_block_indent_with_maybe_space, space, text},
    spec::{format_trimmed_number, is_simple_number, normalize_string},
    write,
};
use oxc_span::GetSpan;
use oxc_syntax::{identifier::is_identifier_name_patched, number::ToJsString};

use crate::{
    comments::{
        FormatLeadingComments, FormatSuppressedNode, FormatTrailingInsideComments,
        has_line_terminator_after_skipping_comments, is_suppressed_before, write_dangling_comments,
    },
    context::JsonFormatContext,
    options::{Expand, JsonVariant, QuoteProps},
    separated::{TrailingSeparator, write_separated},
};

use super::{
    FmtJsonValue, FormatInvalidJson, JsonFormatter, arena_cow_str, format_with,
    literal::FmtJsonString, write_quoted_str,
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
            let inner = format_with(move |f| {
                write_dangling_comments(dangling, f);
            });
            write!(f, [block_indent(&inner), "}"]);
            return;
        }

        // Collect property spans up-front for blank-line detection
        let spans: Vec<_> = self.object.properties.iter().map(GetSpan::span).collect();
        let trailing =
            TrailingSeparator::when_breaking(f.context().options().allow_trailing_comma());

        let is_json5 = f.context().options().variant == JsonVariant::Json5;
        // For `quoteProps: "consistent"` (json5 only):
        // if any key in this object requires quotes, every quotable key is quoted.
        // Computed once over the siblings, threaded into each key.
        let force_quote = is_json5 && json5_consistent_force_quote(self.object, f);

        let properties = format_with(|f| {
            write_separated(f, &spans, trailing, self.object.span.end, |i, f| {
                let property = &self.object.properties[i];
                match property {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        if is_suppressed_before(f, prop.span.start) {
                            write!(f, FormatSuppressedNode(prop.span));
                        } else {
                            write!(f, FormatLeadingComments(prop.span));
                            if is_json5 {
                                json5_write_object_key(&prop.key, force_quote, f);
                            } else {
                                write_object_key(&prop.key, f);
                            }
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
                // `+ 1` skips the opening `{`.
                // Scan only within the object so the slice ends at the closing `}` at the latest.
                let after_brace = self.object.span.start + 1;
                let rest = f.context().source_text().slice_range(after_brace, self.object.span.end);
                has_line_terminator_after_skipping_comments(rest)
            }
            Expand::Never => false,
        };
        write!(
            f,
            [
                group(&soft_block_indent_with_maybe_space(
                    &properties,
                    options.bracket_spacing.value(),
                ))
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
///
/// `json5` diverges (see [`json5_write_object_key`]): keys may stay unquoted.
fn write_object_key<'a>(key: &PropertyKey<'a>, f: &mut JsonFormatter<'_, 'a>) {
    match key {
        PropertyKey::StringLiteral(lit) => FmtJsonString { lit: lit.as_ref() }.fmt(f),
        PropertyKey::StaticIdentifier(ident) => {
            write!(f, [text("\""), text(ident.name.as_str()), text("\"")]);
        }
        PropertyKey::NumericLiteral(lit) => {
            let printed_str = normalized_numeric_key(lit, f);
            if should_quote_numeric_key(printed_str) {
                write!(f, [text("\""), text(printed_str), text("\"")]);
            } else {
                write!(f, text(printed_str));
            }
        }
        _ => write!(f, FormatInvalidJson(key.span())),
    }
}

/// `json5` object-key printing, mirroring Prettier's `language-js/print/key.js`.
///
/// Unlike `json`/`jsonc` (which always quote keys), `json5` keeps keys unquoted where safe:
/// - Identifier key (`{ a: 1 }`): stays bare, unless `force_quote` (consistent) re-quotes it
/// - String key (`{ "a": 1 }`): unquoted to `a` when [`json5_unquoted_key`] allows it
///   - and `quoteProps` permits (`as-needed`, or `consistent` without `force_quote`)
///   - `preserve` and a forced `consistent` keep it quoted
/// - Number key (`{ 1: 1 }`): bare normalized number,
///   - except a forced `consistent` quotes it when it is safe to quote ([`should_quote_numeric_key`])
fn json5_write_object_key<'a>(
    key: &PropertyKey<'a>,
    force_quote: bool,
    f: &mut JsonFormatter<'_, 'a>,
) {
    match key {
        PropertyKey::StaticIdentifier(ident) => {
            let name = ident.name.as_str();
            if force_quote {
                json5_write_quoted_key(name, f);
            } else {
                write!(f, text(name));
            }
        }
        PropertyKey::StringLiteral(lit) => {
            let may_unquote = match f.context().options().quote_props {
                QuoteProps::AsNeeded => true,
                QuoteProps::Consistent => !force_quote,
                QuoteProps::Preserve => false,
            };
            if may_unquote && let Some(unquoted) = json5_unquoted_key(lit) {
                write!(f, text(unquoted));
                return;
            }
            FmtJsonString { lit: lit.as_ref() }.fmt(f);
        }
        PropertyKey::NumericLiteral(lit) => {
            let printed_str = normalized_numeric_key(lit, f);
            if force_quote && should_quote_numeric_key(printed_str) {
                json5_write_quoted_key(printed_str, f);
            } else {
                write!(f, text(printed_str));
            }
        }
        _ => write!(f, FormatInvalidJson(key.span())),
    }
}

/// For a `json5` string-literal key,
/// returns the bare (unquoted) text when it is safe to drop the quotes, mirroring Prettier's `isKeySafeToUnquote`:
/// - the raw body must equal the cooked value (no escapes that would change meaning),
/// - AND the value is a valid identifier name
///
/// NOTE: unlike other JS parsers, the `json5` parser is NOT in Prettier's number-unquote allow-list,
/// so numeric-string keys (`"1.5"`, `"0"`) stay quoted.
///
/// Returns `None` when the key must stay quoted.
fn json5_unquoted_key<'a>(lit: &StringLiteral<'a>) -> Option<&'a str> {
    let raw = lit.raw.as_ref()?.as_str();
    // Body between the quotes; bail on anything that isn't a well-formed quoted literal
    let inner = raw.get(1..raw.len().checked_sub(1)?)?;
    let value = lit.value.as_str();
    // Any escape makes the raw body differ from the cooked value -> not safe to unquote
    if inner != value {
        return None;
    }
    // `a` -> a, but `1.5` stays quoted (json5 only unquotes identifier-name keys)
    is_identifier_name_patched(value).then_some(value)
}

/// Emits `content` as a quoted `json5` key, choosing the quote per the active options
/// (Prettier's `getPreferredQuote`) and escaping the body via the shared `normalize_string`.
fn json5_write_quoted_key<'a>(content: &'a str, f: &mut JsonFormatter<'_, 'a>) {
    let quote_byte = f.context().options().preferred_quote(content);
    let normalized = normalize_string(content, quote_byte, /* quotes_will_change */ false);
    write_quoted_str(f, quote_byte, arena_cow_str(normalized, f));
}

/// Resolves Prettier's `quoteProps: "consistent"` for `object`:
/// returns `true` when at least one key requires quotes (a string key that cannot be
/// safely unquoted), which forces every quotable key in the object to be quoted.
///
/// Caller must already have established the `json5` variant, this only resolves the `quoteProps` mode.
fn json5_consistent_force_quote(object: &ObjectExpression<'_>, f: &JsonFormatter<'_, '_>) -> bool {
    if !matches!(f.context().options().quote_props, QuoteProps::Consistent) {
        return false;
    }
    object.properties.iter().any(|property| {
        matches!(
            property,
            ObjectPropertyKind::ObjectProperty(prop)
                if matches!(&prop.key, PropertyKey::StringLiteral(lit) if json5_unquoted_key(lit).is_none())
        )
    })
}

/// The normalized, arena-resident text of a numeric key (`x.00000` -> `x.0`, etc.).
/// Shared by the `json` and `json5` key writers, which differ only in the quoting decision.
fn normalized_numeric_key<'a>(lit: &NumericLiteral<'a>, f: &JsonFormatter<'_, 'a>) -> &'a str {
    let raw = lit.raw.as_ref().map_or("", oxc_ast::ast::Str::as_str);
    // JSON keeps one trailing decimal zero (`x.00000` -> `x.0`); see `format_trimmed_number`.
    let printed = format_trimmed_number(raw, /* keep_one_trailing_decimal_zero */ true);
    arena_cow_str(printed, f)
}

/// Returns `true` if a normalized numeric key should be wrapped in double quotes.
///
/// Matches Prettier's `shouldQuotePropertyKey` for the `json`/`jsonc` parsers:
/// the key is quoted only when both
/// 1. the printed form is a "simple number" (`\d+` or `\d+\.\d+`)
/// 2. and it equals `String(Number(printed))` — i.e. survives an f64 round-trip without changing shape
///
/// This keeps `0`/`0.1` quoted while leaving `1e2`/`1.0`/`0xdecaf`/ `999999999999999999999999999999`/`0.000...001`
/// (which lose precision or change shape under JS `String(Number)`) unquoted.
fn should_quote_numeric_key(printed: &str) -> bool {
    if !is_simple_number(printed) {
        return false;
    }
    let Ok(v) = printed.parse::<f64>() else { return false };
    v.to_js_string() == printed
}
