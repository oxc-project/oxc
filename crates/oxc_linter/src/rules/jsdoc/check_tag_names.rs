mod inline;
mod property_fixer;
mod typed;
#[cfg(test)]
mod upstream_tests;

use std::borrow::Cow;

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::JSDoc;
use oxc_span::Span;

use crate::{
    config::JSDocMode,
    context::LintContext,
    fixer::{Fix, RuleFix},
    rule::{DefaultRuleConfig, Rule},
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn check_tag_names_diagnostic(span: Span, message: impl Into<Cow<'static, str>>) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CheckTagNames(Box<CheckTagNamesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports invalid block and inline tag names and replaces non-preferred aliases.
    /// Additionally checks for tag names that are redundant when using a type checker such as TypeScript.
    ///
    /// ### Why is this bad?
    ///
    /// Using invalid tags can lead to confusion and make the documentation harder to read.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @Param */
    /// /** @foo */
    ///
    /// /**
    ///  * This is redundant when typed.
    ///  * @type {string}
    ///  */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @param */
    /// ```
    ///
    /// ### Settings
    ///
    /// The default `typescript` mode recognizes TSDoc and TypeDoc block tags such as
    /// `@remarks` and `@inline`. Set `settings.jsdoc.mode` to `jsdoc` for the standard
    /// JSDoc vocabulary or to `closure` or `permissive` for Closure Compiler tags.
    ///
    /// The `definedTags` option and the keys of `settings.jsdoc.structuredTags` also allow custom
    /// block tags. Use `inlineTags` to replace the list of allowed inline tags.
    ///
    /// Configuration for preferred names is done via [`settings.jsdoc.tagNamePreference`](/docs/guide/usage/linter/config-file-reference.html#settings-jsdoc-tagnamepreference).
    /// There is no CLI-only parameter for this rule.
    ///
    /// You can add custom tags by adding a key-value pair where both match the name of the tag you want to add, like so:
    ///
    /// ::: code-group
    ///
    /// ```json [Config (.oxlintrc.json)]
    /// {
    ///   "plugins": ["jsdoc"],
    ///   "rules": {
    ///     "jsdoc/check-tag-names": "error"
    ///   },
    ///   "settings": { // [!code highlight:7]
    ///     "jsdoc": {
    ///       "tagNamePreference": {
    ///         "customTagName": "customTagName"
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    /// :::
    ///
    /// Examples of correct code for this rule with the above configuration, adding the `customTagName` tag:
    ///
    /// ```js
    /// /**
    ///  * @customTagName
    ///  */
    /// ```
    CheckTagNames,
    jsdoc,
    correctness,
    recommended,
    conditional_fix,
    config = CheckTagNamesConfig,
    version = "0.3.2",
    short_description = "Reports invalid block and inline tag names.",
);

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct CheckTagNamesConfig {
    /// Additional tag names to allow.
    defined_tags: Vec<String>,
    /// Automatically remove tags that are redundant with the type system.
    enable_fixer: bool,
    /// Allowed inline tag names, such as `link` in `{@link MyType}`.
    inline_tags: Vec<String>,
    /// Whether to allow JSX-related tags:
    /// - `jsx`
    /// - `jsxFrag`
    /// - `jsxImportSource`
    /// - `jsxRuntime`
    jsx_tags: bool,
    /// If typed is `true`, disallow tags that are unnecessary/duplicative of TypeScript functionality.
    typed: bool,
}

impl Default for CheckTagNamesConfig {
    fn default() -> Self {
        Self {
            defined_tags: vec![],
            enable_fixer: true,
            inline_tags: [
                "link",
                "linkcode",
                "linkplain",
                "tutorial",
                "inheritDoc",
                "label",
                "include",
                "includeCode",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            jsx_tags: false,
            typed: false,
        }
    }
}

fn check_settings(ctx: &LintContext) {
    let settings = &ctx.settings().jsdoc;
    if let JSDocMode::Unrecognized(mode) = &settings.mode {
        ctx.diagnostic(
            OxcDiagnostic::warn(format!("Unrecognized value `{mode}` for `settings.jsdoc.mode`."))
                .with_label(Span::empty(0)),
        );
    }
    for _ in 0..settings.invalid_tag_preferences() {
        ctx.diagnostic(OxcDiagnostic::warn(
            "Invalid `settings.jsdoc.tagNamePreference`. Values must be falsy, a string, or an object."
        ).with_label(Span::empty(0)));
    }
}

// https://github.com/gajus/eslint-plugin-jsdoc/blob/7e9acbf2143fd98b61ea406b7ea8d723b55002ef/src/tagNames.js
const COMMON_BLOCK_TAGS: phf::Set<&'static str> = phf::phf_set![
    "abstract",
    "access",
    "alias",
    "async",
    "augments",
    "author",
    "borrows",
    "callback",
    "class",
    "classdesc",
    "constant",
    "constructs",
    "copyright",
    "default",
    "deprecated",
    "description",
    "enum",
    "event",
    "example",
    "exports",
    "external",
    "file",
    "fires",
    "function",
    "generator",
    "global",
    "hideconstructor",
    "ignore",
    "implements",
    "inheritDoc",
    "inner",
    "instance",
    "interface",
    "kind",
    "lends",
    "license",
    "listens",
    "member",
    "memberof",
    "memberof!",
    "mixes",
    "mixin",
    // Undocumented, but exists
    // https://github.com/jsdoc/jsdoc/blob/a08ac18a11f5b0d93421d1e8ecf632468db2d045/packages/jsdoc-tag/lib/definitions/core.js#L374
    "modifies",
    "module",
    "name",
    "namespace",
    "override",
    "package",
    "param",
    "private",
    "property",
    "protected",
    "public",
    "readonly",
    "requires",
    "returns",
    "see",
    "since",
    "static",
    "summary",
    "this",
    "throws",
    "todo",
    "tutorial",
    "type",
    "typedef",
    "variation",
    "version",
    "yields",
];

// Both TypeScript and Closure modes include TSDoc and TypeDoc tags.
const TYPED_BLOCK_TAGS: phf::Set<&'static str> = phf::phf_set![
    // TSDoc
    "alpha",
    "beta",
    "decorator",
    "eventProperty",
    "experimental",
    "packageDocumentation",
    "privateRemarks",
    "remarks",
    "sealed",
    // TypeDoc
    "category",
    "categoryDescription",
    "disableGroups",
    "document",
    "expand",
    "expandType",
    "group",
    "groupDescription",
    "hidden",
    "hideCategories",
    "hideGroups",
    "inline",
    "inlineType",
    "mergeModuleWith",
    "preventExpand",
    "preventInline",
    "primaryExport",
    "showCategories",
    "showGroups",
    "sortStrategy",
    "useDeclaredType",
    // TypeScript
    "import",
    "jsx",
    "template",
];

