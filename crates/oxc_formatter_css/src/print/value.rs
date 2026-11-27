//! Component value printing.
//!
//! Ports Prettier's
//! - `print/comma-separated-value-group.js`
//! - `print/parenthesized-value-group.js`
//! - `print/misc.js`
//!
//! onto oxc-css-parser's flat `ComponentValue` streams
//! (which mirror `postcss-values-parser` tokens,
//! commas and solidi appear as `Delimiter` components).

use std::borrow::Cow;

use cow_utils::CowUtils;
use smallvec::SmallVec;

use oxc_allocator::Allocator;
use oxc_css_parser::{
    ast::{
        BracketBlock, Calc, CalcOperatorKind, ComponentValue, Delimiter, DelimiterKind, Dimension,
        Function, FunctionName, ImportantAnnotation, InterpolableIdent, InterpolableStr,
        LessBinaryOperation, LessEscapedStr, LessOperationOperatorKind, LessParenthesizedOperation,
        Number, SassBinaryExpression, SassBinaryOperator, SassBinaryOperatorKind,
        SassInterpolatedIdent, SassInterpolatedIdentElement, SassUnaryOperatorKind, Str, Url,
        UrlValue,
    },
    token::Token,
};
use oxc_formatter_core::{
    SourceText, arena_cow_str,
    builders::{
        empty_line, group, hard_line_break, if_group_breaks, indent, soft_block_indent,
        soft_line_break, soft_line_break_or_space, space, text, token,
    },
    format_args,
    spec::{format_trimmed_number, normalize_string},
    write,
};
use oxc_span::Span;

use crate::{
    CssFormatOptions, CssVariant,
    comments::{self, BlockCommentAfter, FormatCommentBeforeContent, FormatLineCommentSuffix},
    format::to_span,
    print::{
        CssFormatter, block_comment_end, format_with, has_comments_between, is_glued, less, scss,
        statement,
    },
};

/// Prettier's `printNumber` + `printCssNumber` (trailing `.0` removal included).
/// Normalize a CSS number for printing.
///
/// Thin wrapper over [`format_trimmed_number`] with `keep_one_trailing_decimal_zero = false`,
/// the CSS policy (`x.00000` → `x`, vs JS's `x.0`).
pub(super) fn print_css_number(raw: &str) -> Cow<'_, str> {
    format_trimmed_number(raw, false)
}

/// Prettier's `css-units-list`: lowercase → canonical casing.
///
/// Three units need special casing (`q→Q`, `hz→Hz`, `khz→kHz`);
/// everything else is identity-mapped,
/// so the bulk lives in a `phf` set whose `get_key` hands back the matching `&'static str`.
fn canonical_unit(lowercased: &str) -> Option<&'static str> {
    static IDENTITY_UNITS: phf::Set<&'static str> = phf::phf_set! {
        "cap", "ch", "cm", "cqb", "cqh", "cqi", "cqmax", "cqmin", "cqw",
        "deg", "dpcm", "dpi", "dppx", "dvb", "dvh", "dvi", "dvmax", "dvmin", "dvw",
        "em", "ex", "fr", "grad", "ic", "in", "lh", "lvb", "lvh", "lvi", "lvmax", "lvmin", "lvw",
        "mm", "ms", "pc", "pt", "px", "rad", "rcap", "rch", "rem", "rex", "ric", "rlh",
        "s", "svb", "svh", "svi", "svmax", "svmin", "svw",
        "turn", "vb", "vh", "vi", "vmax", "vmin", "vw", "x",
    };
    match lowercased {
        "q" => Some("Q"),
        "hz" => Some("Hz"),
        "khz" => Some("kHz"),
        _ => IDENTITY_UNITS.get_key(lowercased).copied(),
    }
}

pub(super) fn write_number<'a>(number: &Number<'a>, f: &mut CssFormatter<'_, 'a>) {
    let printed = print_css_number(number.raw);
    write!(f, text(arena_cow_str(&printed, f)));
}

fn write_dimension<'a>(dimension: &Dimension<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_number(&dimension.value, f);
    print_unit(dimension.unit.raw, f);
}

/// Prettier's `printUnit`.
fn print_unit<'a>(raw_unit: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let lowered = raw_unit.cow_to_ascii_lowercase();
    if let Some(canonical) = canonical_unit(&lowered) {
        write!(f, token(canonical));
    } else {
        write!(f, text(raw_unit));
    }
}

/// Prettier's `printString`.
/// Re-quote per [`CssFormatOptions::preferred_quote`]
/// unless the content contains quotes that would need extra escaping.
///
/// The actual escape rewrite delegates to [`normalize_string`].
fn print_string<'a>(raw: &'a str, options: &CssFormatOptions, allocator: &'a Allocator) -> &'a str {
    let content = &raw[1..raw.len() - 1];
    let enclosing = options.preferred_quote(content);

    if raw.as_bytes()[0] == enclosing {
        return raw;
    }

    quoted(enclosing, &normalize_string(content, enclosing, true), allocator)
}

/// `content` in `quote`s, in the arena.
pub(super) fn quoted<'a>(quote: u8, content: &str, allocator: &'a Allocator) -> &'a str {
    let quote = if quote == b'"' { "\"" } else { "'" };
    allocator.alloc_concat_strs_array([quote, content, quote])
}

pub(super) fn write_str<'a>(str: &Str<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_str_raw(str.raw, f);
}

/// Less `~"..."`: the `~` hugs its re-quoted string (see DIVERGENCES.md#less-escape-hugs-string).
/// Comments after the `~` hug it too, each followed by one space (like Prettier);
/// with a `//` comment, the source after the `~` stays as written.
pub(super) fn write_less_escaped_str<'a>(
    escaped: &LessEscapedStr<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    write!(f, "~");
    let (tilde_end, str_start) = (escaped.span.start + 1, escaped.str.span.start);
    if has_comments_between(tilde_end, str_start, f) {
        let source = f.context().source_text();
        let comments = f.context().comments().take_before(str_start);
        if comments.iter().any(|c| c.inline) {
            write!(f, text(source.slice_range(tilde_end, str_start).trim_start()));
        } else {
            for comment in comments {
                write!(f, [text(source.text_for(&comment.span)), space()]);
            }
        }
    }
    write_str(&escaped.str, f);
}

/// [`write_str`] for a raw quoted-string slice
/// (callers that only hold the source text, no AST node).
pub(super) fn write_str_raw<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    write!(f, text(print_string(raw, f.options(), f.allocator())));
}

/// [`write_str`] for attribute-selector values.
/// Prettier wraps the value in `replaceEndOfLine(..., literallineWithoutBreakParent)`:
/// an escaped literal newline inside the value prints verbatim,
/// but must not force a break in front of the selector.
pub(super) fn write_attribute_str<'a>(str: &Str<'a>, f: &mut CssFormatter<'_, 'a>) {
    write!(f, text(print_string(str.raw, f.options(), f.allocator())).without_expand_parent());
}

/// Emit a raw source slice with [`adjust_numbers_and_strings`] applied
/// (Prettier's raw selector-text print path:
/// spacing and newlines stay verbatim and nothing ever breaks on line width).
pub(super) fn write_adjusted_verbatim<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let adjusted = adjust_numbers_and_strings(raw, f.options(), f.allocator());
    write!(f, text(arena_cow_str(&adjusted, f)));
}

/// Prettier's `adjustNumbers(adjustStrings(...))` over a raw source slice:
/// strings re-quote via `printString`,
/// standalone numbers (not glued to a word part) get `printCssNumber` + lowercased/canonical unit;
/// Everything else (whitespace and newlines included) stays verbatim.
/// Used where Prettier prints raw selector text: Less mixin definitions/calls and `when` guards,
/// and a media keyword glued to its parens (`and(...)`).
///
/// NOTE: every caller's slice is a typed node
/// (`LessMixinParameters`, `LessConditions`, `LessMixinCall` args, `MediaInParens`),
/// so splicing the printed `Number` / `Dimension` / `Str` leaves into the verbatim slice is possible
/// and would retire this byte re-scan, with its comment / `url()` / hash skips.
/// However, we keep the scan for now: `oxc-css-parser` has no visitor to collect those leaves.
///
/// TODO: the slice keeps its source `/` spacing (`.m(@a/ 2)`, `when (@a/ 2 > 1)`), unlike a declaration value's
/// symmetric `/` (DIVERGENCES.md#slash-spacing); Prettier prints the slice as written too.
/// Re-spacing `/` here would re-scan bytes it does not own (escapes, detached rulesets, `./`);
/// the leaf-splicing above, printing the typed operations, is what lifts it.
pub(super) fn adjust_numbers_and_strings<'a>(
    raw: &'a str,
    options: &CssFormatOptions,
    allocator: &'a Allocator,
) -> Cow<'a, str> {
    let bytes = raw.as_bytes();
    // Only strings and numbers ever change: most params / guards have neither
    if !bytes.iter().any(|b| matches!(b, b'"' | b'\'') || b.is_ascii_digit()) {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut changed = false;
    let mut i = 0usize;

    let is_word_start = |b: u8| b == b'_' || b.is_ascii_alphabetic() || b >= 0x80;
    let is_word_char = |b: u8| b == b'_' || b == b'-' || b.is_ascii_alphanumeric() || b >= 0x80;
    // `(?:\d*\.\d+|\d+\.?)(?:e[+-]?\d+)?`:
    // returns the end index, or `start` if no number begins here.
    let scan_number = |start: usize| -> usize {
        let mut j = start;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'.' {
            let mut k = j + 1;
            while k < bytes.len() && bytes[k].is_ascii_digit() {
                k += 1;
            }
            // `\d*\.\d+` needs fraction digits; `\d+\.` allows a bare dot.
            if k > j + 1 {
                j = k;
            } else if j > start {
                j += 1;
            } else {
                return start;
            }
        } else if j == start {
            return start;
        }
        // Exponent.
        if j < bytes.len() && (bytes[j] | 0x20) == b'e' {
            let mut k = j + 1;
            if k < bytes.len() && (bytes[k] == b'+' || bytes[k] == b'-') {
                k += 1;
            }
            let digits_start = k;
            while k < bytes.len() && bytes[k].is_ascii_digit() {
                k += 1;
            }
            if k > digits_start {
                j = k;
            }
        }
        j
    };
    let scan_unit = |start: usize| -> usize {
        let mut j = start;
        while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
            j += 1;
        }
        j
    };
    // A run of numbers (+ units) glued to a word part or to each other stays raw,
    // the same rule as `is_word_glued_number`: `a.10.2` / `1.20.3` stay.
    // (Prettier re-reads the glued `.2` as standalone, `a.10.2` -> `a.100.2`, growing every pass)
    let scan_glued_numbers = |mut j: usize| -> usize {
        loop {
            let number_end = scan_number(j);
            if number_end == j {
                return j;
            }
            j = scan_unit(number_end);
        }
    };

    while i < bytes.len() {
        let b = bytes[i];
        // Comments are the user's: always verbatim.
        let comment_end = match (b, bytes.get(i + 1)) {
            (b'/', Some(b'*')) => block_comment_end(raw, i),
            (b'/', Some(b'/')) if options.variant != CssVariant::Css => {
                raw[i..].find('\n').map_or(raw.len(), |end| i + end)
            }
            _ => i,
        };
        if comment_end > i {
            out.push_str(&raw[i..comment_end]);
            i = comment_end;
            continue;
        }
        // Strings: re-quote, never adjust their content as numbers.
        if b == b'"' || b == b'\'' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b {
                j += if bytes[j] == b'\\' { 2 } else { 1 };
            }
            if j < bytes.len() {
                let string = &raw[i..=j];
                let printed = print_string(string, options, allocator);
                changed |= printed != string;
                out.push_str(printed);
                i = j + 1;
                continue;
            }
            // Unterminated: plain char.
            out.push(b as char);
            i += 1;
            continue;
        }
        // Word part `[$@]?[_a-z-￿][\w-￿-]*`; a number
        // glued to it (plus its unit) is left untouched; a hash `#[\w-]+` (`#1e0`) too.
        let word_start =
            if (b == b'$' || b == b'@') && i + 1 < bytes.len() && is_word_start(bytes[i + 1]) {
                Some(i + 1)
            } else if b == b'#' && i + 1 < bytes.len() && is_word_char(bytes[i + 1]) {
                // A hash token (`#1e0`, a hex color) is a word, never a number: `#1e0` != `#1`
                Some(i + 1)
            } else if is_word_start(b) {
                Some(i)
            } else {
                None
            };
        if let Some(ws) = word_start {
            let mut j = ws;
            while j < bytes.len() && is_word_char(bytes[j]) {
                j += 1;
            }
            // An unquoted `url(...)` is one token: verbatim (`url(img/1.50.png)`)
            let j = if raw[i..j].eq_ignore_ascii_case("url")
                && raw[j..].starts_with('(')
                && !raw[j + 1..].trim_start().starts_with(['"', '\''])
            {
                raw[j..].find(')').map_or(raw.len(), |end| j + end + 1)
            } else {
                scan_glued_numbers(j)
            };
            out.push_str(&raw[i..j]);
            i = j;
            continue;
        }
        // Standalone number (+ optional unit)
        if b.is_ascii_digit() || b == b'.' {
            let number_end = scan_number(i);
            if number_end > i {
                let unit_end = scan_unit(number_end);
                let glued_end = scan_glued_numbers(unit_end);
                if glued_end > unit_end {
                    out.push_str(&raw[i..glued_end]);
                    i = glued_end;
                    continue;
                }
                let number = &raw[i..number_end];
                let unit = &raw[number_end..unit_end];
                let lowered_unit = unit.cow_to_ascii_lowercase();
                let canonical = canonical_unit(&lowered_unit);
                if unit.is_empty() || lowered_unit == "n" || canonical.is_some() {
                    let printed = print_css_number(number);
                    changed |= matches!(printed, Cow::Owned(_));
                    out.push_str(&printed);
                    if !unit.is_empty() {
                        let printed_unit = canonical.unwrap_or(lowered_unit.as_ref());
                        changed |= printed_unit != unit;
                        out.push_str(printed_unit);
                    }
                } else {
                    out.push_str(&raw[i..unit_end]);
                }
                i = unit_end;
                continue;
            }
        }
        // Plain char (always ASCII here; `is_word_start` claims all `>= 0x80` bytes)
        let ch = raw[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    if changed { Cow::Owned(out) } else { Cow::Borrowed(raw) }
}

