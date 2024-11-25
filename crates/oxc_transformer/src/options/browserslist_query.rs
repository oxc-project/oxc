use std::sync::OnceLock;

use dashmap::DashMap;
use serde::Deserialize;

use super::EngineTargets;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum BrowserslistQuery {
    Single(String),
    Multiple(Vec<String>),
}

fn cache() -> &'static DashMap<BrowserslistQuery, EngineTargets> {
    static CACHE: OnceLock<DashMap<BrowserslistQuery, EngineTargets>> = OnceLock::new();
    CACHE.get_or_init(DashMap::new)
}

impl BrowserslistQuery {
    pub fn exec(&self) -> Result<EngineTargets, String> {
        if let Some(v) = cache().get(self) {
            return Ok(v.clone());
        }

        let options = browserslist::Opts {
            mobile_to_desktop: true,
            ignore_unknown_versions: true,
            ..browserslist::Opts::default()
        };

        let result = match self {
            BrowserslistQuery::Single(ref s) => {
                if s.is_empty() {
                    browserslist::resolve(&["defaults"], &options)
                } else {
                    browserslist::resolve(&[s], &options)
                }
            }
            BrowserslistQuery::Multiple(ref s) => browserslist::resolve(s, &options),
        };

        let result = match result {
            Ok(distribs) => {
                let versions = distribs
                    .into_iter()
                    .map(|d| (d.name().to_string(), d.version().to_string()))
                    .collect::<Vec<_>>();
                EngineTargets::parse_versions(versions)
            }
            Err(err) => return Err(format!("failed to resolve query: {err}")),
        };

        cache().insert(self.clone(), result.clone());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserslistQuery;

    #[test]
    fn test_empty() {
        let res = BrowserslistQuery::Single(String::new()).exec().unwrap();
        assert!(!res.is_any_target(), "empty query should return non-empty result");
    }
}
