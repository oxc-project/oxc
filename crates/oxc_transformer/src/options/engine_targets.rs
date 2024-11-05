use browserslist::Version;
use serde::Deserialize;

use oxc_diagnostics::Error;

use super::babel::{BabelTargets, Query};

/// A map of engine names to minimum supported versions.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize)]
#[serde(try_from = "BabelTargets")]
pub struct EngineTargets {
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

impl EngineTargets {
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

    pub fn should_enable(&self, targets: &EngineTargets) -> bool {
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

    pub(crate) fn get_version_mut(&mut self, key: &str) -> Result<&mut Option<Version>, ()> {
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
