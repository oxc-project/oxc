//! Routing contributed by user-configured Prettier plugins.
//!
//! A plugin declares the languages it handles; the JS side loads the plugins and
//! reports those declarations, and this module turns them into the extension and
//! filename lookups that [`super::classify_file_kind`] consults.
//!
//! Delegating to Prettier at all requires the `napi` feature, so the pure Rust
//! build compiles this but never routes through it.
#![cfg_attr(not(feature = "napi"), expect(dead_code))]

use std::{
    cmp::Reverse,
    sync::{Mutex, OnceLock},
};

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

/// One entry of a plugin's `languages` declaration, as reported by the JS side.
#[derive(Debug, Deserialize)]
pub struct PluginLanguage {
    parsers: Vec<String>,
    /// Bare, without the leading dot that Prettier stores.
    extensions: Vec<String>,
    filenames: Vec<String>,
}

/// A plugin that could not be loaded, reported so the caller can warn once.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginFailure {
    pub specifier: String,
    pub message: String,
}

/// What the JS `initExternalServices` callback returns.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPlugins {
    pub languages: Vec<PluginLanguage>,
    pub failures: Vec<PluginFailure>,
}

/// The plugins named by a config, ready to be sent to the JS side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRequest {
    /// Directory the specifiers resolve from, which is the config file's own
    /// directory. Prettier resolves its `plugins` entries the same way.
    pub base: String,
    pub specifiers: Vec<String>,
}

/// Extension and filename routing contributed by the loaded plugins.
#[derive(Debug, Default)]
pub struct PluginLanguages {
    extensions: FxHashMap<String, &'static str>,
    /// Extensions spanning more than one dot-segment, such as `blade.php`.
    /// `Path::extension` only ever yields the last segment, so these are matched
    /// against the whole file name. Kept longest-first so the most specific wins.
    compound_extensions: Vec<(String, &'static str)>,
    filenames: FxHashMap<String, &'static str>,
}

impl PluginLanguages {
    /// The first plugin to claim an extension keeps it, matching the order the
    /// user listed them in.
    pub fn new(languages: Vec<PluginLanguage>) -> Self {
        let mut this = Self::default();
        for language in languages {
            let Some(parser) = language.parsers.first() else { continue };
            let parser = intern(parser);
            for extension in language.extensions {
                if extension.contains('.') {
                    let suffix = format!(".{extension}");
                    if !this.compound_extensions.iter().any(|(known, _)| *known == suffix) {
                        this.compound_extensions.push((suffix, parser));
                    }
                } else {
                    this.extensions.entry(extension).or_insert(parser);
                }
            }
            for file_name in language.filenames {
                this.filenames.entry(file_name).or_insert(parser);
            }
        }
        this.compound_extensions.sort_by_key(|(suffix, _)| Reverse(suffix.len()));
        this
    }

    /// The parser a plugin declared for this file, if any.
    pub fn parser_for(&self, file_name: &str, extension: Option<&str>) -> Option<&'static str> {
        if let Some(parser) = self.filenames.get(file_name) {
            return Some(parser);
        }
        if let Some((_, parser)) =
            self.compound_extensions.iter().find(|(suffix, _)| file_name.ends_with(suffix.as_str()))
        {
            return Some(parser);
        }
        self.extensions.get(extension?).copied()
    }
}

/// Parser names arrive as owned strings but `FileKind` carries
/// `&'static str`, so they are interned once and kept for the process.
///
/// The set is bounded by the user's config, and LSP re-initialization reuses
/// entries rather than leaking a new one per rebuild.
fn intern(name: &str) -> &'static str {
    static INTERNED: OnceLock<Mutex<FxHashSet<&'static str>>> = OnceLock::new();

    let mut interned = INTERNED.get_or_init(|| Mutex::new(FxHashSet::default())).lock().unwrap();
    if let Some(existing) = interned.get(name) {
        return existing;
    }
    let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
    interned.insert(leaked);
    leaked
}

#[cfg(test)]
mod test {
    use super::*;

    fn language(parser: &str, extensions: &[&str], filenames: &[&str]) -> PluginLanguage {
        PluginLanguage {
            parsers: vec![parser.to_string()],
            extensions: extensions.iter().map(|e| (*e).to_string()).collect(),
            filenames: filenames.iter().map(|f| (*f).to_string()).collect(),
        }
    }

    #[test]
    fn routes_extensions_and_filenames() {
        let languages =
            PluginLanguages::new(vec![language("ember-template-tag", &["gjs", "gts"], &[])]);

        assert_eq!(languages.parser_for("a.gjs", Some("gjs")), Some("ember-template-tag"));
        assert_eq!(languages.parser_for("a.gts", Some("gts")), Some("ember-template-tag"));
        assert_eq!(languages.parser_for("a.ts", Some("ts")), None);
        assert_eq!(languages.parser_for("a", None), None);
    }

    #[test]
    fn filename_wins_over_extension() {
        let languages = PluginLanguages::new(vec![
            language("by-extension", &["conf"], &[]),
            language("by-filename", &[], &["special.conf"]),
        ]);

        assert_eq!(languages.parser_for("special.conf", Some("conf")), Some("by-filename"));
        assert_eq!(languages.parser_for("other.conf", Some("conf")), Some("by-extension"));
    }

    #[test]
    fn first_plugin_to_claim_an_extension_keeps_it() {
        let languages = PluginLanguages::new(vec![
            language("first", &["x"], &[]),
            language("second", &["x"], &[]),
        ]);

        assert_eq!(languages.parser_for("a.x", Some("x")), Some("first"));
    }

    #[test]
    fn interning_is_stable() {
        assert_eq!(intern("some-parser").as_ptr(), intern("some-parser").as_ptr());
    }

    #[test]
    fn matches_an_extension_spanning_several_dot_segments() {
        let languages = PluginLanguages::new(vec![language("blade", &["blade.php"], &[])]);

        // `Path::extension` yields only `php` here, so the whole name must be matched.
        assert_eq!(languages.parser_for("index.blade.php", Some("php")), Some("blade"));
        assert_eq!(languages.parser_for("index.php", Some("php")), None);
    }

    #[test]
    fn the_most_specific_compound_extension_wins() {
        let languages = PluginLanguages::new(vec![
            language("short", &["b.php"], &[]),
            language("long", &["a.b.php"], &[]),
        ]);

        assert_eq!(languages.parser_for("x.a.b.php", Some("php")), Some("long"));
        assert_eq!(languages.parser_for("x.c.b.php", Some("php")), Some("short"));
    }

    #[test]
    fn a_language_without_a_parser_routes_nothing() {
        let languages = PluginLanguages::new(vec![PluginLanguage {
            parsers: vec![],
            extensions: vec!["zzz".to_string()],
            filenames: vec![],
        }]);

        assert_eq!(languages.parser_for("a.zzz", Some("zzz")), None);
    }
}
