use std::path::{Path, PathBuf};

use oxc_config::GlobSet;

use crate::core::{oxfmtrc::AssociationsConfig, support::Language};

/// Resolved `associations` for file-to-language routing.
///
/// Consulted before built-in extension detection, so an explicit entry always wins.
/// Among entries, the later one wins, matching `overrides` semantics.
#[derive(Debug)]
pub struct Associations {
    base_dir: Option<PathBuf>,
    entries: Vec<(GlobSet, Language)>,
}

impl Associations {
    pub fn new(config: AssociationsConfig, base_dir: Option<PathBuf>) -> Self {
        Self {
            base_dir,
            entries: config
                .into_entries()
                .into_iter()
                .map(|(pattern, language)| (GlobSet::new([pattern]), language))
                .collect(),
        }
    }

    /// The language of the last entry whose pattern matches `path`, if any.
    pub fn language_for(&self, path: &Path) -> Option<Language> {
        let relative = super::relative_to_config_dir(self.base_dir.as_deref(), path);
        self.entries
            .iter()
            .rev()
            .find(|(glob, _)| glob.is_match(&relative))
            .map(|(_, language)| *language)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn associations(raw: serde_json::Value, base_dir: Option<&str>) -> Associations {
        let config: AssociationsConfig = serde_json::from_value(raw).unwrap();
        Associations::new(config, base_dir.map(PathBuf::from))
    }

    #[test]
    fn no_match_is_none() {
        let associations = associations(json!({ "*.wxml": "html" }), None);
        assert_eq!(associations.language_for(Path::new("src/a.html")), None);
    }

    #[test]
    fn bare_patterns_match_at_any_depth() {
        let associations = associations(json!({ "*.wxml": "html" }), None);
        assert_eq!(associations.language_for(Path::new("src/pages/a.wxml")), Some(Language::Html));
    }

    #[test]
    fn later_entry_wins() {
        let associations =
            associations(json!({ "**/*.html": "angular", "index.html": "html" }), None);
        assert_eq!(associations.language_for(Path::new("src/user.html")), Some(Language::Angular));
        assert_eq!(associations.language_for(Path::new("src/index.html")), Some(Language::Html));
    }

    #[test]
    fn relative_to_base_dir() {
        let associations = associations(json!({ "templates/*.html": "angular" }), Some("/repo"));
        assert_eq!(
            associations.language_for(Path::new("/repo/templates/a.html")),
            Some(Language::Angular)
        );
        assert_eq!(associations.language_for(Path::new("/repo/src/templates/a.html")), None);
    }
}
