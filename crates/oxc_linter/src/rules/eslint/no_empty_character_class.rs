use memchr::memchr2;
// Ported from https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/no-empty-character-class.js
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::CharacterClass,
    visit::{walk::walk_character_class, Visit},
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
    eslint,
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

            let mut finder = EmptyClassFinder { empty_classes: vec![] };
            finder.visit_pattern(pattern);

            for span in finder.empty_classes {
                ctx.diagnostic(no_empty_character_class_diagnostic(span));
            }
        }
    }
}

struct EmptyClassFinder {
    empty_classes: Vec<Span>,
}

impl Visit<'_> for EmptyClassFinder {
    fn visit_character_class(&mut self, class: &CharacterClass) {
        if !class.negative && class.body.is_empty() {
            self.empty_classes.push(class.span);
        } else {
            walk_character_class(self, class);
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

    Tester::new(NoEmptyCharacterClass::NAME, NoEmptyCharacterClass::PLUGIN, pass, fail)
        .test_and_snapshot();
}
