use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-new-array): Do not use `new Array()`.")]
#[diagnostic(severity(warning), help(r#"If the argument is the array's length, consider using `Array.from({{ length: n }})` instead. If the argument is the only element, use `[element]` instead."#))]
struct NoNewArrayDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNewArray;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `new Array()`.
    ///
    /// ### Why is this bad?
    ///
    /// When using the `Array` constructor with one argument, it's not clear whether the argument is meant to be the length of the array or the only element.
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// const array = new Array(1);
    /// const array = new Array(42);
    /// const array = new Array(foo);
    ///
    /// // good
    /// const array = Array.from({ length: 42 });
    /// const array = [42];
    /// ```
    NoNewArray,
    correctness
);

impl Rule for NoNewArray {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else { return };

        let Expression::Identifier(ident) = &new_expr.callee else { return };

        if ident.name != "Array" {
            return;
        }

        if new_expr.arguments.len() != 1 {
            return;
        }

        ctx.diagnostic(NoNewArrayDiagnostic(new_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"const array = Array.from({length: 1})"#,
        r#"const array = new Array()"#,
        r#"const array = new Array"#,
        r#"const array = new Array(1, 2)"#,
        r#"const array = Array(1, 2)"#,
        r#"const array = Array(1)"#,
    ];

    let fail = vec![
        r#"const array = new Array(1)"#,
        r#"const array = new Array(1.5)"#,
        r#"const array = new Array(Number("1"))"#,
        r#"const array = new Array("1")"#,
        r#"const array = new Array(null)"#,
        r#"const array = new Array(("1"))"#,
        r#"const array = new Array((0, 1))"#,
        r#"new Array(0xff)"#,
        r#"new Array(Math.PI | foo)"#,
        r#"new Array(Math.min(foo, bar))"#,
        r#"new Array(Number(foo))"#,
        r#"new Array(Number.MAX_SAFE_INTEGER)"#,
        r#"new Array(parseInt(foo))"#,
        r#"new Array(Number.parseInt(foo))"#,
        r#"new Array(+foo)"#,
        r#"new Array(-Math.PI)"#,
        r#"new Array(-"-2")"#,
        r#"new Array(foo.length)"#,
        r#"const foo = 1; new Array(foo + 2)"#,
        r#"new Array(foo - 2)"#,
        r#"new Array(foo -= 2)"#,
        r#"new Array(foo ? 1 : 2)"#,
        r#"const truthy = "truthy"; new Array(truthy ? 1 : foo)"#,
        r#"const falsy = !"truthy"; new Array(falsy ? foo : 1)"#,
        r#"new Array((1n, 2))"#,
        r#"new Array(Number.NaN)"#,
        r#"new Array(NaN)"#,
        r#"new Array(foo >>> bar)"#,
        r#"new Array(foo >>>= bar)"#,
        r#"new Array(++bar.length)"#,
        r#"new Array(bar.length++)"#,
        r#"new Array(foo = bar.length)"#,
        r#"new Array("0xff")"#,
        r#"new Array(Math.NON_EXISTS_PROPERTY)"#,
        r#"new Array(Math.NON_EXISTS_METHOD(foo))"#,
        r#"new Array(Math[min](foo, bar))"#,
        r#"new Array(Number[MAX_SAFE_INTEGER])"#,
        r#"new Array(new Number(foo))"#,
        r#"const foo = 1; new Array(foo + "2")"#,
        r#"new Array(foo - 2n)"#,
        r#"new Array(foo -= 2n)"#,
        r#"new Array(foo instanceof 1)"#,
        r#"new Array(foo || 1)"#,
        r#"new Array(foo ||= 1)"#,
        r#"new Array(foo ? 1n : 2)"#,
        r#"new Array((1, 2n))"#,
        r#"new Array(-foo)"#,
        r#"new Array(~foo)"#,
        r#"new Array(typeof 1)"#,
        r#"const truthy = "truthy"; new Array(truthy ? foo : 1)"#,
        r#"const falsy = !"truthy"; new Array(falsy ? 1 : foo)"#,
        r#"new Array(unknown ? foo : 1)"#,
        r#"new Array(unknown ? 1 : foo)"#,
        r#"new Array(++foo)"#,
        r#"const array = new Array(foo)"#,
        r#"const array = new Array(length)"#,
    ];

    Tester::new_without_config(NoNewArray::NAME, pass, fail).test_and_snapshot();
}
