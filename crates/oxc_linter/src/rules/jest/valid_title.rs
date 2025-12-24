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

fn title_must_be_string_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Title must be a string")
        .with_help("Replace your title with a string")
        .with_label(span)
}

fn empty_title_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should not have an empty title")
        .with_help("Write a meaningful title for your test")
        .with_label(span)
}

fn duplicate_prefix_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should not have duplicate prefix")
        .with_help("The function name already has the prefix, try to remove the duplicate prefix")
        .with_label(span)
}

fn accidental_space_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should not have leading or trailing spaces")
        .with_help("Remove the leading or trailing spaces")
        .with_label(span)
}

fn disallowed_word_diagnostic(word: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{word} is not allowed in test title"))
        .with_help("It is included in the `disallowedWords` of your config file, try to remove it from your title")
        .with_label(span)
}

fn must_match_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string())
        .with_help("Make sure the title matches the `mustMatch` of your config file")
        .with_label(span)
}

fn must_not_match_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string())
        .with_help("Make sure the title does not match the `mustNotMatch` of your config file")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<ValidTitleConfig>);

#[derive(Debug, Default, Clone)]
pub struct ValidTitleConfig {
    /// Whether to ignore the type of the name passed to `test`.
    ignore_type_of_test_name: bool,
    /// Whether to ignore the type of the name passed to `describe`.
    ignore_type_of_describe_name: bool,
    /// Whether to allow arguments as titles.
    allow_arguments: bool,
    /// A list of disallowed words, which will not be allowed in titles.
    disallowed_words: Vec<CompactStr>,
    /// Whether to ignore leading and trailing spaces in titles.
    ignore_space: bool,
    /// Patterns for titles that must not match.
    must_not_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
    /// Patterns for titles that must be matched for the title to be valid.
    must_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
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
    /// Checks that the titles of Jest and Vitest blocks are valid.
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
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// describe('foo', () => {});
    /// it('bar', () => {});
    /// test('baz', () => {});
    /// ```
    ///
    /// ### Options
    /// ```typescript
    /// interface Options {
    ///     ignoreSpaces?: boolean;
    ///     ignoreTypeOfTestName?: boolean;
    ///     ignoreTypeOfDescribeName?: boolean;
    ///     allowArguments?: boolean;
    ///     disallowedWords?: string[];
    ///     mustNotMatch?: Partial<Record<'describe' | 'test' | 'it', string>> | string;
    ///     mustMatch?: Partial<Record<'describe' | 'test' | 'it', string>> | string;
    /// }
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/valid-title.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/valid-title": "error"
    ///   }
    /// }
    /// ```
    ValidTitle,
    jest,
    correctness,
    conditional_fix
);

