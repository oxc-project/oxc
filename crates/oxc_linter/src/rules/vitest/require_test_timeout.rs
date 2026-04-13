use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, ObjectPropertyKind, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    rules::PossibleJestNode,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionProperty,
        collect_possible_jest_call_node, parse_general_jest_fn_call,
    },
};

fn test_missing_timeout(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test is missing a timeout.")
        .with_help(
            "Add a numeric third argument, a `{ timeout }` option object second argument, or call `vi.setConfig({ testTimeout: ... })` before this test.",
        )
        .with_label(span)
}

fn config_missing_timeout_object(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`vi.setConfig()` is missing a `testTimeout` property.")
        .with_help("Pass an object with a `testTimeout` property to `vi.setConfig()`.")
        .with_label(span)
}

fn test_options_missing_timeout_property(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test options object is missing a `timeout` property.")
        .with_help("Add a `timeout` property to the options object.")
        .with_label(span)
}

fn timeout_must_be_a_number(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Timeout must be a number.")
        .with_help("Use a non-negative numeric literal for the timeout value.")
        .with_label(span)
}

fn timeout_must_be_non_negative(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Timeout must not be negative.")
        .with_help("Use a non-negative numeric literal for the timeout value.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireTestTimeout;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires every test to have a timeout specified, either as a numeric third
    /// argument, a `{ timeout }` option, or via `vi.setConfig({ testTimeout: ... })`.
    ///
    /// ### Why is this bad?
    ///
    /// Tests without an explicit timeout rely on the default, which may be too
    /// generous to catch performance regressions or too short for slow CI
    /// environments, leading to flaky failures.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// it('slow test', async () => {
    ///   await doSomethingSlow()
    /// })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // good (numeric timeout)
    /// test('slow test', async () => {
    ///   await doSomethingSlow()
    /// }, 1000)
    ///
    /// // good (options object)
    /// test('slow test', { timeout: 1000 }, async () => {
    ///   await doSomethingSlow()
    /// })
    ///
    /// // good (file-level)
    /// vi.setConfig({ testTimeout: 1000 })
    ///
    /// test('slow test', async () => {
    ///   await doSomethingSlow()
    /// })
    /// ```
    RequireTestTimeout,
    vitest,
    restriction,
    version = "1.58.0",
);

impl Rule for RequireTestTimeout {
    fn run_once(&self, ctx: &LintContext) {
        let mut config_positions: Vec<Span> = vec![];
        let mut possible_jest_nodes = collect_possible_jest_call_node(ctx);
        possible_jest_nodes.sort_unstable_by_key(|n| n.node.id());

        for possible_jest_node in possible_jest_nodes {
            Self::run_rule(&possible_jest_node, &mut config_positions, ctx);
        }
    }
}

impl RequireTestTimeout {
    pub fn run_rule<'a>(
        possible_jest_node: &PossibleJestNode<'a, '_>,
        config_positions: &mut Vec<Span>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(vi_node) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
            return;
        };

        match vi_node.kind {
            JestFnKind::General(JestGeneralFnKind::Vitest) => {
                if !vi_node.members.first().is_some_and(|member| member.is_name_equal("setConfig"))
                {
                    return;
                }

                if let Some(Argument::ObjectExpression(test_config)) = call_expr.arguments.first() {
                    let Some(ObjectPropertyKind::ObjectProperty(property)) = test_config
                        .properties
                        .iter()
                        .find(|property| is_property_name_equals(property, "testTimeout"))
                    else {
                        return;
                    };

                    config_positions.push(call_expr.span);
                    config_positions.sort_by(|config_a, config_b| {
                        (config_a.end - config_b.end).cmp(&config_a.end)
                    });

                    parse_timeout_value(&property.value, property.value.span(), ctx);
                } else {
                    ctx.diagnostic(config_missing_timeout_object(call_expr.span()));
                }
            }
            JestFnKind::General(JestGeneralFnKind::Test) => {
                if vi_node.members.iter().any(is_todo_or_skipped) || vi_node.name.starts_with('x') {
                    return;
                }

                if !call_expr.arguments.iter().any(|argument| {
                    matches!(
                        argument,
                        Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_)
                    )
                }) {
                    return;
                }

                // test() and it() only have two options, a second parameter with options or a last argument with a number

                if let Some(Argument::ObjectExpression(test_config)) = call_expr.arguments.get(1) {
                    let Some(ObjectPropertyKind::ObjectProperty(property)) = test_config
                        .properties
                        .iter()
                        .find(|property| is_property_name_equals(property, "timeout"))
                    else {
                        ctx.diagnostic(test_options_missing_timeout_property(test_config.span()));
                        return;
                    };

                    parse_timeout_value(&property.value, property.value.span(), ctx);
                } else if let Some(last_argument) = call_expr.arguments.get(2) {
                    let Some(argument_expression) = last_argument.as_expression() else {
                        return;
                    };

                    parse_timeout_value(argument_expression, last_argument.span(), ctx);
                } else {
                    if config_positions
                        .last()
                        .is_some_and(|config_position| config_position.end < call_expr.span.start)
                    {
                        return;
                    }

                    ctx.diagnostic(test_missing_timeout(call_expr.span()));
                }
            }
            _ => {}
        }
    }
}

fn is_todo_or_skipped(member: &KnownMemberExpressionProperty<'_>) -> bool {
    member.is_name_equal("todo") || member.is_name_equal("skip")
}

