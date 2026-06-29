//! At-rule printing. Ports Prettier's `css-atrule` case and the
//! postcss-media-query-parser printing (`media-*` cases).

use cow_utils::CowUtils;
use oxc_formatter_core::{
    Buffer,
    builders::{empty_line, group, hard_line_break, indent, soft_line_break_or_space, space, text},
    write,
};
use raffia::{
    Spanned,
    ast::{
        AtRule, AtRulePrelude, ComponentValue, ImportPrelude, InterpolableStr, KeyframesName,
        MediaCondition, MediaConditionKind, MediaFeature, MediaFeatureComparisonKind,
        MediaFeatureName, MediaInParens, MediaInParensKind, MediaQuery, MediaQueryList,
        SupportsCondition, SupportsConditionKind, SupportsInParens, SupportsInParensKind,
    },
};

use crate::{
    comments::{Gap, classify_gap},
    format::to_span,
    print::{
        CssFormatter, format_with, scss, selector,
        statement::{write_block, write_maybe_lowercase},
        value::{self, ValueContext},
    },
};

/// Mirrors Prettier's `css-atrule`.
pub fn write_at_rule<'a>(at_rule: &AtRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    write!(f, "@");
    let name_span = to_span(at_rule.name.span());
    write_maybe_lowercase(source.text_for(&name_span), f);

    // css-in-js `${}` markers at statement position parse as at-rules.
    // Prettier's `isTemplatePlaceholderNode` rules: the prelude is kept
    // verbatim (postcss leaves params containing `@` markers as an unparsed
    // string), the gap after the name maps to nothing/space/hardline/blank
    // line, and the `;` is printed only when the source has one.
    if at_rule.name.raw.starts_with("prettier-placeholder") {
        write_placeholder_at_rule(at_rule, f);
        return;
    }

    // Comments inside the params: postcss keeps them embedded in the params
    // string / media tokens; reconstruct from the source.
    let region_end = at_rule.block.as_ref().map_or_else(
        || {
            // Up to (excluding) the `;`.
            let end = to_span(at_rule.span()).end;
            let with_semi = crate::print::statement::end_with_semicolon(end, f);
            if with_semi > end { with_semi - 1 } else { end }
        },
        |block| to_span(&block.span).start,
    );
    let has_params_comments = f
        .context()
        .comments()
        .peek()
        .is_some_and(|c| c.span.start >= name_span.end && c.span.end <= region_end);
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
            // String params: whitespace-normalized verbatim, one line.
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
                    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                    write!(f, text(f.allocator().alloc_str(&normalized)));
                }
                write_block_or_semicolon(at_rule, f);
                return;
            }
            _ => {}
        }
    }

    // SCSS control directives wrap the prelude and the gap before `{` in one
    // group: when the prelude breaks, `{` moves to its own line.
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
                    if matches!(&**value, raffia::ast::ComponentValue::SassParenthesizedExpression(_))
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
                crate::comments::write_single_comment(comment, f);
            }
            write!(f, hard_line_break());
            wrote_comment = true;
        }
        if !is_control_directive && !wrote_comment {
            write!(f, space());
        }
        write_block(block, f);
    } else {
        write!(f, ";");
    }
}

/// `@prettier-placeholder-N-id` at-rule body: verbatim prelude, source-driven
/// spacing, `;` only when the source has one. See `write_at_rule`.
fn write_placeholder_at_rule<'a>(at_rule: &AtRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    if let Some(prelude) = &at_rule.prelude {
        let name_end = to_span(at_rule.name.span()).end;
        let prelude_span = to_span(prelude.span());
        let bytes = source.as_bytes();

        let mut pos = name_end;
        if pos == prelude_span.start && bytes[pos as usize] == b':' {
            // A `:` glued to the name: postcss folds it into the NAME itself
            // and Prettier collapses any whitespace after it to one space
            // (`${foo}:\n${bar}` joins back onto one line).
            write!(f, ":");
            pos += 1;
            if pos < prelude_span.end && bytes[pos as usize].is_ascii_whitespace() {
                write!(f, space());
                while pos < prelude_span.end && bytes[pos as usize].is_ascii_whitespace() {
                    pos += 1;
                }
            }
        } else {
            // A `;`-less placeholder swallows the FOLLOWING statements into
            // its prelude, so their leading comments land in this gap; print
            // them with source line structure instead of discarding them.
            for &comment in &f.context().comments().take_before(prelude_span.start).to_vec() {
                write_placeholder_gap(source, pos, comment.span.start, f);
                crate::comments::write_single_comment(comment, f);
                pos = comment.span.end;
            }
            write_placeholder_gap(source, pos, prelude_span.start, f);
            pos = prelude_span.start;
        }

        // The rest is verbatim; embedded newlines stay literal (both Prettier
        // and the parent template printer treat them as `literalline`s).
        write!(f, text(source.slice_range(pos, prelude_span.end)));
        let _ = f.context().comments().take_before(prelude_span.end);
    }
    if at_rule.block.is_some() {
        write_block_or_semicolon(at_rule, f);
    } else {
        let end = to_span(at_rule.span()).end;
        if crate::print::statement::end_with_semicolon(end, f) > end {
            write!(f, ";");
        }
    }
}

