use std::{collections::HashMap, hash::Hash};

use oxc_ast::{
    ast::{Argument, BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use regex::Regex;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(valid-title): {0:?}")]
#[diagnostic(severity(warning), help("{1:?}"))]
struct ValidTitleDiagnostic(CompactStr, &'static str, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<ValidTitleConfig>);

#[derive(Debug, Default, Clone)]
pub struct ValidTitleConfig {
    ignore_type_of_describe_name: bool,
    disallowed_words: Vec<String>,
    ignore_space: bool,
    must_not_match_patterns: HashMap<MatchKind, CompiledMatcherAndMessage>,
    must_match_patterns: HashMap<MatchKind, CompiledMatcherAndMessage>,
}

impl std::ops::Deref for ValidTitle {
    type Target = ValidTitleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks that the title of Jest blocks are valid by ensuring that titles are:
    ///
    /// - not empty,
    /// - is a string,
    /// - not prefixed with their block name,
    /// - have no leading or trailing spaces
    ///
    /// ### Example
    /// ```javascript
    /// describe('', () => {});
    /// describe('foo', () => {
    ///   it('', () => {});
    /// });
    /// it('', () => {});
    /// test('', () => {});
    /// xdescribe('', () => {});
    /// xit('', () => {});
    /// xtest('', () => {});
    /// ```
    ValidTitle,
    correctness
);

impl Rule for ValidTitle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        let get_as_bool = |name: &str| -> bool {
            config
                .and_then(|v| v.get(name))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default()
        };

        let ignore_type_of_describe_name = get_as_bool("ignoreTypeOfDescribeName");
        let ignore_space = get_as_bool("ignoreSpaces");
        let disallowed_words = config
            .and_then(|v| v.get("disallowedWords"))
            .and_then(|v| v.as_array())
            .map(|v| {
                v.iter().filter_map(|v| v.as_str().map(std::string::ToString::to_string)).collect()
            })
            .unwrap_or_default();
        let must_not_match_patterns = config
            .and_then(|v| v.get("mustNotMatch"))
            .and_then(compile_matcher_patterns)
            .unwrap_or_default();
        let must_match_patterns = config
            .and_then(|v| v.get("mustMatch"))
            .and_then(compile_matcher_patterns)
            .unwrap_or_default();
        Self(Box::new(ValidTitleConfig {
            ignore_type_of_describe_name,
            disallowed_words,
            ignore_space,
            must_not_match_patterns,
            must_match_patterns,
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        for node in &collect_possible_jest_call_node(ctx) {
            self.run(node, ctx);
        }
    }
}

impl ValidTitle {
    fn run<'a>(&self, possible_jest_fn_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_fn_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_fn_node, ctx)
        else {
            return;
        };

        if !matches!(
            jest_fn_call.kind,
            JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)
        ) {
            return;
        }

        let Some(Argument::Expression(expr)) = call_expr.arguments.first() else {
            return;
        };

        let need_report_describe_name = !(self.ignore_type_of_describe_name
            && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)));

        match expr {
            Expression::StringLiteral(string_literal) => {
                validate_title(
                    &string_literal.value,
                    string_literal.span,
                    self,
                    &jest_fn_call.name,
                    ctx,
                );
            }
            Expression::TemplateLiteral(template_literal) => {
                if !template_literal.is_no_substitution_template() {
                    return;
                }
                if let Some(quasi) = template_literal.quasi() {
                    validate_title(
                        quasi.as_str(),
                        template_literal.span,
                        self,
                        &jest_fn_call.name,
                        ctx,
                    );
                }
            }
            Expression::BinaryExpression(binary_expr) => {
                if does_binary_expression_contain_string_node(binary_expr) {
                    return;
                }
                if need_report_describe_name {
                    Message::TitleMustBeString.diagnostic(ctx, expr.span());
                }
            }
            _ => {
                if need_report_describe_name {
                    Message::TitleMustBeString.diagnostic(ctx, expr.span());
                }
            }
        }
    }
}

