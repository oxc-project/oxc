use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn prefer_object_has_own_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Disallow use of `Object.prototype.hasOwnProperty.call()` and prefer use of `Object.hasOwn()`."
    ).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferObjectHasOwn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow use of `Object.prototype.hasOwnProperty.call()` and prefer use of `Object.hasOwn()`
    ///
    /// ### Why is this bad?
    ///
    /// It is very common to write code like:
    /// ```javascript
    /// if (Object.prototype.hasOwnProperty.call(object, "foo")) {
    ///     console.log("has property foo");
    /// }
    /// ```
    /// This is a common practice because methods on Object.prototype can sometimes be unavailable or redefined (see the no-prototype-builtins rule).
    /// Introduced in ES2022, Object.hasOwn() is a shorter alternative to Object.prototype.hasOwnProperty.call():
    /// ```javascript
    /// if (Object.hasOwn(object, "foo")) {
    ///   console.log("has property foo")
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Object.prototype.hasOwnProperty.call(obj, "a");
    /// Object.hasOwnProperty.call(obj, "a");
    /// ({}).hasOwnProperty.call(obj, "a");
    /// const hasProperty = Object.prototype.hasOwnProperty.call(object, property);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Object.hasOwn(obj, "a");
    /// const hasProperty = Object.hasOwn(object, property);
    /// ```
    PreferObjectHasOwn,
    eslint,
    style,
    conditional_fix
);

impl Rule for PreferObjectHasOwn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(callee) = call_expr.callee.get_inner_expression().get_member_expr() else {
            return;
        };

        let Some(object) = callee.object().get_inner_expression().get_member_expr() else {
            return;
        };

        let object_property_name = object.static_property_name();
        let is_object = has_left_hand_object(object);
        let is_global_scope = ctx.scopes().find_binding(node.scope_id(), "Object").is_none();

        if is_method_call(call_expr, None, Some(&["call"]), Some(2), Some(2))
            && object_property_name == Some("hasOwnProperty")
            && is_object
            && is_global_scope
        {
            let replace_target_span = callee.span();
            let diagnostic = prefer_object_has_own_diagnostic(call_expr.span);
            if ctx.has_comments_between(replace_target_span) {
                ctx.diagnostic(diagnostic);
            } else {
                ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    let needs_space = replace_target_span.start > 1
                        && !ctx
                            .source_range(Span::new(0, replace_target_span.start))
                            .ends_with([' ', '=', '/', '(']);

                    let replacement = if needs_space { " Object.hasOwn" } else { "Object.hasOwn" };
                    fixer.replace(replace_target_span, replacement)
                });
            }
        }
    }
}

