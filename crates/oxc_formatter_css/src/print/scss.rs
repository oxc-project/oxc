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
        dedent, empty_line, group, hard_line_break, if_group_breaks, indent, soft_line_break,
        soft_line_break_or_space, space, text,
    },
    write,
};
use oxc_span::Span;

use crate::{
    comments::{self, BlockCommentAfter, FormatCommentBeforeContent, FormatLineCommentSuffix},
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
    // Comments between the colon and the value keep their line:
    // a same-line `//` stays on the colon's (`$x: // c`), an own-line one stays own-line,
    // and the value then continues one level under the name.
    // A hard-broken comma list indents itself and claims an own-line lead inside that indent.
    let value_start = to_span(decl.value.span()).start;
    let inline_lead = f.context().comments().iter_before(value_start).any(|c| c.inline);
    let own_line_lead = f
        .context()
        .comments()
        .iter_before(value_start)
        .next()
        .is_some_and(|c| value::comment_is_own_line(c, source));
    let hard_list = top_level_value_breaks_hard(&decl.value, ctx, f);
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        if own_line_lead {
            write!(f, hard_line_break());
        }
        if hard_list && own_line_lead {
            write_top_level_value(&decl.value, ctx, f);
        } else {
            write_top_level_list_element(&decl.value, ctx, f);
        }
    });
    if (inline_lead || own_line_lead) && !hard_list {
        write!(f, indent(&body));
    } else {
        write!(f, body);
    }

    for flag in &decl.flags {
        let span = to_span(flag.span());
        value::flush_trailing_value_comments(span.start, f);
        write!(f, space());
        write!(f, text(source.text_for(&span)));
    }
    statement::write_terminator_tail_comments(to_span(decl.span()).end, f);
}

/// `("a",)` / `$x: "a",`: a single element followed by a comma is a one-element LIST in Sass,
/// and `"a"` alone is a string (dart-sass `type-of`), so the comma is the value and is kept regardless of `trailing_commas`.
/// Multi-element lists drop theirs (the list is a list without it).
/// `comma_spans` holds one span per source comma, so a lone element with a span means a trailing comma.
/// Not a concern for CSS declarations (dart-sass serializes both as `"a"`) nor Less (no list/scalar split).
pub(super) fn is_single_item_list(list: &SassList<'_>) -> bool {
    list.elements.len() == 1 && list.comma_spans.as_ref().is_some_and(|s| !s.is_empty())
}

/// Start offset of the source comma after element `i` (for [`value::write_group_comma`]).
pub(super) fn list_comma_start(list: &SassList<'_>, i: usize) -> Option<u32> {
    list.comma_spans.as_ref().and_then(|s| s.get(i)).map(|sp| to_span(sp).start)
}

/// Whether [`write_top_level_value`] prints `value` one comma entry per line:
/// a comma list whose entries have multiple parts or carry comments, except under a custom property.
/// Such a list indents itself, callers must not add a level.
pub(super) fn top_level_value_breaks_hard<'a>(
    value: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &CssFormatter<'_, 'a>,
) -> bool {
    let elements = match value {
        ComponentValue::SassList(list) if list.comma_spans.is_some() => &list.elements,
        ComponentValue::LessList(list) if list.comma_spans.is_some() => &list.elements,
        _ => return false,
    };
    if elements.len() < 2 || ctx.decl_prop.is_some_and(|p| p.starts_with("--")) {
        return false;
    }
    let value_span = to_span(value.span());
    let has_comments = f
        .context()
        .comments()
        .iter_before(value_span.end)
        .any(|c| c.span.start >= value_span.start);
    has_comments
        || elements.iter().enumerate().any(|(i, el)| {
            let group = match el {
                ComponentValue::SassList(inner) if inner.comma_spans.is_none() => {
                    &inner.elements[..]
                }
                ComponentValue::LessList(inner) if inner.comma_spans.is_none() => {
                    &inner.elements[..]
                }
                other => std::slice::from_ref(other),
            };
            value::comma_group_is_multi(group, i == 0)
        })
}

