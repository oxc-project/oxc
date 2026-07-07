use std::sync::{Arc, LazyLock, Mutex};

use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNodes, NodeId};
use oxc_span::Span;

use crate::{
    config::BrowserslistTargetsConfig,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::compat::{
        BrowserTarget, COMPAT_DATA, FailingRule, RuleMap, RuleMaps, determine_targets_from_config,
        get_rules_for_targets, parse_browserslist_version,
    },
};

fn compat_diagnostic(error_name: &str, unsupported_targets: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{error_name} is not supported in {unsupported_targets}"))
        .with_label(span)
}

fn invalid_browserslist_diagnostic(error: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid browserslist configuration: {error}"))
        .with_label(Span::new(0, 0))
}

/// A single browserslist query, e.g. `"defaults, not ie < 9"`. Equivalent to
/// (and overridden by) the `browsers`/`targets` entries of the `compat`
/// settings.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(transparent)]
pub struct CompatConfig(Option<String>);

#[derive(Debug, Default, Clone)]
pub struct Compat(Box<CompatConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures cross-browser API compatibility: reports usage of browser APIs
    /// (and, optionally, ECMAScript APIs) that are not supported by the
    /// configured browser targets. This is a native port of
    /// [eslint-plugin-compat](https://github.com/amilajack/eslint-plugin-compat).
    ///
    /// Browser targets are configured with browserslist queries through the
    /// `compat` settings (`settings.compat.browsers` or
    /// `settings.compat.targets`) or through the rule's single string option.
    /// When no targets are configured, the browserslist `defaults` query is
    /// used.
    ///
    /// APIs guarded by a feature-detection conditional (e.g.
    /// `if (window.fetch) { fetch() }`) are not reported unless
    /// `settings.compat.ignoreConditionalChecks` is `true`. APIs listed in
    /// `settings.compat.polyfills` are not reported either; the special entry
    /// `"es:all"` excludes all ECMAScript APIs from linting (see
    /// `settings.compat.lintAllEsApis`).
    ///
    /// ### Why is this bad?
    ///
    /// Using an API that is not supported by a browser you target breaks your
    /// application for the users of that browser, typically with a runtime
    /// `ReferenceError` or `TypeError` that only shows up in production.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule
    /// (with `{ "settings": { "compat": { "browsers": ["ie 11"] } } }`):
    /// ```javascript
    /// fetch("/api");
    /// navigator.serviceWorker.register("/sw.js");
    /// new PaymentRequest(methodData, details, options);
    /// ```
    ///
    /// Examples of **correct** code for this rule
    /// (with `{ "settings": { "compat": { "browsers": ["ie 11"] } } }`):
    /// ```javascript
    /// if (window.fetch) {
    ///   fetch("/api");
    /// }
    /// document.querySelector("body");
    /// ```
    Compat,
    compat,
    suspicious,
    config = CompatConfig,
    version = "next",
    short_description = "Ensure cross-browser API compatibility.",
);

type ResolvedTargets = Result<Arc<[BrowserTarget]>, String>;

/// Cache of resolved browserslist targets, keyed by the joined queries.
static TARGETS_CACHE: LazyLock<Mutex<FxHashMap<String, ResolvedTargets>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

fn resolve_targets(queries: &[String]) -> ResolvedTargets {
    let key = queries.join("\u{0}");
    let mut cache = TARGETS_CACHE.lock().unwrap();
    cache
        .entry(key)
        .or_insert_with(|| {
            determine_targets_from_config(queries)
                .map(|targets| parse_browserslist_version(&targets).into())
                .map_err(|error| error.to_string())
        })
        .clone()
}

fn is_polyfilled(polyfills: &FxHashSet<&str>, rule: &FailingRule) -> bool {
    !polyfills.is_empty()
        && (polyfills.contains(rule.id.as_str())
            || polyfills.contains(rule.proto_chain_id.as_str())
            || polyfills.contains(rule.proto_chain_first.as_str()))
}

fn is_inside_if_statement(nodes: &AstNodes, node_id: NodeId) -> bool {
    nodes.ancestor_kinds(node_id).any(|kind| matches!(kind, AstKind::IfStatement(_)))
}

fn identifier_name<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    if let Expression::Identifier(ident) = expr { Some(ident.name.as_str()) } else { None }
}

