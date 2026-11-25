use oxc_ast::ast::CommentContent;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RequireAsteriskPrefix(Box<RequireAsteriskPrefixConfig>);

/// The default prefix style and optional overrides for individual tags.
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default)]
struct RequireAsteriskPrefixConfig(AsteriskPrefixStyle, AsteriskPrefixOptions);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum AsteriskPrefixStyle {
    #[default]
    Always,
    Never,
    Any,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct AsteriskPrefixOptions {
    /// Override the default for a tag (without `@`) or the block's `*description`.
    tags: AsteriskPrefixTags,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct AsteriskPrefixTags {
    /// Require an asterisk on the lines of these tags.
    always: Vec<String>,
    /// Forbid an asterisk on the lines of these tags.
    never: Vec<String>,
    /// Allow either style on the lines of these tags.
    any: Vec<String>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires each line inside a JSDoc block to start with an asterisk by default.
    /// Set the first option to `"never"` to forbid prefixes or `"any"` to allow either style.
    /// The `tags` option can override the style for individual tags and the block description.
    ///
    /// ### Why is this bad?
    ///
    /// A consistent prefix makes documentation comments easier to read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /**
    ///  @param name The user's name.
    ///  */
    /// function greet(name) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /**
    ///  * @param name The user's name.
    ///  */
    /// function greet(name) {}
    /// ```
    RequireAsteriskPrefix,
    jsdoc,
    style,
    fix,
    config = RequireAsteriskPrefixConfig,
    version = "next",
    short_description = "Require a consistent asterisk prefix on JSDoc lines.",
);

impl Rule for RequireAsteriskPrefix {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        for comment in ctx.semantic().comments() {
            if !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal) {
                continue;
            }
            let span = comment.content_span().shrink_left(1);
            let body = ctx.source_range(span);
            if !body.contains('\n') || !body.starts_with(js_whitespace) {
                continue;
            }
            let jsdoc = JSDoc::new(body, span);
            let settings = &ctx.settings().jsdoc;
            if !should_ignore_as_internal(&jsdoc, settings)
                && !should_ignore_as_private(&jsdoc, settings)
            {
                self.check_block(&jsdoc, ctx);
            }
        }
    }
}

fn js_whitespace(ch: char) -> bool {
    is_white_space(ch) || is_line_terminator(ch)
}

impl RequireAsteriskPrefix {
    fn check_block(&self, jsdoc: &JSDoc, ctx: &LintContext) {
        let body = ctx.source_range(jsdoc.span);
        let mut line_start = jsdoc.span.start;
        let mut tag_name = "*description";
        let mut tag_start = None;
        let mut fenced = false;
        let mut lines = body.split('\n').peekable();

        while let Some(line) = lines.next() {
            let start = line_start;
            line_start += u32::try_from(line.len()).unwrap() + 1;
            let opening = start == jsdoc.span.start;
            let line = line.trim_end_matches('\r');
            let trimmed = line.trim_start_matches(js_whitespace);
            let content = if opening {
                trimmed
            } else {
                trimmed.strip_prefix('*').unwrap_or(trimmed).trim_start_matches(js_whitespace)
            };
            let tag = (!fenced)
                .then(|| content.strip_prefix('@'))
                .flatten()
                .and_then(|value| value.split(js_whitespace).next())
                .filter(|name| !name.is_empty() && !name.contains('/'));
            if let Some(name) = tag {
                tag_name = name;
                tag_start = Some(start + u32::try_from(line.len() - content.len()).unwrap());
            }
            if content.matches("```").count() % 2 == 1 {
                fenced = !fenced;
            }

            if opening || (lines.peek().is_none() && tag.is_none()) {
                continue;
            }
            let has_prefix = trimmed.starts_with('*');
            if !self.0.needs_fix(tag_name, has_prefix) {
                continue;
            }

            let prefix = &ctx.source_text()[..(jsdoc.span.start - 3) as usize];
            let before_opener = prefix.rsplit(is_line_terminator).next().unwrap_or_default();
            let column = before_opener.encode_utf16().count();
            let label = Span::empty(start + byte_offset_at_column(line, column));
            let indentation = line.len() - trimmed.len();
            let edit = start + u32::try_from(indentation).unwrap();
            let line_end = start + u32::try_from(line.len()).unwrap();
            let continuation = tag_start
                .filter(|&pos| pos < start)
                .and_then(|pos| jsdoc.tags().iter().find(|tag| tag.kind.span.start == pos));
            let type_range =
                continuation.and_then(|tag| continued_type(tag, Span::new(start, line_end), ctx));
            if has_prefix {
                let diagnostic =
                    OxcDiagnostic::warn("Expected JSDoc line to have no prefix.").with_label(label);
                let mut width = trimmed.len() - content.len();
                // Preserve relative indentation inside a multiline type.
                if let Some((ty, _)) = type_range
                    && ty.end > start
                {
                    let first_line = ctx.source_text()[..ty.start as usize]
                        .rsplit('\n')
                        .next()
                        .unwrap_or_default()
                        .trim_start_matches(js_whitespace);
                    let rest =
                        first_line.strip_prefix("/**").or_else(|| first_line.strip_prefix('*'));
                    let spacing = rest
                        .map_or(0, |text| text.chars().take_while(|&ch| js_whitespace(ch)).count());
                    width = 1 + byte_offset_at_column(&trimmed[1..width], spacing) as usize;
                }
                let span = Span::sized(edit, u32::try_from(width).unwrap());
                ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.delete_range(span));
            } else {
                let diagnostic = OxcDiagnostic::warn("Expected JSDoc line to have the prefix.")
                    .with_label(label);
                let add_space = match type_range {
                    Some((_, end)) => {
                        end < line_end
                            && !ctx.source_range(Span::new(end, line_end)).trim().is_empty()
                    }
                    None => !content.is_empty(),
                };
                let prefix = if add_space || content.starts_with('/') { "* " } else { "*" };
                ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    let replacement = if indentation == 0 {
                        let indent = if before_opener.starts_with('\t') { "\t" } else { " " };
                        format!("{} {prefix}", indent.repeat(column))
                    } else {
                        prefix.to_owned()
                    };
                    fixer.insert_text_before_range(Span::empty(edit), replacement)
                });
            }
            // The upstream rule reports the first offending line of each block.
            break;
        }
    }
}

