use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::JSDocTag;
use oxc_span::Span;
use phf::phf_set;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(check-access): TODO")]
#[diagnostic(severity(warning), help("TODO"))]
struct CheckAccessDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct CheckAccess;

declare_oxc_lint!(
    /// ### What it does
    /// TODO!
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    CheckAccess,
    correctness
);

const ACCESS_LEVELS: phf::Set<&'static str> = phf_set! {
    "package",
    "private",
    "protected",
    "public",
};

// TODO: Diagnostic message
// TODO: Diagnostic span, how to get it?
// TODO: Fixer?
// TODO: Check all tests are surely covered

impl Rule for CheckAccess {
    fn run_once(&self, ctx: &LintContext) {
        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            let tags = jsdoc.tags();

            if has_multiple_tags(tags) {
                ctx.diagnostic(CheckAccessDiagnostic(Span::default()));
            }
            if has_invalid_access_tag(tags) {
                ctx.diagnostic(CheckAccessDiagnostic(Span::default()));
            }
        }
    }
}

fn has_multiple_tags(tags: &[JSDocTag]) -> bool {
    1 < tags
        .iter()
        .map(JSDocTag::tag_name)
        // TODO: Apply settings.tag_name_preference here
        .filter(|tag_name| *tag_name == "access" || ACCESS_LEVELS.contains(tag_name))
        .count()
}

fn has_invalid_access_tag(tags: &[JSDocTag]) -> bool {
    // TODO: Before hand, need to update settings.rs
    // https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md
    // Too many settings there and looks complicated...
    //
    // TODO: Apply settings.tag_name_preference here
    // https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md#alias-preference
    let resolved_access_tag_name = "access";
    let access_tags = tags.iter().filter(|tag| tag.tag_name() == resolved_access_tag_name);

    for access_tag in access_tags {
        if !ACCESS_LEVELS.contains(access_tag.comment.as_str()) {
            return true;
        }
    }

    false
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
        // (
        //     r"
        // 			          /**
        // 			           * @accessLevel foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "access": "accessLevel",
        //         },
        //       },
        //     })),
        // ),
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