/// Source-driven separator inside a placeholder at-rule (see above).
fn write_placeholder_gap(
    source: oxc_formatter_core::SourceText<'_>,
    start: u32,
    end: u32,
    f: &mut CssFormatter<'_, '_>,
) {
    if start == end {
        return;
    }
    match classify_gap(source.bytes_range(start, end)) {
        Gap::None => write!(f, space()),
        Gap::Line => write!(f, hard_line_break()),
        Gap::Blank => write!(f, empty_line()),
    }
}

/// ` {...}` when the at-rule has a block, `;` otherwise.
fn write_block_or_semicolon<'a>(at_rule: &AtRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    if let Some(block) = &at_rule.block {
        write!(f, space());
        write_block(block, f);
    } else {
        write!(f, ";");
    }
}

/// `@media` params containing comments, rebuilt the way
/// postcss-media-query-parser + Prettier's `media-*` cases lay them out:
/// queries split on top-level commas (`,` + line in a group), tokens joined
/// by single spaces, `( feature : value )` re-spaced when the parser would
/// have recognized it (spaces around the `:`), kept verbatim otherwise.
fn write_commented_media_params<'a>(raw: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let queries = split_top_level(raw, b',');
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        for (i, query) in queries.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                write!(f, soft_line_break_or_space());
            }
            let tokens = media_tokens(query.trim());
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

/// `@import` / `@supports` params containing comments, laid out the way
/// Prettier's value parser + fill does. Separator breaks are simulated with
/// static widths (Prettier's fill fit-checks the next item; our core fill
/// measures only up to a hardline). `allow_commas`: comma chunks get an
/// extra indent level for their internal wraps.
fn write_commented_value_params<'a>(
    raw: &'a str,
    start_col: u32,
    allow_commas: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    // One value-parser token (the comma glues to its chunk's tail).
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
        let toks = value_tokens(chunk.trim());
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
                // `)` lands back at column x.
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

/// Value-parser tokens: comments are standalone nodes (split even when
/// touching their neighbors), paren groups glue to their leading word
/// (`url(...)`) and end at `)`.
/// `start` points at the `/` of a `/*`; returns the index of the closing `/`
/// (clamped to the last byte when unterminated).
fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
        i += 1;
    }
    (i + 1).min(bytes.len() - 1)
}

