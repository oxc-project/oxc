use oxc_ast::{
    AstKind,
    ast::{TSType, TSTypeParameter, TSTypeParameterDeclaration},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{FileExtension, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_unnecessary_type_constraint_diagnostic(
    generic_type: &str,
    constraint: &str,
    span: Span,
    constraint_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "constraining the generic type {generic_type:?} to {constraint:?} does nothing and is unnecessary"
    ))
    .with_help(format!("Remove the unnecessary {constraint:?} constraint"))
    .with_labels([span, constraint_span])
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryTypeConstraint;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary constraints on generic types.
    ///
    /// ### Why is this bad?
    ///
    /// Generic type parameters (`<T>`) in TypeScript may be "constrained" with an `extends`
    /// keyword. When no `extends` is provided, type parameters default a constraint to `unknown`.
    /// It is therefore redundant to `extend` from `any` or `unknown`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// interface FooAny<T extends any> {}
    /// interface FooUnknown<T extends unknown> {}
    ///
    /// type BarAny<T extends any> = {};
    /// type BarUnknown<T extends unknown> = {};
    ///
    /// const QuuxAny = <T extends any>() => {};
    ///
    /// function QuuzAny<T extends any>() {}
    /// ```
    ///
    /// ```typescript
    /// class BazAny<T extends any> {
    ///   quxAny<U extends any>() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// interface Foo<T> {}
    ///
    /// type Bar<T> = {};
    ///
    /// const Quux = <T>() => {};
    ///
    /// function Quuz<T>() {}
    /// ```
    ///
    /// ```typescript
    /// class Baz<T> {
    ///   qux<U>() {}
    /// }
    /// ```
    NoUnnecessaryTypeConstraint,
    typescript,
    suspicious,
    suggestion,
    version = "0.0.6",
    short_description = "Disallow unnecessary constraints on generic types.",
);

impl Rule for NoUnnecessaryTypeConstraint {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSTypeParameterDeclaration(decl) = node.kind() else {
            return;
        };

