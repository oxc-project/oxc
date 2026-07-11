//! At-rule printing.
//! Ports Prettier's `css-atrule` case and the `postcss-media-query-parser` printing (`media-*` cases).

use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_css_parser::{
    ast::{
        AtRule, AtRulePrelude, ComponentValue, CustomMediaValue, ImportPrelude, ImportPreludeHref,
        ImportPreludeSupportsKind, InterpolableStr, KeyframesName, LessImportPrelude,
        MediaCondition, MediaConditionKind, MediaFeature, MediaFeatureComparisonKind,
        MediaFeatureName, MediaInParens, MediaInParensKind, MediaQuery, MediaQueryList,
        NamespacePreludeUri, SassAtRootKind, SimpleBlock, SupportsCondition, SupportsConditionKind,
        SupportsInParens, SupportsInParensKind, TokenSeq, UnknownAtRulePrelude,
    },
    pos::Span,
    token::{Token, TokenWithSpan},
};

use oxc_formatter_core::{
    Buffer, FormatElement, arena_cow_str,
    builders::{
        empty_line, group, hard_line_break, indent, soft_line_break, soft_line_break_or_space,
        space, text,
    },
    write,
};

use crate::{
    comments,
    format::to_span,
    print::{
        CssFormatter, format_with, normalize_whitespace, scss, selector, statement,
        value::{self, ValueContext},
        write_maybe_lowercase,
    },
};

/// Mirrors Prettier's `css-atrule`.
pub(super) fn write_at_rule<'a>(at_rule: &AtRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    write!(f, "@");
    let name_span = to_span(at_rule.name.span());
    write_maybe_lowercase(source.text_for(&name_span), f);

    // Comments inside the params:
    // postcss keeps them embedded in the params string / media tokens; reconstruct from the source.
    let region_end =
        statement::params_region_end(at_rule.block.as_ref(), to_span(at_rule.span()).end, f);
    let has_params_comments = f
        .context()
        .comments()
        .peek()
        .is_some_and(|c| c.span.start >= name_span.end && c.span.end <= region_end);

    // `@apply` with Tailwind sorting enabled:
    // the class list becomes one `TailwindClass` element,
    // sorted in a host-supplied batch after IR construction
    // (mirrors prettier-plugin-tailwindcss's `transformCss`, which matches `name === "apply"` case-sensitively).
    // Params containing comments are left to the normal printers, sorting would corrupt them.
    if f.options().sort_tailwindcss
        && at_rule.name.raw == "apply"
        && at_rule.block.is_none()
        && !has_params_comments
        && let Some(prelude) = &at_rule.prelude
    {
        let prelude_span = to_span(prelude.span());
        if write_apply_prelude(source.text_for(&prelude_span), f) {
            write!(f, ";");
            return;
        }
    }

    if let Some(prelude) = &at_rule.prelude {
        // Prettier's parser hands at-rule params to sub-parsers
        // only for a fixed allowlist (`parser-postcss.js`);
        // for everything else `node.params` stays a plain STRING that the printer emits verbatim.
        // oxc-css-parser's Unknown prelude mostly maps to that "everything else"
        // (`@apply`, `@tailwind`, `@custom-variant`, `@source`, …),
        // re-spacing its tokens corrupts constructs like Tailwind's `dark:bg-x` or `py-1.5`.
        // The exception: SCSS-family names parsed AS CSS
        // (oxc-css-parser: Unknown, Prettier: parseValue/parseSelector) keep the structural printers below.
        let unknown_string_params = matches!(prelude, AtRulePrelude::Unknown(_))
            && !is_value_parsed_at_rule(at_rule.name.raw);
        // NOTE: Prettier also keeps `@warn` / `@error` params as a raw string (`media-unknown`),
        // but `oxc-css-parser` parses their prelude structurally (`SassExpr`),
        // so we route them through the normal structured printer
        // for internal consistency over Prettier byte-equality.
        if unknown_string_params {
            let prelude_start = to_span(prelude.span()).start;
            write_verbatim_at_rule_tail(
                name_span.end,
                prelude_start,
                at_rule.block.as_ref(),
                region_end,
                f,
            );
            return;
        }
    }

    // `//` comments have their own layout rules (e.g. less `selector(...)`)
    // handled by the structural printers below.
    let has_inline_params_comment = f
        .context()
        .comments()
        .iter_before(region_end)
        .any(|c| c.inline && c.span.start >= name_span.end);
    if has_params_comments && !has_inline_params_comment {
        let lowercased = at_rule.name.raw.cow_to_ascii_lowercase();
        let raw = source.slice_range(name_span.end, region_end).trim();
        match &*lowercased {
            "media" | "custom-media" if !raw.contains("#{") => {
                let _ = f.context().comments().take_before(region_end);
                write!(f, space());
                write_commented_media_params(raw, f);
                write_block_or_semicolon(at_rule, f);
                return;
            }
            "import" if matches!(at_rule.prelude, Some(AtRulePrelude::Import(_))) => {
                let _ = f.context().comments().take_before(region_end);
                write!(f, space());
                write_commented_value_params(raw, 8, true, f);
                write!(f, ";");
                return;
            }
            "supports" if matches!(at_rule.prelude, Some(AtRulePrelude::Supports(_))) => {
                let _ = f.context().comments().take_before(region_end);
                write!(f, space());
                write_commented_value_params(raw, 10, false, f);
                write_block_or_semicolon(at_rule, f);
                return;
            }
            // String params: whitespace-normalized verbatim, one line
            "keyframes"
            | "page"
            | "font-feature-values"
            | "counter-style"
            | "viewport"
            | "namespace"
            | "styleset"
            | "property"
            | "layer" => {
                let _ = f.context().comments().take_before(region_end);
                if !raw.is_empty() {
                    write!(f, space());
                    let normalized = normalize_whitespace(raw);
                    write!(f, text(f.allocator().alloc_str(&normalized)));
                }
                write_block_or_semicolon(at_rule, f);
                return;
            }
            _ => {}
        }
    }

    // SCSS control directives wrap the prelude and the gap before `{` in one group:
    // when the prelude breaks, `{` moves to its own line.
    let is_control_directive = at_rule.block.is_some()
        && matches!(at_rule.prelude, Some(AtRulePrelude::SassEach(_) | AtRulePrelude::SassFor(_)))
        || (matches!(at_rule.prelude, Some(AtRulePrelude::SassExpr(_)))
            && matches!(at_rule.name.raw, "if" | "else" | "while"));

    if let Some(prelude) = &at_rule.prelude {
        // postcss folds a no-gap, non-paren prelude into the at-rule NAME
        // (`@page:first` stays tight); reproduce by checking the source gap.
        let name_end = to_span(at_rule.name.span()).end;
        let prelude_span = to_span(prelude.span());
        let fused =
            name_end == prelude_span.start && !source.text_for(&prelude_span).starts_with('(');
        if fused {
            write!(f, text(source.text_for(&prelude_span)));
        } else if is_control_directive {
            // A fully parenthesized condition keeps `{` on the `)` line
            // (Prettier's `hasParensAroundNode`).
            let has_parens = matches!(
                prelude,
                AtRulePrelude::SassExpr(value)
                    if matches!(&**value, ComponentValue::SassParenthesizedExpression(_))
            );
            if has_parens {
                write!(f, space());
                write_at_rule_prelude(prelude, f);
                write!(f, space());
            } else {
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, space());
                    write_at_rule_prelude(prelude, f);
                    write!(f, soft_line_break_or_space());
                });
                write!(f, group(&body));
            }
        } else {
            write!(f, space());
            write_at_rule_prelude(prelude, f);
        }
    }

    if let Some(block) = &at_rule.block {
        // An inline comment between the prelude and `{` keeps its own line
        // (Prettier's `lastLineHasInlineComment` → line instead of space).
        let block_start = to_span(&block.span).start;
        let mut wrote_comment = false;
        if f.context().comments().peek().is_some_and(|c| c.inline && c.span.end <= block_start) {
            for &comment in f.context().comments().take_before(block_start) {
                write!(f, hard_line_break());
                comments::write_single_comment(comment, f);
            }
            write!(f, hard_line_break());
            wrote_comment = true;
        }
        if !is_control_directive && !wrote_comment {
            write!(f, space());
        }
        statement::write_block(block, f);
    } else {
        write!(f, ";");
    }
}

