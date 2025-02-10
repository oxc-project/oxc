use cow_utils::CowUtils;
use oxc_ast::{
    ast::{Expression, TSTypeName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_wrapper_object_types(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use wrapper object types.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoWrapperObjectTypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of wrapper object types.
    ///
    /// ### Why is this bad?
    ///
    /// Wrapper object types are types that are defined in the global scope and are not primitive types. These types are not recommended to be used in TypeScript code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let myBigInt: BigInt;
    /// let myBoolean: Boolean;
    /// let myNumber: Number;
    /// let myString: String;
    /// let mySymbol: Symbol;
    ///
    /// let myObject: Object = 'allowed by TypeScript';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let myBigint: bigint;
    /// let myBoolean: boolean;
    /// let myNumber: number;
    /// let myString: string;
    /// let mySymbol: symbol;
    ///
    /// let myObject: object = "Type 'string' is not assignable to type 'object'.";
    /// ```
    NoWrapperObjectTypes,
    typescript,
    correctness,
    fix
);

impl Rule for NoWrapperObjectTypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (ident_name, ident_span, reference_id) = match node.kind() {
            AstKind::TSTypeReference(type_ref) => {
                if let TSTypeName::IdentifierReference(type_name) = &type_ref.type_name {
                    (type_name.name.as_str(), type_name.span, type_name.reference_id())
                } else {
                    return;
                }
            }
            AstKind::TSClassImplements(ts_class_implements) => {
                if let TSTypeName::IdentifierReference(type_name) = &ts_class_implements.expression
                {
                    (type_name.name.as_str(), type_name.span, type_name.reference_id())
                } else {
                    return;
                }
            }
            AstKind::TSInterfaceHeritage(ts_interface_heritage) => {
                if let Expression::Identifier(extends) = &ts_interface_heritage.expression {
                    (extends.name.as_str(), extends.span, extends.reference_id())
                } else {
                    return;
                }
            }
            _ => {
                return;
            }
        };

        if matches!(ident_name, "BigInt" | "Boolean" | "Number" | "Object" | "String" | "Symbol") {
            if ctx.symbols().get_reference(reference_id).symbol_id().is_some() {
                return;
            }

            let can_fix = matches!(node.kind(), AstKind::TSTypeReference(_));

            if can_fix {
                ctx.diagnostic_with_fix(no_wrapper_object_types(ident_span), |fixer| {
                    fixer.replace(ident_span, ident_name.cow_to_ascii_lowercase())
                });
            } else {
                ctx.diagnostic(no_wrapper_object_types(ident_span));
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let value: NumberLike;",
        "let value: Other;",
        "let value: bigint;",
        "let value: boolean;",
        "let value: never;",
        "let value: null;",
        "let value: number;",
        "let value: symbol;",
        "let value: undefined;",
        "let value: unknown;",
        "let value: void;",
        "let value: () => void;",
        "let value: () => () => void;",
        "let Bigint;",
        "let Boolean;",
        "let Never;",
        "let Null;",
        "let Number;",
        "let Symbol;",
        "let Undefined;",
        "let Unknown;",
        "let Void;",
        "interface Bigint {}",
        "interface Boolean {}",
        "interface Never {}",
        "interface Null {}",
        "interface Number {}",
        "interface Symbol {}",
        "interface Undefined {}",
        "interface Unknown {}",
        "interface Void {}",
        "type Bigint = {};",
        "type Boolean = {};",
        "type Never = {};",
        "type Null = {};",
        "type Number = {};",
        "type Symbol = {};",
        "type Undefined = {};",
        "type Unknown = {};",
        "type Void = {};",
        "class MyClass extends Number {}",
        "
        	      type Number = 0 | 1;
        	      let value: Number;
        	    ",
        "
        	      type Bigint = 0 | 1;
        	      let value: Bigint;
        	    ",
        "
        	      type T<Symbol> = Symbol;
        	      type U<UU> = UU extends T<infer Function> ? Function : never;
        	    ",
    ];

    let fail = vec![
        "let value: BigInt;",
        "let value: Boolean;",
        "let value: Number;",
        "let value: Object;",
        "let value: String;",
        "let value: Symbol;",
        "let value: Number | Symbol;",
        "let value: { property: Number };",
        "0 as Number;",
        "type MyType = Number;",
        "type MyType = [Number];",
        "class MyClass implements Number {}",
        "interface MyInterface extends Number {}",
        "type MyType = Number & String;",
    ];

    let fix = vec![
        ("let value: BigInt;", "let value: bigint;", None),
        ("let value: Boolean;", "let value: boolean;", None),
        ("let value: Number;", "let value: number;", None),
        ("let value: Object;", "let value: object;", None),
        ("let value: String;", "let value: string;", None),
        ("let value: Symbol;", "let value: symbol;", None),
        ("let value: Number | Symbol;", "let value: number | symbol;", None),
        ("let value: { property: Number };", "let value: { property: number };", None),
        ("0 as Number;", "0 as number;", None),
        ("type MyType = Number;", "type MyType = number;", None),
        ("type MyType = [Number];", "type MyType = [number];", None),
        ("type MyType = Number & String;", "type MyType = number & string;", None),
    ];

    Tester::new(NoWrapperObjectTypes::NAME, NoWrapperObjectTypes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
