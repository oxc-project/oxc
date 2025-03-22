use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression, MethodDefinitionKind, PropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_accessor_recursion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow recursive access to `this` within getters and setters.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAccessorRecursion;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoAccessorRecursion,
    unicorn,
    correctness,
);

impl Rule for NoAccessorRecursion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MemberExpression(expr) = node.kind() else {
            return;
        };
        let MemberExpression::StaticMemberExpression(member_expr) = expr else {
            return;
        };
        if !matches!(member_expr.object, Expression::ThisExpression(_)) {
            return;
        }
        let key_name = member_expr.property.name.as_str();

        let Some(func) = get_closest_function(node, ctx) else {
            return;
        };
        if let Some(prop_or_method) = ctx.nodes().parent_node(func.id()) {
            match prop_or_method.kind() {
                AstKind::ObjectProperty(property)
                    if !property.computed
                        && matches!(property.kind, PropertyKind::Get | PropertyKind::Set) =>
                {
                    if property.key.name().unwrap().to_string() == key_name {
                        ctx.diagnostic(no_accessor_recursion_diagnostic(member_expr.span));
                    }
                }
                AstKind::MethodDefinition(method_def)
                    if !method_def.computed
                        && matches!(
                            method_def.kind,
                            MethodDefinitionKind::Get | MethodDefinitionKind::Set
                        ) => {}
                _ => {}
            }
        }
    }
}

fn get_closest_function<'a>(node: &AstNode, ctx: &'a LintContext) -> Option<&'a AstNode<'a>> {
    let mut parent = ctx.nodes().parent_node(node.id())?;

    loop {
        match parent.kind() {
            AstKind::Function(_) => {
                break;
            }
            _ => {
                parent = ctx.nodes().parent_node(parent.id())?;
            }
        }
    }
    Some(parent)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
            const foo = {
                set bar(value) {
                    this._bar = value;
                }
            };
        ",
    ];

    let fail = vec![
        r"
            const foo = {
                get bar(value) {
                    this.bar
                }
            };
        ",
    ];

    Tester::new(NoAccessorRecursion::NAME, NoAccessorRecursion::PLUGIN, pass, fail)
        .test_and_snapshot();
}
