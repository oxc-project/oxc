use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_new_array_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `new Array(singleArgument)`.").with_help(r"It's not clear whether the argument is meant to be the length of the array or the only element. If the argument is the array's length, consider using `Array.from({ length: n })`. If the argument is the only element, use `[element]`.").with_label(span)
}

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const array = new Array(1);
    /// const array = new Array(42);
    /// const array = new Array(foo);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const array = Array.from({ length: 42 });
    /// const array = [42];
    /// ```
    NoNewArray,
    unicorn,
    correctness,
    pending
);

impl Rule for NoNewArray {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(ident) = &new_expr.callee else {
            return;
        };

        if ident.name != "Array" {
            return;
        }

        if new_expr.arguments.len() != 1 {
            return;
        }

        ctx.diagnostic(no_new_array_diagnostic(new_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const array = Array.from({length: 1})",
        "const array = new Array()",
        "const array = new Array",
        "const array = new Array(1, 2)",
        "const array = Array(1, 2)",
        "const array = Array(1)",
    ];

    let fail = vec![
        "const array = new Array(1)",
        "const zero = 0;
			const array = new Array(zero);",
        "const length = 1;
			const array = new Array(length);",
        "const array = new Array(1.5)",
        r#"const array = new Array(Number("1"))"#,
        r#"const array = new Array("1")"#,
        "const array = new Array(null)",
        r#"const array = new Array(("1"))"#,
        "const array = new Array((0, 1))",
        r#"const foo = []
			new Array("bar").forEach(baz)"#,
        "new Array(0xff)",
        "new Array(Math.PI | foo)",
        "new Array(Math.min(foo, bar))",
        "new Array(Number(foo))",
        "new Array(Number.MAX_SAFE_INTEGER)",
        "new Array(parseInt(foo))",
        "new Array(Number.parseInt(foo))",
        "new Array(+foo)",
        "new Array(-Math.PI)",
        r#"new Array(-"-2")"#,
        "new Array(foo.length)",
        "const foo = 1; new Array(foo + 2)",
        "new Array(foo - 2)",
        "new Array(foo -= 2)",
        "new Array(foo ? 1 : 2)",
        r#"const truthy = "truthy"; new Array(truthy ? 1 : foo)"#,
        r#"const falsy = !"truthy"; new Array(falsy ? foo : 1)"#,
        "new Array((1n, 2))",
        "new Array(Number.NaN)",
        "new Array(NaN)",
        "new Array(foo >>> bar)",
        "new Array(foo >>>= bar)",
        "new Array(++bar.length)",
        "new Array(bar.length++)",
        "new Array(foo = bar.length)",
        r#"new Array("0xff")"#,
        "new Array(Math.NON_EXISTS_PROPERTY)",
        "new Array(Math.NON_EXISTS_METHOD(foo))",
        "new Array(Math[min](foo, bar))",
        "new Array(Number[MAX_SAFE_INTEGER])",
        "new Array(new Number(foo))",
        r#"const foo = 1; new Array(foo + "2")"#,
        "new Array(foo - 2n)",
        "new Array(foo -= 2n)",
        "new Array(foo instanceof 1)",
        "new Array(foo || 1)",
        "new Array(foo ||= 1)",
        "new Array(foo ? 1n : 2)",
        "new Array((1, 2n))",
        "new Array(-foo)",
        "new Array(~foo)",
        "new Array(typeof 1)",
        r#"const truthy = "truthy"; new Array(truthy ? foo : 1)"#,
        r#"const falsy = !"truthy"; new Array(falsy ? 1 : foo)"#,
        "new Array(unknown ? foo : 1)",
        "new Array(unknown ? 1 : foo)",
        "new Array(++foo)",
        "const array = new Array(foo)",
        "const array = new Array(length)",
        "const foo = []
			new Array(bar).forEach(baz)",
        "const foo = []
			new Array(...bar).forEach(baz)",
    ];

    Tester::new(NoNewArray::NAME, NoNewArray::PLUGIN, pass, fail).test_and_snapshot();
}
