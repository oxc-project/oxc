use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_forward_ref_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `React.forwardRef`.")
        .with_help("In React 19+, `ref` is available as a prop. Use the `ref` prop directly instead of `forwardRef`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoForwardRef;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `React.forwardRef`.
    ///
    /// ### Why is this bad?
    ///
    /// Starting with React 19, `ref` is available as a regular prop.
    /// `forwardRef` is no longer needed and adds unnecessary complexity.
    /// Components should accept `ref` as a prop directly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const MyInput = React.forwardRef((props, ref) => (
    ///   <input ref={ref} {...props} />
    /// ));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function MyInput({ ref, ...props }) {
    ///   return <input ref={ref} {...props} />;
    /// }
    /// ```
    NoForwardRef,
    react,
    style,
    pending
);

impl Rule for NoForwardRef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };

        match &call.callee {
            // React.forwardRef(...)
            Expression::StaticMemberExpression(member) => {
                if member.property.name.as_str() == "forwardRef" {
                    if let Expression::Identifier(obj) = &member.object {
                        if obj.name == "React" {
                            ctx.diagnostic(no_forward_ref_diagnostic(call.span));
                        }
                    }
                }
            }
            // forwardRef(...)
            Expression::Identifier(ident) => {
                if ident.name == "forwardRef" {
                    ctx.diagnostic(no_forward_ref_diagnostic(call.span));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function MyInput({ ref, ...props }) { return <input ref={ref} {...props} />; }",
        "const x = React.memo(() => <div />);",
    ];

    let fail = vec![
        "const MyInput = React.forwardRef((props, ref) => <input ref={ref} />);",
        "const MyInput = forwardRef((props, ref) => <input ref={ref} />);",
    ];

    Tester::new(NoForwardRef::NAME, NoForwardRef::PLUGIN, pass, fail).test_and_snapshot();
}
