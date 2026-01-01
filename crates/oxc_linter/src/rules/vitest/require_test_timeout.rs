use crate::{
    context::LintContext,
    rule::Rule,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
fn require_test_timeout_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Test is missing a timeout.")
        .with_help("Add a timeout to prevent tests from hanging indefinitely.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireTestTimeout;
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all Vitest test cases (`test`, `it`) have an explicit timeout defined.
    ///
    /// ### Why is this bad?
    ///
    /// Tests without timeouts can hang indefinitely, blocking CI/CD pipelines and wasting resources.
    /// Explicit timeouts ensure tests fail fast when they encounter issues, improving the development workflow.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// test('my test', () => {
    ///     expect(true).toBe(true);
    /// });
    ///
    /// it('another test', async () => {
    ///     await someAsyncOperation();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// test('my test', () => {
    ///     expect(true).toBe(true);
    /// }, 5000);
    ///
    /// it('another test', async () => {
    ///     await someAsyncOperation();
    /// }, 10000);
    ///
    /// test('with options', () => {
    ///     expect(true).toBe(true);
    /// }, { timeout: 5000 });
    /// ```
    RequireTestTimeout,
    vitest,
    restriction
);

impl Rule for RequireTestTimeout {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) -> Option<()> {
    let node = possible_jest_node.node;
    let call_expr = node.kind().as_call_expression()?;

    if !is_type_of_jest_fn_call(
        call_expr,
        possible_jest_node,
        ctx,
        &[JestFnKind::General(JestGeneralFnKind::Test)],
    ) {
        return None;
    }

    let has_timeout = check_has_timeout(call_expr);

    if !has_timeout {
        ctx.diagnostic(require_test_timeout_diagnostic(call_expr.span));
    }

    None
}

fn is_valid_unary_timeout(unary: &oxc_ast::ast::UnaryExpression) -> bool {
    // Only accept unary + operator with numeric literals, reject - (negative values)
    matches!(unary.operator, oxc_ast::ast::UnaryOperator::UnaryPlus)
        && matches!(unary.argument.get_inner_expression(), Expression::NumericLiteral(_))
}

fn check_has_timeout(call_expr: &oxc_ast::ast::CallExpression) -> bool {
    let args = &call_expr.arguments;

    // Check for timeout as third argument
    // Numeric: test('name', () => {}, 5000)
    // Options object: test('name', () => {}, { timeout: 5000 })
    if args.len() >= 3
        && let Some(third_arg) = args.get(2).and_then(|arg| arg.as_expression())
    {
        let inner = third_arg.get_inner_expression();

        // Check for numeric timeout
        if matches!(inner, Expression::NumericLiteral(_)) {
            return true;
        }

        // Accept identifiers or member expressions as references to constants, but reject known invalid names
        if let Expression::Identifier(identifier) = inner {
            let name = identifier.name.as_str();

            if name != "undefined" && name != "null" && name != "NaN" {
                return true;
            }
        }

        // Accept member expressions like `config.TIMEOUT`
        if matches!(inner, Expression::StaticMemberExpression(_)) {
            return true;
        }

        // Accept only unary + expressions with numeric literals (reject negative values)
        if let Expression::UnaryExpression(unary) = inner
            && is_valid_unary_timeout(unary)
        {
            return true;
        }

        // Check for options object with a valid timeout property
        if let Expression::ObjectExpression(obj_expr) = inner {
            for prop in &obj_expr.properties {
                let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
                    continue;
                };

                // Check if property key is "timeout" (as identifier or string literal)
                let is_timeout_key = match &obj_prop.key {
                    oxc_ast::ast::PropertyKey::StaticIdentifier(ident) => ident.name == "timeout",
                    oxc_ast::ast::PropertyKey::StringLiteral(lit) => lit.value == "timeout",
                    _ => false,
                };

                if !is_timeout_key {
                    continue;
                }

                // Validate the property's value
                match &obj_prop.value {
                    Expression::NumericLiteral(_) => return true,
                    Expression::Identifier(ident) => {
                        let name = ident.name.as_str();
                        if name != "undefined" && name != "null" && name != "NaN" {
                            return true;
                        }
                    }
                    Expression::StaticMemberExpression(_) => {
                        return true;
                    }
                    Expression::UnaryExpression(unary) => {
                        if is_valid_unary_timeout(unary) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Numeric timeout as third argument
        r#"test("test with timeout", () => {}, 5000);"#,
        r#"it("test with timeout", () => {}, 10000);"#,
        // Options object with timeout as third argument
        r#"test("test with options", () => {}, { timeout: 5000 });"#,
        r#"it("test with options", () => {}, { timeout: 10000 });"#,
        // String literal property keys
        r#"test("string key timeout", () => {}, { 'timeout': 5000 });"#,
        r#"test("string key timeout", () => {}, { "timeout": 5000 });"#,
        // Identifier references
        r#"test("identifier timeout", () => {}, TIMEOUT);"#,
        r#"test("options identifier timeout", () => {}, { timeout: TIMEOUT });"#,
        // Static member expressions
        r#"test("member timeout", () => {}, config.TIMEOUT);"#,
        r#"test("member timeout in options", () => {}, { timeout: config.TIMEOUT });"#,
        // Test modifiers with timeout
        r#"test.skip("skipped test", () => {}, 5000);"#,
        r#"test.only("only test", () => {}, 5000);"#,
        r#"test.concurrent("concurrent test", () => {}, 5000);"#,
        // Unary + with numeric literals (explicitly positive)
        r#"test("unary plus timeout", () => {}, +5000);"#,
        r#"test("unary plus in options", () => {}, { timeout: +10000 });"#,
    ];

    let fail = vec![
        // Missing timeout
        r#"test("test without timeout", () => {});"#,
        r#"it("test without timeout", () => {});"#,
        // Test modifiers without timeout
        r#"test.skip("skipped test", () => {});"#,
        r#"test.only("only test", () => {});"#,
        r#"test.concurrent("concurrent test", () => {});"#,
        // Invalid identifiers as timeout
        r#"test("bad identifier timeout", () => {}, undefined);"#,
        r#"test("bad identifier timeout", () => {}, null);"#,
        r#"test("bad identifier timeout", () => {}, NaN);"#,
        r#"test("options undefined", () => {}, { timeout: undefined });"#,
        r#"test("options null", () => {}, { timeout: null });"#,
        // Options without timeout
        r#"test("options missing timeout", () => {}, { retry: 3 });"#,
        r#"test("options empty", () => {}, {});"#,
        // Negative timeout values (invalid)
        r#"test("negative timeout", () => {}, -5000);"#,
        r#"it("negative timeout", () => {}, -10000);"#,
        r#"test("negative timeout in options", () => {}, { timeout: -5000 });"#,
        r#"it("negative timeout in options", () => {}, { timeout: -10000 });"#,
    ];

    Tester::new(RequireTestTimeout::NAME, RequireTestTimeout::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