/// A single `ComponentValue` in declaration-value position:
/// comma-separated `SassList`s get Prettier's top-level list layout
/// (one entry per line when any entry has multiple parts).
pub(super) fn write_top_level_value<'a>(
    value: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    let (elements, comma_spans, keep_trailing_comma) = match value {
        ComponentValue::SassList(list) => {
            (&list.elements, list.comma_spans.as_ref(), is_single_item_list(list))
        }
        ComponentValue::LessList(list) => (&list.elements, list.comma_spans.as_ref(), false),
        _ => {
            value::write_component_value(value, ctx, f);
            return;
        }
    };
    // `paren_break` only applies to a paren group that IS the whole value,
    // not to parens nested inside lists.
    let ctx = ValueContext { paren_break: false, ..ctx };
    if let Some(comma_spans) = comma_spans {
        // Each element paired with the comma that follows it (see `write_value_groups`)
        let groups: Vec<(&[ComponentValue<'a>], Option<u32>)> = elements
            .iter()
            .enumerate()
            .map(|(i, el)| {
                let group = match el {
                    ComponentValue::SassList(inner) if inner.comma_spans.is_none() => {
                        &inner.elements[..]
                    }
                    ComponentValue::LessList(inner) if inner.comma_spans.is_none() => {
                        &inner.elements[..]
                    }
                    other => std::slice::from_ref(other),
                };
                (group, comma_spans.get(i).map(|sp| to_span(sp).start))
            })
            .collect();
        let force_hard_line = top_level_value_breaks_hard(value, ctx, f);
        value::write_value_groups(&groups, ctx, force_hard_line, keep_trailing_comma, f);
    } else {
        value::write_comma_group(elements, ctx, f);
    }
}

/// [`write_top_level_value`] for a value that owns its leading comments
/// (`k: /* c */ v`, `@include m( // c\n 1)`), the [`value::write_list_element`] of this level.
pub(super) fn write_top_level_list_element<'a>(
    value: &ComponentValue<'a>,
    ctx: ValueContext<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    value::flush_value_comments(to_span(value.span()).start, f);
    write_top_level_value(value, ctx, f);
}

