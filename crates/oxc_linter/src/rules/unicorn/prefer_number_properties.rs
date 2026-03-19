use oxc_ast::{
    AstKind,
    ast::{Expression, UnaryExpression, UnaryOperator, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::RuleFixer,
    globals::GLOBAL_OBJECT_NAMES,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_number_properties_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `Number.{method_name}` instead of the global `{method_name}`"))
        .with_help(format!("Replace it with `Number.{method_name}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct PreferNumberProperties(Box<PreferNumberPropertiesConfig>);

impl std::ops::Deref for PreferNumberProperties {
    type Target = PreferNumberPropertiesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferNumberPropertiesConfig {
    /// If set to `true`, checks for usage of `Infinity` and `-Infinity` as global variables.
    check_infinity: bool,
    /// If set to `true`, checks for usage of `NaN` as a global variable.
    #[serde(rename = "checkNaN")]
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
    dangerous_fix,
    config = PreferNumberPropertiesConfig,
);

impl Rule for PreferNumberProperties {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
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
                            // Use replace on the full call expression span instead of insert_text_before.
                            // This ensures the fix span overlaps with prefer_numeric_literals fixes,
                            // so the fixer's conflict resolution will skip one of them instead of
                            // applying both and producing invalid code like `Number.0o111`.
                            let args_span = Span::new(ident.span.end, call_expr.span.end);
                            let args_text = ctx.source_range(args_span);
                            fixer.replace(call_expr.span, format!("Number.{ident_name}{args_text}"))
                        }
                        match_member_expression!(Expression) => {
                            let member_expr = call_expr.callee.to_member_expression();
                            let mut args_span =
                                Span::new(member_expr.span().end, call_expr.span.end);
                            if let Some(s) = &call_expr.type_arguments {
                                args_span = args_span.merge(s.span());
                            }
                            let args_text = ctx.source_range(args_span);

                            fixer.replace(call_expr.span, format!("Number.{ident_name}{args_text}"))
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
        ("Number.isNaN(10);", None),
        ("Number.isFinite(10);", None),
        (r#"const parseInt = function() {}; parseInt("10", 2);"#, None),
        (r#"const parseFloat = function() {}; parseFloat("10.5");"#, None),
        ("const isNaN = function() {}; isNaN(10);", None),
        ("const isFinite = function() {}; isFinite(10);", None),
        (r#"const {parseInt} = Number; parseInt("10", 2);"#, None),
        (r#"const {parseFloat} = Number; parseFloat("10.5");"#, None),
        ("const {isNaN} = Number; isNaN(10);", None),
        ("const {isFinite} = Number; isFinite(10);", None),
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
        ("global.isFinite = Number.isFinite;", None),
        ("global.isFinite ??= 1;", None),
        ("isFinite ||= 1;", None),
        ("[global.isFinite] = [];", None),
        ("[global.isFinite = 1] = [];", None),
        ("[[global.isFinite = 1]] = [];", None),
        ("[isFinite] = [];", None),
        ("[isFinite = 1] = [];", None),
        ("[[isFinite = 1]] = [];", None),
        ("({foo: global.isFinite} = {});", None),
        ("({foo: global.isFinite = 1} = {});", None),
        ("({foo: {bar: global.isFinite = 1}} = {});", None),
        ("({foo: isFinite} = {});", None),
        ("({foo: isFinite = 1} = {});", None),
        ("({foo: {bar: isFinite = 1}} = {});", None),
        ("delete global.isFinite;", None),
        ("const foo = Number.NaN;", None),
        ("const foo = window.Number.NaN;", None),
        ("const foo = bar.NaN;", None),
        ("const foo = nan;", None),
        (r#"const foo = "NaN";"#, None),
        ("function foo () { const NaN = 2; return NaN }", None),
        ("const {NaN} = {};", None),
        ("const {a: NaN} = {};", None),
        ("const {[a]: NaN} = {};", None),
        ("const [NaN] = [];", None),
        ("function NaN() {}", None),
        ("const foo = function NaN() {}", None),
        ("function foo(NaN) {}", None),
        ("foo = function (NaN) {}", None),
        ("foo = (NaN) => {}", None),
        ("function foo({NaN}) {}", None),
        ("function foo({a: NaN}) {}", None),
        ("function foo({[a]: NaN}) {}", None),
        ("function foo([NaN]) {}", None),
        ("class NaN {}", None),
        ("const Foo = class NaN {}", None),
        ("class Foo {NaN(){}}", None),
        ("class Foo {#NaN(){}}", None),
        ("class Foo3 {NaN = 1}", None),
        ("class Foo {#NaN = 1}", None),
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
        ("const foo = Number.POSITIVE_INFINITY;", None),
        ("const foo = window.Number.POSITIVE_INFINITY;", None),
        ("const foo = bar.POSITIVE_INFINITY;", None),
        ("const foo = Number.Infinity;", None),
        ("const foo = window.Number.Infinity;", None),
        ("const foo = bar.Infinity;", None),
        ("const foo = infinity;", None),
        (r#"const foo = "Infinity";"#, None),
        (r#"const foo = "-Infinity";"#, None),
        (
            r"function foo () {
    const Infinity = 2
    return Infinity
}",
            None,
        ),
        ("const {Infinity} = {};", None),
        ("function Infinity() {}", None),
        ("class Infinity {}", None),
        ("class Foo { Infinity(){}}", None),
        ("const foo = Infinity;", None),
        ("const foo = -Infinity;", None),
        ("const foo = NaN", Some(json!([{"checkNaN":false}]))),
        ("class Foo2 {NaN = 1}", None),
        (
            r"export enum NumberSymbol {
    Decimal,
    NaN,
}",
            None,
        ),
        ("declare var NaN: number;", None),
        (
            "interface NumberConstructor {
    readonly NaN: number;
}",
            None,
        ),
        ("declare function NaN(s: string, radix?: number): number;", None),
        ("class Foo {NaN = 1}", None),
        ("const foo = ++Infinity;", None),
        ("const foo = --Infinity;", None),
        ("const foo = -(--Infinity);", None),
    ];

    let fail = vec![
        (r#"parseInt("10", 2);"#, None),
        (r#"parseFloat("10.5");"#, None),
        ("isNaN(10);", None),
        ("isFinite(10);", None),
        (
            r#"const a = parseInt("10", 2);
    const b = parseFloat("10.5");
    const c = isNaN(10);
    const d = isFinite(10);"#,
            None,
        ),
        ("const foo = NaN;", None),
        ("if (Number.isNaN(NaN)) {}", None),
        ("if (Object.is(foo, NaN)) {}", None),
        ("const foo = bar[NaN];", None),
        ("const foo = {NaN};", None),
        ("const foo = {NaN: NaN};", None),
        ("const {foo = NaN} = {};", None),
        ("const foo = NaN.toString();", None),
        ("class Foo3 {[NaN] = 1}", None),
        ("const foo = Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("const foo = -Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("class Foo2 {[NaN] = 1}", None),
        ("class Foo {[NaN] = 1}", None),
        ("const foo = {[NaN]: 1}", None),
        ("const foo = {[NaN]() {}}", None),
        ("foo[NaN] = 1;", None),
        ("class A {[NaN](){}}", None),
        ("foo = {[NaN]: 1}", None),
        ("const foo = Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("if (Number.isNaN(Infinity)) {}", Some(json!([{"checkInfinity":true}]))),
        ("if (Object.is(foo, Infinity)) {}", Some(json!([{"checkInfinity":true}]))),
        ("const foo = bar[Infinity];", Some(json!([{"checkInfinity":true}]))),
        ("const foo = {Infinity};", Some(json!([{"checkInfinity":true}]))),
        ("const foo = {Infinity: Infinity};", Some(json!([{"checkInfinity":true}]))),
        ("const foo = {[Infinity]: -Infinity};", Some(json!([{"checkInfinity":true}]))),
        ("const foo = {[-Infinity]: Infinity};", Some(json!([{"checkInfinity":true}]))),
        ("const foo = {Infinity: -Infinity};", Some(json!([{"checkInfinity":true}]))),
        ("const {foo = Infinity} = {};", Some(json!([{"checkInfinity":true}]))),
        ("const {foo = -Infinity} = {};", Some(json!([{"checkInfinity":true}]))),
        ("const foo = Infinity.toString();", Some(json!([{"checkInfinity":true}]))),
        ("const foo = -Infinity.toString();", Some(json!([{"checkInfinity":true}]))),
        ("const foo = (-Infinity).toString();", Some(json!([{"checkInfinity":true}]))),
        ("const foo = +Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("const foo = +-Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("const foo = -Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("const foo = -(-Infinity);", Some(json!([{"checkInfinity":true}]))),
        ("const foo = 1 - Infinity;", Some(json!([{"checkInfinity":true}]))),
        ("const foo = 1 - -Infinity;", Some(json!([{"checkInfinity":true}]))),
        (
            r"const isPositiveZero = value => value === 0 && 1 / value === Infinity;",
            Some(json!([{"checkInfinity":true}])),
        ),
        (
            r"const isNegativeZero = value => value === 0 && 1 / value === -Infinity;",
            Some(json!([{"checkInfinity":true}])),
        ),
        ("const {a = NaN} = {};", None),
        ("const {[NaN]: a = NaN} = {};", None),
        ("const [a = NaN] = [];", None),
        ("function foo({a = NaN}) {}", None),
        ("function foo({[NaN]: a = NaN}) {}", None),
        ("function foo([a = NaN]) {}", None),
        ("function foo() {return-Infinity}", Some(json!([{"checkInfinity":true}]))),
        ("globalThis.isNaN(foo);", None),
        ("global.isNaN(foo);", None),
        ("window.isNaN(foo);", None),
        ("self.isNaN(foo);", None),
        ("globalThis.parseFloat(foo);", None),
        ("global.parseFloat(foo);", None),
        ("window.parseFloat(foo);", None),
        ("self.parseFloat(foo);", None),
        ("globalThis.NaN", None),
        ("-globalThis.Infinity", Some(json!([{"checkInfinity":true}]))),
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
            None::<serde_json::Value>,
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
        (r#"parseInt("111", 8)"#, r#"Number.parseInt("111", 8)"#, None),
    ];

    Tester::new(PreferNumberProperties::NAME, PreferNumberProperties::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
