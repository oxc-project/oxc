use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
pub struct NoBadBlocks(Box<NoBadBlocksConfig>);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct NoBadBlocksConfig {
    /// Tags which exempt a block from this rule. By default TypeScript directives
    /// and license tags are ignored.
    ignore: Vec<String>,
    /// Also report comments with three or more opening asterisks that contain no tags.
    prevent_all_multi_asterisk_blocks: bool,
}

impl Default for NoBadBlocksConfig {
    fn default() -> Self {
        Self {
            ignore: [
                "ts-check",
                "ts-expect-error",
                "ts-ignore",
                "ts-nocheck",
                "license",
                "license-end",
                "licstart",
                "licend",
                "source",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            prevent_all_multi_asterisk_blocks: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports block comments that contain a JSDoc tag but have the wrong number of
    /// asterisks in the opening delimiter.
    ///
    /// ### Why is this bad?
    ///
    /// Documentation tools can skip a comment that begins with `/*` or `/***`.
    /// The fix changes its opening delimiter to `/**`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* @param input The input string. */
    /// function parse(input) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /** @param input The input string. */
    /// function parse(input) {}
    /// ```
    NoBadBlocks,
    jsdoc,
    style,
    conditional_fix,
    config = NoBadBlocksConfig,
    version = "next",
    short_description = "Require JSDoc-like comments to begin with two asterisks.",
);

impl Rule for NoBadBlocks {
    fn from_configuration(value: Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        if let Some(message) = structured_tag_error(ctx) {
            ctx.diagnostic(OxcDiagnostic::warn(message).with_label(Span::empty(0)));
            return;
        }
        for comment in ctx.semantic().comments() {
            if !comment.is_block() {
                continue;
            }
            let text = ctx.source_range(comment.span);
            let opener_len =
                1 + text.as_bytes()[1..].iter().take_while(|&&byte| byte == b'*').count();
            if opener_len == 3 {
                continue;
            }

            if opener_len == 2 || !self.0.prevent_all_multi_asterisk_blocks {
                let Some(body) = text.get(opener_len..text.len() - 2) else { continue };
                if (opener_len == 2
                    && body.trim_start_matches(is_js_whitespace).starts_with("eslint"))
                    || !has_documentation_tag(body, &self.0.ignore)
                {
                    continue;
                }
            }

            let opener = Span::sized(comment.span.start, u32::try_from(opener_len).unwrap());
            let diagnostic =
                OxcDiagnostic::warn("Expected JSDoc-like comment to begin with two asterisks.")
                    .with_label(opener);
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(opener, "/**"));
        }
    }
}

fn is_js_whitespace(c: char) -> bool {
    is_white_space(c) || is_line_terminator(c)
}

fn has_documentation_tag(body: &str, ignore: &[String]) -> bool {
    if !body.contains('@') {
        return false;
    }

    let mut has_tag = false;
    let mut fenced = false;
    for (index, line) in body.lines().enumerate() {
        let mut line = line.trim_start_matches(is_js_whitespace);
        if index != 0
            && let Some(rest) = line.strip_prefix('*')
        {
            line = rest.trim_start_matches(is_js_whitespace);
        }
        if !fenced && let Some(rest) = line.strip_prefix('@') {
            let tag = rest.split_once(is_js_whitespace).map_or(rest, |(tag, _)| tag);
            if !tag.is_empty() && !tag.contains('/') {
                if ignore.iter().any(|ignored| ignored == tag) {
                    return false;
                }
                has_tag = true;
            }
        }
        if !line.matches("```").count().is_multiple_of(2) {
            fenced = !fenced;
        }
    }
    has_tag
}

fn structured_tag_error(ctx: &LintContext) -> Option<String> {
    let tags = ctx.settings().json.as_ref()?.get("jsdoc")?.get("structuredTags")?.as_object()?;
    for tag in tags.values() {
        let Some(required) = tag.get("required").and_then(Value::as_array) else { continue };
        for (requirement, field) in [
            ("name", "name"),
            ("type", "type"),
            ("typeOrNameRequired", "name"),
            ("typeOrNameRequired", "type"),
        ] {
            if tag[field] == false && required.iter().any(|value| value == requirement) {
                return Some(format!(
                    "Cannot add \"{requirement}\" to `require` with the tag's `{field}` set to `false`"
                ));
            }
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (
            "
          /**
           * @property foo
           */
      ",
            None,
            None,
        ),
        (
            "
          /**
           * @param foo
           */
           function quux () {

           }
      ",
            None,
            None,
        ),
        (
            "
      function quux () {

      }
      ",
            None,
            None,
        ),
        (
            "
           /* This could just be intended as a regular multiline comment,
              so don't report this */
           function quux () {

           }
      ",
            None,
            None,
        ),
        (
            "
           /* Just a regular multiline comment with an `@` but not appearing
               like a tag in a jsdoc-block, so don't report */
           function quux () {

           }
      ",
            None,
            None,
        ),
        ("/* @ts-check */", None, None),
        ("/* @ts-expect-error */", None, None),
        ("/* @ts-ignore */", None, None),
        ("/* @ts-nocheck */", None, None),
        ("/* */", None, None),
        ("/** */", None, None),
        ("/* @custom */", Some(json!([{ "ignore": ["custom"] }])), None),
        (
            "
          /***
           * This is not JSDoc because of the 3 asterisks, but
           * is allowed without `preventAllMultiAsteriskBlocks`, as
           * some might wish normal multiline comments.
           */
          function quux (foo) {

          }
      ",
            None,
            None,
        ),
        ("/***/", None, None),
        ("/* eslint-disable @stylistic/max-len  */", None, None),
        ("/* See @param in prose. */", None, None),
        ("/* @scope/package */", None, None),
        (
            "/*
 * ```js
 * @decorator
 * ```
 */",
            None,
            None,
        ),
        (
            "/*
 * @param value
 * @license MIT
 */",
            None,
            None,
        ),
        ("// @param value", None, None),
        ("/* \u{85}@param value */", None, None),
        (
            "/** @param alreadyValid */",
            Some(json!([{ "preventAllMultiAsteriskBlocks": true }])),
            None,
        ),
    ];
    let fix = vec![
        (
            "
          /*
           * @param foo
           */
          function quux (foo) {

          }
      ",
            "
          /**
           * @param foo
           */
          function quux (foo) {

          }
      ",
            None,
        ),
        (
            "
          /*
           * @property foo
           */
      ",
            "
          /**
           * @property foo
           */
      ",
            None,
        ),
        ("/* @ts-ignore */", "/** @ts-ignore */", Some(json!([{ "ignore": [] }]))),
        (
            "
      /*
       * Some description.
       *
       * @returns {string} Some string
       */
      function echo() {
        return 'Something';
      }
      ",
            "
      /**
       * Some description.
       *
       * @returns {string} Some string
       */
      function echo() {
        return 'Something';
      }
      ",
            None,
        ),
        (
            "
          /***
           * @param foo
           */
          function quux (foo) {

          }
      ",
            "
          /**
           * @param foo
           */
          function quux (foo) {

          }
      ",
            None,
        ),
        (
            "
          /***
           *
           */
          function quux (foo) {

          }
      ",
            "
          /**
           *
           */
          function quux (foo) {

          }
      ",
            Some(json!([{ "preventAllMultiAsteriskBlocks": true }])),
        ),
        (
            "
          /***************************
           * Helpers
           ***************************/

          /*
           * @param foo The foo.
           */
          function quux (foo) {}
      ",
            "
          /***************************
           * Helpers
           ***************************/

          /**
           * @param foo The foo.
           */
          function quux (foo) {}
      ",
            None,
        ),
        (
            "
          /*
           * @param foo The foo.
           */
          function quux (foo) {}

          /***************************
           * Helpers
           ***************************/
      ",
            "
          /**
           * @param foo The foo.
           */
          function quux (foo) {}

          /***************************
           * Helpers
           ***************************/
      ",
            None,
        ),
        (
            "/*@param input*/\r\n/*** @returns {number} */",
            "/**@param input*/\r\n/** @returns {number} */",
            None,
        ),
        ("/* \u{feff}@tag */", "/** \u{feff}@tag */", None),
        ("const x = 1; /* @param value */", "const x = 1; /** @param value */", None),
        ("/***/", "/**/", Some(json!([{ "preventAllMultiAsteriskBlocks": true }]))),
    ];
    let mut fail =
        fix.iter().map(|(input, _, options)| (*input, options.clone(), None)).collect::<Vec<_>>();
    fail.push((
        "
          function quux() {

          }
      ",
        None,
        Some(json!({ "settings": { "jsdoc": { "structuredTags": {
            "see": { "name": false, "required": ["name"] }
        } } } })),
    ));
    Tester::new(NoBadBlocks::NAME, NoBadBlocks::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
