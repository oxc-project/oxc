//! Rule-map construction for the `compat/compat` rule.
//!
//! Port of the provider node lists (`src/providers/*.ts`) and
//! `getRulesForTargets` (`src/rules/compat.ts`) from eslint-plugin-compat:
//! for a resolved set of browser targets, precompute the set of APIs ("rules")
//! that fail at least one target, keyed for O(1) lookup per AST node.

use std::{
    hash::BuildHasherDefault,
    sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use rustc_hash::{FxHashMap, FxHasher};

use super::{
    data::COMPAT_DATA,
    support::{caniuse_unsupported_targets, mdn_unsupported_targets},
    targets::BrowserTarget,
};

/// An API that fails at least one of the configured targets.
#[derive(Debug)]
pub struct FailingRule {
    /// The rule id, used for polyfill matching (e.g. `"navigator.serviceWorker"`).
    pub id: String,
    /// The full proto chain id (e.g. `"navigator.serviceWorker"`).
    pub proto_chain_id: String,
    /// The first element of the proto chain (e.g. `"navigator"`), used for
    /// whole-API polyfill matching.
    pub proto_chain_first: String,
    /// The object part of the rule (e.g. `"navigator"`).
    pub object: String,
    /// The name shown in the error message, e.g. `"navigator.serviceWorker()"`,
    /// `"URL"`, `"Lookbehind"`.
    pub error_name: String,
    /// The formatted, comma-joined list of unsupported targets, e.g.
    /// `"Safari 10.1, IE 11"`.
    pub unsupported_targets: String,
}

/// An insertion-ordered rule map with O(1) lookup. Iteration order matters for
/// first-match-wins insertion and for the case-insensitive browser-global
/// fallback lookup of member expressions.
#[derive(Debug, Default)]
pub struct RuleMap {
    entries: Vec<(String, Arc<FailingRule>)>,
    index: FxHashMap<String, usize>,
}

impl RuleMap {
    fn insert_if_absent(&mut self, key: &str, rule: &Arc<FailingRule>) {
        if !self.index.contains_key(key) {
            self.index.insert(key.to_string(), self.entries.len());
            self.entries.push((key.to_string(), Arc::clone(rule)));
        }
    }

    pub fn get(&self, key: &str) -> Option<&Arc<FailingRule>> {
        self.index.get(key).map(|&i| &self.entries[i].1)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Arc<FailingRule>)> {
        self.entries.iter().map(|(key, rule)| (key.as_str(), rule))
    }
}

/// The per-node-type rule maps for a resolved set of targets.
#[derive(Debug, Default)]
pub struct RuleMaps {
    /// Keyed by callee name, e.g. `fetch()`.
    pub call_expression: RuleMap,
    /// Keyed by callee name, e.g. `new URL()`.
    pub new_expression: RuleMap,
    /// Keyed by expression name, e.g. a bare `fetch;` statement.
    pub expression_statement: RuleMap,
    /// Keyed by proto chain id and `object.property`.
    pub member_expression: RuleMap,
    /// Secondary index of `member_expression` for the case-insensitive
    /// browser-global fallback: keyed by `lowercased(object).property` for
    /// every dotted key, first match (in `member_expression` insertion order)
    /// wins. Replaces the reference's linear scan
    /// (`findMemberRuleByGlobalObjectCasing`) with an O(1) lookup.
    pub member_expression_by_lower_object: FxHashMap<String, Arc<FailingRule>>,
    /// Keyed by literal syntax fragment, e.g. `?<=` (RegExp lookbehind).
    pub literal: RuleMap,
}

impl RuleMaps {
    /// Returns `true` when no API fails any configured target, in which case
    /// the rule has nothing to lint for this file.
    pub fn is_empty(&self) -> bool {
        // `expression_statement` and the lowercase index are projections of
        // the other maps and cannot be non-empty on their own.
        self.call_expression.is_empty()
            && self.new_expression.is_empty()
            && self.member_expression.is_empty()
            && self.literal.is_empty()
    }
}

/// One entry of the CanIUse provider list.
struct CaniuseEntry {
    caniuse_id: &'static str,
    ast_node_type: char,
    object: &'static str,
    property: Option<&'static str>,
    name: Option<&'static str>,
    syntaxes: &'static [&'static str],
}

const fn caniuse_entry(
    caniuse_id: &'static str,
    ast_node_type: char,
    object: &'static str,
    property: Option<&'static str>,
) -> CaniuseEntry {
    CaniuseEntry { caniuse_id, ast_node_type, object, property, name: None, syntaxes: &[] }
}

