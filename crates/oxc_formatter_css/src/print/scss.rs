//! SCSS-specific printing: variable declarations, maps, lists,
//! control directives, mixins/includes/functions, module system.

use oxc_css_parser::ast::{
    ComponentValue, InterpolableStr, SassEach, SassFor, SassForBoundaryKind, SassForward,
    SassForwardVisibilityModifierKind, SassFunction, SassIfAtRule, SassInclude, SassList, SassMap,
    SassMixin, SassModuleConfig, SassParameters, SassUnaryOperatorKind, SassUse,
    SassUseNamespaceKind, SassVariableDeclaration,
};
use oxc_formatter_core::{
    Buffer,
    builders::{
        dedent, empty_line, expand_parent, group, hard_line_break, if_group_breaks, indent,
        line_suffix, soft_line_break, soft_line_break_or_space, space, text,
    },
    write,
};
use oxc_span::Span;

use crate::{
    comments,
    format::to_span,
    print::{
        CssFormatter, format_with, statement,
        value::{self, ValueContext},
    },
};

/// `$var: value !flags;`
pub(super) fn write_sass_variable_declaration<'a>(
    decl: &SassVariableDeclaration<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    if let Some(namespace) = &decl.namespace {
        let span = to_span(namespace.span());
        write!(f, [text(source.text_for(&span)), "."]);
    }
    write!(f, "$");
    let name_span = to_span(decl.name.name.span());
    write!(f, text(source.text_for(&name_span)));
    // Comments between the name and the colon are kept verbatim
    let colon_end = to_span(&decl.colon_span).end;
    let between = source.slice_range(name_span.end, colon_end);
    if between.trim() == ":" {
        write!(f, ":");
    } else {
        write!(f, text(between.trim_ascii()));
        let _ = f.context().comments().take_before(colon_end);
    }
    write!(f, space());

    let ctx = ValueContext { decl_prop: Some("$"), map_break: true, ..ValueContext::default() };
    // Comments between the colon and the value:
    // inline ones get their own line under the colon (`$x:\n  // c\n  value`).
    let value_start = to_span(decl.value.span()).start;
    if f.context().comments().peek().is_some_and(|c| c.inline && c.span.end <= value_start) {
        let lead = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            for &comment in f.context().comments().take_before(value_start) {
                write!(f, hard_line_break());
                comments::write_single_comment(comment, f);
            }
        });
        write!(f, indent(&lead));
    } else {
        value::flush_value_comments(value_start, f);
    }
    write_top_level_value(&decl.value, ctx, f);

    for flag in &decl.flags {
        let span = to_span(flag.span());
        value::flush_trailing_value_comments(span.start, f);
        write!(f, space());
        write!(f, text(source.text_for(&span)));
    }
    // Comments between the value/flags and the `;`.
    let decl_end = to_span(decl.span()).end;
    let end = statement::end_with_semicolon(decl_end, f);
    let bound = if end > decl_end { end - 1 } else { decl_end };
    if let Some(comment_end) = value::flush_trailing_value_comments(bound, f)
        && end > decl_end
        && comment_end < end - 1
    {
        write!(f, space());
    }
}

/// A single `ComponentValue` in declaration-value position:
/// comma-separated `SassList`s get Prettier's top-level list layout
/// (one entry per line when any entry has multiple parts).
pub(super) fn write_top_level_value<'a>(
    value: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let (elements, is_comma) = match value {
        ComponentValue::SassList(list) => (&list.elements, list.comma_spans.is_some()),
        ComponentValue::LessList(list) => (&list.elements, list.comma_spans.is_some()),
        _ => {
            value::write_component_value(value, ctx, f);
            return;
        }
    };
    // `paren_break` only applies to a paren group that IS the whole value,
    // not to parens nested inside lists.
    let ctx = ValueContext { paren_break: false, ..ctx };
    if is_comma {
        let groups: Vec<&[ComponentValue<'a>]> = elements
            .iter()
            .map(|el| match el {
                ComponentValue::SassList(inner) if inner.comma_spans.is_none() => {
                    &inner.elements[..]
                }
                ComponentValue::LessList(inner) if inner.comma_spans.is_none() => {
                    &inner.elements[..]
                }
                other => std::slice::from_ref(other),
            })
            .collect();
        let value_span = to_span(value.span());
        let has_comments = f
            .context()
            .comments()
            .iter_before(value_span.end)
            .any(|c| c.span.start >= value_span.start);
        let force_hard_line = !ctx.decl_prop.is_some_and(|p| p.starts_with("--"))
            && (groups.iter().enumerate().any(|(i, g)| value::comma_group_is_multi(g, i == 0))
                || has_comments);
        value::write_value_groups(&groups, ctx, force_hard_line, true, f);
    } else {
        value::write_comma_group(elements, ctx, f);
    }
}

