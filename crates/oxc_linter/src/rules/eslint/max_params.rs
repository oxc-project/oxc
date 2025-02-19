use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

fn max_params_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(x0.to_string())
        .with_help(
            "This rule enforces a maximum number of parameters allowed in function definitions.",
        )
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct MaxParams(Box<MaxParamsConfig>);

#[derive(Debug, Clone)]
pub struct MaxParamsConfig {
    max: usize,
}

impl std::ops::Deref for MaxParams {
    type Target = MaxParamsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxParamsConfig {
    fn default() -> Self {
        Self { max: 3 }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce a maximum number of parameters in function definitions
    ///
    /// ### Why is this bad?
    /// Functions that take numerous parameters can be difficult to read and write because it requires the memorization of what each parameter is, its type, and the order they should appear in. As a result, many coders adhere to a convention that caps the number of parameters a function can take.
    ///
    /// ### Example
    /// ```javascript
    /// function foo (bar, baz, qux, qxx) {
    ///     doSomething();
    /// }
    /// ```
    MaxParams,
    eslint,
    style
);

impl Rule for MaxParams {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Self(Box::new(MaxParamsConfig { max }))
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(3, |v| usize::try_from(v).unwrap_or(3));

            Self(Box::new(MaxParamsConfig { max }))
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(function) => {
                if !function.is_declaration() & !function.is_expression() {
                    return;
                }

                if function.params.items.len() > self.max {
                    if let Some(id) = &function.id {
                        let function_name = id.name.as_str();
                        let error_msg = format!(
                            "Function '{}' has too many parameters ({}). Maximum allowed is {}.",
                            function_name,
                            function.params.items.len(),
                            self.max
                        );
                        let span = function.params.span;
                        ctx.diagnostic(max_params_diagnostic(
                            &error_msg,
                            Span::new(span.start, span.end),
                        ));
                    } else {
                        let error_msg = format!(
                            "Function has too many parameters ({}). Maximum allowed is {}.",
                            function.params.items.len(),
                            self.max
                        );
                        let span = function.params.span;
                        ctx.diagnostic(max_params_diagnostic(
                            &error_msg,
                            Span::new(span.start, span.end),
                        ));
                    }
                }
            }
            AstKind::ArrowFunctionExpression(function) => {
                if function.params.items.len() > self.max {
                    let error_msg = format!(
                        "Arrow function has too many parameters ({}). Maximum allowed is {}.",
                        function.params.items.len(),
                        self.max
                    );
                    let span = function.params.span;
                    ctx.diagnostic(max_params_diagnostic(
                        &error_msg,
                        Span::new(span.start, span.end),
                    ));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function test(d, e, f) {}", None),
        ("var test = function(a, b, c) {};", Some(serde_json::json!([3]))),
        ("var test = (a, b, c) => {};", Some(serde_json::json!([3]))),
        ("var test = function test(a, b, c) {};", Some(serde_json::json!([3]))),
        ("var test = function(a, b, c) {};", Some(serde_json::json!([{ "max": 3 }]))),
    ];

    let fail = vec![
        ("function test(a, b, c) {}", Some(serde_json::json!([2]))),
        ("function test(a, b, c, d) {}", None),
        ("var test = function(a, b, c, d) {};", Some(serde_json::json!([3]))),
        ("var test = (a, b, c, d) => {};", Some(serde_json::json!([3]))),
        ("(function(a, b, c, d) {});", Some(serde_json::json!([3]))),
        ("var test = function test(a, b, c) {};", Some(serde_json::json!([1]))),
        ("function test(a, b, c) {}", Some(serde_json::json!([{ "max": 2 }]))),
        ("function test(a, b, c, d) {}", Some(serde_json::json!([{}]))),
        ("function test(a) {}", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "function test(a, b, c) {
			              // Just to make it longer
			            }",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
    ];

    Tester::new(MaxParams::NAME, MaxParams::PLUGIN, pass, fail).test_and_snapshot();
}
