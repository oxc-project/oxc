use rustc_hash::FxHashMap;
use serde::Deserialize;

/// https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettingsJSDoc {
    /// Not apply to `check-access` and `empty-tags` rule
    #[serde(default, rename = "ignorePrivate")]
    pub ignore_private: bool,
    /// Not apply to `empty-tags` rule
    #[serde(default, rename = "ignoreInternal")]
    pub ignore_internal: bool,
    /// Apply to `require-*` rules only
    #[serde(default = "default_true", rename = "ignoreReplacesDocs")]
    pub ignore_replaces_docs: bool,
    /// Apply to `require-*` rules only
    #[serde(default = "default_true", rename = "overrideReplacesDocs")]
    pub override_replaces_docs: bool,
    /// Apply to `require-*` rules only
    #[serde(default, rename = "augmentsExtendsReplacesDocs")]
    pub arguments_extends_replaces_docs: bool,
    /// Apply to `require-*` rules only
    #[serde(default, rename = "implementsReplacesDocs")]
    pub implements_replaces_docs: bool,

    #[serde(default, rename = "tagNamePreference")]
    tag_name_preference: FxHashMap<String, TagNamePreference>,

    // Not planning to support?
    // min_lines
    // max_lines

    // XXX: Need more investigation to understand these...
    // mode: String,
    // preferred_types: FxHashMap<String, PreferredType>,
    // structured_tags: FxHashMap<String, StructuredTag>,
    // contexts: Vec<Context>,
}

impl ESLintSettingsJSDoc {
    pub fn resolve_tag_name<'a>(&self, tag_name: &'a str) -> &'a str {
        // TODO: How...?
        match self.tag_name_preference.get(tag_name) {
            Some(TagNamePreference::TagNameOnly(tag_name)) => {
                println!("{tag_name}");
            }
            Some(TagNamePreference::FalseOnly) => {
                println!("false");
            }
            Some(TagNamePreference::ObjectWithMessage { message }) => {
                println!("{message}");
            }
            Some(TagNamePreference::ObjectWithMessageAndReplacement { message, replacement }) => {
                println!("{message}, {replacement}");
            }
            None => {
                println!("None");
            }
        };

        // https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md#default-preferred-aliases
        match tag_name {
            "virtual" => "abstract",
            "extends" => "augments",
            "constructor" => "class",
            "const" => "constant",
            "defaultvalue" => "default",
            "desc" => "description",
            "host" => "external",
            "fileoverview" | "overview" => "file",
            "emits" => "fires",
            "func" | "method" => "function",
            "var" => "member",
            "arg" | "argument" => "param",
            "prop" => "property",
            "return" => "returns",
            "exception" => "throws",
            "yield" => "yields",
            _ => tag_name,
        }
    }
}

// Deserialize helper types

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum TagNamePreference {
    TagNameOnly(String),
    FalseOnly,
    ObjectWithMessage { message: String },
    ObjectWithMessageAndReplacement { message: String, replacement: String },
}

#[cfg(test)]
mod test {
    use super::ESLintSettingsJSDoc;
    use serde::Deserialize;

    #[test]
    fn parse_defaults() {
        let settings = ESLintSettingsJSDoc::deserialize(&serde_json::json!({})).unwrap();

        assert!(!settings.ignore_private);
        assert!(!settings.ignore_internal);
        assert_eq!(settings.tag_name_preference.len(), 0);
        assert!(settings.ignore_replaces_docs);
        assert!(settings.override_replaces_docs);
        assert!(!settings.arguments_extends_replaces_docs);
        assert!(!settings.implements_replaces_docs);
    }

    #[test]
    fn parse_bools() {
        let settings = ESLintSettingsJSDoc::deserialize(&serde_json::json!({
            "ignorePrivate": true,
            "ignoreInternal": true,
        }))
        .unwrap();

        assert!(settings.ignore_private);
        assert!(settings.ignore_internal);
        assert_eq!(settings.tag_name_preference.len(), 0);
    }

    #[test]
    fn get_preferred_tag_name() {
        let settings = ESLintSettingsJSDoc::deserialize(&serde_json::json!({})).unwrap();
        assert_eq!(settings.resolve_tag_name("foo"), "foo");
        assert_eq!(settings.resolve_tag_name("virtual"), "abstract");
        assert_eq!(settings.resolve_tag_name("fileoverview"), "file");
        assert_eq!(settings.resolve_tag_name("overview"), "file");
    }
}
