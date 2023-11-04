use oxc_allocator::Vec;
use oxc_ast::{
    ast::{Argument, CallExpression, Expression, NewExpression, RegExpLiteral},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-regex-spaces): Spaces are hard to count.")]
#[diagnostic(severity(warning), help("Use a quantifier, e.g. {{2}}"))]
struct NoRegexSpacesDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoRegexSpaces;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow 2+ consecutive spaces in regular expressions.
    ///
    /// ### Why is this bad?
    ///
    /// In a regular expression, it is hard to tell how many spaces are
    /// intended to be matched. It is better to use only one space and
    /// then specify how many spaces are expected using a quantifier.
    ///
    /// ```javascript
    /// var re = /foo {3}bar/;
    /// ```
    ///
    /// ### Example
    /// ```javascript
    /// var re = /foo   bar/;
    /// ```
    NoRegexSpaces,
    restriction,
);

impl Rule for NoRegexSpaces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::RegExpLiteral(lit) => {
                if let Some(span) = Self::find_literal_to_report(lit) {
                    ctx.diagnostic(NoRegexSpacesDiagnostic(span)); // /a  b/
                }
            }

            AstKind::CallExpression(expr) if Self::is_regexp_call_expression(expr) => {
                if let Some(span) = Self::find_expr_to_report(&expr.arguments) {
                    ctx.diagnostic(NoRegexSpacesDiagnostic(span)); // RegExp('a  b')
                }
            }

            AstKind::NewExpression(expr) if Self::is_regexp_new_expression(expr) => {
                if let Some(span) = Self::find_expr_to_report(&expr.arguments) {
                    ctx.diagnostic(NoRegexSpacesDiagnostic(span)); // new RegExp('a  b')
                }
            }

            _ => {}
        }
    }
}

impl NoRegexSpaces {
    fn find_literal_to_report(lit: &RegExpLiteral) -> Option<Span> {
        if Self::has_exempted_char_class(&lit.regex.pattern) {
            return None;
        }

        if Self::has_target_consecutive_spaces(&lit.regex.pattern) {
            return Some(lit.span);
        }

        None
    }

    fn find_expr_to_report(args: &Vec<'_, Argument<'_>>) -> Option<Span> {
        if let Some(Argument::Expression(expr)) = args.get(1) {
            if !expr.is_string_literal() {
                return None; // skip on indeterminate flag, e.g. RegExp('a  b', flags)
            }
        }

        if let Some(Argument::Expression(Expression::StringLiteral(pattern))) = args.get(0) {
            if Self::has_exempted_char_class(&pattern.value) {
                return None; // skip spaces inside char class, e.g. RegExp('[  ]')
            }

            if Self::has_target_consecutive_spaces(&pattern.value) {
                return Some(pattern.span);
            }
        }

        None
    }

    /// Whether a string has 2+ consecutive spaces, unless followed by a quantifier.
    fn has_target_consecutive_spaces(s: &str) -> bool {
        let chars: std::vec::Vec<char> = s.chars().collect();

        for i in 0..chars.len() - 1 {
            if chars[i] == ' ' && chars[i + 1] == ' ' {
                return !(i + 2 < chars.len()
                    && (chars[i + 2] == '+'
                        || chars[i + 2] == '*'
                        || chars[i + 2] == '{'
                        || chars[i + 2] == '?'));
            }
        }

        false
    }

    fn is_regexp_new_expression(expr: &NewExpression<'_>) -> bool {
        expr.callee.is_specific_id("RegExp") && expr.arguments.len() > 0
    }

    fn is_regexp_call_expression(expr: &CallExpression<'_>) -> bool {
        expr.callee.is_specific_id("RegExp") && expr.arguments.len() > 0
    }

    fn has_exempted_char_class(input: &str) -> bool {
        Self::has_char_class(input) && Self::has_no_consecutive_spaces_outside_char_class(input)
    }

