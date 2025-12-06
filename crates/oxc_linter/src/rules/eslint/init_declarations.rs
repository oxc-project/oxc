use oxc_ast::{
    AstKind,
    ast::{
        BindingPatternKind, ForInStatement, ForOfStatement, ForStatement, ForStatementInit,
        ForStatementLeft, VariableDeclarationKind,
    },
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

fn init_declarations_diagnostic(span: Span, mode: &Mode, identifier_name: &str) -> OxcDiagnostic {
    let msg = if Mode::Always == *mode {
        format!("Variable '{identifier_name}' should be initialized on declaration.")
    } else {
        format!("Variable '{identifier_name}' should not be initialized on declaration.")
    };
    OxcDiagnostic::warn(msg)
        .with_help("Require or disallow initialization in variable declarations")
        .with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    /// Requires that variables be initialized on declaration. This is the default behavior.
    #[default]
    Always,
    /// Disallows initialization during declaration.
    Never,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InitDeclarations(Mode, InitDeclarationsConfig);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InitDeclarationsConfig {
    /// When set to `true`, allows uninitialized variables in the init expression of `for`, `for-in`, and `for-of` loops.
    /// Only applies when mode is set to `"never"`.
    ignore_for_loop_init: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require or disallow initialization in variable declarations
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, variables can be assigned during declaration, or at any point afterwards using an assignment statement.
    /// For example, in the following code, foo is initialized during declaration, while bar is initialized later.
    ///
    /// ### Examples
    ///
    /// ```js
    /// var foo = 1;
    /// var bar;
    /// if (foo) {
    ///     bar = 1;
    /// } else {
    ///     bar = 2;
    /// }
    /// ```
    ///
    /// Examples of incorrect code for the default "always" option:
    /// ```js
    /// /* init-declarations: ["error", "always"] */
    /// function foo() {
    ///     var bar;
    ///     let baz;
    /// }
    /// ```
    ///
    /// Examples of incorrect code for the "never" option:
    /// ```js
    /// /* init-declarations: ["error", "never"] */
    /// function foo() {
    ///     var bar = 1;
    ///     let baz = 2;
    ///     for (var i = 0; i < 1; i++) {}
    /// }
    /// ```
    ///
    /// Examples of correct code for the default "always" option:
    /// ```js
    /// /* init-declarations: ["error", "always"] */
    ///
    /// function foo() {
    ///     var bar = 1;
    ///     let baz = 2;
    ///     const qux = 3;
    /// }
    /// ```
    ///
    /// Examples of correct code for the "never" option:
    /// ```js
    /// /* init-declarations: ["error", "never"] */
    ///
    /// function foo() {
    ///     var bar;
    ///     let baz;
    ///     const buzz = 1;
    /// }
    /// ```
    ///
    /// Examples of correct code for the "never", { "ignoreForLoopInit": true } options:
    /// ```js
    /// /* init-declarations: ["error", "never", { "ignoreForLoopInit": true }] */
    /// for (var i = 0; i < 1; i++) {}
    /// ```
    InitDeclarations,
    eslint,
    style,
    config = InitDeclarations,
);

impl Rule for InitDeclarations {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<InitDeclarations>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(decl) = node.kind() {
            let InitDeclarations(mode, config) = &self;
            let parent = ctx.nodes().parent_node(node.id());

            // support for TypeScript's declare variables
            if mode == &Mode::Always {
                if decl.declare {
                    return;
                }
                let declare = ctx.nodes().ancestor_kinds(node.id()).any(|el| match el {
                    AstKind::TSModuleDeclaration(ts_module_decl) => ts_module_decl.declare,
                    AstKind::TSGlobalDeclaration(ts_global_decl) => ts_global_decl.declare,
                    _ => false,
                });
                if declare {
                    return;
                }
            }
            for v in &decl.declarations {
                let BindingPatternKind::BindingIdentifier(identifier) = &v.id.kind else {
                    continue;
                };
                let is_initialized = match parent.kind() {
                    AstKind::ForInStatement(ForInStatement { left, .. })
                    | AstKind::ForOfStatement(ForOfStatement { left, .. }) => {
                        matches!(left, ForStatementLeft::VariableDeclaration(left_node) if left_node.span == decl.span)
                    }
                    // When eslint processes ForStatementInit statements
                    // the default variable is initialized
                    // eg: "for (var a; a < 2; a++)" a is initialized
                    AstKind::ForStatement(ForStatement { init, .. }) => {
                        matches!(init, Some(ForStatementInit::VariableDeclaration(init)) if init.span == decl.span)
                    }
                    _ => v.init.is_some(),
                };

                match mode {
                    Mode::Always if !is_initialized => {
                        ctx.diagnostic(init_declarations_diagnostic(
                            v.span,
                            mode,
                            identifier.name.as_str(),
                        ));
                    }
                    Mode::Never if is_initialized && !config.ignore_for_loop_init => {
                        if matches!(&v.kind, VariableDeclarationKind::Const) {
                            continue;
                        }
                        ctx.diagnostic(init_declarations_diagnostic(
                            v.span,
                            mode,
                            identifier.name.as_str(),
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = null;", None),
        ("foo = true;", None),
        ("var foo = 1, bar = false, baz = {};", None),
        ("function foo() { var foo = 0; var bar = []; }", None),
        ("var fn = function() {};", None),
        ("var foo = bar = 2;", None),
        ("for (var i = 0; i < 1; i++) {}", None),
        ("for (var foo in []) {}", None),
        ("for (var foo of []) {}", None), // { "ecmaVersion": 6 },
        ("let a = true;", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("const a = {};", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        (
            "function foo() { let a = 1, b = false; if (a) { let c = 3, d = null; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { const a = 1, b = true; if (a) { const c = 3, d = null; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a = 1; const b = false; var c = true; }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 6 },
        ("var foo;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("var foo, bar, baz;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("function foo() { var foo; var bar; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("let a;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("const a = 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("function foo() { let a, b; if (a) { let c, d; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        (
            "function foo() { const a = 1, b = true; if (a) { const c = 3, d = null; } }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { let a; const b = false; var c; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        (
            "for(var i = 0; i < 1; i++){}",
            Some(serde_json::json!(["never", { "ignoreForLoopInit": true }])),
        ),
        (
            "for (var foo in []) {}",
            Some(serde_json::json!(["never", { "ignoreForLoopInit": true }])),
        ),
        (
            "for (var foo of []) {}",
            Some(serde_json::json!(["never", { "ignoreForLoopInit": true }])),
        ), // { "ecmaVersion": 6 }
        (
            "for (var a, b = 2; a < 100; a++) {}",
            Some(serde_json::json!(["never", { "ignoreForLoopInit": true }])),
        ),
        // typescript-eslint
        ("declare const foo: number;", Some(serde_json::json!(["always"]))),
        ("declare const foo: number;", Some(serde_json::json!(["never"]))),
        (
            "declare namespace myLib {
                let numberOfGreetings: number;
            }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "declare namespace myLib {
                let numberOfGreetings: number;
            }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "interface GreetingSettings {
                greeting: string;
                duration?: number;
                color?: string;
            }",
            Some(serde_json::json!(["never"])),
        ),
        ("type GreetingLike = string | (() => string) | Greeter;", None),
        (
            "type GreetingLike = string | (() => string) | Greeter;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() {
                var bar: string;
            }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "const class1 = class NAME {
                constructor() {
                    var name1: string = 'hello';
                }
            };",
            None,
        ),
        (
            "const class1 = class NAME {
                static pi: number = 3.14;
            };",
            Some(serde_json::json!(["never"])),
        ),
        (
            "namespace myLib {
                let numberOfGreetings: number;
            }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "namespace myLib {
                let numberOfGreetings: number = 2;
            }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "declare namespace myLib1 {
                const foo: number;
                namespace myLib2 {
                    let bar: string;
                    namespace myLib3 {
                        let baz: object;
                    }
                }
            }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "declare namespace myLib1 {
                const foo: number;
                namespace myLib2 {
                    let bar: string;
                    namespace myLib3 {
                        let baz: object;
                    }
                }
            }",
            Some(serde_json::json!(["never"])),
        ),
    ];

    let fail = vec![
        ("var foo;", Some(serde_json::json!(["always"]))),
        ("for (var a in []) var foo;", Some(serde_json::json!(["always"]))),
        ("var foo, bar = false, baz;", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("function foo() { var foo = 0; var bar; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("function foo() { var foo; var bar = foo; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("let a;", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        (
            "function foo() { let a = 1, b; if (a) { let c = 3, d = null; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { let a; const b = false; var c; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("var foo = bar = 2;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("var foo = true;", Some(serde_json::json!(["never"]))),    // { "ecmaVersion": 6 },
        ("var foo, bar = 5, baz = 3;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("function foo() { var foo; var bar = foo; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("let a = 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        (
            "function foo() { let a = 'foo', b; if (a) { let c, d; } }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { let a; const b = false; var c = 1; }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 6 },
        ("for(var i = 0; i < 1; i++){}", Some(serde_json::json!(["never"]))),
        ("for (var foo in []) {}", Some(serde_json::json!(["never"]))),
        ("for (var foo of []) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 }
        ("for (var a, b = 2; a < 100; a++) {}", Some(serde_json::json!(["never"]))),
        // typescript-eslint
        ("let arr: string[] = ['arr', 'ar'];", Some(serde_json::json!(["never"]))),
        (
            "const class1 = class NAME {
                constructor() {
                    var name1: string = 'hello';
                }
            };",
            Some(serde_json::json!(["never"])),
        ),
        ("let arr: string;", None),
        (
            "namespace myLib {
                let numberOfGreetings: number;
            }",
            None,
        ),
        (
            "namespace myLib {
                let numberOfGreetings: number = 2;
            }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "namespace myLib1 {
                let foo: number;
                namespace myLib2 {
                    let bar: string;
                    namespace myLib3 {
                        let baz: object;
                    }
                }
            }",
            Some(serde_json::json!(["always"])),
        ),
    ];

    Tester::new(InitDeclarations::NAME, InitDeclarations::PLUGIN, pass, fail).test_and_snapshot();
}