/// Emits `@apply` params with the sortable class list as a single `FormatElement::TailwindClass`.
/// Returns `false` (nothing written) when there is nothing sortable,
/// leaving the caller on the normal path.
///
/// Ports prettier-plugin-tailwindcss's `transformCss` pre-processing;
/// the sorter itself (ordering, dedup, whitespace collapse) is host-supplied:
/// - a `!important` tail (incl. SCSS `#{!important}` interpolation forms)
///   is kept out of the sortable part and re-attached verbatim
/// - a Less `~"..."` / `~'...'` escaped-string wrapper is kept and only the
///   inside is sorted (and a `!important` inside it is NOT special)
fn write_apply_prelude<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) -> bool {
    let raw = raw.trim();

    let (wrapper, class_list, important_tail) =
        if let Some(inner) = raw.strip_prefix("~\"").and_then(|r| r.strip_suffix('"')) {
            (Some("\""), inner, None)
        } else if let Some(inner) = raw.strip_prefix("~'").and_then(|r| r.strip_suffix('\'')) {
            (Some("'"), inner, None)
        } else {
            match split_important_tail(raw) {
                Some((classes, tail)) => (None, classes, Some(tail)),
                None => (None, raw, None),
            }
        };

    let class_list = class_list.trim();
    if class_list.is_empty() {
        return false;
    }

    write!(f, space());
    if let Some(quote) = wrapper {
        write!(f, "~");
        write!(f, text(quote));
    }
    let index = f.context_mut().add_tailwind_class(class_list.to_string());
    f.write_element(FormatElement::TailwindClass(index));
    if let Some(quote) = wrapper {
        write!(f, text(quote));
    }
    if let Some(tail) = important_tail {
        write!(f, [space(), text(tail)]);
    }
    true
}

/// Splits off the `!important` tail the Tailwind plugin ignores when sorting:
/// `/\s+(?:!important|#{(['"]*)!important\1})\s*$/`
/// (whitespace before the tail is required; matching is case-sensitive like the plugin's).
/// Returns `(class part, tail text)` when present.
fn split_important_tail(raw: &str) -> Option<(&str, &str)> {
    let trimmed = raw.trim_end();
    ["!important", "#{!important}", "#{'!important'}", "#{\"!important\"}"].iter().find_map(
        |tail| {
            let classes = trimmed.strip_suffix(tail)?;
            classes.ends_with(char::is_whitespace).then_some((classes, *tail))
        },
    )
}

/// Names whose params Prettier's parser DOES hand to a sub-parser
/// (parseValue / parseSelector / parseMediaQuery — parser-postcss.js),
/// so a `oxc-css-parser` Unknown prelude for them must keep the structural printers.
///
/// Case-sensitivity mirrors Prettier:
/// bare `name` comparisons for the SCSS family, lowercased for module/media rules.
fn is_value_parsed_at_rule(name: &str) -> bool {
    matches!(
        name,
        "extend"
            | "nest"
            | "at-root"
            | "namespace"
            | "supports"
            | "if"
            | "else"
            | "for"
            | "each"
            | "while"
            | "debug"
            | "mixin"
            | "include"
            | "function"
            | "return"
            | "define-mixin"
            | "add-mixin"
            | "custom-selector"
    ) || matches!(
        &*name.cow_to_ascii_lowercase(),
        "import" | "use" | "forward" | "media" | "custom-media"
    )
}

/// Verbatim params + block/`;` for at-rules whose params Prettier keeps as a
/// plain string (see the Unknown-prelude early return in `write_at_rule`).
/// The slice runs from the at-rule NAME to the block/`;` so comments stay
/// embedded exactly like postcss's `afterName + params` string.
pub(super) fn write_verbatim_at_rule_tail<'a>(
    name_end: u32,
    prelude_start: u32,
    block: Option<&SimpleBlock<'a>>,
    region_end: u32,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let raw = source.slice_range(name_end, region_end).trim();
    let _ = f.context().comments().take_before(region_end);
    if !raw.is_empty() {
        // postcss keeps a no-gap prelude fused to the NAME (`@a:b` stays tight),
        // but a leading `(` still gets the printer's space.
        if name_end != prelude_start || raw.starts_with('(') {
            write!(f, space());
        }
        write!(f, text(raw));
    }
    if let Some(block) = block {
        // Prettier's `lastLineHasInlineComment`:
        // a trailing `//` line pushes `{` to the next line (it would be swallowed by the comment).
        if raw.split('\n').next_back().is_some_and(|line| line.contains("//")) {
            write!(f, hard_line_break());
        } else {
            write!(f, space());
        }
        statement::write_block(block, f);
    } else {
        write!(f, ";");
    }
}

/// ` {...}` when the at-rule has a block, `;` otherwise.
fn write_block_or_semicolon<'a>(at_rule: &AtRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    if let Some(block) = &at_rule.block {
        write!(f, space());
        statement::write_block(block, f);
    } else {
        write!(f, ";");
    }
}

/// `@media` params containing comments, rebuilt the way
/// `postcss-media-query-parser` + Prettier's `media-*` cases lay them out:
/// queries split on top-level commas (`,` + line in a group),
/// tokens joined by single spaces, `( feature : value )` re-spaced
/// when the parser would have recognized it (spaces around the `:`), kept verbatim otherwise.
fn write_commented_media_params<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let queries = split_top_level(raw, b',');
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        for (i, query) in queries.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                write!(f, soft_line_break_or_space());
            }
            let tokens = tokenize(query.trim(), TokenizeMode::AbsorbComments);
            for (j, token) in tokens.iter().enumerate() {
                if j > 0 {
                    write!(f, space());
                }
                write_media_token(token, f);
            }
        }
    });
    write!(f, group(&indent(&body)));
}

/// `@import` / `@supports` params containing comments,
/// laid out the way Prettier's value parser + fill does.
/// (Required because `oxc-css-parser` drops params-embedded comments, they only survive via this source-rebuild.)
/// Separator breaks are simulated with static widths
/// (Prettier's fill fit-checks the next item; our core fill measures only up to a hardline).
/// `allow_commas`: comma chunks get an extra indent level for their internal wraps.
fn write_commented_value_params<'a>(
    raw: &'a str,
    start_col: u32,
    allow_commas: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    // One value-parser token (the comma glues to its chunk's tail)
    struct Tok<'a> {
        text: &'a str,
        comma: bool,
        chunk: usize,
        hard: bool,
    }
    let width = u32::from(f.options().line_width.value());
    let chunks: Vec<&str> = if allow_commas { split_top_level(raw, b',') } else { vec![raw] };
    let multi = chunks.len() > 1;

    let mut tokens: Vec<Tok<'a>> = vec![];
    for (ci, chunk) in chunks.iter().enumerate() {
        let toks = tokenize(chunk.trim(), TokenizeMode::SplitComments);
        let last = toks.len().saturating_sub(1);
        for (ti, t) in toks.into_iter().enumerate() {
            tokens.push(Tok {
                text: t,
                comma: ci + 1 < chunks.len() && ti == last,
                chunk: ci,
                hard: t.contains('(') && t.contains("/*"),
            });
        }
    }

    let tok_width =
        |t: &Tok<'_>| u32::try_from(t.text.len()).unwrap_or(u32::MAX) + u32::from(t.comma);
    // Per-chunk flat width / hardness (chunk separators compare WHOLE chunks,
    // Prettier's fill over the comma-joined chunk docs).
    let mut chunk_w = vec![0u32; chunks.len()];
    let mut chunk_hard = vec![false; chunks.len()];
    for t in &tokens {
        if chunk_w[t.chunk] > 0 {
            chunk_w[t.chunk] += 1;
        }
        chunk_w[t.chunk] += u32::try_from(t.text.len()).unwrap_or(u32::MAX) + u32::from(t.comma);
        chunk_hard[t.chunk] = chunk_hard[t.chunk] || t.hard;
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let mut x = start_col;
        let mut chunk_start_x = start_col;
        for i in 0..tokens.len() {
            let t = &tokens[i];
            if i > 0 {
                let prev = &tokens[i - 1];
                if prev.chunk == t.chunk {
                    let fits = !prev.hard && !t.hard && x + 1 + tok_width(t) <= width;
                    if fits {
                        write!(f, space());
                        x += 1;
                    } else {
                        write!(f, hard_line_break());
                        if multi {
                            write!(f, text("  "));
                            x = 4;
                        } else {
                            x = 2;
                        }
                    }
                } else {
                    let fits = !chunk_hard[prev.chunk]
                        && !chunk_hard[t.chunk]
                        && chunk_start_x + chunk_w[prev.chunk] + 1 + chunk_w[t.chunk] <= width;
                    if fits {
                        write!(f, space());
                        x += 1;
                    } else {
                        write!(f, hard_line_break());
                        x = 2;
                    }
                    chunk_start_x = x;
                }
            }
            if t.hard {
                write_structured_paren(t.text, x, f);
                // `)` lands back at column x
                x += 1;
            } else {
                let printed = requote_token(t.text, f);
                write!(f, text(printed));
                x += u32::try_from(printed.len()).unwrap_or(0);
            }
            if t.comma {
                write!(f, ",");
                x += 1;
            }
        }
    });
    write!(f, indent(&body));
}

