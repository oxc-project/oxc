use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{should_ignore_as_internal, should_ignore_as_private},
};

fn require_property_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing name in @property tag.")
        .with_help("Add a type name to this @property tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequirePropertyName;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all `@property` tags have names.
    ///
    /// ### Why is this bad?
    ///
    /// The name of a property type should be documented.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /**
    ///  * @typedef {SomeType} SomeTypedef
    ///  * @property {number}
    ///  */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /**
    ///  * @typedef {SomeType} SomeTypedef
    ///  * @property {number} foo
    ///  */
    /// ```
    RequirePropertyName,
    jsdoc,
    correctness
);

impl Rule for RequirePropertyName {
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
                let (_, name_part, _) = tag.type_name_comment();
                if name_part.is_some() {
                    continue;
                };

                ctx.diagnostic(require_property_name_diagnostic(tag_name.span));
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
              "settings": { "jsdoc": {
                "tagNamePreference": {
                  "property": "prop",
                },
              } },
            })),
        ),
    ];

    Tester::new(RequirePropertyName::NAME, RequirePropertyName::PLUGIN, pass, fail)
        .test_and_snapshot();
}