/// `(key: value, ...)`: SCSS maps in map-item positions always break,
/// one item per line, with a trailing comma per the `trailingComma` option.
pub(super) fn write_sass_map<'a>(
    map: &SassMap<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    if map.items.is_empty() {
        // A map with no items may still hold comments (`(\n  // c\n)`).
        // Keep them inside the parens instead of leaking them past `)` as a trailing declaration comment.
        // Block comments stay inline when they fit (`$map: (/* c */);`);
        // a `//` comment glues to the current line but forces the `)` onto its own line,
        // like Prettier's `lineSuffix` + `lineSuffixBoundary` (Prettier #18535).
        let r_paren = to_span(map.span()).end.saturating_sub(1);
        let tail: Vec<comments::CssComment> = f.context().comments().take_before(r_paren).to_vec();
        if tail.is_empty() {
            write!(f, ["(", ")"]);
            return;
        }
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, soft_line_break());
            for (i, &comment) in tail.iter().enumerate() {
                if i > 0 {
                    if tail[i - 1].inline {
                        // Prettier leaves a stray leading space here
                        // (`   // b`, the `join(line)` separator prints before the deferred `lineSuffix` flushes);
                        write!(f, hard_line_break());
                    } else {
                        write!(f, space());
                    }
                }
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
        return;
    }
    // Maps break only in "map item" positions
    // (`$var:` values, map item values, function arguments, Prettier's `isSCSSMapItemNode`).
    // In key position or elsewhere (e.g. `@each ... in (k: v)`) they stay inline.
    if ctx.map_key || !ctx.map_break {
        let source = f.context().source_text();
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, soft_line_break());
            for (i, item) in map.items.iter().enumerate() {
                if i > 0 {
                    write!(f, ",");
                    // A blank line between items in the source is preserved
                    // (Prettier's isNextLineEmpty → hardline).
                    let prev_end = to_span(map.items[i - 1].span()).end;
                    let start = to_span(item.span()).start;
                    if comments::classify_gap(source.bytes_range(prev_end, start))
                        == comments::Gap::Blank
                    {
                        write!(f, empty_line());
                    } else {
                        write!(f, soft_line_break_or_space());
                    }
                }
                // `key: value` may break after the colon when too long
                let pair = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    let mut filler = f.fill();
                    let key = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        value::write_component_value(&item.key, ctx, f);
                        write!(f, ":");
                    });
                    let val = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        value::write_component_value(&item.value, ctx, f);
                    });
                    filler.entry(&soft_line_break_or_space(), &key);
                    filler.entry(&soft_line_break_or_space(), &val);
                    filler.finish();
                });
                write!(f, group(&indent(&pair)));
            }
            // Outside map-item positions (e.g. `@each ... in (k: v)`),
            // `isSCSSMapItemNode` is false → no trailing comma.
            if ctx.map_key && f.options().allow_trailing_comma() {
                write!(f, if_group_breaks(&text(",")));
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
        return;
    }
    let trailing = f.options().allow_trailing_comma();
    let r_paren = to_span(map.span()).end.saturating_sub(1);
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(f, hard_line_break());
        let source = f.context().source_text();
        let mut last_item_block_with_comment = false;
        let mut first_item_has_leading_comment = false;
        for (i, item) in map.items.iter().enumerate() {
            if i > 0 {
                write!(f, ",");
                write!(f, hard_line_break());
                // Preserve one blank line between items
                let prev_end = to_span(map.items[i - 1].span()).end;
                let item_start = to_span(item.span()).start;
                let next_start = f
                    .context()
                    .comments()
                    .peek()
                    .map_or(item_start, |c| c.span.start.min(item_start));
                if comments::classify_gap(source.bytes_range(prev_end, next_start))
                    == comments::Gap::Blank
                {
                    write!(f, empty_line());
                }
            }
            let key_ctx =
                ValueContext { map_key: true, paren_break: false, map_break: false, ..ctx };
            let val_ctx =
                ValueContext { map_key: false, paren_break: true, map_break: true, ..ctx };
            // Nested maps / paren lists hug the colon (`key: (`);
            // the pair never breaks after the colon (Prettier dedents these).
            let value_is_block = matches!(
                item.value,
                ComponentValue::SassMap(_) | ComponentValue::SassParenthesizedExpression(_)
            );

            // Comments between items:
            // block comments join the item when the pair fits on one line (Prettier's fill);
            // `//` comments and pairs that don't fit keep their own line.
            let item_start = to_span(item.span()).start;
            let item_width = to_span(item.span()).end - item_start;
            let mut had_leading_comment = false;
            for &comment in f.context().comments().take_before(item_start) {
                had_leading_comment = true;
                let comment_width = comment.span.end - comment.span.start;
                let fits = !value_is_block
                    && u32::from(f.options().indent_width.value()) + comment_width + 2 + item_width
                        <= u32::from(f.options().line_width.value());
                comments::write_single_comment(comment, f);
                if comment.inline || !fits {
                    write!(f, hard_line_break());
                } else {
                    write!(f, " ");
                }
            }
            if i == 0 {
                first_item_has_leading_comment = had_leading_comment;
            }
            let key_is_block = matches!(
                item.key,
                ComponentValue::SassMap(_) | ComponentValue::SassParenthesizedExpression(_)
            );
            if i + 1 == map.items.len() {
                // Suppressed only when the source ALSO had a trailing comma
                // (the comment lands inside the last comma_group in postcss).
                last_item_block_with_comment = value_is_block
                    && had_leading_comment
                    && map.comma_spans.len() >= map.items.len();
            }
            if key_is_block && !value_is_block {
                // Block keys never break before their value (`): "v",`).
                value::write_component_value(&item.key, key_ctx, f);
                write!(f, [":", space()]);
                write_top_level_value(&item.value, val_ctx, f);
            } else if value_is_block {
                value::write_component_value(&item.key, key_ctx, f);
                write!(f, [":", space()]);
                // A paren/map KEY (or a leading comment, or nesting) keeps the pair's indent on the value
                // (Prettier's dedent applies only when the doc is a plain `group(indent(fill))`).
                let needs_indent =
                    key_is_block || had_leading_comment || f.context().block_depth().get() > 0;
                if needs_indent {
                    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        write_top_level_value(&item.value, val_ctx, f);
                    });
                    write!(f, indent(&body));
                } else {
                    write_top_level_value(&item.value, val_ctx, f);
                }
            } else {
                // `key: value` breaks after the colon when too long
                let pair = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    let mut filler = f.fill();
                    let key = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        // Paren/map keys cancel the pair indent (Prettier's `isKey` → dedent)
                        if matches!(
                            item.key,
                            ComponentValue::SassMap(_)
                                | ComponentValue::SassParenthesizedExpression(_)
                        ) {
                            let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                                value::write_component_value(&item.key, key_ctx, f);
                            });
                            write!(f, dedent(&inner));
                        } else {
                            value::write_component_value(&item.key, key_ctx, f);
                        }
                        write!(f, ":");
                    });
                    let val = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        write_top_level_value(&item.value, val_ctx, f);
                    });
                    filler.entry(&soft_line_break_or_space(), &key);
                    filler.entry(&soft_line_break_or_space(), &val);
                    filler.finish();
                });
                write!(f, group(&indent(&pair)));
            }
        }
        // Own-line comments after the last item make it "non-last" in Prettier's sequence,
        // so it always gets a comma.
        let last_item_end = map.items.last().map_or(0, |it| to_span(it.span()).end);
        let has_ownline_tail = f
            .context()
            .comments()
            .iter_before(r_paren)
            .any(|c| c.span.start >= last_item_end && value::comment_is_own_line(c, source));
        // Prettier drops the trailing comma after a comment-preceded block value
        // (the pair doc isn't the plain dedent shape).
        // A comment before the FIRST item also drops it:
        // the comment becomes `groups[0]` of the paren group,
        // so `isKeyValuePairInParenGroupNode` no longer sees a key-value pair
        // and the group stops being an SCSS map item.
        let printed_comma =
            (trailing && !last_item_block_with_comment && !first_item_has_leading_comment)
                || has_ownline_tail;
        if printed_comma {
            write!(f, ",");
        }
        // The comment goes to its own line only when BOTH a comma is printed
        // and the source comma preceded the comment (true next-slot comment).
        if let Some(last) = map.items.last() {
            let source_comma_start =
                map.comma_spans.get(map.items.len() - 1).map_or(u32::MAX, |sp| to_span(sp).start);
            // Only inside function/include arguments does the comment move
            // to the next slot; `$map:` declarations keep it attached.
            let next_slot = printed_comma
                && ctx.in_args
                && f.context().comments().peek().is_some_and(|c| c.span.start > source_comma_start);
            if !next_slot {
                value::flush_same_line_comments(to_span(last.span()).end, r_paren, f);
            }
        }
        // Own-line comments before `)`: same-line runs stay glued
        let tail: Vec<comments::CssComment> = f.context().comments().take_before(r_paren).to_vec();
        for (i, &comment) in tail.iter().enumerate() {
            if i == 0 || comment.inline || tail[i - 1].inline {
                write!(f, hard_line_break());
            } else {
                // Block comments are fill items: they join when they fit
                write!(f, " ");
            }
            comments::write_single_comment(comment, f);
        }
    });
    write!(f, ["(", indent(&body), hard_line_break(), ")"]);
}

