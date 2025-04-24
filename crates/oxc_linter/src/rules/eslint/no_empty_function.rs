use oxc_ast::{
    AstKind,
    ast::{FunctionType, MethodDefinitionKind, PropertyKind, TSAccessibility},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use serde_json::Value;
use std::borrow::Cow;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_empty_function_diagnostic(span: Span, fn_kind: &str, fn_name: Option<&str>) -> OxcDiagnostic {
    let message = match fn_name {
        Some(name) => Cow::Owned(format!("Unexpected empty {fn_kind} `{name}`.")),
        None => Cow::Owned(format!("Unexpected empty {fn_kind}.")),
    };

    OxcDiagnostic::warn(message)
        .with_help(format!("Consider removing this {fn_kind} or adding logic to it."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyFunction {
    allow: Vec<CompactStr>,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallows the usages of empty functions
    ///
    /// ### Why is this bad?
    /// Empty functions can reduce readability because readers need to guess whether it’s
    /// intentional or not. So writing a clear comment for empty functions is a good practice.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() {
    /// }
    ///
    /// const bar = () => {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo() {
    ///     // do nothing
    /// }
    ///
    /// function foo() {
    ///     return;
    /// }
    /// const add = (a, b) => a + b
    /// ```
    NoEmptyFunction,
    eslint,
    restriction,
);

impl Rule for NoEmptyFunction {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);

        Self {
            allow: obj
                .and_then(|obj| obj.get("allow"))
                .and_then(Value::as_array)
                .map(|v| v.iter().filter_map(Value::as_str).map(CompactStr::from).collect())
                .unwrap_or_default(),
        }
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FunctionBody(fb) = node.kind() else {
            return;
        };
        if !fb.is_empty() || ctx.has_comments_between(fb.span) {
            return;
        }
        let Some(function) = ctx.nodes().parent_node(node.id()) else {
            return;
        };

        match function.kind() {
            AstKind::Function(f) => {
                match f.r#type {
                    FunctionType::FunctionDeclaration => {
                        let Some(f_name) = f.name() else {
                            return;
                        };
                        let (check_kind, fn_kind) = {
                            if f.r#async {
                                ("asyncFunctions", "async function")
                            } else if f.generator {
                                ("generatorFunctions", "generator function")
                            } else {
                                ("functions", "function")
                            }
                        };
                        if !allowed_func(&self.allow, check_kind) {
                            ctx.diagnostic(no_empty_function_diagnostic(
                                fb.span,
                                fn_kind,
                                Some(f_name.into()),
                            ));
                        }
                    }
                    FunctionType::FunctionExpression => {
                        let Some(func_expr_parent) = ctx.nodes().parent_kind(function.id()) else {
                            return;
                        };
                        match func_expr_parent {
                            AstKind::ObjectProperty(prop) => {
                                let key_name = prop.key.name();
                                let key_name_str = key_name.as_deref();
                                match prop.kind {
                                    PropertyKind::Init => {
                                        if prop.method {
                                            let (check_kind, fn_kind) = if f.r#async {
                                                // e.g. "const a = { async foo() { } }"
                                                ("asyncMethods", "async method")
                                            } else if f.generator {
                                                ("generatorMethods", "generator method")
                                            } else {
                                                // e.g. "const a = { foo() { } }"
                                                ("methods", "method")
                                            };
                                            if !allowed_func(&self.allow, check_kind) {
                                                ctx.diagnostic(no_empty_function_diagnostic(
                                                    fb.span,
                                                    fn_kind,
                                                    key_name_str,
                                                ));
                                            }
                                        } else {
                                            let check_kind = if f.r#async {
                                                "asyncFunctions"
                                            } else if f.generator {
                                                "generatorFunctions"
                                            } else {
                                                // e.g. "const a = { foo: function() { }  }"
                                                "functions"
                                            };
                                            if !allowed_func(&self.allow, check_kind) {
                                                ctx.diagnostic(no_empty_function_diagnostic(
                                                    fb.span,
                                                    "method",
                                                    key_name_str,
                                                ));
                                            }
                                        }
                                    }
                                    PropertyKind::Get => {
                                        // e.g. "const a = { get foo() { } }"
                                        if !allowed_func(&self.allow, "getters") {
                                            ctx.diagnostic(no_empty_function_diagnostic(
                                                fb.span,
                                                "getter",
                                                key_name_str,
                                            ));
                                        }
                                    }
                                    PropertyKind::Set => {
                                        if !allowed_func(&self.allow, "setters") {
                                            ctx.diagnostic(no_empty_function_diagnostic(
                                                fb.span,
                                                "setter",
                                                key_name_str,
                                            ));
                                        }
                                    }
                                }
                            }
                            AstKind::MethodDefinition(method) => {
                                let key_name = method.key.name();
                                let key_name_str = key_name.as_deref();
                                match method.kind {
                                    MethodDefinitionKind::Method => {
                                        let kind = if method.r#static {
                                            "static method"
                                        } else {
                                            "method"
                                        };
                                        let check_kind = if method.r#override {
                                            "overrideMethods"
                                        } else if !method.decorators.is_empty() {
                                            "decoratedFunctions"
                                        } else if f.r#async {
                                            "asyncMethods"
                                        } else if f.generator {
                                            "generatorMethods"
                                        } else {
                                            "methods"
                                        };

                                        if !allowed_func(&self.allow, check_kind) {
                                            ctx.diagnostic(no_empty_function_diagnostic(
                                                fb.span,
                                                kind,
                                                key_name_str,
                                            ));
                                        }
                                    }
                                    MethodDefinitionKind::Constructor => {
                                        let check_kind = match method.accessibility {
                                            Some(TSAccessibility::Private) => "privateConstructors",
                                            Some(TSAccessibility::Protected) => {
                                                "protectedConstructors"
                                            }
                                            _ => "constructors",
                                        };
                                        if !allowed_func(&self.allow, check_kind) {
                                            ctx.diagnostic(no_empty_function_diagnostic(
                                                fb.span,
                                                "constructor",
                                                None,
                                            ));
                                        }
                                    }
                                    MethodDefinitionKind::Get => {
                                        if !allowed_func(&self.allow, "getters") {
                                            let kind = if method.r#static {
                                                "static getter"
                                            } else {
                                                "getter"
                                            };
                                            ctx.diagnostic(no_empty_function_diagnostic(
                                                fb.span,
                                                kind,
                                                key_name_str,
                                            ));
                                        }
                                    }
                                    MethodDefinitionKind::Set => {
                                        if !allowed_func(&self.allow, "setters") {
                                            let kind = if method.r#static {
                                                "static setter"
                                            } else {
                                                "setter"
                                            };
                                            ctx.diagnostic(no_empty_function_diagnostic(
                                                fb.span,
                                                kind,
                                                key_name_str,
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            AstKind::ArrowFunctionExpression(arrow_func)
                if !allowed_func(&self.allow, "arrowFunctions") =>
            {
                let kind =
                    if arrow_func.r#async { "async arrow function" } else { "arrow function" };
                ctx.diagnostic(no_empty_function_diagnostic(fb.span, kind, None));
            }
            _ => {}
        }
    }
}

fn allowed_func(allow: &[CompactStr], operator: &str) -> bool {
    allow.iter().any(|s| s == operator)
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("const foo = () => {  };", Some(json!([{ "allow": ["arrowFunctions"] }]))),
        (
            r"
                function foo() {}
                const bar = function() {};
                const obj = {
                    foo: function() {}
                };
            ",
            Some(json!([{ "allow": ["functions"] }])),
        ),
        (
            r"
                function* foo() {}
                const bar = function*() {};
                const obj = {
                    foo: function*() {}
                };
            ",
            Some(json!([{ "allow": ["generatorFunctions"] }])),
        ),
        (
            r"
                const obj = {
                    foo() {}
                };
                class A {
                    foo() {}
                    static foo() {}
                }
            ",
            Some(json!([{ "allow": ["methods"] }])),
        ),
        (
            r"
                const obj = {
                    *foo() {}
                };
                class A {
                    *foo() {}
                    static *foo() {}
                }
            ",
            Some(json!([{ "allow": ["generatorMethods"] }])),
        ),
        (
            r"
                const obj = {
                    get foo() {}
                };
                class A {
                    get foo() {}
                    static get foo() {}
                }
            ",
            Some(json!([{ "allow": ["getters"] }])),
        ),
        (
            r"
                const obj = {
                    set foo(value) {}
                };
                class A {
                    set foo(value) {}
                    static set foo(value) {}
                }
            ",
            Some(json!([{ "allow": ["setters"] }])),
        ),
        (
            r"
                class A {
                    constructor() {}
                }
            ",
            Some(json!([{ "allow": ["constructors"] }])),
        ),
        ("async function a() {  }", Some(json!([{ "allow": ["asyncFunctions"] }]))),
        (
            r"
                const obj = {
                    async foo() {}
                };
                class A {
                    async foo() {}
                    static async foo() {}
                }
            ",
            Some(json!([{ "allow": ["asyncMethods"] }])),
        ),
        (
            "
                function foo() {
                    // empty
                }
            ",
            None,
        ),
        (
            "
        function* baz() {
            // empty
        }
        ",
            None,
        ),
        (
            "
        const bar = () => {
            // empty
        };
        ",
            None,
        ),
        (
            "
        const obj = {
            foo: function() {
                // empty
            },
            bar: function*() {
                // empty
            },
            foobar() {
                // empty
            }
        };
        ",
            None,
        ),
        (
            "
        class A {
            constructor() {
                // empty
            }
            foo() {
                // empty
            }
            *foo1() {
                // empty
            }
            get bar() {
                // empty
            }
            set bar(value) {
                // empty
            }
            static bar() {
                // empty
            }
            static *barr() {
                // empty
            }
            static get baz() {
                // empty
            }
            static set baz(value) {
                // empty
            }
        }
        ",
            None,
        ),
    ];

    let fail = vec![
        ("function foo() {}", None),
        ("const bar = () => {};", None),
        ("function* baz() {}", None),
        (
            "
        const obj = {
            foo: function() {
            },
            bar: function*() {
            },
            foobar() {
            }
        };
        ",
            None,
        ),
        (
            "
        class A {
            constructor() {
            }
            foo() {
            }
            *foo1() {
            }
            get fooz() {
            }
            set fooz(value) {
            }
            static bar() {
            }
            static *barr() {
            }
            static get baz() {
            }
            static set baz(value) {
            }
        }
        ",
            None,
        ),
    ];

    // let pass = vec![
    //     (
    //         r"
    //             const obj = {
    //                 *foo() {}
    //             };
    //             class A {
    //                 *foo() {}
    //                 static *foo() {}
    //             }
    //         ",
    //         Some(json!([{ "allow": ["generatorMethods"] }]))
    //     ),
    // ];
    // let fail = vec![
    //     // ("const bar = () => {};", None),
    // ];

    Tester::new(NoEmptyFunction::NAME, NoEmptyFunction::PLUGIN, pass, fail).test_and_snapshot();
}
