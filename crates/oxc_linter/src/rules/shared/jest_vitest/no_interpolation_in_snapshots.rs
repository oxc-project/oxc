use oxc_ast::{AstKind, ast::TemplateLiteral};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
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

pub fn run<'a>(node: &AstNode<'a>, template: &TemplateLiteral<'a>, ctx: &LintContext<'a>) {
    if template.expressions.is_empty() {
        return;
    }

    let call_node = ctx.nodes().parent_node(node.id());
    let AstKind::CallExpression(call_expr) = call_node.kind() else { return };
    let Some(possible_jest_node) =
        PossibleJestNode::from_call_expression(call_node, call_expr, ctx)
    else {
        return;
    };
    let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, &possible_jest_node, ctx) else {
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

    ctx.diagnostic(no_interpolation_in_snapshots_diagnostic(template.span));
}
