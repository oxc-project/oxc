use aho_corasick::AhoCorasick;
use lazy_static::lazy_static;
use oxc_allocator::{Allocator, Vec};
use oxc_ast::{
    ast::{Argument, CallExpression, NewExpression, RegExpLiteral},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::{Alternative, Disjunction, Pattern, Term},
    Parser, ParserOptions,
};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_regex_spaces_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Spaces are hard to count.")
        .with_help("Use a quantifier, e.g. {2}")
        .with_label(span)
}

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
    pending // TODO: This is somewhat autofixable, but the fixer does not exist yet.
);

lazy_static! {
    static ref DOUBLE_SPACE: AhoCorasick =
        AhoCorasick::new(["  "]).expect("no-regex-spaces: Unable to build AhoCorasick");
}

impl Rule for NoRegexSpaces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::RegExpLiteral(lit) => {
                if let Some(span) = Self::find_literal_to_report(lit, ctx) {
                    ctx.diagnostic(no_regex_spaces_diagnostic(span)); // /a  b/
                }
            }

            AstKind::CallExpression(expr) if Self::is_regexp_call_expression(expr) => {
                if let Some(span) = Self::find_expr_to_report(&expr.arguments) {
                    ctx.diagnostic(no_regex_spaces_diagnostic(span)); // RegExp('a  b')
                }
            }

            AstKind::NewExpression(expr) if Self::is_regexp_new_expression(expr) => {
                if let Some(span) = Self::find_expr_to_report(&expr.arguments) {
                    ctx.diagnostic(no_regex_spaces_diagnostic(span)); // new RegExp('a  b')
                }
            }

            _ => {}
        }
    }
}

impl NoRegexSpaces {
    fn find_literal_to_report(literal: &RegExpLiteral, ctx: &LintContext) -> Option<Span> {
        let pattern_text = literal.regex.pattern.source_text(ctx.source_text());
        let pattern_text = pattern_text.as_ref();
        if !Self::has_double_space(pattern_text) {
            return None;
        }

        let pattern = literal.regex.pattern.as_pattern()?;
        Self::find_consecutive_spaces(pattern)
    }

    fn find_expr_to_report(args: &Vec<'_, Argument<'_>>) -> Option<Span> {
        if let Some(expr) = args.get(1).and_then(Argument::as_expression) {
            if !expr.is_string_literal() {
                return None; // skip on indeterminate flag, e.g. RegExp('a  b', flags)
            }
        }

        let Some(Argument::StringLiteral(pattern)) = args.first() else {
            return None;
        };
        if !Self::has_double_space(&pattern.value) {
            return None;
        }

        let alloc = Allocator::default();
        let pattern_with_slashes = format!("/{}/", &pattern.value);
        let parser = Parser::new(&alloc, pattern_with_slashes.as_str(), ParserOptions::default());
        let regex = parser.parse().ok()?;

        Self::find_consecutive_spaces(&regex.pattern)
            .map(|span| Span::new(span.start + pattern.span.start, span.end + pattern.span.start))
    }

    fn find_consecutive_spaces(pattern: &Pattern) -> Option<Span> {
        let mut last_space_span: Option<Span> = None;
        let mut in_quantifier = false;
        visit_terms(pattern, &mut |term| {
            if let Term::Quantifier(_) = term {
                in_quantifier = true;
                return;
            }
            let Term::Character(ch) = term else {
                return;
            };
            if in_quantifier {
                in_quantifier = false;
                return;
            }
            if ch.value != u32::from(b' ') {
                return;
            }
            if let Some(ref mut space_span) = last_space_span {
                // If this is consecutive with the last space, extend it
                if space_span.end == ch.span.start {
                    space_span.end = ch.span.end;
                }
                // If it is not consecutive, and the last space is only one space, move it up
                else if space_span.size() == 1 {
                    last_space_span.replace(ch.span);
                }
            } else {
                last_space_span = Some(ch.span);
            }
        });

        // return None if last_space_span length is only 1
        if last_space_span.is_some_and(|span| span.size() > 1) {
            last_space_span
        } else {
            None
        }
    }

    fn is_regexp_new_expression(expr: &NewExpression<'_>) -> bool {
        expr.callee.is_specific_id("RegExp") && expr.arguments.len() > 0
    }

    fn is_regexp_call_expression(expr: &CallExpression<'_>) -> bool {
        expr.callee.is_specific_id("RegExp") && expr.arguments.len() > 0
    }

    // For skipping if there aren't any consecutive spaces in the source, to avoid reporting cases
    // where the space is explicitly escaped, like: `RegExp(' \ ')``.
    fn has_double_space(input: &str) -> bool {
        DOUBLE_SPACE.is_match(input)
    }
}

/// Calls the given closure on every [`Term`] in the [`Pattern`].
fn visit_terms<'a, F: FnMut(&'a Term<'a>)>(pattern: &'a Pattern, f: &mut F) {
    visit_terms_disjunction(&pattern.body, f);
}

/// Calls the given closure on every [`Term`] in the [`Disjunction`].
fn visit_terms_disjunction<'a, F: FnMut(&'a Term<'a>)>(disjunction: &'a Disjunction, f: &mut F) {
    for alternative in &disjunction.body {
        visit_terms_alternative(alternative, f);
    }
}

/// Calls the given closure on every [`Term`] in the [`Alternative`].
fn visit_terms_alternative<'a, F: FnMut(&'a Term<'a>)>(alternative: &'a Alternative, f: &mut F) {
    for term in &alternative.body {
        match term {
            Term::LookAroundAssertion(lookaround) => {
                f(term);
                visit_terms_disjunction(&lookaround.body, f);
            }
            Term::Quantifier(quant) => {
                f(term);
                f(&quant.body);
            }
            Term::CapturingGroup(group) => {
                f(term);
                visit_terms_disjunction(&group.body, f);
            }
            Term::IgnoreGroup(group) => {
                f(term);
                visit_terms_disjunction(&group.body, f);
            }
            _ => f(term),
        }
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
        "var foo = / /;",
        r"var foo = /bar \\ baz/;",
        r"var foo = /bar\\ \\ baz/;",
        r"var foo = /bar \\u0020 baz/;",
        r"var foo = /bar \\u0020\\u0020baz/;",
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
        "var foo = /  /;",
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

    Tester::new(NoRegexSpaces::NAME, pass, fail).test_and_snapshot();
}
