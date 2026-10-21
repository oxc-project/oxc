use std::{borrow::Cow, cmp::Ordering, iter::Peekable};

use oxc_ast::{
    AstKind, StaticPropertyName,
    ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;
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
    min_keys: u32,
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
    short_description = "When declaring multiple properties, sorting property names alphabetically makes it easier to find and/or diff necessary properties at a later time.",
);

impl Rule for SortKeys {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    #[expect(clippy::cast_possible_truncation)] // the length of properties can't be over u32::MAX, because the source code is already limited by u32::MAX.
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ObjectExpression(dec) = node.kind() {
            let SortKeysConfig(sort_order, options) = &*self.0;

            if (dec.properties.len() as u32) < options.min_keys {
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
    key: StaticPropertyName<'a>,
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
    let mut prev_key: Option<StaticPropertyName<'_>> = None;

    for (i, prop) in object.properties.iter().enumerate() {
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => {
                prev_key = None;
            }
            ObjectPropertyKind::ObjectProperty(obj) => {
                let Some(key) = obj.key.static_name().or_else(|| {
                    match obj.key.as_expression()?.without_parentheses() {
                        Expression::Identifier(ident) => Some(StaticPropertyName::from(ident.name)),
                        _ => None,
                    }
                }) else {
                    continue;
                };

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

/// Compare two keys according to sort options.
fn compare_keys(
    a: &StaticPropertyName<'_>,
    b: &StaticPropertyName<'_>,
    options: &SortKeysOptions,
) -> Ordering {
    compare_key_names(a.as_js_str(), b.as_js_str(), options)
}

fn compare_key_names(a: JSStr<'_>, b: JSStr<'_>, options: &SortKeysOptions) -> Ordering {
    if options.natural {
        return natural_compare(a, b, options.case_sensitive);
    }
    // Canonical WTF-8 bytes order like code points, so lone surrogates need no decoding.
    // ESLint compares UTF-16 code units instead, which differs for astral characters (#26242).
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if options.case_sensitive {
        a.cmp(b)
    } else {
        a.iter().map(u8::to_ascii_lowercase).cmp(b.iter().map(u8::to_ascii_lowercase))
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

    // A stable sort keeps duplicate keys in source order, so the last one still wins.
    indices.sort_by(|&a, &b| {
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

/// Compare like ESLint's `natural-compare`, which works on UTF-16 code units:
/// a digit run that starts with `1`-`9` compares by numeric value,
/// and other code units compare in the order of [`natural_rank`].
/// `natural-compare` reads a NUL code unit as the end of the string,
/// so each key is compared up to its first NUL.
///
/// Case-insensitive mode lowercases ASCII only, where ESLint lowercases with `toLowerCase`.
fn natural_compare(a: JSStr<'_>, b: JSStr<'_>, case_sensitive: bool) -> Ordering {
    compare_natural(
        a.encode_utf16().take_while(|&unit| unit != 0).map(u32::from),
        b.encode_utf16().take_while(|&unit| unit != 0).map(u32::from),
        case_sensitive,
    )
}

fn compare_natural(
    a: impl Iterator<Item = u32>,
    b: impl Iterator<Item = u32>,
    case_sensitive: bool,
) -> Ordering {
    let (mut a, mut b) = (a.peekable(), b.peekable());
    let normalize = |c: u32| {
        if !case_sensitive && (u32::from(b'A')..=u32::from(b'Z')).contains(&c) {
            c + u32::from(b'a' - b'A')
        } else {
            c
        }
    };
    loop {
        let (Some(&a_char), Some(&b_char)) = (a.peek(), b.peek()) else {
            return a.peek().is_some().cmp(&b.peek().is_some());
        };
        // A leading zero does not start a number, so `a01` compares `0` with `1` as characters.
        if is_nonzero_digit(a_char) && is_nonzero_digit(b_char) {
            match compare_digit_runs(&mut a, &mut b) {
                Ordering::Equal => continue,
                ordering => return ordering,
            }
        }
        a.next();
        b.next();
        let (a_char, b_char) = (normalize(a_char), normalize(b_char));
        if a_char != b_char {
            return natural_rank(a_char).cmp(&natural_rank(b_char));
        }
    }
}

/// The order of `natural-compare` for one code unit: ASCII punctuation, `-`, digits, `A-Z`, `a-z`,
/// then every code unit above U+007F by value.
fn natural_rank(c: u32) -> u32 {
    match c {
        // `-`
        0x2D => 65,
        // `.` and `/`
        0x2E..=0x2F => c - 1,
        // `0`-`9`
        0x30..=0x39 => c + 18,
        // `:` to `@`
        0x3A..=0x40 => c - 11,
        // `A`-`Z`
        0x41..=0x5A => c + 11,
        // `[` to `` ` ``
        0x5B..=0x60 => c - 37,
        // `a`-`z`
        0x61..=0x7A => c + 5,
        // `{` to DEL
        0x7B..=0x7F => c - 63,
        _ => c,
    }
}

fn is_nonzero_digit(c: u32) -> bool {
    (u32::from(b'1')..=u32::from(b'9')).contains(&c)
}

/// Compare two digit runs that start with a nonzero digit, by numeric value and without overflow.
///
/// A longer run is the larger number, and runs of equal length compare digit by digit.
/// Equal runs are consumed in full, so the comparison continues after them.
/// `natural-compare` converts runs to doubles, so runs that differ only beyond 2^53 compare equal
/// there; here they compare exactly.
fn compare_digit_runs(
    a: &mut Peekable<impl Iterator<Item = u32>>,
    b: &mut Peekable<impl Iterator<Item = u32>>,
) -> Ordering {
    let is_digit = |c: &u32| (u32::from(b'0')..=u32::from(b'9')).contains(c);
    let mut ordering = Ordering::Equal;
    loop {
        match (a.next_if(is_digit), b.next_if(is_digit)) {
            (Some(a_digit), Some(b_digit)) => ordering = ordering.then(a_digit.cmp(&b_digit)),
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return ordering,
        }
    }
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
        (
            r#"var obj = {"a1\uD800": 1, "a2": 2, "a10": 3}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
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
        // Digit runs compare by value, and a leading zero compares as a character, like ESLint.
        ("var obj = {a12:1, a100:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {item1Name:1, item10Name:2}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {step01Init:1, step1Build:2}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {a0:1, a00:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {a01:1, a1:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {a9:1, a10:2, a99999999999999999999:3}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {A12:1, a100:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        // Other characters follow natural-compare's order, like ESLint.
        (
            r#"var obj = {"/user/list": 1, "/user-list": 2}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (r#"var obj = {a_b: 1, "a-b": 2}"#, Some(serde_json::json!(["asc", { "natural": true }]))),
        (r#"var obj = {a: 1, "€": 2}"#, Some(serde_json::json!(["asc", { "natural": true }]))),
        // natural-compare stops at a NUL code unit, like ESLint.
        (
            r#"var obj = {"\0b": 1, "\0a": 2}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (r#"var obj = {"a\0z": 1, "a": 2}"#, Some(serde_json::json!(["asc", { "natural": true }]))),
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
        // Lone surrogates order by code point, like every other key.
        (r#"var obj = {"\uD800": 1, "\uDC00": 2}"#, None),
        (r#"var obj = {"a": 1, "\uD800": 2}"#, None),
        (r#"var obj = {"\uDC00": 1, "𐀀": 2}"#, None),
        (r#"var obj = {"\uE000": 1, "😀": 2}"#, None),
        (r#"var obj = {"\uD800": 1, "\uD800\uDC00": 2}"#, None),
        (r#"var obj = {"\uDC00": 1, "\uD800": 2}"#, Some(serde_json::json!(["desc"]))),
        // Natural mode compares UTF-16 code units like natural-compare, so a lone surrogate sorts
        // after ASCII.
        (
            r#"var obj = {"a1": 1, "\uD800": 2}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            r#"var obj = {"A": 1, "b": 2, "\uD800": 3}"#,
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        ("var obj = {a:1, [b]:2, c:3}", None),
        ("const a = 'z'; var obj = {[a]:1, b:2}", None),
        ("var obj = {a:1, [b + c]:2, d:3}", None),
        (
            "var obj = {a:1, [B]:2, c:3}",
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        ("var obj = {[a2]:1, [a10]:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {b:1,\n\n[a]:2}",
            Some(serde_json::json!(["asc", { "allowLineSeparatedGroups": true }])),
        ),
        ("var obj = {b:1, ...rest, [a]:2}", None),
    ];

    let fail = vec![
        (
            r#"var obj = {"a2": 2, "a10": 3, "a1\uD800": 1}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
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
        ("var obj = {a:1, b:3, [a]: -1, c:2}", None),
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
        // Digit runs compare by value, and a leading zero compares as a character, like ESLint.
        ("var obj = {a100:1, a12:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {item10Name:1, item1Name:2}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            "var obj = {step1Build:1, step01Init:2}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        ("var obj = {a00:1, a0:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        ("var obj = {a1:1, a01:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            "var obj = {a100:1, A12:2}",
            Some(serde_json::json!(["asc", { "natural": true, "caseSensitive": false }])),
        ),
        // Other characters follow natural-compare's order, like ESLint.
        (
            r#"var obj = {"/user-list": 1, "/user/list": 2}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (r#"var obj = {"a-b": 1, a_b: 2}"#, Some(serde_json::json!(["asc", { "natural": true }]))),
        (r#"var obj = {"€": 1, a: 2}"#, Some(serde_json::json!(["asc", { "natural": true }]))),
        (
            r#"var obj = {"b\0a": 1, "a\0b": 2}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
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
        // Lone surrogates order by code point, like every other key.
        (r#"var obj = {"\uDC00": 1, "\uD800": 2}"#, None),
        (r#"var obj = {"\uD800": 1, "a": 2}"#, None),
        (r#"var obj = {"\uD800\uDC00": 1, "\uD800": 2}"#, None),
        (r#"var obj = {"\uD800": 1, "\uDC00": 2}"#, Some(serde_json::json!(["desc"]))),
        // Natural mode compares UTF-16 code units like natural-compare, so a lone surrogate sorts
        // after ASCII.
        (
            r#"var obj = {"\uD800": 1, "a1": 2}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            r#"var obj = {"\uD800": 1, "b": 2, "A": 3}"#,
            Some(serde_json::json!(["asc", { "caseSensitive": false }])),
        ),
        (r#"export const obj5 = {a: 1, ["c"]: 3, b: 2};"#, None),
        (r#"const S = Symbol("s"); export const obj6 = {a: 1, [S]: 3, b: 2};"#, None),
        ("var obj = {[c]:1, b:2}", None),
        ("var obj = {[(c)]:1, b:2}", None),
        ("var obj = {c:1, [a]:2, b:3}", Some(serde_json::json!(["desc"]))),
        ("var obj = {[c]:1, [B]:2}", Some(serde_json::json!(["asc", { "caseSensitive": false }]))),
        ("var obj = {[a10]:1, [a2]:2}", Some(serde_json::json!(["asc", { "natural": true }]))),
    ];

    // Add comprehensive fixer tests: the rule now advertises conditional fixes,
    // so provide expect_fix cases.
    let fix = vec![
        // Past 20 properties, an unstable sort could swap the duplicate `b` keys.
        (
            "var obj = {n:0, b:1, g:2, t:3, i:4, s:5, h:6, b:7, c:8, u:9, r:10, q:11, d:12, f:13, j:14, l:15, p:16, e:17, a:18, k:19, o:20, m:21}",
            "var obj = {a:18, b:1, b:7, c:8, d:12, e:17, f:13, g:2, h:6, i:4, j:14, k:19, l:15, m:21, n:0, o:20, p:16, q:11, r:10, s:5, t:3, u:9}",
        ),
        // Lone surrogates keep their source spelling.
        (r#"var obj = {"\uDC00": 1, "\uD800": 2}"#, r#"var obj = {"\uD800": 2, "\uDC00": 1}"#),
        (r#"var obj = {"\uD800": 1, a: 2}"#, r#"var obj = {a: 2, "\uD800": 1}"#),
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
        ("var obj = {a:1, b:3, [a]: -1, c:2}", "var obj = {a:1, b:3, [a]: -1, c:2}"),
        (
            r#"const S = Symbol("s"); export const obj6 = {a: 1, [S]: 3, b: 2};"#,
            r#"const S = Symbol("s"); export const obj6 = {a: 1, [S]: 3, b: 2};"#,
        ),
    ];

    Tester::new(SortKeys::NAME, SortKeys::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_natural_fix() {
    use crate::tester::Tester;
    let fix = vec![
        (
            "var obj = {a100:1, a12:2, a2:3}",
            "var obj = {a2:3, a12:2, a100:1}",
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
        (
            r#"var obj = {"a2": 2, "a10": 3, "a1\uD800": 1}"#,
            r#"var obj = {"a1\uD800": 1, "a2": 2, "a10": 3}"#,
            Some(serde_json::json!(["asc", { "natural": true }])),
        ),
    ];
    Tester::new::<&str>(SortKeys::NAME, SortKeys::PLUGIN, vec![], vec![]).expect_fix(fix).test();
}

/// Sorting the fix relies on a total order: an inconsistent comparator can panic in `sort_by`.
#[test]
fn natural_order_is_total() {
    use oxc_allocator::Allocator;
    use oxc_str::JSStrBuilder;
    let allocator = Allocator::new();
    let mut builder = JSStrBuilder::new_in(&&allocator);
    builder.push_utf16(&[0x61, 0x31, 0xD800]);
    let surrogate = builder.into_js_str();
    let names = [
        "",
        "0",
        "1",
        "01",
        "a",
        "a0",
        "a00",
        "a01",
        "a1",
        "a1b",
        "a01b",
        "a2",
        "a10",
        "a12",
        "a20",
        "a100",
        "A2",
        "a2a",
        "a2b",
        "a99999999999999999999",
        "a100000000000000000000",
        "_",
        "-",
        "\0a",
        "\0b",
        "a\0z",
    ]
    .into_iter()
    .map(JSStr::from)
    .chain([surrogate])
    .collect::<Vec<_>>();
    for case_sensitive in [true, false] {
        for &a in &names {
            for &b in &names {
                let ab = natural_compare(a, b, case_sensitive);
                assert_eq!(ab, natural_compare(b, a, case_sensitive).reverse(), "{a:?} {b:?}");
                for &c in &names {
                    if ab != Ordering::Greater
                        && natural_compare(b, c, case_sensitive) != Ordering::Greater
                    {
                        assert_ne!(
                            natural_compare(a, c, case_sensitive),
                            Ordering::Greater,
                            "{a:?} <= {b:?} <= {c:?}"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(natural_compare(surrogate, JSStr::from("a2"), true), Ordering::Less);
}