/// Port of the `CanIUseProvider` node list from
/// `src/providers/caniuse-provider.ts`.
static CANIUSE_PROVIDER: &[CaniuseEntry] = &[
    caniuse_entry("serviceworkers", 'N', "ServiceWorker", None),
    caniuse_entry("serviceworkers", 'M', "navigator", Some("serviceWorker")),
    caniuse_entry("queryselector", 'M', "document", Some("querySelector")),
    caniuse_entry("intersectionobserver", 'N', "IntersectionObserver", None),
    caniuse_entry("resizeobserver", 'N', "ResizeObserver", None),
    caniuse_entry("payment-request", 'N', "PaymentRequest", None),
    caniuse_entry("promises", 'N', "Promise", None),
    caniuse_entry("promises", 'M', "Promise", Some("resolve")),
    caniuse_entry("promises", 'M', "Promise", Some("all")),
    caniuse_entry("promises", 'M', "Promise", Some("race")),
    caniuse_entry("promises", 'M', "Promise", Some("reject")),
    caniuse_entry("fetch", 'C', "fetch", None),
    caniuse_entry("document-currentscript", 'M', "document", Some("currentScript")),
    caniuse_entry("url", 'N', "URL", None),
    caniuse_entry("urlsearchparams", 'N', "URLSearchParams", None),
    caniuse_entry("high-resolution-time", 'M', "performance", Some("now")),
    caniuse_entry("requestidlecallback", 'C', "requestIdleCallback", None),
    caniuse_entry("requestanimationframe", 'C', "requestAnimationFrame", None),
    caniuse_entry("typedarrays", 'N', "TypedArray", None),
    caniuse_entry("typedarrays", 'N', "Int8Array", None),
    caniuse_entry("typedarrays", 'N', "Uint8Array", None),
    caniuse_entry("typedarrays", 'N', "Uint8ClampedArray", None),
    caniuse_entry("typedarrays", 'N', "Int16Array", None),
    caniuse_entry("typedarrays", 'N', "Uint16Array", None),
    caniuse_entry("typedarrays", 'N', "Int32Array", None),
    caniuse_entry("typedarrays", 'N', "Uint32Array", None),
    caniuse_entry("typedarrays", 'N', "Float32Array", None),
    caniuse_entry("typedarrays", 'N', "Float64Array", None),
    CaniuseEntry {
        caniuse_id: "js-regexp-lookbehind",
        ast_node_type: 'L',
        object: "RegExp",
        property: None,
        name: Some("Lookbehind"),
        syntaxes: &["?<=", "?<!"],
    },
];

/// Buckets of failing rules per AST node type, in provider order
/// (CanIUse provider first, then the MDN provider).
#[derive(Default)]
struct FailingRulesByType {
    call_expression: Vec<Arc<FailingRule>>,
    new_expression: Vec<Arc<FailingRule>>,
    member_expression: Vec<Arc<FailingRule>>,
    expression_statement: Vec<Arc<FailingRule>>,
    literal: Vec<(Arc<FailingRule>, &'static [&'static str])>,
}

fn collect_failing_rules(targets: &[BrowserTarget], lint_all_es_apis: bool) -> FailingRulesByType {
    let mut by_type = FailingRulesByType::default();

    for entry in CANIUSE_PROVIDER {
        let unsupported = caniuse_unsupported_targets(entry.caniuse_id, targets);
        if unsupported.is_empty() {
            continue;
        }
        let id = entry.property.map_or_else(
            || entry.object.to_string(),
            |property| format!("{}.{property}", entry.object),
        );
        let error_name = entry.name.map_or_else(
            || {
                entry.property.map_or_else(
                    || entry.object.to_string(),
                    |property| format!("{}.{property}()", entry.object),
                )
            },
            ToString::to_string,
        );
        let rule = Arc::new(FailingRule {
            proto_chain_id: id.clone(),
            proto_chain_first: entry.object.to_string(),
            object: entry.object.to_string(),
            error_name,
            unsupported_targets: unsupported.join(", "),
            id,
        });
        match entry.ast_node_type {
            'C' => by_type.call_expression.push(rule),
            'N' => by_type.new_expression.push(rule),
            'M' => by_type.member_expression.push(rule),
            'E' => by_type.expression_statement.push(rule),
            'L' => by_type.literal.push((rule, entry.syntaxes)),
            _ => unreachable!(),
        }
    }

    for api in &COMPAT_DATA.mdn {
        if !lint_all_es_apis && api.is_es {
            continue;
        }
        let unsupported = mdn_unsupported_targets(api, targets);
        if unsupported.is_empty() {
            continue;
        }
        let proto_chain: Vec<&str> = api.proto_chain_id.split('.').collect();
        let error_name = if proto_chain.len() == 1 {
            proto_chain[0].to_string()
        } else {
            format!("{}()", api.proto_chain_id)
        };
        let unsupported_targets = unsupported.join(", ");
        for ast_node_type in api.ast_node_types.chars() {
            let rule = Arc::new(FailingRule {
                id: api.proto_chain_id.to_string(),
                proto_chain_id: api.proto_chain_id.to_string(),
                proto_chain_first: proto_chain[0].to_string(),
                object: proto_chain[0].to_string(),
                error_name: error_name.clone(),
                unsupported_targets: unsupported_targets.clone(),
            });
            match ast_node_type {
                'C' => by_type.call_expression.push(rule),
                'N' => by_type.new_expression.push(rule),
                'M' => by_type.member_expression.push(rule),
                'E' => by_type.expression_statement.push(rule),
                'L' => {}
                _ => unreachable!(),
            }
        }
    }

    by_type
}

