use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_delete_var_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("variables should not be deleted").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeleteVar;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The purpose of the `delete` operator is to remove a property from an
    /// object.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `delete` operator on a variable might lead to unexpected
    /// behavior.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x;
    /// delete x;
    /// ```
    NoDeleteVar,
    eslint,
    correctness
);

impl Rule for NoDeleteVar {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UnaryExpression(expr) = node.kind() else {
            return;
        };
        if expr.operator == UnaryOperator::Delete && expr.argument.is_identifier_reference() {
            ctx.diagnostic(no_delete_var_diagnostic(expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("delete x.prop;", None)];

    let fail = vec![("delete x", None)];

    Tester::new(NoDeleteVar::NAME, NoDeleteVar::PLUGIN, pass, fail).test_and_snapshot();
}
