use std::{borrow::Cow, cmp::Ordering, str::Chars};

use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::line_terminator::LineTerminatorSplitter;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct SortKeys(Box<SortKeysConfig>);

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// Sorting order for keys. Accepts "asc" for ascending or "desc" for descending.
pub enum SortOrder {
    Desc,
    #[default]
    Asc,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SortKeysOptions {
    /// Whether the sort comparison is case-sensitive (A < a when true).
    case_sensitive: bool,
    /// Use natural sort order so that, for example, "a2" comes before "a10".
    natural: bool,
    /// Minimum number of properties required in an object before sorting is enforced.
    min_keys: usize,
    /// When true, groups of properties separated by a blank line are sorted independently.
    allow_line_separated_groups: bool,
}

impl Default for SortKeysOptions {
    fn default() -> Self {
        // we follow the eslint defaults
        Self {
            case_sensitive: true,
            natural: false,
            min_keys: 2,
            allow_line_separated_groups: false,
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(default)]
pub struct SortKeysConfig(SortOrder, SortKeysOptions);

fn sort_properties_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Object keys should be sorted").with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When declaring multiple properties, sorting property names alphabetically makes it easier
    /// to find and/or diff necessary properties at a later time.
    ///
    /// ### Why is this bad?
    ///
    /// Unsorted property keys can make the code harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// let myObj = {
    ///   c: 1,
    ///   a: 2,
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// let myObj = {
    ///   a: 2,
    ///   c: 1,
    /// };
    /// ```
    SortKeys,
    eslint,
    style,
    conditional_fix,
    config = SortKeysConfig,
    version = "0.9.4",
);

impl Rule for SortKeys {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ObjectExpression(dec) = node.kind() {
            let SortKeysConfig(sort_order, options) = &*self.0;

            if dec.properties.len() < options.min_keys {
                return;
            }

            if is_object_sorted(dec, ctx.source_text(), sort_order, options) {
                return;
            }

            if let Some((replace_span, replacement)) =
                build_object_fix(dec, ctx, sort_order, options)
            {
                ctx.diagnostic_with_fix(sort_properties_diagnostic(node.span()), |fixer| {
                    fixer.replace(replace_span, replacement)
                });

                return;
            }

            // Fallback: still emit diagnostic if we couldn't produce a safe fix
            ctx.diagnostic(sort_properties_diagnostic(node.span()));
        }
    }
}

struct FixableProperty<'a> {
    key: Cow<'a, str>,
    span: Span,
    text: Cow<'a, str>,
    /// `text` already includes the inter-property `,` (lifted with a same-line `// ...` comment).
    consumed_trailing_comma: bool,
}

/// Check if all property groups within an object are already sorted.
/// This avoids allocating by comparing adjacent keys in-place.
fn is_object_sorted(
    object: &ObjectExpression<'_>,
    source_text: &str,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> bool {
    let mut prev_key: Option<Cow<'_, str>> = None;

    for (i, prop) in object.properties.iter().enumerate() {
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => {
                prev_key = None;
            }
            ObjectPropertyKind::ObjectProperty(obj) => {
                let Some(key) = obj.key.static_name() else { continue };

                if let Some(ref prev) = prev_key {
                    let ordering = compare_keys(prev, &key, options);
                    let is_ordered = match sort_order {
                        SortOrder::Asc => ordering != Ordering::Greater,
                        SortOrder::Desc => ordering != Ordering::Less,
                    };

                    if !is_ordered {
                        return false;
                    }
                }

                if options.allow_line_separated_groups && i + 1 < object.properties.len() {
                    let text_between = extract_text_between_spans(
                        source_text,
                        prop.span(),
                        object.properties[i + 1].span(),
                    );
                    if has_blank_line(text_between) {
                        prev_key = None;
                        continue;
                    }
                }

                prev_key = Some(key);
            }
        }
    }

    true
}

/// Compare two keys according to sort options, without allocating.
fn compare_keys(a: &str, b: &str, options: &SortKeysOptions) -> Ordering {
    if options.natural {
        natural_compare(a, b, options.case_sensitive)
    } else if options.case_sensitive {
        a.cmp(b)
    } else {
        a.bytes().map(|b| b.to_ascii_lowercase()).cmp(b.bytes().map(|b| b.to_ascii_lowercase()))
    }
}

