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

fn require_template_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("@template should have a description")
        .with_help("Describe what the template type parameter represents.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireTemplateDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a description for `@template` tags.
    ///
    /// ### Why is this bad?
    ///
    /// A description explains how the generic type parameter is used by the API.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /** @template T */
    /// function identity(value) { return value; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @template T The type of the value. */
    /// function identity(value) { return value; }
    /// ```
    RequireTemplateDescription,
    jsdoc,
    style,
    version = "next",
    short_description = "Requires a description for `@template` tags.",
);

impl Rule for RequireTemplateDescription {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        for comment in ctx.semantic().comments() {
            if !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal) {
                continue;
            }

            let span = Span::new(comment.span.start + 3, comment.span.end - 2);
            let content = ctx.source_range(span);
            if !content.contains("@template") {
                continue;
            }

            let jsdoc = JSDoc::new(content, span);
            if should_ignore_as_internal(&jsdoc, settings)
                || should_ignore_as_private(&jsdoc, settings)
            {
                continue;
            }

            if jsdoc.tags().iter().any(|tag| {
                tag.kind.parsed() == "template"
                    && !has_template_description(&tag.parsed_type_comment_preserving_lines())
            }) {
                ctx.diagnostic(require_template_description_diagnostic(comment.span));
            }
        }
    }
}

fn has_template_description(body: &str) -> bool {
    let mut lines = body.lines();
    let first_line = lines.next().unwrap_or_default().trim_start();
    if lines.any(|line| !line.trim().is_empty()) {
        return true;
    }

    let first_line = if first_line.starts_with('[') {
        first_line.rfind(']').map_or(first_line, |end| &first_line[end..])
    } else {
        first_line
    };

    let mut previous = ' ';
    for (offset, character) in first_line.char_indices() {
        if character.is_whitespace() && previous != ',' && !previous.is_whitespace() {
            return !first_line[offset..].trim().is_empty();
        }
        previous = character;
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "/** @template T, U - Generic types. */",
        "/** @template [T = string[]] Defaultable type. */",
        "/** @template {object} T The object type. */",
        "/**\n * @template T\n * Generic type on a separate line.\n */",
        "/**\n * @template\n * Description on the following line.\n */",
        "function example() {} /* @template T */",
        "\n        /**\n         * @template {SomeType} Has a description\n         */\n      ",
        "\n        /**\n         * @template Has a description\n         */\n      ",
    ];

    let fail = vec![
        "/** @template */",
        "/** @template T */",
        "/** @template T, U */",
        "/** @template [T = string[]] */",
        "/** @template {object} T */",
        "/**\n * @template\n * {object}\n */",
        "/**\n * @template T\n * @template U\n */",
        "function example() {} /** @template T */",
        "\n        /**\n         * @template {SomeType}\n         */\n      ",
    ];

    Tester::new(RequireTemplateDescription::NAME, RequireTemplateDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}
