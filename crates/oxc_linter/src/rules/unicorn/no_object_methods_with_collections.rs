// Adapted from eslint-plugin-unicorn's no-object-methods-with-collections rule.
// https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v79.0.0/rules/no-object-methods-with-collections.js
// Copyright (c) Sindre Sorhus. MIT license; see THIRD-PARTY-LICENSE.

use oxc_ast::{
    AstKind,
    ast::{Expression, TSType, TSTypeName, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, variable_declaration_kind},
    context::LintContext,
    rule::Rule,
    utils::pad_fix_with_token_boundary,
};

fn no_object_methods_with_collections_diagnostic(
    span: Span,
    method: &str,
    kind: Collection,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`Object.{method}()` does not return {} contents.", kind.name()))
        .with_label(span)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Collection {
    Map,
    Set,
}

impl Collection {
    fn name(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Set => "Set",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoObjectMethodsWithCollections;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `Object.keys()`, `Object.values()`, and `Object.entries()` on `Map` and `Set`.
    ///
    /// ### Why is this bad?
    ///
    /// Collection contents are not enumerable own properties, so these methods usually return
    /// an empty array instead of the collection's keys, values, or entries.
    /// The rule reports only collections that can be identified confidently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Object.keys(new Map([["key", 1]]));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Array.from(new Map([["key", 1]]).keys());
    /// ```
    NoObjectMethodsWithCollections,
    unicorn,
    correctness,
    recommended,
    suggestion,
    version = "next",
    short_description = "Disallow `Object` methods with `Map` or `Set`.",
);

impl Rule for NoObjectMethodsWithCollections {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else { return };
        if call.optional || call.arguments.len() != 1 {
            return;
        }
        let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
            return;
        };
        if member.optional {
            return;
        }
        let Expression::Identifier(object) = member.object.without_parentheses() else {
            return;
        };
        if object.name != "Object" || !object.is_global_reference(ctx.scoping()) {
            return;
        }
        let method = member.property.name.as_str();
        if !matches!(method, "keys" | "values" | "entries") {
            return;
        }
        let Some(argument) = call.arguments[0].as_expression() else { return };
        let Some(kind) = collection_type(argument, ctx, 0) else { return };
        let diagnostic =
            no_object_methods_with_collections_diagnostic(member.property.span, method, kind);

        // Replacing the whole call would discard comments outside the argument.
        if ctx
            .comments_range(call.span.start..call.span.end)
            .any(|comment| !argument.span().contains_inclusive(comment.span))
        {
            ctx.diagnostic(diagnostic);
            return;
        }

        let argument_text = ctx.source_range(argument.span());
        let needs_parens = match argument {
            Expression::TSAsExpression(_)
            | Expression::TSSatisfiesExpression(_)
            | Expression::TSTypeAssertion(_)
            | Expression::ConditionalExpression(_) => true,
            // `new Map.keys()` would parse as `new (Map.keys)()`.
            Expression::NewExpression(_) => !argument_text.ends_with(')'),
            _ => false,
        };
        let receiver =
            if needs_parens { format!("({argument_text})") } else { argument_text.to_string() };
        let mut replacement = format!("Array.from({receiver}.{method}())");
        pad_fix_with_token_boundary(ctx.source_text(), call.span, &mut replacement);
        ctx.diagnostic_with_suggestion(diagnostic, |fixer| fixer.replace(call.span, replacement));
    }
}

