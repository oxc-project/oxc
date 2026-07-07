use std::{
    cell::RefCell,
    hash::BuildHasherDefault,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use rustc_hash::{FxHashSet, FxHasher};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNodes, IsGlobalReference, NodeId};
use oxc_span::Span;

use crate::{
    AstNode,
    config::BrowserslistTargetsConfig,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::compat::{
        BrowserTarget, COMPAT_DATA, FailingRule, RuleMaps, determine_targets_from_config,
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
    /// [eslint-plugin-compat](https://github.com/amilajack/eslint-plugin-compat),
    /// backed by the same data sources ([MDN browser-compat-data](https://github.com/mdn/browser-compat-data)
    /// via `ast-metadata-inferer`, and [caniuse](https://caniuse.com)).
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
    /// // Feature-detection guards are recognized:
    /// if (window.fetch) {
    ///   fetch("/api");
    /// }
    /// // Supported by every configured target:
    /// document.querySelector("body");
    /// // Locally-bound identifiers are not the global API:
    /// import { Set } from "immutable";
    /// new Set();
    /// ```
    ///
    /// Shadowing is scope-aware: a local binding only suppresses reports for
    /// references that actually resolve to it (unlike eslint-plugin-compat,
    /// where any binding anywhere in the file suppresses all same-named
    /// reports file-wide).
    ///
    /// ### Settings
    ///
    /// The rule is configured through the `compat` namespace of `settings`
    /// (this differs from eslint-plugin-compat, which reads top-level
    /// settings keys):
    ///
    /// ```json
    /// {
    ///   "plugins": ["compat"],
    ///   "rules": { "compat/compat": "error" },
    ///   "settings": {
    ///     "compat": {
    ///       "browsers": ["defaults", "not ie < 11"],
    ///       "polyfills": ["Promise", "WebAssembly.compile", "fetch"],
    ///       "lintAllEsApis": false,
    ///       "ignoreConditionalChecks": false
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// #### `browsers` / `targets`
    ///
    /// The browserslist queries to lint against: a single query string
    /// (`"defaults, not ie < 9"`), a list of queries (`["chrome 70",
    /// "firefox 60"]`), or an object with `production`/`development` query
    /// lists (their union is linted). `browsers` takes precedence over
    /// `targets`. Alternatively, a single query string can be passed as the
    /// rule's option: `"compat/compat": ["error", "defaults, not ie < 9"]`
    /// (settings take precedence over the option).
    ///
    /// When no targets are configured at all, the browserslist `defaults`
    /// query is used. Note: unlike eslint-plugin-compat, browserslist
    /// configuration is **not** discovered from `package.json`
    /// (`"browserslist"` field) or `.browserslistrc` files; declare your
    /// targets in the oxlint settings shown above.
    ///
    /// #### `polyfills`
    ///
    /// APIs that are polyfilled and must not be reported. Entries can name a
    /// whole API (`"Promise"`, `"fetch"`), a static member
    /// (`"WebAssembly.compile"`, `"Promise.all"`), or an instance member
    /// (`"Array.push"`, `"String.at"` — written without `.prototype.`,
    /// matching the compat data's naming). The special entry `"es:all"`
    /// excludes all ECMAScript APIs from linting (for codebases transpiled
    /// with Babel/core-js or similar).
    ///
    /// #### `lintAllEsApis`
    ///
    /// Lint ECMAScript APIs (e.g. `Array.from`, `Object.values`) in addition
    /// to web platform APIs, even when `polyfills` contains `"es:all"`.
    ///
    /// #### `ignoreConditionalChecks`
    ///
    /// By default, API usage inside an `if` statement is treated as
    /// feature-detected and is not reported (e.g.
    /// `if ('fetch' in window) { fetch() }`). Set this to `true` to report
    /// incompatible APIs even inside conditionals.
    Compat,
    compat,
    suspicious,
    config = CompatConfig,
    version = "next",
    short_description = "Ensure cross-browser API compatibility.",
);

type ResolvedTargets = Result<Arc<[BrowserTarget]>, String>;

/// Cache of resolved browserslist targets, keyed by the joined queries. A
/// lock-free map is used so that files linted in parallel do not contend once
/// the cache is warm.
static TARGETS_CACHE: LazyLock<
    papaya::HashMap<String, ResolvedTargets, BuildHasherDefault<FxHasher>>,
> = LazyLock::new(papaya::HashMap::default);

fn resolve_targets(queries: &[String]) -> ResolvedTargets {
    let key = queries.join("\u{0}");
    let cache = TARGETS_CACHE.pin();
    if let Some(targets) = cache.get(key.as_str()) {
        return targets.clone();
    }
    // Resolve outside of any lock; a rare concurrent resolution of the same
    // key is cheaper than serializing all files behind a mutex.
    let targets = determine_targets_from_config(queries)
        .map(|targets| parse_browserslist_version(&targets).into())
        .map_err(|error| error.to_string());
    cache.get_or_insert(key, targets).clone()
}

fn is_polyfilled(polyfills: &FxHashSet<String>, rule: &FailingRule) -> bool {
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
fn push_proto_chain<'a>(expr: &Expression<'a>, chain: &mut SmallVec<[&'a str; 8]>) {
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

/// Per-file lint state: the (globally cached) rule maps for the resolved
/// targets plus the per-config polyfill set and flags. Stored in a
/// thread-local, tagged by source-text and rule-instance pointers: each file
/// is processed sequentially on one thread, so all `run` invocations after
/// `Program` hit the cached state.
struct FileState {
    maps: Arc<RuleMaps>,
    polyfills: FxHashSet<String>,
    ignore_conditional_checks: bool,
    /// Browserslist resolution error, reported once on the `Program` node.
    error: Option<String>,
}

thread_local! {
    static FILE_STATE: RefCell<Option<(usize, usize, Rc<FileState>)>> = const { RefCell::new(None) };
}

impl Compat {
    fn file_state(&self, ctx: &LintContext) -> Rc<FileState> {
        let tag = (ctx.source_text().as_ptr() as usize, std::ptr::from_ref(self) as usize);
        FILE_STATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if let Some((source, rule, state)) = slot.as_ref()
                && (*source, *rule) == tag
            {
                return Rc::clone(state);
            }
            let state = Rc::new(self.compute_file_state(ctx));
            *slot = Some((tag.0, tag.1, Rc::clone(&state)));
            state
        })
    }

    fn compute_file_state(&self, ctx: &LintContext) -> FileState {
        let settings = &ctx.settings().compat;
        let queries: Vec<String> = settings
            .browsers
            .as_ref()
            .or(settings.targets.as_ref())
            .map(BrowserslistTargetsConfig::to_queries)
            .or_else(|| self.0.0.clone().map(|query| vec![query]))
            .unwrap_or_default();

        let polyfills: FxHashSet<String> = settings.polyfills.iter().cloned().collect();
        let ignore_conditional_checks = settings.ignore_conditional_checks;

        match resolve_targets(&queries) {
            Ok(targets) => {
                let lint_all_es_apis = settings.lint_all_es_apis || !polyfills.contains("es:all");
                FileState {
                    maps: get_rules_for_targets(&targets, lint_all_es_apis),
                    polyfills,
                    ignore_conditional_checks,
                    error: None,
                }
            }
            Err(error) => FileState {
                maps: Arc::new(RuleMaps::default()),
                polyfills,
                ignore_conditional_checks,
                error: Some(error),
            },
        }
    }
}

fn check_guarded(
    state: &FileState,
    ctx: &LintContext,
    node_id: NodeId,
    rule: &FailingRule,
    span: Span,
) {
    // Cheap set lookups first; the ancestor walk only runs for candidates
    // that are not polyfilled.
    if is_polyfilled(&state.polyfills, rule) {
        return;
    }
    if state.ignore_conditional_checks || !is_inside_if_statement(ctx.nodes(), node_id) {
        ctx.diagnostic(compat_diagnostic(&rule.error_name, &rule.unsupported_targets, span));
    }
}

fn check_literal(state: &FileState, ctx: &LintContext, raw: &str, span: Span) {
    for (syntax, rule) in state.maps.literal.iter() {
        if raw.contains(syntax) {
            // Mirrors the reference: literal checks are not suppressed by
            // feature-detection conditionals.
            if !is_polyfilled(&state.polyfills, rule) {
                ctx.diagnostic(compat_diagnostic(
                    &rule.error_name,
                    &rule.unsupported_targets,
                    span,
                ));
            }
            return;
        }
    }
}

fn check_member_expression<'a>(
    compat: &Compat,
    ctx: &LintContext<'a>,
    node_id: NodeId,
    member: &oxc_ast::ast::StaticMemberExpression<'a>,
) {
    if let Expression::Identifier(object) = &member.object
        && !matches!(object.name.as_str(), "window" | "globalThis")
    {
        // Locally-bound objects can never be the global API. The reference
        // implementation reports and then filters these against the file's
        // bindings at `Program:exit`; resolving the reference up front is
        // equivalent for in-scope bindings (and scope-aware), and lets the
        // common case (`localVar.prop`) bail before any map lookup.
        if !object.is_global_reference(ctx.scoping()) {
            return;
        }
        let state = compat.file_state(ctx);
        let maps = &state.maps;
        if maps.member_expression.is_empty() {
            return;
        }
        let object_name = object.name.as_str();
        let property_name = member.property.name.as_str();
        let mut failing_rule = maps.get_member_rule(object_name, property_name);
        if failing_rule.is_none() {
            // Case-insensitive fallback for browser globals (`crypto` in the
            // AST vs `Crypto` in the metadata); O(1) via the precomputed
            // lowercase index. `cow_to_ascii_lowercase` does not allocate for
            // already-lowercase names.
            if !COMPAT_DATA.browser_globals.contains(object_name) {
                return;
            }
            failing_rule = maps
                .member_expression_by_lower_object
                .get(object_name.cow_to_ascii_lowercase().as_ref())
                .and_then(|rules| rules.get(property_name));
        } else if !COMPAT_DATA.browser_globals.contains(object_name)
            && failing_rule.is_some_and(|rule| rule.object != object_name)
        {
            // For non-global objects the rule must match the object name
            // exactly (e.g. a local `intersectionObserver` variable must not
            // match the `IntersectionObserver` rule).
            failing_rule = None;
        }
        if let Some(rule) = failing_rule {
            check_guarded(&state, ctx, node_id, rule, member.span);
        }
    } else {
        // `window.x` / `globalThis.x` / chained or computed objects: build
        // the proto chain (mirroring `protoChainFromMemberExpression`).
        if let Expression::Identifier(object) = &member.object
            && !object.is_global_reference(ctx.scoping())
        {
            // A local binding named `window`/`globalThis`; the reference
            // filters these reports against the file's bindings.
            return;
        }
        let state = compat.file_state(ctx);
        if state.maps.member_expression.is_empty() {
            return;
        }
        let mut chain: SmallVec<[&str; 8]> = SmallVec::new();
        match &member.object {
            Expression::NewExpression(new_expr) => push_proto_chain(&new_expr.callee, &mut chain),
            Expression::CallExpression(call) => push_proto_chain(&call.callee, &mut chain),
            Expression::ArrayExpression(_) => chain.push("Array"),
            Expression::StringLiteral(_) => chain.push("String"),
            object => push_proto_chain(object, &mut chain),
        }
        chain.push(member.property.name.as_str());
        let segments: &[&str] = if matches!(chain.first(), Some(&"window" | &"globalThis")) {
            &chain[1..]
        } else {
            &chain
        };
        let [object_name, rest @ ..] = segments else { return };
        // Cheap first-segment rejection before joining the rest of the chain.
        if !state.maps.member_expression.contains_key(*object_name) {
            return;
        }
        let rest = rest.join(".");
        if let Some(rule) = state.maps.get_member_rule(object_name, &rest) {
            check_guarded(&state, ctx, node_id, rule, member.span);
        }
    }
}

