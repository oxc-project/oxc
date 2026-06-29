use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_formatter_core::{
    Buffer,
    builders::{block_indent, hard_line_break, indent, space, text},
    write,
};
use raffia::{
    Spanned,
    ast::{Declaration, QualifiedRule, SimpleBlock, Statement},
};

use crate::{
    comments::{flush_trailing_inside_comments, write_leading_comments},
    format::to_span,
    print::{
        CssFormatter, at_rule, format_with, less, scss, selector,
        value::{self, ValueContext},
    },
};

/// Start offset of a statement.
pub fn stmt_start(stmt: &Statement<'_>) -> u32 {
    to_span(stmt.span()).start
}

/// End offset of a statement, extended over a trailing semicolon
/// (raffia's spans exclude it; postcss's `locEnd` includes it, and blank-line
/// detection counts from after the `;`).
pub fn stmt_end(stmt: &Statement<'_>, f: &CssFormatter<'_, '_>) -> u32 {
    end_with_semicolon(to_span(stmt.span()).end, f)
}

/// Extends `end` over any whitespace, comments and a final `;` in the source.
/// End of an at-rule's params region: the block start, or (without a block)
/// the span end — extended over a trailing `;` and shrunk back so the `;`
/// itself stays outside the region.
pub fn params_region_end<'a>(
    block: Option<&raffia::ast::SimpleBlock<'a>>,
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

pub fn end_with_semicolon(end: u32, f: &CssFormatter<'_, '_>) -> u32 {
    let source = f.context().source_text();
    let bytes = source.as_bytes();
    let mut i = end as usize;
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        // Skip a block comment between the statement and its `;`.
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

/// Dispatch a single statement.
pub fn write_statement<'a>(stmt: &Statement<'a>, f: &mut CssFormatter<'_, 'a>) {
    match stmt {
        Statement::QualifiedRule(rule) => write_qualified_rule(rule, f),
        Statement::Declaration(decl) => {
            write_declaration(decl, f);
            // Nested declaration blocks (`background: { ... }`) get no `;`.
            if !matches!(
                decl.value.last(),
                Some(raffia::ast::ComponentValue::SassNestingDeclaration(_))
            ) {
                write!(f, ";");
            }
        }
        Statement::AtRule(at_rule) => at_rule::write_at_rule(at_rule, f),
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
        Statement::SassIfAtRule(if_rule) => scss::write_sass_if_at_rule(if_rule, f),
        Statement::UnknownSassAtRule(unknown) => {
            // Same string-params contract as the Unknown prelude in
            // `write_at_rule`: Prettier prints unknown at-rule params
            // verbatim.
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
    // Comments inside the selector (both `//` and `/* */`) make Prettier
    // print the raw selector verbatim (`selector-unknown`) — reordering them
    // would change which compound they annotate. A trailing `//` comment
    // pushes `{` to the next line.
    let has_inline_comment =
        f.context().comments().iter_before(block_start).any(|c| c.span.start >= sel_span.start);
    if has_inline_comment {
        let raw = source.slice_range(sel_span.start, block_start).trim_end();
        let _ = f.context().comments().take_before(block_start);
        write!(f, text(raw));
        if crate::comments::last_line_has_inline_comment(raw) {
            write!(f, hard_line_break());
        } else {
            write!(f, space());
        }
        write_block(&rule.block, f);
        return;
    }
    selector::write_selector_list(&rule.selector, selector::SelectorListStyle::Hard, f);
    write!(f, space());
    let raw_sel = source.text_for(&sel_span);
    let is_icss = raw_sel.starts_with(":import") || raw_sel.starts_with(":export");
    let was = f.context().in_icss_rule().replace(is_icss);
    write_block(&rule.block, f);
    f.context().in_icss_rule().set(was);
}

/// Mirrors Prettier's `maybeToLowerCase`: lowercase unless the identifier
/// contains variable/interpolation markers.
pub fn write_maybe_lowercase<'a>(value: &'a str, f: &mut CssFormatter<'_, 'a>) {
    if value.contains('$')
        || value.contains('@')
        || value.contains('#')
        || value.starts_with('%')
        || value.starts_with("--")
        || value.starts_with(":--")
        || (value.contains('(') && value.contains(')'))
    {
        write!(f, text(value));
    } else {
        match value.cow_to_ascii_lowercase() {
            Cow::Borrowed(s) => write!(f, text(s)),
            Cow::Owned(s) => write!(f, text(f.allocator().alloc_str(&s))),
        }
    }
}

/// A declaration without trailing semicolon (also used for `@supports (...)` features).
pub fn write_declaration_inline<'a>(decl: &Declaration<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_declaration(decl, f);
}

