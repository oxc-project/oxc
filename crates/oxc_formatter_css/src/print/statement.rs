use cow_utils::CowUtils;
use oxc_css_parser::ast::{
    ComponentValue, Declaration, InterpolableIdent, QualifiedRule, SimpleBlock, Statement,
};

use oxc_formatter_core::{
    Buffer, arena_cow_str,
    builders::{block_indent, dedent, empty_line, hard_line_break, indent, space, text},
    write,
};

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, flush_trailing_inside_comments,
        is_suppression_comment, last_line_has_inline_comment, write_leading_comments,
        write_trailing_same_line_comment,
    },
    format::to_span,
    print::{
        CssFormatter, at_rule, format_with, less, postcss_simple_vars, scss, selector,
        value::{self, ValueContext},
        write_maybe_lowercase,
    },
};

/// Start offset of a statement.
pub(super) fn stmt_start(stmt: &Statement<'_>) -> u32 {
    to_span(stmt.span()).start
}

/// End offset of a statement, extended over a trailing semicolon
/// (oxc-css-parser's spans exclude it; postcss's `locEnd` includes it,
/// and blank-line detection counts from after the `;`).
pub(super) fn stmt_end(stmt: &Statement<'_>, f: &CssFormatter<'_, '_>) -> u32 {
    end_with_semicolon(to_span(stmt.span()).end, f)
}

/// Extends `end` over any whitespace, comments and a final `;` in the source.
/// End of an at-rule's params region: the block start, or (without a block) the span end,
/// extended over a trailing `;` and shrunk back so the `;` itself stays outside the region.
pub(super) fn params_region_end<'a>(
    block: Option<&SimpleBlock<'a>>,
    span_end: u32,
    f: &CssFormatter<'_, 'a>,
) -> u32 {
    block.map_or_else(
        || {
            let with_semi = end_with_semicolon(span_end, f);
            if with_semi > span_end { with_semi - 1 } else { span_end }
        },
        |block| to_span(&block.span).start,
    )
}

pub(super) fn end_with_semicolon(end: u32, f: &CssFormatter<'_, '_>) -> u32 {
    let source = f.context().source_text();
    let bytes = source.as_bytes();
    let mut i = end as usize;
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        // Skip a block comment between the statement and its `;`
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            match source[i + 2..].find("*/") {
                Some(close) => {
                    i = i + 2 + close + 2;
                    continue;
                }
                None => break,
            }
        }
        break;
    }
    if i < bytes.len() && bytes[i] == b';' { u32::try_from(i + 1).unwrap_or(end) } else { end }
}

