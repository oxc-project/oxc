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
use oxc_css_parser::{
    ast::{
        Calc, CalcOperatorKind, ComponentValue, Delimiter, DelimiterKind, Dimension, Function,
        FunctionName, InterpolableIdent, InterpolableStr, LessBinaryOperation,
        LessOperationOperatorKind, LessParenthesizedOperation, Number, SassBinaryExpression,
        SassBinaryOperator, SassBinaryOperatorKind, SassInterpolatedIdent,
        SassInterpolatedIdentElement, SassUnaryOperatorKind, Str, Url, UrlValue,
    },
    token::Token,
};

use oxc_formatter_core::{
    Buffer, SourceText, arena_cow_str,
    builders::{
        empty_line, expand_parent, group, hard_line_break, if_group_breaks, indent,
        soft_line_break, soft_line_break_or_space, space, text, token,
    },
    spec::{format_trimmed_number, normalize_string},
    write,
};

use crate::{
    CssFormatOptions, comments,
    format::to_span,
    print::{CssFormatter, format_with, less, scss, statement},
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
fn print_string<'a>(raw: &'a str, options: &CssFormatOptions) -> Cow<'a, str> {
    let content = &raw[1..raw.len() - 1];
    let enclosing = options.preferred_quote(content);

    if raw.as_bytes()[0] == enclosing {
        return Cow::Borrowed(raw);
    }

    let normalized = normalize_string(content, enclosing, true);
    let mut out = String::with_capacity(raw.len());
    out.push(enclosing as char);
    out.push_str(&normalized);
    out.push(enclosing as char);
    Cow::Owned(out)
}

pub(super) fn write_str<'a>(str: &Str<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_str_raw(str.raw, f);
}

/// [`write_str`] for a raw quoted-string slice
/// (callers that only hold the source text, no AST node).
pub(super) fn write_str_raw<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let printed = print_string(raw, f.options());
    write!(f, text(arena_cow_str(&printed, f)));
}

/// [`write_str`] for attribute-selector values.
/// Prettier wraps the value in `replaceEndOfLine(..., literallineWithoutBreakParent)`:
/// an escaped literal newline inside the value prints verbatim,
/// but must not force a break in front of the selector.
pub(super) fn write_attribute_str<'a>(str: &Str<'a>, f: &mut CssFormatter<'_, 'a>) {
    let printed = print_string(str.raw, f.options());
    write!(f, text(arena_cow_str(&printed, f)).without_expand_parent());
}

/// Prettier's `adjustNumbers(adjustStrings(...))` over a raw source slice:
/// strings re-quote via `printString`,
/// standalone numbers (not glued to a word part) get `printCssNumber` + lowercased/canonical unit;
/// Everything else (whitespace and newlines included) stays verbatim.
/// Used where Prettier prints raw selector text: Less mixin definitions/calls and `when` guards.
pub(super) fn adjust_numbers_and_strings<'a>(
    raw: &'a str,
    options: &CssFormatOptions,
) -> Cow<'a, str> {
    let bytes = raw.as_bytes();
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

    while i < bytes.len() {
        let b = bytes[i];
        // Strings: re-quote, never adjust their content as numbers.
        if b == b'"' || b == b'\'' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b {
                j += if bytes[j] == b'\\' { 2 } else { 1 };
            }
            if j < bytes.len() {
                let printed = print_string(&raw[i..=j], options);
                changed |= matches!(printed, Cow::Owned(_));
                out.push_str(&printed);
                i = j + 1;
                continue;
            }
            // Unterminated: plain char.
            out.push(b as char);
            i += 1;
            continue;
        }
        // Word part `[$@]?[_a-z-￿][\w-￿-]*`; a number
        // glued to it (plus its unit) is left untouched.
        let word_start =
            if (b == b'$' || b == b'@') && i + 1 < bytes.len() && is_word_start(bytes[i + 1]) {
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
            let number_end = scan_number(j);
            if number_end > j {
                j = scan_unit(number_end);
            }
            out.push_str(&raw[i..j]);
            i = j;
            continue;
        }
        // Standalone number (+ optional unit)
        if b.is_ascii_digit() || b == b'.' {
            let number_end = scan_number(i);
            if number_end > i {
                let unit_end = scan_unit(number_end);
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
        // Plain byte; push the whole UTF-8 char
        let ch_len = raw[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&raw[i..i + ch_len]);
        i += ch_len;
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
        let normalized = normalize_quoted_numbers(raw);
        if normalized == raw {
            write!(f, text(raw));
        } else {
            write!(f, text(f.allocator().alloc_str(&normalized)));
        }
        return;
    }

    let enclosing = f.options().preferred_quote(content);
    if raw.as_bytes()[0] == enclosing {
        write!(f, text(raw));
    } else {
        let q = enclosing as char;
        let out = format!("{q}{content}{q}");
        write!(f, text(f.allocator().alloc_str(&out)));
    }
}

