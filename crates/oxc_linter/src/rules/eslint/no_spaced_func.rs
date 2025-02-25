use memchr::memchr;
use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_spaced_func_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Spacing between function identifiers and their applications is disallowed.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSpacedFunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows spacing between function identifiers and their applications.
    ///
    /// ### Why is this bad?
    ///
    /// While it’s possible to have whitespace between the name of a function and the parentheses
    /// that execute it, such patterns tend to look more like errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// fn ()
    /// ```
    ///
    /// ```javascript
    /// fn
    /// ()
    /// ```
    ///
    /// ```javascript
    /// f.b ()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// fn()
    /// ```
    ///
    /// ```javascript
    /// f.b()
    /// ```
    NoSpacedFunc,
    eslint,
    style,
    pending
);

#[derive(PartialEq, Debug)]
enum FuncSpace {
    NotSpaced,
    Spaced(Span),
}

/// Given an identifier reference will determine
/// if there is spacing in the span from the end of the
/// identifier name to the next `(` char.
///
/// For example `foo  ()` would return `FuncSpace::Spaced(span)`
/// where span refers to the whitespace between `foo` and `()`.
fn get_substring_to_lparens(ctx: &LintContext, search_span: Span) -> FuncSpace {
    let src = ctx.source_range(search_span);

    let Some(char_count) = memchr(b'(', src.as_bytes()) else {
        return FuncSpace::NotSpaced;
    };

    let Ok(char_count_to_l_parens) = u32::try_from(char_count) else {
        return FuncSpace::NotSpaced;
    };

    let l_parens_pos = search_span.start + char_count_to_l_parens;
    let span_to_l_parens = Span::new(search_span.start, l_parens_pos);
    let src_to_l_parens = ctx.source_range(span_to_l_parens);

    if src_to_l_parens.is_empty() {
        return FuncSpace::NotSpaced;
    }

    if src_to_l_parens.trim().is_empty() {
        FuncSpace::Spaced(span_to_l_parens)
    } else {
        FuncSpace::NotSpaced
    }
}

fn check_identifier_callee(ctx: &LintContext, ident_end: u32, callee_end: u32) {
    match get_substring_to_lparens(ctx, Span::new(ident_end, callee_end)) {
        FuncSpace::NotSpaced => {}
        FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
    }
}

impl Rule for NoSpacedFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::NewExpression(new_expr) = node.kind() {
            // new Foo ()
            let callee = new_expr.callee.without_parentheses();
            let callee_end = callee.span().end;

            match get_substring_to_lparens(ctx, Span::new(callee_end, new_expr.span().end)) {
                FuncSpace::NotSpaced => {}
                FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
            }
        };

        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let callee_end = call_expr.span().end;

        match &call_expr.callee {
            // f ()
            Expression::Identifier(ident) => {
                check_identifier_callee(ctx, ident.span().end, callee_end);
            }
            Expression::ParenthesizedExpression(paren_exp) => {
                if let Expression::Identifier(_ident) = &paren_exp.expression {
                    check_identifier_callee(ctx, paren_exp.span().end, callee_end);
                }
            }
            // f() () <-- second parens
            Expression::CallExpression(c) => {
                match get_substring_to_lparens(ctx, Span::new(c.span().end, callee_end)) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            // f.a ()
            Expression::StaticMemberExpression(exp) => {
                //  let span_end = exp.span().end;
                let ident_name_span_end = exp.property.span().end;

                match get_substring_to_lparens(ctx, Span::new(ident_name_span_end, callee_end)) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            // function(){ } ()
            Expression::FunctionExpression(func) => {
                match get_substring_to_lparens(ctx, Span::new(func.span().end, callee_end)) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo(1);",
        "f();",
        "f( );",
        "f(a, b);",
        "a.b",
        "a. b",
        "f.b();",
        "f.b().c();",
        "f.b(1).c( );",
        "f()()",
        "f()( )()",
        "(function() {}())",
        "var f = new Foo()",
        "var f = new Foo",
        "f( (0) )",
        "( f )( 0 )",
        "( (f) )( (0) )",
        "( f()() )(0)",
        "(function(){ if (foo) { bar(); } }());",
        "f(0, (1))",
        "describe/**/('foo', function () {});",
        "new (foo())",
    ];

    let fail = vec![
        "f ();",
        "f (a, b);",
        "f
         	();",
        "f.b ();",
        "f.b().c ();",
        "f.b().c().d ();",
        "f() ()",
        "f()() ()",
        "(function() {} ())",
        "var f = new Foo ()",
        "f ( (0) )",
        "f(0) (1)",
        "(f) (0)",
        "f ();
			 t   ();",
    ];

    /*
    Fix pending.
    let fix = vec![
        ("f ();", "f();", None),
        ("f (a, b);", "f(a, b);", None),
        (
            "f
            ();", "f();", None,
        ),
        ("f.b ();", "f.b();", None),
        ("f.b().c ();", "f.b().c();", None),
        ("f() ()", "f()()", None),
        ("(function() {} ())", "(function() {}())", None),
        ("var f = new Foo ()", "var f = new Foo()", None),
        ("f ( (0) )", "f( (0) )", None),
        ("f(0) (1)", "f(0)(1)", None),
        ("(f) (0)", "(f)(0)", None),
        (
            "f ();
             t   ();",
            "f();
             t();",
            None,
        ),
    ];
    */

    Tester::new(NoSpacedFunc::NAME, NoSpacedFunc::PLUGIN, pass, fail)
        //.expect_fix(fix)
        .test_and_snapshot();
}
