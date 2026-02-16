use oxc_ast::{
    AstKind,
    ast::{TSFunctionType, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn max_params_diagnostic(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string())
        .with_help(
            "This rule enforces a maximum number of parameters allowed in function definitions.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MaxParams(Box<MaxParamsConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MaxParamsConfig {
    /// Maximum number of parameters allowed in function definitions.
    #[serde(alias = "maximum")]
    max: usize,
    /// This option controls when to count a `this` parameter.
    ///
    /// - "always": always count `this`
    /// - "never": never count `this`
    /// - "except-void": count `this` only when it is not type `void`
    count_this: Option<CountThis>,
    /// Deprecated alias for `countThis`.
    ///
    /// For example `{ "countVoidThis": true }` would mean that having a function
    /// take a `this` parameter of type `void` is counted towards the maximum number of parameters.
    count_void_this: bool,
}

#[derive(Debug, Clone, Copy, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CountThis {
    /// Always count `this` as a parameter.
    Always,
    /// Never count `this` as a parameter.
    Never,
    /// Count `this` unless it is explicitly typed as `void`.
    #[default]
    ExceptVoid,
}

impl std::ops::Deref for MaxParams {
    type Target = MaxParamsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxParamsConfig {
    fn default() -> Self {
        Self { max: 3, count_this: None, count_void_this: false }
    }
}

impl MaxParamsConfig {
    fn count_this(&self) -> CountThis {
        self.count_this.unwrap_or(if self.count_void_this {
            CountThis::Always
        } else {
            CountThis::ExceptVoid
        })
    }
}

impl MaxParams {
    fn should_count_this_param(&self, is_void_this: bool) -> bool {
        match self.count_this() {
            CountThis::Always => true,
            CountThis::Never => false,
            CountThis::ExceptVoid => !is_void_this,
        }
    }

    fn ts_function_type_param_count(&self, function: &TSFunctionType) -> usize {
        let mut real_len = function.params.items.len();
        if let Some(this_params) = &function.this_param {
            let is_void_this = this_params
                .type_annotation
                .as_ref()
                .is_some_and(|t| matches!(t.type_annotation, TSType::TSVoidKeyword(_)));
            if self.should_count_this_param(is_void_this) {
                real_len += 1;
            }
        }
        real_len
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
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        if let Some(max) = value
            .get(0)
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Ok(Self(Box::new(MaxParamsConfig { max, ..MaxParamsConfig::default() })))
        } else {
            serde_json::from_value::<DefaultRuleConfig<Self>>(value)
                .map(DefaultRuleConfig::into_inner)
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
                    if self.should_count_this_param(is_void_this) {
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
            AstKind::TSFunctionType(function) => {
                let real_len = self.ts_function_type_param_count(function);
                if real_len > self.max {
                    let error_msg = format!(
                        "Function has too many parameters ({}). Maximum allowed is {}.",
                        real_len, self.max
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
        ("var test = (a, b, c) => {};", Some(serde_json::json!([3]))), // { "ecmaVersion": 6 },
        ("var test = function test(a, b, c) {};", Some(serde_json::json!([3]))),
        ("var test = function(a, b, c) {};", Some(serde_json::json!([{ "max": 3 }]))),
        ("function foo() {}", None),
        ("const foo = function () {};", None),
        ("const foo = () => {};", None),
        ("function foo(a) {}", None),
        (
            "
              class Foo {
            	constructor(a) {}
              }
            	  ",
            None,
        ),
        (
            "
              class Foo {
            	method(this: void, a, b, c) {}
              }
            	  ",
            None,
        ),
        (
            "
              class Foo {
            	method(this: Foo, a, b) {}
              }
            	  ",
            None,
        ),
        ("function foo(a, b, c, d) {}", Some(serde_json::json!([{ "max": 4 }]))),
        ("function foo(a, b, c, d) {}", Some(serde_json::json!([{ "maximum": 4 }]))),
        (
            "
              class Foo {
            	method(this: void) {}
              }
            		",
            Some(serde_json::json!([{ "max": 0 }])),
        ),
        (
            "
              class Foo {
            	method(this: void, a) {}
              }
            		",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
              class Foo {
            	method(this: void, a) {}
              }
            		",
            Some(serde_json::json!([{ "countVoidThis": true, "max": 2 }])),
        ),
        ("function testD(this: void, a) {}", Some(serde_json::json!([{ "max": 1 }]))),
        (
            "function testD(this: void, a) {}",
            Some(serde_json::json!([{ "countVoidThis": true, "max": 2 }])),
        ),
        ("const testE = function (this: void, a) {}", Some(serde_json::json!([{ "max": 1 }]))),
        (
            "const testE = function (this: void, a) {}",
            Some(serde_json::json!([{ "countVoidThis": true, "max": 2 }])),
        ),
        (
            "
              declare function makeDate(m: number, d: number, y: number): Date;
            		",
            Some(serde_json::json!([{ "max": 3 }])),
        ),
        (
            "
              type sum = (a: number, b: number) => number;
            		",
            Some(serde_json::json!([{ "max": 2 }])),
        ),
        (
            "function foo(this: unknown[], a, b, c) {}",
            Some(serde_json::json!([{ "max": 3, "countThis": "never" }])),
        ),
        (
            "function foo(this: void, a, b, c) {}",
            Some(serde_json::json!([{ "max": 3, "countThis": "except-void" }])),
        ),
    ];

    let fail = vec![
        ("function test(a, b, c) {}", Some(serde_json::json!([2]))),
        ("function test(a, b, c, d) {}", None),
        ("var test = function(a, b, c, d) {};", Some(serde_json::json!([3]))),
        ("var test = (a, b, c, d) => {};", Some(serde_json::json!([3]))), // { "ecmaVersion": 6 },
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
        ("function foo(a, b, c, d) {}", None),
        ("const foo = function (a, b, c, d) {};", None),
        ("const foo = (a, b, c, d) => {};", None),
        ("const foo = a => {};", Some(serde_json::json!([{ "max": 0 }]))),
        (
            "
              class Foo {
            	method(this: void, a, b, c, d) {}
              }
            		",
            None,
        ),
        (
            "
              class Foo {
            	method(this: void, a) {}
              }
            		",
            Some(serde_json::json!([{ "countVoidThis": true, "max": 1 }])),
        ),
        (
            "
              class Foo {
            	method(this: void, a) {}
              }
            		",
            Some(serde_json::json!([{ "countThis": "always", "max": 1 }])),
        ),
        (
            "function testD(this: void, a) {}",
            Some(serde_json::json!([{ "countVoidThis": true, "max": 1 }])),
        ),
        (
            "function testD(this: void, a) {}",
            Some(serde_json::json!([{ "countThis": "always", "max": 1 }])),
        ),
        (
            "const testE = function (this: void, a) {}",
            Some(serde_json::json!([{ "countThis": "always", "max": 1 }])),
        ),
        (
            "function testFunction(test: void, a: number) {}",
            Some(serde_json::json!([{ "countThis": "except-void", "max": 1 }])),
        ),
        (
            "const testE = function (this: void, a) {}",
            Some(serde_json::json!([{ "countVoidThis": true, "max": 1 }])),
        ),
        (
            "function testFunction(test: void, a: number) {}",
            Some(serde_json::json!([{ "countVoidThis": false, "max": 1 }])),
        ),
        (
            "
              class Foo {
            	method(this: Foo, a, b, c) {}
              }
            		",
            None,
        ),
        (
            "
              declare function makeDate(m: number, d: number, y: number): Date;
            		",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "
              type sum = (a: number, b: number) => number;
            		",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
        (
            "function foo(this: unknown[], a, b, c) {}",
            Some(serde_json::json!([{ "max": 3, "countThis": "always" }])),
        ),
        (
            "function foo(this: unknown[], a, b, c) {}",
            Some(serde_json::json!([{ "max": 3, "countThis": "except-void" }])),
        ),
    ];

    Tester::new(MaxParams::NAME, MaxParams::PLUGIN, pass, fail).test_and_snapshot();
}
