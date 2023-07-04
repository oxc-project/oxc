use regex::{Match, Regex, Matches};
use lazy_static::lazy_static;

use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-control-regex): Unexpected control character(s)")]
#[diagnostic(severity(error), help("Unexpected control character(s) in regular expression: \"{0}\""))]
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
        if let Some((pattern, span)) = regex_pattern(node) {
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
                            if !numeric_part.ends_with('}') {
                                // invalid unicode control character, missing
                                // ending curly. filter it out.
                                return false
                            } else {
                                numeric_part = &numeric_part[1..numeric_part.len() - 1];
                            }
                        }

                        match u32::from_str_radix(numeric_part, 16) {
                            Err(_) => false,
                            Ok(as_num) => as_num <= 0x1f
                        }
                    } else {
                        true
                    }
                }).collect::<Vec<_>>();
            if !violations.is_empty() {
                let violations = violations.join(", ");
                ctx.diagnostic(NoControlRegexDiagnostic(violations.into(), span))
            }
        }
    }
}

/// Returns the regex pattern inside a node, if it's applicable.
///
/// e.g.:
/// * /foo/ -> "foo"
/// * new RegExp("foo") -> foo
fn regex_pattern<'a>(node: &AstNode<'a>) -> Option<(&'a Atom, Span)> {
    match node.kind() {
        // regex literal
        AstKind::RegExpLiteral(reg) => Some((&reg.regex.pattern, reg.span)),
        // new RegEx()
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
                // get pattern from arguments. Missing or non-string arguments
                // will be runtime errors, but are not covered by this rule.
                // Note that we're intentionally reporting the entire "new
                // RegExp("pat") expression, not just "pat".
                Some((&pattern.value, expr.span))
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
                "x1f", // not a control sequence
                r"new RegExp('\x20')", // control sequence in valid range
                r"new RegExp('\xff')", 
                r"let r = /\xff/"
            ],
            [r"new RegExp('\x00')", r"/\x00/",r"new RegExp('\x1f')", r"/\x1f/"]

        ).test();
    }

    #[test]
    fn test_unicode_literals() {
        make_test!(
            [
                r"u00", // not a control sequence
                r"\u00ff" // in valid range
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
        ).test()
    }

    #[test]
    fn test_unicode_brackets() {
        make_test!(
            [r"let r = /\u{ff}/", r"let r = /\u{00ff}/"],
            [r"let r = /\u{0}/", r"let r = /\u{c}/", r"let r = /\u{1F}/"]
        ).test();
    }

    #[test]
    fn test() {
        use crate::tester::Tester;

        let pass = vec![
            ("var regex = /x1f/", None), // not a control pattern
            // ("var regex = /x1F/", None),
            // ("var regex = new RegExp('x1f')", None),
            // ("var regex = RegExp('x1f')", None),
            // ("new RegExp('[')", None),
            // ("RegExp('[')", None),
            // ("new (function foo(){})('\\x1f')", None),
            // // \t and \n are ok
            // ("/\\t/", None),
            // ("/\\n/", None),
            // ("new RegExp(\"\\\\t\");", None),
            // ("new RegExp(\"\\\\n\");", None),
        ];

        let fail = vec![
            // tab 
            // U+0000
            ("/\\x00/", None),
            ("/\\x{0}/", None),
            ("/\\x0000/", None),
            ("var reg = new RegExp(\"\\x0C\")", None), // new RegExp("\x0C") - contains raw U+0000 character
            // somewhere in the middle
            ("/\\x0c/", None),
            ("/\\x000c/", None),
            ("var reg = new RegExp(\"\\\\x0C\")", None), // new RegExp("\\x0C") - \x0c pattern
            // U+001F
            ("/\\x1F/", None),
            ("var regex = /\x1f/", None),
            ("/\\x{{1f}}/", None),
            ("/\\x001F/", None),
            ("var regex = new RegExp('\\x1f\\x1e')", None),
            ("var regex = new RegExp('\\x1fFOO\\x00')", None),
            ("var regex = new RegExp('FOO\\x1fFOO\\x1f')", None),
            ("var regex = RegExp('\\x1f')", None),
            ("var regex = /(?<a>\\x1f)/", None),
        ];

        Tester::new(NoControlRegex::NAME, pass, fail).test();
    }
}
