//! Component value printing.
//!
//! Ports Prettier's `print/comma-separated-value-group.js`,
//! `print/parenthesized-value-group.js` and `print/misc.js` onto raffia's
//! flat `ComponentValue` streams (which mirror postcss-values-parser tokens:
//! commas and solidi appear as `Delimiter` components).

use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_formatter_core::{
    Buffer, Format,
    builders::{
        group, hard_line_break, indent, soft_line_break, soft_line_break_or_space, space, text,
        token,
    },
    write,
};
use raffia::{
    Spanned,
    ast::{ComponentValue, Delimiter, DelimiterKind, Dimension, Function, Number, Str, Url},
};

use crate::{
    format::to_span,
    print::{CssFormatter, format_with},
};

/// Prettier's `css-units-list`: lowercase -> canonical casing.
fn canonical_unit(lowercased: &str) -> Option<&'static str> {
    Some(match lowercased {
        "em" => "em",
        "rem" => "rem",
        "ex" => "ex",
        "rex" => "rex",
        "cap" => "cap",
        "rcap" => "rcap",
        "ch" => "ch",
        "rch" => "rch",
        "ic" => "ic",
        "ric" => "ric",
        "lh" => "lh",
        "rlh" => "rlh",
        "vw" => "vw",
        "svw" => "svw",
        "lvw" => "lvw",
        "dvw" => "dvw",
        "vh" => "vh",
        "svh" => "svh",
        "lvh" => "lvh",
        "dvh" => "dvh",
        "vi" => "vi",
        "svi" => "svi",
        "lvi" => "lvi",
        "dvi" => "dvi",
        "vb" => "vb",
        "svb" => "svb",
        "lvb" => "lvb",
        "dvb" => "dvb",
        "vmin" => "vmin",
        "svmin" => "svmin",
        "lvmin" => "lvmin",
        "dvmin" => "dvmin",
        "vmax" => "vmax",
        "svmax" => "svmax",
        "lvmax" => "lvmax",
        "dvmax" => "dvmax",
        "cm" => "cm",
        "mm" => "mm",
        "q" => "Q",
        "in" => "in",
        "pt" => "pt",
        "pc" => "pc",
        "px" => "px",
        "deg" => "deg",
        "grad" => "grad",
        "rad" => "rad",
        "turn" => "turn",
        "s" => "s",
        "ms" => "ms",
        "hz" => "Hz",
        "khz" => "kHz",
        "dpi" => "dpi",
        "dpcm" => "dpcm",
        "dppx" => "dppx",
        "x" => "x",
        "cqw" => "cqw",
        "cqh" => "cqh",
        "cqi" => "cqi",
        "cqb" => "cqb",
        "cqmin" => "cqmin",
        "cqmax" => "cqmax",
        "fr" => "fr",
        _ => return None,
    })
}

/// Prettier's `printNumber` + `printCssNumber` (trailing `.0` removal included).
pub fn print_css_number(raw: &str) -> Cow<'_, str> {
    if raw.len() == 1 {
        return Cow::Borrowed(raw);
    }
    let lowered = raw.cow_to_ascii_lowercase();

    // Split off scientific notation exponent.
    let (mantissa, exponent) = match lowered.find('e') {
        Some(idx) => (&lowered[..idx], Some(&lowered[idx + 1..])),
        None => (&lowered[..], None),
    };

    // Normalize exponent: remove `+` and leading zeroes; drop `e0`.
    let exponent = exponent.map(|exp| {
        let (sign, digits) = match exp.as_bytes().first() {
            Some(b'+') => ("", &exp[1..]),
            Some(b'-') => ("-", &exp[1..]),
            _ => ("", exp),
        };
        let digits = digits.trim_start_matches('0');
        if digits.is_empty() { String::new() } else { format!("e{sign}{digits}") }
    });

    // Normalize mantissa (the sign, including `+`, is kept).
    let (sign, digits) = match mantissa.as_bytes().first() {
        Some(b'+') => ("+", &mantissa[1..]),
        Some(b'-') => ("-", &mantissa[1..]),
        _ => ("", mantissa),
    };
    let mut digits = digits.to_string();
    if digits.contains('.') {
        // Remove extraneous trailing decimal zeroes and the trailing dot.
        let trimmed = digits.trim_end_matches('0');
        let trimmed = trimmed.strip_suffix('.').unwrap_or(trimmed);
        digits = trimmed.to_string();
    }
    // Make sure numbers always start with a digit.
    if digits.starts_with('.') {
        digits.insert(0, '0');
    } else if digits.is_empty() {
        digits.push('0');
    }

    let exponent = exponent.unwrap_or_default();
    let result = format!("{sign}{digits}{exponent}");
    if result == raw { Cow::Borrowed(raw) } else { Cow::Owned(result) }
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

pub fn write_number<'a>(number: &Number<'a>, f: &mut CssFormatter<'_, 'a>) {
    match print_css_number(number.raw) {
        Cow::Borrowed(s) => write!(f, text(s)),
        Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
    }
}

pub fn write_dimension<'a>(dimension: &Dimension<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_number(&dimension.value, f);
    print_unit(dimension.unit.raw, f);
}

