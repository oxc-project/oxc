//! Module for `browserslist` queries.
//!
//! This file is copied from <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/preset_env_base/src/query.rs>

use std::sync::OnceLock;

use dashmap::DashMap;
use rustc_hash::FxHashMap;
use serde::Deserialize;

use oxc_diagnostics::{Error, OxcDiagnostic};

use super::{version::Version, Versions};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Targets {
    Query(Query),
    EsModules(EsModules),
    Versions(Versions),
    HashMap(FxHashMap<String, QueryOrVersion>),
}

impl Default for Targets {
    fn default() -> Self {
        Targets::Query(Query::Single("defaults".into()))
    }
}

impl Targets {
    /// Create a `Targets` from a browserslist query.
    ///
    /// The usage refer to the [browserslist](https://github.com/browserslist/browserslist?tab=readme-ov-file#queries) documentation.
    pub fn from_query(query: &str) -> Self {
        Targets::Query(Query::Single(query.into()))
    }

    /// Parse the query and return the parsed Versions.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// * The query is not supported.
    /// * The query is invalid.
    pub fn get_targets(self) -> Result<Versions, Error> {
        match self {
            Targets::Versions(v) => Ok(v),
            Targets::Query(q) => q.exec(),
            Targets::HashMap(mut map) => {
                let q = map.remove("browsers").map(|q| match q {
                    QueryOrVersion::Query(q) => q.exec(),
                    QueryOrVersion::Version(_) => unreachable!(),
                });

                let node = match map.remove("node") {
                    Some(QueryOrVersion::Version(v)) => Some(v),
                    Some(QueryOrVersion::Query(v)) => {
                        // We cannot get `current` node version
                        return Err(OxcDiagnostic::error(format!(
                            "Targets: node `{}` is not supported",
                            v.get_value()
                        ))
                        .into());
                    }
                    None => None,
                };

                if map.is_empty() {
                    if let Some(q) = q {
                        let mut q = q?;
                        if let Some(node) = node {
                            q.insert("node".to_string(), node);
                        }
                        return Ok(q);
                    }
                }

                let mut result = Versions::default();
                for (k, v) in &map {
                    match v {
                        QueryOrVersion::Query(q) => {
                            let v = q.exec()?;

                            for (k, v) in v.iter() {
                                result.insert(k.to_string(), *v);
                            }
                        }
                        QueryOrVersion::Version(v) => {
                            result.insert(k.to_string(), *v);
                        }
                    }
                }

                Err(OxcDiagnostic::error(format!("Targets: {result:?}")).into())
            }
            Targets::EsModules(_) => {
                Err(OxcDiagnostic::error("Targets: The `esmodules` is not supported").into())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EsModules {
    #[allow(dead_code)]
    esmodules: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum QueryOrVersion {
    Query(Query),
    Version(Version),
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum Query {
    Single(String),
    Multiple(Vec<String>),
}

type QueryResult = Result<Versions, Error>;

fn cache() -> &'static DashMap<Query, Versions> {
    static CACHE: OnceLock<DashMap<Query, Versions>> = OnceLock::new();
    CACHE.get_or_init(DashMap::new)
}

impl Query {
    fn get_value(&self) -> String {
        match self {
            Query::Single(s) => s.clone(),
            Query::Multiple(s) => s.join(","),
        }
    }

    fn exec(&self) -> QueryResult {
        fn query<T>(s: &[T]) -> QueryResult
        where
            T: AsRef<str>,
        {
            match browserslist::resolve(
                s,
                &browserslist::Opts {
                    mobile_to_desktop: true,
                    ignore_unknown_versions: true,
                    ..browserslist::Opts::default()
                },
            ) {
                Ok(distribs) => {
                    let versions = Versions::parse_versions(distribs);

                    Ok(versions)
                }
                Err(err) => {
                    let msg = format!("failed to resolve query: {err}");
                    Err(OxcDiagnostic::error(msg).into())
                }
            }
        }

        if let Some(v) = cache().get(self) {
            return Ok(v.clone());
        }

        let result = match *self {
            Query::Single(ref s) => {
                if s.is_empty() {
                    query(&["defaults"])
                } else {
                    query(&[s])
                }
            }
            Query::Multiple(ref s) => query(s),
        }?;

        cache().insert(self.clone(), result.clone());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn test_empty() {
        let res = Query::Single(String::new()).exec().unwrap();
        assert!(!res.is_any_target(), "empty query should return non-empty result");
    }
}
