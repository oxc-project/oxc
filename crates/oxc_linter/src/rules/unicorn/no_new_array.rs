use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_new_array_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `new Array(singleArgument)`.")
        .with_help(r"It's not clear whether the argument is meant to be the length of the array or the only element. If the argument is the array's length, consider using `Array.from({ length: n })`. If the argument is the only element, use `[element]`.").with_label(span)
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
    dangerous_suggestion,
    version = "0.0.16",
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

        ctx.diagnostics_with_multiple_fixes(
            no_new_array_diagnostic(new_expr.span),
            (FixKind::DangerousSuggestion, |fixer| {
                let arg = &new_expr.arguments[0];
                let arg_span = arg.span();

                // Uncaught SyntaxError: expected expression, got '...'
                if let Argument::SpreadElement(_) = arg {
                    return fixer.noop();
                }

                fix_numeric_literal(fixer, new_expr.span, arg_span)
            }),
            (FixKind::DangerousSuggestion, |fixer| {
                let arg = &new_expr.arguments[0];
                let arg_span = arg.span();

                fix_to_short_array(fixer, node, ctx, new_expr.span, arg_span)
            }),
        );
    }
}

fn fix_to_short_array(
    fixer: RuleFixer,
    node: &AstNode,
    ctx: &LintContext,
    new_span: Span,
    arg_span: Span,
) -> RuleFix {
    let needs_semi = ast_util::could_be_asi_hazard(node, ctx);
    let fixer = fixer.for_multifix();
    let mut rule_fixes = fixer.new_fix_with_capacity(2).with_message("Replace with [argument]");
    rule_fixes.push(
        fixer.replace(
            Span::new(new_span.start, arg_span.start),
            if needs_semi { ";[" } else { "[" },
        ),
    );
    rule_fixes.push(fixer.replace(Span::new(arg_span.end, new_span.end), "]"));
    rule_fixes
}

