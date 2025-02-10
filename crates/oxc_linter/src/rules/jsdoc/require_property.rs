use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The `@typedef` and `@namespace` tags must include a `@property` tag with the type Object.",
    )
    .with_help("Consider adding a `@property` tag or replacing it with a more specific type.")
    .and_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireProperty;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all `@typedef` and `@namespace` tags have `@property` tags
    /// when their type is a plain `object`, `Object`, or `PlainObject`.
    ///
    /// ### Why is this bad?
    ///
    /// Object type should have properties defined.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /**
    ///  * @typedef {Object} SomeTypedef
    ///  */
    ///
    /// /**
    ///  * @namespace {Object} SomeNamesoace
    ///  */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /**
    ///  * @typedef {Object} SomeTypedef
    ///  * @property {SomeType} propName Prop description
    ///  */
    ///
    /// /**
    ///  * @typedef {object} Foo
    ///  * @property someProp
    ///  */
    /// ```
    RequireProperty,
    jsdoc,
    correctness
);

impl Rule for RequireProperty {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_property_tag_name = settings.resolve_tag_name("property");
        let resolved_typedef_tag_name = settings.resolve_tag_name("typedef");
        let resolved_namespace_tag_name = settings.resolve_tag_name("namespace");

        for jsdoc in ctx
            .semantic()
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            let mut should_report = None;
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                if tag_name == resolved_typedef_tag_name || tag_name == resolved_namespace_tag_name
                {
                    // If this is `true`:
                    // - This JSDoc has multiple `@typedef` or `@namespace` tags
                    // - And previous `@typedef` or `@namespace` tag did not have `@property` tag
                    if let Some(span) = should_report {
                        ctx.diagnostic(require_property_diagnostic(span));
                    }

                    let (Some(type_part), _, _) = tag.type_name_comment() else {
                        continue;
                    };

                    let r#type = type_part.parsed();
                    if r#type == "Object" || r#type == "object" || r#type == "PlainObject" {
                        should_report = Some(tag.kind.span.merge(type_part.span));
                    }
                }

                if tag_name == resolved_property_tag_name {
                    // At least 1 `@property` tag is found
                    should_report = None;
                }
            }

            if let Some(span) = should_report {
                ctx.diagnostic(require_property_diagnostic(span));
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
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "property": "prop",
                },
              } },
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
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "property": "prop",
                },
              } },
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

    Tester::new(RequireProperty::NAME, RequireProperty::PLUGIN, pass, fail).test_and_snapshot();
}