/// `start` points at the `/` of a `/*`; returns the index of the closing `/`
/// (clamped to the last byte when unterminated).
fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
        i += 1;
    }
    (i + 1).min(bytes.len() - 1)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenizeMode {
    /// `@import` / `@supports` values:
    /// a top-level `/* ... */` is its own token (postcss-values splits comments out).
    SplitComments,
    /// `@media` queries:
    /// a top-level `/* ... */` absorbs into the surrounding /// token
    /// (`postcss-media-query-parser` keeps comments inline).
    AbsorbComments,
}

/// Whitespace-separated tokens;
/// comments and balanced paren regions glue to adjacent touching text.
/// A `)` closing a single-line paren region ends the token (postcss-media-query-parser splits there);
/// a multi-line region is a `media-unknown` and keeps its touching suffix.
fn tokenize(raw: &str, mode: TokenizeMode) -> Vec<&str> {
    let bytes = raw.as_bytes();
    let mut tokens = vec![];
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                if depth == 0 && mode == TokenizeMode::SplitComments {
                    if let Some(s) = start.take() {
                        tokens.push(&raw[s..i]);
                    }
                    let end = block_comment_end(bytes, i);
                    tokens.push(&raw[i..=end]);
                    i = end;
                } else {
                    // depth > 0 OR AbsorbComments at depth 0:
                    // comment becomes part of the current token (or starts one if between tokens).
                    if start.is_none() {
                        start = Some(i);
                    }
                    i = block_comment_end(bytes, i);
                }
            }
            b'(' => {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0
                    && let Some(s) = start.take()
                {
                    tokens.push(&raw[s..=i]);
                }
            }
            b' ' | b'\t' | b'\n' | b'\r' if depth == 0 => {
                if let Some(s) = start.take() {
                    tokens.push(&raw[s..i]);
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
        i += 1;
    }
    if let Some(s) = start {
        tokens.push(&raw[s..]);
    }
    tokens
}

/// Re-quotes `'...'` strings in a token to the preferred quote (Prettier's `adjustStrings`).
fn requote_token<'a>(token: &'a str, f: &CssFormatter<'_, 'a>) -> &'a str {
    match f.options().single_quote.requote(token) {
        Cow::Borrowed(s) => s,
        Cow::Owned(s) => f.allocator().alloc_str(&s),
    }
}

/// A comment-bearing paren group always breaks:
/// `(` + indented fill of its words (`:` glued left, spaced right) + `)`.
fn write_structured_paren<'a>(token: &'a str, open_col: u32, f: &mut CssFormatter<'_, 'a>) {
    let width = u32::from(f.options().line_width.value());
    let Some(open) = find_outside_comments(token, b'(') else {
        write!(f, text(token));
        return;
    };
    let Some(close) = token.rfind(')') else {
        write!(f, text(token));
        return;
    };
    write!(f, text(&token[..=open]));
    let inner = &token[open + 1..close];
    // Words, requoted up front; a lone `:` glues to the previous word
    let sq = f.options().single_quote;
    let requote = |w: &str| -> String { sq.requote(w).into_owned() };
    let mut words: Vec<String> = vec![];
    for w in tokenize(inner.trim(), TokenizeMode::SplitComments) {
        if (w == ":" || w.starts_with(':'))
            && let Some(last) = words.last_mut()
        {
            last.push(':');
            if w.len() > 1 {
                words.push(requote(&w[1..]));
            }
            continue;
        }
        words.push(requote(w));
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(f, hard_line_break());
        let mut x = open_col + 2;
        for (i, w) in words.iter().enumerate() {
            let w_len = u32::try_from(w.len()).unwrap_or(0);
            if i > 0 {
                if x + 1 + w_len <= width {
                    write!(f, space());
                    x += 1;
                } else {
                    write!(f, hard_line_break());
                    write!(f, text("  "));
                    x = open_col + 4;
                }
            }
            write!(f, text(f.allocator().alloc_str(w)));
            x += w_len;
        }
    });
    write!(f, indent(&body));
    write!(f, hard_line_break());
    write!(f, ")");
    let suffix = &token[close + 1..];
    if !suffix.is_empty() {
        write!(f, text(suffix));
    }
}

/// Splits on `sep` at paren depth 0, outside comments.
fn split_top_level(raw: &str, sep: u8) -> Vec<&str> {
    let bytes = raw.as_bytes();
    let mut parts = vec![];
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = block_comment_end(bytes, i);
            }
            b'(' => depth += 1,
            b')' => depth -= 1,
            b if b == sep && depth == 0 => {
                parts.push(&raw[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&raw[start..]);
    parts
}

/// One media token: a paren region (with glued prefix/suffix) gets the
/// `( feature: value )` re-spacing when the `:` has space around it.
fn write_media_token<'a>(token: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let Some(open) = find_outside_comments(token, b'(') else {
        write!(f, text(token));
        return;
    };
    let Some(close) = token.rfind(')') else {
        write!(f, text(token));
        return;
    };
    let inner = &token[open + 1..close];
    let colon = find_outside_comments(inner, b':');
    let respace = colon.is_some_and(|c| {
        let before_ws = inner[..c].ends_with([' ', '\t', '\n', '\r']);
        let after_ws = inner[c + 1..].starts_with([' ', '\t', '\n', '\r']);
        before_ws || after_ws
    });
    if !respace {
        // Unparsable for postcss-media-query-parser: verbatim
        write!(f, text(token));
        return;
    }
    let colon = colon.unwrap();
    write!(f, text(&token[..=open]));
    let feature = inner[..colon].trim();
    let value = inner[colon + 1..].trim();
    let normalized = format!("{}: {}", normalize_whitespace(feature), normalize_whitespace(value));
    write!(f, text(f.allocator().alloc_str(&normalized)));
    write!(f, text(&token[close..]));
}