/// Space- or comma-separated SCSS list in a nested position.
pub(super) fn write_sass_list<'a>(
    list: &SassList<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    if list.comma_spans.is_some() {
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let mut filler = f.fill();
            for (i, el) in list.elements.iter().enumerate() {
                let is_last = i + 1 == list.elements.len();
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    value::write_component_value(el, ctx, f);
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
        value::write_comma_group(&list.elements, ctx, f);
    }
}

/// `@each $key, $value in $expr`: printed as one flat comma list
/// (`$k, $v in (a), (b), (c)`), filling and indenting like a value.
pub(super) fn write_sass_each<'a>(each: &SassEach<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let in_span = to_span(&each.in_span);
    let in_tight = in_span.end == to_span(each.expr.span()).start;

    // Comma-list expr: the first element joins the `... in` entry,
    // the rest are separate fill entries (mirrors postcss's flat comma groups).
    let expr_elements: &[ComponentValue<'a>] = match &each.expr {
        ComponentValue::SassList(list) if list.comma_spans.is_some() => &list.elements,
        expr => std::slice::from_ref(expr),
    };

    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let mut filler = f.fill();
        let last_binding = each.bindings.len() - 1;
        for (i, binding) in each.bindings.iter().enumerate() {
            let span = to_span(binding.span());
            let is_last = i == last_binding;
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                if is_last {
                    // `$binding in expr`: breakable before `in`,
                    // with the continuation indented one level deeper.
                    let tail = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        let mut inner = f.fill();
                        let binding = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                            write!(f, text(source.text_for(&span)));
                        });
                        let in_expr = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                            write!(f, "in");
                            if !in_tight {
                                write!(f, " ");
                            }
                            value::write_component_value(
                                &expr_elements[0],
                                ValueContext::default(),
                                f,
                            );
                            if expr_elements.len() > 1 {
                                write!(f, ",");
                            }
                        });
                        inner.entry(&soft_line_break_or_space(), &binding);
                        if span.end == in_span.start {
                            // `in` fused to the binding in the source
                            inner.entry(&format_with(|_| {}), &in_expr);
                        } else {
                            inner.entry(&soft_line_break_or_space(), &in_expr);
                        }
                        inner.finish();
                    });
                    write!(f, group(&indent(&tail)));
                } else {
                    write!(f, text(source.text_for(&span)));
                    write!(f, ",");
                }
            });
            filler.entry(&soft_line_break_or_space(), &content);
        }
        for (i, el) in expr_elements.iter().enumerate().skip(1) {
            let is_last = i + 1 == expr_elements.len();
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                value::write_component_value(el, ValueContext::default(), f);
                if !is_last {
                    write!(f, ",");
                }
            });
            filler.entry(&soft_line_break_or_space(), &content);
        }
        filler.finish();
    });
    write!(f, group(&indent(&body)));
}