// Follow only syntax that proves a single collection family. The depth limit also
// keeps pathological alias cycles from making lint time depend on declaration chains.
fn collection_type<'a>(
    expression: &Expression<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
) -> Option<Collection> {
    if depth >= 16 {
        return None;
    }
    match expression.without_parentheses() {
        Expression::NewExpression(new_expr) => {
            let Expression::Identifier(callee) = new_expr.callee.without_parentheses() else {
                return None;
            };
            if !callee.is_global_reference(ctx.scoping()) {
                return None;
            }
            match callee.name.as_str() {
                "Map" => Some(Collection::Map),
                "Set" => Some(Collection::Set),
                _ => None,
            }
        }
        Expression::Identifier(ident) => {
            let declaration = get_declaration_of_variable(ident, ctx)?;
            match declaration.kind() {
                // A type annotation on a destructuring pattern describes the whole pattern,
                // not the identifier bound from it.
                AstKind::VariableDeclarator(var) if var.id.is_binding_identifier() => {
                    if let Some(annotation) = &var.type_annotation {
                        return collection_type_annotation(
                            &annotation.type_annotation,
                            ctx,
                            depth + 1,
                        );
                    }
                    if variable_declaration_kind(var, ctx.nodes())
                        .is_some_and(VariableDeclarationKind::is_const)
                    {
                        var.init.as_ref().and_then(|init| collection_type(init, ctx, depth + 1))
                    } else {
                        None
                    }
                }
                AstKind::FormalParameter(param) if param.pattern.is_binding_identifier() => {
                    param.type_annotation.as_ref().and_then(|annotation| {
                        collection_type_annotation(&annotation.type_annotation, ctx, depth + 1)
                    })
                }
                _ => None,
            }
        }
        Expression::TSAsExpression(expr) => {
            collection_type_annotation(&expr.type_annotation, ctx, depth + 1)
        }
        Expression::TSSatisfiesExpression(expr) => {
            collection_type_annotation(&expr.type_annotation, ctx, depth + 1)
                .or_else(|| collection_type(&expr.expression, ctx, depth + 1))
        }
        Expression::TSTypeAssertion(expr) => {
            collection_type_annotation(&expr.type_annotation, ctx, depth + 1)
        }
        Expression::TSNonNullExpression(expr) => collection_type(&expr.expression, ctx, depth + 1),
        Expression::ConditionalExpression(expr) => {
            let first = collection_type(&expr.consequent, ctx, depth + 1)?;
            (collection_type(&expr.alternate, ctx, depth + 1) == Some(first)).then_some(first)
        }
        _ => None,
    }
}

fn collection_type_annotation<'a>(
    ty: &TSType<'a>,
    ctx: &LintContext<'a>,
    depth: u8,
) -> Option<Collection> {
    if depth >= 16 {
        return None;
    }
    match ty.without_parenthesized() {
        TSType::TSTypeReference(reference) => {
            let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                return None;
            };
            if name.is_global_reference(ctx.scoping()) {
                return collection_name(&name.name);
            }
            let declaration = get_declaration_of_variable(name, ctx)?;
            let AstKind::TSTypeAliasDeclaration(alias) = declaration.kind() else {
                return None;
            };
            collection_type_annotation(&alias.type_annotation, ctx, depth + 1)
        }
        TSType::TSUnionType(union) => {
            let mut types = union.types.iter();
            let first = collection_type_annotation(types.next()?, ctx, depth + 1)?;
            types
                .all(|ty| collection_type_annotation(ty, ctx, depth + 1) == Some(first))
                .then_some(first)
        }
        _ => None,
    }
}