/// First position of `needle` outside `/* */` comments.
fn find_outside_comments(raw: &str, needle: u8) -> Option<usize> {
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i = block_comment_end(bytes, i) + 1;
            continue;
        }
        if bytes[i] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn write_at_rule_prelude<'a>(prelude: &AtRulePrelude<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match prelude {
        AtRulePrelude::Media(media) => write_media_query_list(media, f),
        AtRulePrelude::CustomMedia(custom) => {
            let name_span = to_span(custom.name.span());
            write!(f, text(source.text_for(&name_span)));
            write!(f, space());
            match &custom.value {
                CustomMediaValue::MediaQueryList(list) => {
                    write_media_query_list(list, f);
                }
                CustomMediaValue::True(ident) | CustomMediaValue::False(ident) => {
                    let span = to_span(ident.span());
                    write!(f, text(source.text_for(&span)));
                }
            }
        }
        AtRulePrelude::Keyframes(name) => write_keyframes_name(name, f),
        AtRulePrelude::CustomSelector(custom) => {
            let custom_span = to_span(custom.custom_selector.span());
            let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                // oxc-css-parser's CustomSelector span excludes the leading `:`
                write!(f, ":");
                write!(f, text(source.text_for(&custom_span)));
                write!(f, soft_line_break_or_space());
                // Selectors share THIS group: when the prelude breaks,
                // each selector gets its own line.
                for (i, complex) in custom.selector.selectors.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",");
                        write!(f, soft_line_break_or_space());
                    }
                    selector::write_complex_selector(complex, f);
                }
            });
            write!(f, group(&indent(&body)));
        }
        AtRulePrelude::Charset(str) => {
            let span = to_span(str.span());
            write!(f, text(source.text_for(&span)));
        }
        AtRulePrelude::Import(import) => write_import_prelude(import, f),
        AtRulePrelude::LessImport(import) => write_less_import_prelude(import, f),
        AtRulePrelude::Namespace(namespace) => {
            if let Some(prefix) = &namespace.prefix {
                let span = to_span(prefix.span());
                write!(f, text(source.text_for(&span)));
                write!(f, space());
            }
            match &namespace.uri {
                NamespacePreludeUri::Str(InterpolableStr::Literal(str)) => {
                    value::write_str(str, f);
                }
                NamespacePreludeUri::Url(url) => value::write_url(url, f),
                // Interpolated strings (`'#{$url}'` etc.):
                // outer quote is still requoted per `singleQuote`, content is verbatim
                // (postcss-values' `value-unknown` path, same as `ComponentValue::InterpolableStr` in value position).
                uri @ NamespacePreludeUri::Str(_) => {
                    let span = to_span(uri.span());
                    value::write_requoted_verbatim(source.text_for(&span), f);
                }
            }
        }
        // Prettier keeps `@page` params verbatim (e.g. `@page:first` stays)
        AtRulePrelude::Page(page) => {
            let span = to_span(page.span());
            write!(f, text(source.text_for(&span)));
        }
        AtRulePrelude::Supports(condition) => write_supports_condition(condition, f),
        AtRulePrelude::Layer(layers) => {
            for (i, layer) in layers.names.iter().enumerate() {
                if i > 0 {
                    write!(f, [",", space()]);
                }
                let span = to_span(layer.span());
                write!(f, text(source.text_for(&span)));
            }
        }
        AtRulePrelude::Property(ident)
        | AtRulePrelude::CounterStyle(ident)
        | AtRulePrelude::FontPaletteValues(ident)
        | AtRulePrelude::PositionTry(ident)
        | AtRulePrelude::ScrollTimeline(ident) => {
            let span = to_span(ident.span());
            write!(f, text(source.text_for(&span)));
        }
        AtRulePrelude::Nest(list) => {
            selector::write_selector_list(list, selector::SelectorListStyle::Line, f);
        }
        AtRulePrelude::SassAtRoot(at_root) => match &at_root.kind {
            SassAtRootKind::Selector(list) => {
                selector::write_selector_list(list, selector::SelectorListStyle::Line, f);
            }
            SassAtRootKind::Query(query) => {
                let span = to_span(query.span());
                write!(f, text(source.text_for(&span)));
            }
        },
        AtRulePrelude::SassExpr(value) => {
            value::write_component_value(value, ValueContext::default(), f);
        }
        AtRulePrelude::SassEach(each) => scss::write_sass_each(each, f),
        AtRulePrelude::SassFor(sass_for) => scss::write_sass_for(sass_for, f),
        AtRulePrelude::SassMixin(mixin) => scss::write_sass_mixin(mixin, f),
        AtRulePrelude::SassFunction(function) => scss::write_sass_function(function, f),
        AtRulePrelude::SassInclude(include) => scss::write_sass_include(include, f),
        AtRulePrelude::SassUse(sass_use) => scss::write_sass_use(sass_use, f),
        AtRulePrelude::SassForward(forward) => scss::write_sass_forward(forward, f),
        AtRulePrelude::SassImport(import) => {
            let paths: Vec<(Span, &str)> =
                import.paths.iter().map(|path| (path.span, path.raw)).collect();
            write_import_path_list(&paths, to_span(import.span()).end, f);
        }
        // Only reached for SCSS-family names parsed AS CSS (see `is_value_parsed_at_rule`);
        // other `Unknown` preludes print verbatim via the `write_at_rule` early return.
        AtRulePrelude::Unknown(unknown) => match &**unknown {
            UnknownAtRulePrelude::ComponentValue(value) => {
                if matches!(value, ComponentValue::InterpolableStr(_)) {
                    let span = to_span(value.span());
                    write!(f, text(source.text_for(&span)));
                } else {
                    value::write_component_value(value, ValueContext::default(), f);
                }
            }
            UnknownAtRulePrelude::TokenSeq(seq) => {
                // Prints a raw token sequence,
                // collapsing whitespace runs to a single breakable space and keeping tight tokens tight
                // (mirrors how Prettier's `parseValue` + comma-group printing treats unknown at-rule params).
                write_token_value(&seq.tokens, true, f);
            }
        },
        // Sass/Less and not-yet-ported preludes: verbatim
        _ => {
            let span = to_span(prelude.span());
            write!(f, text(source.text_for(&span)));
        }
    }
}

/// Prints an `@import` path list (`@import "a", "b", ...`) like Prettier's value-parsed params (module rule):
/// paths fill at the line width with a continuation indent, and comments force one path per line.
/// Used by the SCSS `SassImportPrelude` AND by `ImportPrelude`
/// when its non-standard tail is exactly a comma-separated string list (see `import_as_path_list`).
/// Paths arrive as `(span, raw)` pairs so both callers can feed it without fabricating AST nodes.
fn write_import_path_list<'a>(
    paths: &[(Span, &'a str)],
    last_end: u32,
    f: &mut CssFormatter<'_, 'a>,
) {
    // Comments force the path list to break, one path per line
    let has_comments = f.context().comments().iter_before(last_end).next().is_some();
    if has_comments && paths.len() > 1 {
        // Comments fuse with the following path into ONE fill chunk (Prettier's `commaGroup`).
        // Prettier's fill treats a chunk with a hardline as never-fitting,
        // our core fill measures up to the hardline and calls it fit.
        // So the separator breaks are simulated here with static widths.
        let all: Vec<comments::CssComment> = f.context().comments().iter_before(last_end).collect();
        let n = paths.len();
        let mut leads: Vec<Vec<comments::CssComment>> = Vec::with_capacity(n);
        for (i, path) in paths.iter().enumerate() {
            let path_start = to_span(&path.0).start;
            leads.push(
                all.iter()
                    .filter(|c| c.span.end <= path_start)
                    .filter(|c| i == 0 || c.span.start >= to_span(&paths[i - 1].0).end)
                    .copied()
                    .collect(),
            );
        }
        // Prettier fill: separator stays flat only when
        // [chunk, ", ", next chunk] fits and neither chunk has a comment (hardline).
        let width = u32::from(f.options().line_width.value());
        let indent_w = u32::from(f.options().indent_width.value());
        let chunk_w: Vec<u32> = paths
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let span = to_span(&p.0);
                span.end - span.start + u32::from(i + 1 < n)
            })
            .collect();
        let mut breaks_before = vec![false; n];
        let mut x = 8; // after `@import `
        for i in 0..n {
            if i + 1 == n {
                break;
            }
            let hard = leads[i].iter().any(|c| c.inline);
            let next_hard = leads[i + 1].iter().any(|c| c.inline);
            let fits2 = !hard && !next_hard && x + chunk_w[i] + 1 + chunk_w[i + 1] <= width;
            if fits2 {
                x += chunk_w[i] + 1;
            } else {
                breaks_before[i + 1] = true;
                x = indent_w;
            }
        }
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            for (i, path) in paths.iter().enumerate() {
                if i > 0 {
                    if breaks_before[i] {
                        write!(f, hard_line_break());
                    } else {
                        write!(f, space());
                    }
                }
                for &comment in &leads[i] {
                    f.context().comments().take_before(comment.span.end);
                    comments::write_single_comment(comment, f);
                    if comment.inline {
                        write!(f, hard_line_break());
                    } else {
                        write!(f, space());
                    }
                }
                value::write_str_raw(path.1, f);
                if i + 1 < n {
                    write!(f, ",");
                }
            }
        });
        write!(f, indent(&body));
    } else if has_comments {
        let path = &paths[0];
        let path_start = to_span(&path.0).start;
        let lead: Vec<comments::CssComment> =
            f.context().comments().take_before(path_start).to_vec();
        for &comment in &lead {
            comments::write_single_comment(comment, f);
            if comment.inline {
                write!(f, hard_line_break());
            } else {
                write!(f, space());
            }
        }
        value::write_str_raw(path.1, f);
    } else {
        // Comma-separated path list:
        // Prettier value-parses `@import` params (module rule) and fills them,
        // long lists wrap at the line width with a continuation indent.
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let mut filler = f.fill();
            let n = paths.len();
            for (i, path) in paths.iter().enumerate() {
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    value::write_str_raw(path.1, f);
                    if i + 1 < n {
                        write!(f, ",");
                    }
                });
                filler.entry(&soft_line_break_or_space(), &content);
            }
            filler.finish();
        });
        write!(f, group(&indent(&body)));
    }
}

