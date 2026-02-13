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
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn title_must_be_string_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test title must be a string, a function or class name")
        .with_help("Replace your title with a string, class name, or function name")
        .with_label(span)
}

fn empty_title_diagnostic(function_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{function_name} should not have an empty title"))
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
    OxcDiagnostic::warn(format!("\"{word}\" is not allowed in test title"))
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
    /// Whether to ignore the type of the name passed to `describe`.
    ignore_type_of_describe_name: bool,
    /// Whether to allow arguments as titles.
    allow_arguments: bool,
    /// A list of disallowed words, which will not be allowed in titles.
    disallowed_words: Vec<CompactStr>,
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
    /// Checks that titles of Vitest `describe`, `it`, and `test` blocks are valid.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid titles reduce test readability and can make intent harder to understand.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// describe("", () => {})
    /// test(" test title ", () => {})
    /// it("it should work", () => {})
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// describe("api", () => {})
    /// test("returns 200", () => {})
    /// it("works with empty input", () => {})
    /// ```
    ValidTitle,
    vitest,
    correctness,
    conditional_fix,
    config = Value,
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

        Ok(Self(Box::new(ValidTitleConfig {
            ignore_type_of_describe_name,
            allow_arguments,
            disallowed_words,
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

        // Skip `test.extend(...)` and `test.scoped(...)` forms.
        if jest_fn_call
            .members
            .iter()
            .any(|member| member.is_name_equal("extend") || member.is_name_equal("scoped"))
        {
            return;
        }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        if self.allow_arguments && matches!(arg, Argument::Identifier(_)) {
            return;
        }

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
                if ctx.settings().vitest.typecheck
                    && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe))
                {
                    return;
                }
                if need_report_name {
                    ctx.diagnostic(title_must_be_string_diagnostic(arg.span()));
                }
            }
            _ => {
                if ctx.settings().vitest.typecheck
                    && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe))
                {
                    return;
                }
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
    let un_prefixed_name = name.trim_start_matches(['f', 'x']);

    if title.is_empty() {
        let function_name = if un_prefixed_name == "describe" { "describe" } else { "test" };
        ctx.diagnostic(empty_title_diagnostic(function_name, span));
        return;
    }

    if !valid_title.disallowed_words.is_empty() {
        if let Ok(disallowed_words_reg) = Regex::new(&format!(
            r"(?iu)\b(?:{})\b",
            valid_title.disallowed_words.join("|").cow_replace('.', r"\.")
        )) {
            if let Some(matched) = disallowed_words_reg.find(title) {
                ctx.diagnostic(disallowed_word_diagnostic(matched.as_str(), span));
                return;
            }
        }
    }

    let trimmed_title = title.trim();
    if trimmed_title != title {
        ctx.diagnostic_with_fix(accidental_space_diagnostic(span), |fixer| {
            let inner_span = span.shrink(1);
            let raw_text = fixer.source_range(inner_span);
            let trimmed_raw = raw_text.trim().to_string();
            fixer.replace(inner_span, trimmed_raw)
        });
    }

    let Some(first_word) = title.split(' ').next() else {
        return;
    };

    if first_word.eq_ignore_ascii_case(un_prefixed_name) {
        ctx.diagnostic_with_fix(duplicate_prefix_diagnostic(span), |fixer| {
            let inner_span = span.shrink(1);
            let raw_text = fixer.source_range(inner_span);
            let space_pos = raw_text.find(' ').unwrap_or(raw_text.len());
            let replaced_raw = raw_text[space_pos..].trim().to_string();
            fixer.replace(inner_span, replaced_raw)
        });
        return;
    }

    let Some(vitest_fn_name) = MatchKind::from(un_prefixed_name) else {
        return;
    };

    if let Some((regex, message)) = valid_title.must_not_match_patterns.get(&vitest_fn_name)
        && regex.is_match(title)
    {
        let raw_pattern = regex.as_str();
        let message = match message.as_ref() {
            Some(message) => message.as_str(),
            None => &format!("{un_prefixed_name} should not match {raw_pattern}"),
        };

        ctx.diagnostic(must_not_match_diagnostic(message, span));
        return;
    }

    if let Some((regex, message)) = valid_title.must_match_patterns.get(&vitest_fn_name)
        && !regex.is_match(title)
    {
        let raw_pattern = regex.as_str();
        let message = match message.as_ref() {
            Some(message) => message.as_str(),
            None => &format!("{un_prefixed_name} should match {raw_pattern}"),
        };
        ctx.diagnostic(must_match_diagnostic(message, span));
    }
}

