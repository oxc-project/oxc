use std::hash::Hash;

use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn valid_title_diagnostic(x0: &str, x1: &str, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{x0:?}")).with_help(format!("{x1:?}")).with_label(span2)
}

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<ValidTitleConfig>);

#[derive(Debug, Default, Clone)]
pub struct ValidTitleConfig {
    ignore_type_of_describe_name: bool,
    disallowed_words: Vec<CompactStr>,
    allow_arguments: bool,
    must_not_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
    must_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
}

impl std::ops::Deref for ValidTitle {
    type Target = ValidTitleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks that the titles of Vitest blocks are valid.
    ///
    /// Titles must be:
    /// - not empty,
    /// - strings,
    /// - not prefixed with their block name,
    /// - have no leading or trailing spaces.
    ///
    /// ### Why is this bad?
    ///
    /// Titles that are not valid can be misleading and make it harder to understand the purpose of the test.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
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
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// describe('foo', () => {});
    /// it('bar', () => {});
    /// test('baz', () => {});
    /// ```
    ///
    /// ### Options
    ///
    /// interface Options {
    ///     ignoreTypeOfDescribeName?: boolean;
    ///     disallowedWords?: string[];
    ///     allowArguments?: boolean;
    ///     mustNotMatch?: Partial<Record<'describe' | 'test' | 'it', string>> | string;
    ///     mustMatch?: Partial<Record<'describe' | 'test' | 'it', string>> | string;
    /// }
    ValidTitle,
    vitest,
    correctness,
    conditional_fix,
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
        let allow_arguments = get_as_bool("allowArguments");
        let disallowed_words = config
            .and_then(|v| v.get("disallowedWords"))
            .and_then(|v| v.as_array())
            .map(|v| v.iter().filter_map(|v| v.as_str().map(CompactStr::from)).collect())
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
            allow_arguments,
            must_not_match_patterns,
            must_match_patterns,
        }))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.run(jest_node, ctx);
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

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        let need_report_name = match jest_fn_call.kind {
            JestFnKind::General(JestGeneralFnKind::Describe) => !self.ignore_type_of_describe_name,
            JestFnKind::General(JestGeneralFnKind::Test) => true,
            _ => unreachable!(),
        };

        match arg {
            Argument::StringLiteral(string_literal) => {
                validate_title(
                    &string_literal.value,
                    string_literal.span,
                    self,
                    &jest_fn_call.name,
                    ctx,
                );
            }
            Argument::TemplateLiteral(template_literal) => {
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
            Argument::BinaryExpression(binary_expr) => {
                if does_binary_expression_contain_string_node(binary_expr) {
                    return;
                }
                if need_report_name && !self.allow_arguments {
                    Message::TitleMustBeString.diagnostic(ctx, arg.span());
                }
            }
            _ => {
                if need_report_name && !self.allow_arguments {
                    Message::TitleMustBeString.diagnostic(ctx, arg.span());
                }
            }
        }
    }
}