/// Re-quotes the OUTER quotes of a raw quoted string per `singleQuote`,
/// keeping the content (e.g. `#{...}` interpolation) verbatim.
pub(super) fn write_requoted_verbatim<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    if raw.len() < 2 || !(raw.starts_with('\'') || raw.starts_with('"')) {
        write!(f, text(raw));
        return;
    }

    let content = &raw[1..raw.len() - 1];
    // Interpolated strings whose content has quote characters keep their original quotes
    // (the inner quotes confuse the preference count),
    // but bare quoted numbers inside `#{}` are normalized (postcss sees them as unquoted numbers).
    if content.contains("#{") && (content.contains('"') || content.contains('\'')) {
        // The outer quote re-appearing inside the content splits the string in postcss;
        // every piece gets requoted to the preferred quote.
        let outer = raw.as_bytes()[0] as char;
        let preferred = f.options().single_quote.as_char();
        if outer != preferred && content.contains(outer) && !content.contains(preferred) {
            let replaced = raw.cow_replace(outer, preferred.encode_utf8(&mut [0; 4]));
            let normalized = normalize_quoted_numbers(&replaced);
            write!(f, text(f.allocator().alloc_str(&normalized)));
            return;
        }
        match normalize_quoted_numbers(raw) {
            Cow::Borrowed(_) => write!(f, text(raw)),
            Cow::Owned(normalized) => write!(f, text(f.allocator().alloc_str(&normalized))),
        }
        return;
    }

    let enclosing = f.options().preferred_quote(content);
    if raw.as_bytes()[0] == enclosing {
        write!(f, text(raw));
    } else {
        write!(f, text(quoted(enclosing, content, f.allocator())));
    }
}

/// Normalizes `".5"`-style quoted bare numbers inside an interpolated string.
/// Returns `Cow::Borrowed` when nothing changed.
fn normalize_quoted_numbers(raw: &str) -> Cow<'_, str> {
    let bytes = raw.as_bytes();
    let mut out = String::new();
    // Start of the pending verbatim span.
    // Quotes are ASCII, so the byte scan is safe;
    // everything between rewrites copies span-wise (UTF-8 included).
    let mut copied = 0;
    let mut i = 1;
    // Content bounds inside the outer quotes
    let end = raw.len() - 1;
    while i < end {
        let b = bytes[i];
        if (b == b'"' || b == b'\'') && i + 1 < end {
            // Find the matching close within the content
            if let Some(close_rel) = raw[i + 1..end].find(b as char) {
                let close = i + 1 + close_rel;
                let inner = &raw[i + 1..close];
                if !inner.is_empty() && inner.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
                    if let Cow::Owned(printed) = print_css_number(inner) {
                        out.push_str(&raw[copied..=i]);
                        out.push_str(&printed);
                        copied = close;
                    }
                    i = close + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    if out.is_empty() {
        return Cow::Borrowed(raw);
    }
    out.push_str(&raw[copied..]);
    Cow::Owned(out)
}

fn is_wide_keyword(value: &str) -> bool {
    value.eq_ignore_ascii_case("initial")
        || value.eq_ignore_ascii_case("inherit")
        || value.eq_ignore_ascii_case("unset")
        || value.eq_ignore_ascii_case("revert")
}

pub(super) fn token_depth_delta(token: &Token<'_>) -> i32 {
    match token {
        Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_) => 1,
        Token::RParen(_) | Token::RBracket(_) | Token::RBrace(_) => -1,
        _ => 0,
    }
}

fn is_comma(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::Delimiter(Delimiter { kind: DelimiterKind::Comma, .. }))
}

fn is_solidus(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::Delimiter(Delimiter { kind: DelimiterKind::Solidus, .. }))
}

/// A `/` outside a math function keeps the source's choice, made symmetric (DIVERGENCES.md#slash-spacing):
/// tight when glued on both sides, otherwise one space on both sides.
pub(super) fn is_slash_spaced(
    left: oxc_css_parser::pos::Span,
    slash: oxc_css_parser::pos::Span,
    right: oxc_css_parser::pos::Span,
) -> bool {
    !is_glued(left, slash) || !is_glued(slash, right)
}

/// [`is_slash_spaced`] for a component list:
/// the separator BEFORE `values[i]` when it or `values[i - 1]` is a slash, `None` otherwise.
fn slash_separator(
    values: &[ComponentValue<'_>],
    i: usize,
    is_slash: impl Fn(&ComponentValue<'_>) -> bool,
) -> Option<Separator> {
    let gap = |k: usize| values.get(k).is_some_and(|v| !is_glued(values[k - 1].span(), v.span()));
    let curr_is_slash = is_slash(&values[i]);
    if !curr_is_slash && !is_slash(&values[i - 1]) {
        return None;
    }
    // A leading `/` makes the stream path-like (`-fb-url(/a/b.png)`): it never spaces
    if is_slash(&values[0]) && (i == 1 || !gap(i)) {
        return Some(Separator::Tight);
    }
    // The slash's own two gaps: `values[i]` is the slash, or the one before it
    let slash = if curr_is_slash { i } else { i - 1 };
    // A trailing `/` has no right gap
    let spaced = gap(slash) || gap(slash + 1);
    Some(match (spaced, curr_is_slash) {
        (false, _) => Separator::Tight,
        // A spaced `/` glues to its left operand and breaks after
        (true, true) => Separator::Space,
        (true, false) => Separator::Line,
    })
}

fn is_semicolon(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::Delimiter(Delimiter { kind: DelimiterKind::Semicolon, .. }))
}

fn is_word_like(value: &ComponentValue<'_>) -> bool {
    matches!(
        value,
        ComponentValue::InterpolableIdent(_)
            | ComponentValue::LessVariable(_)
            | ComponentValue::SassVariable(_)
    )
}

fn is_func_like(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::Function(_) | ComponentValue::Url(_))
}

/// `sandstone.10` / `theme(spacing.2.5)` / `1+1+1+1` / `calc(100%+2px)`:
/// per CSS tokenization the glued `.10` / `+1` / `+2px` are numbers,
/// but postcss lexes the whole contiguous run as ONE word
/// (an xstyled / tailwind-theme token, or plugin-processed pseudo-math).
///
/// Such a number stays glued and prints raw (`Separator::Word`),
/// and so does the number heading the run (`is_glued_number_head`): `1.20.3` stays.
/// `.10` must not normalize to `0.1`, `+1` must not gain a space.
/// Two glued number-ish values can never be valid CSS,
/// so a number-ish neighbor is as sure a word sign as a word-like one.
/// (The raw-slice twin of this rule lives in `adjust_numbers_and_strings`; keep them in sync.)
fn is_word_glued_number(values: &[ComponentValue<'_>], i: usize) -> bool {
    i > 0
        && is_number_ish(&values[i])
        && (is_word_like(&values[i - 1]) || is_number_ish(&values[i - 1]))
        && is_glued(values[i - 1].span(), values[i].span())
}

/// A number followed by a glued number: the head of a word run (`1.20` of `1.20.3`).
fn is_glued_number_head(values: &[ComponentValue<'_>], i: usize) -> bool {
    is_number_ish(&values[i]) && i + 1 < values.len() && is_word_glued_number(values, i + 1)
}

fn is_number_ish(value: &ComponentValue<'_>) -> bool {
    matches!(
        value,
        ComponentValue::Number(_) | ComponentValue::Dimension(_) | ComponentValue::Percentage(_)
    )
}

pub(super) fn function_name_text<'a>(func: &Function<'a>) -> &'a str {
    match &func.name {
        FunctionName::Ident(InterpolableIdent::Literal(ident)) => ident.raw,
        _ => "",
    }
}

/// Layout context for a value being printed.
#[derive(Clone, Copy, Default)]
pub(super) struct ValueContext<'a> {
    /// Lowercased property name of the enclosing declaration, if any.
    pub decl_prop: Option<&'a str>,
    /// The value never breaks: `composes` (Prettier's `removeLines`) and
    /// media feature values (Prettier's `media-value` is flat text).
    pub no_break: bool,
    /// SCSS map printed in key position: stays inline (Prettier's `isKey`).
    pub map_key: bool,
    /// A parenthesized COMMA list here breaks one item per line with a trailing comma
    /// (only as a direct map-item value, `isSCSSMapItemNode` needs key-value pairs in the group or its grandparent;
    /// non-comma-list contents never take the break, the comma would change their meaning).
    pub paren_break: bool,
    /// SCSS maps here always break (`$var:` values, function args, map items).
    pub map_break: bool,
    /// Comments before this bound are flushed as wrapped fill items
    /// at the end of the outermost comma group (declaration tail).
    pub tail_bound: Option<u32>,
    /// Inside `url(...)`: colons stay tight (`url(fbglyph:cross-outline)`).
    pub in_url: bool,
    /// Inside function/include arguments
    /// (comment-slot rules differ, and the grid line-structure rule is top-level only, see `is_grid`).
    pub in_args: bool,
    /// A multi-line `raws.between` was printed before the value:
    /// Prettier's printer counts its full width, so the first trailing comment always wraps.
    pub tail_break: bool,
    /// Less variable declaration value (`@var: ...`):
    /// Prettier's `shouldPrecededBySoftline` matches `css-decl` ONLY,
    /// so the value fill starts on the colon line and never breaks right after the colon.
    pub no_leading_softline: bool,
}

impl ValueContext<'_> {
    fn is_grid(&self) -> bool {
        // Only the declaration's TOP-LEVEL value gets the grid line-structure treatment;
        // inside function arguments (`repeat()`, `minmax()`, `theme()`) the normal separator rules apply.
        !self.in_args
            && self.decl_prop.is_some_and(|p| p == "grid" || p.starts_with("grid-template"))
    }
}