        for param in &decl.params {
            let Some(ty) = &param.constraint else {
                continue;
            };

            let (value, ty_span) = match ty {
                TSType::TSAnyKeyword(t) => ("any", t.span),
                TSType::TSUnknownKeyword(t) => ("unknown", t.span),
                _ => continue,
            };

            ctx.diagnostic_with_suggestion(
                no_unnecessary_type_constraint_diagnostic(
                    param.name.name.as_str(),
                    value,
                    param.name.span,
                    ty_span,
                ),
                |fixer| {
                    let replacement = if should_add_trailing_comma(decl, node, param, ty_span, ctx)
                    {
                        ","
                    } else {
                        ""
                    };

                    let fix_span = Span::new(param.name.span.end, ty_span.end);

                    fixer.replace(fix_span, replacement)
                },
            );
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn should_add_trailing_comma(
    decl: &TSTypeParameterDeclaration,
    node: &AstNode,
    param: &TSTypeParameter,
    constraint_span: Span,
    ctx: &LintContext,
) -> bool {
    if !matches!(ctx.nodes().parent_kind(node.id()), AstKind::ArrowFunctionExpression(_))
        || decl.params.len() != 1
        || param.default.is_some()
        || !requires_generic_declaration_disambiguation(ctx)
    {
        return false;
    }

    ctx.source_range(Span::new(constraint_span.end, decl.span.end))
        .bytes()
        .find(|b| !b.is_ascii_whitespace())
        .is_none_or(|b| b != b',')
}

fn requires_generic_declaration_disambiguation(ctx: &LintContext<'_>) -> bool {
    ctx.source_type().is_jsx()
        || matches!(
            ctx.source_type().extension(),
            Some(FileExtension::Mts | FileExtension::Cts | FileExtension::Tsx)
        )
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};
    use std::path::PathBuf;

    let pass = vec![
        "function data() {}",
        "function data<T>() {}",
        "function data<T, U>() {}",
        "function data<T extends number>() {}",
        "function data<T extends number | string>() {}",
        "function data<T extends any | number>() {}",
        "type X = any; function data<T extends X>() {}",
        "const data = () => {};",
        "const data = <T, >() => {};",
        "const data = <T, U>() => {};",
        "const data = <T extends number>() => {};",
        "const data = <T extends number | string>() => {};",
    ];

    let fail = vec![
        "function data<T extends any>() {}",
        "function data<T extends any, U>() {}",
        "function data<T, U extends any>() {}",
        "function data<T extends any, U extends T>() {}",
        "const data = <T extends any>() => {};",
        "const data = <T extends any,>() => {};",
        "const data = <T extends any, >() => {};",
        "const data = <T extends any ,>() => {};",
        "const data = <T extends any , >() => {};",
        "const data = <T extends any = unknown>() => {};",
        "const data = <T extends any, U extends any>() => {};",
        "function data<T extends unknown>() {}",
        "const data = <T extends any>() => {};",
        "const data = <T extends unknown>() => {};",
        "class Data<T extends unknown> {}",
        "const Data = class<T extends unknown> {};",
        "class Data { member<T extends unknown>() {} }",
        "const Data = class { member<T extends unknown>() {} };",
        "interface Data<T extends unknown> {}",
        "type Data<T extends unknown> = {};",
    ];

    let fix: Vec<ExpectFixTestCase> = vec![
        ("function data<T extends any>() {}", "function data<T>() {}").into(),
        ("function data<T extends any, U>() {}", "function data<T, U>() {}").into(),
        ("function data<T, U extends any>() {}", "function data<T, U>() {}").into(),
        ("function data<T extends any, U extends T>() {}", "function data<T, U extends T>() {}")
            .into(),
        ("const data = <T extends any>() => {};", "const data = <T,>() => {};").into(),
        ("const data = <T extends any,>() => {};", "const data = <T,>() => {};").into(),
        ("const data = <T extends any, >() => {};", "const data = <T, >() => {};").into(),
        ("const data = <T extends any ,>() => {};", "const data = <T ,>() => {};").into(),
        ("const data = <T extends any , >() => {};", "const data = <T , >() => {};").into(),
        ("const data = <T extends any = unknown>() => {};", "const data = <T = unknown>() => {};")
            .into(),
        ("const data = <T extends any, U extends any>() => {};", "const data = <T, U>() => {};")
            .into(),
        ("function data<T extends unknown>() {}", "function data<T>() {}").into(),
        (
            "const data = <T extends any>() => {};",
            "const data = <T>() => {};",
            None,
            Some(PathBuf::from("no_unnecessary_type_constraint.ts")),
        )
            .into(),
        (
            "const data = <T extends any>() => {};",
            "const data = <T,>() => {};",
            None,
            Some(PathBuf::from("no_unnecessary_type_constraint.mts")),
        )
            .into(),
        (
            "const data = <T extends any>() => {};",
            "const data = <T,>() => {};",
            None,
            Some(PathBuf::from("no_unnecessary_type_constraint.cts")),
        )
            .into(),
        ("const data = <T extends unknown>() => {};", "const data = <T,>() => {};").into(),
        ("class Data<T extends unknown> {}", "class Data<T> {}").into(),
        ("const Data = class<T extends unknown> {};", "const Data = class<T> {};").into(),
        ("class Data { member<T extends unknown>() {} }", "class Data { member<T>() {} }").into(),
        (
            "const Data = class { member<T extends unknown>() {} };",
            "const Data = class { member<T>() {} };",
        )
            .into(),
        ("interface Data<T extends unknown> {}", "interface Data<T> {}").into(),
        ("type Data<T extends unknown> = {};", "type Data<T> = {};").into(),
    ];

    Tester::new(NoUnnecessaryTypeConstraint::NAME, NoUnnecessaryTypeConstraint::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
