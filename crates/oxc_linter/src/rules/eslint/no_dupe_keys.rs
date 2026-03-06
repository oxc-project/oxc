use oxc_ast::{
    AstKind,
    ast::{ObjectPropertyKind, PropertyKey, PropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// Disallow duplicate keys in object literals.
    ///
    /// This rule can be disabled for TypeScript code, as the TypeScript compiler
    /// enforces this check.
    ///
    /// ### Why is this bad?
    ///
    /// Multiple properties with the same key in object literals can cause
    /// unexpected behavior in your application.
    ///
    /// ### Examples
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
                continue;
            };
            if is_proto_setter_property(prop, &name) {
                continue;
            }
            if let Some((prev_kind, prev_span)) = map.insert(name, (prop.kind, prop.key.span()))
                && (prev_kind == PropertyKind::Init
                    || prop.kind == PropertyKind::Init
                    || prev_kind == prop.kind)
            {
                let name = prop_key_name(&prop.key, ctx);
                ctx.diagnostic(no_dupe_keys_diagnostic(prev_span, prop.key.span(), name));
            }
        }
    }
}

fn is_proto_setter_property(prop: &oxc_ast::ast::ObjectProperty<'_>, name: &str) -> bool {
    name == "__proto__"
        && prop.kind == PropertyKind::Init
        && !prop.computed
        && !prop.shorthand
        && !prop.method
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
        "var foo = { __proto__: 1, two: 2};",
        "var x = { foo: 1, bar: 2 };",
        "var x = { '': 1, bar: 2 };",
        "var x = { '': 1, ' ': 2 };",
        "var x = { '': 1, [null]: 2 };", // { "ecmaVersion": 6 },
        "var x = { '': 1, [a]: 2 };",    // { "ecmaVersion": 6 },
        "var x = { [a]: 1, [a]: 2 };",   // { "ecmaVersion": 6 },
        "+{ get a() { }, set a(b) { } };",
        "var x = { a: b, [a]: b };", // { "ecmaVersion": 6 },
        "var x = { a: b, ...c }",    // { "ecmaVersion": 2018 },
        "var x = { get a() {}, set a (value) {} };", // { "ecmaVersion": 6 },
        "var x = { a: 1, b: { a: 2 } };", // { "ecmaVersion": 6 },
        "var x = ({ null: 1, [/(?<zero>0)/]: 2 })", // { "ecmaVersion": 2018 },
        "var {a, a} = obj",          // { "ecmaVersion": 6 },
        "var x = { 012: 1, 12: 2 };",
        "var x = { 1_0: 1, 1: 2 };", // { "ecmaVersion": 2021 },
        "var x = { __proto__: null, ['__proto__']: null };", // { "ecmaVersion": 6 },
        "var x = { ['__proto__']: null, __proto__: null };", // { "ecmaVersion": 6 },
        "var x = { '__proto__': null, ['__proto__']: null };", // { "ecmaVersion": 6 },
        "var x = { ['__proto__']: null, '__proto__': null };", // { "ecmaVersion": 6 },
        "var x = { __proto__: null, __proto__ };", // { "ecmaVersion": 6 },
        "var x = { __proto__, __proto__: null };", // { "ecmaVersion": 6 },
        "var x = { __proto__: null, __proto__() {} };", // { "ecmaVersion": 6 },
        "var x = { __proto__() {}, __proto__: null };", // { "ecmaVersion": 6 },
        "var x = { __proto__: null, get __proto__() {} };", // { "ecmaVersion": 6 },
        "var x = { get __proto__() {}, __proto__: null };", // { "ecmaVersion": 6 },
        "var x = { __proto__: null, set __proto__(value) {} };", // { "ecmaVersion": 6 },
        "var x = { set __proto__(value) {}, __proto__: null };", // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        "var x = { a: b, ['a']: b };", // { "ecmaVersion": 6 },
        "var x = { y: 1, y: 2 };",
        "var x = { '': 1, '': 2 };",
        "var x = { '': 1, [``]: 2 };", // { "ecmaVersion": 6 },
        "var foo = { 0x1: 1, 1: 2};",
        "var x = { 012: 1, 10: 2 };",
        "var x = { 0b1: 1, 1: 2 };",  // { "ecmaVersion": 6 },
        "var x = { 0o1: 1, 1: 2 };",  // { "ecmaVersion": 6 },
        "var x = { 1n: 1, 1: 2 };",   // { "ecmaVersion": 2020 },
        "var x = { 1_0: 1, 10: 2 };", // { "ecmaVersion": 2021 },
        r#"var x = { "z": 1, z: 2 };"#,
        "var foo = {
              bar: 1,
              bar: 1,
            }",
        "var x = { a: 1, get a() {} };",      // { "ecmaVersion": 6 },
        "var x = { a: 1, set a(value) {} };", // { "ecmaVersion": 6 },
        "var x = { a: 1, b: { a: 2 }, get b() {} };", // { "ecmaVersion": 6 },
        "var x = ({ '/(?<zero>0)/': 1, [/(?<zero>0)/]: 2 })", // { "ecmaVersion": 2018 },
        "var x = { ['__proto__']: null, ['__proto__']: null };", // { "ecmaVersion": 6 },
        "var x = { ['__proto__']: null, __proto__ };", // { "ecmaVersion": 6 },
        "var x = { ['__proto__']: null, __proto__() {} };", // { "ecmaVersion": 6 },
        "var x = { ['__proto__']: null, get __proto__() {} };", // { "ecmaVersion": 6 },
        "var x = { ['__proto__']: null, set __proto__(value) {} };", // { "ecmaVersion": 6 },
        "var x = { __proto__: null, a: 5, a: 6 };", // { "ecmaVersion": 6 }
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).test_and_snapshot();
}