/// Emits `statements` separated by hard lines, preserving at most one blank line
/// between consecutive statements, flushing comments at their source positions.
/// Trailing same-line comments are only claimed when they end before `upper`
/// (a block's closing `}`), so inline rules don't steal comments that belong
/// to the parent. Pass `u32::MAX` at the stylesheet root.
///
/// Mirrors Prettier's `printSequence`.
pub(super) fn write_statement_sequence_bounded<'a>(
    statements: &[Statement<'a>],
    upper: u32,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    // postcss "swallow": a `;`-less placeholder statement absorbs the following
    // statements as a verbatim prelude until a source `;` ends it,
    // so prettier does NOT add a `;` to a declaration that lands in that run
    // (`${m}\ncolor: red` -> no `;`, but `${m}\ncolor: red;\nx: 1` -> `x: 1;`).
    // Track that run so swallowed declarations keep a source-driven `;`.
    let mut swallowed = false;
    for (i, stmt) in statements.iter().enumerate() {
        let start = stmt_start(stmt);
        if i > 0 {
            let prev_end = stmt_end(&statements[i - 1], f);
            // Trailing comment on the same line as the previous statement
            // (but not one that sits after the NEXT statement on that line).
            write_trailing_same_line_comment(prev_end, upper.min(start), f);
            // The gap considered is from the end of the previous statement to the
            // next printed position (comment or stmt).
            let next_start =
                f.context().comments().peek().map_or(start, |c| c.span.start.min(start));
            let gap = classify_gap(source.bytes_range(prev_end, next_start));
            // A `;`-less css-in-js placeholder preserves the source whitespace that
            // follows it: Prettier keeps `${a} ${b}` on one line and `${a}\n${b}` on two
            // (postcss swallows the run as a verbatim prelude).
            // Other statements always start on a new line.
            if matches!(statements[i - 1], Statement::Placeholder(_)) && gap == Gap::None {
                write!(f, space());
            } else {
                write!(f, hard_line_break());
                if gap == Gap::Blank {
                    write!(f, empty_line());
                }
            }
        }

        let is_suppressed = f
            .context()
            .comments()
            .iter_before(start)
            .last()
            .is_some_and(|c| is_suppression_comment(source, c));
        flush_leading_comments(start, f);
        let end = stmt_end(stmt, f);
        // `stmt_end` already consumes a trailing `;`,
        // so a source `;` is present exactly when it advanced past the raw statement span.
        let has_source_semicolon = end > to_span(stmt.span()).end;
        if is_suppressed {
            write!(f, text(source.slice_range(start, end)));
        } else if swallowed && let Statement::Declaration(decl) = stmt {
            // Inside a swallowed run: keep the source `;` instead of forcing one
            write_declaration(decl, f);
            if has_source_semicolon {
                write!(f, ";");
            }
        } else {
            write_statement(stmt, f);
        }
        // A `;`-less placeholder opens a swallowed run; any source `;` closes it
        swallowed = if has_source_semicolon {
            false
        } else {
            swallowed || matches!(stmt, Statement::Placeholder(_))
        };
        // Discard comments inside spans the statement printer didn't claim
        // (e.g. inside selectors/values that are still printed verbatim),
        // so the cursor never points before an already-printed position.
        let _ = f.context().comments().take_before(end);
    }
    if let Some(last) = statements.last() {
        write_trailing_same_line_comment(stmt_end(last, f), upper, f);
    }
}

/// Dispatch a single statement.
pub(super) fn write_statement<'a>(stmt: &Statement<'a>, f: &mut CssFormatter<'_, 'a>) {
    match stmt {
        Statement::QualifiedRule(rule) => write_qualified_rule(rule, f),
        Statement::Declaration(decl) => {
            write_declaration(decl, f);
            if matches!(decl.name, InterpolableIdent::Placeholder(_)) {
                // A css-in-js `${foo}: ${bar}` declaration keeps the source `;`
                // (Prettier preserves it: `${foo}: ${bar};` vs `${foo}: ${bar}`).
                let end = to_span(decl.span()).end;
                if end_with_semicolon(end, f) > end {
                    write!(f, ";");
                }
            } else if !matches!(decl.value.last(), Some(ComponentValue::SassNestingDeclaration(_)))
                || matches!(&decl.name, InterpolableIdent::Literal(ident) if ident.name.starts_with("--"))
            {
                // No `;` after a nested declaration block (`background: { ... }`),
                // except a custom-property rule block (`--p: { ... };`).
                // See AGENTS.md "Known divergences" for the `--*` exception.
                write!(f, ";");
            }
        }
        Statement::AtRule(at_rule) => at_rule::write_at_rule(at_rule, f),
        // A css-in-js placeholder standing in for a whole statement (`${mixin}`).
        // The host replaces the marker with `${expr}`; no separator is added here
        // (the statement loop and the host's template own that).
        Statement::Placeholder(placeholder) => {
            super::write_placeholder(placeholder, f);
            // Preserve a trailing `;` from the source
            // (Prettier keeps `${foo};` as `${foo};` and `${foo}` as `${foo}`):
            // the `;` is consumed by the statement separator, so recover it from the source gap.
            let end = to_span(&placeholder.span).end;
            if end_with_semicolon(end, f) > end {
                write!(f, ";");
            }
        }
        Statement::LessVariableDeclaration(decl) => {
            if less::write_less_variable_declaration(decl, f) {
                write!(f, ";");
            }
        }
        Statement::LessMixinDefinition(def) => less::write_less_mixin_definition(def, f),
        Statement::LessConditionalQualifiedRule(rule) => {
            less::write_less_conditional_qualified_rule(rule, f);
        }
        Statement::LessMixinCall(call) => {
            less::write_less_mixin_call_statement(call, f);
            write!(f, ";");
        }
        Statement::SassVariableDeclaration(decl) => {
            scss::write_sass_variable_declaration(decl, f);
            write!(f, ";");
        }
        Statement::PostcssSimpleVarDeclaration(decl) => {
            postcss_simple_vars::write_postcss_simple_var_declaration(decl, f);
            write!(f, ";");
        }
        Statement::SassIfAtRule(if_rule) => scss::write_sass_if_at_rule(if_rule, f),
        Statement::UnknownSassAtRule(unknown) => {
            // Same string-params contract as the Unknown prelude in `write_at_rule`:
            // Prettier prints unknown at-rule params verbatim.
            let source = f.context().source_text();
            write!(f, "@");
            let name_span = to_span(unknown.name.span());
            write_maybe_lowercase(source.text_for(&name_span), f);
            let region_end =
                params_region_end(unknown.block.as_ref(), to_span(&unknown.span).end, f);
            let prelude_start = unknown
                .prelude
                .as_ref()
                .map_or(region_end, |prelude| to_span(prelude.span()).start);
            at_rule::write_verbatim_at_rule_tail(
                name_span.end,
                prelude_start,
                unknown.block.as_ref(),
                region_end,
                f,
            );
        }
        Statement::KeyframeBlock(keyframe_block) => {
            for (i, sel) in keyframe_block.selectors.iter().enumerate() {
                if i > 0 {
                    write!(f, ",");
                    write!(f, hard_line_break());
                }
                selector::write_keyframe_selector(sel, f);
            }
            write!(f, space());
            write_block(&keyframe_block.block, f);
        }
        // Not yet ported: emit the original source verbatim.
        // - LessExtendRule
        // - LessFunctionCall
        // - LessVariableCall
        _ => {
            let span = to_span(stmt.span());
            let source = f.context().source_text();
            write!(f, text(source.text_for(&span)));
            write!(f, ";");
        }
    }
}