const CLOSURE_BLOCK_TAGS: phf::Set<&'static str> = phf::phf_set![
    "closurePrimitive",
    "customElement",
    "define",
    "dict",
    "export",
    "expose",
    "externs",
    "final",
    "idGenerator",
    "implicitCast",
    "meaning",
    "mixinClass",
    "mixinFunction",
    "ngInject",
    "noalias",
    "nocollapse",
    "nocompile",
    "noinline",
    "nosideeffects",
    "owner",
    "polymer",
    "polymerBehavior",
    "preserve",
    "record",
    "return",
    "struct",
    "suppress",
    "typeSummary",
    "unrestricted",
    "wizaction",
];

fn is_valid_block_tag(name: &str, mode: &JSDocMode) -> bool {
    COMMON_BLOCK_TAGS.contains(name)
        || match mode {
            JSDocMode::JSDoc | JSDocMode::Unrecognized(_) => name == "inheritdoc",
            JSDocMode::TypeScript => {
                matches!(name, "inheritdoc" | "internal" | "overload" | "satisfies")
                    || TYPED_BLOCK_TAGS.contains(name)
            }
            JSDocMode::Closure | JSDocMode::Permissive => {
                TYPED_BLOCK_TAGS.contains(name) || CLOSURE_BLOCK_TAGS.contains(name)
            }
        }
}

const JSX_TAGS: [&str; 4] = ["jsx", "jsxFrag", "jsxImportSource", "jsxRuntime"];

