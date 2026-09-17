use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::JSDoc;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
fn multiline_blocks_diagnostic(span: Span, message: &'static str) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_label(span)
}

/**
 * Diagnostic messages, kept verbatim from `eslint-plugin-jsdoc`
 * so wording and the branch that selects each one match the upstream rule.
 */
const SINGLE_LINE_NOT_PERMITTED: &str =
    "Single line blocks are not permitted by your configuration.";

const ZERO_LINE_TEXT: &str = "Should have no text on the \"0th\" line (after the `/**`).";
const FINAL_LINE_TEXT: &str = "Should have no text on the final line (before the `*/`).";
const MULTILINE_PROHIBITED: &str = "Multiline JSDoc blocks are prohibited by your configuration.";
const MULTILINE_PROHIBITED_SINGLE_LINE_CONFLICT: &str = "Multiline JSDoc blocks are prohibited by your configuration but fixing would result in a single line block which you have prohibited with `noSingleLineBlocks`.";
const MULTILINE_PROHIBITED_MULTIPLE_TAGS: &str =
    "Multiline JSDoc blocks are prohibited by your configuration but the block has multiple tags.";

const MULTILINE_PROHIBITED_DESCRIPTION_WITH_TAG: &str = "Multiline JSDoc blocks are prohibited by your configuration but the block has a description with a tag.";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MultilineBlocks(Box<MultilineBlocksConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Controls how and whether JSDoc blocks may be expressed as single or
    /// multiple line blocks.
    ///
    /// ### Why is this bad?
    ///
    /// Teams often want a consistent shape for their JSDoc comments. They
    /// might want to always spread a block over multiple lines, or keep
    /// short blocks on a single line.
    ///
    /// This rule enforces whichever convention is configured.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with `{ "noSingleLineBlocks": true }`:
    /// ```javascript
    /// /** @tag */
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "noSingleLineBlocks": true }`:
    /// ```javascript
    /// /**
    ///  * @tag
    ///  */
    /// ```
    MultilineBlocks,
    jsdoc,
    restriction,
    pending,
    config = MultilineBlocksConfig,
    version = "next",
    short_description = "Controls how and whether JSDoc blocks can be expressed as single or multiple line blocks.",
);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct MultilineBlocksConfig {
    /// If `true`, any single line block is reported, except one whose leading
    /// tag is listed in `singleLineTags`.
    no_single_line_blocks: bool,

    /// Tags nevertheless allowed as single line blocks when `noSingleLineBlocks`
    /// is set. `"*"` allows any block that carries a tag.
    single_line_tags: Vec<String>,

    /// If `true`, multiline blocks are reported unless permitted by
    /// `minimumLengthForMultiline`, `multilineTags`, or `allowMultipleTags`.
    no_multiline_blocks: bool,

    /// When `noMultilineBlocks` is set, a multiline block is permitted if its
    /// block description is at least this many characters long. When unset the
    /// length never permits a multiline block.
    minimum_length_for_multiline: usize,

    /// When `noMultilineBlocks` is set, the presence of one of these tags
    /// permits a multiline block. `"*"` permits any tag.
    multiline_tags: Vec<String>,

    /// When `noMultilineBlocks` is set, do not report a multiline block that
    /// holds multiple tags (or a description together with a tag), since such a
    /// block cannot be collapsed onto a single line.
    allow_multiple_tags: bool,

    /// For multiline blocks, report any non-whitespace text preceding the `*/`
    /// on the final line.
    no_final_line_text: bool,

    /// For multiline blocks, report any non-whitespace text immediately after
    /// the `/**` on the first line.
    no_zero_line_text: bool,
}

impl Default for MultilineBlocksConfig {
    fn default() -> Self {
        Self {
            no_single_line_blocks: false,
            single_line_tags: vec!["lends".to_string(), "type".to_string()],
            no_multiline_blocks: false,
            // `Number.POSITIVE_INFINITY` upstream: the length gate never fires.
            minimum_length_for_multiline: usize::MAX,
            multiline_tags: vec!["*".to_string()],
            allow_multiple_tags: true,
            no_final_line_text: true,
            no_zero_line_text: true,
        }
    }
}

impl Rule for MultilineBlocks {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        for jsdoc in ctx.jsdoc().iter_all() {
            self.0.check(jsdoc, ctx);
        }
    }
}