/// Comma-separated token value:
/// groups joined by `, ` (breakable), blank lines preserved after multi-token groups.
fn write_token_value<'a>(
    tokens: &[TokenWithSpan<'a>],
    top_level: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    // Split at top-level commas
    let mut groups: Vec<&[TokenWithSpan<'a>]> = vec![];
    let mut depth = 0i32;
    let mut start = 0;
    for (i, tok) in tokens.iter().enumerate() {
        depth += value::token_depth_delta(&tok.token);
        if depth == 0 && matches!(&tok.token, Token::Comma(_)) {
            groups.push(&tokens[start..i]);
            start = i + 1;
        }
    }
    groups.push(&tokens[start..]);
    if groups.len() > 1 && groups.last().is_some_and(|g| g.is_empty()) {
        groups.pop();
    }

    if groups.len() == 1 {
        let only = groups[0];
        // `name( ... )` covering the whole group:
        // the parens govern breaking/indent; anything else gets the continuation indent.
        let whole_call = only.len() > 2
            && matches!(&only[only.len() - 1].token, Token::RParen(_))
            && (matches!(&only[0].token, Token::LParen(_))
                || (matches!(&only[0].token, Token::Ident(_))
                    && matches!(&only[1].token, Token::LParen(_))));
        if top_level && !whole_call {
            let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write_token_comma_group(only, f);
            });
            write!(f, group(&indent(&body)));
        } else if whole_call {
            write_token_comma_group(only, f);
        } else {
            write_token_comma_group_grouped(only, f);
        }
        return;
    }
    let groups_ref = &groups;
    if top_level {
        // Top level: fill (as many groups per line as fit)
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let mut filler = f.fill();
            for (i, group_tokens) in groups_ref.iter().enumerate() {
                let is_last = i + 1 == groups_ref.len();
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write_token_comma_group_grouped(group_tokens, f);
                    if !is_last {
                        write!(f, ",");
                    }
                });
                filler.entry(&soft_line_break_or_space(), &content);
            }
            filler.finish();
        });
        write!(f, group(&indent(&body)));
    } else {
        // Inside parens: `join(line, ...)` when the paren group breaks,
        // every group goes on its own line;
        // blank lines are preserved after key-value-ish groups.
        for (i, group_tokens) in groups_ref.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                let sep_blank =
                    groups_ref[i - 1].iter().any(|t| matches!(&t.token, Token::Colon(_))) && {
                        let prev_end = groups_ref[i - 1].last().map_or(0, |t| to_span(&t.span).end);
                        let next_start =
                            group_tokens.first().map_or(prev_end, |t| to_span(&t.span).start);
                        comments::classify_gap(source.bytes_range(prev_end, next_start))
                            == comments::Gap::Blank
                    };
                if sep_blank {
                    write!(f, empty_line());
                } else {
                    write!(f, soft_line_break_or_space());
                }
            }
            write_token_comma_group_grouped(group_tokens, f);
        }
    }
}

/// Space-separated tokens within one comma group;
/// balanced paren regions are printed as breakable groups.
fn write_token_comma_group<'a>(tokens: &[TokenWithSpan<'a>], f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();

    let hug_lparen = tokens.len() > 1
        && matches!(&tokens[1].token, Token::LParen(_))
        && matches!(&tokens[0].token, Token::Ident(_))
        && !source.text_for(&to_span(&tokens[0].span)).eq_ignore_ascii_case("if");

    let mut filler = f.fill();
    let mut i = 0;
    while i < tokens.len() {
        // A run: tokens glued by gap/punctuation rules;
        // a paren region opener ends the run scan (the region is appended to the same run).
        let mut run_end = i + 1;
        // A run starting at an opener swallows its balanced region
        if matches!(&tokens[i].token, Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_)) {
            let mut depth = 0i32;
            let mut j = i;
            while j < tokens.len() {
                depth += value::token_depth_delta(&tokens[j].token);
                j += 1;
                if depth == 0 {
                    break;
                }
            }
            run_end = j.max(i + 1);
        }
        while run_end < tokens.len() {
            let prev = &tokens[run_end - 1];
            let curr = &tokens[run_end];
            if matches!(&curr.token, Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_)) {
                // Opener glues to the run when fused in source,
                // after a colon (`$arg: (...)`), or as a call (`name(...)`).
                let glued = to_span(&prev.span).end == to_span(&curr.span).start
                    || matches!(&prev.token, Token::Colon(_))
                    || (run_end == 1 && hug_lparen);
                if !glued {
                    break;
                }
                // Append the whole balanced region (and continue the run)
                let mut depth = 0i32;
                let mut j = run_end;
                while j < tokens.len() {
                    depth += value::token_depth_delta(&tokens[j].token);
                    j += 1;
                    if depth == 0 {
                        break;
                    }
                }
                run_end = j;
                continue;
            }
            let tight = to_span(&prev.span).end == to_span(&curr.span).start
                || matches!(
                    &curr.token,
                    Token::Comma(_)
                        | Token::Colon(_)
                        | Token::Semicolon(_)
                        | Token::DotDotDot(_)
                        | Token::Dot(_)
                )
                // Math operators glue to their LEFT operand (break after);
                // comparisons glue to their RIGHT operand (break before).
                || matches!(
                    &curr.token,
                    Token::Plus(_)
                        | Token::Minus(_)
                        | Token::Asterisk(_)
                        | Token::Percent(_)
                        | Token::Solidus(_)
                )
                || matches!(&curr.token, Token::Equal(_))
                || matches!(&prev.token, Token::Equal(_));
            if tight {
                run_end += 1;
            } else {
                break;
            }
        }
        let run = &tokens[i..run_end];
        let run_start = i;
        let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write_token_run(run, run_start == 0 && hug_lparen, f);
        });
        filler.entry(&soft_line_break_or_space(), &content);
        i = run_end;
    }
    filler.finish();
}

/// Wrapper: a comma group is its own breakable group with indent.
/// Except when it contains paren regions,
/// which provide their own indentation (avoids double-indenting `name(...)` contents).
fn write_token_comma_group_grouped<'a>(tokens: &[TokenWithSpan<'a>], f: &mut CssFormatter<'_, 'a>) {
    // A group that IS one call/paren region delegates breaking to the parens
    let whole_region = !tokens.is_empty()
        && matches!(&tokens[tokens.len() - 1].token, Token::RParen(_))
        && (matches!(&tokens[0].token, Token::LParen(_))
            || (tokens.len() > 1
                && matches!(&tokens[0].token, Token::Ident(_))
                && matches!(&tokens[1].token, Token::LParen(_))));
    // A `$key: (region)` pair also delegates to the parens
    let kv_region = !whole_region
        && matches!(tokens.last().map(|t| &t.token), Some(Token::RParen(_)))
        && tokens
            .iter()
            .take_while(|t| {
                !matches!(&t.token, Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_))
            })
            .any(|t| matches!(&t.token, Token::Colon(_)));
    if whole_region || kv_region {
        write_token_comma_group(tokens, f);
        return;
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write_token_comma_group(tokens, f);
    });
    write!(f, group(&indent(&body)));
}

/// One run: spacing normalized; paren regions recurse as breakable groups.
fn write_token_run<'a>(run: &[TokenWithSpan<'a>], hug_lparen: bool, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let mut j = 0;
    while j < run.len() {
        let tok = &run[j];
        // A paren region: recurse
        if matches!(&tok.token, Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_)) {
            let mut depth = 0i32;
            let mut k = j;
            while k < run.len() {
                depth += value::token_depth_delta(&run[k].token);
                k += 1;
                if depth == 0 {
                    break;
                }
            }
            // Unbalanced region: print the opener verbatim and move on
            if depth != 0 || k < j + 2 {
                if j > 0 {
                    write_token_pair_space(run, j, hug_lparen, f);
                }
                let span = to_span(&tok.span);
                write!(f, text(source.text_for(&span)));
                j += 1;
                continue;
            }
            let inner = &run[j + 1..k - 1];
            let (open, close): (&str, &str) = match &tok.token {
                Token::LParen(_) => ("(", ")"),
                Token::LBracket(_) => ("[", "]"),
                _ => ("{", "}"),
            };
            if j > 0 {
                write_token_pair_space(run, j, hug_lparen, f);
            }
            if inner.is_empty() {
                write!(f, [text(open), text(close)]);
            } else {
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, soft_line_break());
                    write_token_value(inner, false, f);
                });
                write!(
                    f,
                    group(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        write!(f, text(open));
                        write!(f, indent(&body));
                        write!(f, soft_line_break());
                        write!(f, text(close));
                    }))
                );
            }
            j = k;
            continue;
        }
        if j > 0 {
            write_token_pair_space(run, j, hug_lparen, f);
        }
        let span = to_span(&tok.span);
        match &tok.token {
            Token::Str(_) => {
                write_raw_str(source.text_for(&span), f);
            }
            Token::Number(_) => {
                let raw = tok.number_raw(source.as_str()).unwrap();
                write!(f, text(arena_cow_str(&value::print_css_number(raw), f)));
            }
            Token::Dimension(_) => {
                let d = tok.dimension(source.as_str()).unwrap();
                write!(f, text(arena_cow_str(&value::print_css_number(d.value.raw), f)));
                write!(f, text(d.unit.raw));
            }
            Token::Percentage(_) => {
                let pct = tok.percentage(source.as_str()).unwrap();
                write!(f, text(arena_cow_str(&value::print_css_number(pct.value.raw), f)));
                write!(f, "%");
            }
            _ => write!(f, text(source.text_for(&span))),
        }
        j += 1;
    }
}

