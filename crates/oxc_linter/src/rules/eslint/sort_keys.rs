use std::cmp::Ordering;
use std::str::Chars;

use cow_utils::CowUtils;
use itertools::all;

use oxc_ast::ast::ObjectPropertyKind;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct SortKeys(Box<SortKeysOptions>);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum SortOrder {
    Desc,
    #[default]
    Asc,
}

#[derive(Debug, Clone)]
pub struct SortKeysOptions {
    sort_order: SortOrder,
    case_sensitive: bool,
    natural: bool,
    min_keys: usize,
    allow_line_separated_groups: bool,
}

impl Default for SortKeysOptions {
    fn default() -> Self {
        // we follow the eslint defaults
        Self {
            sort_order: SortOrder::Asc,
            case_sensitive: true,
            natural: false,
            min_keys: 2,
            allow_line_separated_groups: false,
        }
    }
}

impl std::ops::Deref for SortKeys {
    type Target = SortKeysOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn sort_properties_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Object keys should be sorted").with_label(span0)
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
    style,
    pending
);

impl Rule for SortKeys {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(config_array) = value.as_array() else {
            return Self::default();
        };

        let sort_order = if config_array.is_empty() {
            SortOrder::Asc
        } else {
            config_array[0].as_str().map_or(SortOrder::Asc, |s| match s {
                "desc" => SortOrder::Desc,
                _ => SortOrder::Asc,
            })
        };

        let config = if config_array.len() > 1 {
            config_array[1].as_object().unwrap()
        } else {
            &serde_json::Map::new()
        };

        let case_sensitive =
            config.get("caseSensitive").and_then(serde_json::Value::as_bool).unwrap_or(true);
        let natural = config.get("natural").and_then(serde_json::Value::as_bool).unwrap_or(false);
        let min_keys = config
            .get("minKeys")
            .and_then(serde_json::Value::as_u64)
            .map_or(2, |n| n.try_into().unwrap_or(2));
        let allow_line_separated_groups = config
            .get("allowLineSeparatedGroups")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self(Box::new(SortKeysOptions {
            sort_order,
            case_sensitive,
            natural,
            min_keys,
            allow_line_separated_groups,
        }))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ObjectExpression(dec) = node.kind() {
            if dec.properties.len() < self.min_keys {
                return;
            }

            let mut property_groups: Vec<Vec<String>> = vec![vec![]];

            let source_text = ctx.semantic().source_text();

            for (i, prop) in dec.properties.iter().enumerate() {
                match prop {
                    ObjectPropertyKind::SpreadProperty(_) => {
                        property_groups.push(vec!["<ellipsis_group>".into()]);
                        property_groups.push(vec![]);
                    }
                    ObjectPropertyKind::ObjectProperty(obj) => {
                        let Some(key) = obj.key.static_name() else { continue };
                        if i != dec.properties.len() - 1 && self.allow_line_separated_groups {
                            let text_between = extract_text_between_spans(
                                source_text,
                                prop.span(),
                                dec.properties[i + 1].span(),
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

            if !self.case_sensitive {
                for group in &mut property_groups {
                    *group = group
                        .iter()
                        .map(|s| s.cow_to_lowercase().to_string())
                        .collect::<Vec<String>>();
                }
            }

            let mut sorted_property_groups = property_groups.clone();
            for group in &mut sorted_property_groups {
                if self.natural {
                    natural_sort(group);
                } else {
                    alphanumeric_sort(group);
                }

                if self.sort_order == SortOrder::Desc {
                    group.reverse();
                }
            }

            let is_sorted =
                all(property_groups.iter().zip(&sorted_property_groups), |(a, b)| a == b);

            if !is_sorted {
                ctx.diagnostic(sort_properties_diagnostic(node.span()));
            }
        }
    }
}

fn alphanumeric_sort(arr: &mut [String]) {
    arr.sort_unstable();
}

fn natural_sort(arr: &mut [String]) {
    arr.sort_by(|a, b| {
        let mut a_chars = a.chars();
        let mut b_chars = b.chars();

        loop {
            match (a_chars.next(), b_chars.next()) {
                (Some(a_char), Some(b_char)) if a_char == b_char => continue,
                (Some(a_char), Some(b_char)) if a_char.is_numeric() && b_char.is_numeric() => {
                    let n1 = take_numeric(&mut a_chars, a_char);
                    let n2 = take_numeric(&mut b_chars, b_char);
                    match n1.cmp(&n2) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                (Some(a_char), Some(b_char))
                    if a_char.is_alphanumeric() && !b_char.is_alphanumeric() =>
                {
                    return Ordering::Greater
                }
                (Some(a_char), Some(b_char))
                    if !a_char.is_alphanumeric() && b_char.is_alphanumeric() =>
                {
                    return Ordering::Less
                }
                (Some(a_char), Some(b_char)) if a_char == '[' && b_char.is_alphanumeric() => {
                    return Ordering::Greater
                }
                (Some(a_char), Some(b_char)) if a_char.is_alphanumeric() && b_char == '[' => {
                    return Ordering::Less
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

    Tester::new(SortKeys::NAME, pass, fail).test_and_snapshot();
}
