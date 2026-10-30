use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    str::FromStr,
};

pub use browserslist::Version;
use oxc_syntax::es_target::ESTarget;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::browserslist_query::BrowserslistQuery;
use crate::{babel_targets::BabelTargets, es_target::ESVersion};

use super::{
    Engine,
    es_features::{ESFeature, features},
};

/// A map of engine names to minimum supported versions.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "BabelTargets")]
pub struct EngineTargets(FxHashMap<Engine, Version>);

impl Deref for EngineTargets {
    type Target = FxHashMap<Engine, Version>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EngineTargets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for EngineTargets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, (engine, version)) in self.iter().enumerate() {
            if idx > 0 {
                f.write_str(",")?;
            }
            f.write_str(&engine.to_string())?;
            if *engine == Engine::Es {
                f.write_str(&version.0.to_string())?;
            } else {
                f.write_str(&version.to_string())?;
            }
        }
        Ok(())
    }
}

impl EngineTargets {
    pub fn new(map: FxHashMap<Engine, Version>) -> Self {
        Self(map)
    }

    /// # Errors
    ///
    /// * Query is invalid.
    pub fn try_from_query(query: &str) -> Result<Self, String> {
        BrowserslistQuery::Single(query.to_string()).exec()
    }

    /// Returns true if all fields are empty.
    pub fn is_any_target(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if the target engines support the given ES feature.
    ///
    /// Returns `true` if the feature is NOT supported (needs transformation),
    /// `false` if the feature IS supported (can be used natively).
    /// Use [`Self::supports_es_feature`] for a strict positive capability query.
    pub fn has_feature(&self, feature: ESFeature) -> bool {
        let feature_engine_targets = &features()[&feature];
        for (engine, feature_version) in feature_engine_targets.iter() {
            if let Some(target_version) = self.get(engine) {
                if *engine == Engine::Es {
                    return target_version.0 < feature_version.0;
                }
                if target_version < feature_version {
                    return true;
                }
            }
        }
        false
    }

    /// Precompute the answer of [`Self::has_feature`] for every feature in the
    /// compatibility table.
    ///
    /// [`Self::has_feature`] performs a hash lookup and an engine-table walk per call; callers
    /// that query features repeatedly (e.g. the minifier, per AST node) can compute this cache
    /// once and answer each query with a single bit test via [`FeatureSupportCache::has_feature`].
    pub fn feature_support_cache(&self) -> FeatureSupportCache {
        let mut unsupported = 0u128;
        for &feature in features().keys() {
            if self.has_feature(feature) {
                unsupported |= FeatureSupportCache::bit(feature);
            }
        }
        FeatureSupportCache { unsupported }
    }

    /// Check whether every target engine is known to support the given ES feature.
    ///
    /// Unlike [`Self::has_feature`], this is a strict capability query: an empty target or an
    /// engine missing from the compatibility table is treated as unsupported.
    pub fn supports_es_feature(&self, feature: ESFeature) -> bool {
        if self.is_any_target() {
            return false;
        }

        let feature_engine_targets = &features()[&feature];
        self.iter().all(|(engine, target_version)| {
            feature_engine_targets.get(engine).is_some_and(|feature_version| {
                if *engine == Engine::Es {
                    target_version.0 >= feature_version.0
                } else {
                    target_version >= feature_version
                }
            })
        })
    }

    /// Parses the value returned from `browserslist`.
    pub fn parse_versions(versions: Vec<(String, String)>) -> Self {
        let mut engine_targets = Self::default();
        for (engine, version) in versions {
            let Ok(engine) = Engine::from_str(&engine) else {
                continue;
            };
            let Ok(version) = Version::from_str(&version) else {
                continue;
            };
            engine_targets
                .0
                .entry(engine)
                .and_modify(|v| {
                    if version < *v {
                        *v = version;
                    }
                })
                .or_insert(version);
        }
        engine_targets
    }

    /// # Errors
    ///
    /// * When the query failed to parse.
    pub fn from_target(s: &str) -> Result<Self, String> {
        if s.contains(',') {
            Self::from_target_list(&s.split(',').collect::<Vec<_>>())
        } else {
            Self::from_target_list(&[s])
        }
    }

    /// # Errors
    ///
    /// * When the query failed to parse.
    pub fn from_target_list<S: AsRef<str>>(list: &[S]) -> Result<Self, String> {
        let mut es_target = None;
        let mut engine_targets = EngineTargets::default();

        for s in list {
            let s = s.as_ref();
            // Parse `esXXXX`.
            if let Ok(target) = ESTarget::from_str(s) {
                if let Some(target) = es_target {
                    return Err(format!("'{target}' is already specified."));
                }
                es_target = Some(target);
            } else {
                // Parse `chromeXX`, `edgeXX` etc.
                let (engine, version) = Engine::parse_name_and_version(s)?;
                if engine_targets.insert(engine, version).is_some() {
                    return Err(format!("'{s}' is already specified."));
                }
            }
        }
        engine_targets.insert(Engine::Es, es_target.unwrap_or(ESTarget::default()).version());
        Ok(engine_targets)
    }
}

/// Precomputed answers to [`EngineTargets::has_feature`], built by
/// [`EngineTargets::feature_support_cache`].
///
/// Internally a bitset over [`ESFeature`] discriminants. The representation is private so the
/// discriminant-to-bit mapping never crosses a crate boundary; `test_feature_bits_fit` pins the
/// invariant that every table feature fits in the bitset.
#[derive(Debug, Clone, Copy)]
pub struct FeatureSupportCache {
    unsupported: u128,
}

impl FeatureSupportCache {
    fn bit(feature: ESFeature) -> u128 {
        let bit = feature as u32;
        // At call sites `feature` is a literal variant, so this check constant-folds away.
        // A feature that would not fit is a codegen-time error caught by `test_feature_bits_fit`;
        // panicking here (rather than masking the shift) keeps release builds from silently
        // querying the wrong bit if the generated enum ever outgrows the bitset.
        assert!(bit < 128, "ESFeature discriminant must fit in the feature bitset");
        1u128 << bit
    }

