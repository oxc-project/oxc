use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use browserslist::Version;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use super::{
    babel::BabelTargets,
    engine::Engine,
    es_features::{features, ESFeature},
    BrowserslistQuery,
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

    /// Returns true if all fields are [None].
    pub fn is_any_target(&self) -> bool {
        self.0.is_empty()
    }

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
}
