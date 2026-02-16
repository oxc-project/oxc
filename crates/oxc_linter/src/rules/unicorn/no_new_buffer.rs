use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_new_buffer_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `Buffer.alloc()` or `Buffer.from()` instead of the deprecated `new Buffer()` constructor.")
        .with_help("`new Buffer()` is deprecated, use `Buffer.alloc()` or `Buffer.from()` instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewBuffer;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the deprecated `new Buffer()` constructor.
    ///
    /// ### Why is this bad?
    ///
    /// Enforces the use of [Buffer.from](https://nodejs.org/api/buffer.html#static-method-bufferfromarray) and [Buffer.alloc()](https://nodejs.org/api/buffer.html#static-method-bufferallocsize-fill-encoding) instead of [new Buffer()](https://nodejs.org/api/buffer.html#new-bufferarray), which has been deprecated since Node.js 4.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const buffer = new Buffer(10);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const buffer = Buffer.alloc(10);
    /// ```
    NoNewBuffer,
    unicorn,
    pedantic,
    suggestion
);

impl Rule for NoNewBuffer {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = &new_expr.callee.without_parentheses() else {
            return;
        };
        if ident.name != "Buffer" || !ident.is_global_reference(ctx.scoping()) {
            return;
        }

        // Determine which method to use based on argument type
        let method = determine_buffer_method(new_expr);
        let expr_span = new_expr.span;

        ctx.diagnostic_with_suggestion(no_new_buffer_diagnostic(ident.span), |fixer| {
            let Some(method) = method else {
                return fixer.noop();
            };

            // Build arguments string
            let args_text = new_expr
                .arguments
                .iter()
                .map(|arg| ctx.source_range(arg.span()))
                .collect::<Vec<_>>()
                .join(", ");

            let replacement = format!("Buffer.{method}({args_text})");
            fixer.replace(expr_span, replacement)
        });
    }
}

/// Determines which Buffer method to use based on the first argument.
/// Returns `Some("alloc")` for numeric arguments, `Some("from")` for array/string arguments,
/// or `None` if the type can't be determined (unsafe to fix).
fn determine_buffer_method(new_expr: &oxc_ast::ast::NewExpression) -> Option<&'static str> {
    // Handle spread arguments - unsafe to fix
    if new_expr.arguments.iter().any(Argument::is_spread) {
        return None;
    }

    let first_arg = new_expr.arguments.first()?.as_expression()?;
    let first_arg = first_arg.without_parentheses();

    match first_arg {
        // Numeric literals → Buffer.alloc
        Expression::NumericLiteral(_) => Some("alloc"),
        // String/template literals → Buffer.from
        Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::ArrayExpression(_) => Some("from"),
        // For other expressions, we can't safely determine the type
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const buffer = Buffer",
        r"const buffer = new NotBuffer(1)",
        r"const buffer = Buffer.from('buf')",
        r"const buffer = Buffer.from('7468697320697320612074c3a97374', 'hex')",
        r"const buffer = Buffer.from([0x62, 0x75, 0x66, 0x66, 0x65, 0x72])",
        r"const buffer = Buffer.alloc(10)",
        r"const Buffer = function () {}; new Buffer(10);",
    ];

    let fail = vec![
        r"const buffer = new Buffer([0x62, 0x75, 0x66, 0x66, 0x65, 0x72])",
        r"const buffer = new Buffer([0x62, bar])",
        r"const buffer = new Buffer(10);",
        r"new Buffer(foo.length)",
        r"new Buffer(Math.min(foo, bar))",
        r#"const buffer = new Buffer("string");"#,
        r#"const buffer = new Buffer("7468697320697320612074c3a97374", "hex")"#,
        r"const buffer = new Buffer(`${unknown}`)",
        r"const buffer = new (Buffer)(unknown)",
        r"const buffer = new Buffer(unknown, 2)",
        r"const buffer = new Buffer(...unknown)",
        r"const buffer = new /* comment */ Buffer()",
        r"const buffer = new /* comment */ Buffer",
        r"new Buffer(input, encoding);",
    ];

    let fix = vec![
        // Numeric argument → Buffer.alloc
        (r"const buffer = new Buffer(10);", r"const buffer = Buffer.alloc(10);"),
        // String argument → Buffer.from
        (r#"const buffer = new Buffer("string");"#, r#"const buffer = Buffer.from("string");"#),
        (
            r#"const buffer = new Buffer("7468697320697320612074c3a97374", "hex")"#,
            r#"const buffer = Buffer.from("7468697320697320612074c3a97374", "hex")"#,
        ),
        // Array argument → Buffer.from
        (
            r"const buffer = new Buffer([0x62, 0x75, 0x66, 0x66, 0x65, 0x72])",
            r"const buffer = Buffer.from([0x62, 0x75, 0x66, 0x66, 0x65, 0x72])",
        ),
        (r"const buffer = new Buffer([0x62, bar])", r"const buffer = Buffer.from([0x62, bar])"),
        // Template literal → Buffer.from
        (r"const buffer = new Buffer(`${unknown}`)", r"const buffer = Buffer.from(`${unknown}`)"),
    ];

    Tester::new(NoNewBuffer::NAME, NoNewBuffer::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