/// Prettier's `printString`: re-quote according to `singleQuote` unless the
/// content contains quotes that would need extra escaping.
pub fn write_str<'a>(str: &Str<'a>, f: &mut CssFormatter<'_, 'a>) {
    let raw = str.raw;
    let single_quote = f.options().single_quote.value();
    let content = &raw[1..raw.len() - 1];

    let (preferred, alternate) = if single_quote { ('\'', '"') } else { ('"', '\'') };
    let mut preferred_count = 0usize;
    let mut alternate_count = 0usize;
    for b in content.bytes() {
        if b == preferred as u8 {
            preferred_count += 1;
        } else if b == alternate as u8 {
            alternate_count += 1;
        }
    }
    let enclosing = if preferred_count > alternate_count { alternate } else { preferred };

    if raw.as_bytes()[0] == enclosing as u8 {
        write!(f, text(raw));
        return;
    }

    // makeString: flip escapes as needed.
    let other = if enclosing == '"' { '\'' } else { '"' };
    let mut out = String::with_capacity(raw.len());
    out.push(enclosing);
    let mut chars = content.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(next @ ('"' | '\'' | '\\')) => {
                    if next == other {
                        out.push(other);
                    } else {
                        out.push('\\');
                        out.push(next);
                    }
                }
                Some(next) => {
                    out.push('\\');
                    out.push(next);
                }
                None => out.push('\\'),
            },
            c if c == enclosing => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(enclosing);
    write!(f, text(f.allocator().alloc_str(&out)));
}

/// Re-quotes the OUTER quotes of a raw quoted string per `singleQuote`,
/// keeping the content (e.g. `#{...}` interpolation) verbatim.
pub fn write_requoted_verbatim<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    if raw.len() < 2 || !(raw.starts_with('\'') || raw.starts_with('"')) {
        write!(f, text(raw));
        return;
    }
    let content = &raw[1..raw.len() - 1];
    // Interpolated strings whose content has quote characters keep their
    // original quotes (the inner quotes confuse the preference count) —
    // but bare quoted numbers inside `#{}` are normalized (postcss sees
    // them as unquoted numbers).
    if content.contains("#{") && (content.contains('"') || content.contains('\'')) {
        // The outer quote re-appearing inside the content splits the string
        // in postcss; every piece gets requoted to the preferred quote.
        let outer = raw.as_bytes()[0] as char;
        let preferred = if f.options().single_quote.value() { '\'' } else { '"' };
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
    let single_quote = f.options().single_quote.value();
    let (preferred, alternate) = if single_quote { ('\'', '"') } else { ('"', '\'') };
    let mut pc = 0usize;
    let mut ac = 0usize;
    for b in content.bytes() {
        if b == preferred as u8 {
            pc += 1;
        } else if b == alternate as u8 {
            ac += 1;
        }
    }
    let enclosing = if pc > ac { alternate } else { preferred };
    if raw.as_bytes()[0] == enclosing as u8 {
        write!(f, text(raw));
    } else {
        let out = format!("{enclosing}{content}{enclosing}");
        write!(f, text(f.allocator().alloc_str(&out)));
    }
}

/// Normalizes `".5"`-style quoted bare numbers inside an interpolated string.
fn normalize_quoted_numbers(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    // Skip the outer opening quote.
    out.push(bytes[0] as char);
    i += 1;
    let end = raw.len() - 1;
    while i < end {
        let b = bytes[i];
        if (b == b'"' || b == b'\'') && i + 1 < end {
            // Find the matching close within the content.
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

fn is_comma(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::Delimiter(Delimiter { kind: DelimiterKind::Comma, .. }))
}

fn is_solidus(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::Delimiter(Delimiter { kind: DelimiterKind::Solidus, .. }))
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
    use raffia::ast::FunctionName;
    match &func.name {
        FunctionName::Ident(raffia::ast::InterpolableIdent::Literal(ident)) => ident.raw,
        _ => "",
    }
}

/// Layout context for a value being printed.
#[derive(Clone, Copy, Default)]
pub struct ValueContext<'a> {
    /// Lowercased property name of the enclosing declaration, if any.
    pub decl_prop: Option<&'a str>,
    /// `composes` values never break (Prettier's `removeLines`).
    pub no_break: bool,
    /// SCSS map printed in key position: stays inline (Prettier's `isKey`).
    pub map_key: bool,
    /// A parenthesized comma list here breaks one item per line
    /// (only as a direct map-item value — `isSCSSMapItemNode` needs key-value
    /// pairs in the group or its grandparent).
    pub paren_break: bool,
    /// SCSS maps here always break (`$var:` values, function args, map items).
    pub map_break: bool,
    /// Comments before this bound are flushed as wrapped fill items at the
    /// end of the outermost comma group (declaration tail).
    pub tail_bound: Option<u32>,
    /// Inside `url(...)`: colons stay tight (`url(fbglyph:cross-outline)`).
    pub in_url: bool,
    /// Inside function/include arguments (comment-slot rules differ).
    pub in_args: bool,
    /// This component directly follows a `//` comment: a function call here
    /// gets Prettier's quirky double indent (args +2 levels, `)` +1).
    pub after_inline_comment: bool,
    /// A multi-line `raws.between` was printed before the value: Prettier's
    /// printer counts its full width, so the first trailing comment always
    /// wraps.
    pub tail_break: bool,
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
fn split_comma_groups<'b, 'a>(values: &'b [ComponentValue<'a>]) -> Vec<&'b [ComponentValue<'a>]> {
    let mut groups = vec![];
    let mut start = 0;
    for (i, v) in values.iter().enumerate() {
        if is_comma(v) {
            groups.push(&values[start..i]);
            start = i + 1;
        }
    }
    groups.push(&values[start..]);
    // A trailing comma produces an empty last group; Prettier drops it.
    if groups.len() > 1 && groups.last().is_some_and(|g| g.is_empty()) {
        groups.pop();
    }
    groups
}

