use std::borrow::Cow;

use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::default_true;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum JSDocMode {
    #[default]
    TypeScript,
    JSDoc,
    Closure,
    Permissive,
    #[serde(untagged)]
    #[schemars(skip)]
    Unrecognized(String),
}

impl JSDocMode {
    const fn is_default(&self) -> bool {
        matches!(self, Self::TypeScript)
    }

    const fn uses_closure_tags(&self) -> bool {
        matches!(self, Self::Closure | Self::Permissive)
    }
}

// <https://github.com/gajus/eslint-plugin-jsdoc/blob/v50.5.0/docs/settings.md>
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
pub struct JSDocPluginSettings {
    /// Tag vocabulary used by `jsdoc/check-tag-names`. Defaults to `"typescript"`.
    #[serde(default, skip_serializing_if = "JSDocMode::is_default")]
    pub mode: JSDocMode,

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
    tag_name_preference: IndexMap<String, TagNamePreference>,
    /// Custom tag definitions. `jsdoc/check-tag-names` recognizes each tag name.
    #[serde(default, rename = "structuredTags", skip_serializing_if = "FxHashMap::is_empty")]
    structured_tags: FxHashMap<String, serde_json::Map<String, serde_json::Value>>,
    // Not planning to support for now
    // min_lines: number
    // max_lines: number
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
            mode: JSDocMode::default(),
            ignore_private: false,
            ignore_internal: false,
            // Exists only for these defaults
            ignore_replaces_docs: true,
            override_replaces_docs: true,
            augments_extends_replaces_docs: false,
            implements_replaces_docs: false,
            exempt_destructured_roots_from_checks: false,
            tag_name_preference: IndexMap::default(),
            structured_tags: FxHashMap::default(),
        }
    }
}

impl JSDocPluginSettings {
    pub(crate) fn has_tag_preference(&self, name: &str) -> bool {
        self.tag_name_preference.contains_key(name)
    }

    pub(crate) fn invalid_tag_preferences(&self) -> usize {
        self.tag_name_preference.values().filter(|pref| pref.is_invalid()).count()
    }

    /// A preference destination is valid even if it is normally an alias for another tag.
    fn is_replacement(&self, name: &str) -> bool {
        self.tag_name_preference.values().any(|pref| pref.replacement() == Some(name))
    }

    pub(crate) fn is_configured_tag(&self, name: &str) -> bool {
        self.has_tag_preference(name)
            || self.structured_tags.contains_key(name)
            || self.default_alias(name).is_some()
            || self.is_replacement(name)
    }

    fn default_alias(&self, name: &str) -> Option<&'static str> {
        Some(match name {
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
            "returns" if self.mode.uses_closure_tags() => "return",
            "return" if !self.mode.uses_closure_tags() => "returns",
            "typeParam" if !matches!(self.mode, JSDocMode::JSDoc | JSDocMode::Unrecognized(_)) => {
                "template"
            }
            "exception" => "throws",
            "yield" => "yields",
            _ => return None,
        })
    }

    /// The diagnostic and optional replacement for a blocked or non-preferred tag.
    pub(crate) fn tag_preference(
        &self,
        name: &str,
    ) -> Option<(Cow<'_, str>, Option<Cow<'_, str>>)> {
        if self.is_replacement(name) {
            return None;
        }
        // ESLint accepts the `tag ` prefix for names inherited from Object.prototype.
        // When both spellings are configured, the later entry wins.
        let preference = self.tag_name_preference.iter().rev().find_map(|(key, value)| {
            (key.strip_prefix("tag ").unwrap_or(key) == name).then_some(value)
        });
        let blocked = || Cow::Owned(format!("Blacklisted tag found (`@{name}`)"));
        let mut message = None;
        let replacement: Cow<'_, str> = match preference {
            None => self.default_alias(name)?.into(),
            Some(TagNamePreference::Name(name)) if !name.is_empty() => name.as_str().into(),
            Some(TagNamePreference::Object { message: custom, replacement }) => {
                message = custom.as_deref().filter(|s| !s.is_empty()).map(Cow::Borrowed);
                let Some(replacement) = replacement.as_deref().filter(|s| !s.is_empty()) else {
                    return Some((message.unwrap_or_else(blocked), None));
                };
                replacement.into()
            }
            Some(TagNamePreference::Bool(false)) => return Some((blocked(), None)),
            Some(TagNamePreference::Bool(true)) => "true".into(),
            Some(TagNamePreference::Number(number)) if number.as_f64() != Some(0.0) => {
                number.to_string().into()
            }
            _ => return None,
        };
        let message = message.unwrap_or_else(|| {
            Cow::Owned(format!(
                r#"Invalid JSDoc tag (preference). Replace "{name}" JSDoc tag with "{replacement}"."#
            ))
        });
        Some((message, Some(replacement)))
    }

    /// Resolve original, known tag name to user preferred name.
    pub fn resolve_tag_name<'s>(&'s self, original_name: &'s str) -> &'s str {
        self.tag_name_preference
            .get(original_name)
            .and_then(TagNamePreference::replacement)
            .unwrap_or(original_name)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(untagged)]
enum TagNamePreference {
    Name(String),
    Object {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        replacement: Option<String>,
    },
    Bool(bool),
    Null,
    // Preserve malformed scalar preferences so check-tag-names can report them at the source.
    #[schemars(skip)]
    Number(serde_json::Number),
}

impl TagNamePreference {
    fn replacement(&self) -> Option<&str> {
        match self {
            Self::Name(value) | Self::Object { replacement: Some(value), .. } => Some(value),
            _ => None,
        }
    }

    fn is_invalid(&self) -> bool {
        matches!(self, Self::Bool(true))
            || matches!(self, Self::Number(n) if n.as_f64() != Some(0.0))
    }
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
    fn last_prefixed_or_unprefixed_preference_wins() {
        for (preferences, expected) in [
            (r#"{"constructor":"first","tag constructor":"second"}"#, "second"),
            (r#"{"tag constructor":"first","constructor":"second"}"#, "second"),
        ] {
            let settings: JSDocPluginSettings =
                serde_json::from_str(&format!(r#"{{"tagNamePreference":{preferences}}}"#)).unwrap();
            assert_eq!(
                settings.tag_preference("constructor").unwrap().1.as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn tag_preferences() {
        let settings = JSDocPluginSettings::deserialize(&serde_json::json!({
            "tagNamePreference": {
                "tag constructor": "cons",
                "foo": false,
                "bar": { "message": "do not use bar" },
                "baz": { "message": "prefer prop", "replacement": "prop" },
                "std": "standard",
            },
        }))
        .unwrap();
        assert!(settings.is_configured_tag("standard"));
        assert!(settings.is_configured_tag("constructor"));
        assert_eq!(settings.tag_preference("prop"), None);
        assert_eq!(settings.tag_preference("constructor").unwrap().1.as_deref(), Some("cons"));
        assert_eq!(
            settings.tag_preference("foo"),
            Some(("Blacklisted tag found (`@foo`)".into(), None))
        );
        assert_eq!(settings.tag_preference("bar"), Some((Cow::Borrowed("do not use bar"), None)));
        assert_eq!(
            settings.tag_preference("baz"),
            Some(("prefer prop".into(), Some("prop".into())))
        );
        assert_eq!(
            settings.tag_preference("std"),
            Some((
                "Invalid JSDoc tag (preference). Replace \"std\" JSDoc tag with \"standard\"."
                    .into(),
                Some("standard".into())
            ))
        );
    }
}
