use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
enum CheckAccessDiagnostic {
    #[error("eslint-plugin-jsdoc(check-access): Invalid access level is specified.")]
    #[diagnostic(
        severity(warning),
        help("Valid access levels are `package`, `private`, `protected`, and `public`.")
    )]
    InvalidAccessLevel(#[label] Span),

    #[error("eslint-plugin-jsdoc(check-access): Mixing of @access with @public, @private, @protected, or @package on the same doc block.")]
    #[diagnostic(
        severity(warning),
        help("There should be only one instance of access tag in a JSDoc comment.")
    )]
    RedundantAccessTags(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct CheckAccess;

declare_oxc_lint!(
    /// ### What it does
    /// Checks that `@access` tags use one of the following values:
    /// - "package", "private", "protected", "public"
    ///
    /// Also reports:
    /// - Mixing of `@access` with `@public`, `@private`, `@protected`, or `@package` on the same doc block.
    /// - Use of multiple instances of `@access` (or the `@public`, etc) on the same doc block.
    ///
    /// ### Why is this bad?
    /// It is important to have a consistent way of specifying access levels.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** @access private */
    ///
    /// /** @private */
    ///
    /// // Failing
    /// /** @access private @public */
    ///
    /// /** @access invalidlevel */
    /// ```
    CheckAccess,
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
        access_related_tag_names.insert(resolved_access_tag_name.to_string());
        for level in &ACCESS_LEVELS {
            access_related_tag_names.insert(settings.resolve_tag_name(level));
        }

        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            let mut access_related_tags_count = 0;
            for (span, tag) in jsdoc.tags() {
                if access_related_tag_names.contains(tag.kind) {
                    access_related_tags_count += 1;
                }

                // Has valid access level?
                if tag.kind == resolved_access_tag_name && !ACCESS_LEVELS.contains(&tag.comment()) {
                    ctx.diagnostic(CheckAccessDiagnostic::InvalidAccessLevel(*span));
                }

                // Has redundant access level?
                if 1 < access_related_tags_count {
                    ctx.diagnostic(CheckAccessDiagnostic::RedundantAccessTags(*span));
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
              "jsdoc": {
                "tagNamePreference": {
                  "access": "accessLevel",
                },
              },
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
              "jsdoc": {
                "ignorePrivate": true,
              },
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
              "jsdoc": {
                "ignorePrivate": true,
              },
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
              "jsdoc": {
                "tagNamePreference": {
                  "access": "accessLevel",
                },
              },
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
              "jsdoc": {
                "tagNamePreference": {
                  "access": false,
                },
              },
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
              "jsdoc": {
                "ignorePrivate": true,
              },
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
              "jsdoc": {
                "ignorePrivate": true,
              },
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

    Tester::new(CheckAccess::NAME, pass, fail).test_and_snapshot();
}
