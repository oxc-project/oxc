use oxc_ast::{
    ast::{Argument, CallExpression, Expression, NewExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
pub enum ErrorMessageDiagnostic {
    #[error("eslint-plugin-unicorn(error-message): Pass a message to the {0:1} constructor.")]
    MissingMessage(Atom, #[label] Span),
    #[error("eslint-plugin-unicorn(error-message): Error message should not be an empty string.")]
    EmptyMessage(#[label] Span),
    #[error("eslint-plugin-unicorn(error-message): Error message should be a string.")]
    NotString(#[label] Span),
}

#[derive(Default, Debug, Clone)]
pub struct ErrorMessage;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a `message` value to be passed in when creating an instance of a built-in `Error` object, which leads to more readable and debuggable code.
    ///
    /// ### Example
    /// ```javascript
    /// // Fail
    /// throw Error()
    /// throw new TypeError()
    ///
    /// // Pass
    /// throw new Error('Unexpected token')
    /// throw new TypeError('Number expected')
    ///
    ///
    /// ```
    ErrorMessage,
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
        if args.iter().enumerate().any(|(i, arg)| {
            i <= message_argument_idx
                && match arg {
                    Argument::Expression(_) => false,
                    Argument::SpreadElement(_) => true,
                }
        }) {
            return;
        }

        let message_argument = args.get(message_argument_idx);

        let arg = match message_argument {
            Some(v) => v,
            None => {
                return ctx.diagnostic(ErrorMessageDiagnostic::MissingMessage(
                    constructor_name.clone(),
                    span,
                ))
            }
        };

        let arg = match arg {
            Argument::Expression(v) => v,
            Argument::SpreadElement(_) => {
                return;
            }
        };

        match arg {
            Expression::StringLiteral(lit) => {
                if lit.value.is_empty() {
                    ctx.diagnostic(ErrorMessageDiagnostic::EmptyMessage(lit.span));
                }
            }
            Expression::TemplateLiteral(template_lit) => {
                if template_lit.span.source_text(ctx.source_text()).len() == 2 {
                    ctx.diagnostic(ErrorMessageDiagnostic::EmptyMessage(template_lit.span));
                }
            }
            Expression::ObjectExpression(object_expr) => {
                ctx.diagnostic(ErrorMessageDiagnostic::NotString(object_expr.span));
            }
            Expression::ArrayExpression(array_expr) => {
                ctx.diagnostic(ErrorMessageDiagnostic::NotString(array_expr.span));
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

    Tester::new(ErrorMessage::NAME, pass, fail).test_and_snapshot();
}