/// Mirrors Prettier's top-level declaration value printing
/// (`value-root` -> `value-paren_group` without parens).
pub fn write_declaration_value<'a>(
    values: &[ComponentValue<'a>],
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    // A lone SCSS list/map IS the value (`$var: 1px 2px, 3px;`).
    if values.len() == 1
        && matches!(values[0], ComponentValue::SassList(_) | ComponentValue::SassMap(_))
    {
        crate::print::scss::write_top_level_value(&values[0], ctx, f);
        return;
    }

    let groups = split_comma_groups(values);

    if groups.len() == 1 {
        // Flattened to a single comma group.
        write_comma_group(groups[0], ctx, f);
        return;
    }

    // Multiple comma groups: `shouldBreakList` forces one per line when any
    // group has multiple elements (a real `value-comma_group` survives
    // flattening — comments count as members) and the property is not a
    // custom property.
    let value_start = to_span(values[0].span()).start;
    let value_end = to_span(values[values.len() - 1].span()).end;
    let has_comments =
        f.context().comments().iter_before(value_end).any(|c| c.span.start >= value_start);
    let force_hard_line = !ctx.decl_prop.is_some_and(|p| p.starts_with("--"))
        && (groups.iter().any(|g| g.len() > 1) || has_comments);

    write_value_groups(&groups, ctx, force_hard_line, true, f);
}