/// Normalizes `".5"`-style quoted bare numbers inside an interpolated string.
fn normalize_quoted_numbers(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    // Skip the outer opening quote
    out.push(bytes[0] as char);
    i += 1;
    let end = raw.len() - 1;
    while i < end {
        let b = bytes[i];
        if (b == b'"' || b == b'\'') && i + 1 < end {
            // Find the matching close within the content
            if let Some(close_rel) = raw[i + 1..end].find(b as char) {
                let inner = &raw[i + 1..i + 1 + close_rel];
                if !inner.is_empty() && inner.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
                    out.push(b as char);
                    out.push_str(&print_css_number(inner));
                    out.push(b as char);
                    i = i + 1 + close_rel + 1;
                    continue;
                }
            }
        }
        out.push(b as char);
        i += 1;
    }
    out.push(bytes[end] as char);
    out
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
/// Such a number stays glued and prints raw (`Separator::Word`).
/// `.10` must not normalize to `0.1`, `+1` must not gain a space.
/// Two glued number-ish values can never be valid CSS,
/// so a number-ish neighbor is as sure a word sign as a word-like one.
/// (The raw-slice twin of this rule lives in `adjust_numbers_and_strings`,
/// whose word-part scan skips numbers glued to a word; keep them in sync.)
fn is_word_glued_number(values: &[ComponentValue<'_>], i: usize) -> bool {
    let number_ish = |value: &ComponentValue<'_>| {
        matches!(
            value,
            ComponentValue::Number(_)
                | ComponentValue::Dimension(_)
                | ComponentValue::Percentage(_)
        )
    };
    i > 0
        && number_ish(&values[i])
        && (is_word_like(&values[i - 1]) || number_ish(&values[i - 1]))
        && to_span(values[i - 1].span()).end == to_span(values[i].span()).start
}

/// Is this number-ish or a font-size-capable function? (Prettier's `isPossibleFontSize`)
fn is_possible_font_size(value: &ComponentValue<'_>) -> bool {
    match value {
        ComponentValue::Number(_)
        | ComponentValue::Dimension(_)
        | ComponentValue::Percentage(_) => true,
        ComponentValue::Function(func) => {
            let name = function_name_text(func);
            name.eq_ignore_ascii_case("var")
                || name.eq_ignore_ascii_case("calc")
                || name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.starts_with("--")
        }
        _ => false,
    }
}

