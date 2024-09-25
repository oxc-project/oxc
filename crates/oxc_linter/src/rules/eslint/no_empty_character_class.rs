use memchr::memchr2;
// Ported from https://github.com/eslint/eslint/blob/main/lib/rules/no-empty-character-class.js
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::ast::{
    Alternative, CharacterClass, CharacterClassContents, Disjunction, Pattern, Term,
};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_empty_character_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty character class will not match anything")
        .with_help("Remove the empty character class: `[]`")
        .with_label(span)
}

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
        if let AstKind::RegExpLiteral(lit) = node.kind() {
            let Some(pattern) = lit.regex.pattern.as_pattern() else {
                return;
            };

            // Skip if the pattern doesn't contain a `[` or `]` character
            if memchr2(b'[', b']', lit.regex.pattern.source_text(ctx.source_text()).as_bytes())
                .is_none()
            {
                return;
            }

            visit_terms(pattern, &mut |term| {
                if let Term::CharacterClass(class) = term {
                    check_character_class(ctx, class);
                }
            });
        }
    }
}

fn check_character_class(ctx: &LintContext, class: &CharacterClass) {
    // Class has nothing in it, example: `/[]/`
    if !class.negative && class.body.is_empty() {
        ctx.diagnostic(no_empty_character_class_diagnostic(class.span));
        return;
    }

    // Class has something in it, but might contain empty nested character classes,
    // example: `/[[]]/`
    for term in &class.body {
        if let CharacterClassContents::NestedCharacterClass(class) = term {
            check_character_class(ctx, class);
        }
    }
}

// TODO: Replace with proper regex AST visitor when available
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
        // ES2024
        ("var foo = /[[^]]/v;", None),    // { "ecmaVersion": 2024 }
        ("var foo = /[[\\]]]/v;", None),  // { "ecmaVersion": 2024 }
        ("var foo = /[[\\[]]/v;", None),  // { "ecmaVersion": 2024 }
        ("var foo = /[a--b]/v;", None),   // { "ecmaVersion": 2024 }
        ("var foo = /[a&&b]/v;", None),   // { "ecmaVersion": 2024 }
        ("var foo = /[[a][b]]/v;", None), // { "ecmaVersion": 2024 }
        ("var foo = /[\\q{}]/v;", None),  // { "ecmaVersion": 2024 }
        ("var foo = /[[^]--\\p{ASCII}]/v;", None), // { "ecmaVersion": 2024 }
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
        ("var foo = /[[][]]/v;", None),
        ("var foo = /[[]]|[]/v;", None),
        ("var foo = /[(]\\u{0}*[]/u;", None), // { "ecmaVersion": 2015 }
        // ES2024
        ("var foo = /[]/v;", None),           // { "ecmaVersion": 2024 }
        ("var foo = /[[]]/v;", None),         // { "ecmaVersion": 2024 }
        ("var foo = /[[a][]]/v;", None),      // { "ecmaVersion": 2024 }
        ("var foo = /[a[[b[]c]]d]/v;", None), // { "ecmaVersion": 2024 }
        ("var foo = /[a--[]]/v;", None),      // { "ecmaVersion": 2024 }
        ("var foo = /[[]--b]/v;", None),      // { "ecmaVersion": 2024 }
        ("var foo = /[a&&[]]/v;", None),      // { "ecmaVersion": 2024 }
        ("var foo = /[[]&&b]/v;", None),      // { "ecmaVersion": 2024 }
    ];

    Tester::new(NoEmptyCharacterClass::NAME, pass, fail).test_and_snapshot();
}