/// Top-level comma-group list layout, shared between flat component streams
/// and SCSS comma lists.
pub fn write_value_groups<'a>(
    groups: &[&[ComponentValue<'a>]],
    ctx: ValueContext<'a>,
    force_hard_line: bool,
    _top_level: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    // A single comma group never forces (it isn't a list).
    let force_hard_line = force_hard_line && groups.len() > 1;
    if force_hard_line {
        let body = format_with(|f: &mut CssFormatter<'_, 'a>| {
            write!(f, hard_line_break());
            for (i, group_values) in groups.iter().enumerate() {
                if i > 0 {
                    write!(f, ",");
                    write!(f, hard_line_break());
                }
                // The declaration tail belongs to the LAST group only.
                let gctx = if i + 1 < groups.len() {
                    ValueContext { tail_bound: None, ..ctx }
                } else {
                    ctx
                };
                // Comments between groups (e.g. after the previous comma)
                // fill together with the group they precede; `//` comments
                // (and leading own-line comments) keep their own line.
                let lead: Vec<crate::comments::CssComment> = group_values
                    .first()
                    .map(|first| {
                        f.context().comments().take_before(to_span(first.span()).start).to_vec()
                    })
                    .unwrap_or_default();
                if lead.is_empty() {
                    write_comma_group(group_values, gctx, f);
                } else if i == 0 {
                    let source = f.context().source_text();
                    for &comment in &lead {
                        let own_line = comment_is_own_line(comment, source);
                        crate::comments::write_single_comment(comment, f);
                        if comment.inline || own_line {
                            write!(f, hard_line_break());
                        } else {
                            write!(f, " ");
                        }
                    }
                    write_comma_group(group_values, gctx, f);
                } else {
                    // Prettier-fill the comments with the group they lead.
                    // Simulated with static widths: the group's fit must NOT
                    // include its declaration tail, which our entry would.
                    let width = u32::from(f.options().line_width.value());
                    let group_w =
                        group_values.first().zip(group_values.last()).map_or(0, |(first, last)| {
                            to_span(last.span()).end - to_span(first.span()).start
                        }) + u32::from(i + 1 < groups.len());
                    let mut x = 4u32; // hardline indent under the value
                    for (k, &comment) in lead.iter().enumerate() {
                        crate::comments::write_single_comment(comment, f);
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
            }
        });
        write!(f, indent(&body));
    } else {
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, soft_line_break());
            let mut filler = f.fill();
            for (i, group_values) in groups.iter().enumerate() {
                let is_last = i + 1 == groups.len();
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    if let Some(first) = group_values.first() {
                        flush_value_comments(to_span(first.span()).start, f);
                    }
                    write_comma_group(group_values, ctx, f);
                    if !is_last {
                        write!(f, ",");
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
pub fn comment_is_own_line(
    comment: crate::comments::CssComment,
    source: oxc_formatter_core::SourceText<'_>,
) -> bool {
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
pub fn flush_value_comments(upper_bound: u32, f: &mut CssFormatter<'_, '_>) -> bool {
    let mut last_inline = false;
    for &comment in f.context().comments().take_before(upper_bound) {
        crate::comments::write_single_comment(comment, f);
        if comment.inline {
            write!(f, [oxc_formatter_core::builders::expand_parent(), hard_line_break()]);
        } else {
            write!(f, " ");
        }
        last_inline = comment.inline;
    }
    last_inline
}

/// Emits pending comments that sit on the same line as the just-printed
/// content ending at `prev_end` (` // c` / ` /* c */`), up to `upper_bound`.
pub fn flush_same_line_comments_before(
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
        crate::comments::write_single_comment(comment, f);
        if comment.inline {
            write!(f, [oxc_formatter_core::builders::expand_parent()]);
            return;
        }
    }
}

/// Unbounded variant (used at container tails where everything pending
/// belongs to the container).
pub fn flush_same_line_comments(prev_end: u32, f: &mut CssFormatter<'_, '_>) {
    flush_same_line_comments_before(prev_end, u32::MAX, f);
}

/// Emits pending comments before `upper_bound` as ` /* c */` suffixes
/// (used after the last value component, before `;` / `!important`).
/// Returns the end offset of the last emitted comment.
pub fn flush_trailing_value_comments(
    upper_bound: u32,
    f: &mut CssFormatter<'_, '_>,
) -> Option<u32> {
    let mut last_end = None;
    for &comment in f.context().comments().take_before(upper_bound) {
        write!(f, " ");
        crate::comments::write_single_comment(comment, f);
        if comment.inline {
            write!(f, [oxc_formatter_core::builders::expand_parent(), hard_line_break()]);
        }
        last_end = Some(comment.span.end);
    }
    last_end
}

/// Mirrors Prettier's `printCommaSeparatedValueGroup`:
/// joins components with `line`, except for pairs that must stay tight.
pub fn write_comma_group<'a>(
    values: &[ComponentValue<'a>],
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    if values.is_empty() {
        return;
    }
    // Prettier's `flattenGroups`: a single-element comma group collapses to
    // the element itself (no extra group/indent level).
    if values.len() == 1 && ctx.tail_bound.is_none() {
        let after_inline = flush_value_comments(to_span(values[0].span()).start, f);
        let ctx =
            if after_inline { ValueContext { after_inline_comment: true, ..ctx } } else { ctx };
        write_component_value(&values[0], ctx, f);
        return;
    }
    // A single value WITH a tail bound routes through the fill below so its
    // trailing comments wrap.
    let source = f.context().source_text();
    let upper_bound = to_span(values[values.len() - 1].span()).end;
    // Grid values that break across source lines start on their own line
    // (Prettier's `didBreak` + `parts.unshift(hardline)`) — but breaks caused
    // by inline comments in the gap don't count (the comment-hardline branch
    // runs before the grid check in Prettier).
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
        // Snapshot of pending comments inside the value (for separator
        // decisions; consumption happens inside the entries).
        let pending: Vec<crate::comments::CssComment> =
            f.context().comments().iter_before(upper_bound).collect();
        let tail_bound = ctx.tail_bound;
        let ctx = ValueContext { tail_bound: None, ..ctx };
        let tail_comments: Vec<crate::comments::CssComment> = tail_bound
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
            // An inline comment trailing the PREVIOUS run forces a break.
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
            // Merge runs of tight / non-breaking-space components into one fill entry.
            let mut run_end = i + 1;
            while run_end < values.len()
                && matches!(
                    separator_between(values, run_end, ctx, source),
                    Separator::Tight | Separator::Space
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
                let after_inline = flush_value_comments(start, f);
                for (j, v) in run.iter().enumerate() {
                    if j > 0
                        && separator_between(values, run_start + j, ctx, source) == Separator::Space
                    {
                        write!(f, " ");
                    }
                    let vctx = if j == 0 && after_inline {
                        ValueContext { after_inline_comment: true, ..ctx }
                    } else {
                        ctx
                    };
                    write_component_value(v, vctx, f);
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
                        crate::comments::write_single_comment(comment, f);
                        write!(f, oxc_formatter_core::builders::expand_parent());
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
                            crate::comments::write_single_comment(comment, f);
                            if comment.inline {
                                write!(f, oxc_formatter_core::builders::expand_parent());
                            }
                        }
                    });
                    filler.entry(&soft_line_break_or_space(), &entry);
                }
            }
            i = run_end;
        }
        // Declaration-tail comments wrap as fill items.
        for (k, &comment) in tail_comments.iter().enumerate() {
            let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                f.context().comments().take_before(comment.span.end);
                crate::comments::write_single_comment(comment, f);
                if comment.inline {
                    write!(f, oxc_formatter_core::builders::expand_parent());
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
            raffia::token::Token::Plus(_)
                | raffia::token::Token::Minus(_)
                | raffia::token::Token::Asterisk(_)
                | raffia::token::Token::Percent(_)
                | raffia::token::Token::Solidus(_)
        )
    } else {
        false
    }
}

fn raw_token<'b, 'a>(value: &'b ComponentValue<'a>) -> Option<&'b raffia::token::Token<'a>> {
    if let ComponentValue::TokenWithSpan(token) = value { Some(&token.token) } else { None }
}

/// Decides the separator BEFORE `values[i]` (i >= 1).
///
/// In Prettier's loop terms: `iNode` = `values[i - 1]`, `iNextNode` =
/// `values[i]`, `iPrevNode` = `values[i - 2]`, `iNextNextNode` = `values[i + 1]`.
fn separator_between(
    values: &[ComponentValue<'_>],
    i: usize,
    ctx: ValueContext<'_>,
    source: oxc_formatter_core::SourceText<'_>,
) -> Separator {
    let prev = &values[i - 1];
    let curr = &values[i];
    let prev_span = to_span(prev.span());
    let curr_span = to_span(curr.span());
    // `hasEmptyRawBefore(iNextNode)`
    let gap_empty = prev_span.end == curr_span.start;
    // `hasEmptyRawBefore(iNode)`
    let prev_gap_empty = i >= 2 && to_span(values[i - 2].span()).end == prev_span.start;

    // Grid: preserve source line structure.
    if ctx.is_grid() {
        let gap = source.bytes_range(prev_span.end, curr_span.start);
        if gap.contains(&b'\n') || gap.contains(&b'\r') {
            return Separator::Hard;
        }
        return Separator::Line;
    }

    // Solidus (`/`) spacing rules.
    if is_solidus(curr) || is_solidus(prev) {
        // Path-like streams (`-fb-url(/a/b.png)`): glued tokens stay glued.
        if is_solidus(&values[0]) && gap_empty {
            return Separator::Tight;
        }
        // Fully glued `/` (`center/80%`, `12px/1.5`): one postcss word.
        if is_solidus(curr)
            && gap_empty
            && values.get(i + 1).is_none_or(|n| to_span(curr.span()).end == to_span(n.span()).start)
        {
            return Separator::Tight;
        }
        if is_solidus(prev) && gap_empty && prev_gap_empty {
            return Separator::Tight;
        }
        // `font: 12px/1.5` and custom properties: keep tight when the source
        // is tight around a font-size-capable node.
        if ctx.is_font_or_custom() {
            if is_solidus(curr) && gap_empty && is_possible_font_size(prev) {
                return Separator::Tight;
            }
            if is_solidus(prev) && prev_gap_empty && i >= 2 && is_possible_font_size(&values[i - 2])
            {
                return Separator::Tight;
            }
        }

        // Leading `/` (e.g. `-fb-url(/abs/path)`).
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

    // Less lookups: `@var [@result]` loses the gap (`var [@lookup]` rule).
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

    // Raw token punctuation: `:` hugs left and spaces right, braces hug
    // their contents (custom-property JSON-ish values).
    {
        use raffia::token::Token;
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
            // No space after `:` inside `url(...)`.
            if ctx.in_url && matches!(raw_token(prev), Some(Token::Colon(_))) {
                return Separator::Tight;
            }
            return Separator::Space;
        }
        // `*` is never glued (Prettier excludes multiplication from the
        // tight rules) — except Tailwind's `ident-*` pattern.
        if matches!(raw_token(curr), Some(Token::Asterisk(_)))
            || matches!(raw_token(prev), Some(Token::Asterisk(_)))
        {
            if gap_empty
                && matches!(prev, ComponentValue::InterpolableIdent(raffia::ast::InterpolableIdent::Literal(id)) if id.raw.ends_with('-'))
            {
                return Separator::Tight;
            }
            return Separator::Line;
        }
        // Token-level division, mirroring the structural rules: tight only
        // when glued in the source and no word/function neighbors.
        let is_solidus_tok =
            |v: &ComponentValue<'_>| matches!(raw_token(v), Some(Token::Solidus(_)));
        // A leading `/` makes the stream path-like (`-fb-url(/a/b.png)`):
        // glued tokens stay glued.
        if is_solidus_tok(&values[0]) && gap_empty {
            return Separator::Tight;
        }
        // Token-level font shorthand rule (`--font: var(--size)/2` in
        // custom properties, where the value is a raw token stream).
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

    // Prettier: an at-word placeholder glued to a paren group gets a
    // `softline` (`${fn}(30px)` may break BEFORE the parens, keeping the
    // paren group intact on the next line).
    if gap_empty
        && matches!(raw_token(prev), Some(raffia::token::Token::AtKeyword(_)))
        && matches!(
            curr,
            ComponentValue::SassParenthesizedExpression(_) | ComponentValue::SassMap(_)
        )
    {
        return Separator::SoftBreak;
    }

    // Raw token fallbacks (postcss-values would have produced operator/word
    // tokens): keep gap-free neighbors tight (`10em+12em`, `-fb-url(/a/b)`).
    if gap_empty
        && (matches!(prev, ComponentValue::TokenWithSpan(_))
            || matches!(curr, ComponentValue::TokenWithSpan(_))
            || matches!(prev, ComponentValue::Delimiter(_))
            || matches!(curr, ComponentValue::Delimiter(_)))
    {
        return Separator::Tight;
    }

    // Math operators with surrounding spaces: `a + b` keeps the space before
    // the operator non-breaking and breaks after it (Prettier's
    // `isNextMathOperator` merge).
    if math_op_token(curr) {
        return Separator::Space;
    }

    Separator::Line
}

/// Dispatch a single component value.
pub fn write_component_value<'a>(
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
            match hex.raw.cow_to_ascii_lowercase() {
                Cow::Borrowed(s) => write!(f, text(s)),
                Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
            }
        }
        ComponentValue::InterpolableIdent(raffia::ast::InterpolableIdent::Literal(ident)) => {
            if is_wide_keyword(ident.raw) {
                match ident.raw.cow_to_ascii_lowercase() {
                    Cow::Borrowed(s) => write!(f, text(s)),
                    Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
                }
            } else {
                write!(f, text(ident.raw));
            }
        }
        ComponentValue::InterpolableStr(raffia::ast::InterpolableStr::Literal(str)) => {
            write_str(str, f);
        }
        ComponentValue::InterpolableStr(istr) => {
            let span = to_span(istr.span());
            write_requoted_verbatim(source.text_for(&span), f);
        }
        ComponentValue::Function(func) => write_function(func, ctx, f),
        ComponentValue::Calc(calc) => write_calc(calc, ctx, f),
        ComponentValue::SassMap(map) => crate::print::scss::write_sass_map(map, ctx, f),
        ComponentValue::SassList(list) => crate::print::scss::write_sass_list(list, ctx, f),
        ComponentValue::SassParenthesizedExpression(paren) => {
            // `$var: ((a, b), (c, d))` / map item values: one item per line.
            if ctx.paren_break {
                // Map-item values always break (`isSCSSMapItemNode`),
                // with a trailing comma per option.
                let inner_ctx = ValueContext { paren_break: false, ..ctx };
                let trailing = f.options().allow_trailing_comma();
                let elements: &[ComponentValue<'a>] = match &*paren.expr {
                    ComponentValue::SassList(list) if list.comma_spans.is_some() => &list.elements,
                    expr => std::slice::from_ref(expr),
                };
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
                    if trailing && (ctx.map_key || ctx.paren_break) {
                        write!(f, oxc_formatter_core::builders::if_group_breaks(&text(",")));
                    }
                } else {
                    write_component_value(&paren.expr, inner_ctx, f);
                }
                // Inline comments before `)` stay inside, forcing the break.
                for &comment in f.context().comments().take_before(r_paren) {
                    write!(f, " ");
                    crate::comments::write_single_comment(comment, f);
                    if comment.inline {
                        write!(f, oxc_formatter_core::builders::expand_parent());
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
            let keyword_not = matches!(unary.op.kind, raffia::ast::SassUnaryOperatorKind::Not);
            match unary.op.kind {
                raffia::ast::SassUnaryOperatorKind::Plus => write!(f, "+"),
                raffia::ast::SassUnaryOperatorKind::Minus => write!(f, "-"),
                raffia::ast::SassUnaryOperatorKind::Not => write!(f, ["not", " "]),
            }
            // Keep the source gap (`- pow(2, 2)` stays spaced); `not` already
            // wrote its single separator space.
            if !keyword_not && to_span(unary.op.span()).end != to_span(unary.expr.span()).start {
                write!(f, " ");
            }
            write_component_value(&unary.expr, ctx, f);
        }
        ComponentValue::SassBinaryExpression(binary) => write_sass_binary(binary, ctx, f),
        ComponentValue::SassKeywordArgument(kw) => {
            // Block values (maps/parens) hug the colon without pair indent.
            if matches!(
                &*kw.value,
                ComponentValue::SassMap(_) | ComponentValue::SassParenthesizedExpression(_)
            ) {
                let name_span = to_span(kw.name.span());
                write!(f, [text(source.text_for(&name_span)), ":", " "]);
                write_component_value(&kw.value, ctx, f);
                return;
            }
            // `$name: value` may break after the colon when too long.
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
            crate::print::statement::write_block(&nesting.block, f);
        }
        ComponentValue::SassArbitraryArgument(arg) => {
            write_component_value(&arg.value, ctx, f);
            write!(f, "...");
        }
        ComponentValue::Url(url) => write_url(url, f),
        // Less lookups / nth bracket blocks: `[...]` hugs its contents.
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
        // Everything else (Sass/Less constructs, interpolations, token
        // fallbacks): print the source verbatim until ported structurally.
        _ => {
            if crate::print::less::write_less_component_value(value, f) {
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
    out: &mut Vec<(&'b ComponentValue<'a>, Option<&'b raffia::ast::SassBinaryOperator>)>,
) {
    if let ComponentValue::SassBinaryExpression(binary) = expr {
        flatten_sass_binary(&binary.left, out);
        // Attach this operator to the last operand collected.
        if let Some(last) = out.last_mut() {
            last.1 = Some(&binary.op);
        }
        flatten_sass_binary(&binary.right, out);
    } else {
        out.push((expr, None));
    }
}

/// SCSS binary expression: a flat fill of `operand op` entries; spaces follow
/// the source around each operator, breaking after operators.
fn write_sass_binary<'a>(
    binary: &raffia::ast::SassBinaryExpression<'a>,
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
            let mut run_end = i;
            while let Some(op) = parts[run_end].1 {
                let op_span = to_span(op.span());
                let next_start = to_span(parts[run_end + 1].0.span()).start;
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
                        let op_span = to_span(op.span());
                        let operand_end = to_span(operand.span()).end;
                        let next_start = run.get(j + 1).map(|(next, _)| to_span(next.span()).start);
                        // `op(paren)` fuses like a postcss function token,
                        // which always gets a space before it; word-like
                        // operands force spaces (`$a + $b` even when glued).
                        let fuses_paren = next_start == Some(op_span.end);
                        let op_text = source.text_for(&op_span);
                        let wordish = matches!(op_text, "+" | "-")
                            && (is_word_like(operand)
                                || is_func_like(operand)
                                || run.get(j + 1).is_some_and(|(next, _)| {
                                    is_word_like(next) || is_func_like(next)
                                }));
                        if operand_end != op_span.start
                            || wordish
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
            filler.entry(&soft_line_break_or_space(), &content);
            i = run_end + 1;
        }
        filler.finish();
    });
    write!(f, group(&indent(&body)));
}

/// Calc expression (`a + b` inside `calc(...)`): spaces around `+`/`-` follow
/// the source (invalid syntax otherwise), with a break opportunity after the
/// operator only.
fn write_calc<'a>(
    calc: &raffia::ast::Calc<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    use raffia::ast::CalcOperatorKind;
    let left_end = to_span(calc.left.span()).end;
    let op_span = to_span(calc.op.span());
    let right_start = to_span(calc.right.span()).start;
    let gap_before_op = left_end != op_span.start;
    let gap_after_op = op_span.end != right_start;

    let op_str: &'static str = match calc.op.kind {
        CalcOperatorKind::Plus => "+",
        CalcOperatorKind::Minus => "-",
        CalcOperatorKind::Multiply => "*",
        CalcOperatorKind::Division => "/",
    };

    let head = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write_calc_operand(&calc.left, ctx, f);
        if gap_before_op {
            write!(f, " ");
        }
        write!(f, token(op_str));
        if !gap_after_op {
            write_calc_operand(&calc.right, ctx, f);
        }
    });

    if gap_after_op {
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let mut filler = f.fill();
            let right = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write_calc_operand(&calc.right, ctx, f);
            });
            filler.entry(&soft_line_break_or_space(), &head);
            filler.entry(&soft_line_break_or_space(), &right);
            filler.finish();
        });
        write!(f, group(&indent(&body)));
    } else {
        head.fmt(f);
    }
}

