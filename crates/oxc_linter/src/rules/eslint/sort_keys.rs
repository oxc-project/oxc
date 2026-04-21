use std::{cmp::Ordering, str::Chars};

use cow_utils::CowUtils;
use itertools::all;

use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
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

            let mut property_groups = collect_property_groups(dec, ctx.source_text(), options);

            if !options.case_sensitive {
                for group in &mut property_groups {
                    *group = group
                        .iter()
                        .map(|s| s.cow_to_ascii_lowercase().to_string())
                        .collect::<Vec<String>>();
                }
            }

            let mut sorted_property_groups = property_groups.clone();
            for group in &mut sorted_property_groups {
                if options.natural {
                    natural_sort(group);
                } else {
                    alphanumeric_sort(group);
                }

                if sort_order == &SortOrder::Desc {
                    group.reverse();
                }
            }

            let is_sorted =
                all(property_groups.iter().zip(&sorted_property_groups), |(a, b)| a == b);

            if !is_sorted {
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
}

struct FixableProperty {
    key: String,
    span: Span,
    text: String,
}

fn collect_property_groups(
    object: &ObjectExpression<'_>,
    source_text: &str,
    options: &SortKeysOptions,
) -> Vec<Vec<String>> {
    let mut property_groups: Vec<Vec<String>> = vec![vec![]];

    for (i, prop) in object.properties.iter().enumerate() {
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => {
                property_groups.push(vec!["<ellipsis_group>".into()]);
                property_groups.push(vec![]);
            }
            ObjectPropertyKind::ObjectProperty(obj) => {
                let Some(key) = obj.key.static_name() else { continue };
                if i != object.properties.len() - 1 && options.allow_line_separated_groups {
                    let text_between = extract_text_between_spans(
                        source_text,
                        prop.span(),
                        object.properties[i + 1].span(),
                    );
                    if text_between.contains("\n\n") {
                        property_groups.last_mut().unwrap().push(key.into());
                        property_groups.push(vec!["<linebreak_group>".into()]);
                        property_groups.push(vec![]);
                        continue;
                    }
                }
                property_groups.last_mut().unwrap().push(key.into());
            }
        }
    }

    property_groups
}

fn build_object_fix<'a>(
    object: &'a ObjectExpression<'a>,
    ctx: &LintContext<'a>,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Option<(Span, String)> {
    let props = collect_fixable_properties(object, ctx, sort_order, options)?;
    let indices = sorted_property_indices(&props, sort_order, options);
    let has_nested_fix = props.iter().any(|prop| prop.text.as_str() != ctx.source_range(prop.span));
    let needs_reordering = indices.iter().enumerate().any(|(position, index)| position != *index);

    if !needs_reordering && !has_nested_fix {
        return None;
    }

    let mut separators: Vec<String> = Vec::with_capacity(props.len());
    for i in 0..props.len() {
        if i + 1 < props.len() {
            let sep_start = props[i].span.end;
            let sep_end = props[i + 1].span.start;
            separators.push(ctx.source_range(Span::new(sep_start, sep_end)).to_string());
        } else {
            separators.push(String::new());
        }
    }

    let mut sorted_text = String::new();
    for (position, &index) in indices.iter().enumerate() {
        sorted_text.push_str(&props[index].text);

        if position + 1 < indices.len() {
            let separator = if position < separators.len() && !separators[position].is_empty() {
                separators[position].as_str()
            } else {
                ", "
            };
            sorted_text.push_str(separator);
        }
    }

    Some((Span::new(props[0].span.start, props[props.len() - 1].span.end), sorted_text))
}

