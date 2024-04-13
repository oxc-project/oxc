use rustc_hash::FxHashMap;
use serde::Deserialize;

/// <https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md>
#[derive(Debug, Deserialize, Default)]
pub struct JSDocPluginSettings {
    /// For all rules but NOT apply to `check-access` and `empty-tags` rule
    #[serde(default, rename = "ignorePrivate")]
    pub ignore_private: bool,
    /// For all rules but NOT apply to `empty-tags` rule
    #[serde(default, rename = "ignoreInternal")]
    pub ignore_internal: bool,

    /// Only for `require-(yields|returns|description|example|param|throws)` rule
    #[serde(default = "default_true", rename = "ignoreReplacesDocs")]
    pub ignore_replaces_docs: bool,
    /// Only for `require-(yields|returns|description|example|param|throws)` rule
    #[serde(default = "default_true", rename = "overrideReplacesDocs")]
    pub override_replaces_docs: bool,
    /// Only for `require-(yields|returns|description|example|param|throws)` rule
    #[serde(default, rename = "augmentsExtendsReplacesDocs")]
    pub arguments_extends_replaces_docs: bool,
    /// Only for `require-(yields|returns|description|example|param|throws)` rule
    #[serde(default, rename = "implementsReplacesDocs")]
    pub implements_replaces_docs: bool,

    /// Only for `require-param-type` and `require-param-description` rule
    #[serde(default, rename = "exemptDestructuredRootsFromChecks")]
    pub exempt_destructured_roots_from_checks: bool,

    #[serde(default, rename = "tagNamePreference")]
    tag_name_preference: FxHashMap<String, TagNamePreference>,
    //
    // Not planning to support?
    // min_lines: number
    // max_lines: number
    // mode: string
    //
    // TODO: Need more investigation to understand these usage...
    //
    // Only for `check-types` and `no-undefined-types` rule
    // preferred_types: Record<
    //   string,
    //   false | string | {
    //     message: string;
    //     replacement?: false | string;
    //     skipRootChecking?: boolean;
    //   }
    // >
    //
    // structured_tags: Record<
    //   string,
    //   {
    //     name?: "text" | "namepath-defining" | "namepath-referencing" | false;
    //     type?: boolean | string[];
    //     required?: ("name" | "type" | "typeOrNameRequired")[];
    //   }
    // >
    //
    // I know this but not sure how to implement
    // contexts: string[] | {
    //   disallowName?: string;
    //   allowName?: string;
    //   context?: string;
    //   comment?: string;
    //   tags?: string[];
    //   replacement?: string;
    //   minimum?: number;
    //   message?: string;
    //   forceRequireReturn?: boolean;
    // }[]
}

impl JSDocPluginSettings {
    pub fn is_blocked_tag_name(&self, tag_name: &str) -> Option<String> {
        match self.tag_name_preference.get(tag_name) {
            Some(TagNamePreference::FalseOnly(_)) => Some(format!("Unexpected tag `@{tag_name}`")),
            Some(
                TagNamePreference::ObjectWithMessage { message }
                | TagNamePreference::ObjectWithMessageAndReplacement { message, .. },
            ) => Some(message.to_string()),
            _ => None,
        }
    }

    pub fn resolve_tag_name(&self, tag_name: &str) -> String {
        match self.tag_name_preference.get(tag_name) {
            Some(
                TagNamePreference::TagNameOnly(replacement)
                | TagNamePreference::ObjectWithMessageAndReplacement { replacement, .. },
            ) => replacement.to_string(),
            _ => {
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
                .to_string()
            }
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
    ObjectWithMessageAndReplacement {
        message: String,
        replacement: String,
    },
    ObjectWithMessage {
        message: String,
    },
    #[allow(dead_code)]
    FalseOnly(bool), // Should care `true`...?
}

#[cfg(test)]
mod test {
    use super::JSDocPluginSettings;
    use serde::Deserialize;

    #[test]
    fn parse_defaults() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();

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
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({
            "ignorePrivate": true,
            "ignoreInternal": true,
        }))
        .unwrap();

        assert!(settings.ignore_private);
        assert!(settings.ignore_internal);
        assert_eq!(settings.tag_name_preference.len(), 0);
    }

    #[test]
    fn resolve_tag_name() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();
        assert_eq!(settings.resolve_tag_name("foo"), "foo".to_string());
        assert_eq!(settings.resolve_tag_name("virtual"), "abstract".to_string());
        assert_eq!(settings.resolve_tag_name("fileoverview"), "file".to_string());
        assert_eq!(settings.resolve_tag_name("overview"), "file".to_string());

        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({
            "tagNamePreference": {
                "foo": "bar",
                "virtual": "overridedefault",
                "replace": { "message": "noop", "replacement": "noop" },
                "blocked": { "message": "noop"  },
                "blocked2": false
            }
        }))
        .unwrap();
        assert_eq!(settings.resolve_tag_name("foo"), "bar".to_string());
        assert_eq!(settings.resolve_tag_name("virtual"), "overridedefault".to_string());
        assert_eq!(settings.resolve_tag_name("replace"), "noop".to_string());
        assert_eq!(settings.resolve_tag_name("blocked"), "blocked".to_string());
        assert_eq!(settings.resolve_tag_name("blocked2"), "blocked2".to_string());
    }

    #[test]
    fn is_blocked_tag_name() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();
        assert_eq!(settings.is_blocked_tag_name("foo"), None);

        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({
            "tagNamePreference": {
                "foo": false,
                "bar": { "message": "do not use bar" },
                "baz": { "message": "baz is noop now", "replacement": "noop" }
            }
        }))
        .unwrap();
        assert_eq!(settings.is_blocked_tag_name("foo"), Some("Unexpected tag `@foo`".to_string()));
        assert_eq!(settings.is_blocked_tag_name("bar"), Some("do not use bar".to_string()));
        assert_eq!(settings.is_blocked_tag_name("baz"), Some("baz is noop now".to_string()));
    }
}
