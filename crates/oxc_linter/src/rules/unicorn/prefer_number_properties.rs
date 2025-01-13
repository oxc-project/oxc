use oxc_ast::{
    ast::{match_member_expression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, globals::GLOBAL_OBJECT_NAMES, rule::Rule, AstNode};

fn prefer_number_properties_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `Number.{method_name}` instead of the global `{method_name}`"))
        .with_help(format!("Replace it with `Number.{method_name}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNumberProperties;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows use of `parseInt()`, `parseFloat()`, `isNan()`, `isFinite()`, `Nan`, `Infinity` and `-Infinity` as global variables.
    ///
    /// ### Why is this bad?
    ///
    /// ECMAScript 2015 moved globals onto the `Number` constructor for consistency and to slightly improve them. This rule enforces their usage to limit the usage of globals:
    ///
    /// - [`Number.parseInt()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/parseInt) over [`parseInt()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt)
    /// - [`Number.parseFloat()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/parseFloat) over [`parseFloat()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseFloat)
    /// - [`Number.isNaN()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isNaN) over [`isNaN()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/isNaN) *(they have slightly [different behavior](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isNaN#difference_between_number.isnan_and_global_isnan))*
    /// - [`Number.isFinite()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isFinite) over [`isFinite()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/isFinite) *(they have slightly [different behavior](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isFinite#difference_between_number.isfinite_and_global_isfinite))*
    /// - [`Number.NaN`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/NaN) over [`NaN`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NaN)
    /// - [`Number.POSITIVE_INFINITY`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/POSITIVE_INFINITY) over [`Infinity`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Infinity)
    /// - [`Number.NEGATIVE_INFINITY`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/NEGATIVE_INFINITY) over [`-Infinity`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Infinity)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = parseInt('10', 2);
    /// const bar = parseFloat('10.5');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = Number.parseInt('10', 2);
    /// const bar = Number.parseFloat('10.5');
    /// ```
    PreferNumberProperties,
    unicorn,
    restriction,
    pending
);

impl Rule for PreferNumberProperties {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::MemberExpression(member_expr) => {
                let Expression::Identifier(ident_name) = member_expr.object() else {
                    return;
                };

                if GLOBAL_OBJECT_NAMES.contains(ident_name.name.as_str()) {
                    match member_expr.static_property_name() {
                        Some("NaN") => {
                            ctx.diagnostic(prefer_number_properties_diagnostic(
                                member_expr.span(),
                                "NaN",
                            ));
                        }
                        Some("Infinity") => {
                            ctx.diagnostic(prefer_number_properties_diagnostic(
                                member_expr.span(),
                                "Infinity",
                            ));
                        }
                        _ => {}
                    }
                }
            }
            AstKind::IdentifierReference(ident_ref) => match ident_ref.name.as_str() {
                "NaN" | "Infinity" => {
                    ctx.diagnostic(prefer_number_properties_diagnostic(
                        ident_ref.span,
                        &ident_ref.name,
                    ));
                }
                _ => {}
            },
            AstKind::CallExpression(call_expr) => {
                let Some(ident_name) = extract_ident_from_expression(&call_expr.callee) else {
                    return;
                };

                if matches!(ident_name, "isNaN" | "isFinite" | "parseFloat" | "parseInt") {
                    ctx.diagnostic(prefer_number_properties_diagnostic(
                        call_expr.callee.span(),
                        ident_name,
                    ));
                }
            }
            _ => {}
        }
    }
}

