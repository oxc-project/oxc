use miette::{miette, LabeledSpan};
use oxc_diagnostics::{
    miette::{self, Diagnostic, Severity},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
enum CheckPropertyNamesDiagnostic {
    #[error("eslint-plugin-jsdoc(check-property-names): No root defined for @property path.")]
    #[diagnostic(
        severity(warning),
        help("@property path declaration `{1}` appears before any real property.")
    )]
    NoRoot(#[label] Span, String),
}

#[derive(Debug, Default, Clone)]
pub struct CheckPropertyNames;

declare_oxc_lint!(
    /// ### What it does
    /// Ensures that property names in JSDoc are not duplicated on the same block and that nested properties have defined roots.
    ///
    /// ### Why is this bad?
    /// `@property` tags with the same name can be confusing and may indicate a mistake.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /**
    ///  * @typedef {object} state
    ///  * @property {number} foo
    ///  */
    /// /**
    ///  * @typedef {object} state
    ///  * @property {object} foo
    ///  * @property {number} foo.bar
    ///  */
    ///
    /// // Failing
    /// /**
    ///  * @typedef {object} state
    ///  * @property {number} foo
    ///  * @property {string} foo
    ///  */
    ///
    /// /**
    ///  * @typedef {object} state
    ///  * @property {number} foo.bar
    ///  */
    /// ```
    CheckPropertyNames,
    correctness
);

impl Rule for CheckPropertyNames {
    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let resolved_property_tag_name = settings.resolve_tag_name("property");

        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            let mut seen: FxHashMap<&str, FxHashSet<Span>> = FxHashMap::default();
            for tag in jsdoc.tags() {
                if tag.kind.parsed() != resolved_property_tag_name {
                    continue;
                }
                let (_, name_part, _) = tag.type_name_comment();
                let Some(name_part) = name_part else {
                    continue;
                };

                let type_name = name_part.parsed();

                // Check property path has a root
                if type_name.contains('.') {
                    let mut parts = type_name.split('.').collect::<Vec<_>>();
                    // `foo[].bar` -> `foo[]`
                    parts.pop();
                    let parent_name = parts.join(".");
                    // `foo[]` -> `foo`
                    let parent_name = parent_name.trim_end_matches("[]");

                    if !seen.contains_key(&parent_name) {
                        ctx.diagnostic(CheckPropertyNamesDiagnostic::NoRoot(
                            name_part.span,
                            type_name.to_string(),
                        ));
                    }
                }

                // Check duplicated(report later)
                seen.entry(type_name).or_default().insert(name_part.span);
            }

            for (type_name, spans) in seen.iter().filter(|(_, spans)| 1 < spans.len()) {
                ctx.diagnostic(miette!(
                    severity = Severity::Warning,
                    labels = spans
                        .iter()
                        .map(|span| LabeledSpan::at(
                            (span.start as usize)..(span.end as usize),
                            "Duplicated property".to_string(),
                        ))
                        .collect::<Vec<_>>(),
                    help = format!("@property `{type_name}` is duplicated on the same block."),
                    "eslint-plugin-jsdoc(check-property-names): Duplicate @property found."
                ));
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
			           * @prop foo
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
			           * @property bar
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
			           * @property foo.foo
			           * @property bar
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * Assign the project to a list of employees.
			           * @typedef {SomeType} SomeTypedef
			           * @property {object[]} employees - The employees who are responsible for the project.
			           * @property {string} employees[].name - The name of an employee.
			           * @property {string} employees[].department - The employee's department.
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property {Error} error Exit code
			           * @property {number} [code = 1] Exit code
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @namespace {SomeType} SomeNamespace
			           * @property {Error} error Exit code
			           * @property {number} [code = 1] Exit code
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @class
			           * @property {Error} error Exit code
			           * @property {number} [code = 1] Exit code
			           */
			          function quux (code = 1) {
			            this.error = new Error('oops');
			            this.code = code;
			          }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property foo
			           * @property foo.bar
			           * @property foo.baz
			           * @property bar
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
			           * @property foo.bar
			           * @property foo.bar.baz
			           * @property foo.bar.baz.qux
			           * @property bar
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property {object[]} foo
			           * @property {object[]} foo[].bar
			           * @property {number} foo[].bar[].baz
			           * @property bar
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
			           * @property Foo.Bar
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
			           * @property Foo.Bar
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * Assign the project to a list of employees.
			           * @typedef {SomeType} SomeTypedef
			           * @property {string} employees[].name - The name of an employee.
			           * @property {string} employees[].department - The employee's department.
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
			           * @property cfg
			           * @property cfg.foo
			           * @property cfg.foo
			           */
			          function quux ({foo, bar}) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            "
			      class Test {
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property cfg
			           * @property cfg.foo
			           * @property cfg.foo
			           * @property cfg.foo
			           */
			          quux ({foo, bar}) {
			
			          }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property cfg
			           * @property cfg.foo
			           * @property [cfg.foo]
			           * @property baz
			           */
			          function quux ({foo, bar}, baz) {
			
			          }
			      ",
            None,
            None,
        ),
        (
            r#"
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property cfg
			           * @property cfg.foo
			           * @property [cfg.foo="with a default"]
			           * @property baz
			           */
			          function quux ({foo, bar}, baz) {
			
			          }
			      "#,
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property foo
			           * @property foo.bar
			           * @property foo.bar.baz.qux
			           * @property bar
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @typedef {SomeType} SomeTypedef
			           * @property {object[]} foo
			           * @property {number} foo[].bar[].baz
			           * @property bar
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

    Tester::new(CheckPropertyNames::NAME, pass, fail).test_and_snapshot();
}
