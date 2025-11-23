use oxc_ast::{AstKind, ast::TSType};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn max_params_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string())
        .with_help(
            "This rule enforces a maximum number of parameters allowed in function definitions.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MaxParams(Box<MaxParamsConfig>);

#[derive(Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxParamsConfig {
    /// Maximum number of parameters allowed in function definitions.
    max: usize,
    /// This option is for counting the `this` parameter if it is of type `void`.
    ///
    /// For example `{ "countVoidThis": true }` would mean that having a function
    /// take a `this` parameter of type `void` is counted towards the maximum number of parameters.
    count_void_this: bool,
}

impl std::ops::Deref for MaxParams {
    type Target = MaxParamsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxParamsConfig {
    fn default() -> Self {
        Self { max: 3, count_void_this: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum number of parameters in function definitions which by
    /// default is three.
    ///
    /// ### Why is this bad?
    ///
    /// Functions that take numerous parameters can be difficult to read and
    /// write because it requires the memorization of what each parameter is,
    /// its type, and the order they should appear in. As a result, many coders
    /// adhere to a convention that caps the number of parameters a function
    /// can take.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo (bar, baz, qux, qxx) {
    ///     doSomething();
    /// }
    /// ```
    ///
    /// ```javascript
    /// let foo = (bar, baz, qux, qxx) => {
    ///     doSomething();
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo (bar, baz, qux) {
    ///     doSomething();
    /// }
    /// ```
    ///
    /// ```javascript
    /// let foo = (bar, baz, qux) => {
    ///     doSomething();
    /// };
    /// ```
    MaxParams,
    eslint,
    style,
    config = MaxParamsConfig,
);

impl Rule for MaxParams {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Self(Box::new(MaxParamsConfig { max, count_void_this: false }))
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(3, |v| usize::try_from(v).unwrap_or(3));
            let count_void_this = config
                .and_then(|config| config.get("countVoidThis"))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            Self(Box::new(MaxParamsConfig { max, count_void_this }))
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(function) => {
                if !function.is_declaration() & !function.is_expression() {
                    return;
                }
                let params = &function.params;
                let mut real_len = params.items.len();
                if let Some(this_params) = &function.this_param {
                    let is_void_this = this_params
                        .type_annotation
                        .as_ref()
                        .is_some_and(|t| matches!(t.type_annotation, TSType::TSVoidKeyword(_)));
                    if self.count_void_this || !is_void_this {
                        real_len += 1;
                    }
                }

                if real_len > self.max {
                    if let Some(id) = &function.id {
                        let function_name = id.name.as_str();
                        let error_msg = format!(
                            "Function '{}' has too many parameters ({}). Maximum allowed is {}.",
                            function_name, real_len, self.max
                        );
                        let span = function.params.span;
                        ctx.diagnostic(max_params_diagnostic(&error_msg, span));
                    } else {
                        let error_msg = format!(
                            "Function has too many parameters ({}). Maximum allowed is {}.",
                            real_len, self.max
                        );
                        let span = function.params.span;
                        ctx.diagnostic(max_params_diagnostic(&error_msg, span));
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
                    ctx.diagnostic(max_params_diagnostic(&error_msg, span));
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
        (
            "function testD(this: void, a) {}",
            Some(serde_json::json!([{ "max": 2, "countVoidThis": true }])),
        ),
        (
            "function testD(this: void, a, b) {}",
            Some(serde_json::json!([{ "max": 2, "countVoidThis": false }])),
        ),
        ("const testE = function (this: void, a) {}", Some(serde_json::json!([1]))),
        (
            "const testE = function (this: void, a) {}",
            Some(serde_json::json!([{ "max": 2, "countVoidThis": false }])),
        ),
        (
            "const testE = function (this: any, a) {}",
            Some(serde_json::json!([{ "max": 2, "countVoidThis": true }])),
        ),
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
        (
            "function testD(this: void, a, b) {}",
            Some(serde_json::json!([{ "max": 2, "countVoidThis": true }])),
        ),
        (
            "
                class Foo { method(this: void, a) {} }
            ",
            Some(serde_json::json!([{ "max": 1, "countVoidThis": true }])),
        ),
        (
            "const testE = function (this: void, a) {}",
            Some(serde_json::json!([{ "max": 1, "countVoidThis": true }])),
        ),
        (
            "const testE = function (this: any, a) {}",
            Some(serde_json::json!([{ "max": 1, "countVoidThis": true }])),
        ),
        (
            "const testE = function (this: any, a) {}",
            Some(serde_json::json!([{ "max": 1, "countVoidThis": false }])),
        ),
    ];

    Tester::new(MaxParams::NAME, MaxParams::PLUGIN, pass, fail).test_and_snapshot();
}
