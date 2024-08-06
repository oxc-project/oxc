use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, parse_jest_fn_call, JestFnKind, JestGeneralFnKind,
        ParsedJestFnCallNew, PossibleJestNode, Set,
    },
};

fn unexpected_lowercase(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Enforce lowercase test names")
        .with_help(format!("`{x0:?}`s should begin with lowercase"))
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitleConfig {
    allowed_prefixes: Set<CompactStr>,
    ignore: Set<CompactStr>,
    ignore_top_level_describe: bool,
}

impl std::ops::Deref for PreferLowercaseTitle {
    type Target = PreferLowercaseTitleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferLowercaseTitle(Box<PreferLowercaseTitleConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce `it`, `test` and `describe` to have descriptions that begin with a
    /// lowercase letter. This provides more readable test failures. This rule is not
    /// enabled by default.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// it('Adds 1 + 2 to equal 3', () => {
    ///     expect(sum(1, 2)).toBe(3);
    /// });
    ///
    /// // valid
    /// it('adds 1 + 2 to equal 3', () => {
    ///     expect(sum(1, 2)).toBe(3);
    /// });
    /// ```
    ///
    /// ## Options
    /// ```json
    /// {
    ///     "jest/prefer-lowercase-title": [
    ///         "error",
    ///         {
    ///             "ignore": ["describe", "test"]
    ///         }
    ///     ]
    /// }
    /// ```
    ///
    /// ### `ignore`
    ///
    /// This array option controls which Jest functions are checked by this rule. There
    /// are three possible values:
    /// - `"describe"`
    /// - `"test"`
    /// - `"it"`
    ///
    /// By default, none of these options are enabled (the equivalent of
    /// `{ "ignore": [] }`).
    ///
    /// Example of **correct** code for the `{ "ignore": ["describe"] }` option:
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignore": ["describe"] }] */
    /// describe('Uppercase description');
    /// ```
    ///
    /// Example of **correct** code for the `{ "ignore": ["test"] }` option:
    ///
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignore": ["test"] }] */
    /// test('Uppercase description');
    /// ```
    ///
    /// Example of **correct** code for the `{ "ignore": ["it"] }` option:
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignore": ["it"] }] */
    /// it('Uppercase description');
    /// ```
    ///
    /// ### `allowedPrefixes`
    /// This array option allows specifying prefixes, which contain capitals that titles
    /// can start with. This can be useful when writing tests for API endpoints, where
    /// you'd like to prefix with the HTTP method.
    /// By default, nothing is allowed (the equivalent of `{ "allowedPrefixes": [] }`).
    ///
    /// Example of **correct** code for the `{ "allowedPrefixes": ["GET"] }` option:
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "allowedPrefixes": ["GET"] }] */
    /// describe('GET /live');
    /// ```
    ///
    /// ### `ignoreTopLevelDescribe`
    /// This option can be set to allow only the top-level `describe` blocks to have a
    /// title starting with an upper-case letter.
    /// Example of **correct** code for the `{ "ignoreTopLevelDescribe": true }` option:
    ///
    /// ```js
    /// /* eslint jest/prefer-lowercase-title: ["error", { "ignoreTopLevelDescribe": true }] */
    /// describe('MyClass', () => {
    ///     describe('#myMethod', () => {
    ///         it('does things', () => {
    ///             //
    ///         });
    ///     });
    /// });
    /// ```
    ///
    PreferLowercaseTitle,
    style,
    fix
);

impl Rule for PreferLowercaseTitle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(config) = value.get(0) else {
            return Self::default();
        };

        let ignore_top_level_describe = config
            .get("ignoreTopLevelDescribe")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let ignore = config.get("ignore").and_then(|v| Set::try_from(v).ok()).unwrap_or_default();

        let allowed_prefixes =
            config.get("allowedPrefixes").and_then(|v| Set::try_from(v).ok()).unwrap_or_default();

        let ignore = Self::populate_ignores(ignore);

        Self(Box::new(PreferLowercaseTitleConfig {
            allowed_prefixes,
            ignore,
            ignore_top_level_describe,
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            self.run(possible_jest_node, ctx);
        }
    }
}

