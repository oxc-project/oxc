use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn react_in_jsx_scope_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-react(react-in-jsx-scope): 'React' must be in scope when using JSX")
        .with_help("When using JSX, `<a />` expands to `React.createElement(\"a\")`. Therefore the `React` variable must be in scope.")
        .with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct ReactInJsxScope;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow missing React when using JSX
    ///
    /// ### Why is this bad?
    ///
    /// When using JSX, `<a />` expands to `React.createElement("a")`. Therefore the `React` variable must be in scope.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// var a = <a />;
    ///
    /// // Good
    /// import React from "react";
    /// var a = <a />;
    ///
    /// ```
    ReactInJsxScope,
    suspicious
);

impl Rule for ReactInJsxScope {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let node_span = match node.kind() {
            AstKind::JSXOpeningElement(v) => v.name.span(),
            AstKind::JSXFragment(v) => v.opening_fragment.span,
            _ => return,
        };
        let scope = ctx.scopes();
        let react_name = "React";
        if scope.get_binding(scope.root_scope_id(), react_name).is_some() {
            return;
        }

        if !scope
            .ancestors(node.scope_id())
            .any(|v| scope.get_bindings(v).iter().any(|(k, _)| k.as_str() == react_name))
        {
            ctx.diagnostic(react_in_jsx_scope_diagnostic(node_span));
        }
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

    Tester::new(ReactInJsxScope::NAME, pass, fail).test_and_snapshot();
}
