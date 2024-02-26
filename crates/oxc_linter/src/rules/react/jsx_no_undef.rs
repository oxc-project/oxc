use oxc_ast::{
    ast::{
        JSXElementName, JSXIdentifier, JSXMemberExpression, JSXMemberExpressionObject,
        JSXOpeningElement,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactString, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(jsx-no-undef): Disallow undeclared variables in JSX")]
#[diagnostic(severity(warning), help("'{0}' is not defined."))]
struct JsxNoUndefDiagnostic(CompactString, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct JsxNoUndef;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow undeclared variables in JSX
    ///
    /// ### Why is this bad?
    /// It is most likely a potential ReferenceError caused by a misspelling of a variable or parameter name.
    ///
    /// ### Example
    /// ```jsx
    /// const A = () => <App />
    /// const C = <B />
    /// ```
    JsxNoUndef,
    correctness
);

fn get_member_ident<'a>(expr: &'a JSXMemberExpression<'a>) -> &'a JSXIdentifier {
    match expr.object {
        JSXMemberExpressionObject::Identifier(ref ident) => ident,
        JSXMemberExpressionObject::MemberExpression(ref next_expr) => get_member_ident(next_expr),
    }
}
fn get_resolvable_ident<'a>(node: &'a JSXElementName<'a>) -> Option<&'a JSXIdentifier> {
    match node {
        JSXElementName::Identifier(ref ident)
            if !(ident.name.as_str().starts_with(char::is_lowercase)) =>
        {
            Some(ident)
        }
        JSXElementName::Identifier(_) | JSXElementName::NamespacedName(_) => None,
        JSXElementName::MemberExpression(expr) => Some(get_member_ident(expr)),
    }
}

impl Rule for JsxNoUndef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(JSXOpeningElement { name: el_name, .. }) = &node.kind() {
            if let Some(ident) = get_resolvable_ident(el_name) {
                if ident.name.as_str() == "this" {
                    return;
                }
                let jsx_scope_id = node.scope_id();
                for scope_id in ctx.scopes().ancestors(jsx_scope_id) {
                    if ctx.scopes().has_binding(scope_id, &ident.name) {
                        return;
                    }
                }
                ctx.diagnostic(JsxNoUndefDiagnostic(ident.name.to_compact_string(), ident.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var React, App; React.render(<App />);", None),
        ("var React; React.render(<img />);", None),
        ("var React; React.render(<x-gif />);", None),
        ("var React, app; React.render(<app.Foo />);", None),
        ("var React, app; React.render(<app.foo.Bar />);", None),
        ("var React; React.render(<Apppp:Foo />);", None),
        (
            r"
        var React;
        class Hello extends React.Component {
          render() {
            return <this.props.tag />
          }
        }
        ",
            None,
        ),
        // TODO: Text should be declared in globals ("var React; React.render(<Text />);", None),
        (
            r#"
        import Text from "cool-module";
        const TextWrapper = function (props) {
          return (
            <Text />
          );
        };
        "#,
            None,
        ),
        ("var App; var React; enum A { App };  React.render(<App />);", None),
        ("var React; enum A { App }; var App; React.render(<App />);", None),
    ];

    let fail = vec![
        ("var React; React.render(<App />);", None),
        ("var React; React.render(<Appp.Foo />);", None),
        ("var React; React.render(<appp.Foo />);", None),
        ("var React; React.render(<appp.foo.Bar />);", None),
        ("var React; React.render(<Foo />);", None),
        ("var React; Unknown; React.render(<Unknown />)", None),
        ("var React; { const App = null; }; React.render(<App />);", None),
        ("var React; enum A { App }; React.render(<App />);", None),
    ];

    Tester::new(JsxNoUndef::NAME, pass, fail).test_and_snapshot();
}