fn function_name_text<'a>(func: &Function<'a>) -> &'a str {
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
    /// Inside function/include arguments (comment-slot rules differ).
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
        self.decl_prop.is_some_and(|p| p == "grid" || p.starts_with("grid-template"))
    }

    fn is_font_or_custom(&self) -> bool {
        self.decl_prop.is_some_and(|p| p == "font" || p.starts_with("--"))
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
fn split_comma_groups<'b, 'a>(
    values: &'b [ComponentValue<'a>],
) -> Vec<(&'b [ComponentValue<'a>], Option<u32>)> {
    let mut groups = vec![];
    let mut start = 0;
    let mut depth = 0i32;
    for (i, v) in values.iter().enumerate() {
        if let ComponentValue::TokenWithSpan(tok) = v {
            if depth == 0 && matches!(&tok.token, Token::Comma(_)) {
                groups.push((&values[start..i], Some(to_span(v.span()).start)));
                start = i + 1;
            } else {
                depth += token_depth_delta(&tok.token);
            }
            continue;
        }
        if is_comma(v) {
            groups.push((&values[start..i], Some(to_span(v.span()).start)));
            start = i + 1;
        }
    }
    groups.push((&values[start..], None));
    // A trailing comma produces an empty last group; Prettier drops it
    if groups.len() > 1 && groups.last().is_some_and(|(g, _)| g.is_empty()) {
        groups.pop();
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
    let value_start = to_span(values[0].span()).start;
    let value_end = to_span(values[values.len() - 1].span()).end;
    let has_comments =
        f.context().comments().iter_before(value_end).any(|c| c.span.start >= value_start);
    let force_hard_line = !ctx.decl_prop.is_some_and(|p| p.starts_with("--"))
        && (groups.iter().enumerate().any(|(i, (g, _))| comma_group_is_multi(g, i == 0))
            || has_comments);

    write_value_groups(&groups, ctx, force_hard_line, true, f);
}

/// Does this comma group count as a `value-comma_group` for Prettier's `shouldBreakList`?
/// Multi-element groups do; so does a single ident with a leading `-` in non-initial position,
/// which postcss-values splits into an operator + word
/// (`Arial, -apple-system` breaks the list while `-apple-system, Arial` does not).
pub(super) fn comma_group_is_multi(group: &[ComponentValue<'_>], is_first: bool) -> bool {
    if group.len() > 1 {
        return true;
    }
    // Less `~'...'` lexes as TWO postcss-values nodes (`~` word + string),
    // so it is a real `value-comma_group` in ANY position.
    if matches!(group.first(), Some(ComponentValue::LessEscapedStr(_))) {
        return true;
    }
    !is_first
        && matches!(group.first(),
            Some(ComponentValue::InterpolableIdent(InterpolableIdent::Literal(id)))
                if id.raw.starts_with('-') && !id.raw.starts_with("--"))
}

/// A group's separator comma:
/// comments between the group and its comma stay before the comma (`a /* c */, b`);
/// only comments past it lead the next group.
fn write_group_comma(comma_start: Option<u32>, f: &mut CssFormatter<'_, '_>) {
    if let Some(comma) = comma_start {
        flush_trailing_value_comments(comma, f);
    }
    write!(f, ",");
}

/// Top-level comma-group list layout, shared between flat component streams and SCSS comma lists.
/// Each group comes paired with the start of its trailing comma (see [`split_comma_groups`] / [`write_group_comma`]).
pub(super) fn write_value_groups<'a>(
    groups: &[(&[ComponentValue<'a>], Option<u32>)],
    ctx: ValueContext<'a>,
    force_hard_line: bool,
    _top_level: bool,
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
                // Comments between groups (e.g. after the previous comma)
                // fill together with the group they precede;
                // `//` comments (and leading own-line comments) keep their own line.
                let lead: &'a [comments::CssComment] = group_values.first().map_or(&[], |first| {
                    f.context().comments().take_before(to_span(first.span()).start)
                });
                if lead.is_empty() {
                    write_comma_group(group_values, gctx, f);
                } else if i == 0 {
                    let source = f.context().source_text();
                    for &comment in lead {
                        let own_line = comment_is_own_line(comment, source);
                        comments::write_single_comment(comment, f);
                        if comment.inline || own_line {
                            write!(f, hard_line_break());
                        } else {
                            write!(f, " ");
                        }
                    }
                    write_comma_group(group_values, gctx, f);
                } else {
                    // Prettier-fill the comments with the group they lead.
                    // Simulated with static widths:
                    // the group's fit must NOT include its declaration tail, which our entry would.
                    let width = u32::from(f.options().line_width.value());
                    let group_w =
                        group_values.first().zip(group_values.last()).map_or(0, |(first, last)| {
                            to_span(last.span()).end - to_span(first.span()).start
                        }) + u32::from(!is_last);
                    let mut x = 4u32; // hardline indent under the value
                    for (k, &comment) in lead.iter().enumerate() {
                        comments::write_single_comment(comment, f);
                        x += comment.span.end - comment.span.start;
                        let next_w = lead.get(k + 1).map_or(group_w, |c| c.span.end - c.span.start);
                        if comment.inline {
                            write!(f, hard_line_break());
                            x = 4;
                        } else if x + 1 + next_w <= width {
                            write!(f, space());
                            x += 1;
                        } else {
                            write!(f, hard_line_break());
                            x = 4;
                        }
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
                    if let Some(first) = group_values.first() {
                        flush_value_comments(to_span(first.span()).start, f);
                    }
                    write_comma_group(group_values, gctx, f);
                    if !is_last {
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

/// Emits pending comments that precede `upper_bound` inline
/// (`//` comments force a break after themselves and expand the parent).
/// Returns true when the last emitted comment was a `//` comment.
pub(super) fn flush_value_comments(upper_bound: u32, f: &mut CssFormatter<'_, '_>) -> bool {
    let mut last_inline = false;
    for &comment in f.context().comments().take_before(upper_bound) {
        comments::write_single_comment(comment, f);
        if comment.inline {
            write!(f, [expand_parent(), hard_line_break()]);
        } else {
            write!(f, " ");
        }
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
    loop {
        let Some(comment) = f.context().comments().peek() else { return };
        let source = f.context().source_text();
        if comment.span.start < prev_end
            || comment.span.end > upper_bound
            || comment_is_own_line(comment, source)
        {
            return;
        }
        f.context().comments().take_before(comment.span.end);
        write!(f, " ");
        comments::write_single_comment(comment, f);
        if comment.inline {
            write!(f, [expand_parent()]);
            return;
        }
    }
}

/// Emits pending comments before `upper_bound` as ` /* c */` suffixes
/// (used after the last value component, before `;` / `!important`).
/// Returns the end offset of the last emitted comment.
pub(super) fn flush_trailing_value_comments(
    upper_bound: u32,
    f: &mut CssFormatter<'_, '_>,
) -> Option<u32> {
    let mut last_end = None;
    for &comment in f.context().comments().take_before(upper_bound) {
        write!(f, " ");
        comments::write_single_comment(comment, f);
        if comment.inline {
            write!(f, [expand_parent(), hard_line_break()]);
        }
        last_end = Some(comment.span.end);
    }
    last_end
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
    // Prettier's `flattenGroups`:
    // a single-element comma group collapses to the element itself (no extra group/indent level).
    if values.len() == 1 && ctx.tail_bound.is_none() {
        flush_value_comments(to_span(values[0].span()).start, f);
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
    let upper_bound = to_span(values[values.len() - 1].span()).end;
    // Grid values that break across source lines start on their own line
    // (Prettier's `didBreak` + `parts.unshift(hardline)`),
    // but breaks caused by inline comments in the gap don't count.
    // (the comment-hardline branch runs before the grid check in Prettier)
    let grid_did_break = ctx.is_grid()
        && (1..values.len()).any(|i| {
            separator_between(values, i, ctx, source) == Separator::Hard && {
                let prev_end = to_span(values[i - 1].span()).end;
                let start = to_span(values[i].span()).start;
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
            f.context().comments().iter_before(upper_bound).collect();
        let tail_bound = ctx.tail_bound;
        let ctx = ValueContext { tail_bound: None, ..ctx };
        let tail_comments: Vec<comments::CssComment> = tail_bound
            .map(|bound| {
                f.context()
                    .comments()
                    .iter_before(bound)
                    .filter(|c| c.span.start >= upper_bound)
                    .collect()
            })
            .unwrap_or_default();
        let mut filler = f.fill();
        let mut i = 0;
        while i < values.len() {
            // Separator between the previous component and this one
            // (ignored by the fill builder for the first entry).
            let mut sep =
                if i == 0 { Separator::Line } else { separator_between(values, i, ctx, source) };
            // An inline comment trailing the PREVIOUS run forces a break
            if i > 0 {
                let prev_end = to_span(values[i - 1].span()).end;
                let start = to_span(values[i].span()).start;
                if pending.iter().any(|c| {
                    c.inline
                        && c.span.start >= prev_end
                        && c.span.end <= start
                        && !comment_is_own_line(*c, source)
                }) {
                    sep = Separator::Hard;
                }
            }
            // Merge runs of tight / non-breaking-space components into one fill entry
            let mut run_end = i + 1;
            while run_end < values.len()
                && matches!(
                    separator_between(values, run_end, ctx, source),
                    Separator::Tight | Separator::Word | Separator::Space
                )
            {
                run_end += 1;
            }
            let run_start = i;
            let run = &values[i..run_end];
            let start = to_span(values[i].span()).start;
            let run_end_pos = to_span(values[run_end - 1].span()).end;
            let is_last_run = run_end == values.len();
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                flush_value_comments(start, f);
                for (j, v) in run.iter().enumerate() {
                    if j > 0 {
                        let sep = separator_between(values, run_start + j, ctx, source);
                        if sep == Separator::Space {
                            write!(f, " ");
                        }
                        // A word continuation prints raw, not normalized
                        // (`sandstone.10`, `[0.50]` must survive as-is).
                        if sep == Separator::Word {
                            write!(f, text(source.text_for(&to_span(v.span()))));
                            continue;
                        }
                    }
                    write_component_value(v, ctx, f);
                }
                // Same-line `//` comments stay attached to this entry
                // (a separate entry would break with the expanded group).
                if !is_last_run {
                    let next_start = to_span(values[run_end].span()).start;
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
                        comments::write_single_comment(comment, f);
                        write!(f, expand_parent());
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
            // Trailing same-line comments become their own fill items
            // (they wrap independently when the line is too long).
            if !is_last_run {
                let next_start = to_span(values[run_end].span()).start;
                for &comment in pending.iter().filter(|c| {
                    !c.inline
                        && c.span.start >= run_end_pos
                        && c.span.end <= next_start
                        && !comment_is_own_line(**c, source)
                }) {
                    let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        if f.context().comments().peek().is_some_and(|c| c.span == comment.span) {
                            f.context().comments().take_before(comment.span.end);
                            comments::write_single_comment(comment, f);
                            if comment.inline {
                                write!(f, expand_parent());
                            }
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
                comments::write_single_comment(comment, f);
                if comment.inline {
                    write!(f, expand_parent());
                }
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

/// Decides the separator BEFORE `values[i]` (i >= 1).
///
/// In Prettier's loop terms:
/// - `iNode` = `values[i - 1]`
/// - `iNextNode` = `values[i]`
/// - `iPrevNode` = `values[i - 2]`
/// - `iNextNextNode` = `values[i + 1]`
fn separator_between(
    values: &[ComponentValue<'_>],
    i: usize,
    ctx: ValueContext<'_>,
    source: SourceText<'_>,
) -> Separator {
    let prev = &values[i - 1];
    let curr = &values[i];
    let prev_span = to_span(prev.span());
    let curr_span = to_span(curr.span());
    // `hasEmptyRawBefore(iNextNode)`
    let gap_empty = prev_span.end == curr_span.start;
    // `hasEmptyRawBefore(iNode)`
    let prev_gap_empty = i >= 2 && to_span(values[i - 2].span()).end == prev_span.start;

    // Grid: preserve source line structure:
    // Prettier emits a hardline where the source breaks and a PLAIN SPACE otherwise
    // (never re-wraps a single-line grid value, however long).
    if ctx.is_grid() {
        let gap = source.bytes_range(prev_span.end, curr_span.start);
        if gap.contains(&b'\n') || gap.contains(&b'\r') {
            return Separator::Hard;
        }
        return Separator::Space;
    }

    // Solidus (`/`) spacing rules
    if is_solidus(curr) || is_solidus(prev) {
        // Path-like streams (`-fb-url(/a/b.png)`): glued tokens stay glued
        if is_solidus(&values[0]) && gap_empty {
            return Separator::Tight;
        }
        // Fully glued `/` (`center/80%`, `12px/1.5`): one postcss word
        if is_solidus(curr)
            && gap_empty
            && values.get(i + 1).is_none_or(|n| to_span(curr.span()).end == to_span(n.span()).start)
        {
            return Separator::Tight;
        }
        if is_solidus(prev) && gap_empty && prev_gap_empty {
            return Separator::Tight;
        }
        // `font: 12px/1.5` and custom properties:
        // keep tight when the source is tight around a font-size-capable node.
        if ctx.is_font_or_custom() {
            if is_solidus(curr) && gap_empty && is_possible_font_size(prev) {
                return Separator::Tight;
            }
            if is_solidus(prev) && prev_gap_empty && i >= 2 && is_possible_font_size(&values[i - 2])
            {
                return Separator::Tight;
            }
        }
        // Leading `/` (e.g. `-fb-url(/abs/path)`)
        if i == 1 && is_solidus(prev) {
            return Separator::Tight;
        }

        let require_space_before = values.get(i + 1).is_some_and(is_word_like)
            || values.get(i + 1).is_some_and(is_func_like)
            || is_func_like(prev)
            || is_word_like(prev);
        let require_space_after = is_word_like(curr)
            || is_func_like(curr)
            || (i >= 2 && (is_word_like(&values[i - 2]) || is_func_like(&values[i - 2])));

        let tight_rule = (is_solidus(curr) && !require_space_before)
            || (is_solidus(prev) && !require_space_after);
        // `hasEmptyRawBefore(iNextNode) || (isMathOperator && iPrevNode is math op)`
        let position_rule =
            gap_empty || (is_solidus(prev) && (i < 2 || is_solidus(&values[i - 2])));

        if tight_rule && position_rule {
            return Separator::Tight;
        }
        return Separator::Line;
    }

    // Less lookups: `@var [@result]` loses the gap (`var [@lookup]` rule)
    if matches!(curr, ComponentValue::BracketBlock(_))
        && matches!(
            prev,
            ComponentValue::LessVariable(_)
                | ComponentValue::LessNamespaceValue(_)
                | ComponentValue::BracketBlock(_)
                | ComponentValue::LessMixinCall(_)
        )
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
        // Token-level division, mirroring the structural rules:
        // tight only when glued in the source and no word/function neighbors.
        let is_solidus_tok =
            |v: &ComponentValue<'_>| matches!(raw_token(v), Some(Token::Solidus(_)));
        // A leading `/` makes the stream path-like (`-fb-url(/a/b.png)`):
        // glued tokens stay glued.
        if is_solidus_tok(&values[0]) && gap_empty {
            return Separator::Tight;
        }
        // Token-level font shorthand rule
        // (`--font: var(--size)/2` in custom properties, where the value is a raw token stream).
        if ctx.is_font_or_custom() {
            let font_size_ish = |v: Option<&ComponentValue<'_>>| {
                v.is_some_and(|v| {
                    matches!(
                        raw_token(v),
                        Some(
                            Token::Number(_)
                                | Token::Dimension(_)
                                | Token::Percentage(_)
                                | Token::RParen(_)
                        )
                    )
                })
            };
            if is_solidus_tok(curr) && gap_empty && font_size_ish(Some(prev)) {
                return Separator::Tight;
            }
            if is_solidus_tok(prev)
                && prev_gap_empty
                && font_size_ish(values.get(i.wrapping_sub(2)))
            {
                return Separator::Tight;
            }
        }
        let wordish = |v: Option<&ComponentValue<'_>>| {
            v.is_some_and(|v| {
                matches!(raw_token(v), Some(Token::Ident(_) | Token::RParen(_)))
                    || is_word_like(v)
                    || is_func_like(v)
            })
        };
        if is_solidus_tok(curr) {
            let require_space = wordish(values.get(i + 1)) || wordish(Some(prev));
            return if gap_empty && !require_space { Separator::Tight } else { Separator::Line };
        }
        if is_solidus_tok(prev) {
            let require_space = wordish(Some(curr)) || wordish(values.get(i.wrapping_sub(2)));
            return if gap_empty && !require_space { Separator::Tight } else { Separator::Line };
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
    if gap_empty
        && (matches!(prev, ComponentValue::TokenWithSpan(_) | ComponentValue::Placeholder(_))
            || matches!(curr, ComponentValue::TokenWithSpan(_) | ComponentValue::Placeholder(_))
            || matches!(prev, ComponentValue::Delimiter(_))
            || matches!(curr, ComponentValue::Delimiter(_)))
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
        ComponentValue::Ratio(ratio) => {
            write_number(&ratio.numerator, f);
            write!(f, "/");
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
        ComponentValue::SassMap(map) => scss::write_sass_map(map, ctx, f),
        ComponentValue::SassList(list) => scss::write_sass_list(list, ctx, f),
        ComponentValue::SassParenthesizedExpression(paren) => {
            // `$var: ((a, b), (c, d))` / map item values: one item per line.
            // ONLY a comma-separated list takes this break:
            // the trailing comma is a semantic no-op there and NOWHERE else.
            // NOTE: `(x,)` is a single-element list in Sass,
            // so adding it to `($a + $b)` / `(a b)` / `(-$a)` changes the value
            // and `2 * ($a + $b,)` fails to compile.
            // (Prettier 3.9.1 does all of these; #19091 exempted single-node scalars only)
            if ctx.paren_break
                && let ComponentValue::SassList(list) = &*paren.expr
                && list.comma_spans.is_some()
            {
                // Comma-list map-item values always break (`isSCSSMapItemNode`),
                // with a trailing comma per option.
                let inner_ctx = ValueContext { paren_break: false, ..ctx };
                let trailing = f.options().allow_trailing_comma();
                let elements = &list.elements;
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, hard_line_break());
                    for (i, el) in elements.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",");
                            write!(f, hard_line_break());
                        }
                        write_component_value(el, inner_ctx, f);
                    }
                    if trailing {
                        write!(f, ",");
                    }
                });
                write!(f, ["(", indent(&body), hard_line_break(), ")"]);
                return;
            }
            let inner_ctx = ValueContext { paren_break: false, ..ctx };
            let trailing = f.options().allow_trailing_comma();
            let r_paren = to_span(paren.span()).end.saturating_sub(1);
            let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write!(f, soft_line_break());
                // A comma list directly inside parens shares the paren's
                // indent (no nested group level).
                if let ComponentValue::SassList(list) = &*paren.expr
                    && list.comma_spans.is_some()
                {
                    for (i, el) in list.elements.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",");
                            write!(f, soft_line_break_or_space());
                        }
                        write_component_value(el, inner_ctx, f);
                    }
                    // `ctx.paren_break` cannot be set here,
                    // a comma list with it takes the hard-break branch above.
                    if trailing && ctx.map_key {
                        write!(f, if_group_breaks(&text(",")));
                    }
                } else {
                    write_component_value(&paren.expr, inner_ctx, f);
                }
                // Inline comments before `)` stay inside, forcing the break
                for &comment in f.context().comments().take_before(r_paren) {
                    write!(f, " ");
                    comments::write_single_comment(comment, f);
                    if comment.inline {
                        write!(f, expand_parent());
                    }
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
                && to_span(&unary.op.span).end != to_span(unary.expr.span()).start
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
        ComponentValue::BracketBlock(bracket) => {
            write!(f, "[");
            for (i, v) in bracket.value.iter().enumerate() {
                if i > 0 {
                    write!(f, " ");
                }
                write_component_value(v, ctx, f);
            }
            write!(f, "]");
        }
        ComponentValue::UnicodeRange(range) => {
            let span = to_span(range.span());
            write!(f, text(source.text_for(&span)));
        }
        // `~'...'`: postcss sees a plain string token after the `~`,
        // so it re-quotes per `singleQuote` like any other string.
        ComponentValue::LessEscapedStr(escaped) => {
            write!(f, "~");
            write_str(&escaped.str, f);
        }
        // Less arithmetic: a flat operator fill (break after the operator), mirroring Prettier.
        // Safe to restructure because `oxc-css-parser` only ASTs a whitespace-followed
        // `+`/`-` as a binary operator (see `write_less_binary_operation`).
        ComponentValue::LessBinaryOperation(op) => write_less_binary_operation(op, ctx, f),
        ComponentValue::LessParenthesizedOperation(paren) => {
            write_less_parenthesized_operation(paren, ctx, f);
        }
        // A css-in-js placeholder becomes a typed marker the host replaces with `${expr}`
        ComponentValue::Placeholder(placeholder) => {
            super::write_placeholder(placeholder, f);
        }
        // postcss-simple-vars `$name` reference (Css mode with the opt-in)
        ComponentValue::PostcssSimpleVar(variable) => {
            super::postcss_simple_vars::write_postcss_simple_var(variable, f);
        }
        // Everything else (Sass/Less constructs, interpolations, token fallbacks):
        // print the source verbatim until ported structurally.
        _ => {
            if less::write_less_component_value(value, f) {
                return;
            }
            let span = to_span(value.span());
            write!(f, text(source.text_for(&span)));
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
            let mut division_force_space = false;
            while let Some(op) = parts[run_end].1 {
                if matches!(op.kind, SassBinaryOperatorKind::Multiply) {
                    break;
                }
                let op_span = to_span(&op.span);
                let next = &parts[run_end + 1].0;
                let operand = &parts[run_end].0;
                // Division mirrors postcss-values lexing:
                // a word-led chunk absorbs a directly attached `/` (`$w/2`, `$w/ 2` stay verbatim);
                // otherwise word/func NEIGHBORS force spaces on both sides
                // (`2/$w` -> `2 / $w`, `$w /2` -> `$w / 2`)
                // while plain numbers keep the source gap (`10px/8px`, `1/2`).
                if matches!(op.kind, SassBinaryOperatorKind::Division) {
                    let wf = |v: &ComponentValue<'_>| is_word_like(v) || is_func_like(v);
                    let left_absorbs = to_span(operand.span()).end == op_span.start && wf(operand);
                    if !left_absorbs {
                        let near_wordish = wf(operand)
                            || wf(next)
                            || run_end.checked_sub(1).is_some_and(|k| wf(parts[k].0))
                            || parts.get(run_end + 2).is_some_and(|(o, _)| wf(o));
                        if near_wordish {
                            division_force_space = true;
                            break;
                        }
                    }
                }
                let next_start = to_span(next.span()).start;
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
                    if to_span(operand.span()).end != op_span.start && op_span.end == next_start {
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
                    write_component_value(operand, ctx, f);
                    if let Some(op) = op {
                        let op_span = to_span(&op.span);
                        let operand_end = to_span(operand.span()).end;
                        let next_start = run.get(j + 1).map(|(next, _)| to_span(next.span()).start);
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
                        // Division: space before `/` unless a word-led operand absorbs it
                        // (see the run-merge rule above).
                        let division_spaces =
                            op_text == "/" && division_force_space && j + 1 == run.len();
                        if operand_end != op_span.start
                            || wordish
                            || division_spaces
                            // `*` always spaces (it also ends its run above)
                            || op_text == "*"
                            || (fuses_paren
                                && run.get(j + 1).is_some_and(|(next, _)| {
                                    matches!(next, ComponentValue::SassParenthesizedExpression(_))
                                }))
                        {
                            write!(f, " ");
                        }
                        write!(f, text(source.text_for(&op_span)));
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

/// Calc expression (`a + b` inside `calc(...)`):
/// spaces around `+`/`-` follow the source (invalid syntax otherwise),
/// with a break opportunity after the operator only.
fn write_calc<'a>(calc: &Calc<'a>, ctx: ValueContext<'a>, f: &mut CssFormatter<'_, 'a>) {
    // Prettier prints calc contents as a FLAT run of word/operator fill chunks
    // (postcss-values has no expression tree):
    // every operator glues to its LEFT operand with the break opportunity after it,
    // and continuation lines share ONE uniform indent,
    // nested unparenthesized operations do NOT nest groups/indents.
    // A parenthesized sub-expression stays a single chunk.
    // Source-glued runs (`1px+2px`) stay glued.
    enum Piece<'b, 'a> {
        /// The bool: print the operand's own source parens around it
        /// (computed once at flatten time, `calc_operand_has_own_parens` scans the source).
        Operand(&'b ComponentValue<'a>, bool),
        Op(&'static str),
        Space,
    }

    fn flatten<'b, 'a>(
        calc: &'b Calc<'a>,
        f: &CssFormatter<'_, 'a>,
        chunks: &mut Vec<Vec<Piece<'b, 'a>>>,
        current: &mut Vec<Piece<'b, 'a>>,
    ) {
        let op_str: &'static str = match calc.op.kind {
            CalcOperatorKind::Plus => "+",
            CalcOperatorKind::Minus => "-",
            CalcOperatorKind::Multiply => "*",
            CalcOperatorKind::Division => "/",
            // Sass modulo inside a math function's arguments (`max(1px, 7px % 4)`)
            CalcOperatorKind::Modulo => "%",
        };
        let left_end = to_span(calc.left.span()).end;
        let op_span = to_span(&calc.op.span);
        let right_start = to_span(calc.right.span()).start;

        push_operand(&calc.left, f, chunks, current);
        if left_end != op_span.start {
            current.push(Piece::Space);
        }
        current.push(Piece::Op(op_str));
        if op_span.end != right_start {
            chunks.push(std::mem::take(current));
        }
        push_operand(&calc.right, f, chunks, current);
    }

    fn push_operand<'b, 'a>(
        operand: &'b ComponentValue<'a>,
        f: &CssFormatter<'_, 'a>,
        chunks: &mut Vec<Vec<Piece<'b, 'a>>>,
        current: &mut Vec<Piece<'b, 'a>>,
    ) {
        let wrapped = calc_operand_has_own_parens(operand, f);
        if let ComponentValue::Calc(inner) = operand
            && !wrapped
        {
            flatten(inner, f, chunks, current);
        } else {
            current.push(Piece::Operand(operand, wrapped));
        }
    }

    let mut chunks: Vec<Vec<Piece<'_, 'a>>> = vec![];
    let mut current: Vec<Piece<'_, 'a>> = vec![];
    flatten(calc, f, &mut chunks, &mut current);
    if !current.is_empty() {
        chunks.push(current);
    }

    let write_chunk = |chunk: &[Piece<'_, 'a>], f: &mut CssFormatter<'_, 'a>| {
        for piece in chunk {
            match piece {
                Piece::Operand(operand, wrapped) => {
                    if *wrapped {
                        write!(f, "(");
                    }
                    write_component_value(operand, ctx, f);
                    if *wrapped {
                        write!(f, ")");
                    }
                }
                Piece::Op(op) => write!(f, token(op)),
                Piece::Space => write!(f, " "),
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

/// Less arithmetic (`@a - 2 * @b - (@c / 2)`):
/// a flat operator fill matching Prettier (the `printNumber`/`adjustNumbers` math layout, umbrella prettier/prettier#1811).
/// Every operator glues to its LEFT operand with the break opportunity AFTER it;
/// continuation lines share one uniform indent.
/// Nested unparenthesized operations flatten into the SAME fill;
/// a parenthesized sub-expression is its own nested group (it can break inside its parens).
/// Operator spacing follows the source: `+`/`-` always have whitespace here
/// (`oxc-css-parser` only treats a whitespace-followed `+`/`-` as a binary operator,
/// a signed value like `-@b` in a `margin` shorthand stays a separate `LessNegativeValue`, never an operand here),
/// while `*`/`/` may be glued (`@base*2`).
fn write_less_binary_operation<'a>(
    op: &LessBinaryOperation<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    enum Piece<'b, 'a> {
        Operand(&'b ComponentValue<'a>),
        Paren(&'b LessParenthesizedOperation<'a>),
        Op(&'static str),
        Space,
    }

    fn flatten<'b, 'a>(
        op: &'b LessBinaryOperation<'a>,
        chunks: &mut Vec<Vec<Piece<'b, 'a>>>,
        current: &mut Vec<Piece<'b, 'a>>,
    ) {
        let op_str: &'static str = match op.op.kind {
            LessOperationOperatorKind::Multiply => "*",
            LessOperationOperatorKind::Division => "/",
            LessOperationOperatorKind::Plus => "+",
            LessOperationOperatorKind::Minus => "-",
        };
        let left_end = to_span(op.left.span()).end;
        let op_span = to_span(&op.op.span);
        let right_start = to_span(op.right.span()).start;

        push_operand(&op.left, chunks, current);
        if left_end != op_span.start {
            current.push(Piece::Space);
        }
        current.push(Piece::Op(op_str));
        // Break opportunity only when the source has a gap after the operator
        if op_span.end != right_start {
            chunks.push(std::mem::take(current));
        }
        push_operand(&op.right, chunks, current);
    }

    fn push_operand<'b, 'a>(
        operand: &'b ComponentValue<'a>,
        chunks: &mut Vec<Vec<Piece<'b, 'a>>>,
        current: &mut Vec<Piece<'b, 'a>>,
    ) {
        match operand {
            ComponentValue::LessBinaryOperation(inner) => flatten(inner, chunks, current),
            ComponentValue::LessParenthesizedOperation(paren) => current.push(Piece::Paren(paren)),
            _ => current.push(Piece::Operand(operand)),
        }
    }

    let mut chunks: Vec<Vec<Piece<'_, 'a>>> = vec![];
    let mut current: Vec<Piece<'_, 'a>> = vec![];
    flatten(op, &mut chunks, &mut current);
    if !current.is_empty() {
        chunks.push(current);
    }

    let write_chunk = |chunk: &[Piece<'_, 'a>], f: &mut CssFormatter<'_, 'a>| {
        for piece in chunk {
            match piece {
                Piece::Operand(operand) => write_component_value(operand, ctx, f),
                Piece::Paren(paren) => write_less_parenthesized_operation(paren, ctx, f),
                Piece::Op(op) => write!(f, token(op)),
                Piece::Space => write!(f, " "),
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

/// A Less parenthesized operation (`(@a - @b)`):
/// its own group, so when it breaks the `(` / `)` land on their own lines
/// with the inner operation indented one level (Prettier's `printParenthesizedValueGroup`).
/// When it fits, it stays inline (`(@a - @b)`).
fn write_less_parenthesized_operation<'a>(
    paren: &LessParenthesizedOperation<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    if ctx.no_break {
        write!(f, "(");
        write_component_value(&paren.operation, ctx, f);
        write!(f, ")");
        return;
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(
            f,
            indent(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write!(f, soft_line_break());
                write_component_value(&paren.operation, ctx, f);
            }))
        );
        write!(f, soft_line_break());
    });
    write!(f, [text("("), group(&body), text(")")]);
}

/// Whether a calc operand has ONE pair of source parens of its own to restore.
///
/// `oxc-css-parser` folds `(a - b) * c` into nested `Calc` nodes whose spans EXCLUDE the parens,
/// so they must be recovered from the source
/// (postcss keeps them as a `value-paren_group` and Prettier preserves them).
/// Only an operand position can do this safely:
/// at the top of `calc(...)` the function's own parens are indistinguishable from a redundant pair.
fn calc_operand_has_own_parens(operand: &ComponentValue<'_>, f: &CssFormatter<'_, '_>) -> bool {
    let span = to_span(operand.span());
    let source_len = u32::try_from(f.context().source_text().len()).unwrap_or(u32::MAX);
    own_paren_layers(span.start, span.end, 0, source_len, f) >= 1
}

/// Layers of source parens that belong to a function-argument group itself.
///
/// `oxc-css-parser` drops bare parens around argument values:
/// `min(((@a)), @b)` parses the first argument as just `@a`,
/// and a `Calc` argument's span excludes its outermost source parens (`max(((a - b) / 2), 0)`).
/// postcss keeps every pair as a `value-paren_group`, so Prettier prints them all.
/// The scan is bounded by the function's own parens (`region`).
fn group_own_paren_layers(
    group: &[ComponentValue<'_>],
    region_start: u32,
    region_end: u32,
    f: &CssFormatter<'_, '_>,
) -> u32 {
    let (Some(first), Some(last)) = (group.first(), group.last()) else { return 0 };
    let start = to_span(first.span()).start;
    let end = to_span(last.span()).end;
    if start < region_start || end > region_end {
        return 0;
    }
    own_paren_layers(start, end, region_start, region_end, f)
}

/// How many pairs of source parens around `start..end` (within `region`)
/// belong to that span itself.
///
/// The span may already contain unbalanced parens belonging to CHILD operands,
/// whose own pairs also sit outside their spans (`(a - 1) * b` spans from `a`).
/// Those are reprinted when the child's own chunk is written;
/// the span owns a pair only when a further `(`/`)` exists beyond what the children account for.
fn own_paren_layers(
    start: u32,
    end: u32,
    region_start: u32,
    region_end: u32,
    f: &CssFormatter<'_, '_>,
) -> u32 {
    let source = f.context().source_text();
    let bytes = source.as_bytes();

    // The adjacency scans stop at the first non-paren byte, so they are cheap.
    // Run them first and bail out before the O(span) balance scan
    // (in plain CSS there is almost never an adjacent paren at all).
    let opens = i32::try_from(
        bytes[region_start as usize..start as usize]
            .iter()
            .rev()
            .filter(|b| !b.is_ascii_whitespace())
            .take_while(|&&b| b == b'(')
            .count(),
    )
    .unwrap_or(0);
    if opens == 0 {
        return 0;
    }

    let closes = i32::try_from(
        bytes[end as usize..region_end as usize]
            .iter()
            .filter(|b| !b.is_ascii_whitespace())
            .take_while(|&&b| b == b')')
            .count(),
    )
    .unwrap_or(0);
    if closes == 0 {
        return 0;
    }

    let mut depth = 0i32;
    let mut min_depth = 0i32;
    for &b in &bytes[start as usize..end as usize] {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                min_depth = min_depth.min(depth);
            }
            _ => {}
        }
    }
    // `[need_left x '('] span [need_right x ')']` balances to zero;
    // anything beyond that within the region is the span's own.
    let need_left = -min_depth;
    let need_right = depth - min_depth;

    u32::try_from((opens - need_left).min(closes - need_right).max(0)).unwrap_or(0)
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

    let groups = split_comma_groups(&func.args);

    if func.args.is_empty() {
        write!(f, ["(", ")"]);
        return;
    }

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

    let args_region = (name_span.end + 1, to_span(func.span()).end.saturating_sub(1));
    // One argument group, with the source parens oxc-css-parser dropped restored
    let write_arg_group = move |group_values: &[ComponentValue<'a>],
                                ctx: ValueContext<'a>,
                                f: &mut CssFormatter<'_, 'a>| {
        let layers = group_own_paren_layers(group_values, args_region.0, args_region.1, f);
        for _ in 0..layers {
            write!(f, "(");
        }
        write_comma_group(group_values, ctx, f);
        for _ in 0..layers {
            write!(f, ")");
        }
    };

    // `no_break` (composes `removeLines` / media-value flat text):
    // no break opportunities, args joined inline.
    if ctx.no_break {
        write!(f, "(");
        for (i, &(group_values, comma)) in groups.iter().enumerate() {
            write_arg_group(group_values, ctx, f);
            if i + 1 < groups.len() {
                write_group_comma(comma, f);
                write!(f, " ");
            }
        }
        write!(f, ")");
        return;
    }

    let groups_ref = &groups;
    let r_paren = to_span(func.span()).end.saturating_sub(1);
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
                let prev_end = groups_ref[i - 1].0.last().map_or(0, |v| to_span(v.span()).end);
                let next_start = group_values.first().map_or(prev_end, |v| to_span(v.span()).start);
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
            write_arg_group(group_values, arg_ctx, f);
            // `has_trailing_comma`: the kept `var(--x /* c */,)` comma counts too
            if i + 1 < groups_ref.len() || has_trailing_comma {
                write_group_comma(comma, f);
            }
        }
        // Comments between the last argument and `)` wrap as fill items;
        // `//` comments stay glued to the argument (their hardline follows).
        let tail: &'a [comments::CssComment] = f.context().comments().take_before(r_paren);
        if tail.iter().any(|c| c.inline) {
            for &comment in tail {
                write!(f, " ");
                comments::write_single_comment(comment, f);
                if comment.inline {
                    write!(f, [expand_parent(), hard_line_break()]);
                }
            }
        } else if !tail.is_empty() {
            let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                let mut filler = f.fill();
                // Anchor entry: lets the first comment's separator attach it
                // to the last argument (a fill ignores the first separator).
                filler.entry(&soft_line_break_or_space(), &format_with(|_| {}));
                for &comment in tail {
                    let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        comments::write_single_comment(comment, f);
                        if comment.inline {
                            write!(f, [expand_parent(), hard_line_break()]);
                        }
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
            write_str(str, f);
            for modifier in &url.modifiers {
                write!(f, " ");
                let span = to_span(modifier.span());
                write!(f, text(source.text_for(&span)));
            }
        }
        // Quoted url with interpolation: requote the outer quotes only
        Some(UrlValue::Str(istr)) => {
            let span = to_span(istr.span());
            write_requoted_verbatim(source.text_for(&span), f);
        }
        // Unquoted url contents are printed verbatim, including inner spaces
        _ => {
            let url_span = to_span(url.span());
            let inner_start = to_span(url.name.span()).end + 1;
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
