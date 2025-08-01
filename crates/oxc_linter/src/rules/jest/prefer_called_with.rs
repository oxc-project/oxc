use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn use_to_be_called_with(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toBeCalledWith()` or `toHaveBeenCalledWith()`.")
        .with_help("Prefer toBeCalledWith(/* expected args */)")
        .with_label(span)
}

fn use_have_been_called_with(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toBeCalledWith()` or `toHaveBeenCalledWith()`.")
        .with_help("Prefer toHaveBeenCalledWith(/* expected args */)")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferCalledWith;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Suggest using `toBeCalledWith()` or `toHaveBeenCalledWith()`
    ///
    /// ### Why is this bad?
    ///
    /// When testing function calls, it's often more valuable to assert both
    /// that a function was called AND what arguments it was called with.
    /// Using `toBeCalled()` or `toHaveBeenCalled()` only verifies the function
    /// was invoked, but doesn't validate the arguments, potentially missing
    /// bugs where functions are called with incorrect parameters.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(someFunction).toBeCalled();
    /// expect(someFunction).toHaveBeenCalled();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(noArgsFunction).toBeCalledWith();
    /// expect(roughArgsFunction).toBeCalledWith(expect.anything(), expect.any(Date));
    /// expect(anyArgsFunction).toBeCalledTimes(1);
    /// expect(uncalledFunction).not.toBeCalled();
    /// ```
    PreferCalledWith,
    jest,
    style,
);

impl Rule for PreferCalledWith {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferCalledWith {
    pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let has_not_modifier =
            jest_fn_call.modifiers().iter().any(|modifier| modifier.is_name_equal("not"));

        if has_not_modifier {
            return;
        }

        if let Some(matcher_property) = jest_fn_call.matcher() {
            if let Some(matcher_name) = matcher_property.name() {
                if matcher_name == "toBeCalled" {
                    ctx.diagnostic(use_to_be_called_with(matcher_property.span));
                } else if matcher_name == "toHaveBeenCalled" {
                    ctx.diagnostic(use_have_been_called_with(matcher_property.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect(fn).toBeCalledWith();", None),
        ("expect(fn).toHaveBeenCalledWith();", None),
        ("expect(fn).toBeCalledWith(expect.anything());", None),
        ("expect(fn).toHaveBeenCalledWith(expect.anything());", None),
        ("expect(fn).not.toBeCalled();", None),
        ("expect(fn).rejects.not.toBeCalled();", None),
        ("expect(fn).not.toHaveBeenCalled();", None),
        ("expect(fn).not.toBeCalledWith();", None),
        ("expect(fn).not.toHaveBeenCalledWith();", None),
        ("expect(fn).resolves.not.toHaveBeenCalledWith();", None),
        ("expect(fn).toBeCalledTimes(0);", None),
        ("expect(fn).toHaveBeenCalledTimes(0);", None),
        ("expect(fn);", None),
    ];

    let fail = vec![
        ("expect(fn).toBeCalled();", None),
        ("expect(fn).resolves.toBeCalled();", None),
        ("expect(fn).toHaveBeenCalled();", None),
    ];

    Tester::new(PreferCalledWith::NAME, PreferCalledWith::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
