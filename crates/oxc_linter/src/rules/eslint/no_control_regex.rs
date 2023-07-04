use lazy_static::lazy_static;
use oxc_ast::{
    ast::{Argument, Expression, RegExpFlags},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use regex::{Match, Matches, Regex};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-control-regex): Unexpected control character(s)")]
#[diagnostic(
    severity(error),
    help("Unexpected control character(s) in regular expression: \"{0}\"")
)]
struct NoControlRegexDiagnostic(Atom, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoControlRegex;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoControlRegex,
    correctness
);

impl Rule for NoControlRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let Some((pattern, flags, span)) = regex_pattern(node) {
            // #[cfg(debug_assertions)] {
            {
                let chars = pattern.chars();
                println!("{:?}", chars)
            }
            let ctl_matches = control_patterns(pattern);
            let violations = ctl_matches
                .map(|ctl| ctl.as_str())
                .filter(|ctl| {
                    if ctl.starts_with(r"\x") || ctl.starts_with(r"\u") {
                        let mut numeric_part = &ctl[2..];

                        // extract numeric part from \u{00}
                        if numeric_part.starts_with('{') {
                            let has_unicode_flag = match flags {
                                Some(flags) if flags.contains(RegExpFlags::U) => true,
                                _ => false,
                            };
                            if !has_unicode_flag {
                                return false;
                            }
                            if !numeric_part.ends_with('}') {
                                // invalid unicode control character, missing
                                // ending curly. filter it out.
                                return false;
                            } else {
                                numeric_part = &numeric_part[1..numeric_part.len() - 1];
                            }
                        }

                        match u32::from_str_radix(numeric_part, 16) {
                            Err(_) => false,
                            Ok(as_num) => as_num <= 0x1f,
                        }
                    } else {
                        true
                    }
                })
                .collect::<Vec<_>>();
            if !violations.is_empty() {
                let violations = violations.join(", ");
                ctx.diagnostic(NoControlRegexDiagnostic(violations.into(), span))
            }
        }
    }
}

