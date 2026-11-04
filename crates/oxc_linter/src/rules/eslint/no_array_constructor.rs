use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_str::static_ident;

use crate::{AstNode, ast_util, context::LintContext, rule::Rule};

fn no_array_constructor_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid calls to the `Array` constructor")
        .with_help("Use array literal notation [] instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayConstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows creating arrays with the `Array` constructor.
    ///
    /// ### Why is this bad?
    ///
    /// Use of the `Array` constructor to construct a new array is generally
    /// discouraged in favor of array literal notation because of the
    /// single-argument pitfall and because the `Array` global may be redefined.
    /// The exception is when the `Array` constructor is used to intentionally
    /// create sparse arrays of a specified size by giving the constructor a
    /// single numeric argument.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let arr = new Array();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let arr = [];
    /// let arr2 = Array.from(iterable);
    /// let arr3 = new Array(9);
    /// ```
    NoArrayConstructor,
    eslint,
    pedantic,
    fix,
    version = "0.0.3",
    short_description = "Disallows creating arrays with the `Array` constructor.",
);

impl Rule for NoArrayConstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (span, callee, arguments, type_parameters, optional) = match node.kind() {
            AstKind::CallExpression(call_expr) => (
                call_expr.span,
                &call_expr.callee,
                &call_expr.arguments,
                &call_expr.type_arguments,
                call_expr.optional,
            ),
            AstKind::NewExpression(new_expr) => (
                new_expr.span,
                &new_expr.callee,
                &new_expr.arguments,
                &new_expr.type_arguments,
                false,
            ),
            _ => return,
        };

        let Expression::Identifier(ident) = &callee else {
            return;
        };

        // Checks if last argument is a spread element such as `Array(...args)` or `Array(1, 2, ...args)`.
        let last_arg_is_spread = arguments.last().is_some_and(Argument::is_spread);
        let arg_len = arguments.len();

        if ident.is_global_reference_name(static_ident!("Array"), ctx.scoping())
            && (arg_len != 1 || last_arg_is_spread)
            && type_parameters.is_none()
            && !optional
        {
            ctx.diagnostic_with_fix(no_array_constructor_diagnostic(span), |fixer| {
                if arg_len <= 2 && last_arg_is_spread {
                    return fixer.noop();
                }
                let replacement = if arg_len == 0 {
                    ""
                } else {
                    ctx.source_range(Span::new(arguments[0].span().start, span.end - 1))
                };
                // A `[` opening an expression statement continues the previous line when
                // that line ends without a semicolon: `foo` then `Array()` on the next
                // line would be rewritten to `[]` directly under `foo`, i.e. a member
                // access on it, which does not even parse.
                let semi = if ast_util::could_be_asi_hazard(node, ctx) { ";" } else { "" };
                fixer.replace(span, format!("{semi}[{replacement}]"))
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new Array(x)",
        "Array(x)",
        "new Array(9)",
        "Array(9)",
        "new foo.Array()",
        "foo.Array()",
        "new Array.foo",
        "Array.foo()",
        "new globalThis.Array",
        "const createArray = Array => new Array()",
        "var Array; new Array;",
        // We do not support globals config in tests:
        // "new Array()", // { "globals": { "Array": "off", }, },
        "new Array(x);",
        "Array(x);",
        "new Array(9);",
        "Array(9);",
        "new foo.Array();",
        "foo.Array();",
        "new Array.foo();",
        "Array.foo();",
        "new Array<Foo>(1, 2, 3);",
        "new Array<Foo>();",
        "Array<Foo>(1, 2, 3);",
        "Array<Foo>();",
        "Array<Foo>(3);",
        "Array?.(x);",
        "Array?.(9);",
        "foo?.Array();",
        "Array?.foo();",
        "foo.Array?.();",
        "Array.foo?.();",
        "Array?.<Foo>(1, 2, 3);",
        "Array?.<Foo>();",
    ];

    let fail = vec![
        "new Array()",
        "new Array",
        "new Array(x, y)",
        "new Array(0, 1, 2)",
        // TODO: Catch optional chaining cases:
        // "const array = Array?.();",
        // TODO: Fix this case:
        // "
        // const array = (Array)(
        //     /* foo */ a,
        //     b = c() // bar
        // );
        // ",
        "const array = Array(...args);",
        "const array = Array(...foo, ...bar);",
        "const array = new Array(...args);",
        "const array = Array(5, ...args);",
        "const array = Array(5, 6, ...args);",
        // "a = new (Array);",
        // "a = new (Array) && (foo);",
        "/*a*/Array()",
        "/*a*/Array()/*b*/",
        "Array/*a*/()",
        "/*a*//*b*/Array/*c*//*d*/()/*e*//*f*/;/*g*//*h*/",
        "Array(/*a*/ /*b*/)",
        "Array(/*a*/ x /*b*/, /*c*/ y /*d*/)",
        "/*a*/Array(/*b*/ x /*c*/, /*d*/ y /*e*/)/*f*/;/*g*/",
        "/*a*/new Array",
        "/*a*/new Array/*b*/",
        "new/*a*/Array",
        "new/*a*//*b*/Array/*c*//*d*/()/*e*//*f*/;/*g*//*h*/",
        "new Array(/*a*/ /*b*/)",
        "new Array(/*a*/ x /*b*/, /*c*/ y /*d*/)",
        "new/*a*/Array(/*b*/ x /*c*/, /*d*/ y /*e*/)/*f*/;/*g*/",
        // "new (Array /* a */);",
        // "(/* a */ Array)(1, 2, 3);",
        // "(Array /* a */)(1, 2, 3);",
        // "(Array) /* a */ (1, 2, 3);",
        // "(/* a */(Array))();",
        // "Array?.(0, 1, 2).forEach(doSomething);",
        "new Array();",
        "Array();",
        "new Array(x, y);",
        "Array(x, y);",
        "new Array(0, 1, 2);",
        "Array(0, 1, 2);",
        // "Array?.(0, 1, 2);",
        // "Array?.(x, y);",
        // "Array /*a*/ ?.();",
        // "Array?./*a*/();",
        r#"
                        (function () {
                            Fn
                            Array() // ";" required
                        }) as Fn
                        Array() // ";" not required
                        "#,
        r#"
                        ({
                            foo() {
                                Object
                                Array() // ";" required
                            }
                        }) as Object
                        Array() // ";" not required
                        "#,
    ];

    let fix = vec![
        ("new Array()", "[]"),
        ("new Array", "[]"),
        ("new Array(x, y)", "[x, y]"),
        ("new Array(0, 1, 2)", "[0, 1, 2]"),
        // TODO: Catch this case and fix it:
        // (
        //     "
        //                         const array = (Array)(
        //                             /* foo */ a,
        //                             b = c() // bar
        //                         );
        //                         ",
        //     "
        //                         const array = [
        //                             /* foo */ a,
        //                             b = c() // bar
        //                         ];
        //                         ",
        // ),
        ("const array = Array(5, 6, ...args);", "const array = [5, 6, ...args];"),
        // TODO: Catch this case:
        // ("a = new (Array);", "a = [];"),
        // ("a = new (Array) && (foo);", "a = [] && (foo);"),
        ("/*a*/Array()", "/*a*/[]"),
        ("/*a*/Array()/*b*/", "/*a*/[]/*b*/"),
        // TODO: Preserve comments around callee:
        // ("Array(/*a*/ /*b*/)", "[/*a*/ /*b*/]"),
        // ("Array(/*a*/ x /*b*/, /*c*/ y /*d*/)", "[/*a*/ x /*b*/, /*c*/ y /*d*/]"),
        // (
        //     "/*a*/Array(/*b*/ x /*c*/, /*d*/ y /*e*/)/*f*/;/*g*/",
        //     "/*a*/[/*b*/ x /*c*/, /*d*/ y /*e*/]/*f*/;/*g*/",
        // ),
        ("/*a*/new Array", "/*a*/[]"),
        ("/*a*/new Array/*b*/", "/*a*/[]/*b*/"),
        // TODO: Preserve comments:
        // ("new Array(/*a*/ /*b*/)", "[/*a*/ /*b*/]"),
        // ("new Array(/*a*/ x /*b*/, /*c*/ y /*d*/)", "[/*a*/ x /*b*/, /*c*/ y /*d*/]"),
        ("new Array();", "[];"),
        ("Array();", "[];"),
        ("new Array(x, y);", "[x, y];"),
        ("Array(x, y);", "[x, y];"),
        ("new Array(0, 1, 2);", "[0, 1, 2];"),
        ("Array(0, 1, 2);", "[0, 1, 2];"),
        // A statement-initial `[` continues the line above when it has no semicolon.
        ("foo\nArray()", "foo\n;[]"),
        ("foo\nArray(1, 2)", "foo\n;[1, 2]"),
        ("foo\nnew Array()", "foo\n;[]"),
        ("foo;\nArray()", "foo;\n[]"),
        ("Array()", "[]"),
        ("if (x) Array()", "if (x) []"),
        // TODO: these no longer produce invalid syntax, but they still differ: after a
        // `}) as Fn` line the fixer now inserts a `;` that the expected output below
        // does not have. `[` after a type position is read as an array type (`Fn[]`),
        // so the semicolon is not obviously wrong - the expectation needs a decision
        // before these can be enabled.
        // (
        //     r#"
        //                 (function () {
        //                     Fn
        //                     Array() // ";" required
        //                 }) as Fn
        //                 Array() // ";" not required
        //                 "#,
        //     r#"
        //                 (function () {
        //                     Fn
        //                     ;[] // ";" required
        //                 }) as Fn
        //                 [] // ";" not required
        //                 "#,
        // ),
        // (
        //     r#"
        //                 ({
        //                     foo() {
        //                         Object
        //                         Array() // ";" required
        //                     }
        //                 }) as Object
        //                 Array() // ";" not required
        //                 "#,
        //     r#"
        //                 ({
        //                     foo() {
        //                         Object
        //                         ;[] // ";" required
        //                     }
        //                 }) as Object
        //                 [] // ";" not required
        //                 "#,
        // ),
    ];

    Tester::new(NoArrayConstructor::NAME, NoArrayConstructor::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
