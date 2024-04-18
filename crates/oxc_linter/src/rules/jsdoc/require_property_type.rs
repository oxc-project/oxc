use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(require-property-type): Missing type in @property tag.")]
#[diagnostic(severity(warning), help("Add a {{type}} to this @property tag."))]
struct RequirePropertyTypeDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequirePropertyType;

declare_oxc_lint!(
    /// ### What it does
    /// Requires that each `@property` tag has a type value (within curly brackets).
    ///
    /// ### Why is this bad?
    /// The type of a property should be documented.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /**
    ///  * @typedef {SomeType} SomeTypedef
    ///  * @property {number} foo
    ///  */
    ///
    /// // Failing
    /// /**
    ///  * @typedef {SomeType} SomeTypedef
    ///  * @property foo
    ///  */
    /// ```
    RequirePropertyType,
    correctness
);

impl Rule for RequirePropertyType {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_property_tag_name = settings.resolve_tag_name("property");

        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            for tag in jsdoc.tags() {
                let tag_kind = tag.kind;

                if tag_kind.parsed() != resolved_property_tag_name {
                    continue;
                }
                let (type_part, _, _) = tag.type_name_comment();
                if type_part.is_some() {
                    continue;
                };

                ctx.diagnostic(RequirePropertyTypeDiagnostic(tag_kind.span));
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
			           * @typedef {SomeType} SomeTypedef
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property {number} foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @namespace {SomeType} SomeName
			           * @property {number} foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @class
			           * @property {number} foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @prop {string} foo
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
    ];

    let fail = vec![
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property foo bar
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @prop foo
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
    ];

    Tester::new(RequirePropertyType::NAME, pass, fail).test_and_snapshot();
}
