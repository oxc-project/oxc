use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn prefer_to_be_disabled_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `toBeDisabled()` over checking the `disabled` attribute.")
        .with_label(span)
}

fn is_disabled_attribute_name(argument: &Argument) -> bool {
    match argument {
        Argument::StringLiteral(string_literal) => string_literal.value.as_str() == "disabled",
        Argument::TemplateLiteral(template_literal) => {
            template_literal.single_quasi().is_some_and(|quasi| quasi.as_str() == "disabled")
        }
        _ => false,
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreferToBeDisabled;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces using `toBeDisabled()` when checking that an element
    /// is disabled.
    ///
    /// ### Why is this bad?
    ///
    /// `toHaveAttribute('disabled')` only checks for the presence of the
    /// `disabled` attribute. `toBeDisabled()` expresses the intent more clearly
    /// and matches Testing Library's disabled-state semantics.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(locator).toHaveAttribute('disabled');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(locator).toBeDisabled();
    /// ```
    PreferToBeDisabled,
    vitest,
    style,
    fix,
    version = "1.16.0",
);

impl Rule for PreferToBeDisabled {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(expect_call) = parse_expect_jest_fn_call(call_expr, jest_node, ctx) else {
            return;
        };

        let Some(matcher) = expect_call.matcher() else {
            return;
        };

        if matcher.is_name_unequal("toHaveAttribute")
            || expect_call.modifiers().iter().any(|modifier| modifier.is_name_equal("not"))
        {
            return;
        }

        let Some([attribute_name]) = expect_call.matcher_arguments.map(|args| args.as_slice())
        else {
            return;
        };

        if !is_disabled_attribute_name(attribute_name) {
            return;
        }

        let is_computed = match matcher.parent {
            Some(Expression::ComputedMemberExpression(_)) => true,
            Some(Expression::StaticMemberExpression(_) | Expression::PrivateFieldExpression(_)) => {
                false
            }
            _ => return,
        };

        ctx.diagnostic_with_fix(prefer_to_be_disabled_diagnostic(matcher.span), |fixer| {
            let replacement = if is_computed { "[\"toBeDisabled\"]()" } else { "toBeDisabled()" };
            let start = if is_computed { matcher.span.start - 1 } else { matcher.span.start };
            fixer.replace(Span::new(start, call_expr.span.end), replacement)
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect(locator).toBeDisabled();",
        "expect(locator).toHaveAttribute();",
        "expect(locator).toHaveAttribute('aria-disabled');",
        "expect(locator).toHaveAttribute('disabled', '');",
        "expect(locator).toHaveAttribute('disabled', true);",
        "expect(locator).not.toHaveAttribute('disabled');",
        "expect(locator).toHaveAttribute(attributeName);",
        "expect(locator).toHaveAttribute(`dis${abled}`);",
        "expect(locator).toHaveAttribute;",
        "assert(locator).toHaveAttribute('disabled');",
    ];

    let fail = vec![
        "expect(locator).toHaveAttribute('disabled');",
        "expect(locator).toHaveAttribute(\"disabled\");",
        "expect(locator).toHaveAttribute(`disabled`);",
        "expect(locator)[\"toHaveAttribute\"]('disabled');",
        "expect(locator).resolves.toHaveAttribute('disabled');",
    ];

    let fix = vec![
        ("expect(locator).toHaveAttribute('disabled');", "expect(locator).toBeDisabled();"),
        ("expect(locator).toHaveAttribute(\"disabled\");", "expect(locator).toBeDisabled();"),
        ("expect(locator).toHaveAttribute(`disabled`);", "expect(locator).toBeDisabled();"),
        (
            "expect(locator)[\"toHaveAttribute\"]('disabled');",
            "expect(locator)[\"toBeDisabled\"]();",
        ),
        (
            "expect(locator).resolves.toHaveAttribute('disabled');",
            "expect(locator).resolves.toBeDisabled();",
        ),
    ];

    Tester::new(PreferToBeDisabled::NAME, PreferToBeDisabled::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
