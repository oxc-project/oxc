use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, AssignmentExpression, AssignmentTarget, BindingPattern,
        CallExpression, Declaration, Expression, ObjectPropertyKind, PropertyKey, match_expression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn object(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not add `then` to an object.")
        .with_help("If an object is defined as 'thenable', once it's accidentally used in an await expression, it may cause problems")
        .with_label(span)
}

fn export(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not export `then`.")
        .with_help("If an object is defined as 'thenable', once it's accidentally used in an await expression, it may cause problems")
        .with_label(span)
}

fn class(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not add `then` to a class.")
        .with_help("If an object is defined as 'thenable', once it's accidentally used in an await expression, it may cause problems")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThenable;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `then` property
    ///
    /// ### Why is this bad?
    ///
    /// If an object is defined as "thenable", once it's accidentally
    /// used in an await expression, it may cause problems:
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    ///     async function example() {
    ///     const foo = {
    ///         unicorn: 1,
    ///         then() {},
    ///     };
    ///
    ///     const { unicorn } = await foo;
    ///
    ///     console.log('after'); //<- This will never execute
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    ///     async function example() {
    ///     const foo = {
    ///         unicorn: 1,
    ///         bar() {},
    ///     };
    ///
    ///     const { unicorn } = await foo;
    ///
    ///     console.log('after');
    /// }
    /// ```
    NoThenable,
    unicorn,
    correctness
);