impl Rule for ValidTitle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = value.get(0);
        let get_as_bool = |name: &str| -> bool {
            config
                .and_then(|v| v.get(name))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default()
        };

        let ignore_type_of_test_name = get_as_bool("ignoreTypeOfTestName");
        let ignore_type_of_describe_name = get_as_bool("ignoreTypeOfDescribeName");
        let allow_arguments = get_as_bool("allowArguments");
        let ignore_space = get_as_bool("ignoreSpaces");
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

        Ok(Self(Box::new(ValidTitleConfig {
            ignore_type_of_test_name,
            ignore_type_of_describe_name,
            allow_arguments,
            disallowed_words,
            ignore_space,
            must_not_match_patterns,
            must_match_patterns,
        })))
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

        // Check if extend keyword has been used (vitest feature)
        if let Some(member) = jest_fn_call.members.first()
            && (member.is_name_equal("extend") || member.is_name_equal("scoped"))
        {
            return;
        }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        // Handle typecheck settings - skip for describe when enabled (vitest feature)
        if ctx.settings().vitest.typecheck
            && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe))
        {
            return;
        }

        // Handle allowArguments option (vitest feature)
        if self.allow_arguments && matches!(arg, Argument::Identifier(_)) {
            return;
        }

        let need_report_name = match jest_fn_call.kind {
            JestFnKind::General(JestGeneralFnKind::Test) => !self.ignore_type_of_test_name,
            JestFnKind::General(JestGeneralFnKind::Describe) => !self.ignore_type_of_describe_name,
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
                if let Some(quasi) = template_literal.single_quasi() {
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
                if need_report_name {
                    ctx.diagnostic(title_must_be_string_diagnostic(arg.span()));
                }
            }
            _ => {
                if need_report_name {
                    ctx.diagnostic(title_must_be_string_diagnostic(arg.span()));
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
            let pattern_str = pattern.as_str()?;

            // Check for JS regex literal: /pattern/flags
            if let Some(stripped) = pattern_str.strip_prefix('/')
                && let Some(end) = stripped.rfind('/')
            {
                let (pat, _flags) = stripped.split_at(end);
                // For now, ignore flags and just use the pattern
                let regex = Regex::new(pat).ok()?;
                return Some((regex, None));
            }

            // Fallback: treat as a normal Rust regex with Unicode support
            let reg_str = format!("(?u){pattern_str}");
            let reg = Regex::new(&reg_str).ok()?;
            Some((reg, None))
        }
        MatcherPattern::Vec(pattern) => {
            let pattern_str = pattern.first().and_then(|v| v.as_str())?;

            // Check for JS regex literal: /pattern/flags
            let regex = if let Some(stripped) = pattern_str.strip_prefix('/')
                && let Some(end) = stripped.rfind('/')
            {
                let (pat, _flags) = stripped.split_at(end);
                Regex::new(pat).ok()?
            } else {
                let reg_str = format!("(?u){pattern_str}");
                Regex::new(&reg_str).ok()?
            };

            let message = pattern.get(1).and_then(serde_json::Value::as_str).map(CompactStr::from);
            Some((regex, message))
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
        ctx.diagnostic(empty_title_diagnostic(span));
        return;
    }

    if !valid_title.disallowed_words.is_empty() {
        let Ok(disallowed_words_reg) = Regex::new(&format!(
            r"(?iu)\b(?:{})\b",
            valid_title.disallowed_words.join("|").cow_replace('.', r"\.")
        )) else {
            return;
        };

        if let Some(matched) = disallowed_words_reg.find(title) {
            ctx.diagnostic(disallowed_word_diagnostic(matched.as_str(), span));
        }
        return;
    }

    let trimmed_title = title.trim();
    if !valid_title.ignore_space && trimmed_title != title {
        ctx.diagnostic_with_fix(accidental_space_diagnostic(span), |fixer| {
            let inner_span = span.shrink(1);
            let raw_text = fixer.source_range(inner_span);
            let trimmed_raw = raw_text.trim().to_string();
            fixer.replace(inner_span, trimmed_raw)
        });
    }

    let un_prefixed_name = name.trim_start_matches(['f', 'x']);
    let Some(first_word) = title.split(' ').next() else {
        return;
    };

    if first_word == un_prefixed_name {
        ctx.diagnostic_with_fix(duplicate_prefix_diagnostic(span), |fixer| {
            // Use raw source text to preserve escape sequences
            let inner_span = span.shrink(1);
            let raw_text = fixer.source_range(inner_span);
            // Find the first space in raw text to avoid byte offset issues
            // if the prefix word ever contains escapable characters
            let space_pos = raw_text.find(' ').unwrap_or(raw_text.len());
            let replaced_raw = raw_text[space_pos..].trim().to_string();
            fixer.replace(inner_span, replaced_raw)
        });
        return;
    }

    let Some(jest_fn_name) = MatchKind::from(un_prefixed_name) else {
        return;
    };

    if let Some((regex, message)) = valid_title.must_match_patterns.get(&jest_fn_name)
        && !regex.is_match(title)
    {
        let raw_pattern = regex.as_str();
        let message = match message.as_ref() {
            Some(message) => message.as_str(),
            None => &format!("{un_prefixed_name} should match {raw_pattern}"),
        };
        ctx.diagnostic(must_match_diagnostic(message, span));
    }

    if let Some((regex, message)) = valid_title.must_not_match_patterns.get(&jest_fn_name)
        && regex.is_match(title)
    {
        let raw_pattern = regex.as_str();
        let message = match message.as_ref() {
            Some(message) => message.as_str(),
            None => &format!("{un_prefixed_name} should not match {raw_pattern}"),
        };

        ctx.diagnostic(must_not_match_diagnostic(message, span));
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

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("describe('the correct way to properly handle all the things', () => {});", None, None),
        ("test('that all is as it should be', () => {});", None, None),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([
              { "ignoreTypeOfDescribeName": false, "disallowedWords": ["correct"] },
            ])),
            None,
        ),
        ("it('correctly sets the value', () => {});", Some(serde_json::json!([])), None),
        ("describe('the correct way to properly handle all the things', () => {});", None, None),
        ("test('that all is as it should be', () => {});", None, None),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": {} }])),
            None,
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": " " }])),
            None,
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": [" "] }])),
            None,
        ),
        (
            "it('correctly sets the value #unit', () => {});",
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
            None,
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))" }])),
            None,
        ),
        (
            "it('correctly sets the value', () => {});",
            Some(serde_json::json!([{ "mustMatch": { "test": "#(?:unit|integration|e2e)" } }])),
            None,
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
            None,
        ),
        ("it('is a string', () => {});", None, None),
        ("it('is' + ' a ' + ' string', () => {});", None, None),
        ("it(1 + ' + ' + 1, () => {});", None, None),
        ("test('is a string', () => {});", None, None),
        ("xtest('is a string', () => {});", None, None),
        ("xtest(`${myFunc} is a string`, () => {});", None, None),
        ("describe('is a string', () => {});", None, None),
        ("describe.skip('is a string', () => {});", None, None),
        ("describe.skip(`${myFunc} is a string`, () => {});", None, None),
        ("fdescribe('is a string', () => {});", None, None),
        (
            "describe(String(/.+/), () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
            None,
        ),
        (
            "describe(myFunction, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
            None,
        ),
        (
            "xdescribe(skipFunction, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true, "disallowedWords": [] }])),
            None,
        ),
        ("describe()", None, None),
        ("someFn('', function () {})", None, None),
        ("describe('foo', function () {})", None, None),
        ("describe('foo', function () { it('bar', function () {}) })", None, None),
        ("test('foo', function () {})", None, None),
        ("test.concurrent('foo', function () {})", None, None),
        ("test(`foo`, function () {})", None, None),
        ("test.concurrent(`foo`, function () {})", None, None),
        ("test(`${foo}`, function () {})", None, None),
        ("test.concurrent(`${foo}`, function () {})", None, None),
        ("it('foo', function () {})", None, None),
        ("it.each([])()", None, None),
        ("it.concurrent('foo', function () {})", None, None),
        ("xdescribe('foo', function () {})", None, None),
        ("xit('foo', function () {})", None, None),
        ("xtest('foo', function () {})", None, None),
        ("it()", None, None),
        ("it.concurrent()", None, None),
        ("describe()", None, None),
        ("it.each()()", None, None),
        ("describe('foo', function () {})", None, None),
        ("fdescribe('foo', function () {})", None, None),
        ("xdescribe('foo', function () {})", None, None),
        ("it('foo', function () {})", None, None),
        ("it.concurrent('foo', function () {})", None, None),
        ("fit('foo', function () {})", None, None),
        ("fit.concurrent('foo', function () {})", None, None),
        ("xit('foo', function () {})", None, None),
        ("test('foo', function () {})", None, None),
        ("test.concurrent('foo', function () {})", None, None),
        ("xtest('foo', function () {})", None, None),
        ("xtest(`foo`, function () {})", None, None),
        ("someFn('foo', function () {})", None, None),
        (
            "
                describe('foo', () => {
                it('bar', () => {})
                })
            ",
            None,
            None,
        ),
        (
            "it(`GIVEN...
            `, () => {});",
            Some(serde_json::json!([{ "ignoreSpaces": true }])),
            None,
        ),
        ("describe('foo', function () {})", None, None),
        ("fdescribe('foo', function () {})", None, None),
        ("xdescribe('foo', function () {})", None, None),
        ("xdescribe(`foo`, function () {})", None, None),
        ("test('foo', function () {})", None, None),
        ("test('foo', function () {})", None, None),
        ("xtest('foo', function () {})", None, None),
        ("xtest(`foo`, function () {})", None, None),
        ("test('foo test', function () {})", None, None),
        ("xtest('foo test', function () {})", None, None),
        ("it('foo', function () {})", None, None),
        ("fit('foo', function () {})", None, None),
        ("xit('foo', function () {})", None, None),
        ("xit(`foo`, function () {})", None, None),
        ("it('foos it correctly', function () {})", None, None),
        (
            "
                describe('foo', () => {
                it('bar', () => {})
                })
            ",
            None,
            None,
        ),
        (
            "
                describe('foo', () => {
                it('describes things correctly', () => {})
                })
            ",
            None,
            None,
        ),
        (
            "it(abc, function () {})",
            Some(serde_json::json!([{ "ignoreTypeOfTestName": true }])),
            None,
        ),
        // Vitest-specific tests with allowArguments option
        ("it(foo, () => {});", Some(serde_json::json!([{ "allowArguments": true }])), None),
        ("describe(bar, () => {});", Some(serde_json::json!([{ "allowArguments": true }])), None),
        ("test(baz, () => {});", Some(serde_json::json!([{ "allowArguments": true }])), None),
        // Vitest-specific tests with .extend()
        (
            "export const myTest = test.extend({
                archive: []
            })",
            None,
            None,
        ),
        ("const localTest = test.extend({})", None, None),
        (
            "import { it } from 'vitest'

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

            test('', () => {})",
            None,
            None,
        ),
    ];

    let mut fail = vec![
        (
            "describe('the correct way to do things', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
            None,
        ),
        (
            "it('has ALL the things', () => {})",
            Some(serde_json::json!([{ "disallowedWords": ["all"] }])),
            None,
        ),
        (
            "xdescribe('every single one of them', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["every"] }])),
            None,
        ),
        (
            "describe('Very Descriptive Title Goes Here', function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["descriptive"] }])),
            None,
        ),
        (
            "test(`that the value is set properly`, function () {})",
            Some(serde_json::json!([{ "disallowedWords": ["properly"] }])),
            None,
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
            None,
        ),
        (
            "describe('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
            None,
        ),
        (
            "xdescribe('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
            None,
        ),
        (
            "describe.skip('the test', () => {});",
            Some(serde_json::json!([
              { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } },
            ])),
            None,
        ),
        ("it.each([])(1, () => {});", None, None),
        ("it.skip.each([])(1, () => {});", None, None),
        ("it.skip.each``(1, () => {});", None, None),
        ("it(123, () => {});", None, None),
        ("it.concurrent(123, () => {});", None, None),
        ("it(1 + 2 + 3, () => {});", None, None),
        ("it.concurrent(1 + 2 + 3, () => {});", None, None),
        (
            "test.skip(123, () => {});",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": true }])),
            None,
        ),
        ("describe(String(/.+/), () => {});", None, None),
        (
            "describe(myFunction, () => 1);",
            Some(serde_json::json!([{ "ignoreTypeOfDescribeName": false }])),
            None,
        ),
        ("describe(myFunction, () => {});", None, None),
        ("xdescribe(myFunction, () => {});", None, None),
        ("describe(6, function () {})", None, None),
        ("describe.skip(123, () => {});", None, None),
        ("describe('', function () {})", None, None),
        (
            "
                describe('foo', () => {
                    it('', () => {});
                });
            ",
            None,
            None,
        ),
        ("it('', function () {})", None, None),
        ("it.concurrent('', function () {})", None, None),
        ("test('', function () {})", None, None),
        ("test.concurrent('', function () {})", None, None),
        ("test(``, function () {})", None, None),
        ("test.concurrent(``, function () {})", None, None),
        ("xdescribe('', () => {})", None, None),
        ("xit('', () => {})", None, None),
        ("xtest('', () => {})", None, None),
        ("describe(' foo', function () {})", None, None),
        ("describe.each()(' foo', function () {})", None, None),
        ("describe.only.each()(' foo', function () {})", None, None),
        ("describe(' foo foe fum', function () {})", None, None),
        ("describe('foo foe fum ', function () {})", None, None),
        ("fdescribe(' foo', function () {})", None, None),
        ("fdescribe(' foo', function () {})", None, None),
        ("xdescribe(' foo', function () {})", None, None),
        ("it(' foo', function () {})", None, None),
        ("it.concurrent(' foo', function () {})", None, None),
        ("fit(' foo', function () {})", None, None),
        ("it.skip(' foo', function () {})", None, None),
        ("fit('foo ', function () {})", None, None),
        ("it.skip('foo ', function () {})", None, None),
        (
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works ', () => {});
            ",
            None,
            None,
        ),
        ("xit(' foo', function () {})", None, None),
        ("test(' foo', function () {})", None, None),
        ("test.concurrent(' foo', function () {})", None, None),
        ("test(` foo`, function () {})", None, None),
        ("test.concurrent(` foo`, function () {})", None, None),
        ("test(` foo bar bang`, function () {})", None, None),
        ("test.concurrent(` foo bar bang`, function () {})", None, None),
        ("test(` foo bar bang  `, function () {})", None, None),
        ("test.concurrent(` foo bar bang  `, function () {})", None, None),
        ("xtest(' foo', function () {})", None, None),
        ("xtest(' foo  ', function () {})", None, None),
        (
            "
                describe(' foo', () => {
                    it('bar', () => {})
                })
            ",
            None,
            None,
        ),
        (
            "
                describe('foo', () => {
                    it(' bar', () => {})
                })
            ",
            None,
            None,
        ),
        ("describe('describe foo', function () {})", None, None),
        ("fdescribe('describe foo', function () {})", None, None),
        ("xdescribe('describe foo', function () {})", None, None),
        ("describe('describe foo', function () {})", None, None),
        ("fdescribe(`describe foo`, function () {})", None, None),
        ("test('test foo', function () {})", None, None),
        ("xtest('test foo', function () {})", None, None),
        ("test(`test foo`, function () {})", None, None),
        ("test(`test foo test`, function () {})", None, None),
        ("it('it foo', function () {})", None, None),
        ("fit('it foo', function () {})", None, None),
        ("xit('it foo', function () {})", None, None),
        ("it('it foos it correctly', function () {})", None, None),
        (
            "
                describe('describe foo', () => {
                    it('bar', () => {})
                })
            ",
            None,
            None,
        ),
        (
            "
                describe('describe foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
            None,
            None,
        ),
        (
            "
                describe('foo', () => {
                    it('it bar', () => {})
                })
            ",
            None,
            None,
        ),
        ("it(abc, function () {})", None, None),
        // Vitest-specific fail test with allowArguments: false
        ("test(bar, () => {});", Some(serde_json::json!([{ "allowArguments": false }])), None),
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
        ("fdescribe(' foo', function () {})", "fdescribe('foo', function () {})"),
        ("fdescribe(' foo', function () {})", "fdescribe('foo', function () {})"),
        ("xdescribe(' foo', function () {})", "xdescribe('foo', function () {})"),
        ("it(' foo', function () {})", "it('foo', function () {})"),
        ("it.concurrent(' foo', function () {})", "it.concurrent('foo', function () {})"),
        ("fit(' foo', function () {})", "fit('foo', function () {})"),
        ("it.skip(' foo', function () {})", "it.skip('foo', function () {})"),
        ("fit('foo ', function () {})", "fit('foo', function () {})"),
        ("it.skip('foo ', function () {})", "it.skip('foo', function () {})"),
        (
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works ', () => {});
            ",
            "
                import { test as testThat } from '@jest/globals';

                testThat('foo works', () => {});
            ",
        ),
        ("xit(' foo', function () {})", "xit('foo', function () {})"),
        ("test(' foo', function () {})", "test('foo', function () {})"),
        ("test.concurrent(' foo', function () {})", "test.concurrent('foo', function () {})"),
        ("test(` foo`, function () {})", "test(`foo`, function () {})"),
        ("test.concurrent(` foo`, function () {})", "test.concurrent(`foo`, function () {})"),
        ("test(` foo bar bang`, function () {})", "test(`foo bar bang`, function () {})"),
        (
            "test.concurrent(` foo bar bang`, function () {})",
            "test.concurrent(`foo bar bang`, function () {})",
        ),
        ("test(` foo bar bang  `, function () {})", "test(`foo bar bang`, function () {})"),
        (
            "test.concurrent(` foo bar bang  `, function () {})",
            "test.concurrent(`foo bar bang`, function () {})",
        ),
        ("xtest(' foo', function () {})", "xtest('foo', function () {})"),
        ("xtest(' foo  ', function () {})", "xtest('foo', function () {})"),
        (
            "
                describe(' foo', () => {
                    it('bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        (
            "
                describe('foo', () => {
                    it(' bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        ("describe('describe foo', function () {})", "describe('foo', function () {})"),
        ("fdescribe('describe foo', function () {})", "fdescribe('foo', function () {})"),
        ("xdescribe('describe foo', function () {})", "xdescribe('foo', function () {})"),
        ("describe('describe foo', function () {})", "describe('foo', function () {})"),
        ("fdescribe(`describe foo`, function () {})", "fdescribe(`foo`, function () {})"),
        ("test('test foo', function () {})", "test('foo', function () {})"),
        ("xtest('test foo', function () {})", "xtest('foo', function () {})"),
        ("test(`test foo`, function () {})", "test(`foo`, function () {})"),
        ("test(`test foo test`, function () {})", "test(`foo test`, function () {})"),
        ("it('it foo', function () {})", "it('foo', function () {})"),
        ("fit('it foo', function () {})", "fit('foo', function () {})"),
        ("xit('it foo', function () {})", "xit('foo', function () {})"),
        ("it('it foos it correctly', function () {})", "it('foos it correctly', function () {})"),
        (
            "
                describe('describe foo', () => {
                    it('bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        (
            "
                describe('describe foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('describes things correctly', () => {})
                })
            ",
        ),
        (
            "
                describe('foo', () => {
                    it('it bar', () => {})
                })
            ",
            "
                describe('foo', () => {
                    it('bar', () => {})
                })
            ",
        ),
        // AccidentalSpace: preserve escape sequences when trimming spaces
        (
            "test('issue #225513: Cmd-Click doesn\\'t work on JSDoc {@link URL|LinkText} format ', () => { assert(true); });",
            "test('issue #225513: Cmd-Click doesn\\'t work on JSDoc {@link URL|LinkText} format', () => { assert(true); });",
        ),
        // DuplicatePrefix: preserve escape sequences when removing prefix
        (
            "test('test that it doesn\\'t break', () => {});",
            "test('that it doesn\\'t break', () => {});",
        ),
    ];

    let pass_vitest = vec![("test.scoped({})", None, None), ("it.scoped({})", None, None)];
    let fail_vitest = vec![
        (
            "import { test } from './test-extend'
      			      test('the correct way to properly handle all things', () => {})
      			      ",
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
            Some(
                serde_json::json!({ "settings": { "vitest": { "vitestImports": ["./test-extend"], },} }),
            ),
        ),
        (
            "import { test } from '@/tests/fixtures'
      			      test('the correct way to properly handle all things', () => {})
      			      ",
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
            Some(
                serde_json::json!({ "settings": { "vitest": { "vitestImports": ["@/tests/fixtures"], },} }),
            ),
        ),
        (
            "import { test } from '@/tests/fixtures'
      			      test('the correct way to properly handle all things', () => {})
      			      ",
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
            Some(
                serde_json::json!({ "settings": { "vitest": { "vitestImports": [r"\/fixtures$"] },} }),
            ),
        ),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(ValidTitle::NAME, ValidTitle::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
