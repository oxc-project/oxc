use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, AssignmentExpression, AssignmentTarget,
        BindingPatternKind, CallExpression, Declaration, Expression, MemberExpression,
        ModuleDeclaration, ModuleExportName, ObjectPropertyKind, PropertyKey,
        SimpleAssignmentTarget, VariableDeclarator,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum NoThenableDiagnostic {
    #[error("eslint-plugin-unicorn(no-thenable): Do not add `then` to an object.")]
    #[diagnostic(severity(warning), help("If an object is defined as 'thenable', once it's accidentally used in an await expression, it may cause problems"))]
    Object(#[label] Span),
    #[error("eslint-plugin-unicorn(no-thenable): Do not export `then`.")]
    #[diagnostic(severity(warning), help("If an object is defined as 'thenable', once it's accidentally used in an await expression, it may cause problems"))]
    Export(#[label] Span),
    #[error("eslint-plugin-unicorn(no-thenable): Do not add `then` to a class.")]
    #[diagnostic(severity(warning), help("If an object is defined as 'thenable', once it's accidentally used in an await expression, it may cause problems"))]
    Class(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct NoThenable;

declare_oxc_lint!(
    /// ### What it does
    /// disallow `then` property
    ///
    /// ### Why is this bad?
    /// If an object is defined as "thenable", once it's accidentally
    /// used in an await expression, it may cause problems:
    ///
    ///
    /// ### Example
    /// ```javascript
    /// const foo = {
    ///     unicorn: 1,
    ///     then() {},
    /// };
    ///
    /// const {unicorn} = await foo;
    ///
    /// console.log('after'); //<- This will never execute
    /// ```
    NoThenable,
    correctness
);

impl Rule for NoThenable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(expr) => {
                expr.properties.iter().for_each(|prop| {
                    if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                        if let Some(span) = contains_then(&prop.key, ctx) {
                            ctx.diagnostic(NoThenableDiagnostic::Object(span));
                        }
                    }
                });
            }
            AstKind::PropertyDefinition(def) => {
                if let Some(span) = contains_then(&def.key, ctx) {
                    ctx.diagnostic(NoThenableDiagnostic::Class(span));
                }
            }
            AstKind::MethodDefinition(def) => {
                if let Some(span) = contains_then(&def.key, ctx) {
                    ctx.diagnostic(NoThenableDiagnostic::Class(span));
                }
            }
            AstKind::ModuleDeclaration(ModuleDeclaration::ExportNamedDeclaration(decl)) => {
                // check declaration
                if let Some(ref decl) = decl.declaration {
                    match decl {
                        Declaration::VariableDeclaration(decl) => {
                            for decl in &decl.declarations {
                                check_binding_pattern(&decl.id.kind, ctx);
                            }
                        }
                        Declaration::FunctionDeclaration(decl) => {
                            if let Some(bind) = decl.id.as_ref() {
                                if bind.name == "then" {
                                    ctx.diagnostic(NoThenableDiagnostic::Export(bind.span));
                                }
                            };
                        }
                        Declaration::ClassDeclaration(decl) => {
                            if let Some(bind) = decl.id.as_ref() {
                                if bind.name == "then" {
                                    ctx.diagnostic(NoThenableDiagnostic::Export(bind.span));
                                }
                            };
                        }
                        _ => {}
                    }
                }
                // check specifier
                for spec in &decl.specifiers {
                    match spec.exported {
                        ModuleExportName::Identifier(ref ident) => {
                            if ident.name == "then" {
                                ctx.diagnostic(NoThenableDiagnostic::Export(ident.span));
                            }
                        }
                        ModuleExportName::StringLiteral(ref lit) => {
                            if lit.value == "then" {
                                ctx.diagnostic(NoThenableDiagnostic::Export(lit.span));
                            }
                        }
                    }
                }
            }
            AstKind::CallExpression(expr) => check_call_expression(expr, ctx),
            // foo.then = ...
            AstKind::AssignmentExpression(AssignmentExpression {
                left:
                    AssignmentTarget::SimpleAssignmentTarget(
                        SimpleAssignmentTarget::MemberAssignmentTarget(target),
                    ),
                ..
            }) => match &**target {
                MemberExpression::ComputedMemberExpression(expr) => {
                    if let Some(span) = check_expression(&expr.expression, ctx) {
                        ctx.diagnostic(NoThenableDiagnostic::Class(span));
                    }
                }
                MemberExpression::StaticMemberExpression(expr) => {
                    if expr.property.name == "then" {
                        ctx.diagnostic(NoThenableDiagnostic::Class(expr.span));
                    }
                }
                MemberExpression::PrivateFieldExpression(_) => {}
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
            && match expr.callee {
                Expression::MemberExpression(ref me) => {
                    me.object().get_identifier_reference().map_or(false, |ident_ref| {
                        ident_ref.name == "Reflect" || ident_ref.name == "Object"
                    }) && me.static_property_name() == Some("defineProperty")
                        && !me.optional()
                }
                _ => false,
            }
    } {
    } else if let Argument::Expression(inner) = &expr.arguments[1] {
        if let Some(span) = check_expression(inner, ctx) {
            ctx.diagnostic(NoThenableDiagnostic::Object(span));
        }
    }

    // `Object.fromEntries([['then', …]])`
    if !{
        !expr.optional
            && expr.arguments.len() == 1
            && matches!(expr.arguments[0], Argument::Expression(Expression::ArrayExpression(_)))
            && match expr.callee {
                Expression::MemberExpression(ref me) => {
                    me.object()
                        .get_identifier_reference()
                        .map_or(false, |ident_ref| ident_ref.name == "Object")
                        && me.static_property_name() == Some("fromEntries")
                        && !me.optional()
                }
                _ => false,
            }
    } {
    } else if let Argument::Expression(Expression::ArrayExpression(outer)) = &expr.arguments[0] {
        for inner in &outer.elements {
            // inner item is array
            if let ArrayExpressionElement::Expression(Expression::ArrayExpression(inner)) = inner {
                if inner.elements.len() > 0
                    && !matches!(inner.elements[0], ArrayExpressionElement::SpreadElement(_))
                {
                    if let ArrayExpressionElement::Expression(ref expr) = inner.elements[0] {
                        if let Some(span) = check_expression(expr, ctx) {
                            ctx.diagnostic(NoThenableDiagnostic::Object(span));
                        }
                    }
                }
            }
        }
    }
}

fn check_binding_pattern(pat: &BindingPatternKind, ctx: &LintContext) {
    match pat {
        BindingPatternKind::BindingIdentifier(bind) => {
            if bind.name == "then" {
                ctx.diagnostic(NoThenableDiagnostic::Export(bind.span));
            }
        }
        BindingPatternKind::ObjectPattern(obj) => {
            for prop in &obj.properties {
                check_binding_pattern(&prop.value.kind, ctx);
            }
            if let Some(elem) = obj.rest.as_ref() {
                check_binding_pattern(&elem.argument.kind, ctx);
            }
        }
        BindingPatternKind::ArrayPattern(arr) => {
            for pat in &arr.elements {
                if let Some(pat) = pat.as_ref() {
                    check_binding_pattern(&pat.kind, ctx);
                }
            }
            if let Some(elem) = arr.rest.as_ref() {
                check_binding_pattern(&elem.argument.kind, ctx);
            }
        }
        BindingPatternKind::AssignmentPattern(assign) => {
            check_binding_pattern(&assign.left.kind, ctx);
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
            lit.quasi().and_then(|quasi| if quasi == "then" { Some(lit.span) } else { None })
        }
        Expression::Identifier(ident) => {
            let tab = ctx.semantic().symbols();
            ident.reference_id.get().and_then(|ref_id| {
                tab.get_reference(ref_id).symbol_id().and_then(|symbol_id| {
                    let decl = ctx.semantic().nodes().get_node(tab.get_declaration(symbol_id));
                    if let AstKind::VariableDeclarator(VariableDeclarator {
                        init: Some(Expression::StringLiteral(ref lit)),
                        ..
                    }) = decl.kind()
                    {
                        if lit.value == "then" {
                            Some(lit.span)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
        }
        _ => None,
    }
}

fn contains_then(key: &PropertyKey, ctx: &LintContext) -> Option<Span> {
    match key {
        PropertyKey::Identifier(ident) if ident.name == "then" => Some(ident.span),
        PropertyKey::Expression(expr) => check_expression(expr, ctx),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const then = {}", None),
        ("const notThen = then", None),
        ("const then = then.then", None),
        ("const foo = {notThen: 1}", None),
        ("const foo = {notThen() {}}", None),
        ("const foo = {[then]: 1}", None),
        ("const NOT_THEN = \"no-then\";const foo = {[NOT_THEN]: 1}", None),
        ("function foo({then}) {}", None),
        ("class then {}", None),
        ("class Foo {notThen}", None),
        ("class Foo {notThen() {}}", None),
        ("class Foo {[then]}", None),
        ("class Foo {#then}", None),
        ("class Foo {#then() {}}", None),
        ("class Foo {[then]() {}}", None),
        ("class Foo {get notThen() {}}", None),
        ("class Foo {get #then() {}}", None),
        ("class Foo {get [then]() {}}", None),
        ("class Foo {static notThen}", None),
        ("class Foo {static notThen() {}}", None),
        ("class Foo {static #then}", None),
        ("class Foo {static #then() {}}", None),
        ("class Foo {static [then]}", None),
        ("class Foo {static [then]() {}}", None),
        ("class Foo {static get notThen() {}}", None),
        ("class Foo {static get #then() {}}", None),
        ("class Foo {static get [then]() {}}", None),
        ("class Foo {notThen = then}", None),
        ("foo[then] = 1", None),
        ("foo.notThen = 1", None),
        ("then.notThen = then.then", None),
        ("const NOT_THEN = \"no-then\";foo[NOT_THEN] = 1", None),
        ("foo.then ++", None),
        ("++ foo.then", None),
        ("delete foo.then", None),
        ("typeof foo.then", None),
        ("foo.then != 1", None),
        ("Object.fromEntries([then, 1])", None),
        ("Object.fromEntries([,,])", None),
        ("Object.fromEntries([[,,],[]])", None),
        ("const NOT_THEN = \"not-then\";Object.fromEntries([[NOT_THEN, 1]])", None),
        ("Object.fromEntries([[[\"then\", 1]]])", None),
        ("NotObject.fromEntries([[\"then\", 1]])", None),
        ("Object.notFromEntries([[\"then\", 1]])", None),
        ("Object.fromEntries?.([[\"then\", 1]])", None),
        ("Object?.fromEntries([[\"then\", 1]])", None),
        ("Object.fromEntries([[...\"then\", 1]])", None),
        ("Object.fromEntries([[\"then\", 1]], extraArgument)", None),
        ("Object.fromEntries(...[[\"then\", 1]])", None),
        ("Object.defineProperty(foo, then, 1)", None),
        ("Object.defineProperty(foo, \"not-then\", 1)", None),
        ("const then = \"no-then\";Object.defineProperty(foo, then, 1)", None),
        ("Reflect.defineProperty(foo, then, 1)", None),
        ("Reflect.defineProperty(foo, \"not-then\", 1)", None),
        ("const then = \"no-then\";Reflect.defineProperty(foo, then, 1)", None),
        ("Object.defineProperty(foo, \"then\", )", None),
        ("Object.defineProperty(...foo, \"then\", 1)", None),
        ("Object.defineProperty(foo, ...[\"then\", 1])", None),
        ("export {default} from \"then\"", None),
        ("const then = 1; export {then as notThen}", None),
        ("export default then", None),
        ("export function notThen(){}", None),
        ("export class notThen {}", None),
        ("export default function then (){}", None),
        ("export default class then {}", None),
        ("export default function (){}", None),
        ("export default class {}", None),
        ("export const notThen = 1", None),
        ("export const {then: notThen} = 1", None),
        ("export const {then: notThen = then} = 1", None),
    ];

    let fail = vec![
        ("const foo = {then: 1}", None),
        ("const foo = {[\"then\"]: 1}", None),
        ("const foo = {[`then`]: 1}", None),
        ("const THEN = \"then\";const foo = {[THEN]: 1}", None),
        ("const foo = {then() {}}", None),
        ("const foo = {[\"then\"]() {}}", None),
        ("const foo = {[`then`]() {}}", None),
        ("const THEN = \"then\";const foo = {[THEN]() {}}", None),
        ("const foo = {get then() {}}", None),
        ("const foo = {get [\"then\"]() {}}", None),
        ("const foo = {get [`then`]() {}}", None),
        ("const THEN = \"then\";const foo = {get [THEN]() {}}", None),
        ("class Foo {then}", None),
        ("const Foo = class {then}", None),
        ("class Foo {[\"then\"]}", None),
        ("class Foo {[`then`]}", None),
        ("const THEN = \"then\";class Foo {[THEN]}", None),
        ("class Foo {then() {}}", None),
        ("class Foo {[\"then\"]() {}}", None),
        ("class Foo {[`then`]() {}}", None),
        ("const THEN = \"then\";class Foo {[THEN]() {}}", None),
        ("class Foo {static then}", None),
        ("class Foo {static [\"then\"]}", None),
        ("class Foo {static [`then`]}", None),
        ("const THEN = \"then\";class Foo {static [THEN]}", None),
        ("class Foo {static then() {}}", None),
        ("class Foo {static [\"then\"]() {}}", None),
        ("class Foo {static [`then`]() {}}", None),
        ("const THEN = \"then\";class Foo {static [THEN]() {}}", None),
        ("class Foo {get then() {}}", None),
        ("class Foo {get [\"then\"]() {}}", None),
        ("class Foo {get [`then`]() {}}", None),
        ("const THEN = \"then\";class Foo {get [THEN]() {}}", None),
        ("class Foo {set then(v) {}}", None),
        ("class Foo {set [\"then\"](v) {}}", None),
        ("class Foo {set [`then`](v) {}}", None),
        ("const THEN = \"then\";class Foo {set [THEN](v) {}}", None),
        ("class Foo {static get then() {}}", None),
        ("class Foo {static get [\"then\"]() {}}", None),
        ("class Foo {static get [`then`]() {}}", None),
        ("const THEN = \"then\";class Foo {static get [THEN]() {}}", None),
        ("foo.then = 1", None),
        ("foo[\"then\"] = 1", None),
        ("foo[`then`] = 1", None),
        ("const THEN = \"then\";foo[THEN] = 1", None),
        ("foo.then += 1", None),
        ("foo.then ||= 1", None),
        ("foo.then ??= 1", None),
        ("Object.defineProperty(foo, \"then\", 1)", None),
        ("Object.defineProperty(foo, `then`, 1)", None),
        ("const THEN = \"then\";Object.defineProperty(foo, THEN, 1)", None),
        ("Reflect.defineProperty(foo, \"then\", 1)", None),
        ("Reflect.defineProperty(foo, `then`, 1)", None),
        ("const THEN = \"then\";Reflect.defineProperty(foo, THEN, 1)", None),
        ("Object.fromEntries([[\"then\", 1]])", None),
        ("Object.fromEntries([[\"then\"]])", None),
        ("Object.fromEntries([[`then`, 1]])", None),
        ("const THEN = \"then\";Object.fromEntries([[THEN, 1]])", None),
        ("Object.fromEntries([foo, [\"then\", 1]])", None),
        ("const then = 1; export {then}", None),
        ("const notThen = 1; export {notThen as then}", None),
        ("export {then} from \"foo\"", None),
        ("export function then() {}", None),
        ("export async function then() {}", None),
        ("export function * then() {}", None),
        ("export async function * then() {}", None),
        ("export class then {}", None),
        ("export const then = 1", None),
        ("export let then = 1", None),
        ("export var then = 1", None),
        ("export const [then] = 1", None),
        ("export let [then] = 1", None),
        ("export var [then] = 1", None),
        ("export const [, then] = 1", None),
        ("export let [, then] = 1", None),
        ("export var [, then] = 1", None),
        ("export const [, ...then] = 1", None),
        ("export let [, ...then] = 1", None),
        ("export var [, ...then] = 1", None),
        ("export const {then} = 1", None),
        ("export let {then} = 1", None),
        ("export var {then} = 1", None),
        ("export const {foo, ...then} = 1", None),
        ("export let {foo, ...then} = 1", None),
        ("export var {foo, ...then} = 1", None),
        ("export const {foo: {bar: [{baz: then}]}} = 1", None),
        ("export const notThen = 1, then = 1", None),
    ];

    Tester::new(NoThenable::NAME, pass, fail).test_and_snapshot();
}