/// A paren-delimited map/list value or key: hugs the colon and carries
/// its own break layout (Prettier's `value-paren_group` checks).
fn is_paren_block(value: &ComponentValue<'_>) -> bool {
    matches!(value, ComponentValue::SassMap(_) | ComponentValue::SassParenthesizedExpression(_))
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
                if i > 0 && !tail[i - 1].inline {
                    write!(f, space());
                }
                write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
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
                    value::write_group_comma(
                        map.comma_spans.get(i - 1).map(|sp| to_span(sp).start),
                        f,
                    );
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
                        value::write_list_element(&item.key, ctx, f);
                        write!(f, ":");
                    });
                    let val = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        value::write_list_element(&item.value, ctx, f);
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
        for (i, item) in map.items.iter().enumerate() {
            if i > 0 {
                value::write_group_comma(map.comma_spans.get(i - 1).map(|sp| to_span(sp).start), f);
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
            let value_is_block = is_paren_block(&item.value);

            // Comments between items:
            // block comments join the item when the pair fits on one line (Prettier's fill);
            // `//` comments and pairs that don't fit keep their own line.
            let item_start = to_span(item.span()).start;
            let item_width = to_span(item.span()).end - item_start;
            for &comment in f.context().comments().take_before(item_start) {
                let comment_width = comment.span.end - comment.span.start;
                let fits = !value_is_block
                    && u32::from(f.options().indent_width.value()) + comment_width + 2 + item_width
                        <= u32::from(f.options().line_width.value());
                let block_after =
                    if fits { BlockCommentAfter::Space } else { BlockCommentAfter::HardLine };
                write!(f, FormatCommentBeforeContent::new(comment, block_after));
            }
            let key_is_block = is_paren_block(&item.key);
            if key_is_block && !value_is_block {
                // Block keys never break before their value (`): "v",`).
                value::write_component_value(&item.key, key_ctx, f);
                write!(f, [":", space()]);
                write_top_level_list_element(&item.value, val_ctx, f);
            } else if value_is_block {
                value::write_component_value(&item.key, key_ctx, f);
                write!(f, [":", space()]);
                let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    write_top_level_list_element(&item.value, val_ctx, f);
                });
                // Prettier's dedent applies only when the pair doc is a plain `group(indent(fill))`;
                // a paren/map KEY changes that shape, so it keeps the pair's indent on the value.
                // NOTE: two more dedent skips are deliberately NOT matched here (both known divergences):
                // - LEADING COMMENT (also changes the doc shape; trivia must not change layout)
                // - `@if`/`@each`/... ancestor (an explicit prettier#16607 crash-guard, not doc shape: SAME source, different indent per context)
                if key_is_block {
                    write!(f, indent(&body));
                } else {
                    write!(f, body);
                }
            } else {
                // `key: value` breaks after the colon when too long
                let pair = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    let mut filler = f.fill();
                    let key = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                        // Paren/map keys cancel the pair indent (Prettier's `isKey` → dedent)
                        if key_is_block {
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
                        write_top_level_list_element(&item.value, val_ctx, f);
                    });
                    filler.entry(&soft_line_break_or_space(), &key);
                    filler.entry(&soft_line_break_or_space(), &val);
                    filler.finish();
                });
                write!(f, group(&indent(&pair)));
            }
        }
        // NOTE: Comment presence never changes the comma (Prettier drops it after a leading comment on the FIRST item);
        // see DIVERGENCES.md "map-leading-comment-layout".
        if trailing {
            write!(f, ",");
        }
        // The comment goes to its own line only when BOTH a comma is printed
        // and the source comma preceded the comment (true next-slot comment).
        if let Some(last) = map.items.last() {
            let source_comma_start =
                map.comma_spans.get(map.items.len() - 1).map_or(u32::MAX, |sp| to_span(sp).start);
            // Only inside function/include arguments does the comment move
            // to the next slot; `$map:` declarations keep it attached.
            let next_slot = trailing
                && ctx.in_args
                && f.context().comments().peek().is_some_and(|c| c.span.start > source_comma_start);
            if !next_slot {
                value::flush_same_line_comments(to_span(last.span()).end, r_paren, f);
            }
        }
        // The rest goes below the last item, one line per `//`;
        // block comments after a block comment stay glued.
        // Not `write_paren_tail_comments`:
        // a next-slot comment (above) is same-line in source but still starts a line here,
        // and consecutive own-line block comments glue.
        let mut after_inline = false;
        for (i, &comment) in f.context().comments().take_before(r_paren).iter().enumerate() {
            // A `//` already ended its line
            if !after_inline {
                if i == 0 || comment.inline {
                    write!(f, hard_line_break());
                } else {
                    write!(f, " ");
                }
            }
            write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
            after_inline = comment.inline;
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
        let keep_trailing_comma = is_single_item_list(list);
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let mut filler = f.fill();
            for (i, el) in list.elements.iter().enumerate() {
                let is_last = i + 1 == list.elements.len();
                let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    value::write_component_value(el, ctx, f);
                    if !is_last || keep_trailing_comma {
                        value::write_group_comma(list_comma_start(list, i), f);
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
    let (expr_elements, expr_comma_spans): (&[ComponentValue<'a>], _) = match &each.expr {
        ComponentValue::SassList(list) if list.comma_spans.is_some() => {
            (&list.elements, list.comma_spans.as_ref())
        }
        expr => (std::slice::from_ref(expr), None),
    };
    let expr_comma_start =
        move |i: usize| expr_comma_spans.and_then(|s| s.get(i)).map(|sp| to_span(sp).start);

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
                            value::write_list_element(
                                &expr_elements[0],
                                ValueContext::default(),
                                f,
                            );
                            if expr_elements.len() > 1 {
                                value::write_group_comma(expr_comma_start(0), f);
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
                    value::write_group_comma(
                        each.comma_spans.get(i).map(|sp| to_span(sp).start),
                        f,
                    );
                }
            });
            filler.entry(&soft_line_break_or_space(), &content);
        }
        for (i, el) in expr_elements.iter().enumerate().skip(1) {
            let is_last = i + 1 == expr_elements.len();
            let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                value::write_list_element(el, ValueContext::default(), f);
                if !is_last {
                    value::write_group_comma(expr_comma_start(i), f);
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
    value::write_list_element(&sass_for.start, ValueContext::default(), f);
    match sass_for.boundary.kind {
        SassForBoundaryKind::Inclusive => write!(f, [space(), "through", space()]),
        SassForBoundaryKind::Exclusive => write!(f, [space(), "to", space()]),
    }
    value::write_list_element(&sass_for.end, ValueContext::default(), f);
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
    let comma_start =
        |i: usize| parameters.comma_spans.get(i).map(|sp: &oxc_css_parser::Span| to_span(sp).start);
    let r_paren = to_span(&parameters.span).end.saturating_sub(1);
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        write!(f, soft_line_break());
        for (i, param) in parameters.params.iter().enumerate() {
            if i > 0 {
                value::write_group_comma(comma_start(i - 1), f);
                write!(f, soft_line_break_or_space());
            }
            value::write_text_with_leading_comments(to_span(param.name.span()), f);
            if let Some(default) = &param.default_value {
                write!(f, [":", space()]);
                value::write_list_element(&default.value, ValueContext::default(), f);
            }
        }
        if let Some(arbitrary) = &parameters.arbitrary_param {
            if !parameters.params.is_empty() {
                value::write_group_comma(comma_start(parameters.params.len() - 1), f);
                write!(f, soft_line_break_or_space());
            }
            value::write_text_with_leading_comments(to_span(arbitrary.name.span()), f);
            write!(f, "...");
        }
        value::flush_paren_tail_comments(r_paren, /* body_hard_broken */ false, f);
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
        let comma_spans = &arguments.comma_spans;
        let r_paren = to_span(&arguments.span).end.saturating_sub(1);
        // Same first-argument gate as `write_function` (which see), over typed args
        let first_arg_is_kw =
            args.first().is_some_and(|a| matches!(a, ComponentValue::SassKeywordArgument(_)));
        let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let source = f.context().source_text();
            write!(f, soft_line_break());
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    value::write_group_comma(comma_spans.get(i - 1).map(|sp| to_span(sp).start), f);
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
                write_top_level_list_element(arg, arg_ctx, f);
            }
            value::flush_paren_tail_comments(r_paren, /* body_hard_broken */ false, f);
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
    for (clause, else_span) in if_rule.else_if_clauses.iter().zip(&if_rule.else_spans) {
        write_else_join(to_span(else_span).start, f);
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
        let else_start = if_rule
            .else_spans
            .last()
            .map_or_else(|| to_span(&else_block.span).start, |sp| to_span(sp).start);
        write_else_join(else_start, f);
        statement::write_block(else_block, f);
    }
}

/// `} @else`: comments between them break the join; each keeps its line
/// (`} /* c */` stays on the brace line, an own-line comment stays own-line).
/// `else_start` (the keyword) bounds the take, so a comment after it leads the condition
/// (`@else if /* c */ $b`) instead.
fn write_else_join(else_start: u32, f: &mut CssFormatter<'_, '_>) {
    let source = f.context().source_text();
    let between = f.context().comments().take_before(else_start);
    for &comment in between {
        if value::comment_is_own_line(comment, source) {
            write!(f, hard_line_break());
        } else {
            write!(f, space());
        }
        write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
    }
    if between.is_empty() {
        write!(f, [space(), "@else", space()]);
    } else {
        write!(f, [hard_line_break(), "@else", space()]);
    }
}

/// Control-directive condition followed by a breakable gap before `{`
/// (Prettier wraps the value + line in a group, without extra indent).
/// A fully parenthesized condition keeps `{` on the `)` line.
pub(super) fn write_control_condition<'a>(
    condition: &ComponentValue<'a>,
    f: &mut CssFormatter<'_, 'a>,
) {
    if matches!(condition, ComponentValue::SassParenthesizedExpression(_)) {
        value::write_list_element(condition, ValueContext::default(), f);
        write!(f, space());
        return;
    }
    if matches!(condition, ComponentValue::SassBinaryExpression(_)) {
        write_condition_chain(None, condition, f);
        return;
    }
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        value::write_list_element(condition, ValueContext::default(), f);
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
            // Each part owns its leading comments, or they would leak into the block
            match part {
                CondPart::Op(span) => value::write_text_with_leading_comments(*span, f),
                CondPart::Value(v) => {
                    value::write_list_element(v, ValueContext::default(), f);
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

/// `@use "path" as ns with (...)`.
///
/// The prelude head is ONE fill — `path`, `as`, `ns` are its chunks —
/// so an overflow breaks at the token seams with a +2 continuation.
/// Prettier reaches the same break points through its generic params path
/// (the whole params string is a comma list of `line`-joined words in a fill);
/// pending comments before a token glue to that token's chunk.
pub(super) fn write_sass_use<'a>(sass_use: &SassUse<'a>, f: &mut CssFormatter<'_, 'a>) {
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let mut filler = f.fill();
        let path = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write_module_path(&sass_use.path, f);
        });
        filler.entry(&soft_line_break_or_space(), &path);
        if let Some(namespace) = &sass_use.namespace {
            let as_kw = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                value::flush_value_comments(to_span(&namespace.as_span).start, f);
                write!(f, "as");
            });
            filler.entry(&soft_line_break_or_space(), &as_kw);
            let name = format_with(move |f: &mut CssFormatter<'_, 'a>| match &namespace.kind {
                SassUseNamespaceKind::Named(ident) => {
                    value::write_text_with_leading_comments(to_span(ident.span()), f);
                }
                SassUseNamespaceKind::Unnamed(star) => {
                    value::flush_value_comments(to_span(&star.span).start, f);
                    write!(f, "*");
                }
            });
            filler.entry(&soft_line_break_or_space(), &name);
        }
        filler.finish();
    });
    write!(f, group(&indent(&body)));
    if let Some(config) = &sass_use.config {
        write_sass_module_config(config, f);
    }
}