/// A calc operand, restoring ONE pair of parens from the source when present.
///
/// raffia folds `(a - b) * c` into nested `Calc` nodes whose spans EXCLUDE
/// the parens, so they must be recovered from the source (postcss keeps them
/// as a `value-paren_group` and Prettier preserves them). Only an operand
/// position can do this safely: at the top of `calc(...)` the function's own
/// parens are indistinguishable from a redundant pair.
fn write_calc_operand<'a>(
    operand: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let bytes = source.as_bytes();
    let span = to_span(operand.span());

    // The span may already contain unbalanced parens belonging to CHILD
    // operands, whose own pairs also sit outside their spans
    // (`(a - 1) * b` spans from `a`). Those are reprinted by the child's own
    // `write_calc_operand` call; the operand has a pair of its OWN only when
    // a further `(`/`)` exists beyond what the children account for.
    let mut depth = 0i32;
    let mut min_depth = 0i32;
    for &b in &bytes[span.start as usize..span.end as usize] {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                min_depth = min_depth.min(depth);
            }
            _ => {}
        }
    }
    // `[need_left x '('] span [need_right x ')']` balances to zero.
    let need_left = -min_depth;
    let need_right = depth - min_depth;

    let own_open = {
        let mut skip = need_left;
        bytes[..span.start as usize]
            .iter()
            .rev()
            .filter(|b| !b.is_ascii_whitespace())
            .take_while(|&&b| b == b'(')
            .any(|_| {
                if skip == 0 {
                    return true;
                }
                skip -= 1;
                false
            })
    };
    let own_close = {
        let mut skip = need_right;
        bytes[span.end as usize..]
            .iter()
            .filter(|b| !b.is_ascii_whitespace())
            .take_while(|&&b| b == b')')
            .any(|_| {
                if skip == 0 {
                    return true;
                }
                skip -= 1;
                false
            })
    };
    let wrapped = own_open && own_close;

    if wrapped {
        write!(f, "(");
    }
    write_component_value(operand, ctx, f);
    if wrapped {
        write!(f, ")");
    }
}

