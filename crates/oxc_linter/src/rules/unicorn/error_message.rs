use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::BUILT_IN_ERRORS};

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
    /// Enforces providing a `message` when creating built-in `Error` objects to
    /// improve readability and debugging.
    ///
    /// ### Why is this bad?
    ///
    /// Throwing an `Error` without a message, like `throw new Error()`, provides no context
    /// on what went wrong, making debugging harder. A clear error message improves
    /// code clarity and helps developers quickly identify issues.
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
        let (callee, span, args) = match node.kind() {
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

        // If there is `SpreadElement` before message
        if matches!(args.first(), Some(Argument::SpreadElement(_))) {
            return;
        }

        let constructor_name = &callee.name;
        let message_argument_idx = usize::from(callee.name == "AggregateError");
        let message_argument = args.get(message_argument_idx);

        let Some(arg) = message_argument else {
            return ctx.diagnostic(missing_message(constructor_name, span));
        };

        let diagnostic = match arg {
            Argument::ArrayExpression(array_expr) => not_string(array_expr.span),
            Argument::ObjectExpression(object_expr) => not_string(object_expr.span),
            Argument::StringLiteral(lit) if lit.value.is_empty() => empty_message(lit.span),
            Argument::TemplateLiteral(template_lit)
                if template_lit.span.source_text(ctx.source_text()).len() == 2 =>
            {
                empty_message(template_lit.span)
            }
            _ => return,
        };

        ctx.diagnostic(diagnostic);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "throw new Error('error')",
        "throw new TypeError('error')",
        "throw new MyCustomError('error')",
        "throw new MyCustomError()",
        "throw generateError()",
        "throw foo()",
        "throw err",
        "throw 1",
        "const err = TypeError('error');
            throw err;",
        r#"new Error("message", 0, 0)"#,
        "new Error(foo)",
        r"const errors = [];
            if (condition) {
                errors.push('hello');
            }
            if (errors.length) {
                throw new Error(errors.join('\\\\n'));
            }",
        "new Error(...foo)",
        "/* global x */
            const a = x;
            throw x;",
        // TODO: Get this passing.
        // "const Error = function () {};
        //     const err = new Error({
        //         name: 'Unauthorized',
        //     });",
        r#"new AggregateError(errors, "message")"#,
        "new NotAggregateError(errors)",
        "new AggregateError(...foo)",
        r#"new AggregateError(...foo, "")"#,
        "new AggregateError(errors, ...foo)",
        r#"new AggregateError(errors, message, "")"#,
        r#"new AggregateError("", message, "")"#,
        r#"new SuppressedError(error, suppressed, "message")"#,
        "new NotSuppressedError(error, suppressed)",
        "new SuppressedError(...foo)",
        r#"new SuppressedError(...foo, "")"#,
        "new SuppressedError(error, suppressed, ...foo)",
        r#"new SuppressedError(error, suppressed, message, "")"#,
        r#"new SuppressedError("", "", message, "")"#,
    ];

    let fail = vec![
        "throw new Error()",
        "throw Error()",
        "throw new Error('')",
        "throw new Error(``)",
        "const err = new Error();
            throw err;",
        "let err = 1;
            err = new Error();
            throw err;",
        "let err = new Error();
            err = 1;
            throw err;",
        "const foo = new TypeError()",
        "const foo = new SyntaxError()",
        // TODO: Get all of the comments tests here passing.
        // "const errorMessage = Object.freeze({errorMessage: 1}).errorMessage;
        //     throw new Error(errorMessage)",
        "throw new Error([])",
        "throw new Error([foo])",
        // "throw new Error([0][0])",
        "throw new Error({})",
        "throw new Error({foo})",
        // "throw new Error({foo: 0}.foo)",
        // "throw new Error(lineNumber=2)",
        "const error = new RangeError;",
        "throw Object.assign(new Error(), {foo})",
        "new AggregateError(errors)",
        "AggregateError(errors)",
        r#"new AggregateError(errors, "")"#,
        "new AggregateError(errors, ``)",
        r#"new AggregateError(errors, "", extraArgument)"#,
        // "const errorMessage = Object.freeze({errorMessage: 1}).errorMessage;
        //     throw new AggregateError(errors, errorMessage)",
        "new AggregateError(errors, [])",
        "new AggregateError(errors, [foo])",
        // "new AggregateError(errors, [0][0])",
        "new AggregateError(errors, {})",
        "new AggregateError(errors, {foo})",
        // "new AggregateError(errors, {foo: 0}.foo)",
        // "new AggregateError(errors, lineNumber=2)",
        "const error = new AggregateError;",
        // TODO: Update the rule to get these tests working.
        // "new SuppressedError(error, suppressed,)",
        // "new SuppressedError(error,)",
        // "new SuppressedError()",
        // "SuppressedError(error, suppressed,)",
        // "SuppressedError(error,)",
        // "SuppressedError()",
        // r#"new SuppressedError(error, suppressed, "")"#,
        // "new SuppressedError(error, suppressed, ``)",
        // r#"new SuppressedError(error, suppressed, "", options)"#,
        // "const errorMessage = Object.freeze({errorMessage: 1}).errorMessage;
        //     throw new SuppressedError(error, suppressed, errorMessage)",
        // "new SuppressedError(error, suppressed, [])",
        // "new SuppressedError(error, suppressed, [foo])",
        // "new SuppressedError(error, suppressed, [0][0])",
        // "new SuppressedError(error, suppressed, {})",
        // "new SuppressedError(error, suppressed, {foo})",
        // "new SuppressedError(error, suppressed, {foo: 0}.foo)",
        // "new SuppressedError(error, suppressed, lineNumber=2)",
        // "const error = new SuppressedError;",
    ];

    Tester::new(ErrorMessage::NAME, ErrorMessage::PLUGIN, pass, fail).test_and_snapshot();
}