/// `@forward "path" as p-* show a, b with (...)`.
///
/// Same fill-of-chunks head as [`write_sass_use`]; each member carries its `,`
/// glued so a break lands after the comma, and same-line comments before a `,`
/// stay glued to their member.
pub(super) fn write_sass_forward<'a>(forward: &SassForward<'a>, f: &mut CssFormatter<'_, 'a>) {
    let body = format_with(move |f: &mut CssFormatter<'_, 'a>| {
        let mut filler = f.fill();
        let path = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write_module_path(&forward.path, f);
        });
        filler.entry(&soft_line_break_or_space(), &path);
        if let Some(prefix) = &forward.prefix {
            let as_kw = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                value::flush_value_comments(to_span(&prefix.as_span).start, f);
                write!(f, "as");
            });
            filler.entry(&soft_line_break_or_space(), &as_kw);
            let name = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                value::write_text_with_leading_comments(to_span(prefix.name.span()), f);
                write!(f, "*");
            });
            filler.entry(&soft_line_break_or_space(), &name);
        }
        if let Some(visibility) = &forward.visibility {
            let keyword = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                value::flush_value_comments(to_span(&visibility.modifier.span).start, f);
                match visibility.modifier.kind {
                    SassForwardVisibilityModifierKind::Show => write!(f, "show"),
                    SassForwardVisibilityModifierKind::Hide => write!(f, "hide"),
                }
            });
            filler.entry(&soft_line_break_or_space(), &keyword);
            let members = &visibility.members;
            for (i, member) in members.iter().enumerate() {
                let entry = format_with(move |f: &mut CssFormatter<'_, 'a>| {
                    value::write_text_with_leading_comments(to_span(member.span()), f);
                    if i + 1 < members.len() {
                        let comma = to_span(&visibility.comma_spans[i]).start;
                        write_same_line_trailing_comments(comma, f);
                        write!(f, ",");
                        value::flush_line_comment_after_comma(comma, f);
                    }
                });
                filler.entry(&soft_line_break_or_space(), &entry);
            }
        }
        filler.finish();
    });
    write!(f, group(&indent(&body)));
    if let Some(config) = &forward.config {
        write_sass_module_config(config, f);
    }
}

