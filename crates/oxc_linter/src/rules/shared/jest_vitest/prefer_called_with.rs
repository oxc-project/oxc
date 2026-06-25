use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
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

pub const DOCUMENTATION: &str = r"### What it does

Suggest using `toBeCalledWith()` or `toHaveBeenCalledWith()`

### Why is this bad?

When testing function calls, it's often more valuable to assert both
that a function was called AND what arguments it was called with.
Using `toBeCalled()` or `toHaveBeenCalled()` only verifies the function
was invoked, but doesn't validate the arguments, potentially missing
bugs where functions are called with incorrect parameters.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(someFunction).toBeCalled();
expect(someFunction).toHaveBeenCalled();
```

Examples of **correct** code for this rule:
```javascript
expect(noArgsFunction).toBeCalledWith();
expect(roughArgsFunction).toBeCalledWith(expect.anything(), expect.any(Date));
expect(anyArgsFunction).toBeCalledTimes(1);
expect(uncalledFunction).not.toBeCalled();
```
";

pub fn run_on_jest_node<'a, 'c>(
    possible_jest_node: &PossibleJestNode<'a, 'c>,
    ctx: &'c LintContext<'a>,
) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };

    let has_not_modifier =
        jest_fn_call.modifiers().iter().any(|modifier| modifier.is_name_equal("not"));

    if has_not_modifier {
        return;
    }

    if let Some(matcher_property) = jest_fn_call.matcher()
        && let Some(matcher_name) = matcher_property.name()
    {
        if matcher_name == "toBeCalled" {
            ctx.diagnostic_with_fix(use_to_be_called_with(matcher_property.span), |fixer| {
                fixer.replace(matcher_property.span, "toBeCalledWith")
            });
        } else if matcher_name == "toHaveBeenCalled" {
            ctx.diagnostic_with_fix(use_have_been_called_with(matcher_property.span), |fixer| {
                fixer.replace(matcher_property.span, "toHaveBeenCalledWith")
            });
        }
    }
}