/// Function call: `name(` + args + `)`.
/// Mirrors `value-func` + `value-paren_group` with parens.
fn write_function<'a>(func: &Function<'a>, ctx: ValueContext<'a>, f: &mut CssFormatter<'_, 'a>) {
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
        // Single plain argument: verbatim (matches the `Url` node path).
        if groups.len() == 1
            && groups[0].len() == 1
            && matches!(
                groups[0][0],
                ComponentValue::InterpolableIdent(_) | ComponentValue::TokenWithSpan(_)
            )
        {
            let span = to_span(groups[0][0].span());
            write!(f, text(source.text_for(&span)));
        } else {
            let url_ctx = ValueContext { in_url: true, ..ctx };
            for (i, group_values) in groups.iter().enumerate() {
                if i > 0 {
                    write!(f, [",", " "]);
                }
                write_comma_group(group_values, url_ctx, f);
            }
        }
        write!(f, ")");
        return;
    }

    let groups_ref = &groups;
    let r_paren = to_span(func.span()).end.saturating_sub(1);
    let extra_indent = ctx.after_inline_comment;
    // `var(--baz,)`: an empty fallback is meaningful, so a source trailing
    // comma is preserved — for `var()` ONLY (Prettier's `printTrailingComma`
    // checks `isVarFunctionNode`; every other function drops it).
    let has_trailing_comma = func.args.last().is_some_and(is_comma)
        && function_name_text(func).eq_ignore_ascii_case("var");
    // Function arguments are "map item" positions (maps break).
    let ctx = ValueContext {
        map_break: true,
        paren_break: false,
        in_args: true,
        after_inline_comment: false,
        ..ctx
    };
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let source = f.context().source_text();
        write!(f, soft_line_break());
        for (i, group_values) in groups_ref.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                // Preserve a blank line between argument groups, but only
                // after a multi-part group (Prettier checks comma_groups).
                let prev_end = groups_ref[i - 1].last().map_or(0, |v| to_span(v.span()).end);
                let next_start = group_values.first().map_or(prev_end, |v| to_span(v.span()).start);
                let next_start = f
                    .context()
                    .comments()
                    .peek()
                    .map_or(next_start, |c| c.span.start.min(next_start));
                if prev_end != 0
                    && groups_ref[i - 1].len() > 1
                    && crate::comments::classify_gap(source.bytes_range(prev_end, next_start))
                        == crate::comments::Gap::Blank
                {
                    write!(f, oxc_formatter_core::builders::empty_line());
                } else {
                    write!(f, soft_line_break_or_space());
                }
            }
            write_comma_group(group_values, ctx, f);
        }
        if has_trailing_comma {
            write!(f, ",");
        }
        // Comments between the last argument and `)` wrap as fill items;
        // `//` comments stay glued to the argument (their hardline follows).
        let tail: Vec<crate::comments::CssComment> =
            f.context().comments().take_before(r_paren).to_vec();
        if tail.iter().any(|c| c.inline) {
            for &comment in &tail {
                write!(f, " ");
                crate::comments::write_single_comment(comment, f);
                if comment.inline {
                    write!(f, [oxc_formatter_core::builders::expand_parent(), hard_line_break()]);
                }
            }
        } else if !tail.is_empty() {
            let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                let mut filler = f.fill();
                // Anchor entry: lets the first comment's separator attach it
                // to the last argument (a fill ignores the first separator).
                filler.entry(&soft_line_break_or_space(), &format_with(|_| {}));
                for &comment in &tail {
                    let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        crate::comments::write_single_comment(comment, f);
                        if comment.inline {
                            write!(
                                f,
                                [oxc_formatter_core::builders::expand_parent(), hard_line_break()]
                            );
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
            if extra_indent {
                // Prettier's comment-in-paren-group quirk: arguments sit two
                // levels deep and the `)` one level deep.
                write!(f, indent(&indent(&body)));
                write!(f, indent(&soft_line_break()));
            } else {
                write!(f, indent(&body));
                write!(f, soft_line_break());
            }
            write!(f, ")");
        }))
    );
}

