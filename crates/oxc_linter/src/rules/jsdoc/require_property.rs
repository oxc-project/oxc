use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(require-property): The `@typedef` and `@namespace` tags must include a `@property` tag with the type Object.")]
#[diagnostic(
    severity(warning),
    help("Consider adding a `@property` tag or replacing it with a more specific type.")
)]

struct RequirePropertyDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireProperty;

declare_oxc_lint!(
    /// ### What it does
    /// Requires that all `@typedef` and `@namespace` tags have `@property` tags
    /// when their type is a plain `object`, `Object`, or `PlainObject`.
    ///
    /// ### Why is this bad?
    /// Object type should have properties defined.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /**
    ///  * @typedef {Object} SomeTypedef
    ///  * @property {SomeType} propName Prop description
    ///  */
    /// /**
    ///  * @typedef {object} Foo
    ///  * @property someProp
    ///  */
    ///
    /// // Failing
    /// /**
    ///  * @typedef {Object} SomeTypedef
    ///  */
    /// /**
    ///  * @namespace {Object} SomeNamesoace
    ///  */
    /// ```
    RequireProperty,
    correctness
);

impl Rule for RequireProperty {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_property_tag_name = settings.resolve_tag_name("property");

        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            let mut should_report = None;
            for tag in jsdoc.tags() {
                let tag_kind = tag.kind.parsed();

                if tag_kind == "typedef" || tag_kind == "namespace" {
                    // If this is `true`:
                    // - This JSDoc has multiple `@typedef` or `@namespace` tags
                    // - And previous `@typedef` or `@namespace` tag did not have `@property` tag
                    if let Some(span) = should_report {
                        ctx.diagnostic(RequirePropertyDiagnostic(span));
                    }

                    let (type_part, _, _) = tag.type_name_comment();
                    let Some(type_part) = type_part else {
                        continue;
                    };

                    let r#type = type_part.parsed();
                    if r#type == "Object" || r#type == "object" || r#type == "PlainObject" {
                        should_report = Some(tag.kind.span.merge(&type_part.span));
                    }
                }

                if tag_kind == resolved_property_tag_name {
                    // At least 1 `@property` tag is found
                    should_report = None;
                }
            }

            if let Some(span) = should_report {
                ctx.diagnostic(RequirePropertyDiagnostic(span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			        /**
			         *
			         */
			      ",
            None,
            None,
        ),
        (
            "
			        /**
			         * @property
			         */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {Object} SomeTypedef
			           * @property {SomeType} propName Prop description
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {object} SomeTypedef
			           * @prop {SomeType} propName Prop description
			           */
			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "property": "prop",
                },
              },
            })),
        ),
        (
            "
			          /**
			           * @typedef {object} SomeTypedef
			           * @property
			           * // arbitrary property content
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {object<string, string>} SomeTypedef
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {string} SomeTypedef
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @namespace {object} SomeName
			           * @property {SomeType} propName Prop description
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @namespace {object} SomeName
			           * @property
			           * // arbitrary property content
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {object} SomeTypedef
			           * @property someProp
			           * @property anotherProp This with a description
			           * @property {anotherType} yetAnotherProp This with a type and desc.
			           */
			          function quux () {
			
			          }
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			          /**
			           * @typedef {object} SomeTypedef
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {object} SomeTypedefA
			           * @property someProp A should pass but B should not
			           * @typedef {object} SomeTypedefB
			           */
			      ",
            None,
            None,
        ),
        (
            "
			      class Test {
			          /**
			           * @typedef {object} SomeTypedef
			           */
			          quux () {}
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {PlainObject} SomeTypedef
			           */
			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "property": "prop",
                },
              },
            })),
        ),
        (
            "
			          /**
			           * @namespace {Object} SomeName
			           */
			      ",
            None,
            None,
        ),
    ];

    Tester::new(RequireProperty::NAME, pass, fail).test_and_snapshot();
}