    fn has_char_class(input: &str) -> bool {
        let mut in_character_class = false;

        for c in input.chars() {
            match c {
                '[' => {
                    if in_character_class {
                        return true;
                    }
                    in_character_class = true;
                }
                ']' => {
                    if in_character_class {
                        in_character_class = false;
                    } else {
                        return true;
                    }
                }
                _ => {
                    if in_character_class {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn has_no_consecutive_spaces_outside_char_class(input: &str) -> bool {
        let mut inside_class = false;

        for (i, c) in input.chars().enumerate() {
            match c {
                '[' => inside_class = true,
                ']' => inside_class = false,
                ' ' if input.chars().nth(i + 1) == Some(' ') && !inside_class => {
                    return false;
                }
                _ => {}
            }
        }

        true
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = /foo/;",
        "var foo = RegExp('foo')",
        "var foo = / /;",
        "var foo = RegExp(' ')",
        "var foo = / a b c d /;",
        "var foo = /bar {3}baz/g;",
        "var foo = RegExp('bar {3}baz', 'g')",
        "var foo = new RegExp('bar {3}baz')",
        "var foo = /bar			baz/;",
        "var foo = RegExp('bar			baz');",
        "var foo = new RegExp('bar			baz');",
        "var foo = /  +/;",
        "var foo = /  ?/;",
        "var foo = /  */;",
        "var foo = /  {2}/;",
        "var foo = /  {2}/v;",
        "var foo = /bar \\ baz/;",
        "var foo = /bar\\ \\ baz/;",
        "var foo = /bar \\u0020 baz/;",
        "var foo = /bar \\u0020\\u0020baz/;",
        r"var foo = new RegExp('bar \\ baz')",
        r"var foo = new RegExp('bar\\ \\ baz')",
        r"var foo = new RegExp('bar \\\\ baz')",
        r"var foo = new RegExp('bar\\u0020\\u0020baz')",
        r"var foo = new RegExp('bar \\u0020 baz')",
        "new RegExp('  ', flags)",
        "var foo = /[  ]/;",
        "var foo = /[   ]/;",
        "var foo = / [  ] /;",
        "var foo = / [  ] [  ] /;",
        "var foo = new RegExp('[  ]');",
        "var foo = new RegExp('[   ]');",
        "var foo = new RegExp(' [  ] ');",
        "var foo = RegExp(' [  ] [  ] ');",
        r"var foo = new RegExp(' \[   ');",
        r"var foo = new RegExp(' \[   \] ');",
        "var foo = /[\\q{    }]/v;",
        "var foo = new RegExp('[  ');",
        "new RegExp('[[abc]  ]', flags + 'v')",
    ];

    let fail = vec![
        "var foo = /bar  baz/;",
        "var foo = /bar    baz/;",
        "var foo = / a b  c d /;",
        "var foo = RegExp(' a b c d  ');",
        "var foo = RegExp('bar    baz');",
        "var foo = new RegExp('bar    baz');",
        "var foo = /bar   {3}baz/;",
        "var foo = /bar    ?baz/;",
        "var foo = new RegExp('bar   *baz')",
        "var foo = RegExp('bar   +baz')",
        "var foo = new RegExp('bar    ');",
        r"var foo = /bar\\  baz/;",
        r"var foo = /\\[  /;",
        r"var foo = /\\[  \]/;",
        "var foo = /(?:  )/;",
        "var foo = RegExp('^foo(?=   )');",
        r"var foo = /\\  /",
        r"var foo = / \\  /",
        "var foo = /  foo   /;",
        "var foo = new RegExp('\\d  ')",
        r"var foo = RegExp('\\u0041   ')",
        "var foo = /[   ]  /;",
        "var foo = /  [   ] /;",
        "var foo = RegExp('  [ ]');",
        "var foo = /[[    ]    ]    /v;",
        "var foo = new RegExp('[   ]  ');",
        "var foo = new RegExp('[[    ]    ]    ', 'v');",
    ];

    Tester::new_without_config(NoRegexSpaces::NAME, pass, fail).test_and_snapshot();
}
