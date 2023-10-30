use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-render-return-value): Do not depend on the return value from ReactDOM.render.")]
#[diagnostic(severity(warning), help("Using the return value is a legacy feature."))]
struct NoRenderReturnValueDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoRenderReturnValue;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule will warn you if you try to use the ReactDOM.render() return value.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// vaa inst =ReactDOM.render(<App />, document.body);
    /// function render() {
    ///  return ReactDOM.render(<App />, document.body);
    /// }
    ///
    /// // Good
    /// ReactDOM.render(<App />, document.body);
    /// ```
    NoRenderReturnValue,
    correctness
);

impl Rule for NoRenderReturnValue {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Expression::MemberExpression(member_expr) = &call_expr.callee else { return };
        let Expression::Identifier(ident) = member_expr.object() else { return };
        if ident.name == "ReactDOM" {
            if let Some((property_span, property_name)) = member_expr.static_property_info() {
                if property_name == "render" {
                    if let Some(parent_node) = ctx.nodes().parent_node(node.id()) {
                        if matches!(
                            parent_node.kind(),
                            AstKind::VariableDeclarator(_)
                                | AstKind::ObjectProperty(_)
                                | AstKind::ReturnStatement(_)
                                | AstKind::AssignmentExpression(_)
                        ) {
                            ctx.diagnostic(NoRenderReturnValueDiagnostic(
                                ident.span.merge(&property_span),
                            ));
                        }

                        let is_arrow_function = ctx
                            .scopes()
                            .get_flags(parent_node.scope_id())
                            .contains(ScopeFlags::Arrow);

                        if is_arrow_function {
                            for node_id in ctx.nodes().ancestors(parent_node.id()).skip(1) {
                                let node = ctx.nodes().get_node(node_id);
                                if let AstKind::ArrowExpression(e) = node.kind() {
                                    if e.expression {
                                        ctx.diagnostic(NoRenderReturnValueDiagnostic(
                                            ident.span.merge(&property_span),
                                        ));
                                    } else {
                                        break;
                                    }
                                }
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
        ("ReactDOM.render(<div />, document.body);", None),
        (
            "
        	        let node;
        	        ReactDOM.render(<div ref={ref => node = ref}/>, document.body);
        	      ",
            None,
        ),
        ("ReactDOM.render(<div ref={ref => this.node = ref}/>, document.body);", None),
        ("React.render(<div ref={ref => this.node = ref}/>, document.body);", None),
        ("React.render(<div ref={ref => this.node = ref}/>, document.body);", None),
        ("var foo = React.render(<div />, root);", None),
        ("var foo = render(<div />, root)", None),
        ("var foo = ReactDom.renderder(<div />, root)", None),
        ("var foo = ReactDom.renderder(<div />, root)", None),
        ("export const foo = () => ({ destroy: ({ dom }) => { ReactDOM.unmountComponentAtNode(dom); } });", None),
    ];

    let fail = vec![
        ("var Hello = ReactDOM.render(<div />, document.body);", None),
        (
            "
        	        var o = {
        	          inst: ReactDOM.render(<div />, document.body)
        	        };
        	      ",
            None,
        ),
        (
            "
        	        function render () {
        	          return ReactDOM.render(<div />, document.body)
        	        }
        	      ",
            None,
        ),
        ("var render = (a, b) => ReactDOM.render(a, b)", None),
        ("this.o = ReactDOM.render(<div />, document.body);", None),
        ("var v; v = ReactDOM.render(<div />, document.body);", None),
        ("var inst = ReactDOM.render(<div />, document.body);", None),
        // This rule is only supported for react versions >= 15.0.0, so the following are not supported.
        // See https://github.com/web-infra-dev/oxc/pull/1042#discussion_r1369762147
        // ("var inst = React.render(<div />, document.body);", None),
        // ("var inst = React.render(<div />, document.body);", None),
    ];

    Tester::new(NoRenderReturnValue::NAME, pass, fail).test_and_snapshot();
}