/// Port of `protoChainFromMemberExpression` from eslint-plugin-compat's
/// `src/helpers.ts`. Nodes without a name produce an empty segment (mirroring
/// `undefined` in the reference), which never matches a rule.
fn push_proto_chain<'a>(expr: &Expression<'a>, chain: &mut Vec<&'a str>) {
    match expr {
        Expression::StaticMemberExpression(member) => {
            match &member.object {
                Expression::NewExpression(new_expr) => push_proto_chain(&new_expr.callee, chain),
                Expression::CallExpression(call) => push_proto_chain(&call.callee, chain),
                Expression::ArrayExpression(_) => chain.push("Array"),
                Expression::StringLiteral(_) => chain.push("String"),
                object => push_proto_chain(object, chain),
            }
            chain.push(member.property.name.as_str());
        }
        Expression::ComputedMemberExpression(member) => {
            match &member.object {
                Expression::NewExpression(new_expr) => push_proto_chain(&new_expr.callee, chain),
                Expression::CallExpression(call) => push_proto_chain(&call.callee, chain),
                Expression::ArrayExpression(_) => chain.push("Array"),
                Expression::StringLiteral(_) => chain.push("String"),
                object => push_proto_chain(object, chain),
            }
            chain.push("");
        }
        Expression::Identifier(ident) => chain.push(ident.name.as_str()),
        _ => chain.push(""),
    }
}

/// Secondary lookup for built-in `obj.prop` when the map was keyed with
/// different casing for `object` (e.g. `Crypto` in the metadata vs `crypto`
/// in the AST). The property name must still match the source exactly.
fn find_member_rule_by_global_object_casing<'m>(
    rules_map: &'m RuleMap,
    object_name: &str,
    property_name: &str,
) -> Option<&'m Arc<FailingRule>> {
    for (key, rule) in rules_map.iter() {
        if let Some((key_object, key_property)) = key.split_once('.')
            && key_object.eq_ignore_ascii_case(object_name)
            && key_property == property_name
        {
            return Some(rule);
        }
    }
    None
}

/// A deferred error: the name used for the local-binding filter (mirroring
/// `getName` in the reference), the failing rule, and the span to report.
type DeferredError<'a> = (Option<&'a str>, Arc<FailingRule>, Span);

struct CompatChecker<'a, 'r> {
    rule_maps: &'r RuleMaps,
    polyfills: FxHashSet<&'a str>,
    ignore_conditional_checks: bool,
    errors: Vec<DeferredError<'a>>,
}

impl<'a> CompatChecker<'a, '_> {
    fn handle_failing_rule(
        &mut self,
        filter_name: Option<&'a str>,
        rule: &Arc<FailingRule>,
        span: Span,
    ) {
        if is_polyfilled(&self.polyfills, rule) {
            return;
        }
        self.errors.push((filter_name, Arc::clone(rule), span));
    }

    fn check_guarded(
        &mut self,
        nodes: &AstNodes,
        node_id: NodeId,
        filter_name: Option<&'a str>,
        rule: &Arc<FailingRule>,
        span: Span,
    ) {
        if self.ignore_conditional_checks || !is_inside_if_statement(nodes, node_id) {
            self.handle_failing_rule(filter_name, rule, span);
        }
    }

    fn check_member_expression(
        &mut self,
        nodes: &AstNodes,
        node_id: NodeId,
        member: &oxc_ast::ast::StaticMemberExpression<'a>,
    ) {
        let object_name = identifier_name(&member.object);
        if let Some(object_name) =
            object_name.filter(|name| !matches!(*name, "window" | "globalThis"))
        {
            let property_name = member.property.name.as_str();
            let is_browser_global = COMPAT_DATA.browser_globals.contains(object_name);
            let mut failing_rule = self
                .rule_maps
                .member_expression
                .get(format!("{object_name}.{property_name}").as_str())
                .or_else(|| self.rule_maps.member_expression.get(object_name));
            if failing_rule.is_none() && is_browser_global {
                failing_rule = find_member_rule_by_global_object_casing(
                    &self.rule_maps.member_expression,
                    object_name,
                    property_name,
                );
            }
            if let Some(rule) = failing_rule
                && !is_browser_global
                && rule.object != object_name
            {
                failing_rule = None;
            }
            if let Some(rule) = failing_rule {
                let rule = Arc::clone(rule);
                self.check_guarded(nodes, node_id, Some(object_name), &rule, member.span);
            }
        } else {
            let mut chain = Vec::new();
            match &member.object {
                Expression::NewExpression(new_expr) => {
                    push_proto_chain(&new_expr.callee, &mut chain);
                }
                Expression::CallExpression(call) => push_proto_chain(&call.callee, &mut chain),
                Expression::ArrayExpression(_) => chain.push("Array"),
                Expression::StringLiteral(_) => chain.push("String"),
                object => push_proto_chain(object, &mut chain),
            }
            chain.push(member.property.name.as_str());
            let chain: &[&str] = if matches!(chain.first(), Some(&"window" | &"globalThis")) {
                &chain[1..]
            } else {
                &chain
            };
            let proto_chain_id = chain.join(".");
            if let Some(rule) = self.rule_maps.member_expression.get(&proto_chain_id) {
                let rule = Arc::clone(rule);
                self.check_guarded(nodes, node_id, object_name, &rule, member.span);
            }
        }
    }

    fn check_literal(&mut self, raw: &str, span: Span) {
        for (syntax, rule) in self.rule_maps.literal.iter() {
            if raw.contains(syntax) {
                let rule = Arc::clone(rule);
                // Mirrors the reference: literal checks are not suppressed by
                // feature-detection conditionals, and are filtered against a
                // local binding named `Literal` (the ESTree node type).
                self.handle_failing_rule(Some("Literal"), &rule, span);
                return;
            }
        }
    }
}

