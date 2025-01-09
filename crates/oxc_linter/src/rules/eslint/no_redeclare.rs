use oxc_ast::{
    ast::{BindingIdentifier, BindingPatternKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

use crate::{context::LintContext, rule::Rule};

fn no_redeclare_diagnostic(id_name: &str, decl_span: Span, re_decl_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{id_name}' is already defined.")).with_labels([
        decl_span.label(format!("'{id_name}' is already defined.")),
        re_decl_span.label("It can not be redeclare here."),
    ])
}

fn no_redeclare_as_builtin_in_diagnostic(builtin_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{builtin_name}' is already defined as a built-in global variable."
    ))
    .with_label(
        span.label(format!("'{builtin_name}' is already defined as a built-in global variable.")),
    )
}

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
    eslint,
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

    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext) {
        let symbol_table = ctx.semantic().symbols();
        let decl_node_id = symbol_table.get_declaration(symbol_id);
        match ctx.nodes().kind(decl_node_id) {
            AstKind::VariableDeclarator(var) => {
                if let BindingPatternKind::BindingIdentifier(ident) = &var.id.kind {
                    let symbol_name = symbol_table.get_name(symbol_id);
                    if symbol_name == ident.name.as_str() {
                        for span in ctx.symbols().get_redeclarations(symbol_id) {
                            self.report_diagnostic(ctx, *span, ident);
                        }
                    }
                }
            }
            AstKind::FormalParameter(param) => {
                if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                    let symbol_name = symbol_table.get_name(symbol_id);
                    if symbol_name == ident.name.as_str() {
                        for span in ctx.symbols().get_redeclarations(symbol_id) {
                            self.report_diagnostic(ctx, *span, ident);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl NoRedeclare {
    fn report_diagnostic(&self, ctx: &LintContext, span: Span, ident: &BindingIdentifier) {
        if self.built_in_globals && ctx.env_contains_var(&ident.name) {
            ctx.diagnostic(no_redeclare_as_builtin_in_diagnostic(ident.name.as_str(), ident.span));
        } else {
            ctx.diagnostic(no_redeclare_diagnostic(ident.name.as_str(), ident.span, span));
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

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, pass, fail).test_and_snapshot();
}
