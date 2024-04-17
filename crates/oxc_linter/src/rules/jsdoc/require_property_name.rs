use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(require-property-name): Missing name in @property tag.")]
#[diagnostic(severity(warning), help("Add a type name to this @property tag."))]
struct RequirePropertyNameDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequirePropertyName;

declare_oxc_lint!(
    /// ### What it does
    /// Requires that all `@property` tags have names.
    ///
    /// ### Why is this bad?
    /// The name of a property type should be documented.
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
    ///  * @property {number}
    ///  */
    /// ```
    RequirePropertyName,
    correctness
);

impl Rule for RequirePropertyName {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_property_tag_name = settings.resolve_tag_name("property");

        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            for tag in jsdoc.tags() {
                let tag_kind = tag.kind;

                if tag_kind.parsed() != resolved_property_tag_name {
                    continue;
                }
                let (_, name_part, _) = tag.type_name_comment();
                if name_part.is_some() {
                    continue;
                };

                ctx.diagnostic(RequirePropertyNameDiagnostic(tag_kind.span));
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
			           * @property {string} foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @namespace {SomeType} SomeName
			           * @property {string} foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @class
			           * @property {string} foo
			           */
			      ",
            None,
            None,
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
			           * @property {string}
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @prop {string}
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

    Tester::new(RequirePropertyName::NAME, pass, fail).test_and_snapshot();
}