impl PreferLowercaseTitle {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ParsedJestFnCallNew::GeneralJestFnCall(jest_fn_call)) =
            parse_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        if self.ignore.contains_str(&jest_fn_call.name) {
            return;
        }

        match jest_fn_call.kind {
            JestFnKind::General(JestGeneralFnKind::Describe) => {
                if self.ignore_top_level_describe && node.scope_id() == ctx.scopes().root_scope_id()
                {
                    return;
                }
            }
            JestFnKind::General(JestGeneralFnKind::Test) => {}
            _ => return,
        }

        let Some(arg) = call_expr.arguments.first() else {
            return;
        };

        match arg {
            Argument::StringLiteral(string_expr) => {
                self.lint_string(ctx, string_expr.value.as_str(), string_expr.span);
            }
            Argument::TemplateLiteral(template_expr) => {
                let Some(template_string) = template_expr.quasi() else {
                    return;
                };
                self.lint_string(ctx, template_string.as_str(), template_expr.span);
            }
            _ => {}
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn populate_ignores(ignore: Set<CompactStr>) -> Set<CompactStr> {
        let mut ignores: Vec<&str> = vec![];
        let test_case_name = ["fit", "it", "xit", "test", "xtest"];
        let describe_alias = ["describe", "fdescribe", "xdescribe"];
        let test_name = "test";
        let it_name = "it";

        if ignore.iter().any(|alias| alias == "describe") {
            ignores.extend(describe_alias.iter());
        }

        if ignore.iter().any(|alias| alias == test_name) {
            ignores.extend(test_case_name.iter().filter(|alias| alias.ends_with(test_name)));
        }

        if ignore.iter().any(|alias| alias == it_name) {
            ignores.extend(test_case_name.iter().filter(|alias| alias.ends_with(it_name)));
        }

        ignores.into_iter().map(CompactStr::from).collect()
    }

    #[inline]
    fn is_allowed_literal(&self, literal: &str) -> bool {
        literal.is_empty()
            || self.allowed_prefixes.iter().any(|name| literal.starts_with(name.as_str()))
    }

    fn lint_string<'a>(&self, ctx: &LintContext<'a>, literal: &'a str, span: Span) {
        if self.is_allowed_literal(literal) {
            return;
        }

        let Some(first_char) = literal.chars().next() else {
            return;
        };

        let lower = first_char.to_ascii_lowercase();
        if first_char == lower {
            return;
        }

        ctx.diagnostic_with_fix(unexpected_lowercase(literal, span), |fixer| {
            fixer.replace(Span::sized(span.start + 1, 1), lower.to_string())
        });
    }
}

