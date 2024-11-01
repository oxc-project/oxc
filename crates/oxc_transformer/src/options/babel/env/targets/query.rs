//! Module for `browserslist` queries.

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
        if let Some(v) = cache().get(self) {
            return Ok(v.clone());
        }

        let options = browserslist::Opts {
            mobile_to_desktop: true,
            ignore_unknown_versions: true,
            ..browserslist::Opts::default()
        };

        let result = match self {
            Query::Single(ref s) => {
                if s.is_empty() {
                    browserslist::resolve(&["defaults"], &options)
                } else {
                    browserslist::resolve(&[s], &options)
                }
            }
            Query::Multiple(ref s) => browserslist::resolve(s, &options),
        };

        let result = match result {
            Ok(distribs) => Targets::parse_versions(distribs),
            Err(err) => {
                return Err(OxcDiagnostic::error(format!("failed to resolve query: {err}")).into())
            }
        };

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