fn collect_fixable_properties<'a>(
    object: &'a ObjectExpression<'a>,
    ctx: &LintContext<'a>,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Option<Vec<FixableProperty>> {
    enum SpreadPos {
        Start,
        CanEnd,
        End,
    }

    let property_groups = collect_property_groups(object, ctx.source_text(), options);
    let static_groups_count = property_groups
        .iter()
        .filter(|group| !group.is_empty() && !group.iter().any(|key| key.starts_with('<')))
        .count();

    if static_groups_count != 1 {
        return None;
    }

    let mut spread_pos = SpreadPos::Start;
    let mut props = Vec::with_capacity(object.properties.len());

    for (i, prop) in object.properties.iter().enumerate() {
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => {
                if let Some(next_prop) = object.properties.get(i + 1)
                    && let ObjectPropertyKind::ObjectProperty(_) = next_prop
                    && ctx.has_comments_between(Span::new(prop.span().end, next_prop.span().start))
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
                    key: key.to_string(),
                    span: prop.span(),
                    text: build_property_text(obj, ctx, sort_order, options),
                });

                if i + 1 < object.properties.len() {
                    let next_span = object.properties[i + 1].span();
                    let between = Span::new(prop.span().end, next_span.start);
                    if ctx.has_comments_between(between) {
                        return None;
                    }
                }
            }
        }
    }

    (!props.is_empty()).then_some(props)
}

fn sorted_property_indices(
    props: &[FixableProperty],
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> Vec<usize> {
    let keys_for_cmp: Vec<String> = props
        .iter()
        .map(|prop| {
            if options.case_sensitive {
                prop.key.clone()
            } else {
                prop.key.cow_to_ascii_lowercase().to_string()
            }
        })
        .collect();

    let mut sorted_keys = keys_for_cmp.clone();
    if options.natural {
        natural_sort(&mut sorted_keys);
    } else {
        alphanumeric_sort(&mut sorted_keys);
    }
    if sort_order == &SortOrder::Desc {
        sorted_keys.reverse();
    }

    let mut used = vec![false; keys_for_cmp.len()];
    let mut indices: Vec<usize> = Vec::with_capacity(keys_for_cmp.len());

    for sorted_key in &sorted_keys {
        if let Some(position) = keys_for_cmp
            .iter()
            .enumerate()
            .find(|(index, key)| !used[*index] && key.as_str() == sorted_key.as_str())
            .map(|(index, _)| index)
        {
            used[position] = true;
            indices.push(position);
        }
    }

    indices
}

fn build_property_text<'a>(
    property: &'a ObjectProperty<'a>,
    ctx: &LintContext<'a>,
    sort_order: &SortOrder,
    options: &SortKeysOptions,
) -> String {
    let property_text = ctx.source_range(property.span).to_string();
    let Expression::ObjectExpression(object) = &property.value else {
        return property_text;
    };
    let Some((replace_span, replacement)) = build_object_fix(object, ctx, sort_order, options)
    else {
        return property_text;
    };

    let before_value =
        ctx.source_range(Span::new(property.span.start, replace_span.start)).to_string();
    let after_value = ctx.source_range(Span::new(replace_span.end, property.span.end)).to_string();

    format!("{before_value}{replacement}{after_value}")
}

fn alphanumeric_sort(arr: &mut [String]) {
    arr.sort_unstable();
}

fn natural_sort(arr: &mut [String]) {
    arr.sort_unstable_by(|a, b| {
        let mut a_chars = a.chars();
        let mut b_chars = b.chars();

        loop {
            match (a_chars.next(), b_chars.next()) {
                (Some(a_char), Some(b_char)) if a_char == b_char => {}
                (Some(a_char), Some(b_char))
                    if a_char.is_ascii_digit() && b_char.is_ascii_digit() =>
                {
                    let n1 = take_numeric(&mut a_chars, a_char);
                    let n2 = take_numeric(&mut b_chars, b_char);
                    match n1.cmp(&n2) {
                        Ordering::Equal => {}
                        ord => return ord,
                    }
                }
                (Some(a_char), Some(b_char))
                    if a_char.is_alphanumeric() && !b_char.is_alphanumeric() =>
                {
                    return Ordering::Greater;
                }
                (Some(a_char), Some(b_char))
                    if !a_char.is_alphanumeric() && b_char.is_alphanumeric() =>
                {
                    return Ordering::Less;
                }
                (Some(a_char), Some(b_char)) if a_char == '[' && b_char.is_alphanumeric() => {
                    return Ordering::Greater;
                }
                (Some(a_char), Some(b_char)) if a_char.is_alphanumeric() && b_char == '[' => {
                    return Ordering::Less;
                }
                (Some(a_char), Some(b_char)) => return a_char.cmp(&b_char),
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
            }
        }
    });
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
        // spellchecker:on
    ];

    Tester::new(SortKeys::NAME, SortKeys::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
