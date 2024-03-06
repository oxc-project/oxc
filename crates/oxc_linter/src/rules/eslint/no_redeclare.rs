use oxc_ast::{
    ast::{BindingIdentifier, BindingPatternKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-redeclare): '{0}' is already defined.")]
#[diagnostic(severity(warning))]
struct NoRedeclareDiagnostic(
    CompactStr,
    #[label("'{0}' is already defined.")] pub Span,
    #[label("It can not be redeclare here.")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-redeclare): '{0}' is already defined as a built-in global variable.")]
#[diagnostic(severity(warning))]
struct NoRedeclareAsBuiltiInDiagnostic(
    CompactStr,
    #[label("'{0}' is already defined as a built-in global variable.")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-redeclare): '{0}' is already defined by a variable declaration.")]
#[diagnostic(severity(warning))]
struct NoRedeclareBySyntaxDiagnostic(
    CompactStr,
    #[label("'{0}' is already defined by a variable declaration.")] pub Span,
    #[label("It cannot be redeclared here.")] pub Span,
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
    pedantic
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
        let symbol_table = ctx.semantic().symbols();

        for symbol_id in ctx.symbols().iter() {
            let decl = symbol_table.get_declaration(symbol_id);
            let symbol_name = symbol_table.get_name(symbol_id);
            match ctx.nodes().kind(decl) {
                AstKind::VariableDeclarator(var) => {
                    if let BindingPatternKind::BindingIdentifier(ident) = &var.id.kind {
                        if symbol_name == ident.name.as_str() {
                            for span in ctx.symbols().get_redeclare_variables(symbol_id) {
                                self.report_diagnostic(ctx, *span, ident);
                            }
                        }
                    }
                }
                AstKind::FormalParameter(param) => {
                    if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                        if symbol_name == ident.name.as_str() {
                            for span in ctx.symbols().get_redeclare_variables(symbol_id) {
                                self.report_diagnostic(ctx, *span, ident);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl NoRedeclare {
    fn report_diagnostic(&self, ctx: &LintContext, span: Span, ident: &BindingIdentifier) {
        if self.built_in_globals && ctx.env_contains_var(&ident.name) {
            ctx.diagnostic(NoRedeclareAsBuiltiInDiagnostic(
                ident.name.to_compact_str(),
                ident.span,
            ));
        } else {
            ctx.diagnostic(NoRedeclareDiagnostic(ident.name.to_compact_str(), ident.span, span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = 3; var b = function() { var a = 10; };", None),
        ("var a = 3; a = 10;", None),
        ("if (true) {\n    let b = 2;\n} else {    \nlet b = 3;\n}", None),
        ("var a; class C { static { var a; } }", None),
        ("class C { static { var a; } } var a; ", None),
        ("function a(){} class C { static { var a; } }", None),
        ("var a; class C { static { function a(){} } }", None),
        ("class C { static { var a; } static { var a; } }", None),
        ("class C { static { function a(){} } static { function a(){} } }", None),
        ("class C { static { var a; { function a(){} } } }", None),
        ("class C { static { function a(){}; { function a(){} } } }", None),
        ("class C { static { var a; { let a; } } }", None),
        ("class C { static { let a; { let a; } } }", None),
        ("class C { static { { let a; } { let a; } } }", None),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var self = 1", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var globalThis = foo", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var globalThis = foo", Some(serde_json::json!([{ "builtinGlobals": false }]))),
    ];

    let fail = vec![
        ("switch(foo) { case a: var b = 3;\ncase b: var b = 4}", None),
        ("var a = 3; var a = 10;", None),
        ("var a = {}; var a = [];", None),
        ("var a; function a() {}", None),
        ("function a() {} function a() {}", None),
        ("var a = function() { }; var a = function() { }", None),
        ("var a = function() { }; var a = new Date();", None),
        ("var a = 3; var a = 10; var a = 15;", None),
        ("var a; var a;", None),
        ("export var a; var a;", None),
        ("class C { static { var a; var a; } }", None),
        ("class C { static { var a; { var a; } } }", None),
        ("class C { static { { var a; } var a; } }", None),
        ("class C { static { { var a; } { var a; } } }", None),
        // ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": true }]))),
        (
            "var a; var {a = 0, b: Object = 0} = {};",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
        ),
        // ("var globalThis = 0;", Some(serde_json::json!([{ "builtinGlobals": true }]))),
        (
            "var a; var {a = 0, b: globalThis = 0} = {};",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
        ),
        ("function f() { var a; var a; }", None),
        ("function f(a) { var a; }", None),
        ("function f() { var a; if (test) { var a; } }", None),
        ("for (var a, a;;);", None),
    ];

    Tester::new(NoRedeclare::NAME, pass, fail).test_and_snapshot();
}