impl Rule for NoThenable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(expr) => {
                expr.properties.iter().for_each(|prop| {
                    if let ObjectPropertyKind::ObjectProperty(prop) = prop
                        && let Some(span) = contains_then(&prop.key, ctx)
                    {
                        ctx.diagnostic(object(span));
                    }
                });
            }
            AstKind::PropertyDefinition(def) => {
                if let Some(span) = contains_then(&def.key, ctx) {
                    ctx.diagnostic(class(span));
                }
            }
            AstKind::MethodDefinition(def) => {
                if let Some(span) = contains_then(&def.key, ctx) {
                    ctx.diagnostic(class(span));
                }
            }
            AstKind::ExportNamedDeclaration(decl) => {
                // check declaration
                if let Some(decl) = &decl.declaration {
                    match decl {
                        Declaration::VariableDeclaration(decl) => {
                            for decl in &decl.declarations {
                                check_binding_pattern(&decl.id, ctx);
                            }
                        }
                        Declaration::FunctionDeclaration(decl) => {
                            if let Some(bind) = decl.id.as_ref()
                                && bind.name == "then"
                            {
                                ctx.diagnostic(export(bind.span));
                            }
                        }
                        Declaration::ClassDeclaration(decl) => {
                            if let Some(bind) = decl.id.as_ref()
                                && bind.name == "then"
                            {
                                ctx.diagnostic(export(bind.span));
                            }
                        }
                        _ => {}
                    }
                }
                // check specifier
                for spec in &decl.specifiers {
                    if spec.exported.name() == "then" {
                        ctx.diagnostic(export(spec.exported.span()));
                    }
                }
            }
            AstKind::CallExpression(expr) => check_call_expression(expr, ctx),
            // foo.then = ...
            AstKind::AssignmentExpression(AssignmentExpression { left, .. }) => match left {
                AssignmentTarget::ComputedMemberExpression(expr) => {
                    if let Some(span) = check_expression(&expr.expression, ctx) {
                        ctx.diagnostic(class(span));
                    }
                }
                AssignmentTarget::StaticMemberExpression(expr) => {
                    if expr.property.name == "then" {
                        ctx.diagnostic(class(expr.span));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn check_call_expression(expr: &CallExpression, ctx: &LintContext) {
    // `Object.defineProperty(foo, 'then', …)`
    // `Reflect.defineProperty(foo, 'then', …)`
    if !{
        !expr.optional
            && expr.arguments.len() >= 3
            && !matches!(expr.arguments[0], Argument::SpreadElement(_))
            && match expr.callee.as_member_expression() {
                Some(me) => {
                    me.object().get_identifier_reference().is_some_and(|ident_ref| {
                        ident_ref.name == "Reflect" || ident_ref.name == "Object"
                    }) && me.static_property_name() == Some("defineProperty")
                        && !me.optional()
                }
                _ => false,
            }
    } {
    } else if let Some(inner) = expr.arguments[1].as_expression()
        && let Some(span) = check_expression(inner, ctx)
    {
        ctx.diagnostic(object(span));
    }

    // `Object.fromEntries([['then', …]])`
    if !{
        !expr.optional
            && expr.arguments.len() == 1
            && matches!(expr.arguments[0], Argument::ArrayExpression(_))
            && match expr.callee.as_member_expression() {
                Some(me) => {
                    me.object()
                        .get_identifier_reference()
                        .is_some_and(|ident_ref| ident_ref.name == "Object")
                        && me.static_property_name() == Some("fromEntries")
                        && !me.optional()
                }
                _ => false,
            }
    } {
    } else if let Argument::ArrayExpression(outer) = &expr.arguments[0] {
        for inner in &outer.elements {
            // inner item is array
            if let ArrayExpressionElement::ArrayExpression(inner) = inner
                && !inner.elements.is_empty()
                && !matches!(inner.elements[0], ArrayExpressionElement::SpreadElement(_))
                && let Some(expr) = inner.elements[0].as_expression()
                && let Some(span) = check_expression(expr, ctx)
            {
                ctx.diagnostic(object(span));
            }
        }
    }
}

fn check_binding_pattern(pat: &BindingPattern, ctx: &LintContext) {
    match pat {
        BindingPattern::BindingIdentifier(bind) => {
            if bind.name == "then" {
                ctx.diagnostic(export(bind.span));
            }
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                check_binding_pattern(&prop.value, ctx);
            }
            if let Some(elem) = obj.rest.as_ref() {
                check_binding_pattern(&elem.argument, ctx);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for pat in &arr.elements {
                if let Some(pat) = pat.as_ref() {
                    check_binding_pattern(pat, ctx);
                }
            }
            if let Some(elem) = arr.rest.as_ref() {
                check_binding_pattern(&elem.argument, ctx);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            check_binding_pattern(&assign.left, ctx);
        }
    }
}

fn check_expression(expr: &Expression, ctx: &LintContext<'_>) -> Option<oxc_span::Span> {
    match expr {
        Expression::StringLiteral(lit) => {
            if lit.value == "then" {
                Some(lit.span)
            } else {
                None
            }
        }
        Expression::TemplateLiteral(lit) => {
            lit.single_quasi().and_then(|quasi| if quasi == "then" { Some(lit.span) } else { None })
        }
        Expression::Identifier(ident) => {
            let symbols = ctx.scoping();
            let reference_id = ident.reference_id();
            symbols.get_reference(reference_id).symbol_id().and_then(|symbol_id| {
                let decl = ctx.nodes().get_node(symbols.symbol_declaration(symbol_id));
                let var_decl = decl.kind().as_variable_declarator()?;

                match &var_decl.init {
                    Some(Expression::StringLiteral(lit)) => {
                        if lit.value == "then" {
                            Some(lit.span)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
        }
        _ => None,
    }
}

fn contains_then(key: &PropertyKey, ctx: &LintContext) -> Option<Span> {
    match key {
        PropertyKey::StaticIdentifier(ident) if ident.name == "then" => Some(ident.span),
        match_expression!(PropertyKey) => check_expression(key.to_expression(), ctx),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const then = {}",
        "const notThen = then",
        "const then = then.then",
        "const foo = {notThen: 1}",
        "const foo = {notThen() {}}",
        "const foo = {[then]: 1}",
        r#"const NOT_THEN = "no-then";const foo = {[NOT_THEN]: 1}"#,
        "function foo({then}) {}",
        "({[Symbol.prototype]: 1})",
        "class then {}",
        "class Foo {notThen}",
        "class Foo {notThen() {}}",
        "class Foo {[then]}",
        "class Foo {#then}",
        "class Foo {#then() {}}",
        "class Foo {[then]() {}}",
        "class Foo {get notThen() {}}",
        "class Foo {get #then() {}}",
        "class Foo {get [then]() {}}",
        "class Foo {static notThen}",
        "class Foo {static notThen() {}}",
        "class Foo {static #then}",
        "class Foo {static #then() {}}",
        "class Foo {static [then]}",
        "class Foo {static [then]() {}}",
        "class Foo {static get notThen() {}}",
        "class Foo {static get #then() {}}",
        "class Foo {static get [then]() {}}",
        "class Foo {notThen = then}",
        "class Foo {[Symbol.property]}",
        "class Foo {static [Symbol.property]}",
        "class Foo {get [Symbol.property]() {}}",
        "class Foo {[Symbol.property]() {}}",
        "class Foo {static get [Symbol.property]() {}}",
        "foo[then] = 1",
        "foo.notThen = 1",
        "then.notThen = then.then",
        r#"const NOT_THEN = "no-then";foo[NOT_THEN] = 1"#,
        "foo.then ++",
        "++ foo.then",
        "delete foo.then",
        "typeof foo.then",
        "foo.then != 1",
        "foo[Symbol.property] = 1",
        "Object.fromEntries([then, 1])",
        "Object.fromEntries([,,])",
        "Object.fromEntries([[,,],[]])",
        r#"const NOT_THEN = "not-then";Object.fromEntries([[NOT_THEN, 1]])"#,
        r#"Object.fromEntries([[["then", 1]]])"#,
        r#"NotObject.fromEntries([["then", 1]])"#,
        r#"Object.notFromEntries([["then", 1]])"#,
        r#"Object.fromEntries?.([["then", 1]])"#,
        r#"Object?.fromEntries([["then", 1]])"#,
        r#"Object.fromEntries([[..."then", 1]])"#,
        r#"Object.fromEntries([["then", 1]], extraArgument)"#,
        r#"Object.fromEntries(...[["then", 1]])"#,
        "Object.fromEntries([[Symbol.property, 1]])",
        "Object.defineProperty(foo, then, 1)",
        r#"Object.defineProperty(foo, "not-then", 1)"#,
        r#"const then = "no-then";Object.defineProperty(foo, then, 1)"#,
        "Reflect.defineProperty(foo, then, 1)",
        r#"Reflect.defineProperty(foo, "not-then", 1)"#,
        r#"const then = "no-then";Reflect.defineProperty(foo, then, 1)"#,
        r#"Object.defineProperty(foo, "then", )"#,
        r#"Object.defineProperty(...foo, "then", 1)"#,
        r#"Object.defineProperty(foo, ...["then", 1])"#,
        "Object.defineProperty(foo, Symbol.property, 1)",
        "Reflect.defineProperty(foo, Symbol.property, 1)",
        r#"export {default} from "then""#,
        "const then = 1; export {then as notThen}",
        "export default then",
        "export function notThen(){}",
        "export class notThen {}",
        "export default function then (){}",
        "export default class then {}",
        "export default function (){}",
        "export default class {}",
        "export const notThen = 1",
        "export const {then: notThen} = 1",
        "export const {then: notThen = then} = 1",
    ];

    let fail = vec![
        "const foo = {then: 1}",
        r#"const foo = {["then"]: 1}"#,
        "const foo = {[`then`]: 1}",
        r#"const THEN = "then";const foo = {[THEN]: 1}"#,
        "const foo = {then() {}}",
        r#"const foo = {["then"]() {}}"#,
        "const foo = {[`then`]() {}}",
        r#"const THEN = "then";const foo = {[THEN]() {}}"#,
        "const foo = {get then() {}}",
        r#"const foo = {get ["then"]() {}}"#,
        "const foo = {get [`then`]() {}}",
        r#"const THEN = "then";const foo = {get [THEN]() {}}"#,
        "class Foo {then}",
        "const Foo = class {then}",
        r#"class Foo {["then"]}"#,
        "class Foo {[`then`]}",
        r#"const THEN = "then";class Foo {[THEN]}"#,
        "class Foo {then() {}}",
        r#"class Foo {["then"]() {}}"#,
        "class Foo {[`then`]() {}}",
        r#"const THEN = "then";class Foo {[THEN]() {}}"#,
        "class Foo {static then}",
        r#"class Foo {static ["then"]}"#,
        "class Foo {static [`then`]}",
        r#"const THEN = "then";class Foo {static [THEN]}"#,
        "class Foo {static then() {}}",
        r#"class Foo {static ["then"]() {}}"#,
        "class Foo {static [`then`]() {}}",
        r#"const THEN = "then";class Foo {static [THEN]() {}}"#,
        "class Foo {get then() {}}",
        r#"class Foo {get ["then"]() {}}"#,
        "class Foo {get [`then`]() {}}",
        r#"const THEN = "then";class Foo {get [THEN]() {}}"#,
        "class Foo {set then(v) {}}",
        r#"class Foo {set ["then"](v) {}}"#,
        "class Foo {set [`then`](v) {}}",
        r#"const THEN = "then";class Foo {set [THEN](v) {}}"#,
        "class Foo {static get then() {}}",
        r#"class Foo {static get ["then"]() {}}"#,
        "class Foo {static get [`then`]() {}}",
        r#"const THEN = "then";class Foo {static get [THEN]() {}}"#,
        "foo.then = 1",
        r#"foo["then"] = 1"#,
        "foo[`then`] = 1",
        r#"const THEN = "then";foo[THEN] = 1"#,
        "foo.then += 1",
        "foo.then ||= 1",
        "foo.then ??= 1",
        r#"Object.defineProperty(foo, "then", 1)"#,
        "Object.defineProperty(foo, `then`, 1)",
        r#"const THEN = "then";Object.defineProperty(foo, THEN, 1)"#,
        r#"Reflect.defineProperty(foo, "then", 1)"#,
        "Reflect.defineProperty(foo, `then`, 1)",
        r#"const THEN = "then";Reflect.defineProperty(foo, THEN, 1)"#,
        r#"Object.fromEntries([["then", 1]])"#,
        r#"Object.fromEntries([["then"]])"#,
        "Object.fromEntries([[`then`, 1]])",
        r#"const THEN = "then";Object.fromEntries([[THEN, 1]])"#,
        r#"Object.fromEntries([foo, ["then", 1]])"#,
        "const then = 1; export {then}",
        "const notThen = 1; export {notThen as then}",
        r#"export {then} from "foo""#,
        "export function then() {}",
        "export async function then() {}",
        "export function * then() {}",
        "export async function * then() {}",
        "export class then {}",
        "export const then = 1",
        "export let then = 1",
        "export var then = 1",
        "export const [then] = 1",
        "export let [then] = 1",
        "export var [then] = 1",
        "export const [, then] = 1",
        "export let [, then] = 1",
        "export var [, then] = 1",
        "export const [, ...then] = 1",
        "export let [, ...then] = 1",
        "export var [, ...then] = 1",
        "export const {then} = 1",
        "export let {then} = 1",
        "export var {then} = 1",
        "export const {foo, ...then} = 1",
        "export let {foo, ...then} = 1",
        "export var {foo, ...then} = 1",
        "export const {foo: {bar: [{baz: then}]}} = 1",
        "export const notThen = 1, then = 1",
    ];

    Tester::new(NoThenable::NAME, NoThenable::PLUGIN, pass, fail).test_and_snapshot();
}
