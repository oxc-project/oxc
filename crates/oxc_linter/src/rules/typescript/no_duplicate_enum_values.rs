use oxc_ast::ast::Expression;
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(no-duplicate-enum-values): Disallow duplicate enum member values")]
#[diagnostic(
    severity(warning),
    help("Duplicate values can lead to bugs that are hard to track down")
)]
struct NoDuplicateEnumValuesDiagnostic(#[label] Span, #[label] Span);

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateEnumValues;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow duplicate enum member values.
    ///
    /// ### Why is this bad?
    /// Although TypeScript supports duplicate enum member values, people usually expect members to have unique values within the same enum. Duplicate values can lead to bugs that are hard to track down.
    ///
    /// ### Example
    /// ```javascript
    /// enum E {
    //    A = 0,
    //    B = 0,
    //  }
    /// ```
    NoDuplicateEnumValues,
    pedantic
);

impl Rule for NoDuplicateEnumValues {
    #[allow(clippy::float_cmp)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumBody(enum_body) = node.kind() else { return };
        let mut seen_number_values: Vec<(f64, Span)> = vec![];
        let mut seen_string_values: FxHashMap<&Atom, Span> = FxHashMap::default();
        for enum_member in &enum_body.members {
            let Some(initializer) = &enum_member.initializer else { continue };
            match initializer {
                Expression::NumberLiteral(num) => {
                    if let Some((_, old_span)) =
                        seen_number_values.iter().find(|(v, _)| *v == num.value)
                    {
                        ctx.diagnostic(NoDuplicateEnumValuesDiagnostic(*old_span, num.span));
                    } else {
                        seen_number_values.push((num.value, num.span));
                    }
                }
                Expression::StringLiteral(s) => {
                    if let Some(old_span) = seen_string_values.insert(&s.value, s.span) {
                        ctx.diagnostic(NoDuplicateEnumValuesDiagnostic(old_span, s.span));
                    }
                }
                _ => {}
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

    Tester::new(NoDuplicateEnumValues::NAME, pass, fail).test_and_snapshot();
}