fn value_tokens(raw: &str) -> Vec<&str> {
    let bytes = raw.as_bytes();
    let mut tokens = vec![];
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') && depth == 0 => {
                if let Some(s) = start.take() {
                    tokens.push(&raw[s..i]);
                }
                let end = block_comment_end(bytes, i);
                tokens.push(&raw[i..=end]);
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = block_comment_end(bytes, i);
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

/// Re-quotes `'...'` strings in a token to the preferred quote (Prettier's
/// `adjustStrings`).
fn requote_token<'a>(token: &'a str, f: &CssFormatter<'_, 'a>) -> &'a str {
    let preferred = if f.options().single_quote.value() { '\'' } else { '"' };
    let other = if preferred == '"' { '\'' } else { '"' };
    if !token.contains(other) || token.contains(preferred) {
        return token;
    }
    let replaced = token.cow_replace(other, preferred.encode_utf8(&mut [0; 4]));
    f.allocator().alloc_str(&replaced)
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
    // Words, requoted up front; a lone `:` glues to the previous word.
    let preferred = if f.options().single_quote.value() { '\'' } else { '"' };
    let other = if preferred == '"' { '\'' } else { '"' };
    let requote = |w: &str| -> String {
        if w.contains(other) && !w.contains(preferred) {
            w.cow_replace(other, preferred.encode_utf8(&mut [0; 4])).into_owned()
        } else {
            w.to_string()
        }
    };
    let mut words: Vec<String> = vec![];
    for w in value_tokens(inner.trim()) {
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

/// Whitespace-separated tokens; comments and balanced paren regions glue to
/// adjacent touching text. A `)` closing a single-line paren region ends the
/// token (postcss-media-query-parser splits there); a multi-line region is a
/// `media-unknown` and keeps its touching suffix.
fn media_tokens(raw: &str) -> Vec<&str> {
    let bytes = raw.as_bytes();
    let mut tokens = vec![];
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                if start.is_none() {
                    start = Some(i);
                }
                i = block_comment_end(bytes, i);
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
        // Unparsable for postcss-media-query-parser: verbatim.
        write!(f, text(token));
        return;
    }
    let colon = colon.unwrap();
    write!(f, text(&token[..=open]));
    let feature = inner[..colon].trim();
    let value = inner[colon + 1..].trim();
    let normalized = format!(
        "{}: {}",
        feature.split_whitespace().collect::<Vec<_>>().join(" "),
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    );
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
                raffia::ast::CustomMediaValue::MediaQueryList(list) => {
                    write_media_query_list(list, f);
                }
                raffia::ast::CustomMediaValue::True(ident)
                | raffia::ast::CustomMediaValue::False(ident) => {
                    let span = to_span(ident.span());
                    write!(f, text(source.text_for(&span)));
                }
            }
        }
        AtRulePrelude::Keyframes(name) => write_keyframes_name(name, f),
        AtRulePrelude::CustomSelector(custom) => {
            let custom_span = to_span(custom.custom_selector.span());
            let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                // raffia's CustomSelector span excludes the leading `:`.
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
        AtRulePrelude::Namespace(namespace) => {
            if let Some(prefix) = &namespace.prefix {
                let span = to_span(prefix.span());
                write!(f, text(source.text_for(&span)));
                write!(f, space());
            }
            match &namespace.uri {
                raffia::ast::NamespacePreludeUri::Str(InterpolableStr::Literal(str)) => {
                    value::write_str(str, f);
                }
                raffia::ast::NamespacePreludeUri::Url(url) => value::write_url(url, f),
                uri @ raffia::ast::NamespacePreludeUri::Str(_) => {
                    let span = to_span(uri.span());
                    write!(f, text(source.text_for(&span)));
                }
            }
        }
        // Prettier keeps `@page` params verbatim (e.g. `@page:first` stays).
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
            raffia::ast::SassAtRootKind::Selector(list) => {
                selector::write_selector_list(list, selector::SelectorListStyle::Line, f);
            }
            raffia::ast::SassAtRootKind::Query(query) => {
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
            // Comments force the path list to break, one path per line.
            let last_end = to_span(import.span()).end;
            let has_comments = f.context().comments().iter_before(last_end).next().is_some();
            if has_comments && import.paths.len() > 1 {
                // Comments fuse with the following path into ONE fill chunk
                // (Prettier's comma_group). Prettier's fill treats a chunk
                // with a hardline as never-fitting — our core fill measures
                // up to the hardline and calls it fit — so the separator
                // breaks are simulated here with static widths.
                let all: Vec<crate::comments::CssComment> =
                    f.context().comments().iter_before(last_end).collect();
                let n = import.paths.len();
                let mut leads: Vec<Vec<crate::comments::CssComment>> = Vec::with_capacity(n);
                for (i, path) in import.paths.iter().enumerate() {
                    let path_start = to_span(path.span()).start;
                    leads.push(
                        all.iter()
                            .filter(|c| c.span.end <= path_start)
                            .filter(|c| {
                                i == 0 || c.span.start >= to_span(import.paths[i - 1].span()).end
                            })
                            .copied()
                            .collect(),
                    );
                }
                // Prettier fill: separator stays flat only when
                // [chunk, ", ", next chunk] fits and neither chunk has a
                // comment (hardline).
                let width = u32::from(f.options().line_width.value());
                let indent_w = u32::from(f.options().indent_width.value());
                let chunk_w: Vec<u32> = import
                    .paths
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let span = to_span(p.span());
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
                    for (i, path) in import.paths.iter().enumerate() {
                        if i > 0 {
                            if breaks_before[i] {
                                write!(f, hard_line_break());
                            } else {
                                write!(f, space());
                            }
                        }
                        for &comment in &leads[i] {
                            f.context().comments().take_before(comment.span.end);
                            crate::comments::write_single_comment(comment, f);
                            if comment.inline {
                                write!(f, hard_line_break());
                            } else {
                                write!(f, space());
                            }
                        }
                        value::write_str(path, f);
                        if i + 1 < n {
                            write!(f, ",");
                        }
                    }
                });
                write!(f, indent(&body));
            } else if has_comments {
                let path = &import.paths[0];
                let path_start = to_span(path.span()).start;
                let lead: Vec<crate::comments::CssComment> =
                    f.context().comments().take_before(path_start).to_vec();
                for &comment in &lead {
                    crate::comments::write_single_comment(comment, f);
                    if comment.inline {
                        write!(f, hard_line_break());
                    } else {
                        write!(f, space());
                    }
                }
                value::write_str(path, f);
            } else {
                for (i, path) in import.paths.iter().enumerate() {
                    if i > 0 {
                        write!(f, [",", space()]);
                    }
                    value::write_str(path, f);
                }
            }
        }
        AtRulePrelude::Unknown(unknown) => match &**unknown {
            raffia::ast::UnknownAtRulePrelude::ComponentValue(value) => {
                value::write_component_value(value, ValueContext::default(), f);
            }
            raffia::ast::UnknownAtRulePrelude::TokenSeq(seq) => {
                write_token_seq(seq, f);
            }
        },
        // Sass/Less and not-yet-ported preludes: verbatim.
        _ => {
            let span = to_span(prelude.span());
            write!(f, text(source.text_for(&span)));
        }
    }
}