/// Count contiguous groups of statically-named properties, separated by
/// spreads or blank lines. Replaces the full `collect_property_groups`
/// call used only for the group-count check.
fn count_static_groups(
    object: &ObjectExpression<'_>,
    source_text: &str,
    options: &SortKeysOptions,
) -> usize {
    let mut count = 0;
    let mut in_static_group = false;

    for (i, prop) in object.properties.iter().enumerate() {
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => {
                in_static_group = false;
            }
            ObjectPropertyKind::ObjectProperty(obj) => {
                if obj.key.static_name().is_none() {
                    continue;
                }
                if !in_static_group {
                    count += 1;
                    in_static_group = true;
                }
                if options.allow_line_separated_groups && i + 1 < object.properties.len() {
                    let text_between = extract_text_between_spans(
                        source_text,
                        prop.span(),
                        object.properties[i + 1].span(),
                    );
                    if has_blank_line(text_between) {
                        in_static_group = false;
                    }
                }
            }
        }
    }

    count
}

fn build_object_fix<'a>(
    object: &'a ObjectExpression<'a>,
    ctx: &LintContext<'a>,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Option<(Span, String)> {
    let props = collect_fixable_properties(object, ctx, sort_order, options)?;
    let indices = sorted_property_indices(&props, sort_order, options);
    let has_nested_fix = props.iter().any(|prop| prop.text.as_ref() != ctx.source_range(prop.span));
    let needs_reordering = indices.iter().enumerate().any(|(position, index)| position != *index);

    if !needs_reordering && !has_nested_fix {
        return None;
    }

    // Derive a canonical inter-property whitespace pattern from the first gap in the source
    let canonical_ws: Cow<'_, str> = if let [first, second, ..] = props.as_slice() {
        let raw = ctx.source_range(Span::new(first.span.end, second.span.start));
        raw.split_once(',').map_or(Cow::Borrowed(raw), |(b, a)| Cow::Owned(format!("{b}{a}")))
    } else {
        Cow::Borrowed(" ")
    };

    let mut sorted_text = String::new();
    for (slot, &idx) in indices.iter().enumerate() {
        sorted_text.push_str(&props[idx].text);
        if slot + 1 < indices.len() {
            if !props[idx].consumed_trailing_comma {
                sorted_text.push(',');
            }
            sorted_text.push_str(&canonical_ws);
        }
    }

    // Preserve a trailing comma that was absorbed by the original last prop
    // when the new last prop wouldn't otherwise emit one.
    if let (Some(orig_last), Some(&new_last)) = (props.last(), indices.last())
        && orig_last.consumed_trailing_comma
        && !props[new_last].consumed_trailing_comma
    {
        sorted_text.push(',');
    }

    Some((Span::new(props[0].span.start, props[props.len() - 1].span.end), sorted_text))
}