/// Mirrors Prettier's `css-decl`.
fn write_declaration<'a>(decl: &Declaration<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(decl.name.span());
    let prop = source.text_for(&name_span);
    if f.context().in_less_detached().get() || f.context().in_icss_rule().get() {
        // Less detached rulesets (`parentNode.variable`) and ICSS rules
        // (`insideIcssRuleNode`) keep property-name casing.
        write!(f, text(prop));
    } else {
        write_maybe_lowercase(prop, f);
    }
    // Prettier prints the WHOLE `raws.between` (prop → value, `:` and any
    // comments included) trimmed but otherwise verbatim, with a space before
    // a leading `//` comment and a space before the value.
    let colon_end = to_span(&decl.colon_span).end;
    let between_upper =
        if decl.value.is_empty() { colon_end } else { to_span(decl.value[0].span()).start };
    let between = source.slice_range(name_span.end, between_upper);
    let trimmed_between = between.trim_ascii();
    // `lastLineHasInlineComment`: the value drops to the next line, one
    // indent under the prop (`indent([hardline, dedent(value)])`).
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
    let mut reparsed_important_start: Option<u32> = None;
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
        let prop_lower: &'a str = match prop_lower {
            Cow::Borrowed(s) => s,
            Cow::Owned(s) => f.allocator().alloc_str(&s),
        };

        // `filter: progid:...` values are printed verbatim.
        let value_start = to_span(decl.value[0].span()).start;
        let value_end = to_span(decl.value[decl.value.len() - 1].span()).end;
        let value_text = source.slice_range(value_start, value_end);
        if value_text.starts_with("progid:") {
            write!(f, text(value_text));
        } else if (value_text.contains("\\(") || value_text.contains("\\)"))
            && value_text.contains('\n')
        {
            // Escaped parens break postcss's value parser: the whole value
            // is a `value-unknown`, printed verbatim.
            write!(f, text(value_text));
            let _ = f.context().comments().take_before(value_end);
        } else if (value_text.starts_with('"') || value_text.starts_with('\''))
            && value_text.ends_with(value_text.as_bytes()[0] as char)
            && value_text[1..value_text.len() - 1].contains("#{")
            && decl.value.len() > 1
            // Only when raffia split ONE string apart (no gaps between parts).
            && decl.value.windows(2).all(|w| {
                to_span(w[0].span()).end == to_span(w[1].span()).start
            })
        {
            // A string containing SCSS interpolation that raffia tokenized
            // apart (`"#{".5"}"`): print the pieces glued — numbers get
            // normalized, strings keep their quotes.
            // Interpolation among the pieces → `value-unknown`: verbatim,
            // except bare quoted numbers, which postcss saw unquoted.
            value::write_requoted_verbatim(value_text, f);
            let _ = f.context().comments().take_before(value_end);
        } else if prop.starts_with("--") && value_text.starts_with('{') {
            if value_text.trim_end().ends_with('}')
                && value_text.bytes().filter(|&b| b == b'{').count() == 1
            {
                // `--prop: { decls };` custom-property rule blocks (postcss
                // re-parses these as a rule): print the token stream as a block.
                value::flush_value_comments(value_start, f);
                write_custom_property_block(value_text, f);
            } else {
                // Unparsable rule-ish value (e.g. missing `;` swallowed the
                // following declarations): keep the source verbatim.
                write!(f, text(value_text));
            }
            // The raw text includes any comments; drop them from the cursor.
            let _ = f.context().comments().take_before(value_end);
        } else {
            // Custom property values come back from raffia as a raw token
            // stream (per spec, `<declaration-value>` is any token soup), but
            // Prettier (postcss) value-parses them like any other declaration,
            // so `var(...)` etc. get the normal group/break layout.
            let reparsed = if prop.starts_with("--")
                && decl
                    .value
                    .iter()
                    .all(|v| matches!(v, raffia::ast::ComponentValue::TokenWithSpan(_)))
            {
                reparse_custom_property_value(decl, f)
            } else {
                None
            };
            let values: &[raffia::ast::ComponentValue<'a>] =
                reparsed.as_ref().map_or(&decl.value, |d| &d.value);
            // A custom property's `!important` lands in the REPARSED
            // declaration (the original token-soup decl has `important:
            // None`); remember its source offset so the tail printing below
            // doesn't drop it. The padded copy keeps original offsets.
            reparsed_important_start = reparsed
                .as_ref()
                .and_then(|d| d.important.as_ref())
                .map(|important| to_span(important.span()).start);

            let ctx = ValueContext {
                decl_prop: Some(prop_lower),
                // Prettier applies `removeLines` to `composes` values.
                no_break: prop_lower == "composes",
                // Prettier's printer counts a multi-line between's FULL
                // width, so the first trailing comment always wraps.
                tail_break: trimmed_between.contains('\n'),
                ..ValueContext::default()
            };

            // The `;` position bounds wrapped trailing comments — only when
            // such comments exist (keeps simple declarations on the
            // single-component fast path).
            let decl_end_pre = to_span(decl.span()).end;
            let semi = end_with_semicolon(decl_end_pre, f);
            let bound = if semi > decl_end_pre { semi - 1 } else { decl_end_pre };
            // A single interpolated component is exempt — its fill-chunk fit
            // ignores the line tail (see `is_single_sass_interpolation`).
            let has_tail = decl.important.is_none()
                && reparsed_important_start.is_none()
                && !value::is_single_sass_interpolation(values)
                && f.context().comments().iter_before(bound).any(|c| c.span.start >= value_end);
            let ctx = ValueContext { tail_bound: has_tail.then_some(bound), ..ctx };
            // `prop: <values> { decls }` — a trailing nested block hugs the
            // declaration; leading values are space-joined flat.
            if let Some(raffia::ast::ComponentValue::SassNestingDeclaration(nesting)) =
                values.last()
            {
                for v in &values[..values.len() - 1] {
                    value::write_component_value(v, ctx, f);
                    write!(f, space());
                }
                write_block(&nesting.block, f);
                return;
            }
            if between_breaks {
                // `indent([hardline, dedent(value)])` — the value's own indent
                // is cancelled so it sits exactly one level under the prop.
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write!(f, hard_line_break());
                    let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        value::write_declaration_value(values, ctx, f);
                    });
                    write!(f, oxc_formatter_core::builders::dedent(&inner));
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
    } else if let Some(start) = reparsed_important_start {
        value::flush_trailing_value_comments(start, f);
        write!(f, [space(), "!important"]);
    }
    // Comments between the value and the `;`. Note: the `;` position is the
    // flush bound, so a comment after `;` stays for the trailing-comment pass.
    let decl_end = to_span(decl.span()).end;
    let end = end_with_semicolon(decl_end, f);
    let bound = if end > decl_end { end - 1 } else { decl_end };
    if let Some(comment_end) = value::flush_trailing_value_comments(bound, f) {
        // Preserve a source gap between the last comment and the `;`.
        if end > decl_end && comment_end < end - 1 {
            write!(f, space());
        }
    }
}