/// Normalized spacing between `run[j-1]` and `run[j]`.
#[expect(clippy::match_same_arms)]
fn write_token_pair_space<'a>(
    run: &[TokenWithSpan<'a>],
    j: usize,
    hug_lparen: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let prev = &run[j - 1];
    let tok = &run[j];
    let gap = to_span(&prev.span).end != to_span(&tok.span).start;
    let wordish = |t: &Token<'_>| {
        matches!(t, Token::Ident(_) | Token::DollarVar(_) | Token::AtKeyword(_) | Token::RParen(_))
    };
    let space = match (&prev.token, &tok.token) {
        (Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_), _)
        | (
            _,
            Token::RParen(_)
            | Token::RBracket(_)
            | Token::RBrace(_)
            | Token::Comma(_)
            | Token::Colon(_)
            | Token::Semicolon(_)
            | Token::Equal(_)
            | Token::DotDotDot(_)
            | Token::Dot(_),
        ) => false,
        (Token::Equal(_) | Token::Dot(_), _) => false,
        (_, Token::LParen(_)) if j == 1 && hug_lparen => false,
        (Token::Colon(_) | Token::Comma(_), _) => true,
        (Token::Ident(_), Token::Asterisk(_))
            if !gap && source.text_for(&to_span(&prev.span)).ends_with('-') =>
        {
            false
        }
        (p, Token::Asterisk(_) | Token::Plus(_)) if wordish(p) => true,
        (Token::Asterisk(_) | Token::Plus(_), n) if wordish(n) => true,
        (Token::Asterisk(_) | Token::Plus(_), Token::LParen(_)) => true,
        (
            Token::RParen(_),
            Token::GreaterThan(_)
            | Token::LessThan(_)
            | Token::GreaterThanEqual(_)
            | Token::LessThanEqual(_)
            | Token::EqualEqual(_)
            | Token::ExclamationEqual(_),
        ) => true,
        _ => gap,
    };
    if space {
        write!(f, " ");
    }
}

fn write_raw_str<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    if raw.len() < 2 {
        write!(f, text(raw));
        return;
    }
    let content = &raw[1..raw.len() - 1];
    let enclosing = f.options().preferred_quote(content);
    if raw.as_bytes()[0] == enclosing {
        write!(f, text(raw));
    } else {
        let out = format!("{ch}{content}{ch}", ch = enclosing as char);
        write!(f, text(f.allocator().alloc_str(&out)));
    }
}

fn write_keyframes_name<'a>(name: &KeyframesName<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    if let KeyframesName::Str(InterpolableStr::Literal(str)) = name {
        value::write_str(str, f);
    } else {
        let span = to_span(name.span());
        write!(f, text(source.text_for(&span)));
    }
}

fn write_import_prelude<'a>(import: &ImportPrelude<'a>, f: &mut CssFormatter<'_, 'a>) {
    // Css mode only:
    // a comma-separated string list (`@import 'a', 'b'`) lands in the non-standard `modifiers` tail
    // (its leading comma fails the structured media parse, and Css has no `SassImportPrelude` to fall back to,
    // `oxc-css-parser` routes Scss/Sass there directly).
    // Rebuild the typed path list and print it through the same comment-aware fill.
    if let Some(paths) = import_as_path_list(import, f) {
        write_import_path_list(&paths, to_span(import.span()).end, f);
        return;
    }
    // The whole prelude is one group: a long `@import url(...) media, list;`
    // breaks after the url and puts one query per line, all one indent in.
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write_import_prelude_inner(import, f);
    });
    write!(f, group(&indent(&body)));
}

/// `Some(paths)` (as `(span, raw)` pairs) when the prelude is a plain multi-path import:
/// a string href whose whole tail is `, <string> (, <string>)*` held as raw tokens.
fn import_as_path_list<'a>(
    import: &ImportPrelude<'a>,
    f: &CssFormatter<'_, 'a>,
) -> Option<Vec<(Span, &'a str)>> {
    let modifiers = import.modifiers.as_ref()?;
    let ImportPreludeHref::Str(InterpolableStr::Literal(href)) = &import.href else {
        return None;
    };
    // A path list always begins with a comma glued to the href
    if !matches!(
        modifiers.values.first(),
        Some(ComponentValue::TokenWithSpan(TokenWithSpan { token: Token::Comma(_), .. }))
    ) {
        return None;
    }
    let source = f.context().source_text();
    let mut paths = Vec::with_capacity(1 + modifiers.values.len() / 2);
    paths.push((href.span, href.raw));
    let mut expect_comma = true;
    for value in &modifiers.values {
        let ComponentValue::TokenWithSpan(tok) = value else {
            return None;
        };
        match &tok.token {
            Token::Comma(_) if expect_comma => expect_comma = false,
            Token::Str(_) if !expect_comma => {
                paths.push((tok.span, source.text_for(&to_span(&tok.span))));
                expect_comma = true;
            }
            _ => return None,
        }
    }
    // A trailing comma never arrives here (the parser rejects it)
    (paths.len() > 1 && expect_comma).then_some(paths)
}

fn write_import_prelude_inner<'a>(import: &ImportPrelude<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    // Non-standard import tail (`@import "a", "b"` / `@import "a" b c(d)`):
    // reference compilers accept an arbitrary post-URL mix of idents,
    // functions, parens, and comma-chained imports; oxc-css-parser keeps it as raw tokens
    // (`layer`/`supports`/`media` are all `None` then).
    if let Some(modifiers) = &import.modifiers {
        write_import_modifiers(import, &modifiers.values, f);
        return;
    }
    write_import_href(&import.href, f);
    if let Some(layer) = &import.layer {
        write!(f, space());
        let span = to_span(layer.span());
        write!(f, text(source.text_for(&span)));
    }
    if let Some(supports) = &import.supports {
        // `@import ... supports(<cond>)`.
        // Prettier value-parses `@import` params (a token stream);
        // we instead reprint through the `@supports` structured printers (`oxc-css-parser` parses it structurally).
        // Identical for real-world cases, the divergences are all edge cases absent from real CSS:
        // inherited from `write_supports_condition`
        // (uppercase props lowercase; a source-glued `not`/`and` gains a space),
        // plus one of our own (a width-overflowing condition with no trailing media breaks INSIDE the parens, not before `supports`).
        // Empty `supports()` was the prior data-loss stub.
        write!(f, [space(), "supports("]);
        match &supports.kind {
            // `supports(not (display: inline-grid))`, `supports(font-format(woff2))`
            ImportPreludeSupportsKind::SupportsCondition(condition) => {
                write_supports_condition(condition, f);
            }
            // `supports(display: flex)`: a bare declaration (no inner parens)
            ImportPreludeSupportsKind::Declaration(decl) => {
                statement::write_declaration(decl, f);
            }
        }
        write!(f, ")");
    }
    if let Some(media) = &import.media {
        write!(f, soft_line_break_or_space());
        // No own group/indent: the queries share the prelude-level group
        write_media_query_list_inner(media, f);
    }
}