/// Prints a raw token sequence, collapsing whitespace runs to a single
/// breakable space and keeping tight tokens tight (mirrors how Prettier's
/// `parseValue` + comma-group printing treats unknown at-rule params).
pub fn write_token_seq<'a>(seq: &raffia::ast::TokenSeq<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_tokens(&seq.tokens, f);
}

/// Mini value printer over a raw token stream, normalizing whitespace the way
/// postcss-values-parser + Prettier do: `, ` after commas (no space before),
/// `(`/`)` hug their contents, `:` hugs left, math operators break after.
fn write_tokens<'a>(tokens: &[raffia::token::TokenWithSpan<'a>], f: &mut CssFormatter<'_, 'a>) {
    write_token_value(tokens, true, f);
}

fn token_depth_delta(token: &raffia::token::Token<'_>) -> i32 {
    use raffia::token::Token;
    match token {
        Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_) => 1,
        Token::RParen(_) | Token::RBracket(_) | Token::RBrace(_) => -1,
        _ => 0,
    }
}

/// Comma-separated token value: groups joined by `, ` (breakable), blank
/// lines preserved after multi-token groups.
fn write_token_value<'a>(
    tokens: &[raffia::token::TokenWithSpan<'a>],
    top_level: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    use raffia::token::Token;
    let source = f.context().source_text();
    // Split at top-level commas.
    let mut groups: Vec<&[raffia::token::TokenWithSpan<'a>]> = vec![];
    let mut depth = 0i32;
    let mut start = 0;
    for (i, tok) in tokens.iter().enumerate() {
        depth += token_depth_delta(&tok.token);
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
        // `name( ... )` covering the whole group: the parens govern
        // breaking/indent; anything else gets the continuation indent.
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
        // Top level: fill (as many groups per line as fit).
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
        // Inside parens: `join(line, ...)` — when the paren group breaks,
        // every group goes on its own line; blank lines are preserved after
        // key-value-ish groups.
        for (i, group_tokens) in groups_ref.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                let sep_blank =
                    groups_ref[i - 1].iter().any(|t| matches!(&t.token, Token::Colon(_))) && {
                        let prev_end =
                            groups_ref[i - 1].last().map_or(0, |t| to_span(t.span()).end);
                        let next_start =
                            group_tokens.first().map_or(prev_end, |t| to_span(t.span()).start);
                        crate::comments::classify_gap(source.bytes_range(prev_end, next_start))
                            == crate::comments::Gap::Blank
                    };
                if sep_blank {
                    write!(f, oxc_formatter_core::builders::empty_line());
                } else {
                    write!(f, soft_line_break_or_space());
                }
            }
            write_token_comma_group_grouped(group_tokens, f);
        }
    }
}