impl RequireAsteriskPrefixConfig {
    fn needs_fix(&self, tag: &str, has_prefix: bool) -> bool {
        let tags = &self.1.tags;
        if tags.any.iter().any(|name| name == tag) {
            return false;
        }
        let always = tags.always.iter().any(|name| name == tag);
        let never = tags.never.iter().any(|name| name == tag);
        if has_prefix {
            never || (self.0 == AsteriskPrefixStyle::Never && !always)
        } else {
            always || (self.0 == AsteriskPrefixStyle::Always && !never)
        }
    }
}

/// Upstream anchors the diagnostic at the opening delimiter's column on the offending line.
fn byte_offset_at_column(line: &str, column: usize) -> u32 {
    let mut at = 0;
    for (index, ch) in line.char_indices() {
        at += ch.len_utf16();
        if at > column {
            return u32::try_from(index).unwrap();
        }
    }
    u32::try_from(line.len()).unwrap()
}

/// A continued type or parameter name is not part of the description; the fixer
/// adds a separator only when the line actually contains description text.
fn continued_type(tag: &JSDocTag, line: Span, ctx: &LintContext) -> Option<(Span, u32)> {
    if matches!(
        tag.kind.parsed(),
        "default"
            | "defaultvalue"
            | "description"
            | "example"
            | "file"
            | "fileoverview"
            | "license"
            | "overview"
            | "see"
            | "summary"
    ) {
        return None;
    }
    let raw = ctx.source_range(tag.comment().span);
    let horizontal_space = |ch| js_whitespace(ch) && ch != '\n';
    let mut body = raw.trim_start_matches(horizontal_space);
    while let Some(next) = body.strip_prefix('\n') {
        body = next.trim_start_matches(horizontal_space);
        body = body.strip_prefix('*').unwrap_or(body).trim_start_matches(horizontal_space);
    }
    let span = Span::new(tag.span.end - u32::try_from(body.len()).unwrap(), tag.span.end);
    let tag = JSDocTag::new(tag.kind, body, span);
    let (ty, name, _) = tag.type_name_comment();
    let ty = ty?;
    let end = if matches!(
        tag.kind.parsed(),
        "access"
            | "author"
            | "exception"
            | "kind"
            | "return"
            | "returns"
            | "since"
            | "throws"
            | "version"
            | "variation"
    ) {
        ty.span.end
    } else {
        name.map_or(ty.span.end, |name| name.span.end)
    };
    (ty.span.start < line.end && end > line.start).then_some((ty.span, end))
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let param_no_prefix = "

        /**
         @param {Number} foo
         */
        function quux (foo) {
          // with spaces
        }
      ";
    let param_with_prefix = "

        /**
         * @param {Number} foo
         */
        function quux (foo) {
          // with spaces
        }
      ";
    let description_with_prefix = "

        /**
         * Desc
         *
         */
        function quux (foo) {
          // with spaces
        }
      ";
    let description_no_prefix = "

        /**
         * Desc

         */
        function quux (foo) {
          // with spaces
        }
      ";
    let inline_closing_no_prefix = "
        /**
          @param {Number} foo
         */function quux (foo) {
          // with spaces
        }
      ";

    let pass = vec![
        (
            "
        /**
         * Desc
         *
         * @param {Number} foo
         *   This is more comment.
         */
        function quux (foo) {

        }
      ",
            None,
        ),
        (
            "
        /**
         * Desc
         *
         * @param {{
         * foo: Bar,
         * bar: Baz
         * }} foo
         *
         */
        function quux (foo) {

        }
      ",
            None,
        ),
        (
            "
        /*  <- JSDoc must start with 2 stars.
         So this is unchecked.
         */
        function quux (foo) {}
      ",
            None,
        ),
        (
            "
        /** @param {Number} foo */
        function quux (foo) {
          // with spaces
        }
      ",
            None,
        ),
        (param_no_prefix, Some(json!(["always", { "tags": { "any": ["param"] } }]))),
        (param_with_prefix, Some(json!(["never", { "tags": { "always": ["param"] } }]))),
        (param_with_prefix, Some(json!(["always", { "tags": { "never": ["someOtherTag"] } }]))),
        (param_no_prefix, Some(json!(["always", { "tags": { "never": ["param"] } }]))),
        (param_no_prefix, Some(json!(["never", { "tags": { "always": ["someOtherTag"] } }]))),
        (description_with_prefix, Some(json!(["never", { "tags": { "any": ["*description"] } }]))),
        (description_no_prefix, Some(json!(["always", { "tags": { "any": ["*description"] } }]))),
        (param_no_prefix, Some(json!(["any", { "tags": { "always": ["someOtherTag"] } }]))),
        ("/**@tag\nmissing\n */", None),
        (
            "/**\n * ```js\n @param fake\n * ```\n tail\n */",
            Some(json!(["any", {"tags": {"always": ["param"]}}])),
        ),
    ];

    let fix = vec![
        (param_no_prefix, param_with_prefix, None),
        (
            param_no_prefix,
            param_with_prefix,
            Some(json!(["any", { "tags": { "always": ["param"] } }])),
        ),
        (description_no_prefix, description_with_prefix, None),
        (
            "

        /**
         *
         Desc
         */
        function quux (foo) {
          // with spaces
        }
      ",
            "

        /**
         *
         * Desc
         */
        function quux (foo) {
          // with spaces
        }
      ",
            None,
        ),
        (
            description_with_prefix,
            "

        /**
         Desc
         *
         */
        function quux (foo) {
          // with spaces
        }
      ",
            Some(json!(["never"])),
        ),
        (
            param_no_prefix,
            param_with_prefix,
            Some(json!(["always", { "tags": { "any": ["someOtherTag"] } }])),
        ),
        (
            param_with_prefix,
            param_no_prefix,
            Some(json!(["never", { "tags": { "always": ["someOtherTag"] } }])),
        ),
        (
            param_with_prefix,
            param_no_prefix,
            Some(json!(["always", { "tags": { "never": ["param"] } }])),
        ),
        (
            param_no_prefix,
            param_with_prefix,
            Some(json!(["never", { "tags": { "always": ["param"] } }])),
        ),
        (
            inline_closing_no_prefix,
            "
        /**
          * @param {Number} foo
         */function quux (foo) {
          // with spaces
        }
      ",
            None,
        ),
        (param_with_prefix, param_no_prefix, Some(json!(["never"]))),
        (
            "
        /**
          *@param {Number} foo
         */function quux (foo) {
          // with spaces
        }
      ",
            inline_closing_no_prefix,
            Some(json!(["never"])),
        ),
        ("/**\r\n\ttext\r\n */", "/**\r\n\t* text\r\n */", None),
        (
            "const x = 1; /**\n @param name Description.\n */",
            "const x = 1; /**\n * @param name Description.\n */",
            None,
        ),
        ("/**\n@param */", "/**\n * @param */", None),
        ("/**\nfirst\nsecond\n */", "/**\n * first\nsecond\n */", None),
        (
            "/**\n * @param {{\n}} name Description\n */",
            "/**\n * @param {{\n * }} name Description\n */",
            None,
        ),
        (
            "/**\n * @param {{\n     x: string,\n * }} foo\n */",
            "/**\n * @param {{\n     *x: string,\n * }} foo\n */",
            None,
        ),
        (
            "/**\n @param {{\n *    x: number,\n }} foo\n */",
            "/**\n @param {{\n     x: number,\n }} foo\n */",
            Some(json!(["never"])),
        ),
        (
            "/**\n * @param\n * {{\nbar: Baz\n * }} foo\n */",
            "/**\n * @param\n * {{\n *bar: Baz\n * }} foo\n */",
            None,
        ),
        (
            "/** @param {{\n *\u{3000}x: number,\n }} options\n */",
            "/** @param {{\n x: number,\n }} options\n */",
            Some(json!(["never"])),
        ),
    ];
    let fail = fix.iter().map(|(source, _, options)| (*source, options.clone())).collect();
    Tester::new(RequireAsteriskPrefix::NAME, RequireAsteriskPrefix::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
