use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, MemberExpressionElement, ParsedGeneralJestFnCall,
        PossibleJestNode, parse_general_jest_fn_call,
    },
};

fn no_focused_tests_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected focused test.")
        .with_help("Remove focus from test.")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule reminds you to remove `.only` from your tests by raising a warning
whenever you are using the exclusivity feature.

### Why is this bad?

Jest has a feature that allows you to focus tests by appending `.only` or
prepending `f` to a test-suite or a test-case. This feature is really helpful to
debug a failing test, so you don’t have to execute all of your tests. After you
have fixed your test and before committing the changes you have to remove
`.only` to ensure all tests are executed on your build system.

### Examples

Examples of **incorrect** code for this rule:
```javascript
describe.only('foo', () => {});
it.only('foo', () => {});
describe['only']('bar', () => {});
it['only']('bar', () => {});
test.only('foo', () => {});
test['only']('bar', () => {});
fdescribe('foo', () => {});
fit('foo', () => {});
fit.each`
table
`();
```
";

pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) else {
        return;
    };
    let ParsedGeneralJestFnCall { kind, members, name, .. } = jest_fn_call;
    if !matches!(kind, JestFnKind::General(JestGeneralFnKind::Describe | JestGeneralFnKind::Test)) {
        return;
    }

    if name.starts_with('f') {
        ctx.diagnostic_with_fix(
            no_focused_tests_diagnostic(Span::sized(
                call_expr.span.start,
                u32::try_from(name.len()).unwrap_or(1),
            )),
            |fixer| fixer.delete_range(Span::sized(call_expr.span.start, 1)),
        );

        return;
    }

    let only_node = members.iter().find(|member| member.is_name_equal("only"));
    if let Some(only_node) = only_node {
        ctx.diagnostic_with_fix(no_focused_tests_diagnostic(only_node.span), |fixer| {
            let mut span = only_node.span.expand_left(1);
            if !matches!(only_node.element, MemberExpressionElement::IdentName(_)) {
                span = span.expand_right(1);
            }
            fixer.delete_range(span)
        });
    }
}
