use std::borrow::Cow;

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::default_true;

// <https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md>
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
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
    pub augments_extends_replaces_docs: bool,

    /// Only for `require-(yields|returns|description|example|param|throws)` rule
    #[serde(default, rename = "implementsReplacesDocs")]
    pub implements_replaces_docs: bool,

    /// Only for `require-param-type` and `require-param-description` rule
    #[serde(default, rename = "exemptDestructuredRootsFromChecks")]
    pub exempt_destructured_roots_from_checks: bool,

    #[serde(default, rename = "tagNamePreference")]
    tag_name_preference: FxHashMap<String, TagNamePreference>,
    // Not planning to support for now
    // min_lines: number
    // max_lines: number
    // mode: string("typescript" | "closure" | "jsdoc")
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

// `Default` attribute does not call custom `default = "path"` function!
impl Default for JSDocPluginSettings {
    fn default() -> Self {
        Self {
            ignore_private: false,
            ignore_internal: false,
            // Exists only for these defaults
            ignore_replaces_docs: true,
            override_replaces_docs: true,
            augments_extends_replaces_docs: false,
            implements_replaces_docs: false,
            exempt_destructured_roots_from_checks: false,
            tag_name_preference: FxHashMap::default(),
        }
    }
}

impl JSDocPluginSettings {
    /// Only for `check-tag-names` rule
    /// Return `Some(reason)` if blocked
    pub fn check_blocked_tag_name(&self, tag_name: &str) -> Option<Cow<str>> {
        match self.tag_name_preference.get(tag_name) {
            Some(TagNamePreference::FalseOnly(_)) => {
                Some(Cow::Owned(format!("Unexpected tag `@{tag_name}`.")))
            }
            Some(TagNamePreference::ObjectWithMessage { message }) => Some(Cow::Borrowed(message)),
            _ => None,
        }
    }

    /// Only for `check-tag-names` rule
    /// Return `Some(reason)` if replacement found or default aliased
    pub fn check_preferred_tag_name(&self, original_name: &str) -> Option<Cow<str>> {
        let reason = |preferred_name: &str| -> Cow<str> {
            Cow::Owned(format!("Replace tag `@{original_name}` with `@{preferred_name}`."))
        };

        match self.tag_name_preference.get(original_name) {
            Some(TagNamePreference::TagNameOnly(preferred_name)) => Some(reason(preferred_name)),
            Some(TagNamePreference::ObjectWithMessageAndReplacement { message, .. }) => {
                Some(Cow::Borrowed(message))
            }
            _ => {
                // https://github.com/gajus/eslint-plugin-jsdoc/blob/main/docs/settings.md#default-preferred-aliases
                let aliased_name = match original_name {
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
                    _ => original_name,
                };

                if aliased_name != original_name {
                    return Some(reason(aliased_name));
                }

                None
            }
        }
    }

    /// Only for `check-tag-names` rule
    /// Return all user replacement tag names
    pub fn list_user_defined_tag_names(&self) -> Vec<&str> {
        self.tag_name_preference
            .iter()
            .filter_map(|(_, pref)| match pref {
                TagNamePreference::TagNameOnly(replacement)
                | TagNamePreference::ObjectWithMessageAndReplacement { replacement, .. } => {
                    Some(replacement.as_str())
                }
                _ => None,
            })
            .collect()
    }

    /// Resolve original, known tag name to user preferred name
    /// If not defined, return original name
    pub fn resolve_tag_name<'s>(&'s self, original_name: &'s str) -> &'s str {
        match self.tag_name_preference.get(original_name) {
            Some(
                TagNamePreference::TagNameOnly(replacement)
                | TagNamePreference::ObjectWithMessageAndReplacement { replacement, .. },
            ) => replacement,
            _ => original_name,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
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
    use std::borrow::Cow;

    use serde::Deserialize;

    use super::JSDocPluginSettings;

    #[test]
    fn parse_defaults() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();

        assert!(!settings.ignore_private);
        assert!(!settings.ignore_internal);
        assert_eq!(settings.tag_name_preference.len(), 0);
        assert!(settings.ignore_replaces_docs);
        assert!(settings.override_replaces_docs);
        assert!(!settings.augments_extends_replaces_docs);
        assert!(!settings.implements_replaces_docs);

        let settings = JSDocPluginSettings::default();

        assert!(!settings.ignore_private);
        assert!(!settings.ignore_internal);
        assert_eq!(settings.tag_name_preference.len(), 0);
        assert!(settings.ignore_replaces_docs);
        assert!(settings.override_replaces_docs);
        assert!(!settings.augments_extends_replaces_docs);
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
    fn list_user_defined_tag_names() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();
        assert_eq!(settings.list_user_defined_tag_names().len(), 0);

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
        let mut preferred = settings.list_user_defined_tag_names();
        preferred.sort_unstable();
        assert_eq!(preferred, vec!["bar", "noop", "overridedefault"]);
    }

    #[test]
    fn check_blocked_tag_name() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();
        assert_eq!(settings.check_blocked_tag_name("foo"), None);

        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({
            "tagNamePreference": {
                "foo": false,
                "bar": { "message": "do not use bar" },
                "baz": { "message": "baz is noop now", "replacement": "noop" }
            }
        }))
        .unwrap();
        assert_eq!(
            settings.check_blocked_tag_name("foo"),
            Some(Cow::Borrowed("Unexpected tag `@foo`."))
        );
        assert_eq!(settings.check_blocked_tag_name("bar"), Some(Cow::Borrowed("do not use bar")));
        assert_eq!(settings.check_blocked_tag_name("baz"), None);
    }

    #[test]
    fn check_preferred_tag_name() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({})).unwrap();
        assert_eq!(settings.check_preferred_tag_name("foo"), None);

        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({
            "tagNamePreference": {
                "foo": false,
                "bar": { "message": "do not use bar" },
                "baz": { "message": "baz is noop now", "replacement": "noop" },
                "qux": "quux"
            }
        }))
        .unwrap();
        assert_eq!(settings.check_preferred_tag_name("foo"), None,);
        assert_eq!(settings.check_preferred_tag_name("bar"), None);
        assert_eq!(settings.check_preferred_tag_name("baz"), Some("baz is noop now".into()));
        assert_eq!(
            settings.check_preferred_tag_name("qux"),
            Some("Replace tag `@qux` with `@quux`.".into())
        );
    }
}
