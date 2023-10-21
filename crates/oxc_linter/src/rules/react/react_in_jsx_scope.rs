use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("")]
#[diagnostic(severity(warning), help(""))]
struct ReactInJsxScopeDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ReactInJsxScope;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    ReactInJsxScope,
    correctness
);

impl Rule for ReactInJsxScope {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !matches!(node.kind(), AstKind::JSXOpeningElement(_) | AstKind::JSXFragment(_)) {
            return;
        }

        // const variables = variableUtil.variablesInScope(context);
        // if (variableUtil.findVariable(variables, pragma)) {
        //   return;
        // }
        // report(context, messages.notInScope, 'notInScope', {
        //   node,
        //   data: {
        //     name: pragma,
        //   },
        // });

        let scope = ctx.scopes();
        let nodes = ctx.nodes();
        let symbols = ctx.symbols();

        let mut incl_react = false;

        scope.ancestors(node.scope_id()).for_each(|v| {
            println!("ancestor: {:?}", v);

            println!("{:?}", scope.get_bindings(v));

            scope.get_bindings(v).iter().for_each(|(k, v)| {
                println!("{}: {:?}", k, v);

                if k.as_str() == "React" {
                    incl_react = true;
                }
            });
        });

        if !incl_react {
            ctx.diagnostic(ReactInJsxScopeDiagnostic(Span { start: 0, end: 1 }));
        }

        // scope.ancestors(scope_id).find_map(|id| scope.get_binding(id, &ident.name)).map_or_else(
        //     || {
        //         panic!(
        //             "No binding id found for {}, but this IdentifierReference
        //         is not a global",
        //             &ident.name
        //         );
        //     },
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
        // ("/** @jsx Foo */ var Foo, App; <App />;", None),
        // ("/** @jsx Foo.Bar */ var Foo, App; <App />;", None),
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
        // ("var Foo, App; <App />;", None),
    ];

    let fail = vec![
        ("var App, a = <App />;", None),
        ("var a = <App />;", None),
        ("var a = <img />;", None),
        ("var a = <>fragment</>;", None),
        // ("/** @jsx React.DOM */ var a = <img />;", None),
        // ("/** @jsx Foo.bar */ var React, a = <img />;", None),
        ("var React, a = <img />;", None),
    ];

    Tester::new(ReactInJsxScope::NAME, pass, fail).test_and_snapshot();
}