/// The property part of a rule's proto chain (e.g. `serviceWorker` in
/// `navigator.serviceWorker`), mirroring `rule.property` in the reference.
fn rule_property(rule: &FailingRule) -> Option<&str> {
    let mut parts = rule.proto_chain_id.split('.');
    parts.next()?;
    parts.next()
}

fn build_rule_maps(targets: &[BrowserTarget], lint_all_es_apis: bool) -> RuleMaps {
    let by_type = collect_failing_rules(targets, lint_all_es_apis);
    let mut maps = RuleMaps::default();

    for rule in &by_type.call_expression {
        maps.call_expression.insert_if_absent(&rule.object, rule);
    }
    for rule in &by_type.new_expression {
        maps.new_expression.insert_if_absent(&rule.object, rule);
    }
    for rule in by_type.member_expression.iter().chain(&by_type.call_expression) {
        maps.expression_statement.insert_if_absent(&rule.object, rule);
    }
    for rule in by_type
        .member_expression
        .iter()
        .chain(&by_type.call_expression)
        .chain(&by_type.new_expression)
    {
        maps.member_expression.insert_if_absent(&rule.proto_chain_id, rule);
        let key = rule_property(rule)
            .map_or_else(|| rule.object.clone(), |property| format!("{}.{property}", rule.object));
        maps.member_expression.insert_if_absent(&key, rule);
    }
    for (rule, syntaxes) in &by_type.literal {
        for syntax in *syntaxes {
            maps.literal.insert_if_absent(syntax, rule);
        }
    }

    // Build the case-insensitive index in `member_expression` insertion order
    // so that the first matching entry wins, exactly like the reference's
    // linear scan.
    for (key, rule) in maps.member_expression.iter() {
        if let Some((object, property)) = key.split_once('.') {
            let lower_key = format!("{}.{property}", object.cow_to_ascii_lowercase());
            maps.member_expression_by_lower_object
                .entry(lower_key)
                .or_insert_with(|| Arc::clone(rule));
        }
    }

    maps
}

/// Cache of rule maps, keyed by the resolved targets and the ES-API-inclusion
/// flag (mirroring `getRulesForTargets`' memoization, including the
/// regression fix that keys on both arguments). A lock-free map is used so
/// that files linted in parallel do not contend once the cache is warm.
type RuleMapsCache = papaya::HashMap<String, Arc<RuleMaps>, BuildHasherDefault<FxHasher>>;
static RULE_MAPS_CACHE: LazyLock<RuleMapsCache> = LazyLock::new(RuleMapsCache::default);

/// Get (or compute and cache) the rule maps for a set of resolved targets.
pub fn get_rules_for_targets(targets: &[BrowserTarget], lint_all_es_apis: bool) -> Arc<RuleMaps> {
    // The cache key encodes both the resolved targets and the
    // ES-API-inclusion flag (the `getRulesForTargets` memoization regression).
    let mut key = String::with_capacity(targets.len() * 12 + 1);
    key.push(if lint_all_es_apis { '1' } else { '0' });
    for target in targets {
        key.push(',');
        key.push_str(&target.target);
        key.push(' ');
        key.push_str(&target.version);
    }
    let cache = RULE_MAPS_CACHE.pin();
    if let Some(maps) = cache.get(key.as_str()) {
        return Arc::clone(maps);
    }
    // Build outside of any lock; a rare concurrent build of the same key is
    // cheaper than serializing all files behind a mutex during the build.
    let maps = Arc::new(build_rule_maps(targets, lint_all_es_apis));
    Arc::clone(cache.get_or_insert(key, maps))
}
