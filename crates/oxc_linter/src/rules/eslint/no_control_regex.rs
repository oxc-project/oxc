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
#[diagnostic(severity(error), help("Unexpected control character(s) in regular expression: {0}"))]
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
        if let Some(pattern) = regex_pattern(node) {
            let chars = pattern.chars();
            println!("{:?}", chars)
        }
    }
}

/// Returns the regex pattern inside a node, if it's applicable.
///
/// e.g.:
/// * /foo/ -> "foo"
/// * new RegExp("foo") -> foo
fn regex_pattern<'a>(node: &AstNode<'a>) -> Option<&'a Atom> {
    match node.kind() {
        // regex literal
        AstKind::RegExpLiteral(reg) => Some(&reg.regex.pattern),
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
                // will be runtime errors, but are not covered by this rule
                // let pattern = *pattern;
                Some(&pattern.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Get a list of control characters inside a regular expression pattern.
/// 
/// Note that we are forced to allocate new strings here as [`Atom`] and
/// [`CompactString`] lack impls for string slicing via [`std::ops::Index`]
fn control_patterns(pattern: &Atom) -> Vec<&str> {
    // let pattern = pattern.as_str();
    let buf: [char; 16] = [0 as char; 16];
    // this capacity is arbitrary and may need tuning
    let mut control_chars: Vec<&str> = Vec::with_capacity(4);
    let chars = pattern.chars();
    let mut curr: (isize, isize) = (-1, -1);
    let mut i: isize = 0;

    let s: &str = "hi";
    let x = &s[0..0];
    for char in chars {
        match char {
            '\\' if curr.0 < 0 => {
                // control character found, start recording
                curr.0 = i;
            },
            ctl if ctl >= '\x00' && ctl <= '\x1f' => {
                control_chars.push(&pattern[i..i])
            }
            _ => {
                // noop
            }
        }
        i += 1;
    }

    control_chars
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var regex = /x1f/", None),
        ("var regex = /\x1f/", None),
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
        // U+0000
        ("/\\x00/", None),
        // ("/\\x{0}/", None),
        // ("/\\x0000/", None),
        // ("var reg = new RegExp(\"\\x0C\")", None), // new RegExp("\x0C") - contains raw U+0000 character
        // // somewhere in the middle
        // ("/\\x0c/", None),
        // ("/\\x000c/", None),
        // ("var reg = new RegExp(\"\\\\x0C\")", None), // new RegExp("\\x0C") - \x0c pattern
        // // U+001F
        // ("/\\x1F/", None),
        // ("/\\x{{1f}}/", None),
        // ("/\\x001F/", None),
        // ("var regex = new RegExp('\\x1f\\x1e')", None),
        // ("var regex = new RegExp('\\x1fFOO\\x00')", None),
        // ("var regex = new RegExp('FOO\\x1fFOO\\x1f')", None),
        // ("var regex = RegExp('\\x1f')", None),
        // ("var regex = /(?<a>\\x1f)/", None),
    ];

    Tester::new(NoControlRegex::NAME, pass, fail).test_and_snapshot();
}