/// `@for $i from <start> to|through <end>`
pub(super) fn write_sass_for<'a>(sass_for: &SassFor<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let binding_span = to_span(sass_for.binding.span());
    write!(f, text(source.text_for(&binding_span)));
    write!(f, [space(), "from", space()]);
    value::write_component_value(&sass_for.start, ValueContext::default(), f);
    match sass_for.boundary.kind {
        SassForBoundaryKind::Inclusive => write!(f, [space(), "through", space()]),
        SassForBoundaryKind::Exclusive => write!(f, [space(), "to", space()]),
    }
    value::write_component_value(&sass_for.end, ValueContext::default(), f);
}

/// `@mixin name($params...)`
pub(super) fn write_sass_mixin<'a>(mixin: &SassMixin<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(mixin.name.span());
    write!(f, text(source.text_for(&name_span)));
    if let Some(parameters) = &mixin.parameters {
        write_sass_parameters(parameters, f);
    }
}

/// `@function name($params...)`
pub(super) fn write_sass_function<'a>(function: &SassFunction<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(function.name.span());
    write!(f, text(source.text_for(&name_span)));
    write_sass_parameters(&function.parameters, f);
}

fn write_sass_parameters<'a>(parameters: &SassParameters<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(f, soft_line_break());
        let mut first = true;
        for param in &parameters.params {
            if !first {
                write!(f, ",");
                write!(f, soft_line_break_or_space());
            }
            first = false;
            let name_span = to_span(param.name.span());
            write!(f, text(source.text_for(&name_span)));
            if let Some(default) = &param.default_value {
                write!(f, [":", space()]);
                value::write_component_value(&default.value, ValueContext::default(), f);
            }
        }
        if let Some(arbitrary) = &parameters.arbitrary_param {
            if !first {
                write!(f, ",");
                write!(f, soft_line_break_or_space());
            }
            let span = to_span(arbitrary.name.span());
            write!(f, [text(source.text_for(&span)), "..."]);
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

/// `@include name(args...) [using (params)]`
pub(super) fn write_sass_include<'a>(include: &SassInclude<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    let name_span = to_span(include.name.span());
    write!(f, text(source.text_for(&name_span)));
    if let Some(arguments) = &include.arguments {
        let args = &arguments.args;
        // Same first-argument gate as `write_function` (which see), over typed args
        let first_arg_is_kw =
            args.first().is_some_and(|a| matches!(a, ComponentValue::SassKeywordArgument(_)));
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let source = f.context().source_text();
            write!(f, soft_line_break());
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(f, ",");
                    // Preserve a blank line, but only after a multi-part argument
                    // (Prettier checks `value-comma_group`s only).
                    let prev = &args[i - 1];
                    let prev_is_group = matches!(
                        prev,
                        ComponentValue::SassKeywordArgument(_) | ComponentValue::SassList(_)
                    );
                    let prev_end = to_span(prev.span()).end;
                    let start = to_span(arg.span()).start;
                    if prev_is_group
                        && comments::classify_gap(source.bytes_range(prev_end, start))
                            == comments::Gap::Blank
                    {
                        write!(f, empty_line());
                    } else {
                        write!(f, soft_line_break_or_space());
                    }
                }
                let arg_ctx = ValueContext {
                    map_break: true,
                    in_args: true,
                    paren_break: first_arg_is_kw
                        && matches!(arg, ComponentValue::SassKeywordArgument(_)),
                    ..ValueContext::default()
                };
                write_top_level_value(arg, arg_ctx, f);
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
    if let Some(content_params) = &include.content_block_params {
        write!(f, [space(), "using", space()]);
        write_sass_parameters(&content_params.params, f);
    }
}

