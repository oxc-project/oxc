use oxc_ast::{
    ast::{ObjectPropertyKind, PropertyKey, PropertyKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_dupe_keys_diagnostic(first: Span, second: Span, key: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate key '{key}'"))
        .with_help("Consider removing the duplicated key")
        .with_labels([
            first.label("Key is first defined here"),
            second.label("and duplicated here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoDupeKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate keys in object literals
    ///
    /// ### Why is this bad?
    ///
    /// Multiple properties with the same key in object literals can cause
    /// unexpected behavior in your application.
    ///
    /// It is safe to disable this rule when using TypeScript because
    /// TypeScript's compiler enforces this check.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var foo = {
    ///     bar: "baz",
    ///     bar: "qux"
    /// };
    ///
    /// var foo = {
    ///     "bar": "baz",
    ///     bar: "qux"
    /// };
    ///
    /// var foo = {
    ///     0x1: "baz",
    ///     1: "qux"
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var foo = {
    ///     bar: "baz",
    ///     qux: "qux"
    /// };
    /// ```
    NoDupeKeys,
    eslint,
    correctness
);

impl Rule for NoDupeKeys {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj_expr) = node.kind() else {
            return;
        };
        let len = obj_expr.properties.len();
        if len <= 1 {
            return;
        }
        let mut map = FxHashMap::with_capacity_and_hasher(len, FxBuildHasher);
        for prop in &obj_expr.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                continue;
            };
            let Some(name) = prop.key.static_name() else {
                return;
            };
            if let Some((prev_kind, prev_span)) = map.insert(name, (prop.kind, prop.key.span())) {
                if prev_kind == PropertyKind::Init
                    || prop.kind == PropertyKind::Init
                    || prev_kind == prop.kind
                {
                    let name = prop_key_name(&prop.key, ctx);
                    ctx.diagnostic(no_dupe_keys_diagnostic(prev_span, prop.key.span(), name));
                }
            }
        }
    }
}

fn prop_key_name<'a>(key: &PropertyKey<'a>, ctx: &LintContext<'a>) -> &'a str {
    match key {
        PropertyKey::Identifier(ident) => ident.name.as_str(),
        PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
        PropertyKey::PrivateIdentifier(ident) => ident.name.as_str(),
        PropertyKey::StringLiteral(lit) => lit.value.as_str(),
        PropertyKey::NumericLiteral(lit) => lit.raw.as_ref().unwrap().as_str(),
        _ => ctx.source_range(key.span()),
    }
}
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = { __proto__: 1, two: 2};", None),
        ("var x = { foo: 1, bar: 2 };", None),
        ("var x = { '': 1, bar: 2 };", None),
        ("var x = { '': 1, ' ': 2 };", None),
        ("var x = { '': 1, [null]: 2 };", None),
        ("var x = { '': 1, [a]: 2 };", None),
        ("var x = { [a]: 1, [a]: 2 };", None),
        ("+{ get a() { }, set a(b) { } };", None),
        ("var x = { a: b, [a]: b };", None),
        ("var x = { a: b, ...c }", None),
        ("var x = { get a() {}, set a (value) {} };", None),
        ("var x = { a: 1, b: { a: 2 } };", None),
        ("var x = ({ null: 1, [/(?<zero>0)/]: 2 })", None),
        ("var {a, a} = obj", None),
        // Syntax:error: the '0' prefixed octal literals is not allowed.
        // ("var x = { 012: 1, 12: 2 };", None),
        ("var x = { 1_0: 1, 1: 2 };", None),
        // NOTE: This should fail when we get read the big int value
        ("var x = { 1n: 1, 1: 2 };", None),
    ];

    let fail = vec![
        ("var x = { a: b, ['a']: b };", None),
        ("var x = { y: 1, y: 2 };", None),
        ("var x = { '': 1, '': 2 };", None),
        ("var x = { '': 1, [``]: 2 };", None),
        ("var foo = { 0x1: 1, 1: 2};", None),
        ("var x = { 012: 1, 10: 2 };", None),
        ("var x = { 0b1: 1, 1: 2 };", None),
        ("var x = { 0o1: 1, 1: 2 };", None),
        ("var x = { 1_0: 1, 10: 2 };", None),
        ("var x = { \"z\": 1, z: 2 };", None),
        ("var foo = {\n  bar: 1,\n  bar: 1,\n}", None),
        ("var x = { a: 1, get a() {} };", None),
        ("var x = { a: 1, set a(value) {} };", None),
        ("var x = { a: 1, b: { a: 2 }, get b() {} };", None),
        ("var x = ({ '/(?<zero>0)/': 1, [/(?<zero>0)/]: 2 })", None),
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).test_and_snapshot();
}
