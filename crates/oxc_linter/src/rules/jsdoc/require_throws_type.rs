use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_throws_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@throws` type.")
        .with_help("Add {type} to `@throws` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireThrowsType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a type on the `@throws` tag.
    ///
    /// ### Why is this bad?
    ///
    /// A `@throws` tag should document the type of error that may be thrown.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /** @throws */
    /// function quux () {
    ///   throw new Error('error');
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @throws {Error} */
    /// function quux () {
    ///   throw new Error('error');
    /// }
    /// ```
    RequireThrowsType,
    jsdoc,
    pedantic,
    version = "1.65.0",
);

impl Rule for RequireThrowsType {
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

                if !matches!(tag_name.parsed(), "throws" | "exception") {
                    continue;
                }

                let (type_part, _) = tag.type_comment();

                if type_part.is_some() {
                    continue;
                }

                ctx.diagnostic(require_throws_type_diagnostic(tag_name.span));
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
                     * @throws {SomeType}
                     */
                  ",
        "
                    /**
                     * @exception {SomeType}
                     */
                  ",
    ];

    let fail = vec![
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
                      * @throws Foo.
                      */
                   ",
        "
                     /**
                      * @exception Foo.
                      */
                   ",
    ];

    Tester::new(RequireThrowsType::NAME, RequireThrowsType::PLUGIN, pass, fail).test_and_snapshot();
}
