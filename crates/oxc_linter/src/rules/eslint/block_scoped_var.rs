use oxc_ast::{AstKind, ast::VariableDeclarationKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn redeclaration_diagnostic(decl_span: Span, redecl_span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is used outside of binding context."))
        .with_help(format!("Variable '{name}' is used outside its declaration block. Declare it outside the block or use 'let'/'const'."))
        .with_labels([
            redecl_span.label("it is redeclared here"),
            decl_span.label(format!("'{name}' is first declared here")),
        ])
}
fn use_outside_scope_diagnostic(decl_span: Span, used_span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is used outside of binding context."))
        .with_help(format!("Variable '{name}' is used outside its declaration block. Declare it outside the block or use 'let'/'const'."))
        .with_labels([
            used_span.label(format!("'{name}' is used here")),
            decl_span.label("It is declared in a different scope here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct BlockScopedVar;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Generates warnings when variables are used outside of the block in which they were defined.
    /// This emulates C-style block scope.
    ///
    /// ### Why is this bad?
    ///
    /// This rule aims to reduce the usage of variables outside of their binding context
    /// and emulate traditional block scope from other languages.
    /// This is to help newcomers to the language avoid difficult bugs with variable hoisting.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function doIf() {
    ///     if (true) {
    ///         var build = true;
    ///     }
    ///     console.log(build);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function doIf() {
    ///     var build;
    ///     if (true) {
    ///         build = true;
    ///    }
    ///     console.log(build);
    /// }
    /// ```
    BlockScopedVar,
    eslint,
    suspicious,
);

impl Rule for BlockScopedVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(decl) = node.kind() else {
            return;
        };
        if decl.kind != VariableDeclarationKind::Var {
            return;
        }
        let cur_node_scope_id = node.scope_id();
        if !ctx.scoping().scope_flags(cur_node_scope_id).is_strict_mode() {
            return;
        }
        // `scope_arr` contains all the scopes that are children of the current scope
        // we should eliminate all of them
        let scope_arr = ctx.scoping().iter_all_scope_child_ids(node.scope_id()).collect::<Vec<_>>();

        let declarations = &decl.declarations;
        for item in declarations {
            let id = &item.id;
            // e.g. "var [a, b] = [1, 2]"
            for ident in id.get_binding_identifiers() {
                let name = ident.name.as_str();
                let Some(symbol_id) = ctx.scoping().find_binding(node.scope_id(), name) else {
                    continue;
                };
                // e.g. "if (true} { var a = 4; } else { var a = 4; }"
                // in this case we can't find the reference of 'a' by call `get_resolved_references`
                // so i use `symbol_redeclarations` to find all the redeclarations
                for redeclaration in ctx.scoping().symbol_redeclarations(symbol_id) {
                    let re_scope_id = ctx.nodes().get_node(redeclaration.declaration).scope_id();
                    if !scope_arr.contains(&re_scope_id) && re_scope_id != cur_node_scope_id {
                        ctx.diagnostic(redeclaration_diagnostic(
                            item.id.span(),
                            redeclaration.span,
                            name,
                        ));
                    }
                }
                // e.g. "var a = 4; console.log(a);"
                for reference in ctx.scoping().get_resolved_references(symbol_id) {
                    let reference_scope_id = ctx.nodes().get_node(reference.node_id()).scope_id();
                    if !scope_arr.contains(&reference_scope_id)
                        && reference_scope_id != cur_node_scope_id
                    {
                        ctx.diagnostic(use_outside_scope_diagnostic(
                            item.id.span(),
                            ctx.reference_span(reference),
                            name,
                        ));
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
        "if (1) {
            var a = 4;
        } else {
            let a = 4;
            console.log(a);
        }",
        "function f() { } f(); var exports = { f: f };",
        "var f = () => {}; f(); var exports = { f: f };",
        "!function f(){ f; }",
        "function f() { } f(); var exports = { f: f };",
        "function f() { var a, b; { a = true; } b = a; }",
        "var a; function f() { var b = a; }",
        "function f(a) { }",
        "!function(a) { };",
        "!function f(a) { };",
        "function f(a) { var b = a; }",
        "!function f(a) { var b = a; };",
        "function f() { var g = f; }",
        "function f() { } function g() { var f = g; }",
        "function f() { var hasOwnProperty; { hasOwnProperty; } }",
        "function f(){ a; b; var a, b; }",
        "function f(){ g(); function g(){} }",
        "if (true) { var a = 1; a; }",
        "var a; if (true) { a; }",
        "for (var i = 0; i < 10; i++) { i; }",
        "var i; for(i; i; i) { i; }",
        r#"
            function myFunc(foo) {  "use strict";  var { bar } = foo;  bar.hello();}
        "#,
        r#"
            function myFunc(foo) {  "use strict";  var [ bar ]  = foo;  bar.hello();}
        "#,
        r#"
            const React = require("react/addons");const cx = React.addons.classSet;
        "#,
        r#"import * as y from "./other.js"; y();"#,
        "var v = 1;  function x() { return v; };",
        "function myFunc(...foo) {  return foo;}",
        "var f = () => { var g = f; }",
        "class Foo {}\nexport default Foo;",
        "foo; class C { static {} } var foo; ",
        "var foo; class C { static {} [foo]; } ",
        "class C { static { foo; } } var foo;",
        "var foo; class C { static { foo; } } ",
        "class C { static { if (bar) { foo; } var foo; } }",
        "class C { static { foo; var foo; } }",
        "class C { static { var foo; foo; } }",
        "(function () { foo(); })(); function foo() {}",
        "(function () { foo(); })(); function foo() {}",
        "foo: while (true) { bar: for (var i = 0; i < 13; ++i) {if (i === 7) break foo; } }",
        "foo: while (true) { bar: for (var i = 0; i < 13; ++i) {if (i === 7) continue foo; } }",
        "var a = { foo: bar };",
        "var a = { foo: foo };",
        "var a = { bar: 7, foo: bar };",
        "var a = arguments;",
        r#"
            function a(n) { return n > 0 ? b(n - 1) : "a"; } function b(n) { return n > 0 ? a(n - 1) : "b"; }
        "#,
        "const { dummy: { data, isLoading }, auth: { isLoggedIn } } = this.props;",
        "/*global prevState*/ const { virtualSize: prevVirtualSize = 0 } = prevState;",
        "/*global React*/ let {PropTypes, addons: {PureRenderMixin}} = React; let Test = React.createClass({mixins: [PureRenderMixin]});",
        r"
            function doIf() {
                var build;

                if (true) {
                    build = true;
                }

                console.log(build);
            }
        ",
        r"
            function doIfElse() {
                var build;

                if (true) {
                    build = true;
                } else {
                    build = false;
                }
            }
        ",
        r"
            function doTryCatch() {
                var build;
                var f;

                try {
                    build = 1;
                } catch (e) {
                    f = build;
                }
            }
        ",
    ];

    let fail = vec![
        r"
            if (true) {
                var a = 3;
            } else {
                var b = 4;
                var a = 4;
            }
            console.log(a, b);
        ",
        r"
            if (true) {
                var [a, b] = [1, 2];
            }
            console.log(a, b);
        ",
        r"
            if (true) {
                var a = 4, b = 3;
            }
            console.log(a, b);
        ",
        r"
            if (true) {
                {
                    var a = 4, b = 3;
                }
                var a = 4;
            }
        ",
        "function f(){ x; { var x; } }",
        "function f(){ { var x; } x; }",
        "function f() { var a; { var b = 0; } a = b; }",
        "function f() { try { var a = 0; } catch (e) { var b = a; } }",
        "function a() { for(var b in {}) { var c = b; } c; }",
        "function a() { for(var b of {}) { var c = b; } c; }",
        "function f(){ switch(2) { case 1: var b = 2; b; break; default: b; break;} b; }",
        "for (var a = 0;;) {} a;",
        "for (var a in []) {} a;",
        "for (var a of []) {} a;",
        "{ var a = 0; } a;",
        "if (true) { var a; } a;",
        "if (true) { var a = 1; } else { var a = 2; }",
        "for (var i = 0;;) {} for(var i = 0;;) {}",
        "class C { static { if (bar) { var foo; } foo; } }",
        "{ var foo,  bar; } bar;",
        "if (foo) { var a = 1; } else if (bar) { var a = 2; } else { var a = 3; }",
        r"
            if (true) {
                var build = true;
                console.log(build);
                {
                    conosole.log(build);
                    {
                        console.log(build);
                    }
                }
            }
            console.log(build);
            let t = build;
        ",
        r"
            function doIf() {
                if (true) {
                    var build = true;
                }

                console.log(build);
            }
        ",
        r"
            function doIfElse() {
                if (true) {
                    var build = true;
                } else {
                    var build = false;
                }
            }
        ",
        r"
            function doTryCatch() {
                try {
                    var build = 1;
                } catch (e) {
                    var f = build;
                }
            }
        ",
        r"
            function doFor() {
                for (var x = 1; x < 10; x++) {
                    var y = f(x);
                }
                console.log(y);
            }
        ",
        r"
            class C {
                static {
                    if (something) {
                        var build = true;
                    }
                    build = false;
                }
            }
        ",
    ];

    Tester::new(BlockScopedVar::NAME, BlockScopedVar::PLUGIN, pass, fail).test_and_snapshot();
}
