use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-new-buffer): Use `Buffer.alloc()` or `Buffer.from()` instead of the deprecated `new Buffer()` constructor.")]
#[diagnostic(
    severity(warning),
    help("`new Buffer()` is deprecated, use `Buffer.alloc()` or `Buffer.from()` instead.")
)]
struct NoNewBufferDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// // Bad
    /// const buffer = new Buffer(10);
    ///
    /// // Good
    /// const buffer = Buffer.alloc(10);
    /// ```
    NoNewBuffer,
    pedantic
);

impl Rule for NoNewBuffer {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else { return };

        let Expression::Identifier(ident) = &new_expr.callee.without_parenthesized() else {
            return;
        };
        if ident.name != "Buffer" {
            return;
        }
        ctx.diagnostic(NoNewBufferDiagnostic(ident.span));
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

    Tester::new_without_config(NoNewBuffer::NAME, pass, fail).test_and_snapshot();
}