fn fix_numeric_literal(fixer: RuleFixer, new_span: Span, arg_span: Span) -> RuleFix {
    let fixer = fixer.for_multifix();
    let mut rule_fixes = fixer
        .new_fix_with_capacity(2)
        .with_message("Replace with Array.from({ length: argument })");
    rule_fixes.push(fixer.replace(
        Span::new(new_span.start, arg_span.start),
        // the original rule checks if the argument is named "length" and shortens the code
        // because we are not looking into the (resolved) arguments we always append "length: " to be safe.
        "Array.from({length: ",
    ));
    rule_fixes.push(fixer.replace(Span::new(arg_span.end, new_span.end), "})"));
    rule_fixes
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

    // the fixes are separated into 3 groups, these groups are matched from the original rule,
    // but does not necessarily reflect the actual implementation of the real fix behavior, because we are not looking into the (resolved) arguments.
    let fix_only_to_number: Vec<(&str, (&str, &str))> = vec![
        ("new Array((42 as number))", ("Array.from({length: (42 as number)})", "[(42 as number)]")),
        (
            "const array = new Array(1)",
            ("const array = Array.from({length: 1})", "const array = [1]"),
        ),
        (
            "const zero = 0; const array = new Array(zero);",
            (
                "const zero = 0; const array = Array.from({length: zero});",
                "const zero = 0; const array = [zero];",
            ),
        ),
        (
            "const length = 1; const array = new Array(length);",
            (
                "const length = 1; const array = Array.from({length: length});",
                "const length = 1; const array = [length];",
            ), // the original rules just appends `{length}`, we do not resolve the arguments
        ),
        (
            "const array = new Array(1.5)",
            ("const array = Array.from({length: 1.5})", "const array = [1.5]"),
        ),
        (
            r#"const array = new Array(Number("1"))"#,
            (
                r#"const array = Array.from({length: Number("1")})"#,
                r#"const array = [Number("1")]"#,
            ),
        ),
        (
            "const array = new Array((0, 1))",
            ("const array = Array.from({length: (0, 1)})", "const array = [(0, 1)]"),
        ),
        ("new Array(0xff)", ("Array.from({length: 0xff})", "[0xff]")),
        ("new Array(Math.PI | foo)", ("Array.from({length: Math.PI | foo})", "[Math.PI | foo]")),
        (
            "new Array(Math.min(foo, bar))",
            ("Array.from({length: Math.min(foo, bar)})", "[Math.min(foo, bar)]"),
        ),
        ("new Array(Number(foo))", ("Array.from({length: Number(foo)})", "[Number(foo)]")),
        (
            "new Array(Number.MAX_SAFE_INTEGER)",
            ("Array.from({length: Number.MAX_SAFE_INTEGER})", "[Number.MAX_SAFE_INTEGER]"),
        ),
        ("new Array(parseInt(foo))", ("Array.from({length: parseInt(foo)})", "[parseInt(foo)]")),
        (
            "new Array(Number.parseInt(foo))",
            ("Array.from({length: Number.parseInt(foo)})", "[Number.parseInt(foo)]"),
        ),
        ("new Array(+foo)", ("Array.from({length: +foo})", "[+foo]")),
        ("new Array(-Math.PI)", ("Array.from({length: -Math.PI})", "[-Math.PI]")),
        (r#"new Array(-"-2")"#, (r#"Array.from({length: -"-2"})"#, r#"[-"-2"]"#)),
        ("new Array(foo.length)", ("Array.from({length: foo.length})", "[foo.length]")),
        (
            "const foo = 1; new Array(foo + 2)",
            ("const foo = 1; Array.from({length: foo + 2})", "const foo = 1; [foo + 2]"),
        ),
        ("new Array(foo - 2)", ("Array.from({length: foo - 2})", "[foo - 2]")),
        ("new Array(foo -= 2)", ("Array.from({length: foo -= 2})", "[foo -= 2]")),
        ("new Array(foo ? 1 : 2)", ("Array.from({length: foo ? 1 : 2})", "[foo ? 1 : 2]")),
        (
            r#"const truthy = "truthy"; new Array(truthy ? 1 : foo)"#,
            (
                r#"const truthy = "truthy"; Array.from({length: truthy ? 1 : foo})"#,
                r#"const truthy = "truthy"; [truthy ? 1 : foo]"#,
            ),
        ),
        (
            r#"const falsy = !"truthy"; new Array(falsy ? foo : 1)"#,
            (
                r#"const falsy = !"truthy"; Array.from({length: falsy ? foo : 1})"#,
                r#"const falsy = !"truthy"; [falsy ? foo : 1]"#,
            ),
        ),
        ("new Array((1n, 2))", ("Array.from({length: (1n, 2)})", "[(1n, 2)]")),
        ("new Array(Number.NaN)", ("Array.from({length: Number.NaN})", "[Number.NaN]")),
        ("new Array(NaN)", ("Array.from({length: NaN})", "[NaN]")),
        ("new Array(foo >>> bar)", ("Array.from({length: foo >>> bar})", "[foo >>> bar]")),
        ("new Array(foo >>>= bar)", ("Array.from({length: foo >>>= bar})", "[foo >>>= bar]")),
        ("new Array(++bar.length)", ("Array.from({length: ++bar.length})", "[++bar.length]")),
        ("new Array(bar.length++)", ("Array.from({length: bar.length++})", "[bar.length++]")),
        (
            "new Array(foo = bar.length)",
            ("Array.from({length: foo = bar.length})", "[foo = bar.length]"),
        ),
    ];

    let fix_only_short_array = vec![
        (
            r#"const array = new Array("1")"#,
            (r#"const array = Array.from({length: "1"})"#, r#"const array = ["1"]"#),
        ),
        (
            "const array = new Array(null)",
            ("const array = Array.from({length: null})", "const array = [null]"),
        ),
        (
            r#"const array = new Array(("1"))"#,
            (r#"const array = Array.from({length: ("1")})"#, r#"const array = [("1")]"#),
        ),
        (
            r#"const foo = []
        new Array("bar").forEach(baz)"#,
            (
                r#"const foo = []
        Array.from({length: "bar"}).forEach(baz)"#,
                r#"const foo = []
        ;["bar"].forEach(baz)"#,
            ),
        ),
        (r#"new Array("0xff")"#, (r#"Array.from({length: "0xff"})"#, r#"["0xff"]"#)),
        (
            "new Array(Math.NON_EXISTS_PROPERTY)",
            ("Array.from({length: Math.NON_EXISTS_PROPERTY})", "[Math.NON_EXISTS_PROPERTY]"),
        ),
        (
            r#"const foo = 1; new Array(foo + "2")"#,
            (r#"const foo = 1; Array.from({length: foo + "2"})"#, r#"const foo = 1; [foo + "2"]"#),
        ),
        ("new Array((1, 2n))", ("Array.from({length: (1, 2n)})", "[(1, 2n)]")),
        // spread arguments cannot be a numeric literal
        (
            "const array = new Array(...[foo])",
            ("const array = [...[foo]]", "const array = new Array(...[foo])"),
        ),
        (
            "const array = new Array(...foo)",
            ("const array = [...foo]", "const array = new Array(...foo)"),
        ),
        (
            "const array = new Array(...[...foo])",
            ("const array = [...[...foo]]", "const array = new Array(...[...foo])"),
        ),
        (
            "const array = new Array(...[1])",
            ("const array = [...[1]]", "const array = new Array(...[1])"),
        ),
        (
            r#"const array = new Array(...["1"])"#,
            (r#"const array = [...["1"]]"#, r#"const array = new Array(...["1"])"#),
        ),
        (
            r#"const array = new Array(...[1, "1"])"#,
            (r#"const array = [...[1, "1"]]"#, r#"const array = new Array(...[1, "1"])"#),
        ),
        (
            "const foo = []
        new Array(...bar).forEach(baz)",
            (
                "const foo = []
        ;[...bar].forEach(baz)",
                "const foo = []
        new Array(...bar).forEach(baz)",
            ),
        ),
    ];

    let fix_double = vec![
        (
            "new Array(Math.NON_EXISTS_METHOD(foo))",
            ("Array.from({length: Math.NON_EXISTS_METHOD(foo)})", "[Math.NON_EXISTS_METHOD(foo)]"),
        ),
        (
            "new Array(Math[min](foo, bar))",
            ("Array.from({length: Math[min](foo, bar)})", "[Math[min](foo, bar)]"),
        ),
        (
            "new Array(Number[MAX_SAFE_INTEGER])",
            ("Array.from({length: Number[MAX_SAFE_INTEGER]})", "[Number[MAX_SAFE_INTEGER]]"),
        ),
        (
            "new Array(new Number(foo))",
            ("Array.from({length: new Number(foo)})", "[new Number(foo)]"),
        ),
        ("new Array(foo - 2n)", ("Array.from({length: foo - 2n})", "[foo - 2n]")),
        ("new Array(foo -= 2n)", ("Array.from({length: foo -= 2n})", "[foo -= 2n]")),
        (
            "new Array(foo instanceof 1)",
            ("Array.from({length: foo instanceof 1})", "[foo instanceof 1]"),
        ),
        ("new Array(foo || 1)", ("Array.from({length: foo || 1})", "[foo || 1]")),
        ("new Array(foo ||= 1)", ("Array.from({length: foo ||= 1})", "[foo ||= 1]")),
        ("new Array(foo ? 1n : 2)", ("Array.from({length: foo ? 1n : 2})", "[foo ? 1n : 2]")),
        ("new Array(-foo)", ("Array.from({length: -foo})", "[-foo]")),
        ("new Array(~foo)", ("Array.from({length: ~foo})", "[~foo]")),
        ("new Array(typeof 1)", ("Array.from({length: typeof 1})", "[typeof 1]")),
        (
            r#"const truthy = "truthy"; new Array(truthy ? foo : 1)"#,
            (
                r#"const truthy = "truthy"; Array.from({length: truthy ? foo : 1})"#,
                r#"const truthy = "truthy"; [truthy ? foo : 1]"#,
            ),
        ),
        (
            r#"const falsy = !"truthy"; new Array(falsy ? 1 : foo)"#,
            (
                r#"const falsy = !"truthy"; Array.from({length: falsy ? 1 : foo})"#,
                r#"const falsy = !"truthy"; [falsy ? 1 : foo]"#,
            ),
        ),
        (
            "new Array(unknown ? foo : 1)",
            ("Array.from({length: unknown ? foo : 1})", "[unknown ? foo : 1]"),
        ),
        (
            "new Array(unknown ? 1 : foo)",
            ("Array.from({length: unknown ? 1 : foo})", "[unknown ? 1 : foo]"),
        ),
        ("new Array(++foo)", ("Array.from({length: ++foo})", "[++foo]")),
        (
            "const array = new Array(foo)",
            ("const array = Array.from({length: foo})", "const array = [foo]"),
        ),
        (
            "const array = new Array(length)",
            ("const array = Array.from({length: length})", "const array = [length]"),
            // the original rules just appends `{length}`, we do not resolve the arguments
        ),
        (
            "const foo = []
        new Array(bar).forEach(baz)",
            (
                "const foo = []
        Array.from({length: bar}).forEach(baz)",
                "const foo = []
        ;[bar].forEach(baz)",
            ),
        ),
    ];

    Tester::new(NoNewArray::NAME, NoNewArray::PLUGIN, pass, fail)
        .expect_fix(fix_only_short_array)
        .test_and_snapshot();

    Tester::new::<&str>(NoNewArray::NAME, NoNewArray::PLUGIN, vec![], vec![])
        .expect_fix(fix_only_to_number)
        .test();

    Tester::new::<&str>(NoNewArray::NAME, NoNewArray::PLUGIN, vec![], vec![])
        .expect_fix(fix_double)
        .test();
}
