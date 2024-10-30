//! Module for `browserslist` queries.
//!
//! This file is copied from <https://github.com/swc-project/swc/blob/ea14fc8e5996dcd736b8deb4cc99262d07dfff44/crates/preset_env_base/src/query.rs>

use std::sync::OnceLock;

use dashmap::DashMap;
use serde::Deserialize;

use oxc_diagnostics::{Error, OxcDiagnostic};

use super::Targets;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum Query {
    Single(String),
    Multiple(Vec<String>),
}

type QueryResult = Result<Targets, Error>;

fn cache() -> &'static DashMap<Query, Targets> {
    static CACHE: OnceLock<DashMap<Query, Targets>> = OnceLock::new();
    CACHE.get_or_init(DashMap::new)
}

impl Query {
    pub fn exec(&self) -> QueryResult {
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
                    let versions = Targets::parse_versions(distribs);

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
