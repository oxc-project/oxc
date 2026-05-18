use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, ParsedGeneralJestFnCall, PossibleJestNode,
        parse_general_jest_fn_call,
    },
};

pub const DOCUMENTATION: &str = r"### What it does

This rule raises a warning about disabled tests.

### Why is this bad?

Jest has a feature that allows you to temporarily mark tests as disabled. This
feature is often helpful while debugging or to create placeholders for future
tests. Before committing changes we may want to check that all tests are
running.

### Examples

```js
describe.skip('foo', () => {});
it.skip('foo', () => {});
test.skip('foo', () => {});

describe['skip']('bar', () => {});
it['skip']('bar', () => {});
test['skip']('bar', () => {});

xdescribe('foo', () => {});
xit('foo', () => {});
xtest('foo', () => {});

it('bar');
test('bar');

it('foo', () => {
  pending();
});
```
";

fn no_disabled_tests_diagnostic(x1: &'static str, x2: &'static str, span3: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(x1).with_help(x2).with_label(span3)
}

enum Message {
    MissingFunction,
    Pending,
    DisabledSuiteWithSkip,
    DisabledSuiteWithX,
    DisabledTestWithSkip,
    DisabledTestWithX,
}

impl Message {
    pub fn details(&self) -> (&'static str, &'static str) {
        match self {
            Self::MissingFunction => ("Test is missing function argument", "Add function argument"),
            Self::Pending => ("Call to pending()", "Remove pending() call"),
            Self::DisabledSuiteWithSkip => ("Disabled test suite", "Remove the appending `.skip`"),
            Self::DisabledSuiteWithX => ("Disabled test suite", "Remove x prefix"),
            Self::DisabledTestWithSkip => ("Disabled test", "Remove the appending `.skip`"),
            Self::DisabledTestWithX => ("Disabled test", "Remove x prefix"),
        }
    }
}

pub fn run_on_jest_node<'a, 'c>(
    possible_jest_node: &PossibleJestNode<'a, 'c>,
    ctx: &'c LintContext<'a>,
) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        if let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) {
            let ParsedGeneralJestFnCall { kind, members, name, .. } = jest_fn_call;
            // `test('foo')`
            let kind = match kind {
                JestFnKind::Expect
                | JestFnKind::ExpectTypeOf
                | JestFnKind::Unknown
                | JestFnKind::VitestFixture => return,
                JestFnKind::General(kind) => kind,
            };
            if matches!(kind, JestGeneralFnKind::Test)
                && call_expr.arguments.len() < 2
                && members.iter().all(|member| member.is_name_unequal("todo"))
            {
                let (error, help) = Message::MissingFunction.details();
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.span));
                return;
            }

            // the only jest functions that are with "x" are "xdescribe", "xtest", and "xit"
            // `xdescribe('foo', () => {})`
            if name.starts_with('x') {
                let (error, help) = if matches!(kind, JestGeneralFnKind::Describe) {
                    Message::DisabledSuiteWithX.details()
                } else {
                    Message::DisabledTestWithX.details()
                };
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.callee.span()));
                return;
            }

            // `it.skip('foo', function () {})'`
            // `describe.skip('foo', function () {})'`
            if members.iter().any(|member| member.is_name_equal("skip")) {
                let (error, help) = if matches!(kind, JestGeneralFnKind::Describe) {
                    Message::DisabledSuiteWithSkip.details()
                } else {
                    Message::DisabledTestWithSkip.details()
                };
                ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.callee.span()));
            }
        } else if let Expression::Identifier(ident) = &call_expr.callee
            && ident.name.as_str() == "pending"
            && ctx.is_reference_to_global_variable(ident)
        {
            // `describe('foo', function () { pending() })`
            let (error, help) = Message::Pending.details();
            ctx.diagnostic(no_disabled_tests_diagnostic(error, help, call_expr.span));
        }
    }
}
