use oxc_ast::{AstKind, ast::Argument};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn no_interpolation_in_snapshots_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use string interpolation inside of snapshots")
        .with_help("Remove string interpolation from snapshots")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Prevents the use of string interpolations in snapshots.

### Why is this bad?

Interpolation prevents snapshots from being updated. Instead, properties should
be overloaded with a matcher by using
[property matchers](https://jestjs.io/docs/en/snapshot-testing#property-matchers).

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(something).toMatchInlineSnapshot(
  `Object {
    property: ${interpolated}
  }`,
);

expect(something).toMatchInlineSnapshot(
  { other: expect.any(Number) },
  `Object {
    other: Any<Number>,
    property: ${interpolated}
  }`,
);

expect(errorThrowingFunction).toThrowErrorMatchingInlineSnapshot(
  `${interpolated}`,
);
```
";

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    let Some(matcher) = jest_fn_call.matcher() else {
        return;
    };

    if matcher.is_name_unequal("toMatchInlineSnapshot")
        && matcher.is_name_unequal("toThrowErrorMatchingInlineSnapshot")
    {
        return;
    }

    // Check all since the optional 'propertyMatchers' argument might be present
    // `.toMatchInlineSnapshot(propertyMatchers?, inlineSnapshot)`
    for arg in jest_fn_call.args {
        if let Argument::TemplateLiteral(template_lit) = arg
            && !template_lit.expressions.is_empty()
        {
            ctx.diagnostic(no_interpolation_in_snapshots_diagnostic(template_lit.span));
        }
    }
}
