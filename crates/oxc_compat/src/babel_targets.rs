use std::str::FromStr;

pub use browserslist::Version;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{BrowserslistQuery, Engine, EngineTargets};

/// <https://babel.dev/docs/babel-preset-env#targets>
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BabelTargets {
    String(String),
    Array(Vec<String>),
    /// For Deserializing
    /// * `esmodules`: `boolean`
    /// * `node`: `string | "current" | true`
    /// * `safari`: `string | "tp"`
    /// * `browsers`: `string | Array<string>.`
    /// * `deno`: `string`
    Map(FxHashMap<String, BabelTargetsValue>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BabelTargetsValue {
    String(String),
    Array(Vec<String>),
    Bool(bool),
    Int(u32),
    Float(f64),
}

impl TryFrom<BabelTargets> for EngineTargets {
    type Error = String;

    fn try_from(value: BabelTargets) -> Result<Self, Self::Error> {
        match value {
            BabelTargets::String(s) => BrowserslistQuery::Single(s).exec(),
            BabelTargets::Array(v) => BrowserslistQuery::Multiple(v).exec(),
            BabelTargets::Map(map) => {
                let mut engine_targets = Self::default();
                for (key, value) in map {
                    // Explicitly disabling ES module filtering leaves the targets unchanged.
                    if key == "esmodules" && matches!(value, BabelTargetsValue::Bool(false)) {
                        continue;
                    }
                    if matches!(key.as_str(), "esmodules" | "browsers") {
                        return Err(format!("Babel target `{key}` is not supported"));
                    }
                    if matches!(value, BabelTargetsValue::Int(_) | BabelTargetsValue::Float(_)) {
                        return Err(format!("numeric Babel target `{key}` is not supported"));
                    }
                    let BabelTargetsValue::String(v) = value else {
                        return Err(format!("{value:?} is not a string for {key}."));
                    };
                    if key == "node" && v == "current" {
                        return Err("Babel target `node=current` is not supported".into());
                    }
                    if key == "safari" && v == "tp" {
                        return Err("Babel target `safari=tp` is not supported".into());
                    }
                    // TODO: Some keys are not implemented yet.
                    // <https://babel.dev/docs/options#targets>:
                    // Supported environments: android, chrome, deno, edge, electron, firefox, ie, ios, node, opera, rhino, safari, samsung.
                    let Ok(engine) = Engine::from_str(&key) else {
                        return Err(format!("engine '{key}' is not supported."));
                    };
                    // Version::parse ignores any components after the patch version.
                    if v.split('.').count() > 3 {
                        return Err(format!(
                            "Failed to parse `{v}` for `{key}`: too many version components"
                        ));
                    }
                    match Version::parse(&v) {
                        Ok(version) => {
                            engine_targets.insert(engine, version);
                        }
                        Err(err) => {
                            return Err(format!("Failed to parse `{v}` for `{key}`\n{err:?}"));
                        }
                    }
                }
                Ok(engine_targets)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_babel_targets_are_not_silently_dropped() {
        let cases = [
            ("esmodules", BabelTargetsValue::Bool(true)),
            ("browsers", BabelTargetsValue::Array(vec!["defaults".into()])),
            ("chrome", BabelTargetsValue::Int(120)),
            ("firefox", BabelTargetsValue::Float(120.5)),
            ("node", BabelTargetsValue::String("current".into())),
            ("safari", BabelTargetsValue::String("tp".into())),
        ];
        for (key, value) in cases {
            let targets = BabelTargets::Map(FxHashMap::from_iter([
                ("chrome".into(), BabelTargetsValue::String("100".into())),
                (key.into(), value),
            ]));
            assert!(EngineTargets::try_from(targets).is_err(), "{key} was silently ignored");
        }
    }

    #[test]
    fn malformed_versions_are_not_silently_dropped() {
        for version in ["latest", "", "120.x", "120.1.2.3", "120.1.2.invalid", "65536"] {
            let targets = BabelTargets::Map(FxHashMap::from_iter([
                ("firefox".into(), BabelTargetsValue::String("100".into())),
                ("chrome".into(), BabelTargetsValue::String(version.into())),
            ]));
            let error = EngineTargets::try_from(targets).unwrap_err();
            assert!(error.contains(&format!("`{version}` for `chrome`")), "{error}");
        }
    }

    #[test]
    fn esmodules_false_preserves_targets() {
        let targets = BabelTargets::Map(FxHashMap::from_iter([
            ("esmodules".into(), BabelTargetsValue::Bool(false)),
            ("chrome".into(), BabelTargetsValue::String("49".into())),
            ("node".into(), BabelTargetsValue::String("18.2.1".into())),
        ]));
        let targets = EngineTargets::try_from(targets).unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets.get(&Engine::Chrome), Some(&Version(49, 0, 0)));
        assert_eq!(targets.get(&Engine::Node), Some(&Version(18, 2, 1)));

        let targets = BabelTargets::Map(FxHashMap::from_iter([(
            "esmodules".into(),
            BabelTargetsValue::Bool(false),
        )]));
        assert!(EngineTargets::try_from(targets).unwrap().is_any_target());
    }
}