/// `@if cond { } @else if cond { } @else { }`
pub(super) fn write_sass_if_at_rule<'a>(if_rule: &SassIfAtRule<'a>, f: &mut CssFormatter<'_, 'a>) {
    write!(f, ["@if", space()]);
    write_control_condition(&if_rule.if_clause.condition, f);
    statement::write_block(&if_rule.if_clause.block, f);
    for clause in &if_rule.else_if_clauses {
        // Comments between `}` and `@else` break the join
        let cond_start = to_span(clause.condition.span()).start;
        let mut broke_join = false;
        while let Some(comment) = f.context().comments().peek() {
            if comment.span.end > cond_start {
                break;
            }
            f.context().comments().take_before(comment.span.end);
            write!(f, hard_line_break());
            comments::write_single_comment(comment, f);
            broke_join = true;
        }
        if broke_join {
            write!(f, hard_line_break());
            write!(f, ["@else", space()]);
        } else {
            write!(f, [space(), "@else", space()]);
        }
        // `if` is a value word in postcss, so the condition may break after it
        if matches!(clause.condition, ComponentValue::SassParenthesizedExpression(_)) {
            write!(f, ["if", space()]);
            write_control_condition(&clause.condition, f);
        } else {
            write_condition_chain(Some("if"), &clause.condition, f);
        }
        statement::write_block(&clause.block, f);
    }
    if let Some(else_block) = &if_rule.else_clause {
        write!(f, [space(), "@else", space()]);
        statement::write_block(else_block, f);
    }
}

