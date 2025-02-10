use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_property_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing description in @property tag.")
        .with_help("Add a description to this @property tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequirePropertyDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all `@property` tags have descriptions.
    ///
    /// ### Why is this bad?
    ///
    /// The description of a property should be documented.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /**
    ///  * @typedef {SomeType} SomeTypedef
    ///  * @property {number} foo
    ///  */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /**
    ///  * @typedef {SomeType} SomeTypedef
    ///  * @property {number} foo Foo.
    ///  */
    /// ```
    RequirePropertyDescription,
    jsdoc,
    correctness
);

impl Rule for RequirePropertyDescription {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_property_tag_name = settings.resolve_tag_name("property");

        for jsdoc in ctx
            .semantic()
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind;

                if tag_name.parsed() != resolved_property_tag_name {
                    continue;
                }
                let (_, _, comment_part) = tag.type_name_comment();
                if !comment_part.parsed().is_empty() {
                    continue;
                };

                ctx.diagnostic(require_property_description_diagnostic(tag_name.span));
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
			           * @property foo Foo.
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @namespace {SomeType} SomeName
			           * @property foo Foo.
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @class
			           * @property foo Foo.
			           */
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Typedef with multi-line property type.
			       *
			       * @typedef {object} MyType
			       * @property {function(
			       *   number
			       * )} numberEater Method which takes a number.
			       */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @prop foo ok
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
    ];

    let fail = vec![
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
			           * @property {string} foo
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
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "property": "prop",
                },
              } },
            })),
        ),
    ];

    Tester::new(RequirePropertyDescription::NAME, RequirePropertyDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}