/// Re-parse a custom property's raw token-stream value as a normal
/// declaration so the value gets the standard group/break layout.
///
/// The declaration is rebuilt at the same source offsets — the prefix is
/// blanked out and the `--name` prop replaced by a same-length plain ident —
/// so every span in the re-parsed value stays valid against the original
/// source (Prettier pulls the same offset-preserving trick for
/// custom-property rule blocks in `parser-postcss.js`).
///
/// Returns `None` (caller keeps the token stream) when the value does not
/// parse as a plain declaration value.
fn reparse_custom_property_value<'a>(
    decl: &Declaration<'a>,
    f: &CssFormatter<'_, 'a>,
) -> Option<Declaration<'a>> {
    let source = f.context().source_text();
    let decl_span = to_span(decl.span());
    let name_span = to_span(decl.name.span());

    let mut padded = String::with_capacity(decl_span.end as usize);
    for c in source.slice_range(0, name_span.start).chars() {
        if c == '\n' || c == '\r' {
            padded.push(c);
        } else {
            // One space per BYTE (not per char): spans are byte offsets, so a
            // multi-byte char (e.g. `º` in a comment) must keep its width.
            for _ in 0..c.len_utf8() {
                padded.push(' ');
            }
        }
    }
    for _ in name_span.start..name_span.end {
        padded.push('a');
    }
    padded.push_str(source.slice_range(name_span.end, decl_span.end));

    let allocator = f.allocator();
    let padded: &'a str = allocator.alloc_str(&padded);
    let syntax = f.options().variant.to_raffia();
    let mut parser = raffia::ParserBuilder::new(padded).syntax(syntax).build();
    let reparsed = parser.parse::<Declaration>().ok()?;
    if !parser.recoverable_errors().is_empty() {
        return None;
    }
    Some(reparsed)
}

/// Prints a `--prop: { a: b; c: d }` rule-block value by re-flowing the raw
/// text: one declaration per line, normalized `prop: value;` spacing.
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
            // Re-flow line by line (keeps interior comments in place).
            for (j, line) in decl.lines().map(str::trim).filter(|l| !l.is_empty()).enumerate() {
                if j > 0 {
                    write!(f, hard_line_break());
                }
                // Normalize `prop : value` spacing on declaration lines:
                // split at the first colon OUTSIDE comments, keep the prop
                // side verbatim, re-space the value side tokens.
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
            // A trailing comment-only segment gets no semicolon.
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
pub fn write_block<'a>(block: &SimpleBlock<'a>, f: &mut CssFormatter<'_, 'a>) {
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
        // Dangling comments inside an otherwise empty block.
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
            crate::print::write_statement_sequence_bounded(&block.statements, r_curly, f);
            let last_end = block.statements.last().map_or(block_span.start, |s| stmt_end(s, f));
            flush_trailing_inside_comments(last_end, r_curly, f);
        });
        write!(f, block_indent(&body));
    }

    write!(f, "}");
}