impl Rule for Compat {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<CompatConfig>>(value)?;
        Ok(Self(Box::new(config.into_inner())))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(_) => {
                // Warm the per-file state and surface browserslist
                // configuration errors once per file.
                let state = self.file_state(ctx);
                if let Some(error) = &state.error {
                    ctx.diagnostic(invalid_browserslist_diagnostic(error));
                }
            }
            AstKind::NewExpression(expr) => {
                if let Expression::Identifier(ident) = &expr.callee
                    && ident.is_global_reference(ctx.scoping())
                {
                    let state = self.file_state(ctx);
                    if let Some(rule) = state.maps.new_expression.get(ident.name.as_str()) {
                        check_guarded(&state, ctx, node.id(), rule, expr.span);
                    }
                }
            }
            AstKind::CallExpression(expr) => {
                if let Expression::Identifier(ident) = &expr.callee
                    && ident.is_global_reference(ctx.scoping())
                {
                    let state = self.file_state(ctx);
                    if let Some(rule) = state.maps.call_expression.get(ident.name.as_str()) {
                        check_guarded(&state, ctx, node.id(), rule, expr.span);
                    }
                }
            }
            AstKind::ExpressionStatement(stmt) => {
                if let Expression::Identifier(ident) = &stmt.expression
                    && ident.is_global_reference(ctx.scoping())
                {
                    let state = self.file_state(ctx);
                    if let Some(rule) = state.maps.expression_statement.get(ident.name.as_str()) {
                        check_guarded(&state, ctx, node.id(), rule, stmt.span);
                    }
                }
            }
            AstKind::StaticMemberExpression(member) => {
                check_member_expression(self, ctx, node.id(), member);
            }
            AstKind::RegExpLiteral(lit) => {
                let state = self.file_state(ctx);
                if !state.maps.literal.is_empty() {
                    check_literal(&state, ctx, ctx.source_range(lit.span), lit.span);
                }
            }
            AstKind::StringLiteral(lit) => {
                // Only strings passed to `RegExp(...)` / `new RegExp(...)`
                // are candidates; check the parent before touching any state.
                let is_regexp_argument = match ctx.nodes().parent_kind(node.id()) {
                    AstKind::NewExpression(parent) => {
                        identifier_name(&parent.callee) == Some("RegExp")
                    }
                    AstKind::CallExpression(parent) => {
                        identifier_name(&parent.callee) == Some("RegExp")
                    }
                    _ => false,
                };
                if is_regexp_argument {
                    let state = self.file_state(ctx);
                    if !state.maps.literal.is_empty() {
                        check_literal(&state, ctx, ctx.source_range(lit.span), lit.span);
                    }
                }
            }
            _ => {}
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
