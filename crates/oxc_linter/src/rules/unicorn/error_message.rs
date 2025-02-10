use oxc_ast::{
    ast::{Argument, CallExpression, Expression, NewExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn missing_message(ctor_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Pass a message to the {ctor_name:1} constructor."))
        .with_label(span)
}

fn empty_message(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Error message should not be an empty string.").with_label(span)
}

fn not_string(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Error message should be a string.").with_label(span)
}

#[derive(Default, Debug, Clone)]
pub struct ErrorMessage;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a `message` value to be passed in when creating an instance of a built-in `Error` object, which leads to more readable and debuggable code.
    ///
    /// ### Why is this bad?
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// throw Error()
    /// throw new TypeError()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// throw new Error('Unexpected token')
    /// throw new TypeError('Number expected')
    /// ```
    ErrorMessage,
    unicorn,
    style
);

impl Rule for ErrorMessage {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (callee, span, args) = match &node.kind() {
            AstKind::NewExpression(NewExpression {
                callee: Expression::Identifier(id),
                span,
                arguments,
                ..
            })
            | AstKind::CallExpression(CallExpression {
                callee: Expression::Identifier(id),
                span,
                arguments,
                ..
            }) => (id, *span, arguments),
            _ => return,
        };

        if !BUILT_IN_ERRORS.contains(&callee.name.as_str()) {
            return;
        }

        let constructor_name = &callee.name;
        let message_argument_idx = usize::from(constructor_name.as_str() == "AggregateError");

        // If message is `SpreadElement` or there is `SpreadElement` before message
        if args
            .iter()
            .enumerate()
            .any(|(i, arg)| i <= message_argument_idx && matches!(arg, Argument::SpreadElement(_)))
        {
            return;
        }

        let message_argument = args.get(message_argument_idx);

        let Some(arg) = message_argument else {
            return ctx.diagnostic(missing_message(constructor_name.as_str(), span));
        };

        match arg {
            Argument::StringLiteral(lit) => {
                if lit.value.is_empty() {
                    ctx.diagnostic(empty_message(lit.span));
                }
            }
            Argument::TemplateLiteral(template_lit) => {
                if template_lit.span.source_text(ctx.source_text()).len() == 2 {
                    ctx.diagnostic(empty_message(template_lit.span));
                }
            }
            Argument::ObjectExpression(object_expr) => {
                ctx.diagnostic(not_string(object_expr.span));
            }
            Argument::ArrayExpression(array_expr) => {
                ctx.diagnostic(not_string(array_expr.span));
            }
            _ => {}
        }
    }
}

// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error#Error_types
const BUILT_IN_ERRORS: &[&str] = &[
    "Error",
    "EvalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError",
    "InternalError",
    "AggregateError",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("throw new Error('error')", None),
        ("throw new TypeError('error')", None),
        ("throw new MyCustomError('error')", None),
        ("throw new MyCustomError()", None),
        ("throw generateError()", None),
        ("throw foo()", None),
        ("throw err", None),
        ("throw 1", None),
        ("new Error(\"message\", 0, 0)", None),
        ("new Error(foo)", None),
        ("new Error(...foo)", None),
        ("new AggregateError(errors, \"message\")", None),
        ("new NotAggregateError(errors)", None),
        ("new AggregateError(...foo)", None),
        ("new AggregateError(...foo, \"\")", None),
        ("new AggregateError(errors, ...foo)", None),
        ("new AggregateError(errors, message, \"\")", None),
        ("new AggregateError(\"\", message, \"\")", None),
    ];

    let fail = vec![
        ("throw new Error()", None),
        ("throw Error()", None),
        ("throw new Error('')", None),
        ("throw new Error(``)", None),
        ("const foo = new TypeError()", None),
        ("const foo = new SyntaxError()", None),
        ("throw new Error([])", None),
        ("throw new Error([foo])", None),
        ("throw new Error({})", None),
        ("throw new Error({foo})", None),
        ("const error = new RangeError;", None),
        ("new AggregateError(errors)", None),
        ("AggregateError(errors)", None),
        ("new AggregateError(errors, \"\")", None),
        ("new AggregateError(errors, ``)", None),
        ("new AggregateError(errors, \"\", extraArgument)", None),
        ("new AggregateError(errors, [])", None),
        ("new AggregateError(errors, [foo])", None),
        ("new AggregateError(errors, {})", None),
        ("new AggregateError(errors, {foo})", None),
        ("const error = new AggregateError;", None),
    ];

    Tester::new(ErrorMessage::NAME, ErrorMessage::PLUGIN, pass, fail).test_and_snapshot();
}
