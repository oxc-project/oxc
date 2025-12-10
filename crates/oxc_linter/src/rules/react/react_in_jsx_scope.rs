use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn react_in_jsx_scope_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`React` must be in scope when using JSX.")
        .with_help("When using JSX, `<a />` expands to `React.createElement(\"a\")`. Therefore the `React` variable must be in scope.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ReactInJsxScope;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that React is imported and in-scope when using JSX syntax.
    ///
    /// Note that this rule is **not necessary** on React 17+ if you are using
    /// the new JSX Transform, and you can disable this rule and skip importing
    /// `React` in files with JSX syntax.
    ///
    /// If your `tsconfig.json` has `jsx` set to `react-jsx` or `react-jsxdev`, you are using the new JSX Transform.
    /// For JavaScript projects using Babel, you are using the new JSX Transform if your React preset configuration
    /// (in `.babelrc` or `babel.config.js`) has `runtime: "automatic"`.
    ///
    /// For more information, see
    /// [the React blog post on JSX Transform](https://legacy.reactjs.org/blog/2020/09/22/introducing-the-new-jsx-transform.html#eslint).
    ///
    /// ### Why is this bad?
    ///
    /// When using JSX, `<a />` expands to `React.createElement("a")`. Therefore
    /// the `React` variable must be in scope.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const a = <a />;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import React from "react";
    /// const a = <a />;
    /// ```
    ReactInJsxScope,
    react,
    suspicious
);

impl Rule for ReactInJsxScope {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let node_span = match node.kind() {
            AstKind::JSXOpeningElement(v) => v.name.span(),
            AstKind::JSXFragment(v) => v.opening_fragment.span,
            _ => return,
        };
        let scope = ctx.scoping();
        let react_name = "React";
        if scope.get_binding(scope.root_scope_id(), react_name).is_some() {
            return;
        }

        if scope.find_binding(node.scope_id(), react_name).is_none() {
            ctx.diagnostic(react_in_jsx_scope_diagnostic(node_span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var React, App; <App />;", None),
        ("var React; <img />;", None),
        ("var React; <>fragment</>;", None),
        ("var React; <x-gif />;", None),
        ("var React, App, a=1; <App attr={a} />;", None),
        ("var React, App, a=1; function elem() { return <App attr={a} />; }", None),
        ("var React, App; <App />;", None),
        (
            "
			        import React from 'react/addons';
			        const Button = createReactClass({
			          render() {
			            return (
			              <button {...this.props}>{this.props.children}</button>
			            )
			          }
			        });
			        export default Button;
			      ",
            None,
        ),
        ("var React, a = <img />;", None),
    ];

    let fail = vec![
        ("var App, a = <App />;", None),
        ("var a = <App />;", None),
        ("var a = <img />;", None),
        ("var a = <>fragment</>;", None),
        ("var Foo, a = <img />;", None),
    ];

    Tester::new(ReactInJsxScope::NAME, ReactInJsxScope::PLUGIN, pass, fail).test_and_snapshot();
}