/// Splits a flat component stream at top-level commas.
///
/// A raw `Token::Comma` splits too (postcss value-parses raw token streams the same way):
/// a declaration value the typed grammar rejected falls back to raw tokens,
/// and its comma list must still count as a multi-value list
/// (`background-position: 0 0, 0 spacing(tight), ...` breaks one per line).
/// Raw parens keep their inner commas out of the split.
/// Each group is paired with the start offset of the comma that follows it (`None` for the last group):
/// comments between a group and its comma stay BEFORE the comma (`a /* c */, b`),
/// so group printers need the boundary.
/// Inline up to 4 groups: most values have one.
pub(super) fn split_comma_groups<'b, 'a>(
    values: &'b [ComponentValue<'a>],
) -> SmallVec<[(&'b [ComponentValue<'a>], Option<u32>); 4]> {
    let mut groups = SmallVec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (i, v) in values.iter().enumerate() {
        if let ComponentValue::TokenWithSpan(tok) = v {
            if depth == 0 && matches!(&tok.token, Token::Comma(_)) {
                groups.push((&values[start..i], Some(v.span().start)));
                start = i + 1;
            } else {
                depth += token_depth_delta(&tok.token);
            }
            continue;
        }
        if is_comma(v) {
            groups.push((&values[start..i], Some(v.span().start)));
            start = i + 1;
        }
    }
    // A trailing comma would leave an empty last group; Prettier drops it
    if groups.is_empty() || start < values.len() {
        groups.push((&values[start..], None));
    }
    groups
}

/// Mirrors Prettier's top-level declaration value printing
/// (`value-root` -> `value-paren_group` without parens).
pub(super) fn write_declaration_value<'a>(
    values: &[ComponentValue<'a>],
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    // A lone SCSS list/map IS the value (`$var: 1px 2px, 3px;`)
    if values.len() == 1
        && matches!(values[0], ComponentValue::SassList(_) | ComponentValue::SassMap(_))
    {
        scss::write_top_level_value(&values[0], ctx, f);
        return;
    }

    let groups = split_comma_groups(values);

    if groups.len() == 1 {
        // Flattened to a single comma group
        write_comma_group(groups[0].0, ctx, f);
        return;
    }

    // Multiple comma groups:
    // `shouldBreakList` forces one per line when any group has multiple elements
    // (a real `value-comma_group` survives flattening, comments count as members)
    // and the property is not a custom property.
    let value_start = values[0].span().start;
    let value_end = values[values.len() - 1].span().end;
    let has_comments =
        f.context().comments().iter_before(value_end).any(|c| c.span.start >= value_start);
    let force_hard_line = !ctx.decl_prop.is_some_and(|p| p.starts_with("--"))
        && (groups.iter().any(|(g, _)| comma_group_is_multi(g)) || has_comments);

    write_value_groups(&groups, ctx, force_hard_line, false, f);
}

/// Does this comma group count as a `value-comma_group` for Prettier's `shouldBreakList`?
/// Multi-element groups do.
/// A signed item (`Arial, -apple-system`, `0, -1px`) does not, though postcss-values splits it into an operator + word
/// past the first position (DIVERGENCES.md#signed-value-resplit).
pub(super) fn comma_group_is_multi(group: &[ComponentValue<'_>]) -> bool {
    // Less `~'...'` lexes as TWO postcss-values nodes (`~` word + string),
    // so it is a real `value-comma_group` in ANY position.
    group.len() > 1 || matches!(group.first(), Some(ComponentValue::LessEscapedStr(_)))
}

/// A group's separator comma:
/// a block comment between the group and its comma stays before the comma (`a /* c */, b`),
/// a same-line `//` there rides past it (`a // c\n,` -> `a, // c`: the comma cannot follow a `//`),
/// a same-line `//` right after it stays on its line (`a, // c`);
/// own-line comments and anything past those lead the next group (the line-boundary invariant).
pub(super) fn write_group_comma(comma_start: Option<u32>, f: &mut CssFormatter<'_, '_>) {
    let Some(comma) = comma_start else {
        write!(f, ",");
        return;
    };
    let line_comment_deferred = flush_comments_before_separator(comma, f);
    write!(f, ",");
    flush_line_comment_after_comma(comma, f);
    if line_comment_deferred {
        // Flushes the deferred `//` right behind the comma and ends the line.
        // A hardline, not `expand_parent`:
        // inside a `fill` the latter would also break the separator BEFORE this entry (`@each $k\n in a, // c`),
        // the former keeps it (`@each $k in a, // c`).
        write!(f, hard_line_break());
    }
}

/// The same-line comments before a separator at `separator_start`:
/// block comments glue before it, a `//` rides a `line_suffix` past it (its line ends after the separator).
/// Own-line comments stay pending, they lead what follows the separator.
/// Returns `true` when a `//` was deferred: the caller must end the line right after the separator.
fn flush_comments_before_separator(separator_start: u32, f: &mut CssFormatter<'_, '_>) -> bool {
    flush_trailing_comments_while(0, separator_start, LineCommentAs::Suffix, f, |_, c, source| {
        !comment_is_own_line(c, source)
    })
}

/// How a trailing `//` is emitted by [`flush_trailing_comments_while`].
#[derive(Clone, Copy)]
enum LineCommentAs {
    /// In place, ending the line (`FormatCommentBeforeContent`)
    InPlace,
    /// A `line_suffix`: the token written next (a separator) precedes it on the line;
    /// the caller ends the line after that token
    Suffix,
}

/// A `//` on the line of the comma just written (`a, // c`, `a, /* b */ // c`) stays on that line,
/// the line-boundary invariant; the block comments glued before it on that line come along.
/// Prettier's CSS moves them below as the next element's leading comments instead
/// (its other printers keep them, and so do the other formatter crates), see DIVERGENCES.md#line-comment-after-comma.
pub(super) fn flush_line_comment_after_comma(comma_start: u32, f: &mut CssFormatter<'_, '_>) {
    let comma_end = comma_start + 1;
    let source = f.context().source_text();
    let run_end = line_comment_run_end(comma_end, f.context().comments().iter_remaining(), source);
    if let Some(run_end) = run_end {
        flush_same_line_comments(comma_end, run_end, f);
    }
}

/// Emits the comments glued right after `prev_end` (only spaces/tabs between, up to `upper_bound`)
/// as ` /* c */` / ` // c`, stopping after a `//` (it ends the line).
/// Unlike [`flush_same_line_comments`] a comment further along the line, past other tokens, stays pending.
pub(super) fn flush_glued_comments(prev_end: u32, upper_bound: u32, f: &mut CssFormatter<'_, '_>) {
    flush_trailing_comments_while(
        prev_end,
        upper_bound,
        LineCommentAs::InPlace,
        f,
        |prev_end, comment, source| {
            source.all_bytes_match(prev_end, comment.span.start, |b| matches!(b, b' ' | b'\t'))
        },
    );
}

/// A word with the comments around it: own-line ones before it lead it (`flush_value_comments`),
/// the run glued after it trails it (`flush_glued_comments`, bounded by `upper_bound`).
/// The shape of every structured prelude word (`@import` parts, `@media` words, `@supports` terms, ...).
pub(super) fn write_with_comments<'a>(
    span: Span,
    upper_bound: u32,
    f: &mut CssFormatter<'_, 'a>,
    write: impl FnOnce(&mut CssFormatter<'_, 'a>),
) {
    flush_value_comments(span.start, f);
    write(f);
    flush_glued_comments(span.end, upper_bound, f);
}

/// Emits pending comments before `upper_bound` as ` /* c */` / ` // c` while `keep` accepts them
/// (`prev_end` is the previous comment's end from the second one on), stopping after a `//`.
/// Returns `true` when it stopped at a `//`.
fn flush_trailing_comments_while(
    mut prev_end: u32,
    upper_bound: u32,
    line_comment_as: LineCommentAs,
    f: &mut CssFormatter<'_, '_>,
    keep: impl Fn(u32, comments::CssComment, SourceText<'_>) -> bool,
) -> bool {
    loop {
        let Some(comment) = f.context().comments().peek() else { return false };
        let source = f.context().source_text();
        if comment.span.start < prev_end
            || comment.span.end > upper_bound
            || !keep(prev_end, comment, source)
        {
            return false;
        }
        f.context().comments().take_before(comment.span.end);
        if comment.inline && matches!(line_comment_as, LineCommentAs::Suffix) {
            write!(f, FormatLineCommentSuffix::new(comment).with_leading_space());
            return true;
        }
        write!(f, [space(), FormatCommentBeforeContent::new(comment, BlockCommentAfter::None)]);
        if comment.inline {
            return true;
        }
        prev_end = comment.span.end;
    }
}

/// Comments between a prelude ending at `prelude_end` and its `{` at `block_start`:
/// a run glued to the prelude stays on its line (`@page :first // c`, the `//` ends it so `{` starts the next),
/// own-line ones keep their line before the `{` (DIVERGENCES.md#own-line-comment-before-block).
/// Either way the caller's space before `{` is safe: the printer drops it at the start of a line.
pub(super) fn write_comments_before_block(
    prelude_end: u32,
    block_start: u32,
    f: &mut CssFormatter<'_, '_>,
) {
    flush_glued_comments(prelude_end, block_start, f);
    let own_line = f.context().comments().take_before(block_start);
    if own_line.is_empty() {
        return;
    }
    write!(f, hard_line_break());
    let source = f.context().source_text();
    let last = own_line.len() - 1;
    for (i, &comment) in own_line.iter().enumerate() {
        // The `{` stays on the last comment's line when it is there (`/* c */ {`), as before a JS block
        let after = if i == last
            && comments::classify_gap(source.bytes_range(comment.span.end, block_start))
                == comments::Gap::None
        {
            BlockCommentAfter::None
        } else {
            BlockCommentAfter::HardLine
        };
        write!(f, FormatCommentBeforeContent::new(comment, after));
    }
}

/// End of the comment run glued right after `prev_end` (only spaces/tabs between) that ends in a `//`;
/// `None` when no `//` is glued there (a lone block comment leads the next element instead).
/// `comments` are the pending comments in source order.
pub(super) fn line_comment_run_end(
    prev_end: u32,
    comments: impl Iterator<Item = comments::CssComment>,
    source: SourceText<'_>,
) -> Option<u32> {
    let mut prev_end = prev_end;
    for comment in comments {
        if comment.span.start < prev_end
            || !source.all_bytes_match(prev_end, comment.span.start, |b| matches!(b, b' ' | b'\t'))
        {
            return None;
        }
        if comment.inline {
            return Some(comment.span.end);
        }
        prev_end = comment.span.end;
    }
    None
}

/// Top-level comma-group list layout, shared between flat component streams and SCSS comma lists.
/// Each group comes paired with the start of its trailing comma (see [`split_comma_groups`] / [`write_group_comma`]).
/// `keep_trailing_comma`: the last group's comma is written too (see [`scss::is_single_item_list`]);
/// only a one-group list sets it, so it matters to the fill branch alone.
pub(super) fn write_value_groups<'a>(
    groups: &[(&[ComponentValue<'a>], Option<u32>)],
    ctx: ValueContext<'a>,
    force_hard_line: bool,
    keep_trailing_comma: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    // A single comma group never forces (it isn't a list)
    let force_hard_line = force_hard_line && groups.len() > 1;
    if force_hard_line {
        let body = format_with(|f: &mut CssFormatter<'_, 'a>| {
            write!(f, hard_line_break());
            for (i, &(group_values, comma)) in groups.iter().enumerate() {
                if i > 0 {
                    write!(f, hard_line_break());
                }
                let is_last = i + 1 == groups.len();
                // The declaration tail belongs to the LAST group only
                let gctx = if is_last { ctx } else { ValueContext { tail_bound: None, ..ctx } };
                // Comments between groups lead the group they precede
                // (a `//` on the previous comma's line already went with the comma, see `write_group_comma`);
                // own-line ones keep their line.
                let lead: &'a [comments::CssComment] = group_values
                    .first()
                    .map_or(&[], |first| f.context().comments().take_before(first.span().start));
                if lead.is_empty() {
                    write_comma_group(group_values, gctx, f);
                } else if i == 0 {
                    let source = f.context().source_text();
                    for &comment in lead {
                        let own_line = comment_is_own_line(comment, source);
                        let block_after = if own_line {
                            BlockCommentAfter::HardLine
                        } else {
                            BlockCommentAfter::Space
                        };
                        write!(f, FormatCommentBeforeContent::new(comment, block_after));
                    }
                    write_comma_group(group_values, gctx, f);
                } else {
                    // Prettier-fill the comments with the group they lead.
                    // Simulated with static widths:
                    // the last comment stays on the line when the group's first chunk fits after it
                    // (its glued run, the comma too when that is the whole group);
                    // the group's fit must NOT include its declaration tail, which our entry would.
                    let width = u32::from(f.options().line_width.value());
                    let source = f.context().source_text();
                    let chunk_len = run_end(group_values, 0, gctx, source);
                    let group_w = group_values[chunk_len - 1].span().end
                        - group_values[0].span().start
                        + u32::from(!is_last && chunk_len == group_values.len());
                    let mut x = 4u32; // hardline indent under the value
                    for (k, &comment) in lead.iter().enumerate() {
                        x += comment.span.end - comment.span.start;
                        let next_w = lead.get(k + 1).map_or(group_w, |c| c.span.end - c.span.start);
                        let fits = x + 1 + next_w <= width;
                        let block_after = if fits {
                            BlockCommentAfter::Space
                        } else {
                            BlockCommentAfter::HardLine
                        };
                        write!(f, FormatCommentBeforeContent::new(comment, block_after));
                        x = if !comment.inline && fits { x + 1 } else { 4 };
                    }
                    write_comma_group(group_values, gctx, f);
                }
                if !is_last {
                    write_group_comma(comma, f);
                }
            }
        });
        write!(f, indent(&body));
    } else {
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            if !ctx.no_leading_softline {
                write!(f, soft_line_break());
            }
            let mut filler = f.fill();
            for (i, &(group_values, comma)) in groups.iter().enumerate() {
                let is_last = i + 1 == groups.len();
                // The declaration tail belongs to the LAST group only
                let gctx = if is_last { ctx } else { ValueContext { tail_bound: None, ..ctx } };
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write_comma_group(group_values, gctx, f);
                    if !is_last || keep_trailing_comma {
                        write_group_comma(comma, f);
                    }
                });
                filler.entry(&soft_line_break_or_space(), &content);
            }
            filler.finish();
        });
        write!(f, indent(&group(&body)));
    }
}

