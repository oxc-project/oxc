use oxc_ast::ast::CommentContent;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{JSDoc, JSDocTag};
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::{
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::{AlwaysNever, should_ignore_as_internal, should_ignore_as_private},
};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct RequireHyphenBeforeParamDescription(Box<RequireHyphenBeforeParamDescriptionConfig>);

/// The first option controls hyphens for parameter descriptions (default: `"always"`).
/// The second configures additional tags.
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default)]
struct RequireHyphenBeforeParamDescriptionConfig(AlwaysNever, HyphenTagOptions);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct HyphenTagOptions {
    /// Map tag names to `"always"` or `"never"`. Use `"*"` as a fallback for other tags.
    tags: Option<HyphenTags>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
enum HyphenTags {
    Overrides(BTreeMap<String, AlwaysNever>),
    Legacy(HyphenTagKeyword),
}

// Accepted by eslint-plugin-jsdoc. Only the object form configures additional tags.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum HyphenTagKeyword {
    Any,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a hyphen before a JSDoc parameter description by default.
    /// Set the first option to `"never"` to disallow the hyphen.
    ///
    /// ### Why is this bad?
    ///
    /// A consistent separator makes parameter names and their descriptions easier to distinguish.
    /// A hyphen on a line by itself can render as an empty list item.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /** @param {string} name The user's name. */
    /// function greet(name) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @param {string} name - The user's name. */
    /// function greet(name) {}
    /// ```
    RequireHyphenBeforeParamDescription,
    jsdoc,
    style,
    conditional_fix,
    config = RequireHyphenBeforeParamDescriptionConfig,
    version = "next",
    short_description = "Require or disallow a hyphen before JSDoc parameter descriptions.",
);

impl Rule for RequireHyphenBeforeParamDescription {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let param = settings.resolve_tag_name("param");
        let overrides = match &self.0.1.tags {
            Some(HyphenTags::Overrides(tags)) => Some(tags),
            None | Some(HyphenTags::Legacy(HyphenTagKeyword::Any)) => None,
        };
        let marker = if overrides.is_some() { "@" } else { param };
        for comment in ctx.semantic().comments() {
            if !matches!(comment.content, CommentContent::Jsdoc | CommentContent::JsdocLegal) {
                continue;
            }
            // Include trailing JSDoc blocks, which do not have an associated AST node.
            let span = Span::new(comment.span.start + 3, comment.span.end - 2);
            let source = ctx.source_range(span);
            if !source.starts_with(char::is_whitespace) || !source.contains(marker) {
                continue;
            }
            let jsdoc = JSDoc::new(source, span);
            if should_ignore_as_internal(&jsdoc, settings)
                || should_ignore_as_private(&jsdoc, settings)
            {
                continue;
            }
            for (index, tag) in jsdoc.tags().iter().enumerate() {
                let name = tag.kind.parsed();
                let mode = if name == param {
                    &self.0.0
                } else if let Some(tags) = overrides
                    && settings.preferred_tag_name(name) == name
                    && let Some(mode) = tags
                        .get(name)
                        .or_else(|| {
                            tags.iter().find_map(|(key, mode)| {
                                (settings.preferred_tag_name(key) == name).then_some(mode)
                            })
                        })
                        .or_else(|| tags.get("*"))
                {
                    mode
                } else {
                    continue;
                };
                if let Some(reason) =
                    settings.blocked_tag_message(name, || format!("Unexpected tag `@{name}`"))
                {
                    if !jsdoc.tags()[..index].iter().any(|other| other.kind.parsed() == name) {
                        ctx.diagnostic(
                            OxcDiagnostic::warn(reason.into_owned()).with_label(tag.kind.span),
                        );
                    }
                    continue;
                }
                check_hyphen(tag, mode, ctx);
            }
        }
    }
}