fn callee_args<'a, 'alloc>(
    node: &AstNode<'a>,
) -> Option<(&'a Atom, &'a oxc_allocator::Vec<'alloc, Argument<'a>>)> {
    let (callee, args) = match node.kind() {
        AstKind::NewExpression(expr) => Some((&expr.callee, &expr.arguments)),
        AstKind::CallExpression(expr) => Some((&expr.callee, &expr.arguments)),
        _ => None,
    }?;

    if let Expression::Identifier(fn_or_constructor) = callee {
        Some((&fn_or_constructor.name, args))
    } else {
        None
    }
}
fn callish_expr<'a>(node: &'a AstNode<'a>) -> Option<&'a Expression<'a>> {
    match node.kind() {
        AstKind::NewExpression(expr) => Some(&expr.callee),
        AstKind::CallExpression(expr) => Some(&expr.callee),
        _ => None,
    }
}
// enum Flags(O)
/// Returns the regex pattern inside a node, if it's applicable.
///
/// e.g.:
/// * /foo/ -> "foo"
/// * new RegExp("foo") -> foo
/// 
/// note: [`RegExpFlags`] and [`Span`]s are both tiny and cloneable.
fn regex_pattern<'a>(node: &AstNode<'a>) -> Option<(&'a Atom, Option<RegExpFlags>, Span)> {
    let kind = node.kind();
    match kind {
        // regex literal
        AstKind::RegExpLiteral(reg) => Some((&reg.regex.pattern, Some(reg.regex.flags), reg.span)),

        // FIXME: we need a more graceful way to handle NewExpr/CallExprs

        // new RegExp()
        AstKind::NewExpression(expr) => {
            // constructor is RegExp,
            if let Expression::Identifier(ident) = &expr.callee &&
            ident.name == "RegExp" &&
            // which is provided at least 1 parameter,
            expr.arguments.len() > 0 &&
            // where the first one is a string literal
            // note: improvements required for strings used via identifier
            // references
            let Argument::Expression(Expression::StringLiteral(pattern)) = &expr.arguments[0]
            {
                let flags = if expr.arguments.len() > 1 &&
                    let Argument::Expression(Expression::StringLiteral(flag_arg)) = &expr.arguments[1] {
                        // TODO: how should we handle invalid flags?
                        let mut flags = RegExpFlags::empty();
                        for ch in flag_arg.value.chars() {
                            let flag = RegExpFlags::try_from(ch).ok()?;
                            // TODO: should we check for duplicates?
                            flags |= flag;
                        }
                        Some(flags)
                } else {
                    None
                };
                // get pattern from arguments. Missing or non-string arguments
                // will be runtime errors, but are not covered by this rule.
                // Note that we're intentionally reporting the entire "new
                // RegExp("pat") expression, not just "pat".
                Some((&pattern.value, flags, kind.span()))
            } else {
                None
            }
        }

        // RegExp()
        AstKind::CallExpression(expr) => {
            // constructor is RegExp,
            if let Expression::Identifier(ident) = &expr.callee &&
            ident.name == "RegExp" &&
            // which is provided at least 1 parameter,
            expr.arguments.len() > 0 &&
            // where the first one is a string literal
            // note: improvements required for strings used via identifier
            // references
            let Argument::Expression(Expression::StringLiteral(pattern)) = &expr.arguments[0]
            {
                let flags = if expr.arguments.len() > 1 &&
                    let Argument::Expression(Expression::StringLiteral(flag_arg)) = &expr.arguments[1] {
                        // TODO: how should we handle invalid flags?
                        let mut flags = RegExpFlags::empty();
                        for ch in flag_arg.value.chars() {
                            let flag = RegExpFlags::try_from(ch).ok()?;
                            // TODO: should we check for duplicates?
                            flags |= flag;
                        }
                        Some(flags)
                } else {
                    None
                };
                // get pattern from arguments. Missing or non-string arguments
                // will be runtime errors, but are not covered by this rule.
                // Note that we're intentionally reporting the entire "new
                // RegExp("pat") expression, not just "pat".
                Some((&pattern.value, flags, kind.span()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn control_patterns<'a>(pattern: &'a Atom) -> Matches<'static, 'a> {
    lazy_static! {
        static ref CTL_PAT: Regex = Regex::new(
            r"([\x00-\x1f]|(?:\\x\w{2})|(?:\\u\w{4})|(?:\\u\{\w{1,4}\}))"
            // r"((?:\\x\w{2}))"
        ).unwrap();
    }
    CTL_PAT.find_iter(pattern.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_vec {
        [ $($TestCase:expr),* ] => {
            vec![ $( ($TestCase, None) ),*]
        }
    }

    /// Creates a [`Tester`] from a set of `pass`, `fail` cases, where each case
    /// is a string.
    macro_rules! make_test {
        ([$($Pass:expr),*], [$($Fail:expr),*]) => {
            {
                let pass = test_vec![ $($Pass),* ];
                let fail = test_vec![ $($Fail),* ];
                crate::tester::Tester::new(NoControlRegex::NAME, pass, fail)
            }
        }
    }

    #[test]
    fn test_hex_literals() {
        make_test!(
            [
                "x1f",                 // not a control sequence
                r"new RegExp('\x20')", // control sequence in valid range
                r"new RegExp('\xff')",
                r"let r = /\xff/"
            ],
            [r"new RegExp('\x00')", r"/\x00/", r"new RegExp('\x1f')", r"/\x1f/"]
        )
        .test();
    }

    #[test]
    fn test_unicode_literals() {
        make_test!(
            [
                r"u00",    // not a control sequence
                r"\u00ff"  // in valid range
            ],
            [
                // regex literal
                r"let r = /\u0000/",
                r"let r = /\u000c/",
                r"let r = /\u000C/",
                r"let r = /\u001f/",
                // invalid utf ctl as literal string
                r"let r = new RegExp('\u0000');",
                r"let r = new RegExp('\u000c');",
                r"let r = new RegExp('\u000C');",
                r"let r = new RegExp('\u001f');",
                // invalid utf ctl pattern
                r"let r = new RegExp('\\u0000');",
                r"let r = new RegExp('\\u000c');",
                r"let r = new RegExp('\\u000C');",
                r"let r = new RegExp('\\u001f');"
            ]
        )
        .test()
    }

    #[test]
    fn test_unicode_brackets() {
        make_test!(
            [
                r"let r = /\u{0}/", // no unicode flag, this is valid
                r"let r = /\u{ff}/u",
                r"let r = /\u{00ff}/u",
                r"let r = new RegExp('\\u{1F}', flags);" // flags are unknown
            ],
            [
                r"let r = /\u{0}/u",
                r"let r = /\u{c}/u",
                r"let r = /\u{1F}/u" // todo
                                     // r"let r = new RegExp('\\u{1F}', 'u');" // flags are known & contain u
            ]
        )
        .test();
    }

    #[test]
    fn test() {
        // test cases taken from eslint. See:
        // https://github.com/eslint/eslint/blob/main/tests/lib/rules/no-control-regex.js
        make_test!(
            [
                "var regex = /x1f/;",
                // r"var regex = /\\x1f/",               // todo
                "var regex = new RegExp(\"x1f\");",
                "var regex = RegExp(\"x1f\");",
                "new RegExp('[')",
                "RegExp('[')",
                "new (function foo(){})('\\x1f')",
                r"/\u{20}/u",
                r"/\u{1F}/",
                r"/\u{1F}/g",
                r"new RegExp('\\u{20}', 'u')",
                r"new RegExp('\\u{1F}')",
                r"new RegExp('\\u{1F}', 'g')",
                r"new RegExp('\\u{1F}', flags)" // unknown flags, we assume no 'u'
            ],
            // [],
            [
                r"var regex = /\x1f/",
                r"var regex = /\\\x1f\\x1e/",
                r"var regex = /\\\x1fFOO\\x00/",
                r"var regex = /FOO\\\x1fFOO\\x1f/",
                "var regex = new RegExp('\\x1f\\x1e')",
                "var regex = new RegExp('\\x1fFOO\\x00')",
                "var regex = new RegExp('FOO\\x1fFOO\\x1f')",
                "var regex = RegExp('\\x1f')",
                "var regex = /(?<a>\\x1f)/",
                r"var regex = /(?<\u{1d49c}>.)\x1f/",
                r"new RegExp('\\u{1111}*\\x1F', 'u')",
                r"/\u{1F}/u",
                r"/\u{1F}/ugi",
                r"new RegExp('\\u{1F}', 'u')",
                r"new RegExp('\\u{1F}', 'ugi')"
            ] // [
              // "var regex = RegExp('\\x1f')"
              // ]
        )
        .test()
        // let pass = vec![
        //     ("var regex = /x1f/", None), // not a control pattern
        //     ("var regex = /x1F/", None),
        //     ("var regex = new RegExp('x1f')", None),
        //     ("var regex = RegExp('x1f')", None),
        //     ("new RegExp('[')", None),
        //     ("RegExp('[')", None),
        //     ("new (function foo(){})('\\x1f')", None),
        //     // \t and \n are ok
        //     ("/\\t/", None),
        //     ("/\\n/", None),
        //     ("new RegExp(\"\\\\t\");", None),
        //     ("new RegExp(\"\\\\n\");", None),
        // ];

        // let fail = vec![
        //     // tab
        //     // U+0000
        //     ("/\\x00/", None),
        //     (r"let r = /\u{0}/u", None),
        //     ("/\\x0000/", None),
        //     ("var reg = new RegExp(\"\\x0C\")", None), // new RegExp("\x0C") - contains raw U+0000 character
        //     // somewhere in the middle
        //     ("/\\x0c/", None),
        //     ("/\\x000c/", None),
        //     ("var reg = new RegExp(\"\\\\x0C\")", None), // new RegExp("\\x0C") - \x0c pattern
        //     // U+001F
        //     ("/\\x1F/", None),
        //     ("var regex = /\x1f/", None),
        //     ("/\\x{{1f}}/", None),
        //     ("/\\x001F/", None),
        //     ("var regex = new RegExp('\\x1f\\x1e')", None),
        //     ("var regex = new RegExp('\\x1fFOO\\x00')", None),
        //     ("var regex = new RegExp('FOO\\x1fFOO\\x1f')", None),
        //     ("var regex = RegExp('\\x1f')", None),
        //     ("var regex = /(?<a>\\x1f)/", None),
        // ];

        // Tester::new(NoControlRegex::NAME, pass, fail).test();
    }
}
