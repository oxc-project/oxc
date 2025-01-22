use oxc_ast::{
    ast::{
        match_expression, Argument, ArrayExpressionElement, AssignmentExpression, AssignmentTarget,
        BindingPatternKind, CallExpression, Declaration, Expression, ModuleDeclaration,
        ObjectPropertyKind, PropertyKey,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

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
    /// disallow `then` property
    ///
    /// ### Why is this bad?
    /// If an object is defined as "thenable", once it's accidentally
    /// used in an await expression, it may cause problems:
    ///
    ///
    /// ### Example
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
    NoThenable,
    unicorn,
    correctness
);

impl Rule for NoThenable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(expr) => {
                expr.properties.iter().for_each(|prop| {
                    if let ObjectPropertyKind::ObjectProperty(prop) = prop {
                        if let Some(span) = contains_then(&prop.key, ctx) {
                            ctx.diagnostic(object(span));
                        }
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
            AstKind::ModuleDeclaration(ModuleDeclaration::ExportNamedDeclaration(decl)) => {
                // check declaration
                if let Some(decl) = &decl.declaration {
                    match decl {
                        Declaration::VariableDeclaration(decl) => {
                            for decl in &decl.declarations {
                                check_binding_pattern(&decl.id.kind, ctx);
                            }
                        }
                        Declaration::FunctionDeclaration(decl) => {
                            if let Some(bind) = decl.id.as_ref() {
                                if bind.name == "then" {
                                    ctx.diagnostic(export(bind.span));
                                }
                            };
                        }
                        Declaration::ClassDeclaration(decl) => {
                            if let Some(bind) = decl.id.as_ref() {
                                if bind.name == "then" {
                                    ctx.diagnostic(export(bind.span));
                                }
                            };
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
    } else if let Some(inner) = expr.arguments[1].as_expression() {
        if let Some(span) = check_expression(inner, ctx) {
            ctx.diagnostic(object(span));
        }
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
            if let ArrayExpressionElement::ArrayExpression(inner) = inner {
                if inner.elements.len() > 0
                    && !matches!(inner.elements[0], ArrayExpressionElement::SpreadElement(_))
                {
                    if let Some(expr) = inner.elements[0].as_expression() {
                        if let Some(span) = check_expression(expr, ctx) {
                            ctx.diagnostic(object(span));
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
                ctx.diagnostic(export(bind.span));
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
            let symbols = ctx.semantic().symbols();
            let reference_id = ident.reference_id();
            symbols.get_reference(reference_id).symbol_id().and_then(|symbol_id| {
                let decl = ctx.semantic().nodes().get_node(symbols.get_declaration(symbol_id));
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

    Tester::new(NoThenable::NAME, NoThenable::PLUGIN, pass, fail).test_and_snapshot();
}