    /// Same answer as [`EngineTargets::has_feature`] on the targets this cache was built from:
    /// `true` if the feature is NOT supported by all target engines (needs transformation).
    pub fn has_feature(self, feature: ESFeature) -> bool {
        self.unsupported & Self::bit(feature) != 0
    }
}

/// Pin [`FeatureSupportCache`]'s invariant: every feature in the (generated) compatibility
/// table maps to a distinct bit inside the bitset. Fails loudly if `ESFeature` outgrows it.
#[test]
fn test_feature_bits_fit() {
    for &feature in features().keys() {
        assert!(
            (feature as u32) < 128,
            "ESFeature::{feature:?} discriminant does not fit in FeatureSupportCache"
        );
    }
}

/// The cache must agree with [`EngineTargets::has_feature`] for every feature, for any target.
#[test]
fn test_feature_support_cache_matches_has_feature() {
    for target in [
        EngineTargets::default(),
        EngineTargets::from_target("es2015").unwrap(),
        EngineTargets::from_target("chrome90").unwrap(),
    ] {
        let cache = target.feature_support_cache();
        for &feature in features().keys() {
            assert_eq!(
                cache.has_feature(feature),
                target.has_feature(feature),
                "cache disagrees with has_feature for {feature:?} on target {target}"
            );
        }
    }
}

#[test]
fn test_displayed_value_is_parsable() {
    let target = EngineTargets::new(FxHashMap::from_iter([
        (Engine::Chrome, Version(139, 0, 0)),
        (Engine::Deno, Version(2, 5, 1)),
        (Engine::Es, Version(2024, 0, 0)),
    ]));
    let s = target.to_string();
    let parsed = EngineTargets::from_target(&s).unwrap();
    assert_eq!(target.0, parsed.0);
}

#[test]
fn test_supports_es_feature() {
    use crate::ESFeature::{
        ES2015ArrowFunctions, ES2015RegExpConstructorCanAlterFlags, ES2020OptionalChaining,
    };

    assert!(!EngineTargets::default().supports_es_feature(ES2015ArrowFunctions));

    assert!(
        EngineTargets::from_target("es2015").unwrap().supports_es_feature(ES2015ArrowFunctions)
    );
    assert!(
        EngineTargets::from_target("esnext").unwrap().supports_es_feature(ES2020OptionalChaining)
    );
    assert!(
        !EngineTargets::from_target("es2019").unwrap().supports_es_feature(ES2020OptionalChaining)
    );
    assert!(
        EngineTargets::from_target("es2020").unwrap().supports_es_feature(ES2020OptionalChaining)
    );

    // Browser targets also contain an implicit `esnext`. All configured engines must support the
    // feature, regardless of compatibility-table iteration order.
    assert!(
        !EngineTargets::from_target("chrome90")
            .unwrap()
            .supports_es_feature(ES2020OptionalChaining)
    );
    assert!(
        EngineTargets::from_target("chrome91").unwrap().supports_es_feature(ES2020OptionalChaining)
    );

    assert!(
        !EngineTargets::from_target("chrome48")
            .unwrap()
            .supports_es_feature(ES2015RegExpConstructorCanAlterFlags)
    );
    assert!(
        EngineTargets::from_target("chrome49")
            .unwrap()
            .supports_es_feature(ES2015RegExpConstructorCanAlterFlags)
    );

    // Missing compatibility data is not proof of support.
    let rhino = EngineTargets::new(FxHashMap::from_iter([(Engine::Rhino, Version(1, 7, 15))]));
    assert!(!rhino.supports_es_feature(ES2020OptionalChaining));
}