const ALWAYS_INVALID_TAGS_IF_TYPED: [&str; 13] = [
    "augments",
    "callback",
    "class",
    "enum",
    "implements",
    "private",
    "property",
    "protected",
    "public",
    "readonly",
    "this",
    "type",
    "typedef",
];

const OUTSIDE_AMBIENT_INVALID_TAGS_IF_TYPED: [&str; 27] = [
    "abstract",
    "access",
    "class",
    "constant",
    "constructs",
    // I'm not sure but this seems to be allowed...
    // https://github.com/gajus/eslint-plugin-jsdoc/blob/e343ab5b1efaa59b07c600138aee070b4083857e/src/rules/checkTagNames.js#L140
    // "default",
    "enum",
    "export",
    "exports",
    "function",
    "global",
    "inherits",
    "instance",
    "interface",
    "member",
    "memberOf",
    "memberof",
    "method",
    "mixes",
    "mixin",
    "module",
    "name",
    "namespace",
    "override",
    "property",
    "requires",
    "static",
    "this",
];

impl Rule for CheckTagNames {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let mut jsdocs = ctx
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
            .peekable();
        if jsdocs.peek().is_none() {
            return;
        }
        check_settings(ctx);

        // Only typed checks depend on which declaration owns the comment. For
        // consecutive JSDocs only the nearest one documents the declaration.
        let mut attached = FxHashMap::default();
        if self.0.typed {
            for node in ctx.nodes() {
                if let Some(jsdoc) = ctx.jsdoc().get_one_by_node(ctx.nodes(), node) {
                    attached.insert(jsdoc.span.start, node.kind());
                }
            }
        }
        for jsdoc in jsdocs {
            self.check_block(ctx, jsdoc, attached.get(&jsdoc.span.start).copied());
            inline::check(self, ctx, jsdoc);
        }
    }
}

impl CheckTagNames {
    fn check_block(&self, ctx: &LintContext, jsdoc: &JSDoc, node: Option<AstKind>) {
        let settings = &ctx.settings().jsdoc;
        let config = &self.0;
        let include_prop = !settings.has_tag_preference("prop");
        // The JSDoc parser's description and name spans normalize LF and CRLF.
        // For other line breaks, keep block text intact and still report its tags.
        let fix_typed = config.typed && config.enable_fixer && {
            let content = ctx.source_range(jsdoc.span);
            !content.contains(['\u{2028}', '\u{2029}'])
                && content
                    .match_indices('\r')
                    .all(|(index, _)| content.as_bytes().get(index + 1) == Some(&b'\n'))
        };
        let mut property_fix = if fix_typed {
            node.and_then(|node| property_fixer::property_fix(ctx, jsdoc, node, include_prop))
        } else {
            None
        };

        let grouped = property_fix.is_some();
        for (idx, tag) in jsdoc.tags().iter().enumerate() {
            let name = tag.kind.parsed();
            if grouped && (name == "property" || (name == "prop" && include_prop)) {
                let message = if name == "property" {
                    "'@property' is redundant when using a type system."
                } else {
                    r#"Invalid JSDoc tag (preference). Replace "prop" JSDoc tag with "property"."#
                };
                let diagnostic = check_tag_names_diagnostic(tag.kind.span, message);
                // One edit handles the entire block; attaching it to each tag would
                // duplicate the whole object text for every documented property.
                if let Some(fixes) = property_fix.take() {
                    ctx.diagnostic_with_fix(diagnostic, |_| {
                        RuleFix::from(fixes).with_message(
                            "Remove redundant property tags and preserve their descriptions.",
                        )
                    });
                } else {
                    ctx.diagnostic(diagnostic);
                }
                continue;
            }
            if config.jsx_tags && JSX_TAGS.contains(&name) {
                continue;
            }
            if config.typed && typed::check(ctx, jsdoc, idx, node, fix_typed) {
                continue;
            }
            let valid = is_valid_block_tag(name, &settings.mode)
                || (config.typed && name == "template")
                || settings.is_configured_tag(name)
                || config.defined_tags.iter().any(|custom| custom == name);
            if !valid {
                ctx.diagnostic(check_tag_names_diagnostic(
                    tag.kind.span,
                    format!(r#"Invalid JSDoc tag name "{name}"."#),
                ));
                continue;
            }
            if let Some((message, replacement)) = settings.tag_preference(name) {
                let diagnostic = check_tag_names_diagnostic(tag.kind.span, message.into_owned());
                if let Some(replacement) = replacement {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fixer
                            .replace(tag.kind.span, format!("@{replacement}"))
                            .with_message(format!("Rename the tag to `@{replacement}`."))
                    });
                } else {
                    ctx.diagnostic(diagnostic);
                }
            }
        }
    }
}

