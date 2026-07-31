use oxc_ast::{AstKind, ast::UnaryOperator};
use oxc_checker::types::{Ty, TypeData};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::GetSpan;

use crate::{AstNode, context::LintContext, native_type_aware::TypedApiContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeUnaryMinus;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows using the unary minus operator on a value which is not of type 'number' | 'bigint'.
    ///
    /// ### Why is this bad?
    ///
    /// The unary minus operator should only be used on numeric values. Using it on other types can lead to unexpected behavior due to JavaScript's type coercion rules.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const str: string;
    /// const result1 = -str; // unsafe on string
    ///
    /// declare const bool: boolean;
    /// const result3 = -bool; // unsafe on boolean
    ///
    /// declare const obj: object;
    /// const result4 = -obj; // unsafe on object
    ///
    /// declare const arr: any[];
    /// const result5 = -arr; // unsafe on array
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const num: number;
    /// const result1 = -num; // safe
    ///
    /// declare const bigint: bigint;
    /// const result2 = -bigint; // safe
    ///
    /// const literal = -42; // safe
    ///
    /// const bigintLiteral = -42n; // safe
    ///
    /// declare const union: number | bigint;
    /// const result3 = -union; // safe
    ///
    /// declare const anyValue: any;
    /// const result4 = -anyValue; // allowed by the upstream rule
    ///
    /// declare const neverValue: never;
    /// const result5 = -neverValue; // safe
    ///
    /// // Convert to number first if needed
    /// declare const str: string;
    /// const result6 = -Number(str); // safe conversion
    /// ```
    NoUnsafeUnaryMinus,
    typescript,
    correctness,
    version = "1.12.0",
    short_description = "This rule disallows using the unary minus operator on a value which is not of type 'number' | 'bigint'.",
);

impl Rule for NoUnsafeUnaryMinus {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UnaryExpression(unary) = node.kind() else {
            return;
        };
        if unary.operator != UnaryOperator::UnaryNegation {
            return;
        }

        let span = unary.argument.span();
        let Some(api) = ctx.type_aware() else {
            return;
        };
        let Some(ty) = api.type_at_span(span) else {
            return;
        };
        let Some(unsafe_ty) = first_unsafe_type(api, ty) else {
            return;
        };
        let type_name = api.type_name(span, unsafe_ty).unwrap_or("unknown");

        ctx.diagnostic(
            OxcDiagnostic::warn(format!(
                "Argument of unary negation should be assignable to number | bigint but is {type_name} instead."
            ))
            .with_label(unary.span),
        );
    }
}

fn first_unsafe_type<'a>(api: &TypedApiContext<'a>, ty: Ty<'a>) -> Option<Ty<'a>> {
    match api.type_data(ty) {
        TypeData::Any
        | TypeData::Never
        | TypeData::Number
        | TypeData::NumberLiteral(_)
        | TypeData::Bigint
        | TypeData::BigIntLiteral(_) => None,
        TypeData::Union(union) => union.types.iter().find_map(|ty| first_unsafe_type(api, *ty)),
        TypeData::Intersection(intersection) => {
            if intersection.types.iter().any(|ty| first_unsafe_type(api, *ty).is_none()) {
                None
            } else {
                Some(ty)
            }
        }
        _ => Some(ty),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "+42;",
        "-42;",
        "-42n;",
        "(a: number) => -a;",
        "(a: bigint) => -a;",
        "(a: number | bigint) => -a;",
        "(a: any) => -a;",
        "(a: 1 | 2) => -a;",
        "(a: string) => +a;",
        "(a: number[]) => -a[0];",
        "<T,>(t: T & number) => -t;",
        "(a: { x: number }) => -a.x;",
        "(a: never) => -a;",
        "<T extends number>(t: T) => -t;",
    ];

    let fail = vec![
        "(a: string) => -a;",
        "(a: {}) => -a;",
        "(a: number[]) => -a;",
        "-'hello';",
        "-`hello`;",
        "(a: { x: number }) => -a;",
        "(a: unknown) => -a;",
        "(a: void) => -a;",
        "<T,>(t: T) => -t;",
    ];

    Tester::new(NoUnsafeUnaryMinus::NAME, NoUnsafeUnaryMinus::PLUGIN, pass, fail)
        .test_and_snapshot();
}