fn collect_fixable_properties<'a>(
    object: &'a ObjectExpression<'a>,
    ctx: &LintContext<'a>,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Option<Vec<FixableProperty<'a>>> {
    enum SpreadPos {
        Start,
        CanEnd,
        End,
    }

    if count_static_groups(object, ctx.source_text(), options) != 1 {
        return None;
    }

    // For each property, the source span we'd lift to move it — widened to
    // absorb comments that should travel with the property. Other comments
    // remain in the inter-property gap and trip the guard below.
    let lifted: Vec<Span> = object
        .properties
        .iter()
        .enumerate()
        .map(|(i, prop)| {
            let trailing_boundary =
                object.properties.get(i + 1).map_or(object.span.end, |next| next.span().start);
            lift_property_span(prop.span(), trailing_boundary, ctx)
        })
        .collect();

    if let (Some(&first), Some(&last)) = (lifted.first(), lifted.last())
        && (ctx.has_comments_between(Span::new(object.span.start, first.start))
            || ctx.has_comments_between(Span::new(last.end, object.span.end)))
    {
        return None;
    }

    let mut spread_pos = SpreadPos::Start;
    let mut props = Vec::with_capacity(object.properties.len());

    for (i, prop) in object.properties.iter().enumerate() {
        let gap_after = (i + 1 < object.properties.len())
            .then(|| Span::new(lifted[i].end, lifted[i + 1].start));
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => {
                if let Some(ObjectPropertyKind::ObjectProperty(_)) = object.properties.get(i + 1)
                    && let Some(gap) = gap_after
                    && ctx.has_comments_between(gap)
                {
                    return None;
                }

                match spread_pos {
                    SpreadPos::Start | SpreadPos::End => {}
                    SpreadPos::CanEnd => spread_pos = SpreadPos::End,
                }
            }
            ObjectPropertyKind::ObjectProperty(obj) => {
                match spread_pos {
                    SpreadPos::Start => spread_pos = SpreadPos::CanEnd,
                    SpreadPos::CanEnd => {}
                    SpreadPos::End => return None,
                }

                let key = obj.key.static_name()?;

                props.push(FixableProperty {
                    key,
                    span: lifted[i],
                    text: build_property_text(obj, lifted[i], ctx, sort_order, options),
                    consumed_trailing_comma: lifted[i].end > prop.span().end,
                });

                if let Some(gap) = gap_after
                    && ctx.has_comments_between(gap)
                {
                    return None;
                }
            }
        }
    }

    (!props.is_empty()).then_some(props)
}

/// Widen `prop_span` to absorb a leading jsdoc anchored at its start and a
/// trailing `, // ...` line comment on the same line as its end, bounded by
/// `trailing_boundary` (next property's start, or enclosing object's end).
fn lift_property_span(prop_span: Span, trailing_boundary: u32, ctx: &LintContext<'_>) -> Span {
    let mut start = prop_span.start;
    for comment in ctx.comments_range(..prop_span.start).rev() {
        if comment.attached_to != prop_span.start {
            break;
        }
        if comment.is_jsdoc() {
            start = comment.span.start;
        }
    }

    let mut end = prop_span.end;
    if prop_span.end < trailing_boundary
        && let Some(comment) = ctx.comments_range(prop_span.end..trailing_boundary).next()
        && comment.is_line()
    {
        let between = ctx.source_range(Span::new(prop_span.end, comment.span.start));
        if between.contains(',') && !between.contains('\n') {
            end = comment.span.end;
        }
    }

    Span::new(start, end)
}

fn sorted_property_indices(
    props: &[FixableProperty<'_>],
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..props.len()).collect();

    indices.sort_unstable_by(|&a, &b| {
        let cmp = compare_keys(&props[a].key, &props[b].key, options);
        match sort_order {
            SortOrder::Asc => cmp,
            SortOrder::Desc => cmp.reverse(),
        }
    });

    indices
}

fn build_property_text<'a>(
    property: &'a ObjectProperty<'a>,
    span: Span,
    ctx: &LintContext<'a>,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Cow<'a, str> {
    let Expression::ObjectExpression(object) = &property.value else {
        return Cow::Borrowed(ctx.source_range(span));
    };
    let Some((replace_span, replacement)) = build_object_fix(object, ctx, sort_order, options)
    else {
        return Cow::Borrowed(ctx.source_range(span));
    };

    let before_value = ctx.source_range(Span::new(span.start, replace_span.start));
    let after_value = ctx.source_range(Span::new(replace_span.end, span.end));

    Cow::Owned(format!("{before_value}{replacement}{after_value}"))
}

