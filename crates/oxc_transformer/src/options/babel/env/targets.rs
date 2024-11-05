use oxc_diagnostics::Error;
use rustc_hash::FxHashMap;
use serde::Deserialize;

pub use browserslist::Version;

use super::query::Query;

/// A map of browser names to data for feature support in browser.
///
/// This type mainly stores `minimum version for each browsers with support for
/// a feature`.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize)]
#[serde(try_from = "BabelTargets")]
pub struct Targets {
    chrome: Option<Version>,
    deno: Option<Version>,
    edge: Option<Version>,
    firefox: Option<Version>,
    hermes: Option<Version>,
    ie: Option<Version>,
    ios: Option<Version>,
    node: Option<Version>,
    opera: Option<Version>,
    rhino: Option<Version>,
    safari: Option<Version>,
}

impl Targets {
    /// # Errors
    ///
    /// * Query is invalid.
    pub fn try_from_query(query: &str) -> Result<Self, Error> {
        Query::Single(query.to_string()).exec()
    }

    /// Returns true if all fields are [None].
    pub fn is_any_target(&self) -> bool {
        *self == Self::default()
    }

    pub fn should_enable(&self, targets: &Targets) -> bool {
        if let (Some(v1), Some(v2)) = (&self.chrome, &targets.chrome) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.deno, &targets.deno) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.edge, &targets.edge) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.firefox, &targets.firefox) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.hermes, &targets.hermes) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.ie, &targets.ie) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.ios, &targets.ios) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.node, &targets.node) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.opera, &targets.opera) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.rhino, &targets.rhino) {
            return v1 < v2;
        }
        if let (Some(v1), Some(v2)) = (&self.safari, &targets.safari) {
            return v1 < v2;
        }
        false
    }

    /// Parses the value returned from `browserslist`.
    pub fn parse_versions(versions: Vec<(String, String)>) -> Self {
        let mut targets = Self::default();
        for (name, version) in versions {
            let Ok(browser) = targets.get_version_mut(&name) else {
                continue;
            };
            let Ok(version) = version.parse::<Version>() else {
                continue;
            };
            if browser.is_none() || browser.is_some_and(|v| version < v) {
                browser.replace(version);
            }
        }
        targets
    }

    fn get_version_mut(&mut self, key: &str) -> Result<&mut Option<Version>, ()> {
        match key {
            "chrome" | "and_chr" => Ok(&mut self.chrome),
            "deno" => Ok(&mut self.deno),
            "edge" => Ok(&mut self.edge),
            "firefox" | "and_ff" => Ok(&mut self.firefox),
            "hermes" => Ok(&mut self.hermes),
            "ie" | "ie_mob" => Ok(&mut self.ie),
            "ios" | "ios_saf" => Ok(&mut self.ios),
            "node" => Ok(&mut self.node),
            "opera" | "op_mob" => Ok(&mut self.opera),
            "rhino" => Ok(&mut self.rhino),
            "safari" => Ok(&mut self.safari),
            _ => Err(()),
        }
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
            BabelTargets::String(s) => Query::Single(s).exec(),
            BabelTargets::Array(v) => Query::Multiple(v).exec(),
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
