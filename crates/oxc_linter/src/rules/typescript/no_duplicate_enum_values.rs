use oxc_ast::ast::Expression;
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Disallow duplicate enum member values")]
#[diagnostic(severity(warning))]
struct NoDuplicateEnumValuesDiagnostic(#[label] pub Span);

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
    correctness
);

impl Rule for NoDuplicateEnumValues {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumBody(enum_body) = node.kind() else { return };
        let mut seen_number_values: Vec<f64> = Vec::new();
        let mut seen_string_values: FxHashSet<&Atom> = FxHashSet::default();
        for enum_member in &enum_body.members {
            let Some(initializer) = &enum_member.initializer else { continue };
            match initializer {
                Expression::NumberLiteral(num) => {
                    if seen_number_values.contains(&num.value) {
                        ctx.diagnostic(NoDuplicateEnumValuesDiagnostic(num.span));
                    }
                    seen_number_values.push(num.value);
                }
                Expression::StringLiteral(str) => {
                    if seen_string_values.contains(&str.value) {
                        ctx.diagnostic(NoDuplicateEnumValuesDiagnostic(str.span));
                    }
                    seen_string_values.insert(&str.value);
                }
                _ => {}
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
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