/// Module path of `@use`/`@forward`: literal strings re-quote per the quote
/// option, interpolated paths stay verbatim.
fn write_module_path<'a>(path: &InterpolableStr<'a>, f: &mut CssFormatter<'_, 'a>) {
    if let InterpolableStr::Literal(str) = path {
        value::write_str(str, f);
    } else {
        let source = f.context().source_text();
        let span = to_span(path.span());
        write!(f, text(source.text_for(&span)));
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
/// (consistent with the map printer; Prettier pulls it up: a known divergence).
fn write_sass_module_config<'a>(config: &SassModuleConfig<'a>, f: &mut CssFormatter<'_, 'a>) {
    let source = f.context().source_text();
    // Comments between the module path and `with` stay glued to the head
    // (`@use "a" /* c */ with (`).
    write_same_line_trailing_comments(to_span(&config.with_span).start, f);
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
            write_top_level_list_element(&item.value, item_ctx, f);
            for flag in &item.flags {
                let span = to_span(flag.span());
                write!(f, space());
                write!(f, text(source.text_for(&span)));
            }
            if i + 1 < config.items.len() {
                // An own-line comment before the comma stays pending and
                // leads the next item instead.
                let comma = to_span(&config.comma_spans[i]).start;
                write_same_line_trailing_comments(comma, f);
                write!(f, ",");
                value::flush_line_comment_after_comma(comma, f);
            } else {
                // Comments before `)` (past a trailing comma, which is dropped):
                // same-line ones glue to the last item, own-line ones keep their line.
                let bound = to_span(&config.span).end;
                let mut prev_end =
                    write_same_line_trailing_comments(bound, f).unwrap_or(item_span.end);
                for &comment in f.context().comments().take_before(bound) {
                    comments::write_gap(source.bytes_range(prev_end, comment.span.start), f);
                    write!(f, FormatCommentBeforeContent::new(comment, BlockCommentAfter::None));
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
/// Look-alike of `comments::write_trailing_same_line_comments`, minus its `expand_parent`:
/// the map/config bodies this serves already hard-break,
/// so propagating a break out of a still-flat head (`@use "a" /* c */ with (`) would be a behavior change.
/// Returns the end offset of the last emitted comment.
fn write_same_line_trailing_comments(
    upper_bound: u32,
    f: &mut CssFormatter<'_, '_>,
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
            write!(f, FormatLineCommentSuffix::new(comment).with_leading_space());
        } else {
            write!(f, [space(), FormatCommentBeforeContent::new(comment, BlockCommentAfter::None)]);
        }
        last_end = Some(comment.span.end);
    }
    last_end
}
