//! Base crate for `preset-env`-like crates.
//!
//! This crate provides an interface to convert `browserslist` query to
//! something usable.
//!
//! This file is copied from <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/preset_env_base/src/lib.rs>

use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;
use serde::Deserialize;

pub mod query;
pub mod version;
pub use query::Targets;
use version::Version;

/// A map of browser names to data for feature support in browser.
///
/// This type mainly stores `minimum version for each browsers with support for
/// a feature`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Versions(pub FxHashMap<String, Version>);

impl Deref for Versions {
    type Target = FxHashMap<String, Version>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Versions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Versions {
    /// Returns true if all fields are [None].
    pub fn is_any_target(&self) -> bool {
        self.0.is_empty()
    }

    /// Parses the value returned from `browserslist` as [Versions].
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

        let mut data: Versions = Versions::default();
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

        data
    }

    pub fn should_enable(&self, feature: &Versions) -> bool {
        self.iter().any(|(target_name, target_version)| {
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

#[cfg(test)]
mod tests {
    use crate::env::{targets::version::Version, Versions};

    #[test]
    fn should_enable_android_falls_back_to_chrome() {
        let mut targets = Versions::default();
        targets.insert("android".to_string(), "51.0.0".parse::<Version>().unwrap());
        let mut feature = Versions::default();
        feature.insert("chrome".to_string(), "51.0.0".parse::<Version>().unwrap());
        assert!(!targets.should_enable(&feature));
    }
}