/// Mirrors Prettier's `css-rule`.
fn write_qualified_rule<'a>(rule: &QualifiedRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let sel_span = to_span(rule.selector.span());
    let block_start = to_span(rule.block.span()).start;
    // Comments inside the selector (both `//` and `/* */`) make Prettier print the raw selector verbatim (`selector-unknown`).
    // Reordering them would change which compound they annotate.
    // A trailing `//` comment pushes `{` to the next line.
    let has_inline_comment =
        f.context().comments().iter_before(block_start).any(|c| c.span.start >= sel_span.start);
    if has_inline_comment {
        let raw = source.slice_range(sel_span.start, block_start).trim_end();
        let _ = f.context().comments().take_before(block_start);
        write!(f, text(raw));
        if last_line_has_inline_comment(raw) {
            write!(f, hard_line_break());
        } else {
            write!(f, space());
        }
        write_block(&rule.block, f);
        return;
    }
    selector::write_selector_list(&rule.selector, selector::SelectorListStyle::Hard, f);
    write!(f, space());
    let is_icss = selector::is_icss_selector(&rule.selector);
    let was = f.context().in_icss_rule().replace(is_icss);
    write_block(&rule.block, f);
    f.context().in_icss_rule().set(was);
}

/// Mirrors Prettier's `css-decl`.
/// Used without a trailing semicolon for `@supports (...)` features (the caller skips the `;`).
pub(super) fn write_declaration<'a>(decl: &Declaration<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(decl.name.span());
    let prop = source.text_for(&name_span);
    // Legacy IE hack prefix glued to the property name
    // (`*color: red`; `oxc-css-parser` also accepts `.`/`:`/`#` in Css mode);
    // postcss keeps it as part of the prop, so Prettier preserves it.
    if let Some(prefix) = decl.name_prefix {
        write!(f, text(f.allocator().alloc_str(prefix.encode_utf8(&mut [0; 4]))));
    }
    if let InterpolableIdent::Placeholder(placeholder) = &decl.name {
        // A css-in-js placeholder property name (`${foo}: ...`) is a typed marker
        // the host replaces with `${expr}`; never lowercase it like a real property.
        super::write_placeholder(placeholder, f);
    } else if f.context().in_less_detached().get() || f.context().in_icss_rule().get() {
        // Less detached rulesets (`parentNode.variable`) and ICSS rules
        // (`insideIcssRuleNode`) keep property-name casing.
        write!(f, text(prop));
    } else {
        write_maybe_lowercase(prop, f);
    }
    // Prettier prints the WHOLE `raws.between`
    // (prop → value, `:` and any comments included) trimmed but otherwise verbatim,
    // with a space before a leading `//` comment and a space before the value.
    let colon_end = to_span(&decl.colon_span).end;
    let between_upper =
        if decl.value.is_empty() { colon_end } else { to_span(decl.value[0].span()).start };
    let between = source.slice_range(name_span.end, between_upper);
    let trimmed_between = between.trim_ascii();
    // `lastLineHasInlineComment`: the value drops to the next line,
    // one indent under the prop (`indent([hardline, dedent(value)])`).
    let between_breaks = trimmed_between != ":"
        && trimmed_between.rsplit(['\n', '\r']).next().unwrap_or(trimmed_between).contains("//");
    if trimmed_between == ":" {
        write!(f, ":");
    } else {
        let _ = f.context().comments().take_before(between_upper);
        if trimmed_between.starts_with("//") {
            write!(f, space());
        }
        // Runs of spaces before a same-line `//` comment collapse to one
        // (postcss-less keeps inline comments out of `raws.between`).
        if trimmed_between.contains("  //") {
            let mut out = String::with_capacity(trimmed_between.len());
            for (i, line) in trimmed_between.split('\n').enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                if let Some(pos) = line.find("//") {
                    let (head, tail) = line.split_at(pos);
                    let trimmed_head = head.trim_end();
                    if head.len() > trimmed_head.len() + 1 && !trimmed_head.is_empty() {
                        out.push_str(trimmed_head);
                        out.push(' ');
                        out.push_str(tail);
                        continue;
                    }
                }
                out.push_str(line);
            }
            write!(f, text(f.allocator().alloc_str(&out)));
        } else {
            write!(f, text(trimmed_between));
        }
    }
    if decl.value.is_empty() {
        // Custom properties with a whitespace-only value keep it verbatim
        // (`--one-space: ;` stays as-is). Scan up to the `;` in the source.
        let colon_end = to_span(&decl.colon_span).end;
        let bytes = source.as_bytes();
        let mut i = colon_end as usize;
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b';' && i > colon_end as usize {
            write!(f, text(source.slice_range(colon_end, u32::try_from(i).unwrap())));
        }
    } else {
        write!(f, space());
        let prop_lower = prop.cow_to_ascii_lowercase();
        let prop_lower: &'a str = arena_cow_str(&prop_lower, f);

        // `filter: progid:...` values are printed verbatim.
        let value_start = to_span(decl.value[0].span()).start;
        let value_end = to_span(decl.value[decl.value.len() - 1].span()).end;
        let value_text = source.slice_range(value_start, value_end);
        if value_text.starts_with("progid:") {
            write!(f, text(value_text));
        } else if (value_text.contains("\\(") || value_text.contains("\\)"))
            && value_text.contains('\n')
        {
            // Escaped parens break postcss's value parser:
            // the whole value is a `value-unknown`, printed verbatim.
            write!(f, text(value_text));
            let _ = f.context().comments().take_before(value_end);
        } else if (value_text.starts_with('"') || value_text.starts_with('\''))
            && value_text.ends_with(value_text.as_bytes()[0] as char)
            && value_text[1..value_text.len() - 1].contains("#{")
            && decl.value.len() > 1
            // Only when oxc-css-parser split ONE string apart (no gaps between parts).
            && decl.value.windows(2).all(|w| {
                to_span(w[0].span()).end == to_span(w[1].span()).start
            })
        {
            // A string containing SCSS interpolation that `oxc-css-parser` tokenized apart (`"#{".5"}"`):
            // print the pieces glued.
            // Numbers get normalized, strings keep their quotes.
            // Interpolation among the pieces → `value-unknown`: verbatim,
            // except bare quoted numbers, which postcss saw unquoted.
            value::write_requoted_verbatim(value_text, f);
            let _ = f.context().comments().take_before(value_end);
        } else if prop.starts_with("--") && value_text.starts_with('{') {
            if value_text.trim_end().ends_with('}')
                && value_text.bytes().filter(|&b| b == b'{').count() == 1
            {
                // `--prop: { decls };` custom-property rule blocks
                // (postcss re-parses these as a rule): print the token stream as a block.
                value::flush_value_comments(value_start, f);
                write_custom_property_block(value_text, f);
            } else {
                // Unparsable rule-ish value (e.g. missing `;` swallowed the following declarations):
                // keep the source verbatim.
                write!(f, text(value_text));
            }
            // The raw text includes any comments; drop them from the cursor.
            let _ = f.context().comments().take_before(value_end);
        } else {
            // Custom property values (`--*`) are parsed as a normal `<declaration-value>`
            // via the `try_parsing_value_in_custom_property` parser option, matching Prettier (postcss).
            // The parser keeps the raw token stream when the value does not parse (e.g. `--p: { decls };` rule blocks),
            // so anything reaching here is uniformly a structured `ComponentValue` slice handled below.
            let values = &*decl.value;

            let ctx = ValueContext {
                decl_prop: Some(prop_lower),
                // Prettier applies `removeLines` to `composes` values.
                no_break: prop_lower == "composes",
                // Prettier's printer counts a multi-line between's FULL width,
                // so the first trailing comment always wraps.
                tail_break: trimmed_between.contains('\n'),
                ..ValueContext::default()
            };

            // The `;` position bounds wrapped trailing comments.
            // Only when such comments exist (keeps simple declarations on the single-component fast path).
            let decl_end_pre = to_span(decl.span()).end;
            let semi = end_with_semicolon(decl_end_pre, f);
            let bound = if semi > decl_end_pre { semi - 1 } else { decl_end_pre };
            // A single interpolated component is exempt,
            // its fill-chunk fit ignores the line tail (see `is_single_sass_interpolation`).
            let has_tail = decl.important.is_none()
                && !value::is_single_sass_interpolation(values)
                && f.context().comments().iter_before(bound).any(|c| c.span.start >= value_end);
            let ctx = ValueContext { tail_bound: has_tail.then_some(bound), ..ctx };
            // `prop: <values> { decls }`:
            // A trailing nested block hugs the declaration; leading values are space-joined flat.
            if let Some(ComponentValue::SassNestingDeclaration(nesting)) = values.last() {
                for v in &values[..values.len() - 1] {
                    value::write_component_value(v, ctx, f);
                    write!(f, space());
                }
                write_block(&nesting.block, f);
                return;
            }
            if between_breaks {
                // `indent([hardline, dedent(value)])`,
                // the value's own indent is cancelled so it sits exactly one level under the prop.
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, hard_line_break());
                    let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        value::write_declaration_value(values, ctx, f);
                    });
                    write!(f, dedent(&inner));
                });
                write!(f, indent(&body));
            } else {
                value::write_declaration_value(values, ctx, f);
            }
        }
    }
    if let Some(important) = &decl.important {
        value::flush_trailing_value_comments(to_span(important.span()).start, f);
        write!(f, [space(), "!important"]);
    }
    // Comments between the value and the `;`.
    // NOTE: the `;` position is the flush bound, so a comment after `;` stays for the trailing-comment pass.
    let decl_end = to_span(decl.span()).end;
    let end = end_with_semicolon(decl_end, f);
    let bound = if end > decl_end { end - 1 } else { decl_end };
    if let Some(comment_end) = value::flush_trailing_value_comments(bound, f) {
        // Preserve a source gap between the last comment and the `;`
        if end > decl_end && comment_end < end - 1 {
            write!(f, space());
        }
    }
}