impl MultilineBlocksConfig {
    fn check(&self, jsdoc: &JSDoc, ctx: &LintContext) {
        let span = jsdoc.span;
        let content = ctx.source_range(span);

        // `span` covers the text between `/**` and `*/` (delimiters excluded), so
        // the block spans a single physical line exactly when `content` has no
        // newline. `/**` occupies the 3 bytes before `span.start`, `*/` the 2 after
        // `span.end`.
        if !content.contains('\n') {
            // A single line block can only be reported by `noSingleLineBlocks`, so
            // do no work otherwise. The leading tag name matters only when the
            // block opens with a tag (`content` begins with `@`), not a description.
            if self.no_single_line_blocks {
                let tag_name = if content.trim_start().starts_with('@') {
                    jsdoc.tags().first().map(|tag| tag.kind.parsed())
                } else {
                    None
                };
                if self.is_invalid_single_line(tag_name) {
                    let block_span = Span::new(span.start.saturating_sub(3), span.end + 2);
                    ctx.diagnostic(multiline_blocks_diagnostic(
                        block_span,
                        SINGLE_LINE_NOT_PERMITTED,
                    ));
                }
            }
            return;
        }

        if self.no_multiline_blocks {
            let tags = jsdoc.tags();
            let description = jsdoc.comment().parsed();
            let opener_span = Span::new(span.start.saturating_sub(3), span.start);

            let permitted_by_tag = !tags.is_empty()
                && (self.multiline_tags.iter().any(|tag| tag == "*")
                    || tags.iter().any(|tag| {
                        self.multiline_tags.iter().any(|allowed| allowed == tag.kind.parsed())
                    }));
            // A permitted multiline block still gets the line checks below.
            if permitted_by_tag || description.chars().count() >= self.minimum_length_for_multiline
            {
                self.line_checks(jsdoc, ctx);
                return;
            }

            // Collapsing to a single line would produce a block that
            // `noSingleLineBlocks` itself forbids, so neither shape is allowed.
            if self.no_single_line_blocks {
                let collapses_to_valid_single_line =
                    tags.iter().any(|tag| !self.is_invalid_single_line(Some(tag.kind.parsed())));
                if !collapses_to_valid_single_line {
                    ctx.diagnostic(multiline_blocks_diagnostic(
                        opener_span,
                        MULTILINE_PROHIBITED_SINGLE_LINE_CONFLICT,
                    ));
                    return;
                }
            }

            if tags.len() > 1 {
                if !self.allow_multiple_tags {
                    ctx.diagnostic(multiline_blocks_diagnostic(
                        opener_span,
                        MULTILINE_PROHIBITED_MULTIPLE_TAGS,
                    ));
                    return;
                }
            } else if tags.len() == 1 && !description.is_empty() {
                if !self.allow_multiple_tags {
                    ctx.diagnostic(multiline_blocks_diagnostic(
                        opener_span,
                        MULTILINE_PROHIBITED_DESCRIPTION_WITH_TAG,
                    ));
                    return;
                }
            } else {
                ctx.diagnostic(multiline_blocks_diagnostic(opener_span, MULTILINE_PROHIBITED));
                return;
            }
        }

        self.line_checks(jsdoc, ctx);
    }

    fn is_invalid_single_line(&self, tag_name: Option<&str>) -> bool {
        self.no_single_line_blocks
            && match tag_name {
                None => true,
                Some(name) => {
                    !self.single_line_tags.iter().any(|tag| tag == name)
                        && !self.single_line_tags.iter().any(|tag| tag == "*")
                }
            }
    }

