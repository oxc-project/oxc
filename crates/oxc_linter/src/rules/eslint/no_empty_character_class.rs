// Ported from https://github.com/eslint/eslint/blob/main/lib/rules/no-empty-character-class.js
use lazy_static::lazy_static;
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-empty-character-class): Empty class")]
#[diagnostic(severity(warning))]
struct NoEmptyCharacterClassDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoEmptyCharacterClass;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow empty character classes in regular expressions
    ///
    /// ### Why is this bad?
    /// Because empty character classes in regular expressions do not match anything, they might be typing mistakes.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = /^abc[]/;
    /// ```
    NoEmptyCharacterClass,
    correctness
);

impl Rule for NoEmptyCharacterClass {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        lazy_static! {
            /*
            * plain-English description of the following regexp:
            * 0. `^` fix the match at the beginning of the string
            * 1. `([^\\[]|\\.|\[([^\\\]]|\\.)+\])*`: regexp contents; 0 or more of the following
            * 1.0. `[^\\[]`: any character that's not a `\` or a `[` (anything but escape sequences and character classes)
            * 1.1. `\\.`: an escape sequence
            * 1.2. `\[([^\\\]]|\\.)+\]`: a character class that isn't empty
            * 2. `$`: fix the match at the end of the string
            */
            static ref NO_EMPTY_CLASS_REGEX_PATTERN: Regex =
                Regex::new(r"^([^\\\[]|\\.|\[([^\\\]]|\\.)+\])*$").unwrap();
        }

        if let AstKind::RegExpLiteral(lit) = node.kind() {
            if !NO_EMPTY_CLASS_REGEX_PATTERN.is_match(&lit.regex.pattern) {
                ctx.diagnostic(NoEmptyCharacterClassDiagnostic(lit.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = /^abc[a-zA-Z]/;", None),
        ("var regExp = new RegExp(\"^abc[]\");", None),
        ("var foo = /^abc/;", None),
        ("var foo = /[\\[]/;", None),
        ("var foo = /[\\]]/;", None),
        ("var foo = /[a-zA-Z\\[]/;", None),
        ("var foo = /[[]/;", None),
        ("var foo = /[\\[a-z[]]/;", None),
        ("var foo = /[\\-\\[\\]\\/\\{\\}\\(\\)\\*\\+\\?\\.\\\\^\\$\\|]/g;", None),
        ("var foo = /\\s*:\\s*/gim;", None),
        ("var foo = /[\\]]/uy;", None),
        ("var foo = /[\\]]/s;", None),
        ("var foo = /[\\]]/d;", None),
        ("var foo = /\\[]/", None),
    ];

    let fail = vec![
        ("var foo = /^abc[]/;", None),
        ("var foo = /foo[]bar/;", None),
        ("if (foo.match(/^abc[]/)) {}", None),
        ("if (/^abc[]/.test(foo)) {}", None),
        ("var foo = /[]]/;", None),
        ("var foo = /\\[[]/;", None),
        ("var foo = /\\[\\[\\]a-z[]/;", None),
        ("var foo = /[]]/d;", None),
    ];

    Tester::new(NoEmptyCharacterClass::NAME, pass, fail).test_and_snapshot();
}
