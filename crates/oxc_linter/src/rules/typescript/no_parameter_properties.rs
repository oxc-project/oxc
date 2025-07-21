use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_parameter_properties_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected parameter property")
        .with_help("Parameter properties are not erasable syntax and are incompatible with TypeScript's --erasableSyntaxOnly flag. Consider declaring the property separately.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoParameterProperties;

// Ported from <https://github.com/JoshuaKGoldberg/eslint-plugin-erasable-syntax-only/blob/main/src/rules/parameter-properties.ts>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow TypeScript parameter properties
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript 5.8 introduces the `--erasableSyntaxOnly` flag. When this flag is enabled,
    /// TypeScript will only allow you to use constructs that can be erased from a file, and
    /// will issue an error if it encounters any constructs that cannot be erased.
    ///
    /// Parameter properties are not erasable syntax because they generate runtime code
    /// that modifies the class constructor and cannot be completely removed during compilation.
    /// This makes them incompatible with the `--erasableSyntaxOnly` flag.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Values {
    ///     constructor(
    ///         private value: number,
    ///     ) {}
    /// }
    ///
    /// class ReadOnlyValues {
    ///     constructor(
    ///         readonly value: number,
    ///     ) {}
    /// }
    ///
    /// class PublicValues {
    ///     constructor(
    ///         public value: number,
    ///     ) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Values {
    ///     value: number;
    ///
    ///     constructor(value: number) {
    ///         this.value = value;
    ///     }
    /// }
    ///
    /// class ReadOnlyValues {
    ///     readonly value: number;
    ///
    ///     constructor(value: number) {
    ///         this.value = value;
    ///     }
    /// }
    /// ```
    NoParameterProperties,
    typescript,
    restriction,
    suggestion
);

impl Rule for NoParameterProperties {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::FormalParameter(property) = node.kind()
            && (property.accessibility.is_some() || property.readonly)
        {
            ctx.diagnostic_with_suggestion(
                no_parameter_properties_diagnostic(property.span),
                |fixer| {
                    // Calculate the span to delete (from start of modifiers to start of pattern)
                    let start = property.span.start;
                    let pattern_start = property.pattern.span().start;
                    let modifier_span = Span::new(start, pattern_start);
                    fixer.delete(&modifier_span)
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Valid: Regular parameter without property modifier
        r"
            class Values {
                constructor(
                    value: number,
                ) {}
            }
        ",
    ];

    let fail = vec![
        // Invalid: private parameter property
        r"
            class Values {
                constructor(
                    private value: number,
                ) {}
            }
        ",
        // Invalid: readonly parameter property
        r"
            class Values {
                constructor(
                    readonly value: number,
                ) {}
            }
        ",
        // Invalid: public parameter property
        r"
            class Values {
                constructor(
                    public value: number,
                ) {}
            }
        ",
        // Invalid: protected parameter property
        r"
            class Values {
                constructor(
                    protected value: number,
                ) {}
            }
        ",
        // Invalid: private readonly parameter property
        r"
            class Values {
                constructor(
                    private readonly value: number,
                ) {}
            }
        ",
    ];

    let fix = vec![
        // Remove private modifier
        (
            r"
            class Values {
                constructor(
                    private value: number,
                ) {}
            }
        ",
            r"
            class Values {
                constructor(
                    value: number,
                ) {}
            }
        ",
        ),
        // Remove readonly modifier
        (
            r"
            class Values {
                constructor(
                    readonly value: number,
                ) {}
            }
        ",
            r"
            class Values {
                constructor(
                    value: number,
                ) {}
            }
        ",
        ),
        // Remove public modifier
        (
            r"
            class Values {
                constructor(
                    public value: number,
                ) {}
            }
        ",
            r"
            class Values {
                constructor(
                    value: number,
                ) {}
            }
        ",
        ),
        // Remove protected modifier
        (
            r"
            class Values {
                constructor(
                    protected value: number,
                ) {}
            }
        ",
            r"
            class Values {
                constructor(
                    value: number,
                ) {}
            }
        ",
        ),
    ];

    Tester::new(NoParameterProperties::NAME, NoParameterProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
