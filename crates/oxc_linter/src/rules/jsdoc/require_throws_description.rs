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

fn require_throws_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@throws` description.")
        .with_help("Add description comment to `@throws` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireThrowsDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a description for `@throws` tags.
    ///
    /// ### Why is this bad?
    ///
    /// A `@throws` tag should explain the condition or reason an error may be thrown.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /**
    ///  * @throws {Error}
    ///  */
    /// function quux () {
    ///   throw new Error('error');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /**
    ///  * @throws {Error} Has a description
    ///  */
    /// function quux () {
    ///   throw new Error('error');
    /// }
    /// ```
    RequireThrowsDescription,
    jsdoc,
    style,
    version = "1.65.0",
    short_description = "Requires a description for `@throws` tags.",
);

impl Rule for RequireThrowsDescription {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;

        for comment in ctx.semantic().comments() {
            // The semantic JSDoc index excludes trailing comments.
            if !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal) {
                continue;
            }

            let span = comment.content_span().shrink_left(1);
            let content = ctx.source_range(span);
            if !content.starts_with(char::is_whitespace)
                || !(content.contains("@throws") || content.contains("@exception"))
            {
                continue;
            }

            let jsdoc = JSDoc::new(content, span);
            if should_ignore_as_internal(&jsdoc, settings)
                || should_ignore_as_private(&jsdoc, settings)
            {
                continue;
            }

            if let Some(tag) = jsdoc.tags().iter().find(|tag| {
                matches!(tag.kind.parsed(), "throws" | "exception")
                    && !ctx.source_range(tag.comment().span).starts_with('{')
                    && tag.parsed_type_comment_preserving_lines().trim().is_empty()
            }) {
                ctx.diagnostic(require_throws_description_diagnostic(tag.kind.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function example() {} /** @throws {Error} On failure. */",
        "function example() {} /* @throws {Error} */",
        "function example() {} /**** @throws {Error} */",
        "function example() {} /**@throws {Error} */",
        "function example() {} /** @throws{Error} */",
        "/**\n * @throws\n * {Error} On failure.\n */",
        "
                    /**
                     * @throws {SomeType} Has a description
                     */
                  ",
        "
                    /**
                     * @throws Has a description
                     */
                  ",
        "
                    /**
                     * @exception {SomeType} Has a description
                     */
                  ",
        "
                    /**
                     * @exception Has a description
                     */
                  ",
    ];

    let fail = vec![
        "function example() {} /** @throws {Error} */",
        "function example() {} /** @exception */",
        "/**\n * @throws {TypeError}\n * @exception {RangeError}\n */",
        "/**\n * @throws\n * {Error}\n */",
        "
                    /**
                     * @throws
                     */
                  ",
        "
                    /**
                     * @exception
                     */
                  ",
        "
                    /**
                     * @throws {SomeType}
                     */
                  ",
        "
                    /**
                     * @exception {SomeType}
                     */
                  ",
    ];

    Tester::new(RequireThrowsDescription::NAME, RequireThrowsDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn settings_for_trailing_comments() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            "foo(); /**\n * @internal\n * @throws {Error}\n */",
            None,
            Some(json!({ "settings": { "jsdoc": { "ignoreInternal": true } } })),
        ),
        (
            "foo(); /**\n * @private\n * @exception {Error}\n */",
            None,
            Some(json!({ "settings": { "jsdoc": { "ignorePrivate": true } } })),
        ),
        (
            "foo(); /** @raises {Error} */",
            None,
            Some(
                json!({ "settings": { "jsdoc": { "tagNamePreference": { "throws": "raises" } } } }),
            ),
        ),
    ];
    let fail = vec![
        (
            "foo(); /** @throws {Error} */",
            None,
            Some(
                json!({ "settings": { "jsdoc": { "tagNamePreference": { "throws": "raises" } } } }),
            ),
        ),
        (
            "foo(); /** @exception {Error} */",
            None,
            Some(
                json!({ "settings": { "jsdoc": { "tagNamePreference": { "exception": false } } } }),
            ),
        ),
    ];

    Tester::new(RequireThrowsDescription::NAME, RequireThrowsDescription::PLUGIN, pass, fail)
        .test();
}