/// A non-standard import tail printed like Prettier's value-parsed params:
/// a FILL over the top-level comma groups (as many per line as fit, commas glued left),
/// the href riding as the first entry so `@import "a", "b", ...` wraps exactly where Prettier wraps.
/// Within one group, token runs go through the token-value printer
/// (spaces normalize, strings re-quote, `c( d )` → `c(d)`).
fn write_import_modifiers<'a>(
    import: &ImportPrelude<'a>,
    values: &[ComponentValue<'a>],
    f: &mut CssFormatter<'_, 'a>,
) {
    // Split into subslices at top-level commas (mirrors `write_token_value`).
    // A leading comma (`@import "a", "b"`) leaves an empty first group,
    // whose comma glues straight onto the href.
    let mut groups: Vec<&[ComponentValue<'a>]> = vec![];
    let mut depth = 0i32;
    let mut start = 0;
    for (i, value) in values.iter().enumerate() {
        if let ComponentValue::TokenWithSpan(tok) = value {
            if depth == 0 && matches!(&tok.token, Token::Comma(_)) {
                groups.push(&values[start..i]);
                start = i + 1;
                continue;
            }
            depth += value::token_depth_delta(&tok.token);
        }
    }
    groups.push(&values[start..]);

    let last = groups.len() - 1;
    let first_group = groups[0];
    let mut filler = f.fill();
    let head = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write_import_href(&import.href, f);
        if !first_group.is_empty() {
            write!(f, space());
            write_import_modifier_group(first_group, f);
        }
        if last > 0 {
            write!(f, ",");
        }
    });
    filler.entry(&soft_line_break_or_space(), &head);
    for (i, group_values) in groups.iter().enumerate().skip(1) {
        let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write_import_modifier_group(group_values, f);
            if i < last {
                write!(f, ",");
            }
        });
        filler.entry(&soft_line_break_or_space(), &content);
    }
    filler.finish();
}

/// One comma group of an import tail, printed as space-separated segments:
/// a maximal token run goes through `write_token_comma_group` (spaces normalize, strings re-quote),
/// a rare non-token value (SCSS interpolated string) reprints structurally.
fn write_import_modifier_group<'a>(values: &[ComponentValue<'a>], f: &mut CssFormatter<'_, 'a>) {
    let mut i = 0;
    while i < values.len() {
        if i > 0 {
            write!(f, space());
        }
        if matches!(values[i], ComponentValue::TokenWithSpan(_)) {
            let run_start = i;
            while i < values.len() && matches!(values[i], ComponentValue::TokenWithSpan(_)) {
                i += 1;
            }
            // `write_token_comma_group` needs a contiguous token slice,
            // so the run is copied out of the enum (cold path, small runs).
            let tokens: Vec<TokenWithSpan<'a>> = values[run_start..i]
                .iter()
                .filter_map(|value| match value {
                    ComponentValue::TokenWithSpan(tok) => Some(*tok),
                    _ => None,
                })
                .collect();
            write_token_comma_group(&tokens, f);
        } else {
            value::write_component_value(&values[i], ValueContext::default(), f);
            i += 1;
        }
    }
}

/// Prints an `@import` href; the quote of a string path is normalized per
/// `singleQuote` like Prettier's `adjustStrings`
/// (interpolated paths re-quote the OUTER quotes only, keeping `@{var}` / `#{}` content verbatim).
fn write_import_href<'a>(href: &ImportPreludeHref<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match href {
        ImportPreludeHref::Str(InterpolableStr::Literal(str)) => {
            value::write_str(str, f);
        }
        ImportPreludeHref::Url(url) => value::write_url(url, f),
        // `url()` with SassScript content reprints structurally:
        // `url($dir+"/path")` → `url($dir + "/path")` like Prettier.
        ImportPreludeHref::Function(func) => {
            value::write_function(func, ValueContext::default(), f);
        }
        // Interpolated string path (`@import './@{var}.less'`):
        // re-quote the outer quotes, keep the interpolation content verbatim.
        href @ ImportPreludeHref::Str(_) => {
            let span = to_span(href.span());
            value::write_requoted_verbatim(source.text_for(&span), f);
        }
    }
}

/// Less `@import (options) href media` (e.g. `@import (reference) "x";`).
/// oxc-css-parser parses the options form as a dedicated `LessImportPrelude`,
/// which otherwise falls into the verbatim catch-all and skips quote normalization.
fn write_less_import_prelude<'a>(import: &LessImportPrelude<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        if !import.options.names.is_empty() {
            write!(f, "(");
            for (i, name) in import.options.names.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ");
                }
                let span = to_span(name.span());
                write!(f, text(source.text_for(&span)));
            }
            write!(f, [")", space()]);
        }
        write_import_href(&import.href, f);
        if let Some(media) = &import.media {
            write!(f, soft_line_break_or_space());
            write_media_query_list_inner(media, f);
        }
    });
    write!(f, group(&indent(&body)));
}

/// Mirrors Prettier's `media-query-list`.
/// Queries joined by `,` + line, wrapped in `group(indent(...))`.
fn write_media_query_list<'a>(list: &MediaQueryList<'a>, f: &mut CssFormatter<'_, 'a>) {
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write_media_query_list_inner(list, f);
    });
    // The indent only governs the inter-query `,`-breaks; a single query never breaks internally,
    // so dropping it avoids leaking a level into an embedded `${}` placeholder break
    // (a media value is flat text, like Prettier).
    if list.queries.len() > 1 {
        write!(f, group(&indent(&body)));
    } else {
        write!(f, group(&body));
    }
}

/// The query list without its own group/indent,
/// for callers that provide their own break scope (`@import` preludes).
fn write_media_query_list_inner<'a>(list: &MediaQueryList<'a>, f: &mut CssFormatter<'_, 'a>) {
    for (i, query) in list.queries.iter().enumerate() {
        if i > 0 {
            write!(f, ",");
            write!(f, soft_line_break_or_space());
        }
        write_media_query(query, f);
    }
}

fn write_media_query<'a>(query: &MediaQuery<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match query {
        MediaQuery::ConditionOnly(condition) => write_media_condition(condition, f),
        MediaQuery::WithType(with_type) => {
            if let Some(modifier) = &with_type.modifier {
                let span = to_span(modifier.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
            }
            let span = to_span(with_type.media_type.span());
            write_maybe_lowercase(source.text_for(&span), f);
            if let Some(condition) = &with_type.condition {
                write!(f, space());
                let and_span = to_span(condition.and.span());
                write_maybe_lowercase(source.text_for(&and_span), f);
                write!(f, space());
                write_media_condition(&condition.condition, f);
            }
        }
        _ => {
            let span = to_span(query.span());
            write!(f, text(source.text_for(&span)));
        }
    }
}

fn write_media_condition<'a>(condition: &MediaCondition<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    for (i, kind) in condition.conditions.iter().enumerate() {
        if i > 0 {
            write!(f, space());
        }
        match kind {
            MediaConditionKind::MediaInParens(in_parens) => {
                write_media_in_parens(in_parens, f);
            }
            MediaConditionKind::And(and) => {
                let span = to_span(and.keyword.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
                write_media_in_parens(&and.media_in_parens, f);
            }
            MediaConditionKind::Or(or) => {
                let span = to_span(or.keyword.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
                write_media_in_parens(&or.media_in_parens, f);
            }
            MediaConditionKind::Not(not) => {
                let span = to_span(not.keyword.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
                write_media_in_parens(&not.media_in_parens, f);
            }
        }
    }
}

fn write_media_in_parens<'a>(in_parens: &MediaInParens<'a>, f: &mut CssFormatter<'_, 'a>) {
    match &in_parens.kind {
        MediaInParensKind::MediaCondition(condition) => {
            write!(f, "(");
            write_media_condition(condition, f);
            write!(f, ")");
        }
        MediaInParensKind::MediaFeature(feature) => {
            write!(f, "(");
            write_media_feature(feature, f);
            write!(f, ")");
        }
        // `@media screen and #{$query}` — the interpolation prints verbatim,
        // like Prettier (no parens of its own).
        MediaInParensKind::SassInterpolation(interpolation) => {
            let span = to_span(&interpolation.span);
            write!(f, text(f.context().source_text().text_for(&span)));
        }
        // W3C `<general-enclosed>` forward-compat fallback:
        // any prelude `oxc-css-parser` can't structure (e.g. `(not all)`, `(screen and (color))`)
        // keeps its tokens with whitespace normalized.
        MediaInParensKind::GeneralEnclosed(seq) => {
            // Lenient bare operand (`print and speech`):
            // `oxc-css-parser` accepts an unparenthesized ident as <general-enclosed>;
            // the source shape must survive, so only reprint parens that exist.
            if f.context().source_text().text_for(&to_span(&in_parens.span)).starts_with('(') {
                write_general_enclosed(seq, f);
            } else {
                write_token_seq_normalized(seq, f);
            }
        }
    }
}

