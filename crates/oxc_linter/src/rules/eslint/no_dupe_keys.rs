use std::borrow::Cow;

use rustc_hash::{FxBuildHasher, FxHashMap};
use smallvec::SmallVec;

use oxc_ast::{
    AstKind, StaticPropertyName,
    ast::{ObjectMethodKind, ObjectProperty, ObjectPropertyKind, PropertyKey},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_dupe_keys_diagnostic(first: Span, second: Span, key: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate key '{key}'"))
        .with_help("Consider removing the duplicated key")
        .with_labels([
            first.label("Key is first defined here"),
            second.primary_label("and duplicated here"),
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
    correctness,
    recommended,
    version = "0.0.3",
    short_description = "Disallow duplicate keys in object literals.",
);

const INLINE_CAPACITY: usize = 8;

/// Spans of the latest keys defining the getter and setter of a property name. Properties that
/// are not accessors define both.
#[derive(Debug, Default, Clone, Copy)]
struct Definitions {
    get: Option<Span>,
    set: Option<Span>,
}

type KeyEntry<'a> = (StaticPropertyName<'a>, Definitions);

impl Rule for NoDupeKeys {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj_expr) = node.kind() else {
            return;
        };
        let len = obj_expr.properties.len();
        if len <= 1 {
            return;
        }
        let mut inline: SmallVec<[KeyEntry<'_>; INLINE_CAPACITY]> = SmallVec::new();
        // Bound linear searches to small objects; retain hashing for larger ones.
        let mut map = FxHashMap::with_capacity_and_hasher(
            if len > INLINE_CAPACITY { len } else { 0 },
            FxBuildHasher,
        );
        for prop in &obj_expr.properties {
            let (key, defines_get, defines_set) = match prop {
                ObjectPropertyKind::ObjectProperty(prop) => (&prop.key, true, true),
                ObjectPropertyKind::MethodProperty(prop) => match prop.kind {
                    ObjectMethodKind::Method => (&prop.key, true, true),
                    ObjectMethodKind::Get => (&prop.key, true, false),
                    ObjectMethodKind::Set => (&prop.key, false, true),
                },
                ObjectPropertyKind::SpreadProperty(_) => continue,
            };
            let name = if let PropertyKey::BooleanLiteral(lit) = key {
                StaticPropertyName::from(lit.as_str())
            } else if let Some(name) = key.static_name() {
                name
            } else {
                continue;
            };
            if let ObjectPropertyKind::ObjectProperty(prop) = prop
                && is_proto_setter_property(prop, &name)
            {
                continue;
            }
            let definitions = if len <= INLINE_CAPACITY {
                let index = inline.iter().position(|(key, _)| *key == name).unwrap_or_else(|| {
                    inline.push((name, Definitions::default()));
                    inline.len() - 1
                });
                &mut inline[index].1
            } else {
                map.entry(name).or_default()
            };
            let previous =
                [definitions.get.filter(|_| defines_get), definitions.set.filter(|_| defines_set)]
                    .into_iter()
                    .flatten()
                    .max_by_key(|span| span.start);
            let span = key.span();
            if defines_get {
                definitions.get = Some(span);
            }
            if defines_set {
                definitions.set = Some(span);
            }
            if let Some(previous) = previous {
                let name = prop_key_name(key, ctx);
                ctx.diagnostic(no_dupe_keys_diagnostic(previous, span, &name));
            }
        }
    }
}

fn is_proto_setter_property(prop: &ObjectProperty<'_>, name: &StaticPropertyName<'_>) -> bool {
    name == "__proto__" && !prop.key.is_computed() && !prop.shorthand
}

/// Display spelling for the diagnostic.
/// Key identity uses `StaticPropertyName`; never compare these values.
fn prop_key_name<'a>(key: &PropertyKey<'a>, ctx: &LintContext<'a>) -> Cow<'a, str> {
    match key {
        PropertyKey::Identifier(ident) => Cow::Borrowed(ident.name.as_str()),
        PropertyKey::StaticIdentifier(ident) => Cow::Borrowed(ident.name.as_str()),
        PropertyKey::PrivateIdentifier(ident) => Cow::Borrowed(ident.name.as_str()),
        PropertyKey::StaticStringLiteral(lit) | PropertyKey::StringLiteral(lit) => {
            match lit.value.as_str() {
                Some(name) => Cow::Borrowed(name),
                // Escape lone surrogates for display.
                None => Cow::Owned(lit.value.display().to_string()),
            }
        }
        PropertyKey::StaticNumericLiteral(lit) | PropertyKey::NumericLiteral(lit) => {
            Cow::Borrowed(lit.raw.as_ref().unwrap().as_str())
        }
        _ => Cow::Borrowed(ctx.source_range(key.span())),
    }
}
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"const c = { [/a/ig]: 1, "/a/ig": 2 };"#,
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
        // Lone surrogates are distinct property names.
        r#"var x = { "\uD800": 1, "\uDC00": 2 };"#,
        r#"var x = { "\uD800": 1, "\uD800\uDC00": 2 };"#,
        r#"var x = { "\uD800": 1, "\\uD800": 2 };"#,
        r#"var x = { "\uDC00\uD800": 1, "𐀀": 2 };"#,
        r#"var x = { "true": 0, [false]: 1 };"#,
        "var x = {
            a: 1,
            // eslint-disable-next-line no-dupe-keys
            a: 2,
        };",
    ];

    let fail = vec![
        "const a = { [/a/ig]: 1, [/a/gi]: 2 };",
        r#"const b = { [/a/ig]: 1, "/a/gi": 2 };"#,
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
        // Lone surrogates keep their identity across key syntaxes.
        r#"var x = { "\uD800": 1, "\uD800": 2 };"#,
        r#"var x = { "\uDC00": 1, ["\uDC00"]: 2 };"#,
        r#"var x = { "a\uD800b": 1, [`a\uD800b`]: 2 };"#,
        r#"var x = { "\uD800\uDC00": 1, "𐀀": 2 };"#,
        "var x = { get x() {}, set x(v) {}, get x() {} };",
        "var x = { set x(v) {}, get x() {}, set x(v) {} };",
        "var x = { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8, get x() {}, set x(v) {}, get x() {} };",
        "var x = { get a() {}, set a(v) {}, a: 1 };",
        r#"var x = { "true": 0, [true]: 1 };"#,
        "var x = { false: 0, [false]: 1 };",
        "var x = {
            // eslint-disable-next-line no-dupe-keys
            a: 1,
            a: 2,
        };",
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).test_and_snapshot();
}
