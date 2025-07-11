use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::TSTypeName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn type_diagnostic(banned_type: &str, suggested_type: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use {banned_type:?} as a type. Use \"{suggested_type}\" instead"
    ))
    .with_label(span)
}

fn type_literal(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer explicitly define the object shape")
        .with_help("This type means \"any non-nullish value\", which is slightly better than 'unknown', but it's still a broad type")
        .with_label(span)
}

fn function(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Don't use `Function` as a type")
        .with_help("The `Function` type accepts any function-like value")
        .with_label(span)
}

fn object(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("'The `Object` type actually means \"any non-nullish value\"")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BanTypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule bans specific types and can suggest alternatives. Note that it does not ban the corresponding runtime objects from being used.
    ///
    /// ### Why is this bad?
    ///
    /// Some built-in types have aliases, while some types are considered dangerous or harmful. It's often a good idea to ban certain types to help with consistency and safety.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// let foo: String = 'foo';
    ///
    /// let bar: Boolean = true;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// let foo: string = 'foo';
    ///
    /// let bar: boolean = true;
    /// ```
    BanTypes,
    typescript,
    pedantic,
    pending
);

impl Rule for BanTypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSTypeReference(ty) => {
                let name = match &ty.type_name {
                    TSTypeName::IdentifierReference(v) => &v.name,
                    TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => return,
                };

                match name.as_str() {
                    "String" | "Boolean" | "Number" | "Symbol" | "BigInt" => {
                        ctx.diagnostic(type_diagnostic(
                            name.as_str(),
                            &name.as_str().cow_to_ascii_lowercase(),
                            ty.span,
                        ));
                    }
                    "Object" => {
                        ctx.diagnostic(object(ty.span));
                    }
                    "Function" => {
                        ctx.diagnostic(function(ty.span));
                    }
                    _ => {}
                }
            }
            AstKind::TSTypeLiteral(ty) => {
                if ty.members.is_empty() {
                    ctx.diagnostic(type_literal(ty.span));
                }
            }
            _ => {}
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
        ("let a = Object();", None),
        ("let foo: { x: number; y: number } = { x: 1, y: 1 };", None),
        ("let g = Object.create(null);", None),
        ("let h = String(false);", None),
        ("let b: undefined;", None),
        ("let c: null;", None),
        ("let a: [];", None),
        ("let tuple: [boolean, string] = [true, \"hello\"];", None),
        (
            "
  type Props = {
    onClick: () => void;
  }",
            None,
        ),
    ];

    let fail = vec![
        ("let a: String;", None),
        ("let b: Boolean;", None),
        ("let c: Number;", None),
        ("let d: Symbol;", None),
        ("let e: BigInt;", None),
        ("let f: Object;", None),
        ("let g: Function;", None),
        ("let h: {}; ", None),
        ("let i: { b: String };", None),
        ("let j: { c: String };", None),
        ("function foo(arg0: String) {}", None),
        ("'foo' as String;", None),
        ("'baz' as Function;", None),
        ("let d: Symbol = Symbol('foo');", None),
        ("let baz: [boolean, Boolean] = [true, false];", None),
        ("let z = true as Boolean;", None),
        ("type Props = {};", None),
        ("let fn: Function = () => true", None),
        ("const str: String = 'foo';", None),
        ("const bool: Boolean = true;", None),
        ("const num: Number = 1;", None),
        ("const symb: Symbol = Symbol('foo');", None),
        ("const bigInt: BigInt = 1n;", None),
        (
            "const emptyObj: {

        } = {foo: \"bar\"};",
            None,
        ),
        ("const emptyEmptyObj: {} = { };", None),
        (
            "
        class Test<T = Boolean> extends Foo<String> implements Bar<Object> {
          constructor(foo: String | Object | Function) {}

          arg(): Array<String> {
            const foo: String = 1 as String;
          }
        }",
            None,
        ),
        (
            "
type Props = {
  onClick: Function
}",
            None,
        ),
    ];

    Tester::new(BanTypes::NAME, BanTypes::PLUGIN, pass, fail).test_and_snapshot();
}