type CompiledMatcherAndMessage = (Regex, Option<String>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum MatchKind {
    Describe,
    It,
    Test,
}

#[derive(Copy, Clone)]
enum MatcherPattern<'a> {
    String(&'a serde_json::Value),
    Vec(&'a Vec<serde_json::Value>),
}

impl MatchKind {
    fn from(name: &str) -> Option<Self> {
        match name {
            "describe" => Some(Self::Describe),
            "it" => Some(Self::It),
            "test" => Some(Self::Test),
            _ => None,
        }
    }
}

fn compile_matcher_patterns(
    matcher_patterns: &serde_json::Value,
) -> Option<HashMap<MatchKind, CompiledMatcherAndMessage>> {
    matcher_patterns
        .as_array()
        .map_or_else(
            || {
                // for `{ "describe": "/pattern/" }`
                let obj = matcher_patterns.as_object()?;
                let mut map: HashMap<MatchKind, CompiledMatcherAndMessage> = HashMap::new();
                for (key, value) in obj {
                    let Some(v) = compile_matcher_pattern(MatcherPattern::String(value)) else {
                        continue;
                    };
                    if let Some(kind) = MatchKind::from(key) {
                        map.insert(kind, v);
                    }
                }

                Some(map)
            },
            |value| {
                // for `["/pattern/", "message"]`
                let mut map: HashMap<MatchKind, CompiledMatcherAndMessage> = HashMap::new();
                let v = &compile_matcher_pattern(MatcherPattern::Vec(value))?;
                map.insert(MatchKind::Describe, v.clone());
                map.insert(MatchKind::Test, v.clone());
                map.insert(MatchKind::It, v.clone());
                Some(map)
            },
        )
        .map_or_else(
            || {
                // for `"/pattern/"`
                let string = matcher_patterns.as_str()?;
                let mut map: HashMap<MatchKind, CompiledMatcherAndMessage> = HashMap::new();
                let v = &compile_matcher_pattern(MatcherPattern::String(
                    &serde_json::Value::String(string.to_string()),
                ))?;
                map.insert(MatchKind::Describe, v.clone());
                map.insert(MatchKind::Test, v.clone());
                map.insert(MatchKind::It, v.clone());
                Some(map)
            },
            Some,
        )
}

fn compile_matcher_pattern(pattern: MatcherPattern) -> Option<CompiledMatcherAndMessage> {
    match pattern {
        MatcherPattern::String(pattern) => {
            let reg_str = format!("(?u){}", pattern.as_str()?);
            let reg = Regex::new(&reg_str).ok()?;
            Some((reg, None))
        }
        MatcherPattern::Vec(pattern) => {
            let reg_str = pattern.first().and_then(|v| v.as_str()).map(|v| format!("(?u){v}"))?;
            let reg = Regex::new(&reg_str).ok()?;
            let message = pattern.get(1).map(std::string::ToString::to_string);
            Some((reg, message))
        }
    }
}

fn validate_title(
    title: &str,
    span: Span,
    valid_title: &ValidTitle,
    name: &str,
    ctx: &LintContext,
) {
    if title.is_empty() {
        Message::EmptyTitle.diagnostic(ctx, span);
    }

    if !valid_title.disallowed_words.is_empty() {
        let Ok(disallowed_words_reg) = regex::Regex::new(&format!(
            r#"(?iu)\b(?:{})\b"#,
            valid_title.disallowed_words.join("|").replace('.', r"\.")
        )) else {
            return;
        };

        if let Some(matched) = disallowed_words_reg.find(title) {
            let error = format!("{} is not allowed in test title", matched.as_str());
            ctx.diagnostic(ValidTitleDiagnostic(
                CompactStr::from(error),
                "It is included in the `disallowedWords` of your config file, try to remove it from your title",
                span,
            ));
        }
        return;
    }

    // TODO: support fixer
    if !valid_title.ignore_space && title.trim() != title {
        Message::AccidentalSpace.diagnostic(ctx, span);
    }

    let un_prefixed_name = name.trim_start_matches(['f', 'x']);
    let Some(first_word) = title.split(' ').next() else {
        return;
    };

    // TODO: support fixer
    if first_word == un_prefixed_name {
        Message::DuplicatePrefix.diagnostic(ctx, span);
        return;
    }

    let Some(jest_fn_name) = MatchKind::from(un_prefixed_name) else {
        return;
    };

    if let Some((regex, message)) = valid_title.must_match_patterns.get(&jest_fn_name) {
        if !regex.is_match(title) {
            let raw_pattern = regex.as_str();
            let message = message.as_ref().map_or_else(
                || CompactStr::from(format!("{un_prefixed_name} should match {raw_pattern}")),
                |message| CompactStr::from(message.as_str()),
            );
            ctx.diagnostic(ValidTitleDiagnostic(
                message,
                "Make sure the title matches the `mustMatch` of your config file",
                span,
            ));
        }
    }

    if let Some((regex, message)) = valid_title.must_not_match_patterns.get(&jest_fn_name) {
        if regex.is_match(title) {
            let raw_pattern = regex.as_str();
            let message = message.as_ref().map_or_else(
                || CompactStr::from(format!("{un_prefixed_name} should not match {raw_pattern}")),
                |message| CompactStr::from(message.as_str()),
            );

            ctx.diagnostic(ValidTitleDiagnostic(
                message,
                "Make sure the title not matches the `mustNotMatch` of your config file",
                span,
            ));
        }
    }
}

fn does_binary_expression_contain_string_node(expr: &BinaryExpression) -> bool {
    if expr.left.is_string_literal() || expr.right.is_string_literal() {
        return true;
    }

    match &expr.left {
        Expression::BinaryExpression(left) => does_binary_expression_contain_string_node(left),
        _ => false,
    }
}

enum Message {
    TitleMustBeString,
    EmptyTitle,
    DuplicatePrefix,
    AccidentalSpace,
}

impl Message {
    fn detail(&self) -> (&'static str, &'static str) {
        match self {
            Self::TitleMustBeString => ("Title must be a string", "Replace your title with a string"),
            Self::EmptyTitle => ("Should not have an empty title", "Write a meaningful title for your test"),
            Self::DuplicatePrefix => ("Should not have duplicate prefix", "The function name has already contains the prefix, try remove the duplicate prefix"),
            Self::AccidentalSpace => ("Should not have leading or trailing spaces", "Remove the leading or trailing spaces"),
        }
    }
    fn diagnostic(&self, ctx: &LintContext, span: Span) {
        let (error, help) = self.detail();
        ctx.diagnostic(ValidTitleDiagnostic(CompactStr::from(error), help, span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("describe('the correct way to properly handle all the things', () => {});", None),
        ("test('that all is as it should be', () => {});", None),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([
              { "ignoreTypeOfDescribeName": false, "disallowedWords": ["correct"] },
            ])),
        ),
        ("it('correctly sets the value', () => {});", Some(serde_json::json!([]))),
        ("describe('the correct way to properly handle all the things', () => {});", None),
        ("test('that all is as it should be', () => {});", None),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": {} }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": " " }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": [" "] }])),
        ),
        (
            "it('correctly sets the value #unit', () => {});",
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))" }])),
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": { "test": "#(?:unit|integration|e2e)" } }])),
        ),
        (
            "
            describe('things to test', () => {
                describe('unit tests #unit', () => {
                it('is true', () => {
                    expect(true).toBe(true);
                });
                });

                describe('e2e tests #e2e', () => {
                it('is another test #jest4life', () => {});
                });
            });
            ",
            Some(serde_json::json!([{ "mustMatch": { "test": "^[^#]+$|(?:#(?:unit|e2e))" } }])),
        ),
        ("it('is a string', () => {});", None),
        ("it('is' + ' a ' + ' string', () => {});", None),
        ("it(1 + ' + ' + 1, () => {});", None),
        ("test('is a string', () => {});", None),
        ("xtest('is a string', () => {});", None),
        ("xtest(`${myFunc} is a string`, () => {});", None),
        ("describe('is a string', () => {});", None),
        ("describe.skip('is a string', () => {});", None),
        ("describe.skip(`${myFunc} is a string`, () => {});", None),
        ("fdescribe('is a string', () => {});", None),
        (
            "describe(String(/.+/), () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
        ),
        (
            "describe(myFunction, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
        ),
        (
            "xdescribe(skipFunction, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true, "disallowedWords": [] }])),
        ),
        ("describe()", None),
        ("someFn('', function () {})", None),
        ("describe('foo', function () {})", None),
        ("describe('foo', function () { it('bar', function () {}) })", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("test(`foo`, function () {})", None),
        ("test.concurrent(`foo`, function () {})", None),
        ("test(`${foo}`, function () {})", None),
        ("test.concurrent(`${foo}`, function () {})", None),
        ("it('foo', function () {})", None),
        ("it.each([])()", None),
        ("it.concurrent('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("it()", None),
        ("it.concurrent()", None),
        ("describe()", None),
        ("it.each()()", None),
        ("describe('foo', function () {})", None),
        ("fdescribe('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("it('foo', function () {})", None),
        ("it.concurrent('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("fit.concurrent('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("test.concurrent('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xtest(`foo`, function () {})", None),
        ("someFn('foo', function () {})", None),
        (
            "
                describe('foo', () => {
                it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "it(`GIVEN...
            `, () => {});",
            Some(serde_json::json!([{ "ignoreSpaces": true }])),
        ),
        ("describe('foo', function () {})", None),
        ("fdescribe('foo', function () {})", None),
        ("xdescribe('foo', function () {})", None),
        ("xdescribe(`foo`, function () {})", None),
        ("test('foo', function () {})", None),
        ("test('foo', function () {})", None),
        ("xtest('foo', function () {})", None),
        ("xtest(`foo`, function () {})", None),
        ("test('foo test', function () {})", None),
        ("xtest('foo test', function () {})", None),
        ("it('foo', function () {})", None),
        ("fit('foo', function () {})", None),
        ("xit('foo', function () {})", None),
        ("xit(`foo`, function () {})", None),
        ("it('foos it correctly', function () {})", None),
        (
            "
                describe('foo', () => {
                it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                it('describes things correctly', () => {})
                })
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "test('the correct way to properly handle all things', () => {});",
            Some(serde_json::json!([{ "disallowedWords": ["correct", "properly", "all"] }])),
        ),
        (
            "describe('the correct way to do things', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
        ),
        (
            "it('has ALL the things', () => {})",
            Some(serde_json::json!([{ "disallowedWords": ["all"] }])),
        ),
        (
            "xdescribe('every single one of them', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["every"] }])),
        ),
        (
            "describe('Very Descriptive Title Goes Here', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["descriptive"] }])),
        ),
        (
            "test(`that the value is set properly`, function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["properly"] }])),
        ),
        // TODO: The regex `(?:#(?!unit|e2e))\w+` in those test cases is not valid in Rust
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //         {
        //             "mustNotMatch": r#"(?:#(?!unit|e2e))\w+"#,
        //             "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))",
        //         },
        //     ])),
        // ),
        // (
        //     "
        //         import { describe, describe as context, it as thisTest } from '@jest/globals';

        //         describe('things to test', () => {
        //             context('unit tests #unit', () => {
        //             thisTest('is true', () => {
        //                 expect(true).toBe(true);
        //             });
        //             });

        //             context('e2e tests #e4e', () => {
        //                 thisTest('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //         ",
        //     Some(
        //         serde_json::json!([ { "mustNotMatch": r#"(?:#(?!unit|e2e))\w+"#, "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))", }, ]),
        //     ),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": [
        //           r#"(?:#(?!unit|e2e))\w+"#,
        //           "Please include '#unit' or '#e2e' in titles",
        //         ],
        //         "mustMatch": [
        //           "^[^#]+$|(?:#(?:unit|e2e))",
        //           "Please include '#unit' or '#e2e' in titles",
        //         ],
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": { "describe": [r#"(?:#(?!unit|e2e))\w+"#] },
        //         "mustMatch": { "describe": "^[^#]+$|(?:#(?:unit|e2e))" },
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //                 it('is true', () => {
        //                     expect(true).toBe(true);
        //                 });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": {
        //           "describe": [
        //             r#"(?:#(?!unit|e2e))\w+"#,
        //             "Please include '#unit' or '#e2e' in describe titles",
        //           ],
        //         },
        //         "mustMatch": { "describe": "^[^#]+$|(?:#(?:unit|e2e))" },
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //             it('is true', () => {
        //                 expect(true).toBe(true);
        //             });
        //             });

        //             describe('e2e tests #e4e', () => {
        //                 it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": { "describe": r#"(?:#(?!unit|e2e))\w+"# },
        //         "mustMatch": { "it": "^[^#]+$|(?:#(?:unit|e2e))" },
        //       },
        //     ])),
        // ),
        // (
        //     "
        //         describe('things to test', () => {
        //             describe('unit tests #unit', () => {
        //             it('is true #jest4life', () => {
        //                 expect(true).toBe(true);
        //             });
        //             });

        //             describe('e2e tests #e4e', () => {
        //             it('is another test #e2e #jest4life', () => {});
        //             });
        //         });
        //     ",
        //     Some(serde_json::json!([
        //       {
        //         "mustNotMatch": {
        //           "describe": [
        //             r#"(?:#(?!unit|e2e))\w+"#,
        //             "Please include '#unit' or '#e2e' in describe titles",
        //           ],
        //         },
        //         "mustMatch": {
        //           "it": [
        //             "^[^#]+$|(?:#(?:unit|e2e))",
        //             "Please include '#unit' or '#e2e' in it titles",
        //           ],
        //         },
        //       },
        //     ])),
        // ),
        (
            "test('the correct way to properly handle all things', () => {});",
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
        ),
        (
            "describe('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
        ),
        (
            "xdescribe('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
        ),
        (
            "describe.skip('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
        ),
        ("it.each([])(1, () => {});", None),
        ("it.skip.each([])(1, () => {});", None),
        ("it.skip.each``(1, () => {});", None),
        ("it(123, () => {});", None),
        ("it.concurrent(123, () => {});", None),
        ("it(1 + 2 + 3, () => {});", None),
        ("it.concurrent(1 + 2 + 3, () => {});", None),
        (
            "test.skip(123, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
        ),
        ("describe(String(/.+/), () => {});", None),
        (
            "describe(myFunction, () => 1);",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": false }])),
        ),
        ("describe(myFunction, () => {});", None),
        ("xdescribe(myFunction, () => {});", None),
        ("describe(6, function () {})", None),
        ("describe.skip(123, () => {});", None),
        ("describe('', function () {})", None),
        (
            "
                describe('foo', () => {
                    it('', () => {});
                });
            ",
            None,
        ),
        ("it('', function () {})", None),
        ("it.concurrent('', function () {})", None),
        ("test('', function () {})", None),
        ("test.concurrent('', function () {})", None),
        ("test(``, function () {})", None),
        ("test.concurrent(``, function () {})", None),
        ("xdescribe('', () => {})", None),
        ("xit('', () => {})", None),
        ("xtest('', () => {})", None),
        ("describe(' foo', function () {})", None),
        ("describe.each()(' foo', function () {})", None),
        ("describe.only.each()(' foo', function () {})", None),
        ("describe(' foo foe fum', function () {})", None),
        ("describe('foo foe fum ', function () {})", None),
        ("fdescribe(' foo', function () {})", None),
        ("fdescribe(' foo', function () {})", None),
        ("xdescribe(' foo', function () {})", None),
        ("it(' foo', function () {})", None),
        ("it.concurrent(' foo', function () {})", None),
        ("fit(' foo', function () {})", None),
        ("it.skip(' foo', function () {})", None),
        ("fit('foo ', function () {})", None),
        ("it.skip('foo ', function () {})", None),
        (
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works ', () => {});
            ",
            None,
        ),
        ("xit(' foo', function () {})", None),
        ("test(' foo', function () {})", None),
        ("test.concurrent(' foo', function () {})", None),
        ("test(` foo`, function () {})", None),
        ("test.concurrent(` foo`, function () {})", None),
        ("test(` foo bar bang`, function () {})", None),
        ("test.concurrent(` foo bar bang`, function () {})", None),
        ("test(` foo bar bang  `, function () {})", None),
        ("test.concurrent(` foo bar bang  `, function () {})", None),
        ("xtest(' foo', function () {})", None),
        ("xtest(' foo  ', function () {})", None),
        (
            "
                describe(' foo', () => {
                    it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    it(' bar', () => {})
                })
            ",
            None,
        ),
        ("describe('describe foo', function () {})", None),
        ("fdescribe('describe foo', function () {})", None),
        ("xdescribe('describe foo', function () {})", None),
        ("describe('describe foo', function () {})", None),
        ("fdescribe(`describe foo`, function () {})", None),
        ("test('test foo', function () {})", None),
        ("xtest('test foo', function () {})", None),
        ("test(`test foo`, function () {})", None),
        ("test(`test foo test`, function () {})", None),
        ("it('it foo', function () {})", None),
        ("fit('it foo', function () {})", None),
        ("xit('it foo', function () {})", None),
        ("it('it foos it correctly', function () {})", None),
        (
            "
                describe('describe foo', () => {
                    it('bar', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('describe foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
            None,
        ),
        (
            "
                describe('foo', () => {
                    it('it bar', () => {})
                })
            ",
            None,
        ),
    ];

    Tester::new(ValidTitle::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
