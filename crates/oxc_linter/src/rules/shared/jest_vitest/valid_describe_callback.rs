use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, FunctionBody, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, parse_general_jest_fn_call},
};

fn valid_describe_callback_diagnostic(
    x1: &'static str,
    x2: &'static str,
    span3: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(x1).with_help(x2).with_label(span3)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule validates that the second parameter of a `describe()` function is a
callback function. This callback function:
- should not be
[async](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function)
- should not contain any parameters
- should not contain any `return` statements

### Why is this bad?

Using an improper `describe()` callback function can lead to unexpected test
errors.

### Examples

Examples of **incorrect** code for this rule:
```javascript
// Async callback functions are not allowed
describe('myFunction()', async () => {
  // ...
});

// Callback function parameters are not allowed
describe('myFunction()', done => {
  // ...
});

// Returning a value from a describe block is not allowed
describe('myFunction', () =>
  it('returns a truthy value', () => {
    expect(myFunction()).toBeTruthy();
}));
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
    if !matches!(jest_fn_call.kind, JestFnKind::General(JestGeneralFnKind::Describe)) {
        return;
    }

    let arg_len = call_expr.arguments.len();

    // Handle describe.todo("runPrettierFormat")
    if ctx.frameworks().is_vitest()
        && arg_len == 1
        && let Some(member_expr) = call_expr.callee.as_member_expression()
    {
        let Some(property_name) = member_expr.static_property_name() else {
            return;
        };
        if property_name == "todo" {
            return;
        }
    }

    if arg_len == 0 {
        diagnostic(ctx, call_expr.span, Message::NameAndCallback);
        return;
    }

    if arg_len == 1 {
        // For better error notice, we locate it to arguments[0]
        diagnostic(ctx, call_expr.arguments[0].span(), Message::NameAndCallback);
        return;
    }

    match &call_expr.arguments[1] {
        Argument::FunctionExpression(fn_expr) => {
            if fn_expr.r#async {
                diagnostic(ctx, fn_expr.span, Message::NoAsyncDescribeCallback);
            }
            let no_each_fields =
                jest_fn_call.members.iter().all(|member| member.is_name_unequal("each"));
            if no_each_fields && fn_expr.params.parameters_count() > 0 {
                diagnostic(ctx, fn_expr.span, Message::UnexpectedDescribeArgument);
            }

            let Some(body) = &fn_expr.body else {
                return;
            };
            if let Some(span) = find_first_return_stmt_span(body) {
                diagnostic(ctx, span, Message::UnexpectedReturnInDescribe);
            }
        }
        Argument::ArrowFunctionExpression(arrow_expr) => {
            if arrow_expr.r#async {
                diagnostic(ctx, arrow_expr.span, Message::NoAsyncDescribeCallback);
            }
            let no_each_fields =
                jest_fn_call.members.iter().all(|member| member.is_name_unequal("each"));
            if no_each_fields && arrow_expr.params.parameters_count() > 0 {
                diagnostic(ctx, arrow_expr.span, Message::UnexpectedDescribeArgument);
            }

            if arrow_expr.expression && !arrow_expr.body.statements.is_empty() {
                let stmt = &arrow_expr.body.statements[0];
                let Statement::ExpressionStatement(expr_stmt) = stmt else {
                    return;
                };
                if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                    diagnostic(ctx, call_expr.span, Message::UnexpectedReturnInDescribe);
                }
            }

            if let Some(span) = find_first_return_stmt_span(&arrow_expr.body) {
                diagnostic(ctx, span, Message::UnexpectedReturnInDescribe);
            }
        }
        callback => diagnostic(ctx, callback.span(), Message::SecondArgumentMustBeFunction),
    }
}

fn find_first_return_stmt_span(function_body: &FunctionBody) -> Option<Span> {
    function_body.statements.iter().find_map(|stmt| {
        if let Statement::ReturnStatement(return_stmt) = stmt {
            Some(return_stmt.span)
        } else {
            None
        }
    })
}

fn diagnostic(ctx: &LintContext, span: Span, message: Message) {
    let (error, help) = message.details();
    ctx.diagnostic(valid_describe_callback_diagnostic(error, help, span));
}

#[derive(Clone, Copy)]
enum Message {
    NameAndCallback,
    SecondArgumentMustBeFunction,
    NoAsyncDescribeCallback,
    UnexpectedDescribeArgument,
    UnexpectedReturnInDescribe,
}

impl Message {
    pub fn details(self) -> (&'static str, &'static str) {
        match self {
            Self::NameAndCallback => (
                "Describe requires name and callback arguments",
                "Add name as first argument and callback as second argument",
            ),
            Self::SecondArgumentMustBeFunction => {
                ("Second argument must be a function", "Replace second argument with a function")
            }
            Self::NoAsyncDescribeCallback => {
                ("No async describe callback", "Remove `async` keyword")
            }
            Self::UnexpectedDescribeArgument => (
                "Unexpected argument(s) in describe callback",
                "Remove argument(s) of describe callback",
            ),
            Self::UnexpectedReturnInDescribe => (
                "Unexpected return statement in describe callback",
                "Remove return statement in your describe callback",
            ),
        }
    }
}
