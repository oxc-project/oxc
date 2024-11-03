//! Base crate for `preset-env`-like crates.
//!
//! This crate provides an interface to convert `browserslist` query to
//! something usable.
//!
//! This file is copied from <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/preset_env_base/src/lib.rs>

use std::ops::Deref;

use oxc_diagnostics::Error;
use rustc_hash::FxHashMap;
use serde::Deserialize;

pub use browserslist::Version;

use super::query::Query;

/// A map of browser names to data for feature support in browser.
///
/// This type mainly stores `minimum version for each browsers with support for
/// a feature`.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(try_from = "BabelTargets")] // https://github.com/serde-rs/serde/issues/642#issuecomment-683276351
pub struct Targets(FxHashMap<String, Version>);

impl Deref for Targets {
    type Target = FxHashMap<String, Version>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Targets {
    pub fn new(map: FxHashMap<String, Version>) -> Self {
        Self(map)
    }

    /// # Errors
    ///
    /// * Query is invalid.
    pub fn try_from_query(query: &str) -> Result<Self, Error> {
        Query::Single(query.to_string()).exec().map(|v| v.0).map(Self)
    }

    /// Returns true if all fields are [None].
    pub fn is_any_target(&self) -> bool {
        self.0.is_empty()
    }

    /// Parses the value returned from `browserslist`.
    pub fn parse_versions(distribs: Vec<browserslist::Distrib>) -> Self {
        fn remap(key: &str) -> &str {
            match key {
                "and_chr" => "chrome",
                "and_ff" => "firefox",
                "ie_mob" => "ie",
                "ios_saf" => "ios",
                "op_mob" => "opera",
                _ => key,
            }
        }

        let mut data = FxHashMap::default();
        for dist in distribs {
            let browser = dist.name();
            let browser = remap(browser);
            let version = dist.version();
            match browser {
                "and_qq" | "and_uc" | "baidu" | "bb" | "kaios" | "op_mini" => continue,

                _ => {}
            }

            let version =
                version.split_once('-').map_or(version, |(version, _)| version).parse::<Version>();

            let Ok(version) = version else { continue };

            // lowest version
            let is_lowest = data.get(browser).map_or(true, |v| v > &version);
            if is_lowest {
                data.insert(browser.to_string(), version);
            }
        }

        Self(data)
    }

    pub fn should_enable(&self, feature: &Targets) -> bool {
        self.0.iter().any(|(target_name, target_version)| {
            feature
                .get(target_name)
                .or_else(|| match target_name.as_str() {
                    // Fall back to Chrome versions if Android browser data
                    // is missing from the feature data. It appears the
                    // Android browser has aligned its versioning with Chrome.
                    "android" => feature.get("chrome"),
                    _ => None,
                })
                .map_or(false, |feature_version| feature_version > target_version)
        })
    }
}

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

impl TryFrom<BabelTargets> for Targets {
    type Error = Error;
    fn try_from(value: BabelTargets) -> Result<Self, Self::Error> {
        match value {
            BabelTargets::String(s) => Query::Single(s).exec().map(|v| v.0).map(Self),
            BabelTargets::Array(v) => Query::Multiple(v).exec().map(|v| v.0).map(Self),
            BabelTargets::Map(map) => {
                let mut new_map = FxHashMap::default();
                for (k, v) in map {
                    // TODO: Implement these targets.
                    if matches!(k.as_str(), "esmodules" | "node" | "safari" | "browsers" | "deno") {
                        continue;
                    }
                    // TODO: Implement `Version::from_number`
                    if matches!(v, BabelTargetsValue::Int(_) | BabelTargetsValue::Float(_)) {
                        continue;
                    };
                    let BabelTargetsValue::String(v) = v else {
                        return Err(Error::msg(format!("{v:?} is not a string for {k}.")));
                    };
                    match Version::parse(&v) {
                        Ok(v) => {
                            new_map.insert(k, v);
                        }
                        Err(err) => {
                            return Err(oxc_diagnostics::Error::msg(format!(
                                "Failed to parse `{v}` for `{k}`\n{err:?}"
                            )))
                        }
                    }
                }
                Ok(Self(new_map))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Targets, Version};

    #[test]
    fn should_enable_android_falls_back_to_chrome() {
        let mut targets = Targets::default();
        targets.0.insert("android".to_string(), "51.0.0".parse::<Version>().unwrap());
        let mut feature = Targets::default();
        feature.0.insert("chrome".to_string(), "51.0.0".parse::<Version>().unwrap());
        assert!(!targets.should_enable(&feature));
    }
}
