use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_render_return_value_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not depend on the return value from ReactDOM.render.")
        .with_help("Using the return value is a legacy feature.")
        .with_label(span0)
}

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
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        let Expression::Identifier(ident) = member_expr.object() else {
            return;
        };
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
                            ctx.diagnostic(no_render_return_value_diagnostic(
                                ident.span.merge(&property_span),
                            ));
                        }

                        let scope_id = parent_node.scope_id();
                        if ctx.scopes().get_flags(scope_id).is_arrow() {
                            if let AstKind::ArrowFunctionExpression(e) =
                                ctx.nodes().kind(ctx.scopes().get_node_id(scope_id))
                            {
                                if e.expression {
                                    ctx.diagnostic(no_render_return_value_diagnostic(
                                        ident.span.merge(&property_span),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn should_run(&self, ctx: &LintContext) -> bool {
        ctx.source_type().is_jsx()
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
        (
            "export const foo = () => ({ destroy: ({ dom }) => { ReactDOM.unmountComponentAtNode(dom); } });",
            None,
        ),
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
        // See https://github.com/oxc-project/oxc/pull/1042#discussion_r1369762147
        // ("var inst = React.render(<div />, document.body);", None),
        // ("var inst = React.render(<div />, document.body);", None),
    ];

    Tester::new(NoRenderReturnValue::NAME, pass, fail).test_and_snapshot();
}
