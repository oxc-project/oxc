use oxc_ast::{
    AstKind,
    ast::{Expression, UnaryExpression, UnaryOperator, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{
    AstNode, context::LintContext, fixer::RuleFixer, globals::GLOBAL_OBJECT_NAMES, rule::Rule,
};

fn prefer_number_properties_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `Number.{method_name}` instead of the global `{method_name}`"))
        .with_help(format!("Replace it with `Number.{method_name}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNumberProperties(Box<PreferNumberPropertiesConfig>);

impl std::ops::Deref for PreferNumberProperties {
    type Target = PreferNumberPropertiesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct PreferNumberPropertiesConfig {
    // default is true
    check_infinity: bool,
    // default is true
    check_nan: bool,
}

impl Default for PreferNumberPropertiesConfig {
    fn default() -> Self {
        Self { check_infinity: false, check_nan: true }
    }
}

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
    dangerous_fix
);

impl Rule for PreferNumberProperties {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = PreferNumberPropertiesConfig::default();

        if let Some(value) = value.get(0) {
            if let Some(Value::Bool(val)) = value.get("checkInfinity") {
                config.check_infinity = *val;
            }
            if let Some(Value::Bool(val)) = value.get("checkNaN") {
                config.check_nan = *val;
            }
        }

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            member_expr if member_expr.is_member_expression_kind() => {
                let Some(member_expr) = member_expr.as_member_expression_kind() else {
                    return;
                };
                let Expression::Identifier(ident_name) = member_expr.object() else {
                    return;
                };

                if GLOBAL_OBJECT_NAMES.contains(&ident_name.name.as_str()) {
                    let Some(name) = member_expr.static_property_name() else {
                        return;
                    };
                    if (name == "NaN" && self.check_nan)
                        || (name == "Infinity" && self.check_infinity)
                    {
                        ctx.diagnostic_with_fix(
                            prefer_number_properties_diagnostic(member_expr.span(), name.as_str()),
                            |fixer| fixer.replace(ident_name.span, "Number"),
                        );
                    }
                }
            }
            AstKind::IdentifierReference(ident_ref)
                if ctx.is_reference_to_global_variable(ident_ref) =>
            {
                let ident_name = ident_ref.name.as_str();
                if (ident_name == "NaN" && self.check_nan)
                    || (ident_name == "Infinity" && self.check_infinity)
                    || (matches!(ident_name, "isNaN" | "isFinite" | "parseFloat" | "parseInt")
                        && matches!(ctx.nodes().parent_kind(node.id()), AstKind::ObjectProperty(_)))
                {
                    let (replacement_span, replacement_text) = if ident_name == "Infinity" {
                        if let Some(unary) = find_ancestor_unary(node, ctx) {
                            match unary.operator {
                                UnaryOperator::UnaryNegation => {
                                    (unary.span, "Number.NEGATIVE_INFINITY")
                                }
                                _ => (ident_ref.span, "Number.POSITIVE_INFINITY"),
                            }
                        } else {
                            (ident_ref.span, "Number.POSITIVE_INFINITY")
                        }
                    } else {
                        (ident_ref.span, "")
                    };

                    let fixer = |fixer: RuleFixer<'_, 'a>| match ctx.nodes().parent_kind(node.id())
                    {
                        AstKind::ObjectProperty(prop)
                            if prop.shorthand && ident_name == "Infinity" =>
                        {
                            fixer
                                .insert_text_after(&ident_ref.span, format!(": {replacement_text}"))
                        }
                        AstKind::ObjectProperty(prop) if prop.shorthand => fixer
                            .insert_text_before(&ident_ref.span, format!("{ident_name}: Number.")),
                        _ if ident_name == "Infinity" => {
                            fixer.replace(replacement_span, replacement_text)
                        }
                        _ => fixer.insert_text_before(&ident_ref.span, "Number."),
                    };

                    if ident_name == "isNaN" || ident_name == "isFinite" {
                        ctx.diagnostic_with_dangerous_fix(
                            prefer_number_properties_diagnostic(ident_ref.span, &ident_ref.name),
                            fixer,
                        );
                    } else {
                        ctx.diagnostic_with_fix(
                            prefer_number_properties_diagnostic(ident_ref.span, &ident_ref.name),
                            fixer,
                        );
                    }
                }
            }
            AstKind::CallExpression(call_expr) => {
                let Some(ident_name) = extract_ident_from_expression(&call_expr.callee) else {
                    return;
                };

                if matches!(ident_name, "isNaN" | "isFinite" | "parseFloat" | "parseInt") {
                    if let Expression::Identifier(ident) = &call_expr.callee
                        && !ctx.is_reference_to_global_variable(ident)
                    {
                        return;
                    }

                    let fixer = |fixer: RuleFixer<'_, 'a>| match &call_expr.callee {
                        Expression::Identifier(ident) => {
                            fixer.insert_text_before(&ident.span, "Number.")
                        }
                        match_member_expression!(Expression) => {
                            let member_expr = call_expr.callee.to_member_expression();

                            fixer.replace(member_expr.object().span(), "Number")
                        }
                        _ => unreachable!(),
                    };

                    if ident_name == "isFinite" || ident_name == "isNaN" {
                        ctx.diagnostic_with_dangerous_fix(
                            prefer_number_properties_diagnostic(
                                call_expr.callee.span(),
                                ident_name,
                            ),
                            fixer,
                        );
                    } else {
                        ctx.diagnostic_with_fix(
                            prefer_number_properties_diagnostic(
                                call_expr.callee.span(),
                                ident_name,
                            ),
                            fixer,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Finds the nearest enclosing unary expression ancestor for `node`.
fn find_ancestor_unary<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a UnaryExpression<'a>> {
    ctx.nodes().ancestor_kinds(node.id()).find_map(|ancestor| {
        if let AstKind::UnaryExpression(unary_expr) = ancestor { Some(unary_expr) } else { None }
    })
}

fn extract_ident_from_expression<'b>(expr: &'b Expression<'_>) -> Option<&'b str> {
    match expr {
        Expression::Identifier(ident_name) => Some(ident_name.name.as_str()),
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            let Expression::Identifier(ident_name) = member_expr.object() else {
                return None;
            };

            if GLOBAL_OBJECT_NAMES.contains(&ident_name.name.as_str()) {
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
    use serde_json::json;

    let pass = vec![
        (r#"Number.parseInt("10", 2);"#, None),
        (r#"Number.parseFloat("10.5");"#, None),
        (r"Number.isNaN(10);", None),
        (r"Number.isFinite(10);", None),
        (r#"const parseInt = function() {}; parseInt("10", 2);"#, None),
        (r#"const parseFloat = function() {}; parseFloat("10.5");"#, None),
        (r"const isNaN = function() {}; isNaN(10);", None),
        (r"const isFinite = function() {}; isFinite(10);", None),
        (r#"const {parseInt} = Number; parseInt("10", 2);"#, None),
        (r#"const {parseFloat} = Number; parseFloat("10.5");"#, None),
        (r"const {isNaN} = Number; isNaN(10);", None),
        (r"const {isFinite} = Number; isFinite(10);", None),
        (
            r#"const parseInt = function() {};
function inner() {
	return parseInt("10", 2);
}"#,
            None,
        ),
        (
            r#"const parseFloat = function() {};
function inner() {
	return parseFloat("10.5");
}"#,
            None,
        ),
        (
            r"const isNaN = function() {};
function inner() {
	return isNaN(10);
}",
            None,
        ),
        (
            r"const isFinite = function() {};
function inner() {
	return isFinite(10);
}",
            None,
        ),
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
        (r"function foo () { const NaN = 2; return NaN }", None),
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
        (
            r"NaN: for (const foo of bar) {
	if (a) {
		continue NaN;
	} else {
		break NaN;
	}
}",
            None,
        ),
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
        (r#"const foo = "Infinity";"#, None),
        (r#"const foo = "-Infinity";"#, None),
        (
            r"function foo () {
	const Infinity = 2
	return Infinity
}",
            None,
        ),
        (r"const {Infinity} = {};", None),
        (r"function Infinity() {}", None),
        (r"class Infinity {}", None),
        (r"class Foo { Infinity(){}}", None),
        (r"const foo = Infinity;", None),
        (r"const foo = -Infinity;", None),
        (r"const foo = NaN", Some(json!([{"checkNaN":false}]))),
        (r"class Foo2 {NaN = 1}", None),
        (
            r"export enum NumberSymbol {
	Decimal,
	NaN,
}",
            None,
        ),
        (r"declare var NaN: number;", None),
        (
            r"interface NumberConstructor {
	readonly NaN: number;
}",
            None,
        ),
        (r"declare function NaN(s: string, radix?: number): number;", None),
        (r"class Foo {NaN = 1}", None),
        (r"const foo = ++Infinity;", None),
        (r"const foo = --Infinity;", None),
        (r"const foo = -(--Infinity);", None),
    ];

    let fail = vec![
        (r#"parseInt("10", 2);"#, None),
        (r#"parseFloat("10.5");"#, None),
        (r"isNaN(10);", None),
        (r"isFinite(10);", None),
        (
            r#"const a = parseInt("10", 2);
    const b = parseFloat("10.5");
    const c = isNaN(10);
    const d = isFinite(10);"#,
            None,
        ),
        (r"const foo = NaN;", None),
        (r"if (Number.isNaN(NaN)) {}", None),
        (r"if (Object.is(foo, NaN)) {}", None),
        (r"const foo = bar[NaN];", None),
        (r"const foo = {NaN};", None),
        (r"const foo = {NaN: NaN};", None),
        (r"const {foo = NaN} = {};", None),
        (r"const foo = NaN.toString();", None),
        (r"class Foo3 {[NaN] = 1}", None),
        (r"const foo = Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = -Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"class Foo2 {[NaN] = 1}", None),
        (r"class Foo {[NaN] = 1}", None),
        (r"const foo = {[NaN]: 1}", None),
        (r"const foo = {[NaN]() {}}", None),
        (r"foo[NaN] = 1;", None),
        (r"class A {[NaN](){}}", None),
        (r"foo = {[NaN]: 1}", None),
        (r"const foo = Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"if (Number.isNaN(Infinity)) {}", Some(json!([{"checkInfinity":true}]))),
        (r"if (Object.is(foo, Infinity)) {}", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = bar[Infinity];", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = {Infinity};", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = {Infinity: Infinity};", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = {[Infinity]: -Infinity};", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = {[-Infinity]: Infinity};", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = {Infinity: -Infinity};", Some(json!([{"checkInfinity":true}]))),
        (r"const {foo = Infinity} = {};", Some(json!([{"checkInfinity":true}]))),
        (r"const {foo = -Infinity} = {};", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = Infinity.toString();", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = -Infinity.toString();", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = (-Infinity).toString();", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = +Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = +-Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = -Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = -(-Infinity);", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = 1 - Infinity;", Some(json!([{"checkInfinity":true}]))),
        (r"const foo = 1 - -Infinity;", Some(json!([{"checkInfinity":true}]))),
        (
            r"const isPositiveZero = value => value === 0 && 1 / value === Infinity;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            r"const isNegativeZero = value => value === 0 && 1 / value === -Infinity;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (r"const {a = NaN} = {};", None),
        (r"const {[NaN]: a = NaN} = {};", None),
        (r"const [a = NaN] = [];", None),
        (r"function foo({a = NaN}) {}", None),
        (r"function foo({[NaN]: a = NaN}) {}", None),
        (r"function foo([a = NaN]) {}", None),
        (r"function foo() {return-Infinity}", Some(json!([{"checkInfinity":true}]))),
        (r"globalThis.isNaN(foo);", None),
        (r"global.isNaN(foo);", None),
        (r"window.isNaN(foo);", None),
        (r"self.isNaN(foo);", None),
        (r"globalThis.parseFloat(foo);", None),
        (r"global.parseFloat(foo);", None),
        (r"window.parseFloat(foo);", None),
        (r"self.parseFloat(foo);", None),
        (r"globalThis.NaN", None),
        (r"-globalThis.Infinity", Some(json!([{"checkInfinity":true}]))),
        (
            r"const options = {
            normalize: parseFloat,
             parseInt,
         };

         run(foo, options);",
            None,
        ),
    ];

    let fix = vec![
        (
            r#"const a = parseInt("10", 2);
			const b = parseFloat("10.5");
			const c = isNaN(10);
			const d = isFinite(10);"#,
            r#"const a = Number.parseInt("10", 2);
			const b = Number.parseFloat("10.5");
			const c = Number.isNaN(10);
			const d = Number.isFinite(10);"#,
            None::<Value>,
        ),
        ("const foo = NaN;", "const foo = Number.NaN;", None),
        ("if (Number.isNaN(NaN)) {}", "if (Number.isNaN(Number.NaN)) {}", None),
        ("if (Object.is(foo, NaN)) {}", "if (Object.is(foo, Number.NaN)) {}", None),
        ("const foo = bar[NaN];", "const foo = bar[Number.NaN];", None),
        ("const foo = {NaN};", "const foo = {NaN: Number.NaN};", None),
        ("const foo = {NaN: NaN};", "const foo = {NaN: Number.NaN};", None),
        ("const {foo = NaN} = {};", "const {foo = Number.NaN} = {};", None),
        ("const foo = NaN.toString();", "const foo = Number.NaN.toString();", None),
        ("class Foo3 {[NaN] = 1}", "class Foo3 {[Number.NaN] = 1}", None),
        ("class Foo2 {[NaN] = 1}", "class Foo2 {[Number.NaN] = 1}", None),
        ("class Foo {[NaN] = 1}", "class Foo {[Number.NaN] = 1}", None),
        (
            "const foo = Infinity;",
            "const foo = Number.POSITIVE_INFINITY;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            "const foo = -Infinity;",
            "const foo = Number.NEGATIVE_INFINITY;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            "const foo = -(Infinity);",
            "const foo = Number.NEGATIVE_INFINITY;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            "const foo = -((Infinity));",
            "const foo = Number.NEGATIVE_INFINITY;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            "let a = { Infinity, }",
            "let a = { Infinity: Number.POSITIVE_INFINITY, }",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            "let a = (Infinity)",
            "let a = (Number.POSITIVE_INFINITY)",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            "let a = +(Infinity)",
            "let a = +(Number.POSITIVE_INFINITY)",
            Some(json!([{"checkInfinity":true}])),
        ),
    ];

    Tester::new(PreferNumberProperties::NAME, PreferNumberProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