fn extract_ident_from_expression<'b>(expr: &'b Expression<'_>) -> Option<&'b str> {
    match expr {
        Expression::Identifier(ident_name) => Some(ident_name.name.as_str()),
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            let Expression::Identifier(ident_name) = member_expr.object() else {
                return None;
            };

            if GLOBAL_OBJECT_NAMES.contains(ident_name.name.as_str()) {
                member_expr.static_property_name()
            } else {
                None
            }
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"Number.parseInt("10", 2);"#, None),
        (r#"Number.parseFloat("10.5");"#, None),
        (r"Number.isNaN(10);", None),
        (r"Number.isFinite(10);", None),
        (r"global.isFinite = Number.isFinite;", None),
        (r"global.isFinite ??= 1;", None),
        (r"isFinite ||= 1;", None),
        (r"[global.isFinite] = [];", None),
        (r"[global.isFinite = 1] = [];", None),
        (r"[[global.isFinite = 1]] = [];", None),
        (r"[isFinite] = [];", None),
        (r"[isFinite = 1] = [];", None),
        (r"[[isFinite = 1]] = [];", None),
        (r"({foo: global.isFinite} = {});", None),
        (r"({foo: global.isFinite = 1} = {});", None),
        (r"({foo: {bar: global.isFinite = 1}} = {});", None),
        (r"({foo: isFinite} = {});", None),
        (r"({foo: isFinite = 1} = {});", None),
        (r"({foo: {bar: isFinite = 1}} = {});", None),
        (r"delete global.isFinite;", None),
        (r"const foo = Number.NaN;", None),
        (r"const foo = window.Number.NaN;", None),
        (r"const foo = bar.NaN;", None),
        (r"const foo = nan;", None),
        (r#"const foo = "NaN";"#, None),
        (r"const {NaN} = {};", None),
        (r"const {a: NaN} = {};", None),
        (r"const {[a]: NaN} = {};", None),
        (r"const [NaN] = [];", None),
        (r"function NaN() {}", None),
        (r"const foo = function NaN() {}", None),
        (r"function foo(NaN) {}", None),
        (r"foo = function (NaN) {}", None),
        (r"foo = (NaN) => {}", None),
        (r"function foo({NaN}) {}", None),
        (r"function foo({a: NaN}) {}", None),
        (r"function foo({[a]: NaN}) {}", None),
        (r"function foo([NaN]) {}", None),
        (r"class NaN {}", None),
        (r"const Foo = class NaN {}", None),
        (r"class Foo {NaN(){}}", None),
        (r"class Foo {#NaN(){}}", None),
        (r"class Foo3 {NaN = 1}", None),
        (r"class Foo {#NaN = 1}", None),
        (r#"import {NaN} from "foo""#, None),
        (r#"import {NaN as NaN} from "foo""#, None),
        (r#"import NaN from "foo""#, None),
        (r#"import * as NaN from "foo""#, None),
        (r#"export {NaN} from "foo""#, None),
        (r#"export {NaN as NaN} from "foo""#, None),
        (r#"export * as NaN from "foo""#, None),
        (r"const foo = Number.POSITIVE_INFINITY;", None),
        (r"const foo = window.Number.POSITIVE_INFINITY;", None),
        (r"const foo = bar.POSITIVE_INFINITY;", None),
        (r"const foo = Number.Infinity;", None),
        (r"const foo = window.Number.Infinity;", None),
        (r"const foo = bar.Infinity;", None),
        (r"const foo = infinity;", None),
        (r#"const foo = "Infinite";"#, None),
        (r#"const foo = "-Infinity";"#, None),
        (r"const {Infinity} = {};", None),
        (r"function Infinity() {}", None),
        (r"class Infinity {}", None),
        (r"class Foo { Infinity(){}}", None),
        // (r#"const foo = Infinity;"#, Some(serde_json::json!([{"checkInfinity": false}]))),
        // (r#"const foo = -Infinity;"#, Some(serde_json::json!([{"checkInfinity": false}]))),
        (r"class Foo2 {NaN = 1}", None),
        (r"declare var NaN: number;", None),
        (r"declare function NaN(s: string, radix?: number): number;", None),
        (r"class Foo {NaN = 1}", None),
    ];

    let fail = vec![
        (r"const foo = NaN;", None),
        (r"if (Number.isNaN(NaN)) {}", None),
        (r"if (Object.is(foo, NaN)) {}", None),
        (r"const foo = bar[NaN];", None),
        (r"const foo = {NaN};", None),
        (r"const foo = {NaN: NaN};", None),
        (r"const {foo = NaN} = {};", None),
        (r"const foo = NaN.toString();", None),
        (r"class Foo3 {[NaN] = 1}", None),
        (r"class Foo2 {[NaN] = 1}", None),
        (r"class Foo {[NaN] = 1}", None),
        (r"const foo = {[NaN]: 1}", None),
        (r"const foo = {[NaN]() {}}", None),
        (r"foo[NaN] = 1;", None),
        (r"class A {[NaN](){}}", None),
        (r"foo = {[NaN]: 1}", None),
        (r"const foo = Infinity;", None),
        (r"if (Number.isNaN(Infinity)) {}", None),
        (r"if (Object.is(foo, Infinity)) {}", None),
        (r"const foo = bar[Infinity];", None),
        (r"const foo = {Infinity};", None),
        (r"const foo = {Infinity: Infinity};", None),
        (r"const foo = {[Infinity]: -Infinity};", None),
        (r"const foo = {[-Infinity]: Infinity};", None),
        (r"const foo = {Infinity: -Infinity};", None),
        (r"const {foo = Infinity} = {};", None),
        (r"const {foo = -Infinity} = {};", None),
        (r"const foo = Infinity.toString();", None),
        (r"const foo = -Infinity.toString();", None),
        (r"const foo = (-Infinity).toString();", None),
        (r"const foo = +Infinity;", None),
        (r"const foo = +-Infinity;", None),
        (r"const foo = -Infinity;", None),
        (r"const foo = -(-Infinity);", None),
        (r"const foo = 1 - Infinity;", None),
        (r"const foo = 1 - -Infinity;", None),
        (r"const isPositiveZero = value => value === 0 && 1 / value === Infinity;", None),
        (r"const isNegativeZero = value => value === 0 && 1 / value === -Infinity;", None),
        (r"const {a = NaN} = {};", None),
        (r"const {[NaN]: a = NaN} = {};", None),
        (r"const [a = NaN] = [];", None),
        (r"function foo({a = NaN}) {}", None),
        (r"function foo({[NaN]: a = NaN}) {}", None),
        (r"function foo([a = NaN]) {}", None),
        (r"function foo() {return-Infinity}", None),
        (r"globalThis.isNaN(foo);", None),
        (r"global.isNaN(foo);", None),
        (r"window.isNaN(foo);", None),
        (r"self.isNaN(foo);", None),
        (r"globalThis.parseFloat(foo);", None),
        (r"global.parseFloat(foo);", None),
        (r"window.parseFloat(foo);", None),
        (r"self.parseFloat(foo);", None),
        (r"globalThis.NaN", None),
        (r"-globalThis.Infinity", None),
    ];

    Tester::new(PreferNumberProperties::NAME, PreferNumberProperties::PLUGIN, pass, fail)
        .test_and_snapshot();
}
