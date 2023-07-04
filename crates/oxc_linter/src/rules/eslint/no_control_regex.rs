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
use regex::{Matches, Regex};

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
    /// Disallows control characters and some escape sequences that match
    /// control characters in regular expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Control characters are special, invisible characters in the ASCII range
    /// 0-31. These characters are rarely used in JavaScript strings so a
    /// regular expression containing elements that explicitly match these
    /// characters is most likely a mistake.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```javascript
    /// var pattern1 = /\x00/;
    /// var pattern2 = /\x0C/;
    /// var pattern3 = /\x1F/;
    /// var pattern4 = /\u000C/;
    /// var pattern5 = /\u{C}/u;
    /// var pattern6 = new RegExp("\x0C"); // raw U+000C character in the pattern
    /// var pattern7 = new RegExp("\\x0C"); // \x0C pattern
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```javascript
    /// var pattern1 = /\x20/;
    /// var pattern2 = /\u0020/;
    /// var pattern3 = /\u{20}/u;
    /// var pattern4 = /\t/;
    /// var pattern5 = /\n/;
    /// var pattern6 = new RegExp("\x20");
    /// var pattern7 = new RegExp("\\t");
    /// var pattern8 = new RegExp("\\n");
    /// ```
    NoControlRegex,
    correctness
);

impl Rule for NoControlRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let Some((pattern, flags, span)) = regex_pattern(node) {
            let mut violations: Vec<&str> = Vec::with_capacity(usize::min(4, pattern.len()));
            for m in control_patterns(pattern) {
                let ctl = m.as_str();

                // check for an even number of backslashes, since these will
                // prevent the pattern from being a control sequence
                if ctl.starts_with('\\') && m.start() > 0 {
                    let pattern_chars: Vec<char> = pattern.chars().collect(); // ew

                    let mut first_backslash = m.start();
                    while first_backslash > 0 && pattern_chars[first_backslash] == '\\' {
                        first_backslash -= 1;
                    }

                    let mut num_slashes = m.start() - first_backslash;
                    if first_backslash == 0 && pattern_chars[first_backslash] == '\\' {
                        num_slashes += 1;
                    }
                    // even # of slashes
                    if num_slashes % 2 == 0 {
                        continue;
                    }
                }

                if ctl.starts_with(r"\x") || ctl.starts_with(r"\u") {
                    let mut numeric_part = &ctl[2..];

                    // extract numeric part from \u{00}
                    if numeric_part.starts_with('{') {
                        let has_unicode_flag = match flags {
                            Some(flags) if flags.contains(RegExpFlags::U) => true,
                            _ => {
                                continue;
                            }
                        };

                        // 1. Unicode control pattern is missing a curly brace
                        //    and is therefore invalid. (note: we may want to
                        //    report this?)
                        // 2. Unicode flag is missing, which is needed for
                        //    interpreting \u{`nn`} as a unicode character
                        if !has_unicode_flag || !numeric_part.ends_with('}') {
                            continue;
                        } else if  {
                            continue;
                        } else {
                            numeric_part = &numeric_part[1..numeric_part.len() - 1];
                        }
                    }

                    match u32::from_str_radix(numeric_part, 16) {
                        Err(_) => continue,
                        Ok(as_num) if as_num > 0x1f => continue,
                        Ok(_) => { /* noop */ }
                    }
                }

                violations.push(ctl);
            }

            if !violations.is_empty() {
                let violations = violations.join(", ");
                ctx.diagnostic(NoControlRegexDiagnostic(violations.into(), span));
            }
        }
    }
}

fn extract_flags<'a>(args: &'a oxc_allocator::Vec<'a, Argument<'a>>) -> Option<RegExpFlags> {
    if args.len() > 1 &&
    let Argument::Expression(Expression::StringLiteral(flag_arg)) = &args[1] {
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
    }
}

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
                // get pattern from arguments. Missing or non-string arguments
                // will be runtime errors, but are not covered by this rule.
                // Note that we're intentionally reporting the entire "new
                // RegExp("pat") expression, not just "pat".
                Some((&pattern.value, extract_flags(&expr.arguments), kind.span()))
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
                // get pattern from arguments. Missing or non-string arguments
                // will be runtime errors, but are not covered by this rule.
                // Note that we're intentionally reporting the entire "new
                // RegExp("pat") expression, not just "pat".
                Some((&pattern.value, extract_flags(&expr.arguments), kind.span()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn control_patterns(pattern: &Atom) -> Matches<'static, '_> {
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
                r"let r = /\u{1F}/u",
                r"let r = new RegExp('\\u{1F}', 'u');" // flags are known & contain u
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
                r"var regex = /\\x1f/",
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
            ]
        )
        .test_and_snapshot()
    }
}