/// Control-directive condition followed by a breakable gap before `{`
/// (Prettier wraps the value + line in a group, without extra indent).
/// A fully parenthesized condition keeps `{` on the `)` line.
fn write_control_condition<'a>(condition: &ComponentValue<'a>, f: &mut CssFormatter<'_, 'a>) {
    if matches!(condition, ComponentValue::SassParenthesizedExpression(_)) {
        value::write_component_value(condition, ValueContext::default(), f);
        write!(f, space());
        return;
    }
    if matches!(condition, ComponentValue::SassBinaryExpression(_)) {
        write_condition_chain(None, condition, f);
        return;
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        value::write_component_value(condition, ValueContext::default(), f);
        write!(f, soft_line_break_or_space());
    });
    write!(f, group(&body));
}

/// Operand-or-operator part of a flattened control-directive condition.
enum CondPart<'b, 'a> {
    Value(&'b ComponentValue<'a>),
    /// Operator/keyword raw text (`and`, `or`, `not`, `==`, `*`, ...).
    Op(Span),
}

fn flatten_condition<'b, 'a>(cond: &'b ComponentValue<'a>, out: &mut Vec<CondPart<'b, 'a>>) {
    match cond {
        ComponentValue::SassBinaryExpression(binary) => {
            flatten_condition(&binary.left, out);
            out.push(CondPart::Op(to_span(&binary.op.span)));
            flatten_condition(&binary.right, out);
        }
        ComponentValue::SassUnaryExpression(unary)
            if matches!(unary.op.kind, SassUnaryOperatorKind::Not) =>
        {
            out.push(CondPart::Op(to_span(&unary.op.span)));
            flatten_condition(&unary.expr, out);
        }
        other => out.push(CondPart::Value(other)),
    }
}