/// When the block separates two tokens, keep the whitespace needed for tokenization and ASI.
fn remove_comment(source: &str, full: Span) -> Fix {
    let span = remove_comment_span(source, full);
    let left = &source[..span.start as usize];
    let right = &source[span.end as usize..];
    let left_code = left.trim_end();
    let right_code = right.trim_start();
    if left_code.is_empty() || right_code.is_empty() {
        return Fix::delete(span);
    }
    let newline = ['\r', '\n', '\u{2028}', '\u{2029}'];
    let removed = span.source_text(source);
    if removed.contains(newline)
        && !left[left_code.len()..].contains(newline)
        && !right[..right.len() - right_code.len()].contains(newline)
    {
        return Fix::new(if removed.contains("\r\n") { "\r\n" } else { "\n" }, span);
    }
    if left.len() == left_code.len() && right.len() == right_code.len() {
        return Fix::new(" ", span);
    }
    Fix::delete(span)
}

/// Include the line break before a standalone comment. At the beginning of a
/// line without a preceding newline, remove one following whitespace character.
fn remove_comment_span(source: &str, span: Span) -> Span {
    let before = &source[..span.start as usize];
    if let Some((line, indent)) = before.rsplit_once('\n')
        && indent.bytes().all(|b| matches!(b, b' ' | b'\t'))
    {
        let start = line.strip_suffix('\r').unwrap_or(line).len();
        return Span::new(u32::try_from(start).unwrap(), span.end);
    }
    let after = &source[span.end as usize..];
    let following = if after.starts_with("\r\n") {
        2
    } else {
        after
            .chars()
            .next()
            .filter(|c| c.is_whitespace())
            .map_or(0, |c| u32::try_from(c.len_utf8()).unwrap())
    };
    Span::new(span.start, span.end + following)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                      /**
                       * @param foo (pass: valid name)
                       */
                      function quux (foo) {

                      }
                  ",
            None,
            None,
        ),
        (
            "
                      /**
                       * @memberof! foo (pass: valid name)
                       */
                      function quux (foo) {

                      }
                  ",
            None,
            None,
        ),
        (
            "
                      /**
                       * @bar foo (pass: invalid name but defined)
                       */
                      function quux (foo) {

                      }
                  ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
                      /**
                       * @baz @bar foo (pass: invalid names but defined)
                       */
                      function quux (foo) {

                      }
                  ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "baz", "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
                      /**
                       * @baz @bar foo (pass: invalid names but user preferred)
                       */
                      function quux (foo) {

                      }
                  ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "param": "baz",
                  "returns": {
                    "message": "Prefer `bar`",
                    "replacement": "bar",
                  },
                  "todo": false,
                },
              }},
            })),
        ),
        (
            "
                      /**
                       * @arg foo (pass: invalid name but user preferred)
                       */
                      function quux (foo) {

                      }
                  ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "param": "arg",
                },
              }},
            })),
        ),
        (
            "
                  /**
                   * @returns (pass: valid name)
                   */
                  function quux (foo) {}
                  ",
            None,
            None,
        ),
        ("", None, None),
        (
            "
                      /**
                       * (pass: no tag)
                       */
                      function quux (foo) {

                      }
                  ",
            None,
            None,
        ),
        (
            "
                      /**
                       * @todo (pass: valid name)
                       */
                      function quux () {

                      }
                  ",
            None,
            None,
        ),
        (
            "
                      /**
                       * @extends Foo (pass: invalid name but user preferred)
                       */
                      function quux () {

                      }
                  ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "augments": {
                    "message": "@extends is to be used over @augments.",
                    "replacement": "extends",
                  },
                },
              }},
            })),
        ),
        (
            "
                      /**
                       * (Set tag name preference to itself to get aliases to
                       *   work along with main tag name.)
                       * @augments Bar
                       * @extends Foo (pass: invalid name but user preferred)
                       */
                      function quux () {
                      }
                  ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "extends": "extends",
                },
              }},
            })),
        ),
        (
            "
                  /**
                   * Registers the `target` class as a transient dependency; each time the dependency is resolved a new instance will be created.
                   *
                   * @param target - The class / constructor function to register as transient.
                   *
                   * @example ```ts
                  @transient()
                  class Foo { }
                  ```
                   * @param Time for a new tag (pass: valid names)
                   */
                  export function transient<T>(target?: T): T {
                    // ...
                  }
            ",
            None,
            None,
        ),
        (
            "
                    /** @jsx h */
                    /** @jsxFrag Fragment */
                    /** @jsxImportSource preact */
                    /** @jsxRuntime automatic (pass: valid jsx names)*/
                  ",
            Some(serde_json::json!([
              {
                "jsxTags": true,
              },
            ])),
            None,
        ),
        (
            "
                  /**
                   * @internal (pass: valid name)
                   */
                  ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": { }},
            })),
        ),
        (
            "
                    /**
                     * @overload
                     * @satisfies (pass: valid names)
                     */
                  ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": { }},
            })),
        ),
        (
            "
                    /**
                     * @module
                     * A comment related to the module
                     */
                  ",
            None,
            None,
        ),
        (
            "
                    /**
                     * @module
                     * A comment related to the module
                     */
                  ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        // Typed
        (
            "
                          /** @default 0 */
                          let a;
                        ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                    /** @template name */
                    let a;
                  ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                    /** @param param - takes information */
                    function takesOne(param) {}
                  ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        // https://github.com/oxc-project/oxc/issues/10910
        (
          "
          /**
           * @see [@parcel/watcher](https://github.com/parcel-bundler/watcher)
           */
          function quux (foo) { }
      ",
          Some(serde_json::json!([ { "definedTags": [] } ])),
          None,
      ),
       // https://github.com/oxc-project/oxc/issues/13570
        (
          "
          /**
           * @import { Page } from '@playwright/test';
           */
          function quux (foo) { }
      ",
          Some(serde_json::json!([ { "definedTags": [] } ])),
          None,
      ),
        (
          "
          /**
           * @see [[[[]@foo]
           */
          function quux (foo) { }
      ",
          None,
          None,
      ),
      (
          "
          /**
           * @license bcrypt.js (c) 2013 Daniel Wirtz <dcode@dcode.io>
           * Released under the Apache License, Version 2.0
           */
          function quux () { }
      ",
          None,
          None,
      ),
      (
          "
          /**
           * @see Uses @vue/shared package
           */
          function quux () { }
      ",
          None,
          None,
      ),
    ];

    let fail = vec![
        (
            "
                            /** @typoo {string} (fail: invalid name) */
                            let a;
                          ",
            None,
            None,
        ),
        (
            "
                              /**
                               * @Param (fail: invalid name)
                               */
                              function quux () {

                              }
                          ",
            None,
            None,
        ),
        (
            "
                              /**
                               * @foo (fail: invalid name)
                               */
                              function quux () {

                              }
                          ",
            None,
            None,
        ),
        (
            "
                              /**
                               * @arg foo (fail: invalid name, default aliased)
                               */
                              function quux (foo) {

                              }
                          ",
            None,
            None,
        ),
        (
            "
                              /**
                               * @param foo (fail: valid name but user preferred)
                               */
                              function quux (foo) {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "param": "arg",
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @constructor foo (fail: invalid name and user preferred)
                               */
                              function quux (foo) {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "constructor": "cons",
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @arg foo (fail: invalid name and user preferred)
                               */
                              function quux (foo) {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "arg": "somethingDifferent",
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @param foo (fail: valid name but user preferred)
                               */
                              function quux (foo) {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "param": "parameter",
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @bar foo (fail: invalid name)
                               */
                              function quux (foo) {

                              }
                          ",
            None,
            None,
        ),
        (
            "
                              /**
                               * @baz @bar foo (fail: invalid name)
                               */
                              function quux (foo) {

                              }
                          ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
                                /**
                                 * @bar
                                 * @baz (fail: invalid name)
                                 */
                                function quux (foo) {

                                }
                            ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
                              /**
                               * @todo (fail: valid name but blocked)
                               */
                              function quux () {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "todo": false,
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @todo (fail: valid name but blocked)
                               */
                              function quux () {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please resolve to-dos or add to the tracker",
                  },
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @todo (fail: valid name but blocked)
                               */
                              function quux () {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please use x-todo instead of todo",
                    "replacement": "x-todo",
                  },
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @property {object} a
                               * @prop {boolean} b (fail: invalid name, default aliased)
                               */
                              function quux () {

                              }
                          ",
            None,
            None,
        ),
        (
            "
                              /**
                               * @abc foo (fail: invalid name and user preferred)
                               * @abcd bar
                               */
                              function quux () {

                              }
                          ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "abcd",
                ],
              },
            ])),
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "abc": "abcd",
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @abc (fail: invalid name and user preferred)
                               * @abcd
                               */
                              function quux () {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "abc": "abcd",
                },
              }},
            })),
        ),
        (
            "
                            /** @jsx h */
                            /** @jsxFrag Fragment */
                            /** @jsxImportSource preact */
                            /** @jsxRuntime automatic */
                          ",
            None,
            None,
        ),
        (
            "
                          /**
                           * @constructor (fail: invalid name)
                           */
                          function Test() {
                            this.works = false;
                          }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "returns": "return",
                },
              }},
            })),
        ),
        (
            "
                              /**
                               * @todo (fail: valid name but blocked)
                               */
                              function quux () {

                              }
                          ",
            None,
            Some(serde_json::json!({
              "settings" : { "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please don't use todo",
                  },
                },
              }},
            })),
        ),
        // Typed
        (
            "/** @type {string} */let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /**
                             * Existing comment.
                             *  @type {string}
                             */
                            let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                          /** @typedef {Object} MyObject
                           * @property {string} id - my id
                           */
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                          /**
                           * @property {string} id - my id
                           */
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                          /** @typedef {Object} MyObject */
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                          /** @typedef {Object} MyObject
                           */
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /** @abstract */
                            let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            const a = {
                              /** @abstract */
                              b: true,
                            };
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /** @template */
                            let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /**
                             * Prior description.
                             *
                             * @template
                             */
                            let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
    ];

    let dts_pass: Vec<(&'static str, Option<serde_json::Value>, Option<serde_json::Value>)> = vec![
        (
            "
                            /** @default 0 */
                            declare let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /** @abstract */
                            let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /** @abstract */
                            declare let a;
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            /** @abstract */
                            declare namespace b { let a; }
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
        (
            "
                            declare namespace test {
                              /** @abstract */
                              let a;
                            }
                          ",
            Some(serde_json::json!([
              {
                "typed": true,
              },
            ])),
            None,
        ),
    ];
    let dts_fail: Vec<(&'static str, Option<serde_json::Value>, Option<serde_json::Value>)> =
        vec![(
            "
                            /** @typoo {string} (fail: invalid name) */
                            let a;
                          ",
            None,
            None,
        )];

    Tester::new(CheckTagNames::NAME, CheckTagNames::PLUGIN, pass, fail)
        .expect_fix(vec![
            (
                "/**\r * Keep.\r * @type {string}\r * @deprecated Keep.\r */\rconst x = 1;",
                "/**\r * Keep.\r * @type {string}\r * @deprecated Keep.\r */\rconst x = 1;",
                Some(serde_json::json!([{ "typed": true }])),
            ),
            (
                "/**\u{2028} * Keep.\u{2028} * @property x - description\u{2028} */\u{2028}const obj = {x: true};",
                "/**\u{2028} * Keep.\u{2028} * @property x - description\u{2028} */\u{2028}const obj = {x: true};",
                Some(serde_json::json!([{ "typed": true }])),
            ),
            (
                "/**\n * Summary.\n * @prop {{\n *   a: boolean\n * }} x - First.\n * - Second.\n */\nconst obj = {\n  x: true,\n};",
                "/**\n * Summary.\n */\nconst obj = {\n  /**\n   * - First.\n   * - Second.\n   */\n  x: true,\n};",
                Some(serde_json::json!([{ "typed": true }])),
            ),
            (
                "const t = typeof/** @type {string} */value;",
                "const t = typeof value;",
                Some(serde_json::json!([{ "typed": true }])),
            ),
            (
                "function f() { return\n/** @type {string} */foo; }",
                "function f() { return\nfoo; }",
                Some(serde_json::json!([{ "typed": true }])),
            ),
        ])
        .test_and_snapshot();
    // Currently only 1 snapshot can be saved under a rule name
    Tester::new(CheckTagNames::NAME, CheckTagNames::PLUGIN, dts_pass, dts_fail)
        .change_rule_path("test.d.ts")
        .expect_fix(vec![("/** @arg foo */", "/** @param foo */")])
        .test();
}

#[test]
fn upstream_block_tag_modes() {
    use crate::tester::Tester;
    use serde_json::json;

    // The independent tag lists and cases below come from eslint-plugin-jsdoc.
    const JSDOC: &str = concat!(
        "modifies abstract access alias async augments author borrows callback ",
        "class classdesc constant constructs copyright default deprecated description ",
        "enum event example exports external file fires function generator ",
        "global hideconstructor ignore implements inheritdoc inheritDoc inner ",
        "instance interface kind lends license listens member memberof memberof! ",
        "mixes mixin module name namespace override package param private property ",
        "protected public readonly requires returns see since static summary ",
        "this throws todo tutorial type typedef variation version yields",
    );
    const TYPESCRIPT_EXTRA: &str = concat!(
        "alpha beta decorator eventProperty experimental packageDocumentation ",
        "privateRemarks remarks sealed category categoryDescription disableGroups ",
        "document expand expandType group groupDescription hidden hideCategories ",
        "hideGroups inline inlineType mergeModuleWith preventExpand preventInline ",
        "primaryExport showCategories showGroups sortStrategy useDeclaredType ",
        "import internal jsx overload satisfies template",
    );
    let block = |tags: &str| {
        let lines = tags.split_ascii_whitespace().collect::<Vec<_>>().join("\n * @");
        format!("/** \n * @{lines}\n */\nfunction quux (foo) {{}}")
    };
    let all_jsdoc = block(JSDOC);
    let all_typescript = block(&format!("{JSDOC} {TYPESCRIPT_EXTRA}"));

    let mode = |name| Some(json!({ "settings": { "jsdoc": { "mode": name } } }));
    let pass = vec![
        (all_jsdoc.as_str(), None, None),
        (all_typescript.as_str(), None, mode("typescript")),
        ("/** @return */", None, mode("closure")),
        ("/** @externs */", None, mode("closure")),
        ("/** @suppress */", None, mode("permissive")),
        ("/** @inline */", None, mode("closure")),
        ("/** @remarks */", None, mode("permissive")),
        ("/** @internal */", None, mode("typescript")),
        ("/**\n * @overload\n * @satisfies\n */", None, mode("typescript")),
        (
            "/** @typeParam T */",
            None,
            Some(json!({ "settings": { "jsdoc": {
            "tagNamePreference": { "template": "typeParam" }
        } } })),
        ),
    ];
    let strict_cases: Vec<_> = TYPESCRIPT_EXTRA.split_ascii_whitespace().map(block).collect();
    let mut fail = vec![
        (all_typescript.as_str(), None, mode("jsdoc")),
        ("/** @returns */", None, mode("closure")),
        ("/** @overload */", None, mode("closure")),
        ("/** @externs */", None, mode("typescript")),
        ("/** @inheritdoc */", None, mode("permissive")),
        ("/** @typeParam T */", None, mode("typescript")),
        ("/** @Remarks */", None, mode("typescript")),
        (
            "/** @remarks */",
            None,
            Some(json!({ "settings": { "jsdoc": {
            "tagNamePreference": { "remarks": false }
        } } })),
        ),
    ];
    fail.extend(strict_cases.iter().map(|source| (source.as_str(), None, mode("jsdoc"))));

    Tester::new(CheckTagNames::NAME, CheckTagNames::PLUGIN, pass, fail)
        .expect_fix(vec![
            ("/** @typeParam T */", "/** @template T */", None, None),
            (
                "/** @date */",
                "/** @date */",
                None,
                Some(json!({ "settings": { "jsdoc": { "tagNamePreference": { "tag date": "date2" } } } })),
            ),
        ])
        .test();
}
