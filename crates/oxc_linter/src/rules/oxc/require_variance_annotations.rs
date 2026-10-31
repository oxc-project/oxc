use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn require_variance_annotations_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Type parameter is missing a variance annotation.")
        .with_help("Add `in`, `out`, or `in out` to describe how the type parameter is used.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireVarianceAnnotations;

// Inspired by <https://github.com/DetachHead/eslint-plugin-detachhead/blob/master/src/rules/require-variance-annotations.ts>.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires explicit variance annotations on type parameters of classes,
    /// interfaces, and type aliases for object, function, constructor, and mapped types.
    /// Parentheses and references to local object, function, and constructor
    /// type aliases are also supported.
    /// Types that require type checking to resolve, such as imported types,
    /// qualified names, instantiated mapped types, unions, and intersections,
    /// are not checked.
    ///
    /// ### Why is this bad?
    ///
    /// Explicit variance documents how a generic type uses its type parameters:
    /// `in` for inputs, `out` for outputs, and `in out` for both. It can also help
    /// TypeScript compare types efficiently and catch changes to a public API's
    /// variance. Variance annotations require TypeScript 4.7 or later.
    ///
    /// This rule does not infer which annotation is appropriate or provide an autofix.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// interface Producer<T> {
    ///   get(): T;
    /// }
    /// type Consumer<T> = (value: T) => void;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface Producer<out T> {
    ///   get(): T;
    /// }
    /// type Consumer<in T> = (value: T) => void;
    /// type State<in out T> = { get: () => T; set: (value: T) => void };
    ///
    /// // TypeScript does not allow variance annotations here.
    /// type Optional<T> = T | undefined;
    /// function identity<T>(value: T): T { return value; }
    /// ```
    RequireVarianceAnnotations,
    oxc,
    restriction,
    version = "next",
    short_description = "Require explicit variance annotations on generic type parameters.",
);

impl Rule for RequireVarianceAnnotations {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSTypeParameterDeclaration(declaration) = node.kind() else {
            return;
        };
        if declaration.params.iter().all(|param| param.r#in || param.out) {
            return;
        }

        match ctx.nodes().parent_kind(node.id()) {
            AstKind::Class(_) | AstKind::TSInterfaceDeclaration(_) => {}
            AstKind::TSTypeAliasDeclaration(alias)
                if supports_variance(&alias.type_annotation, ctx) => {}
            _ => return,
        }

        for param in &declaration.params {
            if !param.r#in && !param.out {
                ctx.diagnostic(require_variance_annotations_diagnostic(param.name.span));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn supports_variance<'a>(mut ty: &'a TSType<'a>, ctx: &LintContext<'a>) -> bool {
    let mut seen_aliases = FxHashSet::default();
    loop {
        match ty {
            TSType::TSTypeLiteral(_) | TSType::TSFunctionType(_) | TSType::TSConstructorType(_) => {
                return true;
            }
            // Instantiating a homomorphic mapped type can produce an array or
            // primitive, which cannot have variance annotations.
            TSType::TSMappedType(_) => return seen_aliases.is_empty(),
            TSType::TSParenthesizedType(parenthesized) => ty = &parenthesized.type_annotation,
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                    return false;
                };
                let Some(symbol_id) =
                    ctx.scoping().get_reference(identifier.reference_id()).symbol_id()
                else {
                    return false;
                };
                let AstKind::TSTypeAliasDeclaration(alias) =
                    ctx.symbol_declaration(symbol_id).kind()
                else {
                    return false;
                };
                // Invalid recursive aliases must not cause an infinite loop.
                if !seen_aliases.insert(symbol_id) {
                    return false;
                }
                ty = &alias.type_annotation;
            }
            _ => return false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Plain {}",
        "interface Plain {}",
        "class Consumer<in T> { consume!: (value: T) => void; }",
        "class Producer<out T> { value!: T; }",
        "class State<in out T> { get!: () => T; set!: (value: T) => void; }",
        "interface Transform<in T, out U> { transform(value: T): U; }",
        "type ObjectType<out T> = { value: T };",
        "type Callback<in T> = (value: T) => void;",
        "type Factory<out T> = new () => { value: T };",
        "type Mapped<out T> = { [K in keyof T]: T[K] };",
        "declare class Producer<out T> { value: T; }",
        "type Optional<T> = T | undefined;",
        "type Intersection<T> = { value: T } & { other: T };",
        "type ArrayType<T> = T[];",
        "type Tuple<T> = [T];",
        "type Primitive<T> = number;",
        "type Identity<T> = T;",
        "type Conditional<T> = T extends string ? { value: T } : never;",
        "type Indexed<T> = T[keyof T];",
        "type Keys<T> = keyof T;",
        "type Wrapped<T> = Array<T>;",
        "type Qualified<T> = Namespace.Box<T>;",
        "import type { Box } from 'box'; type Alias<T> = Box<T>;",
        "interface Box<out T> { value: T } type Alias<T> = Box<T>;",
        "class Box<out T> { value!: T } type Alias<T> = Box<T>;",
        "type Optional<T> = T | undefined; type Alias<T> = Optional<T>;",
        "type Mapped<out T> = { [K in keyof T]: T[K] }; type Alias<T> = Mapped<number>;",
        "type Mapped<out T> = { [K in keyof T]: T[K] }; type Alias<T> = Mapped<T[]>;",
        "type Box<out T> = { value: T }; type Alias<Box> = Box;",
        "type Cycle<T> = Cycle<T>;",
        "type A<T> = B<T>; type B<T> = A<T>;",
        "function identity<T>(value: T): T { return value; }",
        "const identity = <T,>(value: T): T => value;",
        "type Callback = <T>(value: T) => T;",
        "type Factory = new <T>(value: T) => { value: T };",
        "interface Methods { method<T>(value: T): T; <U>(value: U): U; }",
        "class Methods { method<T>(value: T): T { return value; } }",
        "const methods = { method<T>(value: T): T { return value; } };",
        "type ObjectType<out T> = { value: T; method<U>(value: U): U };",
        "type Infer<T> = T extends (infer U)[] ? U : never;",
    ];

    let fail = vec![
        "class Box<T> { value!: T; }",
        "const Box = class<T> { value!: T; };",
        "abstract class Box<T> { abstract value: T; }",
        "interface Box<T> { value: T; }",
        "type Box<T> = { value: T };",
        "type Callback<T> = (value: T) => void;",
        "type Factory<T> = new () => { value: T };",
        "type Factory<T> = abstract new () => { value: T };",
        "type Mapped<T> = { [K in keyof T]: T[K] };",
        "type Parenthesized<T> = (({ value: T }));",
        "type Callback<T> = ((value: T) => void);",
        "type Box<out T> = { value: T }; type Alias<T> = Box<T>;",
        "type Box<out T> = { value: T }; type A<out T> = Box<T>; type B<T> = A<T>;",
        "type Alias<T> = Box<T>; type Box<out T> = { value: T };",
        "type Box<out T> = { value: T }; type Alias<Box> = { value: Box };",
        "interface Pair<out T, U> { first: T; second: U; }",
        "interface Pair<T, U> { first: T; second: U; }",
        "class Box<const T extends string = string> { value!: T; }",
        "declare class Box<T> { value: T; }",
        "declare interface Box<T> { value: T; }",
        "declare type Box<T> = { value: T };",
        "export interface Box<T> { value: T; }",
        "export default class Box<T> { value!: T; }",
    ];

    Tester::new(RequireVarianceAnnotations::NAME, RequireVarianceAnnotations::PLUGIN, pass, fail)
        .test_and_snapshot();
}