/// Prettier's control-directive condition layout (`group(indent(parts))`, NOT a fill):
/// a space before every operator/keyword,
/// a breakable line after it — breaking is all-or-nothing.
fn write_condition_chain<'a>(
    prefix: Option<&'static str>,
    condition: &ComponentValue<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let mut parts = Vec::new();
    flatten_condition(condition, &mut parts);
    let source = f.context().source_text();
    let parts_ref = &parts;
    let inner = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        if let Some(word) = prefix {
            write!(f, text(word));
            // Separator to the first part: space when it's an operator
            if let Some(CondPart::Op(_)) = parts_ref.first() {
                write!(f, space());
            } else {
                write!(f, soft_line_break_or_space());
            }
        }
        let part_span = |p: &CondPart<'_, 'a>| match p {
            CondPart::Op(span) => *span,
            CondPart::Value(v) => to_span(v.span()),
        };
        for (i, part) in parts_ref.iter().enumerate() {
            if i > 0 {
                // Tokens glued in the source stay glued (postcss parses
                // `$type==ocean` as ONE word and prints it verbatim).
                let glued = part_span(&parts_ref[i - 1]).end == part_span(part).start;
                if !glued {
                    match part {
                        CondPart::Op(_) => write!(f, space()),
                        CondPart::Value(_) => write!(f, soft_line_break_or_space()),
                    }
                }
            }
            match part {
                CondPart::Op(span) => write!(f, text(source.text_for(span))),
                CondPart::Value(v) => {
                    value::write_component_value(v, ValueContext::default(), f);
                }
            }
        }
    });
    write!(
        f,
        group(&format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, indent(&inner));
            write!(f, soft_line_break_or_space());
        }))
    );
}

/// `@use "path" as ns with (...)` / `@forward "path" as p-* show a, b with (...)`
pub(super) fn write_sass_use<'a>(sass_use: &SassUse<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    if let InterpolableStr::Literal(str) = &sass_use.path {
        value::write_str(str, f);
    } else {
        let span = to_span(sass_use.path.span());
        write!(f, text(source.text_for(&span)));
    }
    if let Some(namespace) = &sass_use.namespace {
        write!(f, [space(), "as", space()]);
        match &namespace.kind {
            SassUseNamespaceKind::Named(ident) => {
                let span = to_span(ident.span());
                write!(f, text(source.text_for(&span)));
            }
            SassUseNamespaceKind::Unnamed(_) => write!(f, "*"),
        }
    }
    if let Some(config) = &sass_use.config {
        write_sass_module_config(config, f);
    }
}

pub(super) fn write_sass_forward<'a>(forward: &SassForward<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    if let InterpolableStr::Literal(str) = &forward.path {
        value::write_str(str, f);
    } else {
        let span = to_span(forward.path.span());
        write!(f, text(source.text_for(&span)));
    }
    if let Some(prefix) = &forward.prefix {
        let span = to_span(prefix.name.span());
        write!(f, [space(), "as", space(), text(source.text_for(&span)), "*"]);
    }
    if let Some(visibility) = &forward.visibility {
        match visibility.modifier.kind {
            SassForwardVisibilityModifierKind::Show => {
                write!(f, [space(), "show", space()]);
            }
            SassForwardVisibilityModifierKind::Hide => {
                write!(f, [space(), "hide", space()]);
            }
        }
        for (i, member) in visibility.members.iter().enumerate() {
            if i > 0 {
                write!(f, [",", space()]);
            }
            let span = to_span(member.span());
            write!(f, text(source.text_for(&span)));
        }
    }
    if let Some(config) = &forward.config {
        write_sass_module_config(config, f);
    }
}

