use oxc_ast::{
    ast::{Expression, TSEnumMember},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn no_duplicate_enum_values_diagnostic(
    first_init_span: Span,
    second_member: &TSEnumMember,
    value: &str,
) -> OxcDiagnostic {
    let second_name = second_member.id.static_name();
    // Unwrap will never panic since violations are only reported for members
    // with initializers.
    let second_init_span = second_member.initializer.as_ref().map(GetSpan::span).unwrap();

    OxcDiagnostic::warn(format!("Duplicate enum value `{value}`"))
        .with_help(format!("Give {second_name} a unique value"))
        .with_labels([
            first_init_span.label(format!("{value} is first used as an initializer here")),
            second_init_span.label("and is re-used here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateEnumValues;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow duplicate enum member values.
    ///
    /// ### Why is this bad?
    /// Although TypeScript supports duplicate enum member values, people
    /// usually expect members to have unique values within the same enum.
    /// Duplicate values can lead to bugs that are hard to track down.
    ///
    /// ### Examples
    ///
    /// This rule disallows defining an enum with multiple members initialized
    /// to the same value. Members without initializers will not be checked.
    ///
    /// Example of **incorrect** code:
    /// ```ts
    /// enum E {
    ///     A = 0,
    ///     B = 0,
    /// }
    /// ```
    /// ```ts
    /// enum E {
    ///     A = 'A',
    ///     B = 'A',
    /// }
    /// ```
    ///
    /// Example of **correct** code:
    /// ```ts
    /// enum E {
    ///    A = 0,
    ///    B = 1,
    /// }
    /// ```
    /// ```ts
    /// enum E {
    ///    A = 'A',
    ///    B = 'B',
    /// }
    /// ```
    /// ```ts
    /// enum E {
    ///    A,
    ///    B,
    /// }
    /// ```
    NoDuplicateEnumValues,
    typescript,
    correctness
);

impl Rule for NoDuplicateEnumValues {
    #[allow(clippy::float_cmp)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumDeclaration(enum_body) = node.kind() else {
            return;
        };
        let mut seen_number_values: Vec<(f64, Span)> = vec![];
        let mut seen_string_values: FxHashMap<&str, Span> = FxHashMap::default();
        for enum_member in &enum_body.members {
            let Some(initializer) = &enum_member.initializer else {
                continue;
            };
            match initializer {
                Expression::NumericLiteral(num) => {
                    if let Some((_, old_span)) =
                        seen_number_values.iter().find(|(v, _)| *v == num.value)
                    {
                        ctx.diagnostic(no_duplicate_enum_values_diagnostic(
                            *old_span,
                            enum_member,
                            num.raw.as_ref().unwrap().as_str(),
                        ));
                    } else {
                        seen_number_values.push((num.value, num.span));
                    }
                }
                Expression::StringLiteral(s) => {
                    if let Some(old_span) = seen_string_values.insert(s.value.as_str(), s.span) {
                        // Formatting here for prettier messages. This makes it
                        // look like "Duplicate enum value 'A'"
                        let v = format!("'{}'", s.value);
                        ctx.diagnostic(no_duplicate_enum_values_diagnostic(
                            old_span,
                            enum_member,
                            &v,
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			enum E {
			  A,
			  B,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 1,
			  B,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 1,
			  B = 2,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 'A',
			  B = 'B',
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 'A',
			  B = 'B',
			  C,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 'A',
			  B = 'B',
			  C = 2,
			  D = 1 + 1,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 3,
			  B = 2,
			  C,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 'A',
			  B = 'B',
			  C = 2,
			  D = foo(),
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = '',
			  B = 0,
			}
			    ",
            None,
        ),
        (
            "
			enum E {
			  A = 0,
			  B = -0,
			  C = NaN,
			}
			    ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
			enum E {
			  A = 1,
			  B = 1,
			}
			      ",
            None,
        ),
        (
            "
			enum E {
			  A = 'A',
			  B = 'A',
			}
			      ",
            None,
        ),
        (
            "
			enum E {
			  A = 'A',
			  B = 'A',
			  C = 1,
			  D = 1,
			}
			      ",
            None,
        ),
    ];

    Tester::new(NoDuplicateEnumValues::NAME, NoDuplicateEnumValues::PLUGIN, pass, fail)
        .test_and_snapshot();
}