type CompiledMatcherAndMessage = (Regex, Option<CompactStr>);

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
) -> Option<FxHashMap<MatchKind, CompiledMatcherAndMessage>> {
    matcher_patterns
        .as_array()
        .map_or_else(
            || {
                // for `{ "describe": "/pattern/" }`
                let obj = matcher_patterns.as_object()?;
                let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
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
                let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
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
                let mut map: FxHashMap<MatchKind, CompiledMatcherAndMessage> = FxHashMap::default();
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
            let message = pattern.get(1).and_then(serde_json::Value::as_str).map(CompactStr::from);
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
        let Ok(disallowed_words_reg) = Regex::new(&format!(
            r"(?iu)\b(?:{})\b",
            valid_title.disallowed_words.join("|").cow_replace('.', r"\.")
        )) else {
            return;
        };

        if let Some(matched) = disallowed_words_reg.find(title) {
            let error = format!("{} is not allowed in test title", matched.as_str());
            ctx.diagnostic(valid_title_diagnostic(
                &error,
                "It is included in the `disallowedWords` of your config file, try to remove it from your title",
                span,
            ));
        }
        return;
    }

    let trimmed_title = title.trim();
    if trimmed_title != title {
        let (error, help) = Message::AccidentalSpace.detail();
        ctx.diagnostic_with_fix(valid_title_diagnostic(error, help, span), |fixer| {
            fixer.replace(span.shrink(1), trimmed_title.to_string())
        });
    }

    let un_prefixed_name = name.trim_start_matches(['f', 'x']);
    let Some(first_word) = title.split(' ').next() else {
        return;
    };

    if first_word == un_prefixed_name {
        let (error, help) = Message::DuplicatePrefix.detail();
        ctx.diagnostic_with_fix(valid_title_diagnostic(error, help, span), |fixer| {
            let replaced_title = title[first_word.len()..].trim().to_string();
            fixer.replace(span.shrink(1), replaced_title)
        });
        return;
    }

    let Some(jest_fn_name) = MatchKind::from(un_prefixed_name) else {
        return;
    };

    if let Some((regex, message)) = valid_title.must_match_patterns.get(&jest_fn_name) {
        if !regex.is_match(title) {
            let raw_pattern = regex.as_str();
            let message = match message.as_ref() {
                Some(message) => message.as_str(),
                None => &format!("{un_prefixed_name} should match {raw_pattern}"),
            };
            ctx.diagnostic(valid_title_diagnostic(
                message,
                "Make sure the title matches the `mustMatch` of your config file",
                span,
            ));
        }
    }

    if let Some((regex, message)) = valid_title.must_not_match_patterns.get(&jest_fn_name) {
        if regex.is_match(title) {
            let raw_pattern = regex.as_str();
            let message = match message.as_ref() {
                Some(message) => message.as_str(),
                None => &format!("{un_prefixed_name} should not match {raw_pattern}"),
            };

            ctx.diagnostic(valid_title_diagnostic(
                message,
                "Make sure the title does not match the `mustNotMatch` of your config file",
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
            Self::TitleMustBeString => {
                ("Title must be a string", "Replace your title with a string")
            }
            Self::EmptyTitle => {
                ("Should not have an empty title", "Write a meaningful title for your test")
            }
            Self::DuplicatePrefix => (
                "Should not have duplicate prefix",
                "The function name already contains the prefix, try removing the duplicate prefix",
            ),
            Self::AccidentalSpace => (
                "Should not have leading or trailing spaces",
                "Remove the leading or trailing spaces",
            ),
        }
    }

    fn diagnostic(&self, ctx: &LintContext, span: Span) {
        let (error, help) = self.detail();
        ctx.diagnostic(valid_title_diagnostic(error, help, span));
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
                { "ignoreTypeOfDescribeName": false, "disallowedWords": ["incorrectly"] }
            ])),
        ),
        ("it('correctly sets the value', () => {});", Some(serde_json::json!([]))),
        // TODO: Unsure if we support the { "vitest": { "typecheck": true }} option
        //      (
        //          "
        //              function foo(){}
        //              describe(foo, () => {
        //                  test('item', () => {
        //                  expect(0).toBe(0)
        //                  })
        //              })
        // ",
        //          Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        //      ),
        //      (
        //          "
        //              declare const outerName: string;
        //              describe(outerName, () => {
        //                  test('item', () => {
        //                      expect(0).toBe(0)
        //                  })
        //              })
        //      ",
        //          Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        //      ),
        //      (
        //          "
        //              declare const outerName: 'a';
        //              describe(outerName, () => {
        //                  test('item', () => {
        //                      expect(0).toBe(0)
        //                  })
        //              })
        //          ",
        //          Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        //      ),
        //      (
        //          "
        //              declare const outerName: `${'a'}`;
        //              describe(outerName, () => {
        //                  test('item', () => {
        //                      expect(0).toBe(0)
        //                  })
        //              })
        //          ",
        //          Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        //      ),
        //      (
        //          "
        //              class foo{}
        //              describe(foo, () => {
        //                  test('item', () => {
        //                      expect(0).toBe(0)
        //                  })
        //              })
        //          ",
        //          Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        //      ),
        ("it(foo, () => {});", Some(serde_json::json!([{ "allowArguments": true }]))),
        ("describe(bar, () => {});", Some(serde_json::json!([{ "allowArguments": true }]))),
        ("test(baz, () => {});", Some(serde_json::json!([{ "allowArguments": true }]))),
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
                export const myTest = test.extend({
                    archive: []
			    })
			",
            None,
        ),
        ("const localTest = test.extend({})", None),
        (
            "
                import { it } from 'vitest'

                const test = it.extend({
                    fixture: [
                        async ({}, use) => {
                            setup()
                            await use()
                            teardown()
                        },
                        { auto: true }
                    ],
                })

                test('', () => {})
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
        ("test(bar, () => {});", Some(serde_json::json!([{ "allowArguments": false }]))),
        (
            "
                describe('things to test', () => {
                    describe('unit tests #unit', () => {
                        it('is true', () => {
                            expect(true).toBe(true);
                        });
                    });

                    describe('e2e tests #e4e', () => {
                        it('is another test #e2e #vitest4life', () => {});
                    });
                });
            ",
            Some(serde_json::json!([{
                "mustNotMatch": r"(?:#(?!unit|e2e))\w+",
                "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))"
            }])),
        ),
        (
            "
                describe('things to test', () => {
                    describe('unit tests #unit', () => {
                        it('is true', () => {
                            expect(true).toBe(true);
                        });
                    });

                    describe('e2e tests #e4e', () => {
                        it('is another test #e2e #vitest4life', () => {});
                    });
                });
            ",
            Some(serde_json::json!([{
                "mustNotMatch": [r"(?:#(?!unit|e2e))\w+", "Please include '#unit' or '#e2e' in titles"],
                "mustMatch": ["^[^#]+$|(?:#(?:unit|e2e))", "Please include '#unit' or '#e2e' in titles"]
            }])),
        ),
        (
            "test('the correct way to properly handle all things', () => {});",
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
        ),
        (
            "describe.skip('the test', () => {});",
            Some(serde_json::json!([{ "mustMatch": { "describe": "(?:unit|integration|e2e)" } }])),
        ),
        ("it.each([])(1, () => {});", None),
        ("it.skip.each([])(1, () => {});", None),
        ("it.skip.each``(1, () => {});", None),
        ("it(123, () => {});", None),
        ("it.concurrent(123, () => {});", None),
        ("it(1 + 2 + 3, () => {});", None),
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
        // TODO: Unsure if we support the { "vitest": { "typecheck": true }} option
        // (
        //     "it('', function () {})",
        //     Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        // ),
        ("it.concurrent('', function () {})", None),
        ("test('', function () {})", None),
        ("test.concurrent('', function () {})", None),
        ("test.concurrent(``, function () {})", None),
        ("xdescribe('', () => {})", None),
        ("describe(' foo', function () {})", None),
        ("describe.each()(' foo', function () {})", None),
        ("describe.only.each()(' foo', function () {})", None),
        ("describe(' foo foe fum', function () {})", None),
        ("describe('foo foe fum ', function () {})", None),
        ("it.skip(' foo', function () {})", None),
        ("fit('foo ', function () {})", None),
        ("it.skip('foo ', function () {})", None),
    ];

    let fix = vec![
        ("describe(' foo', function () {})", "describe('foo', function () {})"),
        ("describe.each()(' foo', function () {})", "describe.each()('foo', function () {})"),
        (
            "describe.only.each()(' foo', function () {})",
            "describe.only.each()('foo', function () {})",
        ),
        ("describe(' foo foe fum', function () {})", "describe('foo foe fum', function () {})"),
        ("describe('foo foe fum ', function () {})", "describe('foo foe fum', function () {})"),
        ("it.skip(' foo', function () {})", "it.skip('foo', function () {})"),
        ("fit('foo ', function () {})", "fit('foo', function () {})"),
        ("it.skip('foo ', function () {})", "it.skip('foo', function () {})"),
    ];

    Tester::new(ValidTitle::NAME, ValidTitle::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
