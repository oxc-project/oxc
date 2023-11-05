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
    fn find_literal_to_report(literal: &RegExpLiteral) -> Option<Span> {
        if Self::has_exempted_char_class(&literal.regex.pattern) {
            return None;
        }

        if let Some((idx_start, idx_end)) =
            Self::find_consecutive_spaces_indices(&literal.regex.pattern)
        {
            let start = literal.span.start + idx_start + 1;
            let end = literal.span.start + idx_end + 2;

            return Some(Span { start, end });
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

            if let Some((idx_start, idx_end)) =
                Self::find_consecutive_spaces_indices(&pattern.value)
            {
                let start = pattern.span.start + idx_start + 1;
                let end = pattern.span.start + idx_end + 2;

                return Some(Span { start, end });
            }
        }

        None
    }

    fn find_consecutive_spaces_indices(input: &str) -> Option<(u32, u32)> {
        let mut start: Option<u32> = None;
        let mut consecutive_spaces = 0;

        for (cur_idx, char) in input.char_indices() {
            if char == ' ' {
                consecutive_spaces += 1;
                if start.is_none() {
                    start = Some(u32::try_from(cur_idx).unwrap());
                }
                if consecutive_spaces >= 2 {
                    if let Some(next_char) = input.chars().nth(cur_idx + 1) {
                        if consecutive_spaces > 2 && "+*{?".contains(next_char) {
                            return start
                                .map(|start_idx| (start_idx, u32::try_from(cur_idx).unwrap()));
                        }

                        if !"+*{?".contains(next_char) && next_char != ' ' {
                            return start
                                .map(|start_idx| (start_idx, u32::try_from(cur_idx).unwrap()));
                        }
                    } else {
                        return start.map(|start_idx| (start_idx, u32::try_from(cur_idx).unwrap()));
                    }
                }
            } else {
                start = Some(u32::try_from(cur_idx).unwrap() + 1);
                consecutive_spaces = 0;
            }
        }

        None
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
        r"var foo = new RegExp('\\d  ')",
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
