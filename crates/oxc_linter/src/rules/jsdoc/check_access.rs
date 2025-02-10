use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, utils::should_ignore_as_internal};

fn invalid_access_level(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid access level is specified or missing.")
        .with_help("Valid access levels are `package`, `private`, `protected`, and `public`.")
        .with_label(span)
}

fn redundant_access_tags(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Mixing of @access with @public, @private, @protected, or @package on the same doc block.",
    )
    .with_help("There should be only one instance of access tag in a JSDoc comment.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct CheckAccess;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks that `@access` tags use one of the following values:
    /// - "package", "private", "protected", "public"
    ///
    /// Also reports:
    /// - Mixing of `@access` with `@public`, `@private`, `@protected`, or `@package` on the same doc block.
    /// - Use of multiple instances of `@access` (or the `@public`, etc) on the same doc block.
    ///
    /// ### Why is this bad?
    ///
    /// It is important to have a consistent way of specifying access levels.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @access private @public */
    ///
    /// /** @access invalidlevel */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @access private */
    ///
    /// /** @private */
    /// ```
    CheckAccess,
    jsdoc,
    restriction
);

const ACCESS_LEVELS: phf::Set<&'static str> = phf_set! {
    "package",
    "private",
    "protected",
    "public",
};

impl Rule for CheckAccess {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_access_tag_name = settings.resolve_tag_name("access");

        let mut access_related_tag_names = FxHashSet::default();
        access_related_tag_names.insert(resolved_access_tag_name);
        for level in &ACCESS_LEVELS {
            access_related_tag_names.insert(settings.resolve_tag_name(level));
        }

        for jsdoc in ctx
            .semantic()
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
        {
            let mut access_related_tags_count = 0;
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                if access_related_tag_names.contains(tag_name) {
                    access_related_tags_count += 1;
                }

                // Has valid access level?
                let comment = tag.comment();
                if tag_name == resolved_access_tag_name
                    && !ACCESS_LEVELS.contains(&comment.parsed())
                {
                    ctx.diagnostic(invalid_access_level(comment.span_trimmed_first_line()));
                }

                // Has redundant access level?
                if 1 < access_related_tags_count {
                    ctx.diagnostic(redundant_access_tags(tag.kind.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"
			          /**
			           *
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @access public
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @accessLevel package
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "access": "accessLevel",
                },
              } },
            })),
        ),
        (
            r"
			      class MyClass {
			        /**
			         * @access private
			         */
			        myClassField = 1
			      }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @public
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @private
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "ignorePrivate": true,
              } },
            })),
        ),
        (
            r"
			      (function(exports, require, module, __filename, __dirname) {
			      // Module code actually lives in here
			      });
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            r"
			          /**
			           * @access foo
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @access foo
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "ignorePrivate": true,
              } },
            })),
        ),
        (
            r"
        			          /**
        			           * @accessLevel foo
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "access": "accessLevel",
                },
              } },
            })),
        ),
        (
            r"
			          /**
			           * @access
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "access": false,
                },
              } },
            })),
        ),
        (
            r"
			      class MyClass {
			        /**
			         * @access
			         */
			        myClassField = 1
			      }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @access public
			           * @public
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @access public
			           * @access private
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @access public
			           * @access private
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "ignorePrivate": true,
              } },
            })),
        ),
        (
            r"
			          /**
			           * @public
			           * @private
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        (
            r"
			          /**
			           * @public
			           * @private
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            Some(serde_json::json!({
              "settings": { "jsdoc": {
                "ignorePrivate": true,
              } },
            })),
        ),
        (
            r"
			          /**
			           * @public
			           * @public
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
    ];

    Tester::new(CheckAccess::NAME, CheckAccess::PLUGIN, pass, fail).test_and_snapshot();
}
