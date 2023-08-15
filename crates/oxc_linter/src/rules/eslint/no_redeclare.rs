use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, globals::BUILTINS, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-redeclare): '{0}' is already defined.")]
#[diagnostic(severity(warning))]
struct NoRedeclareDiagnostic(Atom, #[label("'{0}' is already defined.")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-redeclare): '{0}' is already defined as a built-in global variable.")]
#[diagnostic(severity(warning))]
struct NoRedeclareAsBuiltiInDiagnostic(
    Atom,
    #[label("'{0}' is already defined as a built-in global variable.")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-redeclare): '{0}' is already defined by a variable declaration.")]
#[diagnostic(severity(warning))]
struct NoRedeclareBySyntaxDiagnostic(
    Atom,
    #[label("'{0}' is already defined by a variable declaration.")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct NoRedeclare {
    built_in_globals: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow variable redeclaration
    ///
    /// ### Why is this bad?
    ///
    /// n JavaScript, it’s possible to redeclare the same variable name using var. This can lead to confusion as to where the variable is actually declared and initialized.
    ///
    /// ### Example
    /// ```javascript
    /// var a = 3;
    /// var a = 10;
    /// ```
    NoRedeclare,
    correctness
);

impl Rule for NoRedeclare {
    fn from_configuration(value: serde_json::Value) -> Self {
        let built_in_globals = value
            .get(0)
            .and_then(|config| config.get("builtinGlobals"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { built_in_globals }
    }

    fn run_once(&self, ctx: &LintContext) {
        let redeclare_variables = ctx.semantic().redeclare_variables();
        let total = redeclare_variables.len();

        for current in 0..=total {
            if let Some(current_variable) = redeclare_variables.get(current) {
                for idx in (current + 1)..=total {
                    if let Some(next_variable) = redeclare_variables.get(idx) {
                        if next_variable.name == current_variable.name
                            && next_variable.scope_id == current_variable.scope_id
                        {
                            if self.built_in_globals {
                                if let Some(&value) = BUILTINS.get(&current_variable.name) {
                                    if value {
                                        ctx.diagnostic(NoRedeclareAsBuiltiInDiagnostic(
                                            current_variable.name.clone(),
                                            current_variable.span,
                                        ));
                                    } else {
                                        ctx.diagnostic(NoRedeclareDiagnostic(
                                            current_variable.name.clone(),
                                            current_variable.span,
                                        ));
                                    }
                                } else {
                                    ctx.diagnostic(NoRedeclareDiagnostic(
                                        current_variable.name.clone(),
                                        current_variable.span,
                                    ));
                                }
                            } else {
                                ctx.diagnostic(NoRedeclareDiagnostic(
                                    current_variable.name.clone(),
                                    current_variable.span,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = 3; var b = function() { var a = 10; };", None),
        ("var a = 3; a = 10;", None),
        // ("if (true) {\n    let b = 2;\n} else {    \nlet b = 3;\n}", None),
        ("var a; class C { static { var a; } }", None),
        ("class C { static { var a; } } var a; ", None),
        ("function a(){} class C { static { var a; } }", None),
        ("var a; class C { static { function a(){} } }", None),
        ("class C { static { var a; } static { var a; } }", None),
        ("class C { static { function a(){} } static { function a(){} } }", None),
        ("class C { static { var a; { function a(){} } } }", None),
        ("class C { static { function a(){}; { function a(){} } } }", None),
        // ("class C { static { var a; { let a; } } }", None),
        // ("class C { static { let a; { let a; } } }", None),
        // ("class C { static { { let a; } { let a; } } }", None),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var self = 1", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var globalThis = foo", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var globalThis = foo", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // Comments and built-ins.
        ("/*globals Array */", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("/*globals a */", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("/*globals a */", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("/*globals a:off */", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("/*globals a */", Some(serde_json::json!([{ "builtinGlobals": false }]))),
    ];

    let fail = vec![
        ("var a = 3; var a = 10;", None),
        // ("switch(foo) { case a: var b = 3;\ncase b: var b = 4}", None),
        ("var a = 3; var a = 10;", None),
        ("var a = {}; var a = [];", None),
        // ("var a; function a() {}", None),
        // ("function a() {} function a() {}", None),
        ("var a = function() { }; var a = function() { }", None),
        ("var a = function() { }; var a = new Date();", None),
        ("var a = 3; var a = 10; var a = 15;", None),
        ("var a; var a;", None),
        // ("export vars a; var a;", None),

        // `var` redeclaration in class static blocks. Redeclaration of functions is not allowed in class static blocks.
        ("class C { static { var a; var a; } }", None),
        ("class C { static { var a; { var a; } } }", None),
        ("class C { static { { var a; } var a; } }", None),
        ("class C { static { { var a; } { var a; } } }", None),
        // ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var a; var {a = 0, b: Object = 0} = {};", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var a; var {a = 0, b: Object = 0} = {};", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var a; var {a = 0, b: Object = 0} = {};", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var a; var {a = 0, b: Object = 0} = {};", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var globalThis = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("var a; var {a = 0, b: globalThis = 0} = {};", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("/*global b:false*/ var b = 1;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // ("/*global b:true*/ var b = 1;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("function f() { var a; var a; }", None),
        // ("function f(a) { var a; }", None),
        ("function f() { var a; if (test) { var a; } }", None),
        ("for (var a, a;;);", None),
    ];

    Tester::new(NoRedeclare::NAME, pass, fail).test_and_snapshot();
}