impl Rule for Compat {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<CompatConfig>>(value)?;
        Ok(Self(Box::new(config.into_inner())))
    }

    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().compat;

        let queries: Vec<String> = settings
            .browsers
            .as_ref()
            .or(settings.targets.as_ref())
            .map(BrowserslistTargetsConfig::to_queries)
            .or_else(|| self.0.0.clone().map(|query| vec![query]))
            .unwrap_or_default();

        let targets = match resolve_targets(&queries) {
            Ok(targets) => targets,
            Err(error) => {
                ctx.diagnostic(invalid_browserslist_diagnostic(&error));
                return;
            }
        };

        let polyfills: FxHashSet<&str> = settings.polyfills.iter().map(String::as_str).collect();
        let lint_all_es_apis = settings.lint_all_es_apis || !polyfills.contains("es:all");
        let rule_maps = get_rules_for_targets(&targets, lint_all_es_apis);

        let mut checker = CompatChecker {
            rule_maps: &rule_maps,
            polyfills,
            ignore_conditional_checks: settings.ignore_conditional_checks,
            errors: Vec::new(),
        };

        let nodes = ctx.nodes();
        for node in nodes.iter() {
            match node.kind() {
                AstKind::NewExpression(expr) => {
                    if let Some(name) = identifier_name(&expr.callee)
                        && let Some(rule) = checker.rule_maps.new_expression.get(name)
                    {
                        let rule = Arc::clone(rule);
                        checker.check_guarded(nodes, node.id(), Some(name), &rule, expr.span);
                    }
                }
                AstKind::CallExpression(expr) => {
                    if let Some(name) = identifier_name(&expr.callee)
                        && let Some(rule) = checker.rule_maps.call_expression.get(name)
                    {
                        let rule = Arc::clone(rule);
                        checker.check_guarded(nodes, node.id(), Some(name), &rule, expr.span);
                    }
                }
                AstKind::ExpressionStatement(stmt) => {
                    if let Some(name) = identifier_name(&stmt.expression)
                        && let Some(rule) = checker.rule_maps.expression_statement.get(name)
                    {
                        let rule = Arc::clone(rule);
                        checker.check_guarded(nodes, node.id(), Some(name), &rule, stmt.span);
                    }
                }
                AstKind::StaticMemberExpression(member) => {
                    checker.check_member_expression(nodes, node.id(), member);
                }
                AstKind::RegExpLiteral(lit) => {
                    checker.check_literal(ctx.source_range(lit.span), lit.span);
                }
                AstKind::StringLiteral(lit) => {
                    let is_regexp_argument = match nodes.parent_kind(node.id()) {
                        AstKind::NewExpression(parent) => {
                            identifier_name(&parent.callee) == Some("RegExp")
                        }
                        AstKind::CallExpression(parent) => {
                            identifier_name(&parent.callee) == Some("RegExp")
                        }
                        _ => false,
                    };
                    if is_regexp_argument {
                        checker.check_literal(ctx.source_range(lit.span), lit.span);
                    }
                }
                _ => {}
            }
        }

        // Do not report errors for locally-bound identifiers
        // (e.g. `import { Set } from 'immutable'; new Set();`).
        let scoping = ctx.scoping();
        let local_bindings: FxHashSet<&str> = scoping.symbol_names().collect();
        for (filter_name, rule, span) in checker.errors {
            if let Some(name) = filter_name
                && local_bindings.contains(name)
            {
                continue;
            }
            ctx.diagnostic(compat_diagnostic(&rule.error_name, &rule.unsupported_targets, span));
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    // Builds the per-case eslint config with `compat` settings, mirroring the
    // per-case `settings` of eslint-plugin-compat's test/e2e.spec.ts. The
    // reference RuleTester config sets `lintAllEsApis: true` for every case
    // (flat config merges settings), which is replicated here per case.
    #[expect(clippy::unnecessary_wraps)]
    fn browsers(list: &[&str]) -> Option<serde_json::Value> {
        Some(json!({ "settings": { "compat": { "browsers": list, "lintAllEsApis": true } } }))
    }
    #[expect(clippy::unnecessary_wraps, clippy::needless_pass_by_value)]
    fn settings(compat: serde_json::Value) -> Option<serde_json::Value> {
        Some(json!({ "settings": { "compat": compat } }))
    }

    // Port of the `valid` cases of test/e2e.spec.ts, in order.
    let pass = vec![
        // Ignore ES APIs if not supported by the (ExplorerMobile 10) MDN data
        ("Array.from()", None, browsers(&["ExplorerMobile 10"])),
        // Feature detection cases
        ("if (fetch) { fetch() }", None, browsers(&["ExplorerMobile 10"])),
        ("if (Array.prototype.flat) { new Array.flat() }", None, browsers(&["ExplorerMobile 10"])),
        ("if (fetch && otherConditions) { fetch() }", None, browsers(&["ExplorerMobile 10"])),
        ("if (window.fetch) { fetch() }", None, browsers(&["ExplorerMobile 10"])),
        ("if ('fetch' in window) { fetch() }", None, browsers(&["ExplorerMobile 10"])),
        ("window", None, browsers(&["ExplorerMobile 10"])),
        ("document.fonts()", None, browsers(&["edge 79"])),
        // Note: the reference uses `android 147`, a version that no longer
        // exists in oxc-browserslist's caniuse data; `last 1 android version`
        // preserves the intent (a current Android WebView).
        (
            "import * as serviceWorker from './serviceWorker'; serviceWorker.register(false);",
            None,
            browsers(&["chrome 52", "last 1 android version"]),
        ),
        (
            "navigator.permissions.query({ name: 'local-network-access' }).then((permissionStatus) => { permissionStatus.addEventListener('change', () => {}); });",
            None,
            browsers(&["chrome 52", "last 1 android version"]),
        ),
        (
            "const abortController = new AbortController(); abortController.abort();",
            None,
            browsers(&["chrome 70"]),
        ),
        (
            "const mutationObserver = new MutationObserver(() => {}); mutationObserver.observe(document.body, { childList: true });",
            None,
            browsers(&["chrome 52"]),
        ),
        (
            "const intersectionObserver = new IntersectionObserver(() => {}); intersectionObserver.observe(document.body);",
            None,
            browsers(&["chrome 70"]),
        ),
        (
            "const IntersectionObserver = 'test'; IntersectionObserver.trim();",
            None,
            browsers(&["chrome 30"]),
        ),
        // Import cases
        ("import { Set } from 'immutable'; new Set();", None, browsers(&["ie 9"])),
        ("const { Set } = require('immutable'); new Set();", None, browsers(&["ie 9"])),
        ("const { Set } = require('immutable'); new Set();", None, browsers(&["current node"])),
        (
            "const { Set } = require('immutable'); new Set();",
            None,
            browsers(&["ie 9", "current node"]),
        ),
        ("const Set = require('immutable').Set; new Set();", None, browsers(&["ie 9"])),
        ("Promise.resolve()", None, browsers(&["node 10"])),
        (
            "const { Set } = require('immutable'); (() => { new Set(); })();",
            None,
            browsers(&["ie 9"]),
        ),
        ("import Set from 'immutable'; new Set();", None, browsers(&["ie 9"])),
        ("function Set() {} new Set();", None, browsers(&["ie 9"])),
        ("const Set = () => {}; new Set();", None, browsers(&["ie 9"])),
        ("const bar = () => { const Set = () => {}; new Set(); }", None, browsers(&["ie 9"])),
        ("const bar = () => { class Set {} new Set() }", None, browsers(&["ie 9"])),
        ("const bar = () => { const Set = {}; new Set() }", None, browsers(&["ie 9"])),
        ("const bar = () => { function Set() {} new Set() }", None, browsers(&["ie 9"])),
        ("document.documentElement()", None, browsers(&["Safari 11", "Opera 57", "Edge 17"])),
        ("document.getElementsByTagName()", None, browsers(&["Safari 11", "Opera 57", "Edge 17"])),
        (
            "Promise.resolve('foo')",
            None,
            settings(
                json!({ "polyfills": ["Promise"], "browsers": ["ie 8"], "lintAllEsApis": true }),
            ),
        ),
        ("history.back()", None, browsers(&["Safari 11", "Opera 57", "Edge 17"])),
        // No configured targets: browserslist `defaults`.
        ("document.querySelector()", None, settings(json!({ "lintAllEsApis": true }))),
        ("new ServiceWorker()", None, browsers(&["chrome 57", "firefox 50"])),
        (
            "document.currentScript()",
            None,
            browsers(&["chrome 57", "firefox 50", "safari 10", "edge 14"]),
        ),
        ("document.querySelector()", None, browsers(&["ChromeAndroid 80"])),
        ("document.hasFocus()", None, browsers(&["Chrome 27"])),
        ("new URL()", None, browsers(&["ChromeAndroid 78", "ios 11"])),
        (
            "document.currentScript('some')",
            None,
            browsers(&["chrome 57", "firefox 50", "safari 10", "edge 14"]),
        ),
        (
            "WebAssembly.compile()",
            None,
            settings(json!({
                "browsers": ["chrome 40"],
                "polyfills": ["WebAssembly", "WebAssembly.compile"],
                "lintAllEsApis": true,
            })),
        ),
        ("new IntersectionObserver(() => {}, {});", None, browsers(&["chrome 58"])),
        ("new URL('http://example')", None, browsers(&["chrome 32", "safari 7.1", "firefox 26"])),
        ("new URLSearchParams()", None, browsers(&["chrome 49", "safari 10.1", "firefox 44"])),
        // Port of test/compat-lookup-behavior.spec.ts valid cases:
        // JS identifiers are case-sensitive; `new url()` is not the URL constructor.
        ("void new url();", None, browsers(&["ie 8"])),
        // document rules always carry a property; wrong case must not match
        // document.querySelector.
        ("void document.QuerySelector;", None, browsers(&["ie 8"])),
        // Port of the getRulesForTargets memoization regression
        // (test/compat-lookup-behavior.spec.ts): `polyfills: ["es:all"]` must
        // drop ES-kind rules even when the same targets were already linted
        // with ES rules included.
        (
            "Array.from([1, 2, 3]);",
            None,
            settings(json!({ "browsers": ["ie 8"], "polyfills": ["es:all"] })),
        ),
        // Rule option (a single browserslist query string), supported targets.
        ("fetch()", Some(json!(["chrome 70"])), settings(json!({ "lintAllEsApis": true }))),
    ];

    // Port of the `invalid` cases of test/e2e.spec.ts, in order. Expected
    // messages (asserted via the snapshot) follow the reference, e.g.
    // "fetch is not supported in IE Mobile 10".
    let fail = vec![
        (
            "if (fetch) { fetch() }",
            None,
            settings(json!({
                "browsers": ["ExplorerMobile 10"],
                "ignoreConditionalChecks": true,
                "lintAllEsApis": true,
            })),
        ),
        ("window?.fetch?.('example.com')", None, browsers(&["ie 9"])),
        (
            "navigator.hardwareConcurrency; navigator.serviceWorker; new SharedWorker();",
            None,
            browsers(&["ie 9"]),
        ),
        (
            "const event = new CustomEvent('cat', { detail: { hazcheeseburger: true } }); window.dispatchEvent(event);",
            None,
            browsers(&["ie 8"]),
        ),
        ("Array.from()", None, browsers(&["ie 8"])),
        (
            "Promise.allSettled()",
            None,
            browsers(&["Chrome >= 72", "Firefox >= 72", "Safari >= 12", "Edge >= 79"]),
        ),
        ("location.origin", None, browsers(&["ie 10"])),
        ("import { Map } from 'immutable'; new Set()", None, browsers(&["ie 9"])),
        ("new Set()", None, browsers(&["ie 9"])),
        ("new TypedArray()", None, browsers(&["ie 9"])),
        ("new Int8Array()", None, browsers(&["ie 9"])),
        ("new AnimationEvent", None, browsers(&["chrome 40"])),
        ("Object.values({})", None, browsers(&["safari 9"])),
        ("new ServiceWorker()", None, browsers(&["chrome 31"])),
        ("new IntersectionObserver(() => {}, {});", None, browsers(&["chrome 49"])),
        ("new PaymentRequest(methodData, details, options)", None, browsers(&["chrome 57"])),
        ("navigator.serviceWorker", None, browsers(&["safari 10.1"])),
        ("window.document.fonts()", None, browsers(&["ie 8"])),
        ("new Map().size", None, browsers(&["ie 8"])),
        ("new window.Map().size", None, browsers(&["ie 8"])),
        ("new Array().flat", None, browsers(&["ie 8"])),
        ("globalThis.fetch()", None, browsers(&["ie 11"])),
        ("fetch()", None, browsers(&["ie 11"])),
        ("Promise.resolve()", None, browsers(&["ie 10"])),
        ("Promise.all()", None, browsers(&["ie 10"])),
        ("Promise.race()", None, browsers(&["ie 10"])),
        ("Promise.reject()", None, browsers(&["ie 10"])),
        ("new URL('http://example')", None, browsers(&["chrome 31", "safari 7", "firefox 25"])),
        ("new URLSearchParams()", None, browsers(&["chrome 48", "safari 10", "firefox 28"])),
        ("performance.now()", None, browsers(&["ie 9"])),
        ("new ResizeObserver()", None, browsers(&["ie 11", "safari 12"])),
        ("'foo'.at(5)", None, browsers(&["ie 11", "safari 12"])),
        ("[].at(5)", None, browsers(&["ie 11", "safari 12"])),
        ("Object.entries({}), Object.values({})", None, browsers(&["Android >= 4", "iOS >= 7"])),
        ("window.requestIdleCallback(() => {})", None, browsers(&["safari 12"])),
        ("window.requestAnimationFrame(() => {})", None, browsers(&["OperaMini all"])),
        (
            "/(?<=y)x/, new RegExp('(?<!y)x'), 'x', true, false, null, 1, 0n",
            None,
            browsers(&["Safari >= 16.3", "iOS >= 16.3"]),
        ),
        ("crypto.randomUUID()", None, browsers(&["chrome 52", "safari 14"])),
        ("[].includes()", None, browsers(&["ie 11"])),
        ("'strsd'.includes()", None, browsers(&["ie 11"])),
        ("[1, 2, [3, 4]].flat()", None, browsers(&["ie 11"])),
        ("[1,2,3].flatMap(x => [x, x])", None, browsers(&["chrome 68"])),
        ("Object.fromEntries([])", None, browsers(&["chrome 72"])),
        ("'text'.replaceAll('x', 's')", None, browsers(&["chrome 84"])),
        ("navigator.serviceWorker.register('/service_worker.js');", None, browsers(&["chrome 39"])),
        (
            "const abortController = new AbortController(); abortController.abort();",
            None,
            browsers(&["chrome 65"]),
        ),
        (
            "const mutationObserver = new MutationObserver(() => {}); mutationObserver.observe(document.body, { childList: true });",
            None,
            browsers(&["chrome 25"]),
        ),
        (
            "const intersectionObserver = new IntersectionObserver(() => {}); intersectionObserver.observe(document.body);",
            None,
            browsers(&["chrome 50"]),
        ),
        (
            "navigator.permissions.query({ name: 'local-network-access' }).then((permissionStatus) => { permissionStatus.addEventListener('change', () => {}); });",
            None,
            browsers(&["chrome 41"]),
        ),
        // Port of test/compat-lookup-behavior.spec.ts invalid cases.
        ("void new URL();", None, browsers(&["ie 8"])),
        ("void document.querySelector;", None, browsers(&["ie 8"])),
        // Port of the getRulesForTargets memoization regression: without
        // `es:all`, ES APIs are linted (computed lintAllEsApis is true even
        // when the setting is unset).
        ("Array.from([1, 2, 3]);", None, settings(json!({ "browsers": ["ie 8"] }))),
        // Rule option (a single browserslist query string), unsupported targets.
        ("fetch()", Some(json!(["ie 11"])), settings(json!({ "lintAllEsApis": true }))),
        // Invalid browserslist configuration reports instead of panicking.
        ("fetch()", None, browsers(&["edge 100000"])),
    ];

    Tester::new(Compat::NAME, Compat::PLUGIN, pass, fail).test_and_snapshot();
}