/// Space-separated tokens within one comma group; balanced paren regions are
/// printed as breakable groups.
fn write_token_comma_group<'a>(
    tokens: &[raffia::token::TokenWithSpan<'a>],
    f: &mut CssFormatter<'_, 'a>,
) {
    use raffia::token::Token;
    let source = f.context().source_text();

    let hug_lparen = tokens.len() > 1
        && matches!(&tokens[1].token, Token::LParen(_))
        && matches!(&tokens[0].token, Token::Ident(_))
        && !source.text_for(&to_span(tokens[0].span())).eq_ignore_ascii_case("if");

    let mut filler = f.fill();
    let mut i = 0;
    while i < tokens.len() {
        // A run: tokens glued by gap/punctuation rules; a paren region opener
        // ends the run scan (the region is appended to the same run).
        let mut run_end = i + 1;
        // A run starting at an opener swallows its balanced region.
        if matches!(&tokens[i].token, Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_)) {
            let mut depth = 0i32;
            let mut j = i;
            while j < tokens.len() {
                depth += token_depth_delta(&tokens[j].token);
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
                // Opener glues to the run when fused in source, after a
                // colon (`$arg: (...)`), or as a call (`name(...)`).
                let glued = to_span(prev.span()).end == to_span(curr.span()).start
                    || matches!(&prev.token, Token::Colon(_))
                    || (run_end == 1 && hug_lparen);
                if !glued {
                    break;
                }
                // Append the whole balanced region (and continue the run).
                let mut depth = 0i32;
                let mut j = run_end;
                while j < tokens.len() {
                    depth += token_depth_delta(&tokens[j].token);
                    j += 1;
                    if depth == 0 {
                        break;
                    }
                }
                run_end = j;
                continue;
            }
            let tight = to_span(prev.span()).end == to_span(curr.span()).start
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

/// Wrapper: a comma group is its own breakable group with indent —
/// except when it contains paren regions, which provide their own
/// indentation (avoids double-indenting `name(...)` contents).
fn write_token_comma_group_grouped<'a>(
    tokens: &[raffia::token::TokenWithSpan<'a>],
    f: &mut CssFormatter<'_, 'a>,
) {
    use raffia::token::Token;
    // A group that IS one call/paren region delegates breaking to the parens.
    let whole_region = !tokens.is_empty()
        && matches!(&tokens[tokens.len() - 1].token, Token::RParen(_))
        && (matches!(&tokens[0].token, Token::LParen(_))
            || (tokens.len() > 1
                && matches!(&tokens[0].token, Token::Ident(_))
                && matches!(&tokens[1].token, Token::LParen(_))));
    // A `$key: (region)` pair also delegates to the parens.
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
fn write_token_run<'a>(
    run: &[raffia::token::TokenWithSpan<'a>],
    hug_lparen: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    use raffia::token::Token;
    let source = f.context().source_text();
    let mut j = 0;
    while j < run.len() {
        let tok = &run[j];
        // A paren region: recurse.
        if matches!(&tok.token, Token::LParen(_) | Token::LBracket(_) | Token::LBrace(_)) {
            let mut depth = 0i32;
            let mut k = j;
            while k < run.len() {
                depth += token_depth_delta(&run[k].token);
                k += 1;
                if depth == 0 {
                    break;
                }
            }
            // Unbalanced region: print the opener verbatim and move on.
            if depth != 0 || k < j + 2 {
                if j > 0 {
                    write_token_pair_space(run, j, hug_lparen, f);
                }
                let span = to_span(tok.span());
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
                    write!(f, oxc_formatter_core::builders::soft_line_break());
                    write_token_value(inner, false, f);
                });
                write!(
                    f,
                    group(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        write!(f, text(open));
                        write!(f, indent(&body));
                        write!(f, oxc_formatter_core::builders::soft_line_break());
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
        let span = to_span(tok.span());
        match &tok.token {
            Token::Str(_) => write_raw_str(source.text_for(&span), f),
            Token::Number(n) => match value::print_css_number(n.raw) {
                std::borrow::Cow::Borrowed(s) => write!(f, text(s)),
                std::borrow::Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
            },
            Token::Dimension(d) => {
                match value::print_css_number(d.value.raw) {
                    std::borrow::Cow::Borrowed(s) => write!(f, text(s)),
                    std::borrow::Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
                }
                write!(f, text(d.unit.raw));
            }
            Token::Percentage(pct) => {
                match value::print_css_number(pct.value.raw) {
                    std::borrow::Cow::Borrowed(s) => write!(f, text(s)),
                    std::borrow::Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
                }
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
    run: &[raffia::token::TokenWithSpan<'a>],
    j: usize,
    hug_lparen: bool,
    f: &mut CssFormatter<'_, 'a>,
) {
    use raffia::token::Token;
    let prev = &run[j - 1];
    let tok = &run[j];
    let gap = to_span(prev.span()).end != to_span(tok.span()).start;
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
        (Token::Ident(i), Token::Asterisk(_)) if !gap && i.raw.ends_with('-') => false,
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
    let single_quote = f.options().single_quote.value();
    if raw.len() < 2 {
        write!(f, text(raw));
        return;
    }
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
    } else {
        let out = format!("{enclosing}{content}{enclosing}");
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
    let source = f.context().source_text();
    match &import.href {
        raffia::ast::ImportPreludeHref::Str(InterpolableStr::Literal(str)) => {
            value::write_str(str, f);
        }
        raffia::ast::ImportPreludeHref::Url(url) => value::write_url(url, f),
        href @ raffia::ast::ImportPreludeHref::Str(_) => {
            let span = to_span(href.span());
            write!(f, text(source.text_for(&span)));
        }
    }
    if let Some(layer) = &import.layer {
        write!(f, space());
        let span = to_span(layer.span());
        write!(f, text(source.text_for(&span)));
    }
    if let Some(supports) = &import.supports {
        write!(f, [space(), "supports(", ")"]);
        let _ = supports;
    }
    if let Some(media) = &import.media {
        write!(f, space());
        write_media_query_list(media, f);
    }
}

/// Mirrors Prettier's `media-query-list`: queries joined by `,` + line,
/// wrapped in `group(indent(...))`.
pub fn write_media_query_list<'a>(list: &MediaQueryList<'a>, f: &mut CssFormatter<'_, 'a>) {
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        for (i, query) in list.queries.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                write!(f, soft_line_break_or_space());
            }
            write_media_query(query, f);
        }
    });
    write!(f, group(&indent(&body)));
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
    }
}

fn write_media_feature_name<'a>(name: &MediaFeatureName<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let span = to_span(name.span());
    match name {
        MediaFeatureName::Ident(_) => write_maybe_lowercase(source.text_for(&span), f),
        MediaFeatureName::SassVariable(_) => write!(f, text(source.text_for(&span))),
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
    value::write_component_value(value, ValueContext::default(), f);
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
    let source = f.context().source_text();
    for (i, kind) in condition.conditions.iter().enumerate() {
        if i > 0 {
            write!(f, space());
        }
        match kind {
            SupportsConditionKind::SupportsInParens(in_parens) => {
                write_supports_in_parens(in_parens, f);
            }
            SupportsConditionKind::And(and) => {
                let span = to_span(and.keyword.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
                write_supports_in_parens(&and.condition, f);
            }
            SupportsConditionKind::Or(or) => {
                let span = to_span(or.keyword.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
                write_supports_in_parens(&or.condition, f);
            }
            SupportsConditionKind::Not(not) => {
                let span = to_span(not.keyword.span());
                write_maybe_lowercase(source.text_for(&span), f);
                write!(f, space());
                write_supports_in_parens(&not.condition, f);
            }
        }
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
            crate::print::statement::write_declaration_inline(&feature.decl, f);
            write!(f, ")");
        }
        SupportsInParensKind::Selector(list) => {
            write!(f, "selector(");
            // An inline comment inside `selector(...)` forces the parens
            // open and trails the selector.
            // raffia's span may stop at the selector; scan to the `)`.
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
                        crate::comments::write_single_comment(comment, f);
                    }
                });
                write!(f, [indent(&body), hard_line_break(), ")"]);
            } else {
                selector::write_selector_list(list, selector::SelectorListStyle::Line, f);
                write!(f, ")");
            }
        }
        SupportsInParensKind::Function(func) => {
            let func_value = raffia::ast::ComponentValue::Function(func.clone());
            value::write_component_value(&func_value, ValueContext::default(), f);
        }
    }
}
