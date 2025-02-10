use oxc_ast::{
    ast::{
        BindingPatternKind, ForInStatement, ForOfStatement, ForStatementLeft,
        VariableDeclarationKind,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

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

#[derive(Debug, Default, PartialEq, Clone)]
enum Mode {
    #[default]
    Always,
    Never,
}

impl Mode {
    pub fn from(raw: &str) -> Self {
        if raw == "never" {
            Self::Never
        } else {
            Self::Always
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InitDeclarations {
    mode: Mode,
    ignore_for_loop_init: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Require or disallow initialization in variable declarations
    ///
    /// ### Why is this bad?
    /// In JavaScript, variables can be assigned during declaration, or at any point afterwards using an assignment statement.
    /// For example, in the following code, foo is initialized during declaration, while bar is initialized later.
    ///
    /// ### Examples
    /// var foo = 1;
    /// var bar;
    /// if (foo) {
    ///     bar = 1;
    /// } else {
    ///     bar = 2;
    /// }
    ///
    /// Examples of incorrect code for the default "always" option:
    /// ```js
    /// /*eslint init-declarations: ["error", "always"]*/
    /// function foo() {
    ///     var bar;
    ///     let baz;
    /// }
    /// ```
    ///
    /// Examples of incorrect code for the "never" option:
    /// ```js
    /// /*eslint init-declarations: ["error", "never"]*/
    /// function foo() {
    ///     var bar = 1;
    ///     let baz = 2;
    ///     for (var i = 0; i < 1; i++) {}
    /// }
    /// ```
    ///
    /// Examples of correct code for the default "always" option:
    /// ```js
    /// /*eslint init-declarations: ["error", "always"]*/
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
    /// /*eslint init-declarations: ["error", "never"]*/
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
    /// /*eslint init-declarations: ["error", "never", { "ignoreForLoopInit": true }]*/
    /// for (var i = 0; i < 1; i++) {}
    /// ```
    InitDeclarations,
    eslint,
    style
);

impl Rule for InitDeclarations {
    fn from_configuration(value: Value) -> Self {
        let obj1 = value.get(0);
        let obj2 = value.get(1);

        Self {
            mode: obj1.and_then(Value::as_str).map(Mode::from).unwrap_or_default(),
            ignore_for_loop_init: obj2
                .and_then(|v| v.get("ignoreForLoopInit"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::VariableDeclaration(decl) = node.kind() {
            let Some(parent) = ctx.nodes().parent_node(node.id()) else {
                return;
            };
            // support for TypeScript's declare variables
            if self.mode == Mode::Always {
                if decl.declare {
                    return;
                }
                let decl_ancestor =
                    ctx.nodes().ancestor_kinds(node.id()).skip(1).find(|el| {
                        matches!(el, AstKind::TSModuleDeclaration(ts_module_decl) if ts_module_decl.declare)
                    });
                if decl_ancestor.is_some() {
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
                    AstKind::ForStatementInit(_) => true,
                    _ => v.init.is_some(),
                };

                match self.mode {
                    Mode::Always if !is_initialized => {
                        ctx.diagnostic(init_declarations_diagnostic(
                            v.span,
                            &self.mode,
                            identifier.name.as_str(),
                        ));
                    }
                    Mode::Never if is_initialized && !self.ignore_for_loop_init => {
                        if let VariableDeclarationKind::Const = &v.kind {
                            continue;
                        }
                        ctx.diagnostic(init_declarations_diagnostic(
                            v.span,
                            &self.mode,
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