fn collection_name(name: &str) -> Option<Collection> {
    match name {
        "Map" | "ReadonlyMap" => Some(Collection::Map),
        "Set" | "ReadonlySet" => Some(Collection::Set),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "function f({ size }: Set<string>) { Object.keys(size) }",
        "declare const x: any; const { a }: Map<string, number> = x; Object.keys(a)",
        "Object.keys({a: 1})",
        "Object.values({a: 1})",
        "Object.entries({a: 1})",
        "Object.keys([1, 2])",
        "Object.values([1, 2])",
        "Object.entries([1, 2])",
        "Object.values(array)",
        "Object.entries(collection)",
        "Object.keys(new WeakMap())",
        "Object.values(new WeakMap())",
        "Object.entries(new WeakMap())",
        "Object.keys(new ReadonlyMap())",
        "Object.keys(new WeakSet())",
        "Object.values(new WeakSet())",
        "Object.entries(new WeakSet())",
        "Object?.keys(new Map())",
        "Object[method](new Map())",
        r#"Object["keys"](new Map())"#,
        r#"Object["values"](new Set())"#,
        r#"Object["entries"](new Map())"#,
        "Object.keys?.(new Map())",
        "globalThis.Object.keys(new Map())",
        "window.Object.values(new Set())",
        "const Object = {keys: value => value}; Object.keys(new Map())",
        "const Map = class {keys() { return []; }}; Object.keys(new Map())",
        "const Set = class {values() { return []; }}; Object.values(new Set())",
        "const map = getMap(); map.foo = 1; Object.keys(map);",
        "function foo(map: WeakMap<object, string>) { return Object.keys(map); }",
        "function foo(set: WeakSet<object>) { return Object.values(set); }",
        r#"import {Map} from "immutable"; function foo(map: Map<string, number>) { return Object.keys(map); }"#,
        "interface Map<K, V> { keys(): K[] } function foo(map: Map<string, number>) { return Object.keys(map); }",
        "type Set<T> = {values(): T[]}; function foo(set: Set<string>) { return Object.values(set); }",
        "export interface Map<K, V> { keys(): K[] } function foo(map: Map<string, number>) { return Object.keys(map); }",
        "export type Set<T> = {values(): T[]}; function foo(set: Set<string>) { return Object.values(set); }",
        "function foo(collection: Map<string, number> | Set<string>) { return Object.entries(collection); }",
        "export {}; class Map { keys() { return []; } } const map = new Map(); Object.keys(map);",
        "interface Map<K, V> { keys(): K[] } declare const map: Map<string, number>; Object.keys(map);",
        "export {}; interface Map<K, V> { keys(): K[] } declare const map: Map<string, number>; Object.keys(map);",
        "export {}; type Map<K, V> = {keys(): K[]}; declare const map: Map<string, number>; Object.keys(map);",
        "interface Collection { keys(): string[]; } declare const collection: Collection; Object.keys(collection);",
        "declare const collection: Map<string, number> | Set<string>; Object.entries(collection);",
        "declare const collection: unknown; Object.values(collection);",
        "declare const collection: any; Object.entries(collection);",
        "const collection: unknown = new Map(); Object.keys(collection);",
        "Object.keys(new Map() as unknown)",
        "Object.values(<unknown>new Set())",
        "let map = new Map(); Object.keys(map);",
        "Object.keys(new Map(), extra)",
        "Object.keys(...[new Map()])",
        "const map = new WeakMap(); Object.keys(map);",
        "function f(map: Map<string, number> | Set<string>) { Object.keys(map); }",
    ];

    let fail = vec![
        "Object.keys(new Map())",
        "Object.values(new Map())",
        "Object.entries(new Map())",
        "Object.keys(new Set())",
        "Object.values(new Set())",
        "Object.entries(new Set())",
        "const map = new Map(); Object.keys(map);",
        "const set = new Set(); Object.values(set);",
        "Object.keys((new Map()))",
        "Object.values((new Set()))",
        "Object.keys(/* keep */ new Map())",
        "function foo(map: Map<string, number>) { return Object.keys(map); }",
        "function foo(map: ReadonlyMap<string, number>) { return Object.values(map); }",
        "function foo(set: Set<string>) { return Object.entries(set); }",
        "function foo(set: ReadonlySet<string>) { return Object.keys(set); }",
        "Object.entries(value as Map<string, number>);",
        "Object.values(value satisfies Set<string>);",
        "const map = value as Map<string, number>; Object.entries(map);",
        "const set = value satisfies Set<string>; Object.values(set);",
        "const map = <Map<string, number>>value; Object.keys(map);",
        "const map = value as Map<string, number>; Object.keys(map!);",
        "type Items = Map<string, number>; declare const items: Items; Object.keys(items);",
        "type Items = ReadonlySet<string>; declare const items: Items; Object.values(items);",
        "type Items = Map<string, number> | ReadonlyMap<string, number>; declare const items: Items; Object.entries(items);",
        "Object.values((value as Set<string>));",
        "Object.values((value as /* keep */ Set<string>))",
        "Object./* keep */keys(new Map())",
        "Object.keys(new Map() /* keep */)",
        "function f() {return(Object).entries(new Map([[1, /* keep */ 2]]));}",
        "function f() {throw(Object).entries(new Map([[1, /* keep */ 2]]));}",
        "if (flag) {}else(Object).keys(new Map(/* keep */));",
        "do(Object).values(new Set(/* keep */));while (flag);",
        "const map = new Map(); const alias = map; Object.keys(alias);",
        "Object.keys(flag ? new Map() : new Map());",
        "Object.values(new Set(// keep\n))",
        "Object.values(new Set(// keep\r\n))",
    ];

    let fix = vec![
        ("Object.keys(new Map)", "Array.from((new Map).keys())", None, FixKind::Suggestion),
        (
            "Object.keys((flag) ? new Map() : new Map())",
            "Array.from(((flag) ? new Map() : new Map()).keys())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.entries((value) as Map<string, number>)",
            "Array.from(((value) as Map<string, number>).entries())",
            None,
            FixKind::Suggestion,
        ),
        ("Object.keys(new Map())", "Array.from(new Map().keys())", None, FixKind::Suggestion),
        ("Object.values(new Set())", "Array.from(new Set().values())", None, FixKind::Suggestion),
        ("Object.entries(new Map())", "Array.from(new Map().entries())", None, FixKind::Suggestion),
        (
            "Object.keys(new Map(/* keep */))",
            "Array.from(new Map(/* keep */).keys())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.values((value as /* keep */ Set<string>))",
            "Array.from((value as /* keep */ Set<string>).values())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.entries(value as Map<string, number>)",
            "Array.from((value as Map<string, number>).entries())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.values((/* keep */ new Set()))",
            "Array.from((/* keep */ new Set()).values())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.entries(new Map([[1, /* keep */ 2]]))",
            "Array.from(new Map([[1, /* keep */ 2]]).entries())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.values(new Set(// keep\n))",
            "Array.from(new Set(// keep\n).values())",
            None,
            FixKind::Suggestion,
        ),
        (
            "Object.values(new Set(// keep\r\n))",
            "Array.from(new Set(// keep\r\n).values())",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() {return(Object).entries(new Map([[1, /* keep */ 2]]));}",
            "function f() {return Array.from(new Map([[1, /* keep */ 2]]).entries());}",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() {throw(Object).entries(new Map([[1, /* keep */ 2]]));}",
            "function f() {throw Array.from(new Map([[1, /* keep */ 2]]).entries());}",
            None,
            FixKind::Suggestion,
        ),
        (
            "if (flag) {}else(Object).keys(new Map(/* keep */));",
            "if (flag) {}else Array.from(new Map(/* keep */).keys());",
            None,
            FixKind::Suggestion,
        ),
        (
            "do(Object).values(new Set(/* keep */));while (flag);",
            "do Array.from(new Set(/* keep */).values());while (flag);",
            None,
            FixKind::Suggestion,
        ),
        (
            "const type = typeof(Object).keys(new Map());",
            "const type = typeof Array.from(new Map().keys());",
            None,
            FixKind::Suggestion,
        ),
        (
            "for (const key of(Object).keys(new Map())) {}",
            "for (const key of Array.from(new Map().keys())) {}",
            None,
            FixKind::Suggestion,
        ),
        (
            "let evaluations = 0; const result = Object.entries(new Map((++evaluations, [[1, 2]])));",
            "let evaluations = 0; const result = Array.from(new Map((++evaluations, [[1, 2]])).entries());",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(
        NoObjectMethodsWithCollections::NAME,
        NoObjectMethodsWithCollections::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path_extension("ts")
    .expect_fix(fix)
    .test_and_snapshot();
}