/// Skip leading whitespace and the decoration on continuation lines, retaining a slice
/// of the original text so edits keep the rest of the comment byte-for-byte.
fn first_content(mut text: &str) -> &str {
    loop {
        text = text.trim_start_matches(|c: char| c.is_whitespace() && c != '\n');
        let Some(next) = text.strip_prefix('\n') else { return text };
        text = next.trim_start_matches(|c: char| c.is_whitespace() && c != '\n');
        text = text.strip_prefix('*').unwrap_or(text);
    }
}

fn template_name_len(line: &str) -> usize {
    // A comma and subsequent whitespace separate names within one @template tag.
    // A bracketed name can include a default type containing spaces and square brackets.
    let start = if line.starts_with('[') { line.rfind(']').unwrap_or(0) } else { 0 };
    let mut previous = ' ';
    for (offset, ch) in line[start..].char_indices() {
        if ch.is_whitespace() && previous != ',' && !previous.is_whitespace() {
            return start + offset;
        }
        previous = ch;
    }
    line.len()
}

fn description<'a>(tag: &JSDocTag, ctx: &LintContext<'a>) -> Option<(&'a str, bool)> {
    let name = tag.kind.parsed();
    let body = ctx.source_range(tag.comment().span);
    let content = first_content(body);
    if content.is_empty() {
        return None;
    }
    // eslint-plugin-jsdoc tokenizes these tags as free-form descriptions.
    let description_only = matches!(
        name,
        "default"
            | "defaultvalue"
            | "description"
            | "example"
            | "file"
            | "fileoverview"
            | "license"
            | "overview"
            | "summary"
    );
    let no_type = description_only || name == "see";
    let no_name = description_only
        || matches!(
            name,
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
        )
        || (name == "see"
            && body
                .lines()
                .next()
                .is_some_and(|line| lazy_regex::regex_is_match!(r"\{@link.+?\}", line)));

    let body_span = Span::new(tag.span.end - u32::try_from(content.len()).unwrap(), tag.span.end);
    let trimmed_tag = JSDocTag::new(tag.kind, content, body_span);
    let ty = (!no_type && content.starts_with('{')).then(|| trimmed_tag.r#type()).flatten();
    let has_type = ty.is_some();
    let rest = ty.map_or(body, |ty| ctx.source_range(Span::new(ty.span.end, tag.span.end)));
    let name_and_description = if has_type {
        first_content(rest)
    } else {
        rest.trim_start_matches(|c: char| c.is_whitespace() && c != '\n')
    };
    let first_line = name_and_description.split('\n').next().unwrap_or_default();
    let name_len = if no_name || first_line.trim().is_empty() {
        0
    } else if name == "template" {
        template_name_len(first_line)
    } else if let Some(end_quote) = first_line.strip_prefix('"').and_then(|line| line.find('"')) {
        end_quote + 2
    } else {
        // Whitespace is part of the name inside optional parameters and property paths
        // such as `[value = default]` and `options["property name"]`.
        let mut brackets = 0i32;
        first_line
            .char_indices()
            .find_map(|(index, ch)| {
                match ch {
                    '[' => brackets += 1,
                    ']' => brackets -= 1,
                    c if brackets == 0 && c.is_whitespace() => return Some(index),
                    _ => {}
                }
                None
            })
            .unwrap_or(first_line.len())
    };
    Some((&name_and_description[name_len..], name_len > 0))
}

fn check_hyphen(tag: &JSDocTag, mode: &AlwaysNever, ctx: &LintContext) {
    let Some((after_name, has_name)) = description(tag, ctx) else { return };
    let text = first_content(after_name);
    if text.is_empty() {
        return;
    }
    let before = &after_name[..after_name.len() - text.len()];
    let start = tag.span.end - u32::try_from(text.len()).unwrap();
    let name = tag.kind.parsed();
    // A hyphen at the end of the tag is allowed. The special case is a bare hyphen
    // immediately followed by a newline within the description.
    let hyphen_newline = text
        .strip_prefix("-\n")
        .is_some_and(|rest| rest.contains('\n') || !first_content(&text[1..]).is_empty());
    if let Some(rest) = text.strip_prefix('-') {
        if *mode == AlwaysNever::Always && !hyphen_newline {
            return;
        }
        let end = rest.find(['\n', '\r']).unwrap_or(rest.len());
        let spaces = end - rest[..end].trim_start().len();
        let remove_start = if hyphen_newline && has_name && !before.contains('\n') {
            start - u32::try_from(before.len()).unwrap()
        } else {
            start
        };
        let message = if *mode == AlwaysNever::Always {
            format!("There must be no hyphen followed by newline after the @{name} name.")
        } else {
            format!("There must be no hyphen before @{name} description.")
        };
        let edit = Span::new(remove_start, start + 1 + u32::try_from(spaces).unwrap());
        let diagnostic = OxcDiagnostic::warn(message).with_label(Span::sized(start, 1));
        ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.delete_range(edit));
    } else if *mode == AlwaysNever::Always && !(has_name && before.contains('\n')) {
        let word_len = text.find(char::is_whitespace).unwrap_or(text.len());
        let label = Span::sized(start, u32::try_from(word_len).unwrap());
        let diagnostic =
            OxcDiagnostic::warn(format!("There must be a hyphen before @{name} description."))
                .with_label(label);
        ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.insert_text_before_range(label, "- "));
    }
}
#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let param_with_hyphen = "
          /**
           * @param foo - Foo.
           */
          function quux () {

          }
      ";
    let param_without_hyphen = "
          /**
           * @param foo Foo.
           */
          function quux () {

          }
      ";
    let property_with_hyphen = "
          /**
           * @typedef {SomeType} ATypeDefName
           * @property foo - Foo.
           */
      ";
    let property_without_hyphen = "
          /**
           * @typedef {SomeType} ATypeDefName
           * @property foo Foo.
           */
      ";
    let multiline_description = r#"
        /**
         * @param foo
         * The possible values for `foo` are as follows.
         * - `"option1"`: Description of option 1.
         * - `"option2"`: Description of option 2.
         */
      "#;
    let disabled_param_tag = "
          /**
           * @param foo
           */
          function quux (foo) {

          }
      ";

    let pass = vec![
        ("/** @returns Returns a result. */", Some(json!(["always", {"tags": "any"}]))),
        ("/**@param name A comment without JSDoc whitespace. */", None),
        ("/****\n * @param name A decorative comment.\n */", None),
        ("/** @arg name An unconfigured alias. */", None),
        ("/**\n * @param name -\n */", None),
        (param_with_hyphen, Some(json!(["always"]))),
        (
            "
          /**
           * @param foo     - Foo.
           */
          function quux () {

          }
      ",
            Some(json!(["always"])),
        ),
        (
            "
          /**
           * @param foo - Foo.
           * @returns {SomeType} A description.
           */
          function quux () {

          }
      ",
            Some(json!(["always", {"tags": {"returns": "never"}}])),
        ),
        (param_without_hyphen, Some(json!(["never"]))),
        (
            "
          /**
           * @param foo
           */
          function quux () {

          }
      ",
            None,
        ),
        (
            "
          /**
           *
           */
          function quux () {

          }
      ",
            Some(json!(["always", {"tags": {"*": "always"}}])),
        ),
        (property_with_hyphen, Some(json!(["always", {"tags": {"property": "always"}}]))),
        (property_without_hyphen, Some(json!(["never", {"tags": {"property": "never"}}]))),
        (property_with_hyphen, Some(json!(["never", {"tags": {"*": "always"}}]))),
        (
            "
      /** Entry point for module.
       *
       * @param {!Array<string>} argv Command-line arguments.
       */
      function main(argv) {
      };
      ",
            Some(json!(["never"])),
        ),
        (
            "
        /**
         * @template {any} T - Arg 1
         * @template {string | number} K - Arg 2
         * @template {any} [R=(K extends keyof T ? T[K] : never)] - Arg 3  ->  Errors here
         * @typedef {any} Test
         */
      ",
            Some(json!(["always", {"tags": {"template": "always"}}])),
        ),
        (
            r#"
        /**
         * @param foo - The possible values for `foo` are as follows.
         * - `"option1"`: Description of option 1.
         * - `"option2"`: Description of option 2.
         */
      "#,
            None,
        ),
        (multiline_description, None),
        (
            r#"
        /**
         * @param foo
         * - `"option1"`: Description of option 1.
         * - `"option2"`: Description of option 2.
         */
      "#,
            None,
        ),
    ];

    // The rule-level upstream assertions check the result of a single fix pass.
    let two_tags = "
          /**
           * @param foo Foo.
           * @returns {SomeType} - A description.
           */
          function quux () {

          }
      ";
    let param_fixed = "
          /**
           * @param foo - Foo.
           * @returns {SomeType} - A description.
           */
          function quux () {

          }
      ";
    let both_fixed = "
          /**
           * @param foo - Foo.
           * @returns {SomeType} A description.
           */
          function quux () {

          }
      ";

    let fix = vec![
        (
            "function greet() {} /** @param name An inline comment. */",
            "function greet() {} /** @param name - An inline comment. */",
            None,
            None,
        ),
        (
            "/** @prop key Description. */",
            "/** @prop key - Description. */",
            Some(json!(["always", {"tags": {"property": "always"}}])),
            Some(json!({"settings": {"jsdoc": {"tagNamePreference": {"property": "prop"}}}})),
        ),
        (
            "/** @see {@link hi */",
            "/** @see {@link - hi */",
            Some(json!(["always", {"tags": {"see": "always"}}])),
            None,
        ),
        (
            "/** @arg name A preferred tag. */",
            "/** @arg name - A preferred tag. */",
            None,
            Some(json!({"settings": {"jsdoc": {"tagNamePreference": {"param": "arg"}}}})),
        ),
        (
            r#"/** @param "full name" A quoted parameter. */"#,
            r#"/** @param "full name" - A quoted parameter. */"#,
            None,
            None,
        ),
        (
            r#"/** @param options["property name"] A value. */"#,
            r#"/** @param options["property name"] - A value. */"#,
            None,
            None,
        ),
        (
            "/** @returns A value. */",
            "/** @returns - A value. */",
            Some(json!(["always", {"tags": {"return": "always"}}])),
            None,
        ),
        (
            "/** @param foo{bar} A parameter. */",
            "/** @param foo{bar} - A parameter. */",
            None,
            None,
        ),
        (param_without_hyphen, param_with_hyphen, Some(json!(["always"])), None),
        (
            param_without_hyphen,
            param_with_hyphen,
            Some(json!(["always", {"tags": {"*": "never"}}])),
            None,
        ),
        (
            "
          /**
           * @param foo Foo.
           * @returns {SomeType} - Hyphen here.
           */
          function quux () {

          }
      ",
            "
          /**
           * @param foo - Foo.
           * @returns {SomeType} - Hyphen here.
           */
          function quux () {

          }
      ",
            Some(json!(["always", {"tags": {"*": "never", "returns": "always"}}])),
            None,
        ),
        (param_without_hyphen, param_with_hyphen, None, None),
        (param_with_hyphen, param_without_hyphen, Some(json!(["never"])), None),
        (
            "
          /**
           * @param foo    - Foo.
           */
          function quux () {

          }
      ",
            "
          /**
           * @param foo    Foo.
           */
          function quux () {

          }
      ",
            Some(json!(["never"])),
            None,
        ),
        (
            "
          /**
           * @param foo - foo
           * @param foo foo
           */
          function quux () {

          }
      ",
            "
          /**
           * @param foo - foo
           * @param foo - foo
           */
          function quux () {

          }
      ",
            Some(json!(["always"])),
            None,
        ),
        (
            "
          /**
           * @param foo foo
           * bar
           * @param bar - bar
           */
          function quux () {

          }
      ",
            "
          /**
           * @param foo - foo
           * bar
           * @param bar - bar
           */
          function quux () {

          }
      ",
            Some(json!(["always"])),
            None,
        ),
        (
            disabled_param_tag,
            disabled_param_tag,
            None,
            Some(json!({"settings":{"jsdoc":{"tagNamePreference":{"param":false}}}})),
        ),
        (
            property_without_hyphen,
            property_with_hyphen,
            Some(json!(["always", {"tags": {"property": "always"}}])),
            None,
        ),
        (
            "
          /**
           * @template TempA, TempB A desc.
           */
      ",
            "
          /**
           * @template TempA, TempB - A desc.
           */
      ",
            Some(json!(["always", {"tags": {"template": "always"}}])),
            None,
        ),
        (
            property_with_hyphen,
            property_without_hyphen,
            Some(json!(["never", {"tags": {"property": "never"}}])),
            None,
        ),
        // eslint-plugin-jsdoc replaces the whole block, so its first pass only applies
        // the parameter fix. Oxlint can apply both non-overlapping edits in a single pass.
        (two_tags, param_fixed, None, None),
        (two_tags, both_fixed, Some(json!(["always", {"tags": {"returns": "never"}}])), None),
        (
            "
      /**
       * Split a unit to metric prefix and basic unit.
       *
       * @param {string} unit - Unit to split.
       * @param {string} [basicUnit] - Basic unit regardless of the metric prefix.
       *     If omitted, basic unit will be inferred by trying to remove the metric
       *     prefix in `unit`.
       *
       * @returns {{ prefix: string, basicUnit: string }} - Split result.
       *     If `unit` does not have a metric prefix, `''` is returned for `prefix`.
       *     If `unit` does not have a basic unit, `''` is returned for `basicUnit`.
       */
      ",
            "
      /**
       * Split a unit to metric prefix and basic unit.
       *
       * @param {string} unit - Unit to split.
       * @param {string} [basicUnit] - Basic unit regardless of the metric prefix.
       *     If omitted, basic unit will be inferred by trying to remove the metric
       *     prefix in `unit`.
       *
       * @returns {{ prefix: string, basicUnit: string }} Split result.
       *     If `unit` does not have a metric prefix, `''` is returned for `prefix`.
       *     If `unit` does not have a basic unit, `''` is returned for `basicUnit`.
       */
      ",
            Some(json!(["always", {"tags": {"*": "never", "property": "always"}}])),
            None,
        ),
        (
            "
      /**
       * @returns {{
       *   prefix: string, basicUnit: string
       * }} - Split result.
       */
      ",
            "
      /**
       * @returns {{
       *   prefix: string, basicUnit: string
       * }} Split result.
       */
      ",
            Some(json!(["always", {"tags": {"*": "never", "property": "always"}}])),
            None,
        ),
        (
            "
        /**
         * @param {(
         *  | string
         *  | number
         * )} input The input value
         */
        function test(input) {}
      ",
            "
        /**
         * @param {(
         *  | string
         *  | number
         * )} input - The input value
         */
        function test(input) {}
      ",
            None,
            None,
        ),
        (
            "
        /**
         * @template [O=unknown]
         * @param {string} name The name of the thing.
         */
        function test(name) {}
      ",
            "
        /**
         * @template [O=unknown]
         * @param {string} name - The name of the thing.
         */
        function test(name) {}
      ",
            None,
            None,
        ),
        (
            r#"
        /**
         * @param foo -
         * The possible values for `foo` are as follows.
         * - `"option1"`: Description of option 1.
         * - `"option2"`: Description of option 2.
         */
      "#,
            multiline_description,
            None,
            None,
        ),
    ];
    let pass = pass.into_iter().map(|(source, options)| (source, options, None)).collect();
    let fail = fix
        .iter()
        .map(|(source, _, options, settings)| (*source, options.clone(), settings.clone()))
        .collect();

    Tester::new(
        RequireHyphenBeforeParamDescription::NAME,
        RequireHyphenBeforeParamDescription::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
