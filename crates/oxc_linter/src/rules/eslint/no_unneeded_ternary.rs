use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::{
    AstKind,
    ast::{BinaryOperator, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn no_unneeded_ternary_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary use of boolean literals in conditional expression")
        .with_help("Remove this ternary operator")
        .with_label(span)
}

fn no_unneeded_ternary_conditional_expression_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary use of conditional expression for default assignment")
        .with_help("Remove this ternary operator and use the variable directly")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoUnneededTernary {
    default_assignment: bool,
}

impl Default for NoUnneededTernary {
    fn default() -> Self {
        Self { default_assignment: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow ternary operators when simpler alternatives exist
    ///
    /// ### Why is this bad?
    ///
    /// It’s a common mistake in JavaScript to use a conditional expression to select between two
    /// Boolean values instead of using ! to convert the test to a Boolean.
    ///
    /// Another common mistake is using a single variable as both the conditional test and the
    /// consequent. In such cases, the logical OR can be used to provide the same functionality.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const isYes = answer === 1 ? true : false;
    /// const isNo = answer === 1 ? false : true;
    ///
    /// foo(bar ? bar : 1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const isYes = answer === 1;
    /// const isNo = answer !== 1;
    ///
    /// foo(bar || 1);
    /// ```
    NoUnneededTernary,
    eslint,
    suspicious,
    fix_dangerous
);

impl Rule for NoUnneededTernary {
    fn from_configuration(value: serde_json::Value) -> Self {
        let default_assignment = value
            .get(0)
            .and_then(|v| v.get("defaultAssignment"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        Self { default_assignment }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ConditionalExpression(expr) = node.kind() else {
            return;
        };
        if matches!(expr.consequent, Expression::BooleanLiteral(_))
            && matches!(expr.alternate, Expression::BooleanLiteral(_))
        {
            ctx.diagnostic_with_dangerous_fix(no_unneeded_ternary_diagnostic(expr.span), |fixer| {
                let (Expression::BooleanLiteral(left), Expression::BooleanLiteral(right)) =
                    (&expr.consequent, &expr.alternate)
                else {
                    return fixer.noop();
                };
                if left.value == right.value {
                    return fixer.replace_with(expr, &expr.consequent);
                }
                let replacement;
                let test_expr_source = ctx.source_range(expr.test.span());
                match &expr.test {
                    Expression::BinaryExpression(binary) => {
                        if left.value {
                            replacement = test_expr_source.to_string();
                        } else {
                            replacement = match binary.operator {
                                BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                                    let op = if matches!(binary.operator, BinaryOperator::Equality)
                                    {
                                        "!="
                                    } else {
                                        "!=="
                                    };
                                    format!(
                                        "{} {op} {}",
                                        ctx.source_range(binary.left.span()),
                                        ctx.source_range(binary.right.span())
                                    )
                                }
                                BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                                    let op =
                                        if matches!(binary.operator, BinaryOperator::Inequality) {
                                            "=="
                                        } else {
                                            "==="
                                        };
                                    format!(
                                        "{} {op} {}",
                                        ctx.source_range(binary.left.span()),
                                        ctx.source_range(binary.right.span())
                                    )
                                }
                                _ => {
                                    format!("!({test_expr_source})")
                                }
                            };
                        }
                    }
                    Expression::UnaryExpression(unary) if left.value && unary.operator.is_not() => {
                        // !x ? true : false => !x
                        replacement = test_expr_source.to_string();
                    }
                    _ => {
                        let prefix = if left.value { "!!" } else { "!" };
                        replacement = if without_parenthesize(&expr.test) {
                            format!("{prefix}{test_expr_source}")
                        } else {
                            format!("{prefix}({test_expr_source})")
                        };
                    }
                }
                fixer.replace(expr.span, replacement)
            });
        } else if let (Some(test), Some(cons)) = (
            (&expr.test.get_inner_expression().get_identifier_reference()),
            (&expr.consequent.get_inner_expression().get_identifier_reference()),
        ) {
            if !self.default_assignment && test.name == cons.name {
                ctx.diagnostic_with_dangerous_fix(
                    no_unneeded_ternary_conditional_expression_diagnostic(expr.span),
                    |fixer| {
                        // x ? x : 1 => x || 1
                        // x ? x : y ? 1 : 2 => x || (y ? 1 : 2)
                        let prefix = ctx.source_range(expr.test.span());
                        let alternate_str = ctx.source_range(expr.alternate.span());
                        let suffix = if expr.alternate.is_primary_expression() {
                            alternate_str.to_string()
                        } else {
                            format!("({alternate_str})")
                        };
                        let replacement = format!("{prefix} || {suffix}");
                        fixer.replace(expr.span, replacement)
                    },
                );
            }
        }
    }
}

fn without_parenthesize(node: &Expression) -> bool {
    matches!(
        node,
        Expression::Identifier(_)
            | Expression::UnaryExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::UpdateExpression(_)
            | Expression::CallExpression(_)
            | Expression::ChainExpression(_)
            | Expression::ImportExpression(_)
            | Expression::NewExpression(_)
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("config.newIsCap = config.newIsCap !== false", None),
        ("var a = x === 2 ? 'Yes' : 'No';", None),
        ("var a = x === 2 ? true : 'No';", None),
        ("var a = x === 2 ? 'Yes' : false;", None),
        ("var a = x === 2 ? 'true' : 'false';", None),
        ("var a = foo ? foo : bar;", None),
        (
            "var value = 'a';var canSet = true;var result = value || (canSet ? 'unset' : 'can not set')",
            None,
        ),
        ("var a = foo ? bar : foo;", None),
        ("foo ? bar : foo;", None),
        ("var a = f(x ? x : 1)", None),
        ("f(x ? x : 1);", None),
        ("foo ? foo : bar;", None),
        ("var a = foo ? 'Yes' : foo;", None),
        ("var a = foo ? 'Yes' : foo;", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        ("var a = foo ? bar : foo;", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        ("foo ? bar : foo;", Some(serde_json::json!([{ "defaultAssignment": false }]))),
    ];

    let fail = vec![
        ("var a = x === 2 ? true : false;", None),
        ("var a = (x === 2) ? true : false;", None),
        ("var a = x >= 2 ? true : false;", None),
        ("var a = x ? true : false;", None),
        ("var a = x === 1 ? false : true;", None),
        ("var a = x != 1 ? false : true;", None),
        ("var a = foo() ? false : true;", None),
        ("var a = !foo() ? false : true;", None),
        ("var a = foo + bar ? false : true;", None),
        ("var a = x instanceof foo ? false : true;", None),
        ("var a = foo ? false : false;", None),
        ("var a = foo() ? false : false;", None),
        ("var a = x instanceof foo ? true : false;", None),
        ("var a = !foo ? true : false;", None),
        (
            "
			                var value = 'a'
			                var canSet = true
			                var result = value ? value : canSet ? 'unset' : 'can not set'
			            ",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "foo ? foo : (bar ? baz : qux)",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "function* fn() { foo ? foo : yield bar }",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ), // { "ecmaVersion": 6 },
        ("var a = foo ? foo : 'No';", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        (
            "var a = ((foo)) ? (((((foo))))) : ((((((((((((((bar))))))))))))));",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        ("var a = b ? b : c => c;", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2015 },
        ("var a = b ? b : c = 0;", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2015 },
        ("var a = b ? b : (c => c);", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2015 },
        ("var a = b ? b : (c = 0);", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2015 },
        ("var a = b ? b : (c) => (c);", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2015 },
        (
            "var a = b ? b : c, d; // this is ((b ? b : c), (d))",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ), // { "ecmaVersion": 2015 },
        ("var a = b ? b : (c, d);", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2015 },
        ("f(x ? x : 1);", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        ("x ? x : 1;", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        ("var a = foo ? foo : bar;", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        ("var a = foo ? foo : a ?? b;", Some(serde_json::json!([{ "defaultAssignment": false }]))), // { "ecmaVersion": 2020 },
        ("foo as any ? false : true", None), // {                "parser": require(parser("typescript-parsers/unneeded-ternary-1")),                "ecmaVersion": 6            },
        ("foo ? foo : bar as any", Some(serde_json::json!([{ "defaultAssignment": false }]))), // {                "parser": require(parser("typescript-parsers/unneeded-ternary-2")),                "ecmaVersion": 6            }
    ];

    // I keep the fix tets commented until they are implemented
    let fix = vec![
        ("var a = x === 2 ? true : false;", "var a = x === 2;", None),
        ("var a = x >= 2 ? true : false;", "var a = x >= 2;", None),
        ("var a = x ? true : false;", "var a = !!x;", None),
        ("var a = x === 1 ? false : true;", "var a = x !== 1;", None),
        ("var a = x != 1 ? false : true;", "var a = x == 1;", None),
        ("var a = foo() ? false : true;", "var a = !foo();", None),
        ("var a = !foo() ? false : true;", "var a = !!foo();", None),
        ("var a = foo + bar ? false : true;", "var a = !(foo + bar);", None),
        ("var a = x instanceof foo ? false : true;", "var a = !(x instanceof foo);", None),
        ("var a = foo ? false : false;", "var a = false;", None),
        ("var a = x instanceof foo ? true : false;", "var a = x instanceof foo;", None),
        ("var a = !foo ? true : false;", "var a = !foo;", None),
        (
            "var result = value ? value : canSet ? 'unset' : 'can not set'",
            "var result = value || (canSet ? 'unset' : 'can not set')",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "foo ? foo : (bar ? baz : qux)",
            "foo || (bar ? baz : qux)",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "function* fn() { foo ? foo : yield bar }",
            "function* fn() { foo || (yield bar) }",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = foo ? foo : 'No';",
            "var a = foo || 'No';",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = ((foo)) ? (((((foo))))) : ((((((((((((((bar))))))))))))));",
            "var a = ((foo)) || ((((((((((((((bar))))))))))))));",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : c => c;",
            "var a = b || (c => c);",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : c = 0;",
            "var a = b || (c = 0);",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : (c => c);",
            "var a = b || (c => c);",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : (c = 0);",
            "var a = b || (c = 0);",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : (c) => (c);",
            "var a = b || ((c) => (c));",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : c, d; // this is ((b ? b : c), (d))",
            "var a = b || c, d; // this is ((b ? b : c), (d))",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = b ? b : (c, d);",
            "var a = b || (c, d);",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        ("f(x ? x : 1);", "f(x || 1);", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        ("x ? x : 1;", "x || 1;", Some(serde_json::json!([{ "defaultAssignment": false }]))),
        (
            "var a = foo ? foo : bar;",
            "var a = foo || bar;",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "var a = foo ? foo : a ?? b;",
            "var a = foo || (a ?? b);",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        ("foo as any ? false : true", "!(foo as any)", None),
        (
            "foo ? foo : bar as any",
            "foo || (bar as any)",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        (
            "a ? a : b ?? 2",
            "a || (b ?? 2)",
            Some(serde_json::json!([{ "defaultAssignment": false }])),
        ),
        ("let a = {} satisfies User ? true : false", "let a = !!({} satisfies User)", None),
    ];
    Tester::new(NoUnneededTernary::NAME, NoUnneededTernary::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