/// Prints a `--prop: { a: b; c: d }` rule-block value by re-flowing the raw text:
/// one declaration per line, normalized `prop: value;` spacing.
fn write_custom_property_block<'a>(value_text: &'a str, f: &mut CssFormatter<'_, 'a>) {
    let inner = value_text.trim_end();
    let inner = &inner[1..inner.len() - 1];
    // Split on `;`, keeping a comment that follows a `;` on the SAME line
    // attached to the previous declaration (`...; /* c */`).
    let mut decls: Vec<(&str, Option<&str>)> = vec![];
    for seg in inner.split(';') {
        let (first_line, rest) = match seg.find('\n') {
            Some(idx) => (&seg[..idx], &seg[idx..]),
            None => (seg, ""),
        };
        let prefix = first_line.trim();
        if !decls.is_empty()
            && !prefix.is_empty()
            && prefix.starts_with("/*")
            && prefix.ends_with("*/")
            && !rest.trim().is_empty()
        {
            decls.last_mut().unwrap().1 = Some(prefix);
            let remainder = rest.trim();
            if !remainder.is_empty() {
                decls.push((remainder, None));
            }
        } else {
            let t = seg.trim();
            if !t.is_empty() {
                decls.push((t, None));
            }
        }
    }
    if decls.is_empty() {
        write!(f, ["{", hard_line_break(), "}"]);
        return;
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(f, hard_line_break());
        for (i, (decl, trailing)) in decls.iter().enumerate() {
            if i > 0 {
                write!(f, hard_line_break());
            }
            // Re-flow line by line (keeps interior comments in place)
            for (j, line) in decl.lines().map(str::trim).filter(|l| !l.is_empty()).enumerate() {
                if j > 0 {
                    write!(f, hard_line_break());
                }
                // Normalize `prop : value` spacing on declaration lines:
                // split at the first colon OUTSIDE comments, keep the prop side verbatim,
                // re-space the value side tokens.
                if let Some(colon) = find_colon_outside_comments(line) {
                    let (p, v) = line.split_at(colon);
                    let v = &v[1..];
                    write!(f, [text(p.trim_end()), ":", space()]);
                    let normalized = respace_value_tokens(v.trim());
                    if normalized == v.trim() {
                        write!(f, text(v.trim()));
                    } else {
                        write!(f, text(f.allocator().alloc_str(&normalized)));
                    }
                } else {
                    write!(f, text(line));
                }
            }
            // A trailing comment-only segment gets no semicolon
            if !(i + 1 == decls.len() && decl.starts_with("/*") && decl.ends_with("*/")) {
                write!(f, ";");
            }
            if let Some(comment) = trailing {
                write!(f, [space(), text(comment)]);
            }
        }
    });
    write!(f, ["{", indent(&body), hard_line_break(), "}"]);
}