/// `<general-enclosed>` prints its `TokenSeq` inside its own parens,
/// tokens verbatim but whitespace normalized:
/// a source gap becomes one space, paren inner edges stay tight, `:`/`,` space right only.
/// Gap-based spacing never fuses tokens the source kept apart
/// (`and (color)` can't collapse into a function token `and(`).
/// NOTE: Deliberately more normalized than Prettier.
fn write_general_enclosed<'a>(seq: &TokenSeq<'a>, f: &mut CssFormatter<'_, 'a>) {
    write!(f, "(");
    write_token_seq_normalized(seq, f);
    write!(f, ")");
}

/// The `<general-enclosed>` token stream without the surrounding parens.
fn write_token_seq_normalized<'a>(seq: &TokenSeq<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    for (i, tok) in seq.tokens.iter().enumerate() {
        if i > 0 {
            let prev = &seq.tokens[i - 1];
            let needs_space = match (&prev.token, &tok.token) {
                (Token::LParen(_) | Token::LBracket(_), _)
                | (
                    _,
                    Token::RParen(_)
                    | Token::RBracket(_)
                    | Token::Comma(_)
                    | Token::Semicolon(_)
                    | Token::Colon(_),
                ) => false,
                (Token::Comma(_) | Token::Colon(_), _) => true,
                _ => to_span(&prev.span).end != to_span(&tok.span).start,
            };
            if needs_space {
                write!(f, space());
            }
        }
        let span = to_span(&tok.span);
        write!(f, text(source.text_for(&span)));
    }
}

fn write_media_feature_name<'a>(name: &MediaFeatureName<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let span = to_span(name.span());
    match name {
        MediaFeatureName::Ident(_) => write_maybe_lowercase(source.text_for(&span), f),
        MediaFeatureName::SassVariable(_) | MediaFeatureName::PostcssSimpleVar(_) => {
            write!(f, text(source.text_for(&span)));
        }
    }
}

fn write_comparison(kind: &MediaFeatureComparisonKind, f: &mut CssFormatter<'_, '_>) {
    match kind {
        MediaFeatureComparisonKind::LessThan => write!(f, "<"),
        MediaFeatureComparisonKind::LessThanOrEqual => write!(f, "<="),
        MediaFeatureComparisonKind::GreaterThan => write!(f, ">"),
        MediaFeatureComparisonKind::GreaterThanOrEqual => write!(f, ">="),
        MediaFeatureComparisonKind::Equal => write!(f, "="),
    }
}

fn write_media_feature_value<'a>(value: &ComponentValue<'a>, f: &mut CssFormatter<'_, 'a>) {
    // Prettier's `media-value` is flat TEXT (`adjustNumbers(adjustStrings(...))`),
    // a media query never breaks inside a feature value, however long.
    value::write_component_value(value, ValueContext { no_break: true, ..Default::default() }, f);
}

fn write_media_feature<'a>(feature: &MediaFeature<'a>, f: &mut CssFormatter<'_, 'a>) {
    match feature {
        MediaFeature::Plain(plain) => {
            write_media_feature_name(&plain.name, f);
            write!(f, [":", space()]);
            write_media_feature_value(&plain.value, f);
        }
        MediaFeature::Boolean(boolean) => {
            write_media_feature_name(&boolean.name, f);
        }
        MediaFeature::Range(range) => {
            write_media_feature_value(&range.left, f);
            write!(f, space());
            write_comparison(&range.comparison.kind, f);
            write!(f, space());
            write_media_feature_value(&range.right, f);
        }
        MediaFeature::RangeInterval(interval) => {
            write_media_feature_value(&interval.left, f);
            write!(f, space());
            write_comparison(&interval.left_comparison.kind, f);
            write!(f, space());
            write_media_feature_name(&interval.name, f);
            write!(f, space());
            write_comparison(&interval.right_comparison.kind, f);
            write!(f, space());
            write_media_feature_value(&interval.right, f);
        }
    }
}

fn write_supports_condition<'a>(condition: &SupportsCondition<'a>, f: &mut CssFormatter<'_, 'a>) {
    // A fill of keywords and parenthesized terms:
    // a long condition breaks AFTER `and`/`or`, one indent in
    // (`postcss-values` prints the params as a value group, so each word/paren is its own fill entry).
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let mut filler = f.fill();
        for kind in &condition.conditions {
            let (keyword, in_parens) = match kind {
                SupportsConditionKind::SupportsInParens(in_parens) => (None, in_parens),
                SupportsConditionKind::And(and) => (Some(&and.keyword), &and.condition),
                SupportsConditionKind::Or(or) => (Some(&or.keyword), &or.condition),
                SupportsConditionKind::Not(not) => (Some(&not.keyword), &not.condition),
            };
            if let Some(keyword) = keyword {
                let kw = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    let span = to_span(keyword.span());
                    write_maybe_lowercase(f.context().source_text().text_for(&span), f);
                });
                filler.entry(&soft_line_break_or_space(), &kw);
            }
            let term = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write_supports_in_parens(in_parens, f);
            });
            filler.entry(&soft_line_break_or_space(), &term);
        }
        filler.finish();
    });
    // Only a multi-term condition gets the indent:
    // a lone term may carry hardlines of its own (`selector(\n :focus-visible // c\n)`)
    // that must not be re-indented.
    if condition.conditions.len() > 1 {
        write!(f, group(&indent(&body)));
    } else {
        write!(f, group(&body));
    }
}

fn write_supports_in_parens<'a>(in_parens: &SupportsInParens<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    match &in_parens.kind {
        SupportsInParensKind::SupportsCondition(condition) => {
            write!(f, "(");
            write_supports_condition(condition, f);
            write!(f, ")");
        }
        SupportsInParensKind::Feature(feature) => {
            write!(f, "(");
            statement::write_declaration(&feature.decl, f);
            write!(f, ")");
        }
        SupportsInParensKind::Selector(list) => {
            write!(f, "selector(");
            // An inline comment inside `selector(...)` forces the parens
            // open and trails the selector.
            // oxc-css-parser's span may stop at the selector; scan to the `)`.
            let span_end = to_span(in_parens.span()).end;
            let bytes = source.as_bytes();
            let mut scan = span_end.saturating_sub(1) as usize;
            while scan < bytes.len() && bytes[scan] != b')' {
                scan += 1;
            }
            let r_paren = u32::try_from(scan).unwrap_or(span_end);
            let has_inline_inside = f.context().comments().iter_before(r_paren).any(|c| c.inline);
            if has_inline_inside {
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, hard_line_break());
                    selector::write_selector_list(list, selector::SelectorListStyle::Line, f);
                    for &comment in f.context().comments().take_before(r_paren) {
                        write!(f, space());
                        comments::write_single_comment(comment, f);
                    }
                });
                write!(f, [indent(&body), hard_line_break(), ")"]);
            } else {
                selector::write_selector_list(list, selector::SelectorListStyle::Line, f);
                write!(f, ")");
            }
        }
        SupportsInParensKind::Function(func) => {
            value::write_function(func, ValueContext::default(), f);
        }
        SupportsInParensKind::GeneralEnclosed(seq) => write_general_enclosed(seq, f),
        // Sass only: a lone interpolation as a condition operand
        // (`@supports #{"(a: b)"}`); reprints structurally like other interpolations
        // (inner spaces collapse, string literals re-quote).
        SupportsInParensKind::Interpolation(ident) => {
            selector::write_interpolable_ident(ident, f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::split_important_tail;

    #[test]
    fn important_tail_forms() {
        // Plugin regex: /\s+(?:!important|#{(['"]*)!important\1})\s*$/
        assert_eq!(split_important_tail("p-4 flex !important"), Some(("p-4 flex ", "!important")));
        assert_eq!(split_important_tail("p-4 #{!important}"), Some(("p-4 ", "#{!important}")));
        assert_eq!(split_important_tail("p-4 #{'!important'}"), Some(("p-4 ", "#{'!important'}")));
        assert_eq!(
            split_important_tail("p-4 #{\"!important\"}"),
            Some(("p-4 ", "#{\"!important\"}"))
        );
        // Trailing whitespace after the tail is fine (`\s*$`).
        assert_eq!(split_important_tail("p-4 !important  "), Some(("p-4 ", "!important")));
        // Whitespace BEFORE the tail is required (`\s+`).
        assert_eq!(split_important_tail("p-4!important"), None);
        assert_eq!(split_important_tail("!important"), None);
        // Case-sensitive, like the plugin.
        assert_eq!(split_important_tail("p-4 !IMPORTANT"), None);
        // Not at the end -> not a tail.
        assert_eq!(split_important_tail("p-4 !important flex"), None);
    }
}
