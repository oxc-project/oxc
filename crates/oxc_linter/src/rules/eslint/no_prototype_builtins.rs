use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint(no-prototype-builtins): do not access Object.prototype method {0:?} from target object"
)]
#[diagnostic(
    severity(warning),
    help("to avoid prototype pollution, use `Object.prototype.{0}.call` instead")
)]
struct NoPrototypeBuiltinsDiagnostic(String, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoPrototypeBuiltins;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow calling some Object.prototype methods directly on objects
    ///
    /// ### Why is this bad?
    ///
    /// In ECMAScript 5.1, Object.create was added, which enables the creation of objects with a specified [[Prototype]].
    /// Object.create(null) is a common pattern used to create objects that will be used as a Map.
    /// This can lead to errors when it is assumed that objects will have properties from Object.prototype. This rule prevents calling some Object.prototype methods directly from an object.
    /// Additionally, objects can have properties that shadow the builtins on Object.prototype, potentially causing unintended behavior or denial-of-service security vulnerabilities.
    /// For example, it would be unsafe for a webserver to parse JSON input from a client and call hasOwnProperty directly on the resulting object, because a malicious client could send a JSON value like {"hasOwnProperty": 1} and cause the server to crash.
    ///
    /// To avoid subtle bugs like this, it’s better to always call these methods from Object.prototype. For example, foo.hasOwnProperty("bar") should be replaced with Object.prototype.hasOwnProperty.call(foo, "bar").
    ///
    ///
    /// ### Example
    /// ```javascript
    /// var hasBarProperty = foo.hasOwnProperty("bar");
    /// var isPrototypeOfBar = foo.isPrototypeOf(bar);
    /// var barIsEnumerable = foo.propertyIsEnumerable("bar");
    /// ```
    NoPrototypeBuiltins,
    correctness
);

const DISALLOWED_PROPS: &[&str; 3] = &["hasOwnProperty", "isPrototypeOf", "propertyIsEnumerable"];

impl Rule for NoPrototypeBuiltins {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(expr) = node.kind()
        && let Some(member_expr) = expr.callee.get_member_expr()
        && let Some(prop_name) = member_expr.static_property_name()
        && DISALLOWED_PROPS.contains(&prop_name){
            ctx.diagnostic(NoPrototypeBuiltinsDiagnostic(
                    prop_name.to_string(),
                    member_expr.span(),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Object.prototype.hasOwnProperty.call(foo, 'bar')",
        "Object.prototype.isPrototypeOf.call(foo, 'bar')",
        "Object.prototype.propertyIsEnumerable.call(foo, 'bar')",
        "Object.prototype.hasOwnProperty.apply(foo, ['bar'])",
        "Object.prototype.isPrototypeOf.apply(foo, ['bar'])",
        "Object.prototype.propertyIsEnumerable.apply(foo, ['bar'])",
        "foo.hasOwnProperty",
        "foo.hasOwnProperty.bar()",
        "foo(hasOwnProperty)",
        "hasOwnProperty(foo, 'bar')",
        "isPrototypeOf(foo, 'bar')",
        "propertyIsEnumerable(foo, 'bar')",
        "({}.hasOwnProperty.call(foo, 'bar'))",
        "({}.isPrototypeOf.call(foo, 'bar'))",
        "({}.propertyIsEnumerable.call(foo, 'bar'))",
        "({}.hasOwnProperty.apply(foo, ['bar']))",
        "({}.isPrototypeOf.apply(foo, ['bar']))",
        "({}.propertyIsEnumerable.apply(foo, ['bar']))",
        "foo[hasOwnProperty]('bar')",
        "foo['HasOwnProperty']('bar')",
        "foo[`isPrototypeOff`]('bar')",
        "foo?.['propertyIsEnumerabl']('bar')",
        "foo[1]('bar')",
        "foo[null]('bar')",
        "class C { #hasOwnProperty; foo() { obj.#hasOwnProperty('bar'); } }",
        "foo['hasOwn' + 'Property']('bar')",
        "foo[`hasOwnProperty${''}`]('bar')",
    ];

    let fail = vec![
        "foo.hasOwnProperty('bar')",
        "foo.isPrototypeOf('bar')",
        "foo.propertyIsEnumerable('bar')",
        "foo.bar.hasOwnProperty('bar')",
        "foo.bar.baz.isPrototypeOf('bar')",
        "foo['hasOwnProperty']('bar')",
        "foo[`isPrototypeOf`]('bar').baz",
        "foo.bar[\"propertyIsEnumerable\"]('baz')",
        "foo?.hasOwnProperty('bar')",
        "(foo?.hasOwnProperty)('bar')",
        "foo?.['hasOwnProperty']('bar')",
        "(foo?.[`hasOwnProperty`])('bar')",
    ];

    Tester::new_without_config(NoPrototypeBuiltins::NAME, pass, fail).test_and_snapshot();
}
