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
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode,
        collect_possible_jest_call_node, is_type_of_jest_fn_call, parse_general_jest_fn_call,
    },
};

fn valid_title_diagnostic(error_message: &str, help_message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(error_message.to_string())
        .with_help(help_message.to_string())
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<ValidTitleConfig>);

#[derive(Debug, Default, Clone)]
pub struct ValidTitleConfig {
    ignore_type_of_describe_name: bool,
    allow_arguments: bool,
    disallowed_words: FxHashSet<CompactStr>,
    must_not_match_patterns: FxHashMap<MatchKind, CompiledMatcherAndMessage>,
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
    ///     ignoreTypeOfDescribeName?: boolean;
    ///     allowArguments?: boolean;
    ///     disallowedWords?: string[];
    ///     mustNotMatch?: Partial<Record<'describe' | 'test' | 'it', string>> | string;
    ///     mustMatch?: Partial<Record<'describe' | 'test' | 'it', string>> | string;
    /// }
    /// ```
    ///
    ValidTitle,
    vitest,
    style,
    conditional_fix
);

impl Rule for ValidTitle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);

        // Helper function to get boolean values with default
        let get_bool = |name: &str| -> bool {
            config
                .and_then(|v| v.get(name))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default()
        };

        // Helper function to get string array values as FxHashSet
        let get_string_set = |name: &str| -> FxHashSet<CompactStr> {
            config
                .and_then(|v| v.get(name))
                .and_then(|v| v.as_array())
                .map(|v| v.iter().filter_map(|v| v.as_str().map(CompactStr::from)).collect())
                .unwrap_or_default()
        };

        let ignore_type_of_describe_name = get_bool("ignoreTypeOfDescribeName");
        let allow_arguments = get_bool("allowArguments");
        let disallowed_words = get_string_set("disallowedWords");
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
            allow_arguments,
            disallowed_words,
            must_not_match_patterns,
            must_match_patterns,
        }))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let possible_jest_nodes = collect_possible_jest_call_node(ctx);

        for jest_node in possible_jest_nodes {
            self.process_jest_node(&jest_node, ctx);
        }
    }
}

impl ValidTitle {
    fn process_jest_node<'a>(
        &self,
        possible_jest_fn_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_fn_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_type_of_jest_fn_call(
            call_expr,
            possible_jest_fn_node,
            ctx,
            &[
                JestFnKind::General(JestGeneralFnKind::Describe),
                JestFnKind::General(JestGeneralFnKind::Test),
            ],
        ) {
            return;
        }

        // Parse the Jest function call for additional details
        let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_fn_node, ctx)
        else {
            return;
        };

        // Check if extend keyword has been used
        if let Some(member) = jest_fn_call.members.first()
            && member.is_name_equal("extend") {
                return;
            }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        // Handle typecheck settings - skip for describe when enabled
        if ctx.settings().vitest.typecheck
            && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe))
        {
            return;
        }

        // Handle allowArguments option
        if self.allow_arguments && matches!(arg, Argument::Identifier(_)) {
            return;
        }

        // Process the argument based on its type
        self.process_argument(arg, &jest_fn_call, ctx);
    }

    fn process_argument<'a>(
        &self,
        arg: &Argument<'a>,
        jest_fn_call: &ParsedGeneralJestFnCall<'a>,
        ctx: &LintContext<'a>,
    ) {
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
                if template_literal.is_no_substitution_template()
                    && let Some(quasi) = template_literal.single_quasi() {
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
                if !does_binary_expression_contain_string_node(binary_expr) {
                    self.report_non_string_argument(arg, jest_fn_call, ctx);
                }
            }
            Argument::Identifier(_) => {
                if !self.allow_arguments {
                    self.report_non_string_argument(arg, jest_fn_call, ctx);
                }
            }
            _ => {
                self.report_non_string_argument(arg, jest_fn_call, ctx);
            }
        }
    }

    fn report_non_string_argument<'a>(
        &self,
        arg: &Argument<'a>,
        jest_fn_call: &ParsedGeneralJestFnCall<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Skip check for describe when ignoreTypeOfDescribeName is true
        if self.ignore_type_of_describe_name
            && matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe))
        {
            return;
        }
        Message::TitleMustBeString.diagnostic(ctx, arg.span());
    }
}

