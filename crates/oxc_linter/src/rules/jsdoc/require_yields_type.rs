use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_yields_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@yields` type.")
        .with_help("Add {type} to `@yields` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireYieldsType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a type on the `@yields` tag.
    ///
    /// ### Why is this bad?
    ///
    /// A `@yields` tag should document the type yielded by the generator.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /** @yields */
    /// function * quux () {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @yields {string} */
    /// function * quux () {}
    /// ```
    RequireYieldsType,
    jsdoc,
    pedantic,
    version = "1.65.0",
);

impl Rule for RequireYieldsType {
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

                let (type_part, _) = tag.type_comment();

                if type_part.is_some() {
                    continue;
                }

                ctx.diagnostic(require_yields_type_diagnostic(tag_name.span));
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
                     * @yields {SomeType}
                     */
                  ",
        "
                    /**
                     * @yield {SomeType}
                     */
                  ",
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
                     * @yield Foo.
                     */
                  ",
    ];

    Tester::new(RequireYieldsType::NAME, RequireYieldsType::PLUGIN, pass, fail).test_and_snapshot();
}
