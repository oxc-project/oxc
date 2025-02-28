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
    /// This rule enforces that no whitespace exists between a function's name and its
    /// invocation parentheses. It promotes a compact, consistent style for calling functions.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript allows spaces or newlines between a function name and its parentheses,
    /// but this is uncommon and often mistaken for a typo or unfinished code. Such spacing
    /// can confuse developers, disrupt the visual flow of the code, and deviate from widely
    /// accepted style guides like Airbnb or StandardJS.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// fn ()
    ///
    /// fn
    /// ()
    ///
    /// f.b ()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// fn()
    ///
    /// f.b()
    /// ```
    NoSpacedFunc,
    eslint,
    style,
    fix
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
fn get_substring_to_l_parens(ctx: &LintContext, search_span: Span) -> FuncSpace {
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

    if src_to_l_parens.is_empty() || !src_to_l_parens.trim().is_empty() {
        return FuncSpace::NotSpaced;
    }

    FuncSpace::Spaced(span_to_l_parens)
}

impl Rule for NoSpacedFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let span = match node.kind() {
            AstKind::NewExpression(new_expr) => {
                let callee = new_expr.callee.without_parentheses();
                Span::new(callee.span().end, new_expr.span().end)
            }
            AstKind::CallExpression(call_expr) => {
                let start = match &call_expr.callee {
                    // f ()
                    Expression::Identifier(ident) => ident.span().end,
                    Expression::ParenthesizedExpression(paren_exp) => {
                        if let Expression::Identifier(_) = &paren_exp.expression {
                            paren_exp.span().end
                        } else {
                            return;
                        }
                    }
                    // f() () <-- second parens
                    Expression::CallExpression(c) => c.span().end,
                    // f.a ()
                    Expression::StaticMemberExpression(exp) => exp.property.span().end,
                    // function(){ } ()
                    Expression::FunctionExpression(func) => func.span().end,
                    _ => return,
                };
                Span::new(start, call_expr.span().end)
            }
            _ => return,
        };

        if let FuncSpace::Spaced(span) = get_substring_to_l_parens(ctx, span) {
            ctx.diagnostic_with_fix(no_spaced_func_diagnostic(span), |fixer| fixer.delete(&span));
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

    let fix = vec![
        ("f ();", "f();", None),
        ("f (a, b);", "f(a, b);", None),
        (
            "f
            ();",
            "f();",
            None,
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

    Tester::new(NoSpacedFunc::NAME, NoSpacedFunc::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