fn is_property_name_equals(property: &ObjectPropertyKind<'_>, name: &str) -> bool {
    let ObjectPropertyKind::ObjectProperty(object_pair) = property else {
        return false;
    };

    let Some(object_key_name) = object_pair.key.static_name() else {
        return false;
    };

    object_key_name == name
}

fn parse_timeout_value(expression: &Expression<'_>, span: Span, ctx: &LintContext<'_>) {
    match expression {
        Expression::NumericLiteral(_) => {}
        Expression::UnaryExpression(expression) => {
            let Expression::NumericLiteral(_) = &expression.argument else {
                ctx.diagnostic(timeout_must_be_a_number(span));
                return;
            };

            if matches!(expression.operator, UnaryOperator::UnaryPlus) {
                return;
            }

            ctx.diagnostic(timeout_must_be_non_negative(span));
        }
        _ => ctx.diagnostic(timeout_must_be_a_number(span)),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    /*
     * Commented tests are invalid because Vitest doesn't allow that API for timeout https://vitest.dev/api/test.html#timeout
     */
    let pass = vec![
        r#"test.todo("a")"#,
        r#"xit("a", () => {})"#,
        r#"test("a", () => {}, 0)"#,
        r#"it("a", () => {}, 500)"#,
        r#"it.skip("a", () => {})"#,
        r#"test.skip("a", () => {})"#,
        r#"test("a", () => {}, 1000)"#,
        r#"it.only("a", () => {}, 1234)"#,
        r#"test.only("a", () => {}, 1234)"#,
        r#"it.concurrent("a", () => {}, 400)"#,
        //r#"test("a", () => {}, { timeout: 0 })"#,
        r#"test.concurrent("a", () => {}, 400)"#,
        //r#"test("a", () => {}, { timeout: 500 })"#,
        r#"test("a", { timeout: 500 }, () => {})"#,
        r#"vi.setConfig({ testTimeout: 1000 }); test("a", () => {})"#,
        //r#"test("a", { foo: 1 }, { timeout: 500 }, () => {})"#,
        r#"test("a", { timeout: 500 }, 1000, () => {})"#,
        r#"test("a", () => {}, 1000, { extra: true })"#,
        r#"test("a", () => {}, +500)"#,
        r#"vi.setConfig({ testTimeout: +500 }); test("a", () => {})"#,
        r#"test("a", { timeout: +500 }, () => {})"#,
        r#"vi.setConfig({ testTimeout: 0 }); test("a", () => {})"#,
        r#"vi.setConfig({ testTimeout: 1000 }); test("a", () => {}); vi.setConfig({ testTimeout: 2000 }); test("b", () => {})"#,
        r#"vi.setConfig({ testTimeout: 1000 }); test("a", () => {}, 500)"#,
        r#"describe("suite", () => {})"#,
    ];

    let fail = vec![
        r#"test("a", () => {})"#,
        r#"it("a", () => {})"#,
        r#"test.only("a", () => {})"#,
        r#"test.concurrent("a", () => {})"#,
        r#"it.concurrent("a", () => {})"#,
        r#"vi.setConfig({}); test("a", () => {})"#,
        r#"const t = 500; test("a", { timeout: t }, () => {})"#,
        r#"test("a", () => {}, { timeout: null })"#,
        r#"test("a", () => {}, { timeout: undefined })"#,
        r#"test("a", () => {}, -100)"#,
        r#"test("a", () => {}, { timeout: -1 })"#,
        r#"vi.setConfig({ testTimeout: null }); test("a", () => {})"#,
        r#"vi.setConfig({ testTimeout: undefined }); test("a", () => {})"#,
        r#"test("a", () => {}); vi.setConfig({ testTimeout: 1000 })"#,
        r#"const opts = { timeout: 1000 }; test("a", { ...opts }, () => {})"#,
        r#"const opts = { timeout: 1000 }; test("a", { ...opts }, { foo: 1 }, () => {})"#,
        r#"test("a", () => {}, { timeout: -1 }, { timeout: 500 })"#,
        r#"test("a", () => {}, { timeout: 500 }, { timeout: -1 })"#,
        r#"test("a", () => {}, { timeout: -1 }, 1000)"#,
        //r#"test("a", () => {}, 1000, { timeout: -1 })"#,
        r#"test("a", () => {}, null)"#,
        r#"test("a", () => {}, "1000")"#,
        r#"test("a", { timeout: "1000" }, () => {})"#,
        r#"vi.setConfig({ testTimeout: "1000" })"#,
        r#"vi.setConfig({ testTimeout: -"1" })"#,
        r#"test("a", { timeout: null }, () => {})"#,
        r#"test("a", () => {}, undefined)"#,
        r#"vi.setConfig({ testTimeout: -100 }); test("a", () => {})"#,
        r#"test("a", { timeout: -500 }, () => {})"#,
        r#"vi.setConfig("invalid"); test("a", () => {})"#,
        r#"test("a", { retries: 3 }, () => {})"#,
        r#"vi.setConfig({ testTimeout: 1000 }); test("a", () => {}); vi.setConfig({ testTimeout: null }); test("b", () => {})"#,
        r#"vi.setConfig({ testTimeout: 1000 }); test("a", () => {}, -1)"#,
    ];

    Tester::new(RequireTestTimeout::NAME, RequireTestTimeout::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
