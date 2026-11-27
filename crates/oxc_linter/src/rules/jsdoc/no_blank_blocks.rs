use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::ast::CommentContent;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_blank_blocks_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("No empty blocks").with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoBlankBlocks {
    /// Whether to automatically remove blank JSDoc blocks.
    enable_fixer: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports and optionally removes blocks with whitespace only.
    ///
    /// ### Why is this bad?
    ///
    /// Blank JSDoc blocks add noise without providing any documentation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /** */
    ///
    /// /**
    ///  *
    ///  */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @tag */
    ///
    /// /**
    ///  * Text
    ///  */
    ///
    /// /**
    ///  * @tag
    ///  */
    /// ```
    NoBlankBlocks,
    jsdoc,
    style,
    conditional_fix,
    config = NoBlankBlocks,
    version = "1.78.0",
    short_description = "Reports and optionally removes blocks with whitespace only.",
);

impl Rule for NoBlankBlocks {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let source_text = ctx.source_text();

        for comment in ctx.semantic().comments() {
            if !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal) {
                continue;
            }
            let content_span = Span::new(comment.span.start + 3, comment.span.end - 2);
            if !is_blank_jsdoc(ctx.source_range(content_span)) {
                continue;
            }

            let diagnostic = no_blank_blocks_diagnostic(content_span);
            if self.enable_fixer
                && let Some(span) = fix_span(comment.span, source_text)
            {
                ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.delete_range(span));
            } else {
                ctx.diagnostic(diagnostic);
            }
        }
    }
}

/// Returns whether the contents between `/**` and `*/` contain only whitespace and JSDoc line delimiters
fn is_blank_jsdoc(content: &str) -> bool {
    content.split('\n').enumerate().all(|(index, line)| {
        let line = line.trim();
        line.is_empty() || (index > 0 && line == "*")
    })
}

/// Expands a standalone JSDoc block to include its indentation and following line ending.
fn fix_span(comment_span: Span, source_text: &str) -> Option<Span> {
    let prefix = &source_text[..comment_span.start as usize];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);

    if !prefix[line_start..].bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
        return None;
    }

    let suffix = &source_text[comment_span.end as usize..];
    let line_end = suffix.find(['\r', '\n']).unwrap_or(suffix.len());
    if !suffix[..line_end].bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
        return None;
    }

    let line_ending_len = if suffix[line_end..].starts_with("\r\n") {
        2
    } else {
        usize::from(line_end < suffix.len())
    };

    Some(Span::new(
        u32::try_from(line_start).unwrap_or(comment_span.start),
        comment_span.end + u32::try_from(line_end + line_ending_len).unwrap_or_default(),
    ))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function example() {} /* */", None),
        ("function example() {} /** @tag */", None),
        ("function example() {} /****/", None),
        (
            "
                    /** @tag */
                  ",
            None,
        ),
        (
            "
                    /**
                     * Text
                     */
                  ",
            None,
        ),
        (
            "
                    /**
                     * @tag
                     */
                  ",
            None,
        ),
    ];

    let fail = vec![
        ("function example() {} /** */", None),
        (
            "
                    /** */
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     */
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     *
                     */
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     *
                     *
                     */
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     *
                     *
                     */
                  ",
            Some(serde_json::json!([ { "enableFixer": false, }, ])),
        ),
        (
            "
                    /**
                     *
                     *
                     */
                  ",
            None,
        ),
        ("foo();\r\n/** */\r\nbar();", Some(serde_json::json!([ { "enableFixer": true, }, ]))),
    ];

    let fix = vec![
        (
            "
                    /** */
                  ",
            "
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     */
                  ",
            "
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     *
                     */
                  ",
            "
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "
                    /**
                     *
                     *
                     */
                  ",
            "
                  ",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "foo();\r\n/** */\r\nbar();",
            "foo();\r\nbar();",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
        (
            "const result = typeof/** */value;",
            "const result = typeof/** */value;",
            Some(serde_json::json!([ { "enableFixer": true, }, ])),
        ),
    ];

    Tester::new(NoBlankBlocks::NAME, NoBlankBlocks::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