#[test]
fn debug() {
    let pass = vec![];
    let fail = vec![("describe(\"Foo\", function () {})", None)];
    crate::tester::Tester::new(PreferLowercaseTitle::NAME, pass, fail)
        .with_jest_plugin(true)
        .test();
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("it.each()", None),
        ("it.each()(1)", None),
        ("randomFunction()", None),
        ("foo.bar()", None),
        ("it()", None),
        ("it(' ', function () {})", None),
        ("it(true, function () {})", None),
        ("it(MY_CONSTANT, function () {})", None),
        ("it(\" \", function () {})", None),
        ("it(` `, function () {})", None),
        ("it('foo', function () {})", None),
        ("it(\"foo\", function () {})", None),
        ("it(`foo`, function () {})", None),
        ("it(\"<Foo/>\", function () {})", None),
        ("it(\"123 foo\", function () {})", None),
        ("it(42, function () {})", None),
        ("it(``)", None),
        ("it(\"\")", None),
        ("it(42)", None),
        ("test()", None),
        ("test('foo', function () {})", None),
        ("test(\"foo\", function () {})", None),
        ("test(`foo`, function () {})", None),
        ("test(\"<Foo/>\", function () {})", None),
        ("test(\"123 foo\", function () {})", None),
        ("test(\"42\", function () {})", None),
        ("test(``)", None),
        ("test(\"\")", None),
        ("test(42)", None),
        ("describe()", None),
        ("describe('foo', function () {})", None),
        ("describe(\"foo\", function () {})", None),
        ("describe(`foo`, function () {})", None),
        ("describe(\"<Foo/>\", function () {})", None),
        ("describe(\"123 foo\", function () {})", None),
        ("describe(\"42\", function () {})", None),
        ("describe(function () {})", None),
        ("describe(``)", None),
        ("describe(\"\")", None),
        (
            "
                describe.each()(1);
                describe.each()(2);
            ",
            None,
        ),
        ("jest.doMock(\"my-module\")", None),
        (
            "
                import { jest } from '@jest/globals';
                jest.doMock('my-module');
            ",
            None,
        ),
        ("describe(42)", None),
        (
            "describe(42)",
            Some(serde_json::json!([{ "ignore": "undefined", "allowedPrefixes": "undefined" }])),
        ),
        // ignore = describe
        ("describe('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        (
            "describe(\"Foo\", function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        ("describe(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        ("fdescribe(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        (
            "describe.skip(`Foo`, function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        // ignore=test
        ("test('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("test(\"Foo\", function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("test(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("xtest(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("test.only(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        // ignore=it
        ("it('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("it(\"Foo\", function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("it(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("fit(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("it.skip(`Foo`, function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        // allowedPrefixes
        (
            "it('GET /live', function () {})",
            Some(serde_json::json!([{ "allowedPrefixes": ["GET"] }])),
        ),
        (
            "it(\"POST /live\", function () {})",
            Some(serde_json::json!([{ "allowedPrefixes": ["GET", "POST"] }])),
        ),
        (
            "it(`PATCH /live`, function () {})",
            Some(serde_json::json!([{ "allowedPrefixes": ["GET", "PATCH"] }])),
        ),
        // ignoreTopLevelDescribe
        (
            "describe(\"MyClass\", () => {});",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('#myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('Strings', () => {
                    it('are strings', () => {
                        expect('abc').toBe('abc');
                    });
                });

                describe('Booleans', () => {
                    it('are booleans', () => {
                        expect(true).toBe(true);
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
    ];

    let fail = vec![
        ("it('Foo', function () {})", None),
        ("xit('Foo', function () {})", None),
        ("it(\"Foo\", function () {})", None),
        ("it(`Foo`, function () {})", None),
        ("test('Foo', function () {})", None),
        ("xtest('Foo', function () {})", None),
        ("test(\"Foo\", function () {})", None),
        ("test(`Foo`, function () {})", None),
        ("describe('Foo', function () {})", None),
        ("describe(\"Foo\", function () {})", None),
        ("describe(`Foo`, function () {})", None),
        (
            "
                import { describe as context } from '@jest/globals';
                context(`Foo`, () => {});
            ",
            None,
        ),
        ("describe(`Some longer description`, function () {})", None),
        ("fdescribe(`Some longer description`, function () {})", None),
        ("it.each(['green', 'black'])('Should return %', () => {})", None),
        ("describe.each(['green', 'black'])('Should return %', () => {})", None),
        // ignore describe
        ("test('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        ("xit('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["describe"] }]))),
        // ignore test
        ("describe('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("it('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        ("xit('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["test"] }]))),
        // ignore it
        ("describe('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("test('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        ("xtest('Foo', function () {})", Some(serde_json::json!([{ "ignore": ["it"] }]))),
        // allowedPrefixes

        // ignoreTopLevelDescribe
        (
            "it(\"Works!\", () => {});",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                import { describe, describe as context } from '@jest/globals';
                describe('MyClass', () => {
                    context('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": false }])),
        ),
    ];

    let fix = vec![
        ("it('Foo', function () {})", "it('foo', function () {})", None),
        ("xit('Foo', function () {})", "xit('foo', function () {})", None),
        ("it(\"Foo\", function () {})", "it(\"foo\", function () {})", None),
        ("it(`Foo`, function () {})", "it(`foo`, function () {})", None),
        ("test('Foo', function () {})", "test('foo', function () {})", None),
        ("xtest('Foo', function () {})", "xtest('foo', function () {})", None),
        ("test(\"Foo\", function () {})", "test(\"foo\", function () {})", None),
        ("test(`Foo`, function () {})", "test(`foo`, function () {})", None),
        ("describe('Foo', function () {})", "describe('foo', function () {})", None),
        ("describe(\"Foo\", function () {})", "describe(\"foo\", function () {})", None),
        ("describe(`Foo`, function () {})", "describe(`foo`, function () {})", None),
        (
            "
                import { describe as context } from '@jest/globals';
                context(`Foo`, () => {});
            ",
            "
                import { describe as context } from '@jest/globals';
                context(`foo`, () => {});
            ",
            None,
        ),
        (
            "describe(`Some longer description`, function () {})",
            "describe(`some longer description`, function () {})",
            None,
        ),
        (
            "fdescribe(`Some longer description`, function () {})",
            "fdescribe(`some longer description`, function () {})",
            None,
        ),
        (
            "it.each(['green', 'black'])('Should return %', () => {})",
            "it.each(['green', 'black'])('should return %', () => {})",
            None,
        ),
        (
            "describe.each(['green', 'black'])('Should return %', () => {})",
            "describe.each(['green', 'black'])('should return %', () => {})",
            None,
        ),
        // ignore describe
        (
            "test('Foo', function () {})",
            "test('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        (
            "xit('Foo', function () {})",
            "xit('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["describe"] }])),
        ),
        // ignore test
        (
            "describe('Foo', function () {})",
            "describe('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["test"] }])),
        ),
        (
            "it('Foo', function () {})",
            "it('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["test"] }])),
        ),
        (
            "xit('Foo', function () {})",
            "xit('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["test"] }])),
        ),
        // ignore it
        (
            "describe('Foo', function () {})",
            "describe('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["it"] }])),
        ),
        (
            "test('Foo', function () {})",
            "test('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["it"] }])),
        ),
        (
            "xtest('Foo', function () {})",
            "xtest('foo', function () {})",
            Some(serde_json::json!([{ "ignore": ["it"] }])),
        ),
        // allowedPrefixes empty
        // ignoreTopLevelDescribe
        (
            "it(\"Works!\", () => {});",
            "it(\"works!\", () => {});",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            "
                describe('MyClass', () => {
                    describe('myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                import { describe, describe as context } from '@jest/globals';
                describe('MyClass', () => {
                    context('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            "
                import { describe, describe as context } from '@jest/globals';
                describe('MyClass', () => {
                    context('myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": true }])),
        ),
        (
            "
                describe('MyClass', () => {
                    describe('MyMethod', () => {
                        it('Does things', () => {
                            //
                        });
                    });
                });
            ",
            "
                describe('myClass', () => {
                    describe('myMethod', () => {
                        it('does things', () => {
                            //
                        });
                    });
                });
            ",
            Some(serde_json::json!([{ "ignoreTopLevelDescribe": false }])),
        ),
    ];

    Tester::new(PreferLowercaseTitle::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
