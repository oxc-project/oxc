use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_create_ref_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("React.createRef() should not be used in function components")
        .with_help("Use useRef() hook instead of React.createRef() in function components. createRef creates a new ref object on every render.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCreateRef;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects use of `React.createRef()` or `createRef()`.
    ///
    /// ### Why is this bad?
    ///
    /// `createRef()` creates a new ref object on every render. In function
    /// components, use `useRef()` instead, which persists the ref across renders.
    /// In class components, create refs in the constructor instead of in render.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// const ref = React.createRef();
    /// const ref = createRef();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const ref = useRef(null);
    /// ```
    NoCreateRef,
    oxc,
    correctness,
    none
);

impl Rule for NoCreateRef {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let is_create_ref = match &call_expr.callee {
            Expression::Identifier(ident) => ident.name == "createRef",
            Expression::StaticMemberExpression(member) => {
                member.property.name == "createRef"
                    && matches!(&member.object, Expression::Identifier(obj) if obj.name == "React")
            }
            _ => false,
        };

        if is_create_ref {
            ctx.diagnostic(no_create_ref_diagnostic(call_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass =
        vec!["const ref = useRef(null);", "const ref = React.useRef(null);", "foo.createRef()"];

    let fail = vec!["const ref = React.createRef();", "const ref = createRef();", "createRef();"];
    Tester::new(NoCreateRef::NAME, NoCreateRef::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
