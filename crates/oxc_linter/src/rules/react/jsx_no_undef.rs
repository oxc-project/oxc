use oxc_ast::{
    ast::{IdentifierReference, JSXElementName, JSXMemberExpression, JSXMemberExpressionObject},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn jsx_no_undef_diagnostic(ident_name: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{ident_name}' is not defined.")).with_label(span1)
}

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
    react,
    correctness
);

fn get_resolvable_ident<'a>(node: &'a JSXElementName<'a>) -> Option<&'a IdentifierReference<'a>> {
    match node {
        JSXElementName::Identifier(_)
        | JSXElementName::NamespacedName(_)
        | JSXElementName::ThisExpression(_) => None,
        JSXElementName::IdentifierReference(ident) => Some(ident),
        JSXElementName::MemberExpression(expr) => get_member_ident(expr),
    }
}

fn get_member_ident<'a>(
    mut expr: &'a JSXMemberExpression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    loop {
        match &expr.object {
            JSXMemberExpressionObject::IdentifierReference(ident) => return Some(ident),
            JSXMemberExpressionObject::ThisExpression(_) => return None,
            JSXMemberExpressionObject::MemberExpression(next_expr) => {
                expr = next_expr;
            }
        }
    }
}

impl Rule for JsxNoUndef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(elem) = &node.kind() {
            if let Some(ident) = get_resolvable_ident(&elem.name) {
                let reference = ctx.symbols().get_reference(ident.reference_id());
                if reference.symbol_id().is_some() {
                    return;
                }
                let name = ident.name.as_str();
                if ctx.globals().is_enabled(name) {
                    return;
                }
                ctx.diagnostic(jsx_no_undef_diagnostic(name, ident.span));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use serde_json::json;

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
        ("var React; import App = require('./app'); React.render(<App />);", None),
        (
            "
        var React;
        import { Foo } from './foo';
        import App = Foo.App;
        React.render(<App />);
        ",
            None,
        ),
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

    Tester::new(JsxNoUndef::NAME, JsxNoUndef::PLUGIN, pass, fail).test_and_snapshot();

    let pass = vec![("let x = <A.B />;", None, Some(json!({ "globals": {"A": "readonly" } })))];
    let fail = vec![("let x = <A.B />;", None, None)];
    Tester::new(JsxNoUndef::NAME, JsxNoUndef::PLUGIN, pass, fail).test();
}
