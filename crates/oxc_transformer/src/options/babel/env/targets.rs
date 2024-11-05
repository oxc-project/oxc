use rustc_hash::FxHashMap;
use serde::Deserialize;

use oxc_diagnostics::Error;

pub use browserslist::Version;

use crate::options::{BrowserslistQuery, EngineTargets};

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
    type Error = Error;
    fn try_from(value: BabelTargets) -> Result<Self, Self::Error> {
        match value {
            BabelTargets::String(s) => BrowserslistQuery::Single(s).exec(),
            BabelTargets::Array(v) => BrowserslistQuery::Multiple(v).exec(),
            BabelTargets::Map(map) => {
                let mut targets = Self::default();
                for (key, value) in map {
                    // TODO: Implement these targets.
                    if matches!(key.as_str(), "esmodules" | "browsers") {
                        continue;
                    }
                    // TODO: Implement `Version::from_number`
                    if matches!(value, BabelTargetsValue::Int(_) | BabelTargetsValue::Float(_)) {
                        continue;
                    };
                    let BabelTargetsValue::String(v) = value else {
                        return Err(Error::msg(format!("{value:?} is not a string for {key}.")));
                    };
                    // TODO: Implement this target.
                    if key == "node" && v == "current" {
                        continue;
                    }
                    // TODO: Implement this target.
                    if key == "safari" && v == "tp" {
                        continue;
                    }
                    // TODO: Some keys are not implemented yet.
                    // <https://babel.dev/docs/options#targets>:
                    // Supported environments: android, chrome, deno, edge, electron, firefox, ie, ios, node, opera, rhino, safari, samsung.
                    let Ok(target) = targets.get_version_mut(&key) else {
                        continue;
                    };
                    match Version::parse(&v) {
                        Ok(version) => {
                            target.replace(version);
                        }
                        Err(err) => {
                            return Err(oxc_diagnostics::Error::msg(format!(
                                "Failed to parse `{v}` for `{key}`\n{err:?}"
                            )))
                        }
                    }
                }
                Ok(targets)
            }
        }
    }
}
