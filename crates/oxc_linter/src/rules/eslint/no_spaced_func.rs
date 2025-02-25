use memchr::memchr;
use oxc_ast::AstKind::IdentifierName;
use oxc_ast::ast::Expression;
use oxc_ast::{AstKind, ast::IdentifierReference};
//use oxc_ast::ast::ModuleExportName::IdentifierName;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_spaced_func_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
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
fn is_ident_end_to_l_parens_whitespace(
    ctx: &LintContext,
    ident_end: u32,
    search_end: u32,
) -> FuncSpace {
    let sx = Span::new(ident_end, search_end);
    println!("sx {0:?}", sx);

    let src = ctx.source_range(sx);

    let Some(r_parens_pos_usize) = memchr(b'(', src.as_bytes()) else {
        //println!("span {0:?}", callee_end);
        return FuncSpace::NotSpaced;
    };

    let l_parens_after_n_chars: u32 = u32::try_from(r_parens_pos_usize).unwrap();
    let l_parens_pos = ident_end + l_parens_after_n_chars;
    let span_to_l_parens = Span::new(ident_end, l_parens_pos);
    let str_between_ident_and_l_parens = ctx.source_range(span_to_l_parens);

    println!(" snippet: {0:?}", str_between_ident_and_l_parens);
    if str_between_ident_and_l_parens.is_empty() {
        return FuncSpace::NotSpaced;
    }

    if str_between_ident_and_l_parens.trim().is_empty() {
        return FuncSpace::Spaced(span_to_l_parens);
    } else {
        return FuncSpace::NotSpaced;
    }
}

impl Rule for NoSpacedFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let callee_end = call_expr.span().end; //.contains_inclusive();
        println!("callee_end {0:?}", callee_end);
        println!("{node:?}");
        let callee = call_expr.callee.get_inner_expression(); //.contains_inclusive();

        match callee.without_parentheses() {
            // f()
            Expression::Identifier(ident) => {
                let ident_end = ident.span().end;

                match is_ident_end_to_l_parens_whitespace(ctx, ident_end, callee_end) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            // f() () <-- second parens
            Expression::CallExpression(c) => {
                match is_ident_end_to_l_parens_whitespace(ctx, c.span().end, callee_end) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            // f.a ()
            Expression::StaticMemberExpression(exp) => {
                let span_end = exp.span().end;
                let ident = &exp.property;
                let ident_name_span_end = ident.span().end;

                println!("static ident span {ident_name_span_end:?}");
                println!("static ident name {0:?}", ident.name);

                match is_ident_end_to_l_parens_whitespace(ctx, ident_name_span_end, callee_end) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            // function(){ }() <-- second parens
            Expression::FunctionExpression(func) => {
                match is_ident_end_to_l_parens_whitespace(ctx, func.span().end, callee_end) {
                    FuncSpace::NotSpaced => {}
                    FuncSpace::Spaced(span) => ctx.diagnostic(no_spaced_func_diagnostic(span)),
                }
            }
            _ => {} //     Expression::StaticMemberExpression(static_member_expression) => todo!(),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo(1);",
        "f();",
        "f( );", // I added
        "f(a, b);",
        "a.b",  // I added
        "a. b", // I added
        "f.b();",
        "f.b().c();",
        "f.b(1).c( );", // I added
        "f()()",
        "f()( )()", // I added
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
        "f.b().c().d ();", // I added
        "f() ()",
        "f()() ()", // I added
        "(function() {} ())",
        "var f = new Foo ()",
        "f ( (0) )",
        "f(0) (1)",
        "(f) (0)",
        "f ();
			 t   ();",
    ];

    /*
    Fix pending. Fix is just to delete at the problematic span.
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
