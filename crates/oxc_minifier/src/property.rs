//! Property-name mangling engine.
//!
//! This module implements opt-in property-name mangling (`obj.longName` -> `obj.a`).
//! It is **off by default**: nothing is mangled unless the user supplies a `mangle`
//! regex via [`ManglePropertiesOptions`].
//!
//! This file currently contains the pure-logic foundation: the option/cache types,
//! the eligibility check, and the name-assignment function. The AST collect/rewrite
//! passes are added in later stages.

// The eligibility/assignment helpers are the foundation consumed by the collect/rewrite
// passes added in a later stage; they are currently exercised only by unit tests.
#![allow(dead_code)]

use oxc_mangler::base54;
use oxc_str::CompactStr;
use rustc_hash::{FxHashMap, FxHashSet};

/// Property names that are always reserved regardless of the user's regex.
///
/// These are well-known protocol / interop names whose mangling would break
/// reflection, JSON serialization, promises, or common host behavior.
const PROTOCOL_DENYLIST: &[&str] =
    &["then", "toJSON", "toString", "valueOf", "length", "name", "message"];

/// Options controlling property mangling.
///
/// Feature is **off** when `mangle` is `None`.
#[derive(Default)]
pub struct ManglePropertiesOptions {
    /// Names matching this regex are candidates for mangling. `None` => feature off.
    pub mangle: Option<lazy_regex::Regex>,
    /// Names matching this regex are reserved (never mangled), even if `mangle` matches.
    pub reserve: Option<lazy_regex::Regex>,
    /// Explicit reserved names. Added to (never replaces) the always-reserved set.
    pub reserved: FxHashSet<CompactStr>,
    /// Whether to mangle quoted keys. v1: always `false`.
    pub mangle_quoted: bool,
    /// Whether to emit human-readable debug names. v1: always `false` (deferred).
    pub debug: bool,
    /// Cross-build name cache (old -> new / reserved).
    pub cache: PropertyMangleCache,
}

/// Persistent old-name -> assigned-name cache, so repeated builds produce stable names.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct PropertyMangleCache {
    pub map: FxHashMap<CompactStr, CacheValue>,
}

/// A cached decision for a property name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheValue {
    /// The name was mangled to this new name.
    Name(CompactStr),
    /// The name is reserved and must never be mangled.
    Reserved,
}

/// Whether `name` is always reserved, regardless of the user's regex.
fn is_always_reserved(name: &str) -> bool {
    matches!(name, "__proto__" | "constructor" | "prototype") || PROTOCOL_DENYLIST.contains(&name)
}

/// Whether `name` is eligible for mangling under `opts`.
fn eligible(opts: &ManglePropertiesOptions, name: &str) -> bool {
    opts.mangle.as_ref().is_some_and(|re| re.is_match(name))
        && !opts.reserve.as_ref().is_some_and(|re| re.is_match(name))
        && !opts.reserved.contains(name)
        && !is_always_reserved(name)
}

/// Assign final mangled names.
///
/// `candidates` are eligible unquoted names; `reserved` are program-wide reservations.
/// Returns the old -> new map and mutates the shared `cache`.
///
/// The iteration order is deterministic (sorted) so that a shared cache reproduces
/// the same names across builds, and the produced names are pairwise disjoint and
/// never collide with reserved/always-reserved names.
fn assign(
    candidates: &FxHashSet<CompactStr>,
    reserved: &FxHashSet<CompactStr>,
    cache: &mut PropertyMangleCache,
) -> FxHashMap<CompactStr, CompactStr> {
    // Deterministic order so a shared cache is reproducible.
    let mut names: Vec<&CompactStr> = candidates.difference(reserved).collect();
    names.sort_unstable();
    // Seed `seeded` with existing cache targets so freshly-generated names never alias
    // a name that a (possibly later) cached candidate will reuse. This set is used only
    // to constrain newly generated names; reusing a candidate's own cached name is fine.
    let seeded: FxHashSet<CompactStr> = cache
        .map
        .values()
        .filter_map(|v| match v {
            CacheValue::Name(n) => Some(n.clone()),
            CacheValue::Reserved => None,
        })
        .collect();
    // Names actually claimed by an output during this build.
    let mut assigned: FxHashSet<CompactStr> = FxHashSet::default();
    let mut counter: u32 = 0;
    let mut map = FxHashMap::default();
    for name in names {
        match cache.map.get(name) {
            Some(CacheValue::Reserved) => {}
            Some(CacheValue::Name(n)) => {
                // Cache validation: never reuse a name that collides this build.
                if reserved.contains(n.as_str()) || assigned.contains(n) || is_always_reserved(n) {
                    continue; // safe-skip
                }
                map.insert(name.clone(), n.clone());
                assigned.insert(n.clone());
            }
            None => {
                let n = loop {
                    let c = CompactStr::from(base54(counter).as_str());
                    counter += 1;
                    if !seeded.contains(&c)
                        && !assigned.contains(&c)
                        && !reserved.contains(&c)
                        && !is_always_reserved(&c)
                    {
                        break c;
                    }
                };
                map.insert(name.clone(), n.clone());
                assigned.insert(n.clone());
                cache.map.insert(name.clone(), CacheValue::Name(n));
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_regex::Regex;

    fn opts(re: &str) -> ManglePropertiesOptions {
        ManglePropertiesOptions {
            mangle: Some(Regex::new(re).unwrap()),
            reserve: None,
            reserved: FxHashSet::default(),
            mangle_quoted: false,
            debug: false,
            cache: PropertyMangleCache::default(),
        }
    }

    #[test]
    fn eligibility() {
        let o = opts("^_");
        assert!(eligible(&o, "_foo"));
        assert!(!eligible(&o, "foo")); // no regex match
        assert!(!eligible(&o, "__proto__")); // always reserved
        assert!(eligible(&o, "_then")); // matches ^_, not in denylist => eligible
        // protocol denylist wins even if it matches:
        let o2 = opts(".");
        assert!(!eligible(&o2, "then"));
        assert!(!eligible(&o2, "toJSON"));
    }

    #[test]
    fn assignment_is_deterministic_and_disjoint() {
        let cands: FxHashSet<CompactStr> =
            ["_a", "_b"].iter().map(|s| CompactStr::from(*s)).collect();
        let reserved = FxHashSet::default();
        let mut cache = PropertyMangleCache::default();
        let m1 = assign(&cands, &reserved, &mut cache);
        let m2 = assign(&cands, &reserved, &mut PropertyMangleCache::default());
        assert_eq!(m1, m2); // deterministic
        let names: FxHashSet<_> = m1.values().collect();
        assert_eq!(names.len(), m1.len()); // no two map to the same name
    }

    #[test]
    fn cache_reuse_and_reserved() {
        let cands: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("_a")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("_a".into(), CacheValue::Name("Z".into()));
        let m = assign(&cands, &FxHashSet::default(), &mut cache);
        assert_eq!(m[&CompactStr::from("_a")].as_str(), "Z"); // honors cache
    }

    #[test]
    fn cache_collision_is_skipped_not_corrupted() {
        let cands: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("_a")).collect();
        let reserved: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("b")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("_a".into(), CacheValue::Name("b".into())); // collides with reserved `b`
        let m = assign(&cands, &reserved, &mut cache);
        assert!(!m.contains_key(&CompactStr::from("_a"))); // skipped, not mapped to `b`
    }
}
