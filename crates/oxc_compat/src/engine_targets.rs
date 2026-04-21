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