type CompiledMatcherAndMessage = (Regex, Option<CompactStr>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum MatchKind {
    Describe,
    It,
    Test,
}

#[derive(Debug)]
enum MatcherPattern<'a> {
    String(&'a serde_json::Value),
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
    let mut patterns = FxHashMap::default();

    match matcher_patterns {
        serde_json::Value::String(_pattern) => {
            let compiled = compile_matcher_pattern(&MatcherPattern::String(matcher_patterns))?;
            patterns.insert(MatchKind::Describe, compiled.clone());
            patterns.insert(MatchKind::It, compiled.clone());
            patterns.insert(MatchKind::Test, compiled);
        }
        serde_json::Value::Array(arr) => {
            if arr.len() == 1 {
                // Single element array should be treated as a string pattern
                let pattern = &arr[0];
                let compiled = compile_matcher_pattern(&MatcherPattern::String(pattern))?;
                patterns.insert(MatchKind::Describe, compiled.clone());
                patterns.insert(MatchKind::It, compiled.clone());
                patterns.insert(MatchKind::Test, compiled);
            } else if arr.len() == 2 {
                let pattern = &arr[0];
                let message = arr.get(1).and_then(|v| v.as_str()).map(CompactStr::from);
                let compiled = compile_matcher_pattern(&MatcherPattern::String(pattern))?;
                let compiled_with_message = (compiled.0, message);
                patterns.insert(MatchKind::Describe, compiled_with_message.clone());
                patterns.insert(MatchKind::It, compiled_with_message.clone());
                patterns.insert(MatchKind::Test, compiled_with_message);
            }
        }
        serde_json::Value::Object(obj) => {
            for (key, value) in obj {
                let kind = MatchKind::from(key)?;
                let compiled = compile_matcher_pattern(&MatcherPattern::String(value))?;
                patterns.insert(kind, compiled);
            }
        }
        _ => return None,
    }

    Some(patterns)
}

fn compile_matcher_pattern(pattern: &MatcherPattern) -> Option<CompiledMatcherAndMessage> {
    match pattern {
        MatcherPattern::String(value) => {
            let pattern_str = value.as_str()?;
            // Check for JS regex literal: /pattern/flags
            if let Some(stripped) = pattern_str.strip_prefix('/')
                && let Some(end) = stripped.rfind('/') {
                    let (pat, _flags) = stripped.split_at(end);
                    // For now, ignore flags and just use the pattern
                    let regex = Regex::new(pat).ok()?;
                    return Some((regex, None));
                }
            // Fallback: treat as a normal Rust regex
            let regex = Regex::new(pattern_str).ok()?;
            Some((regex, None))
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
    // Check for empty title first
    if title.is_empty() {
        let function_name = if name.starts_with(['f', 'x']) {
            if name.contains("describe") { "describe" } else { "test" }
        } else {
            name
        };
        let error = format!("`{function_name}` should not have an empty title");
        ctx.diagnostic(valid_title_diagnostic(
            &error,
            "Write a meaningful title for your test",
            span,
        ));
        return;
    }

    // Check disallowed words
    if !valid_title.disallowed_words.is_empty() {
        check_disallowed_words(title, span, valid_title, ctx);
        return;
    }

    // Check for leading/trailing spaces
    let trimmed_title = title.trim();
    if trimmed_title != title {
        let (error, help) = Message::AccidentalSpace.detail();
        ctx.diagnostic_with_fix(valid_title_diagnostic(error, help, span), |fixer| {
            fixer.replace(span.shrink(1), CompactStr::from(trimmed_title))
        });
        return;
    }

    // Check for duplicate prefix
    let un_prefixed_name = name.trim_start_matches(['f', 'x']);
    if let Some(first_word) = title.split(' ').next()
        && first_word == un_prefixed_name {
            let (error, help) = Message::DuplicatePrefix.detail();
            ctx.diagnostic_with_fix(valid_title_diagnostic(error, help, span), |fixer| {
                let replaced_title = title[first_word.len()..].trim();
                fixer.replace(span.shrink(1), CompactStr::from(replaced_title))
            });
            return;
        }

    // Check pattern matching
    if let Some(jest_fn_name) = MatchKind::from(un_prefixed_name) {
        check_pattern_matching(title, span, valid_title, jest_fn_name, un_prefixed_name, ctx);
    }
}

fn check_disallowed_words(title: &str, span: Span, valid_title: &ValidTitle, ctx: &LintContext) {
    // Build regex pattern only once and cache it
    let pattern = format!(
        r"(?iu)\b(?:{})\b",
        valid_title
            .disallowed_words
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("|")
            .cow_replace('.', r"\.")
    );

    if let Ok(disallowed_words_reg) = Regex::new(&pattern)
        && let Some(matched) = disallowed_words_reg.find(title) {
            let error =
                CompactStr::from(format!("`{}` is not allowed in test title", matched.as_str()));
            ctx.diagnostic(valid_title_diagnostic(
                error.as_str(),
                format!("`{}` is included in the `disallowedWords` list, try to remove it from your test title", matched.as_str()).as_str(),
                span,
            ));
        }
}

fn check_pattern_matching(
    title: &str,
    span: Span,
    valid_title: &ValidTitle,
    jest_fn_name: MatchKind,
    un_prefixed_name: &str,
    ctx: &LintContext,
) {
    // Check must match patterns
    if let Some((regex, message)) = valid_title.must_match_patterns.get(&jest_fn_name)
        && !regex.is_match(title) {
            let raw_pattern = regex.as_str();
            if let Some(message) = message.as_ref() {
                ctx.diagnostic(valid_title_diagnostic(
                    message.as_str(),
                    "Make sure the title matches the configured `mustMatch` pattern",
                    span,
                ));
            } else {
                // Use CompactStr for the message to avoid heap allocation
                let msg =
                    CompactStr::from(format!("`{un_prefixed_name}` should match `{raw_pattern}`"));
                ctx.diagnostic(valid_title_diagnostic(
                    msg.as_str(),
                    "Make sure the title matches the configured `mustMatch` pattern",
                    span,
                ));
            }
            return;
        }

    // Check must not match patterns
    if let Some((regex, message)) = valid_title.must_not_match_patterns.get(&jest_fn_name)
        && regex.is_match(title) {
            let raw_pattern = regex.as_str();
            if let Some(message) = message.as_ref() {
                ctx.diagnostic(valid_title_diagnostic(
                    message.as_str(),
                    "Make sure the title does not match the configured `mustNotMatch` pattern",
                    span,
                ));
            } else {
                // Use CompactStr for the message to avoid heap allocation
                let msg = CompactStr::from(format!(
                    "`{un_prefixed_name}` should not match `{raw_pattern}`"
                ));
                ctx.diagnostic(valid_title_diagnostic(
                    msg.as_str(),
                    "Make sure the title does not match the configured `mustNotMatch` pattern",
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
    DuplicatePrefix,
    AccidentalSpace,
}

impl Message {
    fn detail(&self) -> (&'static str, &'static str) {
        match self {
            Self::TitleMustBeString => {
                ("Title must be a string", "Replace your title with a string")
            }
            Self::DuplicatePrefix => (
                "Should not have duplicate prefix",
                "The function name already contains the prefix, try to remove the duplicate",
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
        (r#"describe("the correct way to properly handle all the things", () => {});"#, None, None),
        (r#"test("that all is as it should be", () => {});"#, None, None),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(
                serde_json::json!([        {          "ignoreTypeOfDescribeName": false,          "disallowedWords": ["incorrectly"],        },      ]),
            ),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "disallowedWords": "undefined" }])),
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
        (
            "it(foo, () => {});",
            Some(serde_json::json!([        {          "allowArguments": true,        },      ])),
            None,
        ),
        (
            "describe(bar, () => {});",
            Some(serde_json::json!([        {          "allowArguments": true,        },      ])),
            None,
        ),
        (
            "test(baz, () => {});",
            Some(serde_json::json!([        {          "allowArguments": true,        },      ])),
            None,
        ),
        (r#"describe("the correct way to properly handle all the things", () => {});"#, None, None),
        (r#"test("that all is as it should be", () => {});"#, None, None),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": {} }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "/ /u.source" }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": ["/ /u.source"] }])),
            None,
        ),
        (
            r#"it("correctly sets the value #unit", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "/#(?:unit|integration|e2e)/u.source" }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "/^[^#]+$|(?:#(?:unit|e2e))/u.source" }])),
            None,
        ),
        (
            r#"it("correctly sets the value", () => {});"#,
            Some(
                serde_json::json!([{ "mustMatch": { "test": "/#(?:unit|integration|e2e)/u.source" } }]),
            ),
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
            Some(
                serde_json::json!([{ "mustMatch": { "test": "/^[^#]+$|(?:#(?:unit|e2e))/u.source" } }]),
            ),
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

    let fail = vec![
        (
            r#"test("the correct way to properly handle all things", () => {});"#,
            Some(serde_json::json!([{ "disallowedWords": ["correct", "properly", "all"] }])),
            None,
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
                serde_json::json!([        {          "mustNotMatch": "/(?:#(?!unit|e2e))\\w+/u.source",          "mustMatch": "/^[^#]+$|(?:#(?:unit|e2e))/u.source",        },      ]),
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
                serde_json::json!([        {          "mustNotMatch": [            "/(?:#(?!unit|e2e))\\w+/u.source",            "Please include \"#unit\" or \"#e2e\" in titles",          ],          "mustMatch": [            "/^[^#]+$|(?:#(?:unit|e2e))/u.source",            "Please include \"#unit\" or \"#e2e\" in titles",          ],        },      ]),
            ),
            None,
        ),
        (
            r#"test("the correct way to properly handle all things", () => {});"#,
            Some(serde_json::json!([{ "mustMatch": "/#(?:unit|integration|e2e)/u.source" }])),
            None,
        ),
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
        (r#"it.skip(" foo", function () {})"#, None, None),
        (r#"fit("foo ", function () {})"#, None, None),
        (r#"it.skip("foo ", function () {})"#, None, None),
    ];

    let fix = vec![
        (r#"describe(" foo", function () {})"#, r#"describe("foo", function () {})"#, None),
        (
            r#"describe.each()(" foo", function () {})"#,
            r#"describe.each()("foo", function () {})"#,
            None,
        ),
        (
            r#"describe.only.each()(" foo", function () {})"#,
            r#"describe.only.each()("foo", function () {})"#,
            None,
        ),
        (
            r#"describe(" foo foe fum", function () {})"#,
            r#"describe("foo foe fum", function () {})"#,
            None,
        ),
        (
            r#"describe("foo foe fum ", function () {})"#,
            r#"describe("foo foe fum", function () {})"#,
            None,
        ),
        (r#"it.skip(" foo", function () {})"#, r#"it.skip("foo", function () {})"#, None),
        (r#"fit("foo ", function () {})"#, r#"fit("foo", function () {})"#, None),
        (r#"it.skip("foo ", function () {})"#, r#"it.skip("foo", function () {})"#, None),
    ];
    Tester::new(ValidTitle::NAME, ValidTitle::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