/// `true` when only whitespace including a newline precedes the comment.
pub(super) fn comment_is_own_line(comment: comments::CssComment, source: SourceText<'_>) -> bool {
    let bytes = source.as_bytes();
    let mut i = comment.span.start as usize;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b'\n' | b'\r' => return true,
            b' ' | b'\t' => {}
            _ => return false,
        }
    }
    true
}

/// Comments between the last element and its closing `)`.
///
/// - Same-line: glued to the element, `a /* c */)`
///   - A `//` also forces the group open, so `)` lands on the next line
/// - Own-line `//`: keeps its own line (DIVERGENCES.md#own-line-trailing-comment-keeps-line)
/// - Own-line block comment: glued too, like any value-level block comment
///   - it carries no line semantics, and keeping it own-line would pin a wrapped layout
///   - With `body_hard_broken` it keeps its own line instead:
///     the body already prints one element per line, so there is no wrapped layout to pin.
/// - After a `//`: always a new line (the `//` ends its own line).
/// - `after_content == false` (`[ /* c */ ]`): the comments ARE the body, nothing to space them off.
///
/// Returns true when the last comment was a `//`.
fn write_paren_tail_comments(
    tail: &[comments::CssComment],
    body_hard_broken: bool,
    after_content: bool,
    f: &mut CssFormatter<'_, '_>,
) -> bool {
    let source = f.context().source_text();
    let mut after_inline = false;
    for (i, &comment) in tail.iter().enumerate() {
        let keeps_line = comment.inline || body_hard_broken;
        if after_inline {
            // Already on a fresh line
        } else if keeps_line && comment_is_own_line(comment, source) {
            write!(f, hard_line_break());
        } else if i > 0 || after_content {
            write!(f, space());
        }
        write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
        after_inline = comment.inline;
    }
    after_inline
}

/// Drains the comments before `r_paren` and writes them per [`write_paren_tail_comments`].
pub(super) fn flush_paren_tail_comments(
    r_paren: u32,
    body_hard_broken: bool,
    f: &mut CssFormatter<'_, '_>,
) {
    let tail = f.context().comments().take_before(r_paren);
    write_paren_tail_comments(tail, body_hard_broken, /* after_content */ true, f);
}

/// `[a, b]` is a Sass list like `(a, b)`: same construct, same output
/// (Prettier prints the brackets as words of the OUTER comma list, see DIVERGENCES.md#bracket-list-layout).
fn write_bracket_block<'a>(
    bracket: &BracketBlock<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let groups = split_comma_groups(&bracket.value);
    let r_bracket = bracket.span().end.saturating_sub(1);
    // `[1,]` is a COMMA list, `[1]` a space list (dart-sass `list.separator`):
    // the source comma is the value, like `(1,)` (see `scss::is_single_item_list`).
    // A multi-element list drops it: the separator is comma either way.
    let keep_trailing_comma = groups.len() == 1;
    // Comments before `]` are the tail's; nothing past the bracket belongs inside.
    let inner_ctx = ValueContext { paren_break: false, tail_bound: None, ..ctx };
    // Grid values never break (Prettier keeps them on one line, see `grid_did_break`):
    // a line name stays `[top-end]` however long the declaration.
    let flat = ctx.no_break || ctx.is_grid();
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        for (i, &(group, comma)) in groups.iter().enumerate() {
            if i > 0 {
                if flat {
                    write!(f, " ");
                } else {
                    write!(f, soft_line_break_or_space());
                }
            }
            write_comma_group(group, inner_ctx, f);
            if i + 1 < groups.len() || (keep_trailing_comma && comma.is_some()) {
                write_group_comma(comma, f);
            }
        }
        let tail: &'a [comments::CssComment] = f.context().comments().take_before(r_bracket);
        // A trailing `//` ends its own line, so `]` stays out of it even with no group to expand here.
        write_paren_tail_comments(
            tail,
            /* body_hard_broken */ false,
            !bracket.value.is_empty(),
            f,
        );
    });
    if flat {
        write!(f, ["[", body, "]"]);
    } else {
        write!(
            f,
            group(&format_args!(
                "[",
                indent(&format_args!(soft_line_break(), body)),
                soft_line_break(),
                "]"
            ))
        );
    }
}

/// A raw source token that owns its leading comments (`/* c */ $param`, `/* c */ ==`).
pub(super) fn write_text_with_leading_comments(span: Span, f: &mut CssFormatter<'_, '_>) {
    flush_value_comments(span.start, f);
    let source = f.context().source_text();
    write!(f, text(source.text_for(&span)));
}

/// A value printed verbatim from source (raw custom-property text, `progid:`, ...):
/// its leading comments are flushed as usual,
/// and the ones inside the slice are claimed so the terminator-tail pass does not print them again.
pub(super) fn write_verbatim_value(span: Span, f: &mut CssFormatter<'_, '_>) {
    write_text_with_leading_comments(span, f);
    let _ = f.context().comments().take_before(span.end);
}

/// Emits pending comments that precede `upper_bound` inline
/// (`//` comments force a break after themselves and expand the parent).
/// Returns true when the last emitted comment was a `//` comment.
pub(super) fn flush_value_comments(upper_bound: u32, f: &mut CssFormatter<'_, '_>) -> bool {
    let mut last_inline = false;
    for &comment in f.context().comments().take_before(upper_bound) {
        write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::Space));
        last_inline = comment.inline;
    }
    last_inline
}

/// Emits pending comments that sit on the same line as the just-printed
/// content ending at `prev_end` (` // c` / ` /* c */`), up to `upper_bound`.
/// Container tails pass their own end as the bound,
/// so a same-line comment belonging to a FOLLOWING statement is never pulled inside.
pub(super) fn flush_same_line_comments(
    prev_end: u32,
    upper_bound: u32,
    f: &mut CssFormatter<'_, '_>,
) {
    flush_trailing_comments_while(
        prev_end,
        upper_bound,
        LineCommentAs::InPlace,
        f,
        |_, c, source| !comment_is_own_line(c, source),
    );
}

/// Emits pending comments before `upper_bound` as ` /* c */` suffixes
/// (used after the last value component, before `;` / `!important`).
pub(super) fn flush_trailing_value_comments(upper_bound: u32, f: &mut CssFormatter<'_, '_>) {
    for &comment in f.context().comments().take_before(upper_bound) {
        write!(f, " ");
        write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
    }
}

/// `!important`, normalized (`!  IMPORTANT` too), with a comment between `!` and the identifier kept there:
/// `! /* c */ important` (DIVERGENCES.md#important-comment-run).
/// Comments BEFORE the `!` are the caller's (a trailing flush, or the value list's separator).
pub(super) fn write_important_annotation(
    important: &ImportantAnnotation<'_>,
    f: &mut CssFormatter<'_, '_>,
) {
    let ident_start = important.ident.span.start;
    let has_inner_comment = f
        .context()
        .comments()
        .peek()
        .is_some_and(|c| c.span.start >= important.span.start && c.span.end <= ident_start);
    if !has_inner_comment {
        write!(f, "!important");
        return;
    }
    write!(f, "!");
    flush_trailing_value_comments(ident_start, f);
    write!(f, [space(), "important"]);
}

/// A trailing `!important` (declaration, Less mixin call):
/// the comments before it, then ` !important`.
pub(super) fn write_trailing_important(
    important: &ImportantAnnotation<'_>,
    f: &mut CssFormatter<'_, '_>,
) {
    flush_trailing_value_comments(important.span.start, f);
    write!(f, space());
    write_important_annotation(important, f);
}

/// A value that is exactly one sass interpolation (`--p: #{fn(...)};`).
/// Prettier's value parser splits `#{` into multiple fill chunks,
/// and a fill chunk's fit ignores the rest of the line (`;`, trailing comments).
/// Unlike a bare func (`calc`), whose group fit counts them.
/// `write_comma_group` routes this shape through a fill entry for the chunk-isolated fit,
/// and `write_declaration` exempts it from tail-comment counting.
pub(super) fn is_single_sass_interpolation(values: &[ComponentValue<'_>]) -> bool {
    values.len() == 1
        && matches!(
            values[0],
            ComponentValue::InterpolableIdent(InterpolableIdent::SassInterpolated(_))
        )
}

/// One bare element of a comma list (`/* c */ 1,`).
/// Leading comments belong to the element and flush BEFORE any group/indent it opens
/// (same reason as the head flush in [`write_comma_group`]);
/// a bare scalar has no comma group of its own to do this.
pub(super) fn write_list_element<'a>(
    el: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    flush_value_comments(el.span().start, f);
    write_component_value(el, ctx, f);
}