fn natural_compare(a: &str, b: &str, case_sensitive: bool) -> Ordering {
    let mut a_chars = a.chars();
    let mut b_chars = b.chars();

    loop {
        let a_next = a_chars.next();
        let b_next = b_chars.next();

        match (a_next, b_next) {
            (None, None) => return Ordering::Equal,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (Some(a_raw), Some(b_raw)) => {
                let a_char = if case_sensitive { a_raw } else { a_raw.to_ascii_lowercase() };
                let b_char = if case_sensitive { b_raw } else { b_raw.to_ascii_lowercase() };

                if a_char == b_char {
                    continue;
                }
                if a_char.is_ascii_digit() && b_char.is_ascii_digit() {
                    let n1 = take_numeric(&mut a_chars, a_char);
                    let n2 = take_numeric(&mut b_chars, b_char);
                    match n1.cmp(&n2) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                if a_char.is_alphanumeric() && !b_char.is_alphanumeric() {
                    return Ordering::Greater;
                }
                if !a_char.is_alphanumeric() && b_char.is_alphanumeric() {
                    return Ordering::Less;
                }
                if a_char == '[' && b_char.is_alphanumeric() {
                    return Ordering::Greater;
                }
                if a_char.is_alphanumeric() && b_char == '[' {
                    return Ordering::Less;
                }
                return a_char.cmp(&b_char);
            }
        }
    }
}

fn take_numeric(iter: &mut Chars, first: char) -> u32 {
    let mut sum = first.to_digit(10).unwrap();
    for c in iter.by_ref() {
        if let Some(digit) = c.to_digit(10) {
            sum = sum * 10 + digit;
        } else {
            break;
        }
    }
    sum
}

fn extract_text_between_spans(source_text: &str, current_span: Span, next_span: Span) -> &str {
    let cur_span_end = current_span.end as usize;
    let next_span_start = next_span.start as usize;
    &source_text[cur_span_end..next_span_start]
}

fn has_blank_line(text: &str) -> bool {
    LineTerminatorSplitter::new(text).skip(1).any(str::is_empty)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var obj = {'':1, [``]:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {[``]:1, '':2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {'':1, a:2}", Some(serde_json::json!([]))),
        ("var obj = {[``]:1, a:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {_:2, a:1, b:3} // default", Some(serde_json::json!([]))),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!([]))),
        ("var obj = {a:2, b:3, b_:1}", Some(serde_json::json!([]))),
        ("var obj = {C:3, b_:1, c:2}", Some(serde_json::json!([]))),
        ("var obj = {$:1, A:3, _:2, a:4}", Some(serde_json::json!([]))),
        ("var obj = {1:1, '11':2, 2:4, A:3}", Some(serde_json::json!([]))),
        ("var obj = {'#':1, 'Z':2, À:3, è:4}", Some(serde_json::json!([]))),
        ("var obj = { [/(?<zero>0)/]: 1, '/(?<zero>0)/': 2 }", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        // ("var obj = {a:1, b:3, [a + b]: -1, c:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {'':1, [f()]:2, a:3}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        // ("var obj = {a:1, [b++]:2, '':3}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, ...z, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {b:1, ...z, a:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...a, b:1, ...c, d:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...a, b:1, ...d, ...c, e:2, z:5}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {b:1, ...c, ...d, e:2}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, ...z, '':2}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {'':1, ...z, 'a':2}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, a:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...c, a:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, b:1, ...z}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...x, a:1, ...c, ...d, f:5, e:4}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("function fn(...args) { return [...args].length; }", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        (
            "function g() {}; function f(...args) { return g(...args); }",
            Some(serde_json::json!([])),
        ), // { "ecmaVersion": 2018 },
        ("let {a, b} = {}", Some(serde_json::json!([]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, b:{x:1, y:1}, c:1}", Some(serde_json::json!([]))),
        ("var obj = {_:2, a:1, b:3} // asc", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:2, b:3, b_:1}", Some(serde_json::json!(["asc"]))),
        ("var obj = {C:3, b_:1, c:2}", Some(serde_json::json!(["asc"]))),
        ("var obj = {$:1, A:3, _:2, a:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {1:1, '11':2, 2:4, A:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {'#':1, 'Z':2, À:3, è:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc", { "minKeys": 4 }]))),
        (
            "var obj = {_:2, a:1, b:3} // asc, insensitive",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!(["asc", { "caseSensitive": false }]))),
        (
            "var obj = {a:2, b:3, b_:1}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, C:3, c:2}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:3, C:2}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, '11':2, 2:4, A:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, 'Z':2, À:3, è:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, A:3, _:2, a:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false, "minKeys": 5 }])),
        ),
        (
            "var obj = {_:2, a:1, b:3} // asc, natural",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {a:1, b:3, c:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {a:2, b:3, b_:1}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {C:3, b_:1, c:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {1:1, 2:4, '11':2, A:3}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {'#':1, 'Z':2, À:3, è:4}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {'a²': 1, 'b³': 2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "minKeys": 4 }])),
        ),
        (
            "var obj = {_:2, a:1, b:3} // asc, natural, insensitive",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, b:3, c:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:2, b:3, b_:1}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, C:3, c:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:3, C:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, '11':2, A:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, 'Z':2, À:3, è:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["asc", { "natural": true, "caseSensitive": false, "minKeys": 4 }]),
            ),
        ),
        ("var obj = {b:3, a:1, _:2} // desc", Some(serde_json::json!(["desc"]))),
        ("var obj = {c:2, b:3, a:1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {b_:1, b:3, a:2}", Some(serde_json::json!(["desc"]))),
        ("var obj = {c:2, b_:1, C:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:4, _:2, A:3, $:1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {A:3, 2:4, '11':2, 1:1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {è:4, À:3, 'Z':2, '#':1}", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["desc", { "minKeys": 4 }]))),
        (
            "var obj = {b:3, a:1, _:2} // desc, insensitive",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, b:3, a:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, b:3, a:2}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, C:3, b_:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {C:2, c:3, b_:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:4, A:3, _:2, $:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {A:3, 2:4, '11':2, 1:1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {è:4, À:3, 'Z':2, '#':1}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["desc", { "caseSensitive": false, "minKeys": 5 }])),
        ),
        (
            "var obj = {b:3, a:1, _:2} // desc, natural",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        ("var obj = {c:2, b:3, a:1}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {b_:1, b:3, a:2}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {c:2, b_:1, C:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {a:4, A:3, _:2, $:1}", Some(serde_json::json!(["desc", { "natural": true }]))),
        (
            "var obj = {A:3, '11':2, 2:4, 1:1}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {è:4, À:3, 'Z':2, '#':1}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "minKeys": 4 }])),
        ),
        (
            "var obj = {b:3, a:1, _:2} // desc, natural, insensitive",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, b:3, a:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, b:3, a:2}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {c:2, C:3, b_:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {C:2, c:3, b_:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:4, A:3, _:2, $:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {A:3, '11':2, 2:4, 1:1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {è:4, À:3, 'Z':2, '#':1}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["desc", { "natural": true, "caseSensitive": false, "minKeys": 4 }]),
            ),
        ),
        (
            "
                            var obj = {
                                e: 1,
                                f: 2,
                                g: 3,

                                a: 4,
                                b: 5,
                                c: 6
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "var obj = {\r\n  c: 1,\r\n  d: 2,\r\n\r\n  a: 3,\r\n  b: 4,\r\n};",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "var obj = {\u{2028}  c: 1,\u{2028}  d: 2,\u{2028}\u{2029}  a: 3,\u{2028}  b: 4,\u{2028}};",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
                            var obj = {
                                b: 1,

                                // comment
                                a: 2,
                                c: 3
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
                            var obj = {
                                b: 1

                                ,

                                // comment
                                a: 2,
                                c: 3
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
                            var obj = {
                                c: 1,
                                d: 2,

                                b() {
                                },
                                e: 4
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                c: 1,
                                d: 2,
                                // comment

                                // comment
                                b() {
                                },
                                e: 4
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                              b,

                              [a+b]: 1,
                              a
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                c: 1,
                                d: 2,

                                a() {

                                },

                                // abce
                                f: 3,

                                /*

                                */
                                [a+b]: 1,
                                cc: 1,
                                e: 2
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            r#"
                            var obj = {
                                b: "/*",

                                a: "*/",
                            }
                        "#,
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
                            var obj = {
                                b,
                                /*
                                */ //

                                a
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                b,

                                /*
                                */ //
                                a
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                b: 1

                                ,a: 2
                            };
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                b: 1
                            // comment before comma

                            ,
                            a: 2
                            };
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                              b,

                              a,
                              ...z,
                              c
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "
                            var obj = {
                              b,

                              [foo()]: [

                              ],
                              a
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 2018 }
    ];

    let fail = vec![
        ("var obj = {a:1, '':2} // default", None),
        ("var obj = {a:1, [``]:2} // default", None), // { "ecmaVersion": 6 },
        ("var obj = {a:1, _:2, b:3} // default", None),
        ("var obj = {a:1, c:2, b:3}", None),
        ("var obj = {b_:1, a:2, b:3}", None),
        ("var obj = {b_:1, c:2, C:3}", None),
        ("var obj = {$:1, _:2, A:3, a:4}", None),
        ("var obj = {1:1, 2:4, A:3, '11':2}", None),
        ("var obj = {'#':1, À:3, 'Z':2, è:4}", None),
        ("var obj = { null: 1, [/(?<zero>0)/]: 2 }", None), // { "ecmaVersion": 2018 },
        ("var obj = {...z, c:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...c, d:4, b:1, ...y, ...f, e:2, a:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {c:1, b:1, ...a}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, ...a, c:1, b:1}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, b:1, a:1, ...d, ...c}", Some(serde_json::json!([]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, a:2, b:0, ...x, ...c}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, a:2, b:0, ...x}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {...z, '':1, a:2}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 2018 },
        ("var obj = {a:1, [b+c]:2, '':3}", None),                           // { "ecmaVersion": 6 },
        ("var obj = {'':1, [b+c]:2, a:3}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {b:1, [f()]:2, '':3, a:4}", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        // ("var obj = {a:1, b:3, [a]: -1, c:2}", None), // { "ecmaVersion": 6 },
        ("var obj = {a:1, c:{y:1, x:1}, b:1}", None),
        ("var obj = {a:1, _:2, b:3} // asc", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["asc"]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {1:1, 2:4, A:3, '11':2}", Some(serde_json::json!(["asc"]))),
        ("var obj = {'#':1, À:3, 'Z':2, è:4}", Some(serde_json::json!(["asc"]))),
        ("var obj = {a:1, _:2, b:3}", Some(serde_json::json!(["asc", { "minKeys": 3 }]))),
        (
            "var obj = {a:1, _:2, b:3} // asc, insensitive",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc", { "caseSensitive": false }]))),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, A:3, _:2, a:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false, "minKeys": 3 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // asc, natural",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {$:1, A:3, _:2, a:4}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "minKeys": 2 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // asc, natural, insensitive",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, c:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, A:3, _:2, a:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, '11':2, 2:4, A:3}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["asc", { "natural": true, "caseSensitive": false, "minKeys": 3 }]),
            ),
        ),
        ("var obj = {'':1, a:'2'} // desc", Some(serde_json::json!(["desc"]))),
        ("var obj = {[``]:1, a:'2'} // desc", Some(serde_json::json!(["desc"]))), // { "ecmaVersion": 6 },
        ("var obj = {a:1, _:2, b:3} // desc", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["desc"]))),
        ("var obj = {1:1, 2:4, A:3, '11':2}", Some(serde_json::json!(["desc"]))),
        ("var obj = {'#':1, À:3, 'Z':2, è:4}", Some(serde_json::json!(["desc"]))),
        ("var obj = {a:1, _:2, b:3}", Some(serde_json::json!(["desc", { "minKeys": 3 }]))),
        (
            "var obj = {a:1, _:2, b:3} // desc, insensitive",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, c:2, b:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:2, C:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["desc", { "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["desc", { "caseSensitive": false, "minKeys": 2 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // desc, natural",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        ("var obj = {a:1, c:2, b:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {b_:1, a:2, b:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {b_:1, c:2, C:3}", Some(serde_json::json!(["desc", { "natural": true }]))),
        ("var obj = {$:1, _:2, A:3, a:4}", Some(serde_json::json!(["desc", { "natural": true }]))),
        (
            "var obj = {1:1, 2:4, A:3, '11':2}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["desc", { "natural": true }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "minKeys": 3 }])),
        ),
        (
            "var obj = {a:1, _:2, b:3} // desc, natural, insensitive",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, c:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, a:2, b:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {b_:1, c:2, C:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {$:1, _:2, A:3, a:4}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {1:1, 2:4, '11':2, A:3}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {'#':1, À:3, 'Z':2, è:4}",
            Some(serde_json::json!(["desc", { "natural": true, "caseSensitive": false }])),
        ),
        (
            "var obj = {a:1, _:2, b:3}",
            Some(
                serde_json::json!(["desc", { "natural": true, "caseSensitive": false, "minKeys": 2 }]),
            ),
        ),
        (
            "
                            var obj = {
                                b: 1,
                                c: 2,
                                a: 3
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": false }])),
        ),
        (
            "
                            let obj = {
                                b

                                ,a
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                             var obj = {
                                b: 1,
                                c () {

                                },
                                a: 3
                              }
                         ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                             var obj = {
                                a: 1,
                                b: 2,

                                z () {

                                },
                                y: 3
                              }
                         ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                             var obj = {
                                b: 1,
                                c () {
                                },
                                // comment
                                a: 3
                              }
                         ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                              b,
                              [a+b]: 1,
                              a // sort-keys: 'a' should be before 'b'
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                c: 1,
                                d: 2,
                                // comment
                                // comment
                                b() {
                                },
                                e: 4
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            var obj = {
                                c: 1,
                                d: 2,

                                z() {

                                },
                                f: 3,
                                /*


                                */
                                [a+b]: 1,
                                b: 1,
                                e: 2
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            r#"
                            var obj = {
                                b: "/*",
                                a: "*/",
                            }
                        "#,
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        (
            "
                            var obj = {
                                b: 1
                                // comment before comma
                                , a: 2
                            };
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "
                            let obj = {
                              b,
                              [foo()]: [
                              // ↓ this blank is inside a property and therefore should not count

                              ],
                              a
                            }
                        ",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ), // { "ecmaVersion": 2018 }
    ];

    // Add comprehensive fixer tests: the rule now advertises conditional fixes,
    // so provide expect_fix cases.
    let fix = vec![
        // Basic alphabetical sorting
        ("var obj = {b:1, a:2}", "var obj = {a:2, b:1}"),
        // Case sensitivity - lowercase comes after uppercase, so a:2 should come after B:1
        ("var obj = {a:1, B:2}", "var obj = {B:2, a:1}"),
        // Trailing commas preserved
        ("var obj = {b:1, a:2,}", "var obj = {a:2, b:1,}"),
        // With spaces and various formatting
        ("var obj = { z: 1, a: 2 }", "var obj = { a: 2, z: 1 }"),
        // Three properties
        ("var obj = {c:1, a:2, b:3}", "var obj = {a:2, b:3, c:1}"),
        // Mixed types
        ("var obj = {2:1, a:2, 1:3}", "var obj = {1:3, 2:1, a:2}"),
        // Spreading at the start
        ("var obj = {...z, b:1, a:2}", "var obj = {...z, a:2, b:1}"),
        // Spreading at the start when one of the keys is the empty string
        ("var obj = {...z, a:1, '':2}", "var obj = {...z, '':2, a:1}"),
        // No fix when a leading spread has a trailing comment
        ("var obj = {...z, /*c*/ b:1, a:2}", "var obj = {...z, /*c*/ b:1, a:2}"),
        // Spreading multiple times at the start
        ("var obj = {...z, ...y, b:1, a:2,}", "var obj = {...z, ...y, a:2, b:1,}"),
        // Spreading at the end
        ("var obj = { b:1, a:2, ...z}", "var obj = { a:2, b:1, ...z}"),
        // Spreading multiple times at the end
        ("var obj = {b:1, a:2, ...z, ...y}", "var obj = {a:2, b:1, ...z, ...y}"),
        // Spreading at both the start and end
        ("var obj = {...z, b:1, a:2, ...y}", "var obj = {...z, a:2, b:1, ...y}"),
        // Spreading multiple times at both the start and end
        (
            "var obj = { ...z, ...y, b:1, a:2, ...x, ...w, }",
            "var obj = { ...z, ...y, a:2, b:1, ...x, ...w, }",
        ),
        // Multi-line formatting should be preserved (issue #16391)
        (
            "const obj = {
    val: 'germany',
    key: 'de',
    id: 123,
}",
            "const obj = {
    id: 123,
    key: 'de',
    val: 'germany',
}",
        ),
        // Multi-line with different indentation
        (
            "var obj = {
  c: 1,
  a: 2,
  b: 3
}",
            "var obj = {
  a: 2,
  b: 3,
  c: 1
}",
        ),
        // spellchecker:off
        (
            "const values = {
    b: {
        bb: 2,
        ba: 1,
    },
    a: {
        ab: 2,
        aa: 1,
    },
};",
            "const values = {
    a: {
        aa: 1,
        ab: 2,
    },
    b: {
        ba: 1,
        bb: 2,
    },
};",
        ),
        (
            r#"const config = {
    variants: {
        variant: {
            default: "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
            outline:
                "bg-background hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        },
        size: {
            default: "h-8 text-sm",
            sm: "h-7 text-xs",
            lg: "h-12 text-sm group-data-[collapsible=icon]:p-0!",
        },
    },
    defaultVariants: {
        variant: "default",
        size: "default",
    },
};"#,
            r#"const config = {
    defaultVariants: {
        size: "default",
        variant: "default",
    },
    variants: {
        size: {
            default: "h-8 text-sm",
            lg: "h-12 text-sm group-data-[collapsible=icon]:p-0!",
            sm: "h-7 text-xs",
        },
        variant: {
            default: "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
            outline:
                "bg-background hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        },
    },
};"#,
        ),
        // spellchecker:on
        (
            "const values = {
  c: 3,
  b: 2,
  a: 1, // Inline comment on a
};",
            "const values = {
  a: 1, // Inline comment on a
  b: 2,
  c: 3,
};",
        ),
        (
            "const values = {
  c: 3,
  a: 1, // Inline comment on a
  b: 2,
};",
            "const values = {
  a: 1, // Inline comment on a
  b: 2,
  c: 3,
};",
        ),
        (
            "const values = {
  c: 3, // c
  b: 2, // b
  a: 1, // a
};",
            "const values = {
  a: 1, // a
  b: 2, // b
  c: 3, // c
};",
        ),
        // jsdoc comments are carried with their property when reordered
        (
            "const values = {
  /** jsdoc Comment on c */
  c: 3,
  /** jsdoc Comment on b */
  b: 2,
  /** jsdoc Comment on a */
  a: 1,
};",
            "const values = {
  /** jsdoc Comment on a */
  a: 1,
  /** jsdoc Comment on b */
  b: 2,
  /** jsdoc Comment on c */
  c: 3,
};",
        ),
        (
            "const values = { /** inline jsdoc on b */ b: 2, a: 1}",
            "const values = { a: 1, /** inline jsdoc on b */ b: 2}",
        ),
        (
            "const values = { b: 2, /** inline jsdoc on a */ a: 1}",
            "const values = { /** inline jsdoc on a */ a: 1, b: 2}",
        ),
        // '/*' and '//' comments currently not autofixed
        (
            "const values = {
  // Comment above c
  c: 3,
  // Comment above b
  b: 2,
  // Comment above a
  a: 1,
};",
            "const values = {
  // Comment above c
  c: 3,
  // Comment above b
  b: 2,
  // Comment above a
  a: 1,
};",
        ),
        (
            "const values = {
  /* Comment above c */
  c: 3,
  /* Comment above b */
  b: 2,
  /* Comment above a */
  a: 1,
};",
            "const values = {
  /* Comment above c */
  c: 3,
  /* Comment above b */
  b: 2,
  /* Comment above a */
  a: 1,
};",
        ),
        // Missing trailing comma not currently handled
        (
            "const values = {
  c: 3, // c
  b: 2, // b
  a: 1  // a
};",
            "const values = {
  c: 3, // c
  b: 2, // b
  a: 1  // a
};",
        ),
        // Not sure where these comments belong -> should probably not autofix
        (
            "const values = { /* comment */ b: 2, a: 1}",
            "const values = { /* comment */ b: 2, a: 1}",
        ),
        ("const values = {b: 2, /* comment */ a: 1}", "const values = {b: 2, /* comment */ a: 1}"),
        (
            "const values = {b: 2, a: 1 /* comment */ }",
            "const values = {b: 2, a: 1 /* comment */ }",
        ),
        // ****************
    ];

    Tester::new(SortKeys::NAME, SortKeys::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