/// `url(...)`: contents are never reformatted.
pub fn write_url<'a>(url: &Url<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(url.name.span());
    let name = source.text_for(&name_span);
    match name.cow_to_ascii_lowercase() {
        Cow::Borrowed(s) => write!(f, text(s)),
        Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
    }
    write!(f, "(");
    match &url.value {
        Some(raffia::ast::UrlValue::Str(raffia::ast::InterpolableStr::Literal(str))) => {
            write_str(str, f);
            for modifier in &url.modifiers {
                write!(f, " ");
                let span = to_span(modifier.span());
                write!(f, text(source.text_for(&span)));
            }
        }
        // Quoted url with interpolation: requote the outer quotes only.
        Some(raffia::ast::UrlValue::Str(istr)) => {
            let span = to_span(istr.span());
            write_requoted_verbatim(source.text_for(&span), f);
        }
        // Unquoted url contents are printed verbatim, including inner spaces.
        _ => {
            let url_span = to_span(url.span());
            let inner_start = to_span(url.name.span()).end + 1;
            // raffia's url span may stop before trailing padding; scan to `)`.
            let bytes = source.as_bytes();
            let mut close = url_span.end.saturating_sub(1) as usize;
            while close < bytes.len() && bytes[close] != b')' {
                close += 1;
            }
            let inner_end = u32::try_from(close).unwrap_or(url_span.end.saturating_sub(1));
            if inner_start < inner_end {
                let inner = source.slice_range(inner_start, inner_end);
                // Escaped parens keep their padding verbatim; otherwise trim.
                if inner.contains("\\(") || inner.contains("\\)") {
                    write!(f, text(inner));
                } else {
                    let trimmed = inner.trim();
                    // `url($a+$b)`: SCSS concatenation gets spaced.
                    if trimmed.contains('$') && trimmed.contains('+') && !trimmed.contains(' ') {
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