/// `with ($var: value, ...)`:
/// configurations always break, one item per line, without a trailing comma.
///
/// Comments follow Prettier's comma-group handling:
/// - a leading `//` comment sits on its own line
/// - a leading block comment glues to its item
/// - a same-line trailing comment stays at the end of the item's line
/// - and a blank line after an item's comma is preserved
///
/// EXCEPT an own-line trailing comment, which keeps its own line
/// (consistent with the map printer; Prettier pulls it up — see "Known divergences").
fn write_sass_module_config<'a>(config: &SassModuleConfig<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    // Comments between the module path and `with` stay glued to the head
    // (`@use "a" /* c */ with (`).
    write_config_trailing_comments(to_span(&config.with_span).start, f);
    write!(f, [space(), "with", space(), "("]);
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(f, hard_line_break());
        for (i, item) in config.items.iter().enumerate() {
            let item_span = to_span(&item.span);
            if i > 0 {
                // Prettier's `isNextLineEmpty` after an item:
                // a blank line after the comma survives, measured up to the next item or its first leading comment.
                let comma_end = to_span(&config.comma_spans[i - 1]).end;
                // Clamped: an own-line comment left pending by the previous
                // item's trailing flush can start BEFORE the comma.
                let next_start = f
                    .context()
                    .comments()
                    .iter_before(item_span.start)
                    .next()
                    .map_or(item_span.start, |c| c.span.start)
                    .max(comma_end);
                if comments::classify_gap(source.bytes_range(comma_end, next_start))
                    == comments::Gap::Blank
                {
                    write!(f, empty_line());
                } else {
                    write!(f, hard_line_break());
                }
            }
            // Leading comments: `//` on its own line, block glued inline
            value::flush_value_comments(item_span.start, f);
            let span = to_span(item.variable.span());
            write!(f, [text(source.text_for(&span)), ":", space()]);
            // No first-argument gate here (cf. `write_function`):
            // config items are structurally always `$var: value` pairs, so the gate holds by construction.
            let item_ctx =
                ValueContext { paren_break: true, map_break: true, ..ValueContext::default() };
            write_top_level_value(&item.value, item_ctx, f);
            for flag in &item.flags {
                let span = to_span(flag.span());
                write!(f, space());
                write!(f, text(source.text_for(&span)));
            }
            if i + 1 < config.items.len() {
                // An own-line comment before the comma stays pending and
                // leads the next item instead.
                write_config_trailing_comments(to_span(&config.comma_spans[i]).start, f);
                write!(f, ",");
            } else {
                // Comments before `)` (past a trailing comma, which is dropped):
                // same-line ones glue to the last item, own-line ones keep their line.
                let bound = to_span(&config.span).end;
                let mut prev_end =
                    write_config_trailing_comments(bound, f).unwrap_or(item_span.end);
                for &comment in f.context().comments().take_before(bound) {
                    comments::write_gap(source.bytes_range(prev_end, comment.span.start), f);
                    comments::write_single_comment(comment, f);
                    prev_end = comment.span.end;
                }
            }
        }
    });
    write!(f, [indent(&body), hard_line_break(), ")"]);
}

/// Emits pending SAME-LINE comments before `upper_bound` glued to the just-printed
/// content (`$a: 1 /* c */,`); an own-line comment stops the loop and stays pending.
/// Inline `//` comments ride a `line_suffix`,
/// so a following `,` lands before the comment text (Prettier prints `$a: 1, // c`).
/// Returns the end offset of the last emitted comment.
fn write_config_trailing_comments<'a>(
    upper_bound: u32,
    f: &mut CssFormatter<'_, 'a>,
) -> Option<u32> {
    let mut last_end = None;
    while let Some(comment) = f.context().comments().peek() {
        if comment.span.end > upper_bound
            || value::comment_is_own_line(comment, f.context().source_text())
        {
            break;
        }
        f.context().comments().take_before(comment.span.end);
        if comment.inline {
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                write!(f, space());
                comments::write_single_comment(comment, f);
            });
            write!(f, line_suffix(&content));
        } else {
            write!(f, space());
            comments::write_single_comment(comment, f);
        }
        last_end = Some(comment.span.end);
    }
    last_end
}
