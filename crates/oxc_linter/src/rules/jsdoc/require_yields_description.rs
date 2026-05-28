use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_yields_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@yields` description.")
        .with_help("Add description comment to `@yields` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireYieldsDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a description for `@yields` tags.
    ///
    /// ### Why is this bad?
    ///
    /// A `@yields` tag should explain what the generator yields.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /**
    ///  * @yields {string}
    ///  */
    /// function * quux () {
    ///   yield 'value';
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /**
    ///  * @yields {string} The next value.
    ///  */
    /// function * quux () {
    ///   yield 'value';
    /// }
    /// ```
    RequireYieldsDescription,
    jsdoc,
    style,
    version = "next",
);

impl Rule for RequireYieldsDescription {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;

        for jsdoc in ctx
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind;

                if !matches!(tag_name.parsed(), "yield" | "yields") {
                    continue;
                }

                let (_, comment_part) = tag.type_comment();

                if comment_part.parsed().is_empty() {
                    ctx.diagnostic(require_yields_description_diagnostic(tag_name.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
                    /**
                     * @yields {SomeType} Has a description
                     */
                  ",
        "
                    /**
                     * @yields Has a description
                     */
                  ",
        "
                    /**
                     * @yield {SomeType} Has a description
                     */
                  ",
        "
                    /**
                     * @yield Has a description
                     */
                  ",
        r#"
                    /**
                     * @yields
                     *  The results.
                     */
                    export function *test1(): IterableIterator<string> { yield "hello"; }
                  "#, // { "parser": typescriptEslintParser, }
    ];

    let fail = vec![
        "
                    /**
                     * @yields
                     */
                  ",
        "
                    /**
                     * @yield
                     */
                  ",
        "
                    /**
                     * @yields {SomeType}
                     */
                  ",
        "
                    /**
                     * @yield {SomeType}
                     */
                  ",
    ];

    Tester::new(RequireYieldsDescription::NAME, RequireYieldsDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}