/// Mirrors Prettier's `printCommaSeparatedValueGroup`.
/// Joins components with `line`, except for pairs that must stay tight.
pub(super) fn write_comma_group<'a>(
    values: &[ComponentValue<'a>],
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    if values.is_empty() {
        return;
    }
    // Comments leading the FIRST value flush at this level, before any group/indent opens:
    // `//` comment's forced hardline inside `indent(&body)` would drop the value one level deeper
    // and break the fill run after it.
    flush_value_comments(values[0].span().start, f);
    // Prettier's `flattenGroups`:
    // a single-element comma group collapses to the element itself (no extra group/indent level).
    if values.len() == 1 && ctx.tail_bound.is_none() {
        // EXCEPT a sass interpolation:
        // route through a fill entry to get the chunk-isolated fit (see `is_single_sass_interpolation`).
        if is_single_sass_interpolation(values) {
            let value = &values[0];
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write_component_value(value, ctx, f);
            });
            let mut filler = f.fill();
            filler.entry(&soft_line_break_or_space(), &content);
            filler.finish();
        } else {
            write_component_value(&values[0], ctx, f);
        }
        return;
    }
    // A single value WITH a tail bound routes through the fill below
    // so its trailing comments wrap.
    let source = f.context().source_text();
    let upper_bound = values[values.len() - 1].span().end;
    // Grid values that break across source lines start on their own line
    // (Prettier's `didBreak` + `parts.unshift(hardline)`),
    // but breaks caused by inline comments in the gap don't count.
    // (the comment-hardline branch runs before the grid check in Prettier)
    let grid_did_break = ctx.is_grid()
        && (1..values.len()).any(|i| {
            separator_between(values, i, ctx, source) == Separator::Hard && {
                let prev_end = values[i - 1].span().end;
                let start = values[i].span().start;
                !f.context()
                    .comments()
                    .iter_before(upper_bound)
                    .any(|c| c.span.start >= prev_end && c.span.end <= start)
            }
        });
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        if grid_did_break {
            write!(f, hard_line_break());
        }
        // Snapshot of pending comments inside the value
        // (for separator decisions; consumption happens inside the entries).
        let pending: Vec<comments::CssComment> =
            f.context().comments().iter_before(upper_bound).copied().collect();
        let tail_bound = ctx.tail_bound;
        let ctx = ValueContext { tail_bound: None, ..ctx };
        let tail_comments: Vec<comments::CssComment> = tail_bound.map_or_default(|bound| {
            f.context()
                .comments()
                .iter_before(bound)
                .filter(|c| c.span.start >= upper_bound)
                .copied()
                .collect()
        });
        let mut filler = f.fill();
        let mut i = 0;
        while i < values.len() {
            // Separator between the previous component and this one
            // (ignored by the fill builder for the first entry).
            let mut sep =
                if i == 0 { Separator::Line } else { separator_between(values, i, ctx, source) };
            // An inline comment trailing the PREVIOUS run forces a break
            if i > 0 {
                let prev_end = values[i - 1].span().end;
                let start = values[i].span().start;
                if pending.iter().any(|c| {
                    c.inline
                        && c.span.start >= prev_end
                        && c.span.end <= start
                        && !comment_is_own_line(*c, source)
                }) {
                    sep = Separator::Hard;
                }
            }
            let run_end = run_end(values, i, ctx, source);
            let run_start = i;
            let run = &values[i..run_end];
            let start = values[i].span().start;
            let run_end_pos = values[run_end - 1].span().end;
            let is_last_run = run_end == values.len();
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                flush_value_comments(start, f);
                for (j, v) in run.iter().enumerate() {
                    let sep =
                        (j > 0).then(|| separator_between(values, run_start + j, ctx, source));
                    if sep == Some(Separator::Space) {
                        write!(f, " ");
                    }
                    let is_word_continuation = sep == Some(Separator::Word);
                    // A comment inside the run stays where it is (`1px /* c */ / 2`), never past the next member;
                    // a pair kept tight regardless of its gap still spaces around it (`if($a, 1 /* c */ ; 2)`)
                    if j > 0
                        && f.context()
                            .comments()
                            .peek()
                            .is_some_and(|c| c.span.end <= v.span().start)
                    {
                        if sep != Some(Separator::Space) {
                            write!(f, " ");
                        }
                        flush_value_comments(v.span().start, f);
                    }
                    // A word continuation or a glued number head prints raw, not normalized
                    // (`sandstone.10`, `[0.50]`, `1.20.3` must survive as-is).
                    if is_word_continuation || is_glued_number_head(values, run_start + j) {
                        write!(f, text(source.text_for(&to_span(v.span()))));
                        continue;
                    }
                    write_component_value(v, ctx, f);
                }
                // Same-line `//` comments stay attached to this entry
                // (a separate entry would break with the expanded group).
                if !is_last_run {
                    let next_start = values[run_end].span().start;
                    while let Some(comment) = f.context().comments().peek() {
                        if !comment.inline
                            || comment.span.start < run_end_pos
                            || comment.span.end > next_start
                            || comment_is_own_line(comment, f.context().source_text())
                        {
                            break;
                        }
                        f.context().comments().take_before(comment.span.end);
                        write!(f, " ");
                        write!(
                            f,
                            FormatCommentBeforeContent::new(comment, BlockCommentAfter::None)
                        );
                    }
                }
            });
            match sep {
                Separator::Hard => {
                    filler.entry(&hard_line_break(), &content);
                }
                _ if ctx.no_break => {
                    filler.entry(&text(" "), &content);
                }
                Separator::SoftBreak => {
                    filler.entry(&soft_line_break(), &content);
                }
                _ => {
                    filler.entry(&soft_line_break_or_space(), &content);
                }
            }
            // Block comments between runs become their own fill items, own-line ones included:
            // keeping those own-line would freeze a previously wrapped layout (= not idempotent).
            if !is_last_run {
                let next_start = values[run_end].span().start;
                for &comment in pending.iter().filter(|c| {
                    !c.inline && c.span.start >= run_end_pos && c.span.end <= next_start
                }) {
                    let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        if f.context().comments().peek().is_some_and(|c| c.span == comment.span) {
                            f.context().comments().take_before(comment.span.end);
                            write!(
                                f,
                                FormatCommentBeforeContent::new(comment, BlockCommentAfter::None)
                            );
                        }
                    });
                    filler.entry(&soft_line_break_or_space(), &entry);
                }
            }
            i = run_end;
        }
        // Declaration-tail comments wrap as fill items
        for (k, &comment) in tail_comments.iter().enumerate() {
            let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                f.context().comments().take_before(comment.span.end);
                write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
            });
            if k == 0 && ctx.tail_break {
                filler.entry(&hard_line_break(), &entry);
            } else {
                filler.entry(&soft_line_break_or_space(), &entry);
            }
        }
        filler.finish();
    });
    write!(f, group(&indent(&body)));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Separator {
    /// No separator (components merged into one run).
    Tight,
    /// `Tight`, AND the right component prints raw from source:
    /// the pair continues ONE postcss word (`sandstone.10`, `foo[0.50]`).
    Word,
    /// Plain space, no break opportunity (word before a math operator).
    Space,
    /// Breakable, no space when flat (placeholder glued to a paren group).
    SoftBreak,
    /// Breakable space.
    Line,
    /// Forced line break (grid rows).
    Hard,
}

/// Math-operator-ish raw token (`+`, `-`, `*`, `%`, `/`) parsed as a fallback token.
fn math_op_token(value: &ComponentValue<'_>) -> bool {
    if let ComponentValue::TokenWithSpan(token) = value {
        matches!(
            &token.token,
            Token::Plus(_)
                | Token::Minus(_)
                | Token::Asterisk(_)
                | Token::Percent(_)
                | Token::Solidus(_)
        )
    } else {
        false
    }
}

fn raw_token<'b, 'a>(value: &'b ComponentValue<'a>) -> Option<&'b Token<'a>> {
    if let ComponentValue::TokenWithSpan(token) = value { Some(&token.token) } else { None }
}

/// A lone `-` that oxc-css-parser reads as a Less keyword, not an operator or a sign (`1 -"t"`).
fn is_less_keyword_minus(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::InterpolableIdent(InterpolableIdent::Literal(id)) if id.raw == "-")
}