/// First `:` outside `/* ... */` comments, or None.
fn find_colon_outside_comments(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            match line[i + 2..].find("*/") {
                Some(close) => i = i + 2 + close + 2,
                None => return None,
            }
            continue;
        }
        if bytes[i] == b':' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Single spaces between value tokens and comments (`/* c */ #fff /* c */`).
fn respace_value_tokens(v: &str) -> String {
    let mut out = String::with_capacity(v.len() + 8);
    let bytes = v.as_bytes();
    let mut i = 0;
    let mut last_was_token = false;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            match v[i + 2..].find("*/") {
                Some(close) => i = i + 2 + close + 2,
                None => i = bytes.len(),
            }
        } else {
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && !(bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*'))
            {
                i += 1;
            }
        }
        if last_was_token {
            out.push(' ');
        }
        out.push_str(&v[start..i]);
        last_was_token = true;
    }
    out
}

/// Mirrors Prettier's block printing: `{` + indented statements + `}`.
/// An empty block prints as `{\n}`.
pub(super) fn write_block<'a>(block: &SimpleBlock<'a>, f: &mut CssFormatter<'_, 'a>) {
    let depth = f.context().block_depth();
    depth.set(depth.get() + 1);
    write_block_inner(block, f);
    let depth = f.context().block_depth();
    depth.set(depth.get() - 1);
}

fn write_block_inner<'a>(block: &SimpleBlock<'a>, f: &mut CssFormatter<'_, 'a>) {
    let block_span = to_span(&block.span);
    let r_curly = block_span.end.saturating_sub(1);

    write!(f, "{");

    if block.statements.is_empty() {
        // Dangling comments inside an otherwise empty block
        let comments = f.context().comments().take_before(r_curly);
        if comments.is_empty() {
            write!(f, hard_line_break());
        } else {
            let body = format_with(|f: &mut CssFormatter<'_, 'a>| {
                let last_end = comments.last().unwrap().span.end;
                write_leading_comments(comments, last_end, f);
            });
            write!(f, block_indent(&body));
        }
    } else {
        let body = format_with(|f: &mut CssFormatter<'_, 'a>| {
            write_statement_sequence_bounded(&block.statements, r_curly, f);
            let last_end = block.statements.last().map_or(block_span.start, |s| stmt_end(s, f));
            flush_trailing_inside_comments(last_end, r_curly, f);
        });
        write!(f, block_indent(&body));
    }

    write!(f, "}");
}
