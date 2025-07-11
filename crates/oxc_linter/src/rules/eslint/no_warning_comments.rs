use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn no_warning_comments_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoWarningComments;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoWarningComments,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoWarningComments {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // ctx.diagnostic(
        //     OxcDiagnostic::warn("Warning comments should be avoided")
        //         .with_help("Use a command-like statement that tells the user how to fix the issue")
        // )
    }
}

#[ignore]
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("// any comment", Some(serde_json::json!([{ "terms": ["fixme"] }]))),
        ("// any comment", Some(serde_json::json!([{ "terms": ["fixme", "todo"] }]))),
        ("// any comment", None),
        ("// any comment", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "// any comment with TODO, FIXME or XXX",
            Some(serde_json::json!([{ "location": "start" }])),
        ),
        ("// any comment with TODO, FIXME or XXX", None),
        ("/* any block comment */", Some(serde_json::json!([{ "terms": ["fixme"] }]))),
        ("/* any block comment */", Some(serde_json::json!([{ "terms": ["fixme", "todo"] }]))),
        ("/* any block comment */", None),
        ("/* any block comment */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "/* any block comment with TODO, FIXME or XXX */",
            Some(serde_json::json!([{ "location": "start" }])),
        ),
        ("/* any block comment with TODO, FIXME or XXX */", None),
        ("/* any block comment with (TODO, FIXME's or XXX!) */", None),
        (
            "// comments containing terms as substrings like TodoMVC",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "anywhere" }])),
        ),
        (
            "// special regex characters don't cause a problem",
            Some(serde_json::json!([{ "terms": ["[aeiou]"], "location": "anywhere" }])),
        ),
        (
            r#"/*eslint no-warning-comments: [2, { "terms": ["todo", "fixme", "any other term"], "location": "anywhere" }]*/

			var x = 10;
			"#,
            None,
        ),
        (
            r#"/*eslint no-warning-comments: [2, { "terms": ["todo", "fixme", "any other term"], "location": "anywhere" }]*/

			var x = 10;
			"#,
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        ("// foo", Some(serde_json::json!([{ "terms": ["foo-bar"] }]))),
        (
            "/** multi-line block comment with lines starting with
			TODO
			FIXME or
			XXX
			*/",
            None,
        ),
        ("//!TODO ", Some(serde_json::json!([{ "decoration": ["*"] }]))),
    ];

    let fail = vec![
        ("// fixme", None),
        ("// any fixme", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("// any fixme", Some(serde_json::json!([{ "terms": ["fixme"], "location": "anywhere" }]))),
        ("// any FIXME", Some(serde_json::json!([{ "terms": ["fixme"], "location": "anywhere" }]))),
        ("// any fIxMe", Some(serde_json::json!([{ "terms": ["fixme"], "location": "anywhere" }]))),
        (
            "/* any fixme */",
            Some(serde_json::json!([{ "terms": ["FIXME"], "location": "anywhere" }])),
        ),
        (
            "/* any FIXME */",
            Some(serde_json::json!([{ "terms": ["FIXME"], "location": "anywhere" }])),
        ),
        (
            "/* any fIxMe */",
            Some(serde_json::json!([{ "terms": ["FIXME"], "location": "anywhere" }])),
        ),
        (
            "// any fixme or todo",
            Some(serde_json::json!([{ "terms": ["fixme", "todo"], "location": "anywhere" }])),
        ),
        (
            "/* any fixme or todo */",
            Some(serde_json::json!([{ "terms": ["fixme", "todo"], "location": "anywhere" }])),
        ),
        ("/* any fixme or todo */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("/* fixme and todo */", None),
        ("/* fixme and todo */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("/* any fixme */", Some(serde_json::json!([{ "location": "anywhere" }]))),
        ("/* fixme! */", Some(serde_json::json!([{ "terms": ["fixme"] }]))),
        (
            "// regex [litera|$]",
            Some(serde_json::json!([{ "terms": ["[litera|$]"], "location": "anywhere" }])),
        ),
        ("/* eslint one-var: 2 */", Some(serde_json::json!([{ "terms": ["eslint"] }]))),
        (
            "/* eslint one-var: 2 */",
            Some(serde_json::json!([{ "terms": ["one"], "location": "anywhere" }])),
        ),
        (
            "/* any block comment with TODO, FIXME or XXX */",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "/* any block comment with (TODO, FIXME's or XXX!) */",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "/**
			 *any block comment
			*with (TODO, FIXME's or XXX!) **/",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "// any comment with TODO, FIXME or XXX",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        ("// TODO: something small", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "// TODO: something really longer than 40 characters",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "/* TODO: something
			 really longer than 40 characters
			 and also a new line */",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        ("// TODO: small", Some(serde_json::json!([{ "location": "anywhere" }]))),
        (
            "// https://github.com/eslint/eslint/pull/13522#discussion_r470293411 TODO",
            Some(serde_json::json!([{ "location": "anywhere" }])),
        ),
        (
            "// Comment ending with term followed by punctuation TODO!",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "anywhere" }])),
        ),
        (
            "// Comment ending with term including punctuation TODO!",
            Some(serde_json::json!([{ "terms": ["todo!"], "location": "anywhere" }])),
        ),
        (
            "// Comment ending with term including punctuation followed by more TODO!!!",
            Some(serde_json::json!([{ "terms": ["todo!"], "location": "anywhere" }])),
        ),
        (
            "// !TODO comment starting with term preceded by punctuation",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "anywhere" }])),
        ),
        (
            "// !TODO comment starting with term including punctuation",
            Some(serde_json::json!([{ "terms": ["!todo"], "location": "anywhere" }])),
        ),
        (
            "// !!!TODO comment starting with term including punctuation preceded by more",
            Some(serde_json::json!([{ "terms": ["!todo"], "location": "anywhere" }])),
        ),
        (
            "// FIX!term ending with punctuation followed word character",
            Some(serde_json::json!([{ "terms": ["FIX!"], "location": "anywhere" }])),
        ),
        (
            "// Term starting with punctuation preceded word character!FIX",
            Some(serde_json::json!([{ "terms": ["!FIX"], "location": "anywhere" }])),
        ),
        (
            "//!XXX comment starting with no spaces (anywhere)",
            Some(serde_json::json!([{ "terms": ["!xxx"], "location": "anywhere" }])),
        ),
        (
            "//!XXX comment starting with no spaces (start)",
            Some(serde_json::json!([{ "terms": ["!xxx"], "location": "start" }])),
        ),
        (
            "/*
			TODO undecorated multi-line block comment (start)
			*/",
            Some(serde_json::json!([{ "terms": ["todo"], "location": "start" }])),
        ),
        (
            "///// TODO decorated single-line comment with decoration array
			 /////",
            Some(
                serde_json::json!([				{ "terms": ["todo"], "location": "start", "decoration": ["*", "/"] },			]),
            ),
        ),
        (
            "///*/*/ TODO decorated single-line comment with multiple decoration characters (start)
			 /////",
            Some(
                serde_json::json!([				{ "terms": ["todo"], "location": "start", "decoration": ["*", "/"] },			]),
            ),
        ),
        (
            "//**TODO term starts with a decoration character",
            Some(
                serde_json::json!([				{ "terms": ["*todo"], "location": "start", "decoration": ["*"] },			]),
            ),
        ),
    ];

    Tester::new(NoWarningComments::NAME, NoWarningComments::PLUGIN, pass, fail).test_and_snapshot();
}