/// The end of the run starting at `values[start]`: tight / non-breaking-space components merge into one fill entry.
fn run_end(
    values: &[ComponentValue<'_>],
    start: usize,
    ctx: ValueContext<'_>,
    source: SourceText<'_>,
) -> usize {
    (start + 1..values.len())
        .find(|&i| {
            !matches!(
                separator_between(values, i, ctx, source),
                Separator::Tight | Separator::Word | Separator::Space
            )
        })
        .unwrap_or(values.len())
}

/// Decides the separator BEFORE `values[i]` (i >= 1).
fn separator_between(
    values: &[ComponentValue<'_>],
    i: usize,
    ctx: ValueContext<'_>,
    source: SourceText<'_>,
) -> Separator {
    let sep = base_separator(values, i, ctx);
    // Grid: preserve source line structure:
    // Prettier emits a hardline where the source breaks and a PLAIN SPACE otherwise
    // (never re-wraps a single-line grid value, however long).
    // Only LAYOUT outcomes are overridden:
    // a glued pair (`Tight`/`Word`) is part of ONE postcss word (`sandstone.10`, `1fr/2fr`)
    // and survives verbatim in grid values too, grid never splits what other properties keep glued.
    if ctx.is_grid() && matches!(sep, Separator::Line | Separator::Space) {
        let prev_end = values[i - 1].span().end;
        let curr_start = values[i].span().start;
        return if source.bytes_contain(prev_end, curr_start, b'\n') {
            Separator::Hard
        } else {
            Separator::Space
        };
    }
    sep
}

/// The separator rules WITHOUT the grid line-structure override (`separator_between` applies that on top).
fn base_separator(values: &[ComponentValue<'_>], i: usize, ctx: ValueContext<'_>) -> Separator {
    let prev = &values[i - 1];
    let curr = &values[i];
    // `hasEmptyRawBefore(iNextNode)`
    let gap_empty = is_glued(prev.span(), curr.span());
    if let Some(sep) = slash_separator(values, i, is_solidus) {
        return sep;
    }

    // Less lookups: `@var [@result]` loses the gap (`var [@lookup]` rule).
    // A `[...]` continues a lookup chain only when the chain HEAD is a Less value (`@config[@key][@key2]`);
    // bracket runs without one are NOT lookups (CSS grid line names: `[row1-end]` newline `[row2-start]` stays two values).
    if matches!(curr, ComponentValue::BracketBlock(_))
        && values[..i]
            .iter()
            .rev()
            .find(|v| !matches!(v, ComponentValue::BracketBlock(_)))
            .is_some_and(|head| {
                matches!(
                    head,
                    ComponentValue::LessVariable(_)
                        | ComponentValue::LessNamespaceValue(_)
                        | ComponentValue::LessMixinCall(_)
                )
            })
    {
        return Separator::Tight;
    }

    // A source-glued `[...]` is part of ONE postcss word
    // (`theme(fontSize.af-md[0])`, `foo[0]bar`, `10px[0]`): keep the glue and print verbatim;
    // a source gap (`af-md [0]`) keeps two words.
    // Less lookup chains never reach here (the rule above wins),
    // so their brackets still print structurally.
    // NOTE: Prettier re-splits SOME glued neighbors (`var(--x)[0]` -> `var(--x) [0]`,
    // a word-lexing artifact of `[` not extending a word across `)`);
    // one gap-based rule never adds a space the source doesn't have.
    if gap_empty
        && (matches!(curr, ComponentValue::BracketBlock(_))
            || matches!(prev, ComponentValue::BracketBlock(_)))
    {
        return Separator::Word;
    }

    // Raw token punctuation:
    // `:` hugs left and spaces right, braces hug their contents (custom-property JSON-ish values).
    {
        // SCSS `if(...)` separates branches with `;` parsed as a `Delimiter`
        // (not a raw `Token::Semicolon`); like every `;` it hugs the value on its left,
        // dropping any source space before it (Prettier #19384).
        if is_semicolon(curr) {
            return Separator::Tight;
        }
        if matches!(
            raw_token(curr),
            Some(
                Token::Colon(_)
                    | Token::RBrace(_)
                    | Token::Comma(_)
                    | Token::RParen(_)
                    | Token::RBracket(_)
                    | Token::Semicolon(_)
            )
        ) || matches!(
            raw_token(prev),
            Some(Token::LBrace(_) | Token::LParen(_) | Token::LBracket(_))
        ) {
            return Separator::Tight;
        }
        if matches!(raw_token(prev), Some(Token::Colon(_) | Token::Comma(_))) {
            // No space after `:` inside `url(...)`
            if ctx.in_url && matches!(raw_token(prev), Some(Token::Colon(_))) {
                return Separator::Tight;
            }
            return Separator::Space;
        }
        // `*` is never glued (Prettier excludes multiplication from the tight rules),
        // except Tailwind's `ident-*` pattern.
        if matches!(raw_token(curr), Some(Token::Asterisk(_)))
            || matches!(raw_token(prev), Some(Token::Asterisk(_)))
        {
            if gap_empty
                && matches!(prev, ComponentValue::InterpolableIdent(InterpolableIdent::Literal(id)) if id.raw.ends_with('-'))
            {
                return Separator::Tight;
            }
            return Separator::Line;
        }
        // Token-level division: the same rule as the typed `/`
        if let Some(sep) =
            slash_separator(values, i, |v| matches!(raw_token(v), Some(Token::Solidus(_))))
        {
            return sep;
        }
    }

    // Prettier: an at-word placeholder glued to a paren group gets a `softline`
    // (`${fn}(30px)` may break BEFORE the parens, keeping the paren group intact on the next line).
    if gap_empty
        && matches!(prev, ComponentValue::Placeholder(_))
        && matches!(
            curr,
            ComponentValue::SassParenthesizedExpression(_) | ComponentValue::SassMap(_)
        )
    {
        return Separator::SoftBreak;
    }

    // Raw token fallbacks (postcss-values would have produced operator/word tokens):
    // keep gap-free neighbors tight (`10em+12em`, `-fb-url(/a/b)`).
    // So does the Less keyword `-` (`1 -"t"`, `1-`): a space after it would make it an operator
    if gap_empty
        && (matches!(prev, ComponentValue::TokenWithSpan(_) | ComponentValue::Placeholder(_))
            || matches!(curr, ComponentValue::TokenWithSpan(_) | ComponentValue::Placeholder(_))
            || matches!(prev, ComponentValue::Delimiter(_))
            || matches!(curr, ComponentValue::Delimiter(_))
            || is_less_keyword_minus(prev)
            || is_less_keyword_minus(curr))
    {
        return Separator::Tight;
    }

    // A word-glued number is part of ONE postcss word (`sandstone.10`); see `is_word_glued_number`.
    if gap_empty && is_word_glued_number(values, i) {
        return Separator::Word;
    }

    // postcss-values lexes `1#{$var}` as ONE word:
    // a neighbor glued to an interpolated ident stays glued.
    {
        let interpolated = |v: &ComponentValue<'_>| {
            matches!(
                v,
                ComponentValue::InterpolableIdent(
                    InterpolableIdent::SassInterpolated(_) | InterpolableIdent::LessInterpolated(_),
                )
            )
        };
        if gap_empty && (interpolated(prev) || interpolated(curr)) {
            return Separator::Tight;
        }
    }

    // Math operators with surrounding spaces:
    // `a + b` keeps the space before the operator non-breaking and breaks after it
    // (Prettier's `isNextMathOperator` merge).
    if math_op_token(curr) {
        return Separator::Space;
    }

    Separator::Line
}

/// Dispatch a single component value.
/// Leading comments are the CALLER's job (`write_list_element` / `write_comma_group`):
/// flushing here would land a `//` hardline inside the group/indent an arm opens.
pub(super) fn write_component_value<'a>(
    value: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    match value {
        ComponentValue::Delimiter(delimiter) => match delimiter.kind {
            DelimiterKind::Comma => write!(f, ","),
            DelimiterKind::Solidus => write!(f, "/"),
            DelimiterKind::Semicolon => write!(f, ";"),
        },
        ComponentValue::Number(number) => write_number(number, f),
        ComponentValue::Dimension(dimension) => write_dimension(dimension, f),
        ComponentValue::Percentage(percentage) => {
            write_number(&percentage.value, f);
            write!(f, "%");
        }
        // The `/` spacing of a declaration value (DIVERGENCES.md#slash-spacing)
        ComponentValue::Ratio(ratio) => {
            let spaced =
                is_slash_spaced(ratio.numerator.span, ratio.solidus_span, ratio.denominator.span);
            write_number(&ratio.numerator, f);
            if spaced {
                write!(f, space());
            }
            // A comment between the parts stays there
            flush_value_comments(ratio.solidus_span.start, f);
            write!(f, "/");
            if spaced {
                write!(f, space());
            }
            flush_value_comments(ratio.denominator.span.start, f);
            write_number(&ratio.denominator, f);
        }
        ComponentValue::HexColor(hex) => {
            write!(f, "#");
            let lower = hex.raw.cow_to_ascii_lowercase();
            write!(f, text(arena_cow_str(&lower, f)));
        }
        ComponentValue::InterpolableIdent(InterpolableIdent::Literal(ident)) => {
            if is_wide_keyword(ident.raw) {
                let lower = ident.raw.cow_to_ascii_lowercase();
                write!(f, text(arena_cow_str(&lower, f)));
            } else {
                write!(f, text(ident.raw));
            }
        }
        ComponentValue::InterpolableIdent(InterpolableIdent::SassInterpolated(interp)) => {
            // Prettier's value parser splits `#{fn(...)}` into
            // multiple fill chunks wrapped in the value's `group(indent(fill))`.
            // So a breaking call inside the interpolation carries ONE extra indent level.
            // ```
            // --prop: #{fn(
            //     $args,
            //     $args2,
            //   )};
            // ```
            // `args` +2 and `)}` +1, both relative to the property.
            let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write_sass_interpolated_ident(interp, ctx, f);
            });
            write!(f, indent(&body));
        }
        ComponentValue::InterpolableStr(InterpolableStr::Literal(str)) => {
            write_str(str, f);
        }
        ComponentValue::InterpolableStr(istr) => {
            let span = to_span(istr.span());
            write_requoted_verbatim(source.text_for(&span), f);
        }
        ComponentValue::Function(func) => write_function(func, ctx, f),
        ComponentValue::Calc(calc) => write_calc(calc, ctx, f),
        ComponentValue::CalcParenthesized(paren) => {
            write_parenthesized(&paren.expr, paren.span.end - 1, ctx, f);
        }
        ComponentValue::SassMap(map) => scss::write_sass_map(map, ctx, f),
        ComponentValue::SassList(list) => scss::write_sass_list(list, ctx, f),
        ComponentValue::SassParenthesizedExpression(paren) => {
            // `$var: ((a, b), (c, d))` / map item values: one item per line.
            // ONLY a comma-separated list takes this break:
            // the trailing comma is a semantic no-op there and NOWHERE else.
            //
            // NOTE: `(x,)` is a single-element list in Sass,
            // so adding it to `($a + $b)` / `(a b)` / `(-$a)` changes the value
            // and `2 * ($a + $b,)` fails to compile.
            // (Prettier does all of these; #19091 exempted single-node scalars only)
            //
            // The reverse also holds: `("a",)` keeps its comma regardless of `trailing_commas`
            // (see `is_single_item_list`; prettier#19928 preserves it too).
            let comma_list = match &*paren.expr {
                ComponentValue::SassList(list) if list.comma_spans.is_some() => Some(list),
                _ => None,
            };
            let keep_trailing_comma = comma_list.is_some_and(scss::is_single_item_list);
            // A single-item list of ONE component value (`k: ("a",)`, `k: ((1 2),)`, `k: (fn(1),)`)
            // is not a map item and stays flat (`isSCSSMapItemNode`'s "parenthesized scalar" exit):
            // only a multi-token item (`k: (1 2,)`, `k: ($a + $b,)`) is a `value-comma_group` there.
            let single_scalar_item = keep_trailing_comma
                && comma_list.is_some_and(|list| {
                    !matches!(
                        list.elements[0],
                        ComponentValue::SassList(_) | ComponentValue::SassBinaryExpression(_)
                    )
                });
            let inner_ctx = ValueContext { paren_break: false, ..ctx };
            let trailing = f.options().allow_trailing_comma();
            let r_paren = paren.span().end.saturating_sub(1);
            if ctx.paren_break
                && !single_scalar_item
                && let Some(list) = comma_list
            {
                // Comma-list map-item values always break (`isSCSSMapItemNode`),
                // with a trailing comma per option.
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, hard_line_break());
                    for (i, el) in list.elements.iter().enumerate() {
                        if i > 0 {
                            write!(f, hard_line_break());
                        }
                        write_list_element(el, inner_ctx, f);
                        let is_last = i + 1 == list.elements.len();
                        if !is_last || keep_trailing_comma {
                            write_group_comma(scss::list_comma_start(list, i), f);
                        } else if trailing {
                            write!(f, ",");
                        }
                    }
                    flush_paren_tail_comments(r_paren, /* body_hard_broken */ true, f);
                });
                write!(f, ["(", indent(&body), hard_line_break(), ")"]);
                return;
            }
            let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write!(f, soft_line_break());
                // A comma list directly inside parens shares the paren's
                // indent (no nested group level).
                if let Some(list) = comma_list {
                    for (i, el) in list.elements.iter().enumerate() {
                        if i > 0 {
                            write!(f, soft_line_break_or_space());
                        }
                        write_list_element(el, inner_ctx, f);
                        let is_last = i + 1 == list.elements.len();
                        if !is_last || keep_trailing_comma {
                            write_group_comma(scss::list_comma_start(list, i), f);
                        } else if trailing && ctx.map_key {
                            // `ctx.paren_break` cannot be set here,
                            // a comma list with it takes the hard-break branch above.
                            write!(f, if_group_breaks(&text(",")));
                        }
                    }
                } else {
                    write_list_element(&paren.expr, inner_ctx, f);
                }
                flush_paren_tail_comments(r_paren, /* body_hard_broken */ false, f);
            });
            write!(
                f,
                group(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, "(");
                    write!(f, indent(&body));
                    write!(f, soft_line_break());
                    write!(f, ")");
                }))
            );
        }
        ComponentValue::SassUnaryExpression(unary) => {
            let keyword_not = matches!(unary.op.kind, SassUnaryOperatorKind::Not);
            match unary.op.kind {
                SassUnaryOperatorKind::Plus => write!(f, "+"),
                SassUnaryOperatorKind::Minus => write!(f, "-"),
                SassUnaryOperatorKind::Not => write!(f, ["not", " "]),
            }
            // Prettier glues a unary `+`/`-` to its operand even across a source gap
            // (`- ( $x / 2 )` → `-($x / 2)`, `- 2deg` → `-2deg`).
            // EXCEPT before a function call, where the gap is preserved (`- pow(2, 2)` stays spaced).
            // `not` already wrote its space.
            if !keyword_not
                && is_func_like(&unary.expr)
                && !is_glued(unary.op.span, unary.expr.span())
            {
                write!(f, " ");
            }
            write_component_value(&unary.expr, ctx, f);
        }
        ComponentValue::SassBinaryExpression(binary) => write_sass_binary(binary, ctx, f),
        ComponentValue::SassKeywordArgument(kw) => {
            // Block values (maps/parens) hug the colon without pair indent
            if matches!(
                &*kw.value,
                ComponentValue::SassMap(_) | ComponentValue::SassParenthesizedExpression(_)
            ) {
                let name_span = to_span(kw.name.span());
                write!(f, [text(source.text_for(&name_span)), ":", " "]);
                write_component_value(&kw.value, ctx, f);
                return;
            }
            // Only a value that IS the paren group is a map item;
            // a paren nested in a math expression (`$foo: 2 * ($bar + $baz)`) never breaks (Prettier #18530).
            let ctx = ValueContext { paren_break: false, ..ctx };
            // `$name: value` may break after the colon when too long
            let pair = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                let mut filler = f.fill();
                let name_span = to_span(kw.name.span());
                let key = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, [text(f.context().source_text().text_for(&name_span)), ":"]);
                });
                let val = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write_component_value(&kw.value, ctx, f);
                });
                filler.entry(&soft_line_break_or_space(), &key);
                filler.entry(&soft_line_break_or_space(), &val);
                filler.finish();
            });
            write!(f, group(&indent(&pair)));
        }
        ComponentValue::SassNestingDeclaration(nesting) => {
            statement::write_block(&nesting.block, f);
        }
        ComponentValue::SassArbitraryArgument(arg) => {
            write_component_value(&arg.value, ctx, f);
            write!(f, "...");
        }
        ComponentValue::Url(url) => write_url(url, f),
        // Less lookups / nth bracket blocks: `[...]` hugs its contents
        ComponentValue::BracketBlock(bracket) => write_bracket_block(bracket, ctx, f),
        ComponentValue::UnicodeRange(range) => {
            let span = to_span(range.span());
            write!(f, text(source.text_for(&span)));
        }
        ComponentValue::LessEscapedStr(escaped) => write_less_escaped_str(escaped, f),
        // Less arithmetic: a flat operator fill (break after the operator), mirroring Prettier.
        // Safe to restructure because `oxc-css-parser` only ASTs a whitespace-followed
        // `+`/`-` as a binary operator (see `write_less_binary_operation`).
        ComponentValue::LessBinaryOperation(op) => write_less_binary_operation(op, ctx, f),
        ComponentValue::LessParenthesizedOperation(paren) => {
            write_parenthesized(&paren.operation, paren.span.end - 1, ctx, f);
        }
        // A css-in-js placeholder becomes a typed marker the host replaces with `${expr}`
        ComponentValue::Placeholder(placeholder) => {
            super::write_placeholder(placeholder, f);
        }
        // postcss-simple-vars `$name` reference (Css mode with the opt-in)
        ComponentValue::PostcssSimpleVar(variable) => {
            super::postcss_simple_vars::write_postcss_simple_var(variable, f);
        }
        // A mid-value or variable-value `!important` (the trailing one of a declaration is `Declaration::important`)
        ComponentValue::ImportantAnnotation(important) => write_important_annotation(important, f),
        // Everything else (Sass/Less constructs, interpolations, token fallbacks):
        // print the source verbatim until ported structurally.
        _ => {
            if less::write_less_component_value(value, f) {
                return;
            }
            // The slice holds any comment inside the node; claim it so no later flush repeats it
            write_verbatim_value(to_span(value.span()), f);
        }
    }
}