fn has_left_hand_object(node: &MemberExpression) -> bool {
    let object = node.object().get_inner_expression();

    if let Expression::ObjectExpression(object_expr) = object {
        return object_expr.properties.len() == 0;
    }

    let object_node_to_check = match object.get_member_expr() {
        Some(member_expr) => {
            if member_expr.static_property_name() == Some("prototype") {
                member_expr.object()
            } else {
                object
            }
        }
        _ => object,
    };

    if let Expression::Identifier(ident) = object_node_to_check.get_inner_expression() {
        return ident.name == "Object";
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Object",
        "Object(obj, prop)",
        "Object.hasOwnProperty",
        "Object.hasOwnProperty(prop)",
        "hasOwnProperty(obj, prop)",
        "foo.hasOwnProperty(prop)",
        "foo.hasOwnProperty(obj, prop)",
        "Object.hasOwnProperty.call",
        "foo.Object.hasOwnProperty.call(obj, prop)",
        "foo.hasOwnProperty.call(obj, prop)",
        "foo.call(Object.prototype.hasOwnProperty, Object.prototype.hasOwnProperty.call)",
        "Object.foo.call(obj, prop)",
        "Object.hasOwnProperty.foo(obj, prop)",
        "Object.hasOwnProperty.call.foo(obj, prop)",
        "Object[hasOwnProperty].call(obj, prop)",
        "Object.hasOwnProperty[call](obj, prop)",
        "class C { #hasOwnProperty; foo() { Object.#hasOwnProperty.call(obj, prop) } }",
        "class C { #call; foo() { Object.hasOwnProperty.#call(obj, prop) } }",
        "(Object) => Object.hasOwnProperty.call(obj, prop)",
        "Object.prototype",
        "Object.prototype(obj, prop)",
        "Object.prototype.hasOwnProperty",
        "Object.prototype.hasOwnProperty(obj, prop)",
        "Object.prototype.hasOwnProperty.call",
        "foo.Object.prototype.hasOwnProperty.call(obj, prop)",
        "foo.prototype.hasOwnProperty.call(obj, prop)",
        "Object.foo.hasOwnProperty.call(obj, prop)",
        "Object.prototype.foo.call(obj, prop)",
        "Object.prototype.hasOwnProperty.foo(obj, prop)",
        "Object.prototype.hasOwnProperty.call.foo(obj, prop)",
        "Object.prototype.prototype.hasOwnProperty.call(a, b);",
        "Object.hasOwnProperty.prototype.hasOwnProperty.call(a, b);",
        "Object.prototype[hasOwnProperty].call(obj, prop)",
        "Object.prototype.hasOwnProperty[call](obj, prop)",
        "class C { #hasOwnProperty; foo() { Object.prototype.#hasOwnProperty.call(obj, prop) } }",
        "class C { #call; foo() { Object.prototype.hasOwnProperty.#call(obj, prop) } }",
        "Object[prototype].hasOwnProperty.call(obj, prop)",
        "class C { #prototype; foo() { Object.#prototype.hasOwnProperty.call(obj, prop) } }",
        "(Object) => Object.prototype.hasOwnProperty.call(obj, prop)",
        "({})",
        "({}(obj, prop))",
        "({}.hasOwnProperty)",
        "({}.hasOwnProperty(prop))",
        "({}.hasOwnProperty(obj, prop))",
        "({}.hasOwnProperty.call)",
        "({}).prototype.hasOwnProperty.call(a, b);",
        "({}.foo.call(obj, prop))",
        "({}.hasOwnProperty.foo(obj, prop))",
        "({}[hasOwnProperty].call(obj, prop))",
        "({}.hasOwnProperty[call](obj, prop))",
        "({}).hasOwnProperty[call](object, property)",
        "({})[hasOwnProperty].call(object, property)",
        "class C { #hasOwnProperty; foo() { ({}.#hasOwnProperty.call(obj, prop)) } }",
        "class C { #call; foo() { ({}.hasOwnProperty.#call(obj, prop)) } }",
        "({ foo }.hasOwnProperty.call(obj, prop))",
        "(Object) => ({}).hasOwnProperty.call(obj, prop)",
        r#"
			        let obj = {};
			        Object.hasOwn(obj,"");
			        "#,
        "const hasProperty = Object.hasOwn(object, property);",
        // "/* global Object: off */
        // 	        ({}).hasOwnProperty.call(a, b);",
    ];

    let fail = vec![
        "Object.hasOwnProperty.call(obj, 'foo')",
        "Object.hasOwnProperty.call(obj, property)",
        "Object.prototype.hasOwnProperty.call(obj, 'foo')",
        "({}).hasOwnProperty.call(obj, 'foo')",
        "Object/* comment */.prototype.hasOwnProperty.call(a, b);",
        "const hasProperty = Object.prototype.hasOwnProperty.call(object, property);",
        "const hasProperty = (( Object.prototype.hasOwnProperty.call(object, property) ));",
        "const hasProperty = (( Object.prototype.hasOwnProperty.call ))(object, property);",
        "const hasProperty = (( Object.prototype.hasOwnProperty )).call(object, property);",
        "const hasProperty = (( Object.prototype )).hasOwnProperty.call(object, property);",
        "const hasProperty = (( Object )).prototype.hasOwnProperty.call(object, property);",
        "const hasProperty = {}.hasOwnProperty.call(object, property);",
        "const hasProperty={}.hasOwnProperty.call(object, property);",
        "const hasProperty = (( {}.hasOwnProperty.call(object, property) ));",
        "const hasProperty = (( {}.hasOwnProperty.call ))(object, property);",
        "const hasProperty = (( {}.hasOwnProperty )).call(object, property);",
        "const hasProperty = (( {} )).hasOwnProperty.call(object, property);",
        "function foo(){return {}.hasOwnProperty.call(object, property)}",
        "function foo(){return{}.hasOwnProperty.call(object, property)}",
        "function foo(){return/*comment*/{}.hasOwnProperty.call(object, property)}",
        "async function foo(){return await{}.hasOwnProperty.call(object, property)}",
        "async function foo(){return await/*comment*/{}.hasOwnProperty.call(object, property)}",
        "for (const x of{}.hasOwnProperty.call(object, property).toString());",
        "for (const x of/*comment*/{}.hasOwnProperty.call(object, property).toString());",
        "for (const x in{}.hasOwnProperty.call(object, property).toString());",
        "for (const x in/*comment*/{}.hasOwnProperty.call(object, property).toString());",
        "function foo(){return({}.hasOwnProperty.call)(object, property)}",
        "Object['prototype']['hasOwnProperty']['call'](object, property);",
        "Object[`prototype`][`hasOwnProperty`][`call`](object, property);",
        "Object['hasOwnProperty']['call'](object, property);",
        "Object[`hasOwnProperty`][`call`](object, property);",
        "({})['hasOwnProperty']['call'](object, property);",
        "({})[`hasOwnProperty`][`call`](object, property);",
        // Issue: <https://github.com/oxc-project/oxc/issues/7450>
        "Object.prototype.hasOwnProperty.call(C,x);",
    ];

    let fix = vec![
        ("Object.hasOwnProperty.call(obj, 'foo')", "Object.hasOwn(obj, 'foo')", None),
        ("Object.hasOwnProperty.call(obj, property)", "Object.hasOwn(obj, property)", None),
        ("Object.prototype.hasOwnProperty.call(obj, 'foo')", "Object.hasOwn(obj, 'foo')", None),
        ("({}).hasOwnProperty.call(obj, 'foo')", "Object.hasOwn(obj, 'foo')", None),
        (
            "const hasProperty = Object.prototype.hasOwnProperty.call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty = (( Object.prototype.hasOwnProperty.call(object, property) ));",
            "const hasProperty = (( Object.hasOwn(object, property) ));",
            None,
        ),
        (
            "const hasProperty = (( Object.prototype.hasOwnProperty.call ))(object, property);",
            "const hasProperty = (( Object.hasOwn ))(object, property);",
            None,
        ),
        (
            "const hasProperty = (( Object.prototype.hasOwnProperty )).call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty = (( Object.prototype )).hasOwnProperty.call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty = (( Object )).prototype.hasOwnProperty.call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty = {}.hasOwnProperty.call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty={}.hasOwnProperty.call(object, property);",
            "const hasProperty=Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty = (( {}.hasOwnProperty.call(object, property) ));",
            "const hasProperty = (( Object.hasOwn(object, property) ));",
            None,
        ),
        (
            "const hasProperty = (( {}.hasOwnProperty.call ))(object, property);",
            "const hasProperty = (( Object.hasOwn ))(object, property);",
            None,
        ),
        (
            "const hasProperty = (( {}.hasOwnProperty )).call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "const hasProperty = (( {} )).hasOwnProperty.call(object, property);",
            "const hasProperty = Object.hasOwn(object, property);",
            None,
        ),
        (
            "function foo(){return {}.hasOwnProperty.call(object, property)}",
            "function foo(){return Object.hasOwn(object, property)}",
            None,
        ),
        (
            "function foo(){return{}.hasOwnProperty.call(object, property)}",
            "function foo(){return Object.hasOwn(object, property)}",
            None,
        ),
        (
            "function foo(){return/*comment*/{}.hasOwnProperty.call(object, property)}",
            "function foo(){return/*comment*/Object.hasOwn(object, property)}",
            None,
        ),
        (
            "async function foo(){return await{}.hasOwnProperty.call(object, property)}",
            "async function foo(){return await Object.hasOwn(object, property)}",
            None,
        ),
        (
            "async function foo(){return await/*comment*/{}.hasOwnProperty.call(object, property)}",
            "async function foo(){return await/*comment*/Object.hasOwn(object, property)}",
            None,
        ),
        (
            "for (const x of{}.hasOwnProperty.call(object, property).toString());",
            "for (const x of Object.hasOwn(object, property).toString());",
            None,
        ),
        (
            "for (const x of/*comment*/{}.hasOwnProperty.call(object, property).toString());",
            "for (const x of/*comment*/Object.hasOwn(object, property).toString());",
            None,
        ),
        (
            "for (const x in{}.hasOwnProperty.call(object, property).toString());",
            "for (const x in Object.hasOwn(object, property).toString());",
            None,
        ),
        (
            "for (const x in/*comment*/{}.hasOwnProperty.call(object, property).toString());",
            "for (const x in/*comment*/Object.hasOwn(object, property).toString());",
            None,
        ),
        (
            "function foo(){return({}.hasOwnProperty.call)(object, property)}",
            "function foo(){return(Object.hasOwn)(object, property)}",
            None,
        ),
        (
            "Object['prototype']['hasOwnProperty']['call'](object, property);",
            "Object.hasOwn(object, property);",
            None,
        ),
        (
            "Object[`prototype`][`hasOwnProperty`][`call`](object, property);",
            "Object.hasOwn(object, property);",
            None,
        ),
        (
            "Object['hasOwnProperty']['call'](object, property);",
            "Object.hasOwn(object, property);",
            None,
        ),
        (
            "Object[`hasOwnProperty`][`call`](object, property);",
            "Object.hasOwn(object, property);",
            None,
        ),
        (
            "({})['hasOwnProperty']['call'](object, property);",
            "Object.hasOwn(object, property);",
            None,
        ),
        (
            "({})[`hasOwnProperty`][`call`](object, property);",
            "Object.hasOwn(object, property);",
            None,
        ),
        // Issue: <https://github.com/oxc-project/oxc/issues/7450>
        ("Object.prototype.hasOwnProperty.call(C,x);", " Object.hasOwn(C,x);", None),
    ];
    Tester::new(PreferObjectHasOwn::NAME, PreferObjectHasOwn::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
