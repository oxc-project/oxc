use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_compat::{Engine, EngineTargets, Version};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn unnecessary_polyfill_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use built-in instead.").with_label(span)
}

fn unnecessary_core_js_module_diagnostic(span: Span, core_js_module: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "All polyfilled features imported from `{core_js_module}` are available as built-ins. Use the built-ins instead."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryPolyfills {
    node_target: Option<Version>,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NoUnnecessaryPolyfillsOptions {
    #[schemars(with = "Option<serde_json::Value>")]
    targets: Option<EngineTargets>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of built-ins over unnecessary polyfills.
    ///
    /// ### Why is this bad?
    ///
    /// Importing polyfills for features already available in your target runtime
    /// adds avoidable dependency, bundle, and maintenance cost.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import assign from "object-assign";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const assign = Object.assign;
    /// ```
    NoUnnecessaryPolyfills,
    unicorn,
    restriction,
    config = NoUnnecessaryPolyfillsOptions
);

impl Rule for NoUnnecessaryPolyfills {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let options =
            serde_json::from_value::<DefaultRuleConfig<NoUnnecessaryPolyfillsOptions>>(value)
                .map(DefaultRuleConfig::into_inner)?;
        let node_target = options.targets.and_then(|targets| targets.get(&Engine::Node).copied());

        Ok(Self { node_target })
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(node_target) = self.node_target else {
            return;
        };
        let Some((module_name, span)) = get_imported_module_name_and_span(node) else {
            return;
        };

        if module_name.starts_with('.') || module_name.starts_with('/') {
            return;
        }

        match classify_polyfill(module_name, node_target) {
            Some(DiagnosticKind::Polyfill) => {
                ctx.diagnostic(unnecessary_polyfill_diagnostic(span));
            }
            Some(DiagnosticKind::CoreJsModule) => {
                ctx.diagnostic(unnecessary_core_js_module_diagnostic(span, module_name));
            }
            None => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DiagnosticKind {
    Polyfill,
    CoreJsModule,
}

fn get_imported_module_name_and_span<'a>(node: &AstNode<'a>) -> Option<(&'a str, Span)> {
    match node.kind() {
        AstKind::ImportDeclaration(import_decl) => {
            Some((import_decl.source.value.as_str(), import_decl.source.span))
        }
        AstKind::ImportExpression(import_expr) => {
            let Expression::StringLiteral(string_lit) = import_expr.source.get_inner_expression()
            else {
                return None;
            };
            Some((string_lit.value.as_str(), string_lit.span))
        }
        AstKind::CallExpression(call_expr)
            if !call_expr.optional
                && call_expr.callee.is_specific_id("require")
                && call_expr.arguments.len() == 1 =>
        {
            call_expr.arguments.first().and_then(Argument::as_expression).and_then(|arg| match arg
                .get_inner_expression()
            {
                Expression::StringLiteral(string_lit) => {
                    Some((string_lit.value.as_str(), string_lit.span))
                }
                _ => None,
            })
        }
        _ => None,
    }
}

fn classify_polyfill(module_name: &str, node_target: Version) -> Option<DiagnosticKind> {
    let normalized_core_js_module = normalize_core_js_module_name(module_name);
    if let Some(features) = core_js_module_features(normalized_core_js_module.as_ref()) {
        if features.len() > 1 {
            if has_any_supported_feature(node_target, features) {
                return Some(DiagnosticKind::CoreJsModule);
            }
        } else if has_any_supported_feature(node_target, features) {
            return Some(DiagnosticKind::Polyfill);
        }
        return None;
    }

    let lowercased = module_name.cow_to_ascii_lowercase();
    let features = polyfill_module_features(lowercased.as_ref())?;
    if has_any_supported_feature(node_target, features) {
        Some(DiagnosticKind::Polyfill)
    } else {
        None
    }
}

fn normalize_core_js_module_name(module_name: &str) -> Cow<'_, str> {
    if let Some(rest) = module_name.strip_prefix("core-js-pure/") {
        Cow::Owned(format!("core-js/{rest}"))
    } else {
        Cow::Borrowed(module_name)
    }
}

#[derive(Debug, Clone, Copy)]
enum Feature {
    ArrayFindIndex,
    ArrayFrom,
    ArrayLastIndexOf,
    ArrayTyped,
    CodePointAt,
    Float64Array,
    ObjectAssign,
    ObjectGetOwnPropertyDescriptors,
    ObjectSetPrototypeOf,
    Promise,
    PromiseFinally,
    RegExpEscape,
    StringPadStart,
    Symbol,
    WeakMap,
}

#[derive(Debug, Clone, Copy)]
struct NodeVersion(u32, u32, u32);

fn minimum_node_version(feature: Feature) -> NodeVersion {
    match feature {
        Feature::ArrayFindIndex => NodeVersion(4, 0, 0),
        Feature::ArrayFrom => NodeVersion(7, 1, 0),
        Feature::ArrayLastIndexOf => NodeVersion(6, 6, 0),
        Feature::ArrayTyped => NodeVersion(16, 1, 0),
        Feature::CodePointAt | Feature::ObjectAssign | Feature::ObjectSetPrototypeOf => {
            NodeVersion(4, 1, 0)
        }
        Feature::Float64Array => NodeVersion(17, 0, 0),
        Feature::ObjectGetOwnPropertyDescriptors | Feature::StringPadStart => NodeVersion(8, 1, 0),
        Feature::Promise | Feature::Symbol => NodeVersion(15, 1, 0),
        Feature::PromiseFinally => NodeVersion(10, 5, 0),
        Feature::RegExpEscape => NodeVersion(24, 0, 0),
        Feature::WeakMap => NodeVersion(12, 0, 0),
    }
}

fn has_any_supported_feature(node_target: Version, features: &[Feature]) -> bool {
    features.iter().copied().any(|feature| {
        let min_version = minimum_node_version(feature);
        version_is_gte(node_target, min_version)
    })
}

fn version_is_gte(version: Version, min_version: NodeVersion) -> bool {
    (u32::from(version.0), u32::from(version.1), u32::from(version.2))
        >= (min_version.0, min_version.1, min_version.2)
}

fn core_js_module_features(module_name: &str) -> Option<&'static [Feature]> {
    match module_name {
        "core-js/features/array/last-index-of" => Some(&[Feature::ArrayLastIndexOf]),
        "core-js/features/array/from" => Some(&[Feature::ArrayFrom, Feature::Symbol]),
        "core-js/features/typed-array" => Some(&[Feature::ArrayTyped, Feature::Float64Array]),
        "core-js/full/regexp/escape" => Some(&[Feature::RegExpEscape]),
        _ => None,
    }
}

fn polyfill_module_features(module_name: &str) -> Option<&'static [Feature]> {
    match module_name {
        "array-find-index" | "mdn-polyfills/array.prototype.findindex" => {
            Some(&[Feature::ArrayFindIndex])
        }
        "array-from" => Some(&[Feature::ArrayFrom]),
        "code-point-at" => Some(&[Feature::CodePointAt]),
        "es6-promise" | "promise-polyfill" => Some(&[Feature::Promise]),
        "es6-symbol" => Some(&[Feature::Symbol]),
        "object-assign" => Some(&[Feature::ObjectAssign]),
        "object.getownpropertydescriptors" => Some(&[Feature::ObjectGetOwnPropertyDescriptors]),
        "p-finally" => Some(&[Feature::PromiseFinally]),
        "regexp.escape" => Some(&[Feature::RegExpEscape]),
        "setprototypeof" => Some(&[Feature::ObjectSetPrototypeOf]),
        "string.prototype.padstart" => Some(&[Feature::StringPadStart]),
        "typed-array-float64-array-polyfill" => Some(&[Feature::Float64Array]),
        "weakmap-polyfill" => Some(&[Feature::WeakMap]),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"require("object-assign")"#, Some(serde_json::json!([{"targets": {"node": "0.1.0"}}]))),
        (
            r#"import regexpEscape from "regexp.escape""#,
            Some(serde_json::json!([{"targets": {"node": "18"}}])),
        ),
        (
            r#"require("core-js/full/regexp/escape")"#,
            Some(serde_json::json!([{"targets": {"node": "18"}}])),
        ),
        (
            r#"require("this-is-not-a-polyfill")"#,
            Some(serde_json::json!([{"targets": {"node": "0.1.0"}}])),
        ),
        (
            r#"import assign from "object-assign""#,
            Some(serde_json::json!([{"targets": {"node": "0.1.0"}}])),
        ),
        (r#"import("object-assign")"#, Some(serde_json::json!([{"targets": {"node": "0.1.0"}}]))),
        (r#"require("object-assign")"#, Some(serde_json::json!([{"targets": "node <4"}]))),
        (r#"require("object-assign")"#, Some(serde_json::json!([{"targets": "node >3"}]))),
        ("require()", Some(serde_json::json!([{"targets": "node >3"}]))),
        (r#"import("")"#, Some(serde_json::json!([{"targets": "node >3"}]))),
        ("import(null)", Some(serde_json::json!([{"targets": "node >3"}]))),
        ("require(null)", Some(serde_json::json!([{"targets": "node >3"}]))),
        (r#"require("" )"#, Some(serde_json::json!([{"targets": "node >3"}]))),
        (r#"import ExtendableError from "es6-error""#, None),
    ];

    let fail = vec![
        (r#"require("setprototypeof")"#, Some(serde_json::json!([{"targets": "node >4"}]))),
        (
            r#"require("core-js/features/array/last-index-of")"#,
            Some(serde_json::json!([{"targets": "node >6.5"}])),
        ),
        (
            r#"require("core-js-pure/features/array/from")"#,
            Some(serde_json::json!([{"targets": "node >7"}])),
        ),
        (
            r#"require("core-js/features/array/from")"#,
            Some(serde_json::json!([{"targets": "node >7"}])),
        ),
        (
            r#"require("core-js/features/typed-array")"#,
            Some(serde_json::json!([{"targets": "node >16"}])),
        ),
        (r#"require("es6-symbol")"#, Some(serde_json::json!([{"targets": "node >15"}]))),
        (r#"require("code-point-at")"#, Some(serde_json::json!([{"targets": "node >4"}]))),
        (
            r#"require("object.getownpropertydescriptors")"#,
            Some(serde_json::json!([{"targets": "node >8"}])),
        ),
        (
            r#"require("string.prototype.padstart")"#,
            Some(serde_json::json!([{"targets": "node >8"}])),
        ),
        (r#"require("p-finally")"#, Some(serde_json::json!([{"targets": "node >10.4"}]))),
        (r#"require("promise-polyfill")"#, Some(serde_json::json!([{"targets": "node >15"}]))),
        (r#"require("es6-promise")"#, Some(serde_json::json!([{"targets": "node >15"}]))),
        (r#"require("object-assign")"#, Some(serde_json::json!([{"targets": "node 6"}]))),
        (r#"import assign from "object-assign""#, Some(serde_json::json!([{"targets": "node 6"}]))),
        (r#"import("object-assign")"#, Some(serde_json::json!([{"targets": "node 6"}]))),
        (r#"require("object-assign")"#, Some(serde_json::json!([{"targets": "node >6"}]))),
        (r#"require("object-assign")"#, Some(serde_json::json!([{"targets": "node 8"}]))),
        (r#"require("array-from")"#, Some(serde_json::json!([{"targets": "node >7"}]))),
        (r#"require("array-find-index")"#, Some(serde_json::json!([{"targets": "node >4.0.0"}]))),
        (r#"require("array-find-index")"#, Some(serde_json::json!([{"targets": "node >4"}]))),
        (r#"require("array-find-index")"#, Some(serde_json::json!([{"targets": "node 4"}]))),
        (
            r#"require("mdn-polyfills/Array.prototype.findIndex")"#,
            Some(serde_json::json!([{"targets": "node 4"}])),
        ),
        (r#"require("weakmap-polyfill")"#, Some(serde_json::json!([{"targets": "node 12"}]))),
        (
            r#"require("typed-array-float64-array-polyfill")"#,
            Some(serde_json::json!([{"targets": "node 17"}])),
        ),
        (
            r#"import regexpEscape from "regexp.escape""#,
            Some(serde_json::json!([{"targets": {"node": "24"}}])),
        ),
        (
            r#"require("core-js/full/regexp/escape")"#,
            Some(serde_json::json!([{"targets": {"node": "24"}}])),
        ),
    ];

    Tester::new(NoUnnecessaryPolyfills::NAME, NoUnnecessaryPolyfills::PLUGIN, pass, fail)
        .test_and_snapshot();
}
