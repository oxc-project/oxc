use oxc_ast::ast::CommentContent;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::JSDoc;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_next_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("@next should have a description")
        .with_help("Describe the value the caller can pass to the generator's `next()` method.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireNextDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a description for `@next` tags.
    ///
    /// ### Why is this bad?
    ///
    /// A description explains what a generator expects the caller to pass to `next()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /** @next {number} */
    /// function* counter() { let count = 0; while (true) count = yield count; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @next {number} The new counter value. */
    /// function* counter() { let count = 0; while (true) count = yield count; }
    /// ```
    RequireNextDescription,
    jsdoc,
    style,
    version = "next",
    short_description = "Requires a description for `@next` tags.",
);

impl Rule for RequireNextDescription {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        // A JSDoc block can also follow code on the same line.
        for comment in ctx.semantic().comments() {
            if !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal) {
                continue;
            }

            let span = comment.content_span().shrink_left(1);
            let content = ctx.source_range(span);
            if !content.starts_with(char::is_whitespace) || !content.contains("@next") {
                continue;
            }

            let jsdoc = JSDoc::new(content, span);
            if should_ignore_as_internal(&jsdoc, settings)
                || should_ignore_as_private(&jsdoc, settings)
            {
                continue;
            }

            // The upstream rule matches one block at a time and requires a separate tag token.
            if jsdoc.tags().iter().any(|tag| {
                tag.kind.parsed() == "next"
                    && !ctx.source_range(tag.comment().span).starts_with('{')
                    && tag.parsed_type_comment_preserving_lines().trim().is_empty()
            }) {
                ctx.diagnostic(require_next_description_diagnostic(comment.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "/** @next Value sent to the generator. */",
        "/**\n * @next {number}\n * Value sent to the generator.\n */",
        "/** Example uses @next in a sentence. */",
        "/** @nextValue {number} */",
        "function* gen() {} /* @next {number} */",
        "/**@next {number} */",
        "/** @next{number} */",
        "\n        /**\n         * @next {SomeType} Has a description\n         */\n      ",
    ];

    let fail = vec![
        "/** @next */",
        "/**\n * @next\n * {{ value: number }}\n */",
        "function* gen() {} /** @next {number} */",
        "/**\n * @next {number}\n * @next {string}\n */",
        "/** @next */ /** @next */",
        "\n        /**\n         * @next {SomeType}\n         */\n      ",
    ];

    Tester::new(RequireNextDescription::NAME, RequireNextDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}