/// Flattens a left-leaning SCSS binary chain (`a == 0 and b == 0`) into
/// `(operand, following-op)` pairs, mirroring postcss's flat token stream.
fn flatten_sass_binary<'b, 'a>(
    expr: &'b ComponentValue<'a>,
    out: &mut Vec<(&'b ComponentValue<'a>, Option<&'b SassBinaryOperator>)>,
) {
    if let ComponentValue::SassBinaryExpression(binary) = expr {
        flatten_sass_binary(&binary.left, out);
        // Attach this operator to the last operand collected
        if let Some(last) = out.last_mut() {
            last.1 = Some(&binary.op);
        }
        flatten_sass_binary(&binary.right, out);
    } else {
        out.push((expr, None));
    }
}

/// SCSS binary expression: a flat fill of `operand op` entries;
/// spaces follow the source around each operator, breaking after operators.
fn write_sass_binary<'a>(
    binary: &SassBinaryExpression<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let mut parts = Vec::new();
    flatten_sass_binary(&binary.left, &mut parts);
    if let Some(last) = parts.last_mut() {
        last.1 = Some(&binary.op);
    }
    flatten_sass_binary(&binary.right, &mut parts);

    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let mut filler = f.fill();
        let mut i = 0;
        while i < parts.len() {
            // Merge a run while operators are tight to the next operand.
            // `*` never merges: Prettier excludes multiplication from the tight rules (`$m*100` → `$m * 100`).
            // Word-like operands force spaces on BOTH sides even when glued (`$a+"str"` → `$a + "str"`),
            // so they end the run too.
            let mut run_end = i;
            while let Some(op) = parts[run_end].1 {
                if matches!(op.kind, SassBinaryOperatorKind::Multiply) {
                    break;
                }
                let op_span = to_span(op.span);
                let next = &parts[run_end + 1].0;
                let operand = &parts[run_end].0;
                // A glued `/` stays in this run, a spaced one ends it (spaced both sides below)
                if matches!(op.kind, SassBinaryOperatorKind::Division) {
                    if is_slash_spaced(operand.span(), op.span, next.span()) {
                        break;
                    }
                    run_end += 1;
                    continue;
                }
                let next_start = next.span().start;
                if matches!(source.text_for(&op_span), "+" | "-")
                    && (is_word_like(operand)
                        || is_func_like(operand)
                        || is_word_like(next)
                        || is_func_like(next))
                {
                    // An asymmetric `+`/`-` (whitespace BEFORE, glued AFTER) is a signed operand in postcss-values lexing,
                    // Prettier keeps it glued (`$a -$b` stays `$a -$b`, NOT `$a - $b`).
                    // It matters for the ambiguous Sass `margin: -$a -$b` list/subtraction case (dart-sass deprecates the binary reading).
                    // Merge it into this run so no fill space lands after the operator.
                    if operand.span().end != op_span.start && op_span.end == next_start {
                        run_end += 1;
                        continue;
                    }
                    break;
                }
                if op_span.end == next_start {
                    run_end += 1;
                } else {
                    break;
                }
            }
            let run = &parts[i..=run_end];
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                for (j, (operand, op)) in run.iter().enumerate() {
                    // A comment between the operands and operators stays there
                    flush_value_comments(operand.span().start, f);
                    write_component_value(operand, ctx, f);
                    if let Some(op) = op {
                        let op_span = to_span(op.span);
                        let operand_end = operand.span().end;
                        let next_start = run.get(j + 1).map(|(next, _)| next.span().start);
                        // `op(paren)` fuses like a postcss function token,
                        // which always gets a space before it;
                        // word-like operands force spaces (`$a + $b` even when glued).
                        let fuses_paren = next_start == Some(op_span.end);
                        let op_text = source.text_for(&op_span);
                        let wordish = matches!(op_text, "+" | "-")
                            && (is_word_like(operand)
                                || is_func_like(operand)
                                || run.get(j + 1).is_some_and(|(next, _)| {
                                    is_word_like(next) || is_func_like(next)
                                }));
                        // Only a spaced `/` ends a run (see the run-merge rule above)
                        let division_spaces = op_text == "/" && j + 1 == run.len();
                        if operand_end != op_span.start
                            || wordish
                            || division_spaces
                            // `*` always spaces (it also ends its run above)
                            || op_text == "*"
                            // A glued `/` stays glued whatever follows it (DIVERGENCES.md#slash-spacing)
                            || (fuses_paren
                                && op_text != "/"
                                && run.get(j + 1).is_some_and(|(next, _)| {
                                    matches!(next, ComponentValue::SassParenthesizedExpression(_))
                                }))
                        {
                            write!(f, " ");
                        }
                        write_text_with_leading_comments(op_span, f);
                    }
                }
            });
            // Media feature values (`ValueContext::no_break`) are flat text and never break
            // (Prettier's `media-value`).
            if ctx.no_break {
                filler.entry(&text(" "), &content);
            } else {
                filler.entry(&soft_line_break_or_space(), &content);
            }
            i = run_end + 1;
        }
        filler.finish();
    });
    write!(f, group(&indent(&body)));
}

/// A piece of a flat operator fill (`write_calc`, `write_less_binary_operation`).
enum OpPiece<'b, 'a> {
    Operand(&'b ComponentValue<'a>),
    /// A Less parenthesized operation: its own nested group (it can break inside its parens)
    Paren(&'b LessParenthesizedOperation<'a>),
    /// An operator, printed as its source text
    Op {
        span: oxc_css_parser::pos::Span,
        space_before: bool,
    },
}

/// A flat operator fill, mirroring how Prettier prints math (postcss-values has no expression tree):
/// every operator glues to its LEFT operand with the break opportunity after it,
/// continuation lines share ONE uniform indent, and nested unparenthesized operations flatten into the same fill.
#[derive(Default)]
struct OpChunks<'b, 'a> {
    chunks: Vec<Vec<OpPiece<'b, 'a>>>,
    current: Vec<OpPiece<'b, 'a>>,
}

impl<'b, 'a> OpChunks<'b, 'a> {
    fn push_operand(&mut self, operand: &'b ComponentValue<'a>) {
        self.current.push(OpPiece::Operand(operand));
    }

    fn push_paren(&mut self, paren: &'b LessParenthesizedOperation<'a>) {
        self.current.push(OpPiece::Paren(paren));
    }

    /// A space after the operator is also the fill's break opportunity
    fn push_op(&mut self, span: oxc_css_parser::pos::Span, space_before: bool, space_after: bool) {
        self.current.push(OpPiece::Op { span, space_before });
        if space_after {
            self.chunks.push(std::mem::take(&mut self.current));
        }
    }

    /// The operand pushed last: the left neighbor of the operator about to be pushed
    fn last_operand(&self) -> Option<&'b ComponentValue<'a>> {
        match self.current.last() {
            Some(OpPiece::Operand(v)) => Some(*v),
            _ => None,
        }
    }

    fn write(mut self, ctx: ValueContext<'a>, f: &mut CssFormatter<'_, 'a>) {
        if !self.current.is_empty() {
            self.chunks.push(self.current);
        }
        let chunks = self.chunks;
        let write_chunk = |chunk: &[OpPiece<'_, 'a>], f: &mut CssFormatter<'_, 'a>| {
            for piece in chunk {
                // A comment between the operands and operators stays there
                match piece {
                    OpPiece::Operand(operand) => {
                        flush_value_comments(operand.span().start, f);
                        write_component_value(operand, ctx, f);
                    }
                    OpPiece::Paren(paren) => {
                        flush_value_comments(paren.span.start, f);
                        write_parenthesized(&paren.operation, paren.span.end - 1, ctx, f);
                    }
                    OpPiece::Op { span, space_before } => {
                        if *space_before {
                            write!(f, " ");
                        }
                        write_text_with_leading_comments(to_span(*span), f);
                    }
                }
            }
        };

        if ctx.no_break || chunks.len() == 1 {
            for (i, chunk) in chunks.iter().enumerate() {
                if i > 0 {
                    write!(f, " ");
                }
                write_chunk(chunk, f);
            }
            return;
        }

        let chunks_ref = &chunks;
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let mut filler = f.fill();
            for chunk in chunks_ref {
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write_chunk(chunk, f);
                });
                filler.entry(&soft_line_break_or_space(), &content);
            }
            filler.finish();
        });
        write!(f, group(&indent(&body)));
    }
}

/// Calc expression (`a + b` inside `calc(...)`), an [`OpChunks`] fill.
fn write_calc<'a>(calc: &Calc<'a>, ctx: ValueContext<'a>, f: &mut CssFormatter<'_, 'a>) {
    fn flatten<'b, 'a>(calc: &'b Calc<'a>, out: &mut OpChunks<'b, 'a>) {
        push_operand(&calc.left, out);
        // `*` and `/` are always spaced (DIVERGENCES.md#math-function-operator-spacing);
        // `+`/`-` (and a Sass `%`) keep the source: `+`/`-` require whitespace, a glued one is invalid
        let spaced =
            matches!(calc.op.kind, CalcOperatorKind::Multiply | CalcOperatorKind::Division);
        out.push_op(
            calc.op.span,
            spaced || !is_glued(calc.left.span(), calc.op.span),
            spaced || !is_glued(calc.op.span, calc.right.span()),
        );
        push_operand(&calc.right, out);
    }

    // A parenthesized sub-expression (`CalcParenthesized`) is one operand: its parens come from the AST
    fn push_operand<'b, 'a>(operand: &'b ComponentValue<'a>, out: &mut OpChunks<'b, 'a>) {
        if let ComponentValue::Calc(inner) = operand {
            flatten(inner, out);
        } else {
            out.push_operand(operand);
        }
    }

    let mut out = OpChunks::default();
    flatten(calc, &mut out);
    out.write(ctx, f);
}