fn does_binary_expression_contain_string_node(expr: &BinaryExpression) -> bool {
    if expr.left.is_string_literal() || expr.right.is_string_literal() {
        return true;
    }

    let left_has_string = matches!(&expr.left, Expression::BinaryExpression(left) if does_binary_expression_contain_string_node(left));
    let right_has_string = matches!(&expr.right, Expression::BinaryExpression(right) if does_binary_expression_contain_string_node(right));

    left_has_string || right_has_string
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"describe("the correct way to properly handle all the things", () => {});"#, None, None),
        (r#"test("that all is as it should be", () => {});"#, None, None),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(
                serde_json::json!([ { "ignoreTypeOfDescribeName": false, "disallowedWords": ["incorrectly"], }, ]),
            ),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "disallowedWords": null }])),
            None,
        ),
        (
            "
                  function foo(){}
                  describe(foo, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  })
                 ",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            "
                  declare const outerName: string;
                  describe(outerName, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  })
                 ",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            "
                  declare const outerName: 'a';
                  describe(outerName, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  })
                 ",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            "
                  declare const outerName: `${'a'}`;
                  describe(outerName, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  })
                 ",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            "
                  class foo{}
                  describe(foo, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  })
                  ",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            "
                  type Func = (params: object) => void
                  const func: Func = (params) => console.log(params)
                  describe(func, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  });",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            "
                  interface Func {
                    (params: object): void
                  }
                  const func: Func = (params) => console.log(params)
                  describe(func, () => {
                    test('item', () => {
                      expect(0).toBe(0)
                    })
                  });",
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        ("it(foo, () => {});", Some(serde_json::json!([ { "allowArguments": true, }, ])), None),
        (
            "describe(bar, () => {});",
            Some(serde_json::json!([ { "allowArguments": true, }, ])),
            None,
        ),
        ("test(baz, () => {});", Some(serde_json::json!([ { "allowArguments": true, }, ])), None),
        (r#"describe("the correct way to properly handle all the things", () => {});"#, None, None),
        (r#"test("that all is as it should be", () => {});"#, None, None),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": {} }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": " " }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": [" "] }])),
            None,
        ),
        (
            r#"it("correctly sets the value #unit", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))" }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": { "test": "#(?:unit|integration|e2e)" } }])),
            None,
        ),
        (
            "describe('things to test', () => {
                  describe('unit tests #unit', () => {
                    it('is true', () => {
                   expect(true).toBe(true);
                    });
                  });
            
                  describe('e2e tests #e2e', () => {
                    it('is another test #jest4life', () => {});
                  });
                   });",
            Some(serde_json::json!([{ "mustMatch": { "test": "^[^#]+$|(?:#(?:unit|e2e))" } }])),
            None,
        ),
        (r#"it("is a string", () => {});"#, None, None),
        (r#"it("is" + " a " + " string", () => {});"#, None, None),
        (r#"it(1 + " + " + 1, () => {});"#, None, None),
        (r#"test("is a string", () => {});"#, None, None),
        (r#"xtest("is a string", () => {});"#, None, None),
        ("xtest(`${myFunc} is a string`, () => {});", None, None),
        (r#"describe("is a string", () => {});"#, None, None),
        (r#"describe.skip("is a string", () => {});"#, None, None),
        ("describe.skip(`${myFunc} is a string`, () => {});", None, None),
        (r#"fdescribe("is a string", () => {});"#, None, None),
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
        (r#"someFn("", function () {})"#, None, None),
        (r#"describe("foo", function () {})"#, None, None),
        (r#"describe("foo", function () { it("bar", function () {}) })"#, None, None),
        (r#"test("foo", function () {})"#, None, None),
        (r#"test.concurrent("foo", function () {})"#, None, None),
        ("test(`foo`, function () {})", None, None),
        ("test.concurrent(`foo`, function () {})", None, None),
        ("test(`${foo}`, function () {})", None, None),
        ("test.concurrent(`${foo}`, function () {})", None, None),
        ("test.scoped({})", None, None),
        ("it.scoped({})", None, None),
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
        (r#"describe("foo", function () {})"#, None, None),
        (r#"fdescribe("foo", function () {})"#, None, None),
        (r#"xdescribe("foo", function () {})"#, None, None),
        (r#"it("foo", function () {})"#, None, None),
        (r#"it.concurrent("foo", function () {})"#, None, None),
        (r#"fit("foo", function () {})"#, None, None),
        (r#"fit.concurrent("foo", function () {})"#, None, None),
        (r#"xit("foo", function () {})"#, None, None),
        (r#"test("foo", function () {})"#, None, None),
        (r#"test.concurrent("foo", function () {})"#, None, None),
        (r#"xtest("foo", function () {})"#, None, None),
        ("xtest(`foo`, function () {})", None, None),
        (r#"someFn("foo", function () {})"#, None, None),
        (
            "
                import { test } from 'vitest';
            
                export const myTest = test.extend({
                  archive: []
                })",
            None,
            None,
        ),
        (
            "
                    import { test } from 'vitest';
            
                    const it = test.extend({})
                    it('passes', () => {})
                  ",
            None,
            None,
        ),
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
            
                    test('passes', () => {})
                  ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            r#"test("the correct way to properly handle all things", () => {});"#,
            Some(serde_json::json!([{ "disallowedWords": ["correct", "properly", "all"] }])),
            None,
        ),
        (
            r#"test("the correct way to properly handle all things", () => {});"#,
            Some(serde_json::json!([{ "disallowedWords": ["correct", "properly", "all"] }])),
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (
            r#"describe("the correct way to do things", function () {})"#,
            Some(serde_json::json!([{ "disallowedWords": ["correct"] }])),
            None,
        ),
        (
            r#"it("has ALL the things", () => {})"#,
            Some(serde_json::json!([{ "disallowedWords": ["all"] }])),
            None,
        ),
        (
            r#"xdescribe("every single one of them", function () {})"#,
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
        ("test(bar, () => {});", Some(serde_json::json!([{ "allowArguments": false }])), None),
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
                   });",
            Some(
                serde_json::json!([ { "mustNotMatch": "(?:#(?!unit|e2e))\\w+", "mustMatch": "^[^#]+$|(?:#(?:unit|e2e))", }, ]),
            ),
            None,
        ),
        (
            " describe('things to test', () => {
                  describe('unit tests #unit', () => {
                    it('is true', () => {
                   expect(true).toBe(true);
                    });
                  });
            
                  describe('e2e tests #e4e', () => {
                    it('is another test #e2e #vitest4life', () => {});
                  });
                   });",
            Some(
                serde_json::json!([ { "mustNotMatch": [ "(?:#(?!unit|e2e))\\w+", "Please include \"#unit\" or \"#e2e\" in titles", ], "mustMatch": [ "^[^#]+$|(?:#(?:unit|e2e))", "Please include \"#unit\" or \"#e2e\" in titles", ], }, ]),
            ),
            None,
        ),
        (
            r#"test("the correct way to properly handle all things", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "#(?:unit|integration|e2e)" }])),
            None,
        ),
        (
            r#"describe.skip("the test", () => {});"#,
            Some(
                serde_json::json!([ { "mustMatch": { "describe": "#(?:unit|integration|e2e)" } }, ]),
            ),
            None,
        ),
        ("it.each([])(1, () => {});", None, None),
        ("it.skip.each([])(1, () => {});", None, None),
        ("it.skip.each``(1, () => {});", None, None),
        ("it(123, () => {});", None, None),
        ("it.concurrent(123, () => {});", None, None),
        ("it(1 + 2 + 3, () => {});", None, None),
        (r#"describe("", function () {})"#, None, None),
        (
            "
                   describe('foo', () => {
                  it('', () => {});
                   });
                 ",
            None,
            None,
        ),
        (r#"it("", function () {})"#, None, None),
        (
            r#"it("", function () {})"#,
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (r#"it.concurrent("", function () {})"#, None, None),
        (r#"test("", function () {})"#, None, None),
        (r#"test.concurrent("", function () {})"#, None, None),
        ("test.concurrent(``, function () {})", None, None),
        ("xdescribe('', () => {})", None, None),
        (r#"describe(" foo", function () {})"#, None, None),
        (r#"describe.each()(" foo", function () {})"#, None, None),
        (r#"describe.only.each()(" foo", function () {})"#, None, None),
        (r#"describe(" foo foe fum", function () {})"#, None, None),
        (r#"describe("foo foe fum ", function () {})"#, None, None),
        (
            r#"describe("foo foe fum ", function () {})"#,
            None,
            Some(serde_json::json!({ "settings": { "vitest": { "typecheck": true } } })),
        ),
        (r#"it.skip(" foo", function () {})"#, None, None),
        (r#"fit("foo ", function () {})"#, None, None),
        (r#"it.skip("foo ", function () {})"#, None, None),
    ];

    let fix = vec![
        (r#"describe(" foo", function () {})"#, r#"describe("foo", function () {})"#),
        (r#"describe.each()(" foo", function () {})"#, r#"describe.each()("foo", function () {})"#),
        (
            r#"describe.only.each()(" foo", function () {})"#,
            r#"describe.only.each()("foo", function () {})"#,
        ),
        (
            r#"describe(" foo foe fum", function () {})"#,
            r#"describe("foo foe fum", function () {})"#,
        ),
        (
            r#"describe("foo foe fum ", function () {})"#,
            r#"describe("foo foe fum", function () {})"#,
        ),
        (r#"it.skip(" foo", function () {})"#, r#"it.skip("foo", function () {})"#),
        (r#"fit("foo ", function () {})"#, r#"fit("foo", function () {})"#),
        (r#"it.skip("foo ", function () {})"#, r#"it.skip("foo", function () {})"#),
    ];

    Tester::new(ValidTitle::NAME, ValidTitle::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
