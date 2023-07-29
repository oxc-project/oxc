use oxc_ast::{
    ast::{ObjectPropertyKind, PropertyKey, PropertyKind, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;
use lazy_static::lazy_static;

use crate::{ast_util::calculate_hash, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-dupe-keys): Disallow duplicate keys in object literals")]
#[diagnostic(severity(warning), help("Consider removing the duplicated key"))]
struct NoDupeKeysDiagnostic(#[label] pub Span, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDupeKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate keys in object literals
    ///
    /// ### Why is this bad?
    ///
    /// Multiple properties with the same key in object literals can cause unexpected behavior in your application.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = {
    ///     bar: "baz",
    ///     bar: "qux"
    /// }
    /// ```
    NoDupeKeys,
    correctness
);

impl Rule for NoDupeKeys {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj_expr) = node.kind() else { return };
        let mut map = FxHashMap::default();
        for prop in &obj_expr.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else { continue };
            let Some(hash) = calculate_property_kind_hash(&prop.key) else { continue };
            if let Some((prev_kind, prev_span)) = map.insert(hash, (prop.kind, prop.key.span())) {
                if prev_kind == PropertyKind::Init
                    || prop.kind == PropertyKind::Init
                    || prev_kind == prop.kind
                {
                    ctx.diagnostic(NoDupeKeysDiagnostic(prev_span, prop.key.span()));
                }
            }
        }
    }
}

// todo: should this be located within oxc_ast?
fn calculate_property_kind_hash(key: &PropertyKey) -> Option<u64> {
    lazy_static! {
        static ref NULL_HASH: u64 = calculate_hash(&"null");
    }

    match key {
        PropertyKey::Identifier(ident) => Some(calculate_hash(&ident)),
        PropertyKey::PrivateIdentifier(_) => None,
        PropertyKey::Expression(expr) => match expr {
            Expression::StringLiteral(lit) => Some(calculate_hash(&lit.value)),
            // note: hashes won't work as expected if these aren't strings. Save
            // NumberLiteral I don't think this should be too much of a problem
            // b/c most people don't use `null`, regexes, etc. as object
            // property keys when writing real code.
            Expression::RegExpLiteral(lit) => Some(calculate_hash(&lit.regex.to_string())),
            Expression::NumberLiteral(lit) => Some(calculate_hash(&lit.value.to_string())),
            Expression::BigintLiteral(lit) => Some(calculate_hash(&lit.value.to_string())),
            Expression::NullLiteral(_) => Some(*NULL_HASH),
            Expression::TemplateLiteral(lit) => {
                lit.expressions.is_empty().then(|| lit.quasi()).flatten().map(calculate_hash)
            }
            _ => None,
        },
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
        ("var x = { 1n: 1, 1: 2 };", None),
        ("var x = { 1_0: 1, 10: 2 };", None),
        ("var x = { \"z\": 1, z: 2 };", None),
        ("var foo = {\n  bar: 1,\n  bar: 1,\n}", None),
        ("var x = { a: 1, get a() {} };", None),
        ("var x = { a: 1, set a(value) {} };", None),
        ("var x = { a: 1, b: { a: 2 }, get b() {} };", None),
        ("var x = ({ '/(?<zero>0)/': 1, [/(?<zero>0)/]: 2 })", None),
    ];

    Tester::new(NoDupeKeys::NAME, pass, fail).test_and_snapshot();
}