/// Less arithmetic (`@a - 2 * @b - (@c / 2)`), an [`OpChunks`] fill
/// (Prettier's `printNumber`/`adjustNumbers` math layout, umbrella prettier/prettier#1811).
///
/// Operator spacing mirrors Prettier's Less word lexing (postcss-values-parser) plus its math-operator rules:
/// - `+`/`-` keep the source glue (`10px+20px`, `@a+1`: a glued sign folds into the next word),
///   except after a CALL (`fade(@c, 4%)-1` → `fade(@c, 4%) - 1`, a sign the parser split off)
/// - `*` is always spaced (`2em*1em` → `2em * 1em`), unless an `@word` on its left swallowed it
///   (an atword only ends at whitespace / `(` / `)` / `,` / `/` / quotes: `@base*2` stays glued)
/// - `/` keeps the source's choice, made symmetric (DIVERGENCES.md#slash-spacing): `@w/2` stays, `@w/ 2` → `@w / 2`;
///   `./` keeps the source glue
///
/// Only a parsed operation gets here: outside parens a `/` after a plain number (`font: 12px/1.5`)
/// is a value delimiter for `oxc-css-parser`, printed by the same `/` rule (DIVERGENCES.md#slash-spacing).
///
/// Signed values never reach here: `-@b` in a `margin` shorthand stays a separate `LessNegativeValue`, not an operand.
fn write_less_binary_operation<'a>(
    op: &LessBinaryOperation<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    fn flatten<'b, 'a>(op: &'b LessBinaryOperation<'a>, out: &mut OpChunks<'b, 'a>) {
        push_operand(&op.left, out);
        let left_operand = out.last_operand();
        let source_before = !is_glued(op.left.span(), op.op.span);
        let source_after = !is_glued(op.op.span, op.right.span());
        // Spaces on both sides regardless of the source (see the doc comment), else the source glue
        let forced = match op.op.kind {
            LessOperationOperatorKind::Multiply => {
                source_before || !left_operand.is_some_and(is_less_atword)
            }
            LessOperationOperatorKind::Division => {
                is_slash_spaced(op.left.span(), op.op.span, op.right.span())
            }
            LessOperationOperatorKind::DotDivision => false,
            LessOperationOperatorKind::Plus | LessOperationOperatorKind::Minus => {
                left_operand.is_some_and(is_func_like)
            }
        };
        out.push_op(op.op.span, forced || source_before, forced || source_after);
        push_operand(&op.right, out);
    }

    /// postcss-values-parser's `atword`: `@var`, `@@var`, `$prop` and lookups (`@config[key]`)
    fn is_less_atword(value: &ComponentValue<'_>) -> bool {
        matches!(
            value,
            ComponentValue::LessVariable(_)
                | ComponentValue::LessVariableVariable(_)
                | ComponentValue::LessPropertyVariable(_)
                | ComponentValue::LessNamespaceValue(_)
        )
    }

    fn push_operand<'b, 'a>(operand: &'b ComponentValue<'a>, out: &mut OpChunks<'b, 'a>) {
        match operand {
            ComponentValue::LessBinaryOperation(inner) => flatten(inner, out),
            ComponentValue::LessParenthesizedOperation(paren) => out.push_paren(paren),
            _ => out.push_operand(operand),
        }
    }

    let mut out = OpChunks::default();
    flatten(op, &mut out);
    out.write(ctx, f);
}

/// A parenthesized operand (Less operation `(@a - @b)`, `calc()` sub-expression):
/// its own group, so when it breaks the `(` / `)` land on their own lines
/// with the content indented one level (Prettier's `printParenthesizedValueGroup`).
/// When it fits, it stays inline (`(@a - @b)`).
/// `(value)`, `close` its `)`: the comments before it stay inside.
fn write_parenthesized<'a>(
    value: &ComponentValue<'a>,
    close: u32,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let content = format_with(move |f| {
        write_component_value(value, ctx, f);
        flush_trailing_value_comments(close, f);
    });
    if ctx.no_break {
        write!(f, ["(", content, ")"]);
    } else {
        write!(f, [text("("), group(&soft_block_indent(&content)), text(")")]);
    }
}

/// Function call: `name(` + args + `)`.
/// Mirrors `value-func` + `value-paren_group` with parens.
pub(super) fn write_function<'a>(
    func: &Function<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let name_span = to_span(func.name.span());
    write!(f, text(source.text_for(&name_span)));

    if func.args.is_empty() {
        write!(f, ["(", ")"]);
        return;
    }
    let groups = split_comma_groups(&func.args);

    // `url(...)` as a function (e.g. `url(if(...))`): parens hug the content,
    // no break opportunities of their own (Prettier's `isURLFunctionNode`).
    if function_name_text(func).eq_ignore_ascii_case("url") {
        write!(f, "(");
        // Single plain argument: verbatim (matches the `Url` node path)
        if let [([arg], _)] = groups.as_slice()
            && matches!(
                arg,
                ComponentValue::InterpolableIdent(_) | ComponentValue::TokenWithSpan(_)
            )
        {
            let span = to_span(arg.span());
            write!(f, text(source.text_for(&span)));
        } else {
            let url_ctx = ValueContext { in_url: true, ..ctx };
            for (i, &(group_values, comma)) in groups.iter().enumerate() {
                write_comma_group(group_values, url_ctx, f);
                if i + 1 < groups.len() {
                    write_group_comma(comma, f);
                    write!(f, " ");
                }
            }
        }
        write!(f, ")");
        return;
    }

    // `no_break` (composes `removeLines` / media-value flat text):
    // no break opportunities, args joined inline.
    if ctx.no_break {
        write!(f, "(");
        for (i, &(group_values, comma)) in groups.iter().enumerate() {
            write_comma_group(group_values, ctx, f);
            if i + 1 < groups.len() {
                write_group_comma(comma, f);
                write!(f, " ");
            }
        }
        flush_trailing_value_comments(func.span().end.saturating_sub(1), f);
        write!(f, ")");
        return;
    }

    let groups_ref = &groups;
    let r_paren = func.span().end.saturating_sub(1);
    // `var(--baz,)`: an empty fallback is meaningful,
    // so a source trailing comma is preserved for `var()` ONLY
    // (Prettier's `printTrailingComma` checks `isVarFunctionNode`; every other function drops it).
    let has_trailing_comma = func.args.last().is_some_and(is_comma)
        && function_name_text(func).eq_ignore_ascii_case("var");
    // Function arguments are "map item" positions (maps break)
    let ctx = ValueContext {
        map_break: true,
        paren_break: false,
        in_args: true,
        no_leading_softline: false,
        ..ctx
    };
    // A keyword argument's paren-group value is a map item
    // ONLY when the call's FIRST argument is a key-value pair
    // (Prettier's `isKeyValuePairInParenGroupNode` checks `groups[0]` alone):
    // `func($k: (1, 2), a)` breaks, `func(a, $k: (1, 2))` does not.
    let is_kw_arg = |g: &[ComponentValue<'a>]| {
        g.len() == 1 && matches!(g[0], ComponentValue::SassKeywordArgument(_))
    };
    let first_arg_is_kw = groups.first().is_some_and(|(g, _)| is_kw_arg(g));
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let source = f.context().source_text();
        write!(f, soft_line_break());
        for (i, &(group_values, comma)) in groups_ref.iter().enumerate() {
            if i > 0 {
                // Preserve a blank line between argument groups,
                // but only after a multi-part group (Prettier checks `comma_groups`).
                let prev_end = groups_ref[i - 1].0.last().map_or(0, |v| v.span().end);
                let next_start = group_values.first().map_or(prev_end, |v| v.span().start);
                let next_start = f
                    .context()
                    .comments()
                    .peek()
                    .map_or(next_start, |c| c.span.start.min(next_start));
                if prev_end != 0
                    && groups_ref[i - 1].0.len() > 1
                    && comments::classify_gap(source.bytes_range(prev_end, next_start))
                        == comments::Gap::Blank
                {
                    write!(f, empty_line());
                } else {
                    write!(f, soft_line_break_or_space());
                }
            }
            let arg_ctx =
                ValueContext { paren_break: first_arg_is_kw && is_kw_arg(group_values), ..ctx };
            write_comma_group(group_values, arg_ctx, f);
            // `has_trailing_comma`: the kept `var(--x /* c */,)` comma counts too
            if i + 1 < groups_ref.len() || has_trailing_comma {
                write_group_comma(comma, f);
            }
        }
        // Block comments between the last argument and `)` wrap as fill items;
        // with a `//` among them they take the paren-tail layout instead.
        let tail: &'a [comments::CssComment] = f.context().comments().take_before(r_paren);
        if tail.iter().any(|c| c.inline) {
            write_paren_tail_comments(
                tail, /* body_hard_broken */ false, /* after_content */ true, f,
            );
        } else if !tail.is_empty() {
            let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                let mut filler = f.fill();
                // Anchor entry: lets the first comment's separator attach it
                // to the last argument (a fill ignores the first separator).
                filler.entry(&soft_line_break_or_space(), &format_with(|_| {}));
                for &comment in tail {
                    let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        write!(
                            f,
                            FormatCommentBeforeContent::new(comment, BlockCommentAfter::None)
                        );
                    });
                    filler.entry(&soft_line_break_or_space(), &entry);
                }
                filler.finish();
            });
            write!(f, [space(), indent(&inner)]);
        }
    });
    write!(
        f,
        group(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, "(");
            write!(f, indent(&body));
            write!(f, soft_line_break());
            write!(f, ")");
        }))
    );
}

/// `url(...)`: contents are never reformatted.
/// `#{ $a + $b }` normalizes to `#{$a + $b}`: postcss-values tokenizes through the interpolation,
/// so Prettier reprints the expression.
/// (no inner padding, one space around operators;
/// idents/variables keep their case, dimensions/strings get the normal value normalization)
pub(super) fn write_sass_interpolated_ident<'a>(
    interp: &SassInterpolatedIdent<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    for element in &interp.elements {
        match element {
            SassInterpolatedIdentElement::Static(part) => {
                write!(f, text(part.raw));
            }
            SassInterpolatedIdentElement::Expression(expr) => {
                write!(f, "#{");
                write_component_value(expr, ctx, f);
                write!(f, "}");
            }
            SassInterpolatedIdentElement::Placeholder(placeholder) => {
                super::write_placeholder(placeholder, f);
            }
        }
    }
}

pub(super) fn write_url<'a>(url: &Url<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(url.name.span());
    // Prettier preserves function-name casing (`URL(...)` stays uppercase)
    write!(f, text(source.text_for(&name_span)));
    write!(f, "(");
    match &url.value {
        Some(UrlValue::Str(InterpolableStr::Literal(str))) => {
            flush_value_comments(str.span.start, f);
            write_str(str, f);
            for modifier in &url.modifiers {
                write!(f, " ");
                let span = to_span(modifier.span());
                write!(f, text(source.text_for(&span)));
            }
            // The comments before the `)` stay inside it
            flush_trailing_value_comments(url.span.end - 1, f);
        }
        // Quoted url with interpolation: requote the outer quotes only
        Some(UrlValue::Str(istr)) => {
            let span = to_span(istr.span());
            write_requoted_verbatim(source.text_for(&span), f);
        }
        Some(UrlValue::LessEscapedStr(escaped)) => write_less_escaped_str(escaped, f),
        // Unquoted url contents are printed verbatim, including inner spaces
        _ => {
            let url_span = url.span();
            let inner_start = url.name.span().end + 1;
            // oxc-css-parser's url span may stop before trailing padding; scan to `)`
            let bytes = source.as_bytes();
            let mut close = url_span.end.saturating_sub(1) as usize;
            while close < bytes.len() && bytes[close] != b')' {
                close += 1;
            }
            let inner_end = u32::try_from(close).unwrap_or(url_span.end.saturating_sub(1));
            if inner_start < inner_end {
                let inner = source.slice_range(inner_start, inner_end);
                // Escaped parens keep their padding verbatim; otherwise trim
                if inner.contains("\\(") || inner.contains("\\)") {
                    write!(f, text(inner));
                } else {
                    let trimmed = inner.trim();
                    // `url($a+$b)`: SCSS concatenation gets spaced.
                    // SCSS only, in Css mode (postcss-simple-vars `$var`s)
                    // Prettier keeps the raw url contents verbatim.
                    if matches!(f.options().variant, crate::options::CssVariant::Scss)
                        && trimmed.contains('$')
                        && trimmed.contains('+')
                        && !trimmed.contains(' ')
                    {
                        let spaced = trimmed.cow_replace('+', " + ");
                        write!(f, text(f.allocator().alloc_str(&spaced)));
                    } else {
                        write!(f, text(trimmed));
                    }
                }
            }
        }
    }
    write!(f, ")");
}
