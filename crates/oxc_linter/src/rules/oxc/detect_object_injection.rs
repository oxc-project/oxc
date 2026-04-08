use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn variable_assigned_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Variable Assigned to Object Injection Sink")
        .with_help(
            "Avoid indexing objects with untrusted dynamic keys without validating the key first.",
        )
        .with_label(span)
}

fn function_call_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Function Call Object Injection Sink")
        .with_help("Avoid passing dynamically indexed object access into security-sensitive flows without validating the key first.")
        .with_label(span)
}

fn generic_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Generic Object Injection Sink")
        .with_help(
            "Avoid indexing objects with untrusted dynamic keys without validating the key first.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectObjectInjection;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects computed object member access with identifier keys such as `object[key]`.
    ///
    /// ### Why is this bad?
    ///
    /// Dynamically indexing objects with attacker-controlled keys can lead to
    /// prototype pollution, unexpected property access, or other unsafe behavior
    /// when the key is not validated first.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const value = object[key];
    /// object[key] = nextValue;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const value = object.safeKey;
    /// const value = object["fixed-key"];
    /// ```
    DetectObjectInjection,
    oxc,
    suspicious,
    none
);

impl Rule for DetectObjectInjection {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ComputedMemberExpression(member_expr) = node.kind() else {
            return;
        };

        if !matches!(&member_expr.expression, Expression::Identifier(_)) {
            return;
        }

        match ctx.nodes().parent_kind(node.id()) {
            AstKind::VariableDeclarator(_) => {
                ctx.diagnostic(variable_assigned_diagnostic(member_expr.span));
            }
            AstKind::CallExpression(_) => {
                ctx.diagnostic(function_call_diagnostic(member_expr.span));
            }
            _ => {
                ctx.diagnostic(generic_diagnostic(member_expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const obj = {};",
        "const value = obj.staticKey;",
        "const value = obj['static-key'];",
        "const value = obj[0];",
        "const value = obj[key + suffix];",
        "const value = obj[getKey()];",
        "obj['safe']();",
    ];

    let fail = vec![
        "const value = obj[key];",
        "foo(obj[key]);",
        "obj[key] = nextValue;",
        "return obj[key];",
        "obj[key]();",
    ];

    Tester::new(DetectObjectInjection::NAME, DetectObjectInjection::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
