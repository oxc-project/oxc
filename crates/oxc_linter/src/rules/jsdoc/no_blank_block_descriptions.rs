use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_blank_block_descriptions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("No blank block descriptions").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoBlankBlockDescriptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents empty lines in JSDoc block descriptions when tags are present.
    /// When no tags are present, it prevents extra empty lines in the block
    /// description.
    ///
    /// ### Why is this bad?
    ///
    /// Unnecessary empty lines make JSDoc blocks inconsistent and add visual
    /// noise without improving readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /**
    ///  *
    ///  * @param {number} x
    ///  */
    ///
    /// /**
    ///  *
    ///  *
    ///  */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /**
    ///  * Non-empty description
    ///  * @param {number} x
    ///  */
    ///
    /// /**
    ///  * @param {number} x
    ///  */
    ///
    /// /**
    ///  *
    ///  */
    ///
    /// /** */
    ///
    /// /** Some description. */
    ///
    /// /** @someTag */
    /// ```
    NoBlankBlockDescriptions,
    jsdoc,
    style,
    pending,
    version = "next",
    short_description = "Prevents empty lines in JSDoc block descriptions.",
);

impl Rule for NoBlankBlockDescriptions {
    fn run_once(&self, ctx: &LintContext) {
        for jsdoc in ctx.jsdoc().iter_all() {
            let comment = jsdoc.comment();
            let content = comment.parsed_preserving_whitespace();

            if !content.trim().is_empty() {
                continue;
            }

            let description_line_count = count_description_lines(&content);
            if description_line_count == 0 {
                continue;
            }

            if jsdoc.tags().is_empty() && description_line_count < 2 {
                continue;
            }

            ctx.diagnostic(no_blank_block_descriptions_diagnostic(jsdoc.span));
        }
    }
}

/// Count block-description lines, excluding the structural newlines immediately after `/**`
/// and before either the first tag or `*/`.
fn count_description_lines(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }

    // `split` preserves the trailing empty item needed to distinguish a structural newline.
    let mut count = content.split('\n').count();

    if content.starts_with('\n') {
        count -= 1;
    }

    if content.ends_with('\n') {
        count -= 1;
    }

    count
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "/**\n * Non-empty description\n * @param {number} x\n */",
        "/**\n * Non-empty description\n *\n */",
        "/**\n * @param {number} x\n */",
        "/**\n *\n */",
        "/**\n */",
        "/** */",
        "/** Some desc. */",
        "/** @someTag */",
    ];

    let fail = vec!["/**\n *\n * @param {number} x\n */", "/**\n *\n *\n */"];

    Tester::new(NoBlankBlockDescriptions::NAME, NoBlankBlockDescriptions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