    #[expect(clippy::cast_possible_truncation)]
    fn line_checks(&self, jsdoc: &JSDoc, ctx: &LintContext) {
        let span = jsdoc.span;
        let content = ctx.source_range(span);

        // 0th line: text after `/**`, up to the first newline.
        let first_line = content.split('\n').next().unwrap_or("");
        if self.no_zero_line_text {
            let text = first_line.trim();
            if !text.is_empty() {
                let offset = (first_line.len() - first_line.trim_start().len()) as u32;
                let start = span.start + offset;
                ctx.diagnostic(multiline_blocks_diagnostic(
                    Span::new(start, start + text.len() as u32),
                    ZERO_LINE_TEXT,
                ));
                return;
            }
        }

        // Final line: text before `*/`, after the last newline and any `*` prefix.
        let last_line = content.rsplit('\n').next().unwrap_or("");

        if self.no_final_line_text {
            let without_star = {
                let trimmed = last_line.trim_start();
                trimmed.strip_prefix('*').unwrap_or(trimmed)
            };
            let text = without_star.trim();

            if !text.is_empty() {
                let line_start = span.end.saturating_sub(last_line.len() as u32);
                // When a tag opens on the final line, only its trailing description
                // counts - a bare tag (`* @tag */`) is not "text", matching how the
                // reference reads the final line's `description` token.
                let has_description =
                    match jsdoc.tags().iter().rev().find(|tag| tag.kind.span.start >= line_start) {
                        Some(tag) => !tag.type_name_comment().2.parsed().is_empty(),
                        None => true,
                    };
                if has_description {
                    let offset = (last_line.len() - without_star.trim_start().len()) as u32;
                    let start = line_start + offset;
                    ctx.diagnostic(multiline_blocks_diagnostic(
                        Span::new(start, start + text.len() as u32),
                        FINAL_LINE_TEXT,
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Test cases ported from `eslint-plugin-jsdoc`'s `multiline-blocks` suite.
    // The `requireSingleLineUnderCount` option is intentionally not yet
    // implemented (it depends on per-line token widths that the JSDoc parser
    // does not expose), so its cases are omitted here.
    let pass = vec![
        ("/** Not reported */", None, None),
        ("/**\n *  Not reported\n */", None, None),
        (
            "/** Reported up here\n * because the rest is multiline\n */",
            Some(serde_json::json!([{ "noZeroLineText": false }])),
            None,
        ),
        ("/** @tag */", None, None),
        ("/** @lends */", Some(serde_json::json!([{ "noSingleLineBlocks": true }])), None),
        (
            "/** @tag */",
            Some(serde_json::json!([{ "noSingleLineBlocks": true, "singleLineTags": ["tag"] }])),
            None,
        ),
        (
            "/** @tag */",
            Some(serde_json::json!([{ "noSingleLineBlocks": true, "singleLineTags": ["*"] }])),
            None,
        ),
        ("/**\n *\n */", None, None),
        ("/**\n *\n */", Some(serde_json::json!([{ "noMultilineBlocks": false }])), None),
        ("/** Test */", Some(serde_json::json!([{ "noMultilineBlocks": true }])), None),
        (
            "/**\n * This is long enough to be permitted by our config.\n */",
            Some(
                serde_json::json!([{ "minimumLengthForMultiline": 25, "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This is long enough to be permitted by our config.\n */",
            Some(
                serde_json::json!([{ "minimumLengthForMultiline": 50, "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has the right tag so is permitted.\n * @theRightTag\n */",
            Some(
                serde_json::json!([{ "multilineTags": ["theRightTag"], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/** This has no tags but is single line so is not permitted. */",
            Some(serde_json::json!([{ "multilineTags": ["*"], "noMultilineBlocks": true }])),
            None,
        ),
        (
            "/**\n * This has the wrong tags so is not permitted.\n * @notTheRightTag\n */",
            Some(
                serde_json::json!([{ "minimumLengthForMultiline": 10, "multilineTags": ["onlyThisIsExempted"], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has the wrong tags so is not permitted.\n * @theRightTag\n */",
            Some(
                serde_json::json!([{ "minimumLengthForMultiline": 500, "multilineTags": ["theRightTag"], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        ("/** tag */", None, None),
        (
            "/**\n * @lends This is ok per multiline\n */",
            Some(serde_json::json!([{ "noMultilineBlocks": true, "noSingleLineBlocks": true }])),
            None,
        ),
        (
            "/**\n * This has too many tags so cannot be fixed to a single line.\n * @oneTag\n * @anotherTag\n */",
            Some(serde_json::json!([{ "multilineTags": [], "noMultilineBlocks": true }])),
            None,
        ),
        (
            "/**\n * This has too many tags so cannot be fixed to a single line.\n * @oneTag\n * @anotherTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": true, "multilineTags": [], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has a tag and description so cannot be fixed to a single line.\n * @oneTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": true, "multilineTags": [], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has a tag and description so cannot be fixed to a single line.\n * @oneTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": false, "multilineTags": ["oneTag"], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/** @someTag with Description */",
            Some(serde_json::json!([{ "noFinalLineText": true }])),
            None,
        ),
        // A bare tag on the final line is not "final line text" (only a trailing
        // description is). Regression guard from a differential test against
        // `eslint-plugin-jsdoc`.
        ("/**\n * text\n * @tag */", Some(serde_json::json!([{ "noFinalLineText": true }])), None),
    ];

    let fail = vec![
        ("/** Reported up here\n * because the rest is multiline\n */", None, None),
        (
            "/** Reported up here\n * because the rest is multiline\n */",
            Some(serde_json::json!([{ "noZeroLineText": true }])),
            None,
        ),
        (
            "/** @abc {aType} aName Reported up here\n * because the rest is multiline\n */",
            Some(serde_json::json!([{ "noZeroLineText": true }])),
            None,
        ),
        ("/** @tag */", Some(serde_json::json!([{ "noSingleLineBlocks": true }])), None),
        ("/** @tag {someType} */", Some(serde_json::json!([{ "noSingleLineBlocks": true }])), None),
        (
            "/** @tag {someType} aName */",
            Some(serde_json::json!([{ "noSingleLineBlocks": true }])),
            None,
        ),
        (
            "/** @tag */",
            Some(
                serde_json::json!([{ "noSingleLineBlocks": true, "singleLineTags": ["someOtherTag"] }]),
            ),
            None,
        ),
        (
            "/** desc */",
            Some(serde_json::json!([{ "noSingleLineBlocks": true, "singleLineTags": ["*"] }])),
            None,
        ),
        ("/**\n * Desc.\n */", Some(serde_json::json!([{ "noMultilineBlocks": true }])), None),
        ("/** desc\n *\n */", Some(serde_json::json!([{ "noMultilineBlocks": true }])), None),
        (
            "/** desc\n *\n */",
            Some(serde_json::json!([{ "noMultilineBlocks": true, "noSingleLineBlocks": true }])),
            None,
        ),
        ("/**\n *\n */", Some(serde_json::json!([{ "noMultilineBlocks": true }])), None),
        (
            "/**\n * This is not long enough to be permitted.\n */",
            Some(
                serde_json::json!([{ "minimumLengthForMultiline": 100, "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This is not long enough to be permitted.\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": true, "minimumLengthForMultiline": 100, "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has the wrong tags so is not permitted.\n * @notTheRightTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": false, "multilineTags": ["onlyThisIsExempted"], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has too many tags so cannot be fixed to a single line.\n * @oneTag\n * @anotherTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": false, "multilineTags": [], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has a tag and description so cannot be fixed to a single line.\n * @oneTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": false, "multilineTags": [], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * This has no tags so is not permitted.\n */",
            Some(serde_json::json!([{ "multilineTags": ["*"], "noMultilineBlocks": true }])),
            None,
        ),
        (
            "/**\n * This has the wrong tags so is not permitted.\n * @notTheRightTag\n */",
            Some(
                serde_json::json!([{ "allowMultipleTags": false, "minimumLengthForMultiline": 500, "multilineTags": ["onlyThisIsExempted"], "noMultilineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * @lends This can be safely fixed to a single line.\n */",
            Some(
                serde_json::json!([{ "multilineTags": [], "noMultilineBlocks": true, "noSingleLineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * @type {aType} This can be safely fixed to a single line.\n */",
            Some(
                serde_json::json!([{ "multilineTags": [], "noMultilineBlocks": true, "noSingleLineBlocks": true }]),
            ),
            None,
        ),
        (
            "/**\n * @aTag\n */",
            Some(serde_json::json!([{ "multilineTags": [], "noMultilineBlocks": true }])),
            None,
        ),
        (
            "/**\n * This is a problem when single and multiline are blocked.\n */",
            Some(serde_json::json!([{ "noMultilineBlocks": true, "noSingleLineBlocks": true }])),
            None,
        ),
        (
            "/** This comment is bad\n * It should not have text on line zero\n */",
            Some(
                serde_json::json!([{ "minimumLengthForMultiline": 50, "noMultilineBlocks": true, "noZeroLineText": true }]),
            ),
            None,
        ),
        (
            "/**\n * @lends This can be safely fixed\n * to a single\n * line. */",
            Some(serde_json::json!([{ "multilineTags": [], "noMultilineBlocks": true }])),
            None,
        ),
        (
            "/**\n * @someTag {aType} with Description */",
            Some(serde_json::json!([{ "noFinalLineText": true }])),
            None,
        ),
        ("/**\n * Description */", Some(serde_json::json!([{ "noFinalLineText": true }])), None),
    ];

    Tester::new(MultilineBlocks::NAME, MultilineBlocks::PLUGIN, pass, fail).test_and_snapshot();
}
