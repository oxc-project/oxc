use std::path::PathBuf;

use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{
        ArrayValidation, InstanceType, Metadata, ObjectValidation, Schema, SchemaObject,
        SubschemaValidation,
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// External language plugin entry (e.g. Vue, Svelte, Angular templates).
///
/// Configured via `languagePlugins` in oxlint config. See
/// <https://github.com/oxc-project/oxc/discussions/21936>.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguagePluginEntry {
    /// Directory containing the config file that specified this plugin.
    pub config_dir: PathBuf,
    /// Plugin specifier (path, package name, or URL).
    pub specifier: String,
    /// Optional custom name/alias for the plugin.
    pub name: Option<String>,
    /// Optional project-owned file globs that select this language plugin.
    ///
    /// When omitted, Oxlint falls back to the plugin's `defaultFiles`
    /// (extension / filename defaults such as `.vue`).
    pub pattern: Option<Vec<String>>,
    /// Opaque options forwarded to the language plugin.
    ///
    /// Not part of `Hash`/`Eq`; compared by pointer identity of the serialized form
    /// is avoided by storing a canonical JSON string for hashing.
    pub options_json: Option<String>,
}

impl LanguagePluginEntry {
    /// Parsed plugin options, if any.
    pub fn options(&self) -> Option<Value> {
        self.options_json.as_ref().and_then(|s| serde_json::from_str(s).ok())
    }
}

/// Custom deserializer for [`LanguagePluginEntry`].
///
/// Supports:
/// * String: `"vue-language-plugin"`
/// * Object (jsPlugins-style): `{ "name": "alias", "specifier": "./plugin.ts", "pattern"?, "options"? }`
/// * Object (RFC-style): `{ "name": "vue-language-plugin", "pattern"?, "options"? }`
///   where `name` is treated as the specifier when `specifier` is omitted.
impl<'de> Deserialize<'de> for LanguagePluginEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PluginObject {
            name: Option<String>,
            specifier: Option<String>,
            pattern: Option<PatternField>,
            options: Option<Value>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PatternField {
            One(String),
            Many(Vec<String>),
        }

        impl PatternField {
            fn into_vec(self) -> Vec<String> {
                match self {
                    Self::One(s) => vec![s],
                    Self::Many(v) => v,
                }
            }
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PluginEntry {
            String(String),
            Object(PluginObject),
        }

        let entry = PluginEntry::deserialize(deserializer)?;

        let (specifier, name, pattern, options_json) = match entry {
            PluginEntry::String(specifier) => (specifier, None, None, None),
            PluginEntry::Object(obj) => {
                let pattern = obj.pattern.map(PatternField::into_vec);
                let options_json = match obj.options {
                    Some(Value::Null) => None,
                    Some(v) => Some(
                        serde_json::to_string(&v)
                            .map_err(serde::de::Error::custom)?,
                    ),
                    None => None,
                };

                match (obj.specifier, obj.name) {
                    (Some(specifier), name) => (specifier, name, pattern, options_json),
                    (None, Some(name)) => (name, None, pattern, options_json),
                    (None, None) => {
                        return Err(serde::de::Error::custom(
                            "language plugin object must include `specifier` or `name`",
                        ));
                    }
                }
            }
        };

        Ok(LanguagePluginEntry {
            config_dir: PathBuf::default(),
            specifier,
            name,
            pattern,
            options_json,
        })
    }
}

impl Serialize for LanguagePluginEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        let has_extra = self.name.is_some() || self.pattern.is_some() || self.options_json.is_some();
        if !has_extra {
            return self.specifier.serialize(serializer);
        }

        let mut map = serializer.serialize_map(None)?;
        if let Some(name) = &self.name {
            map.serialize_entry("name", name)?;
            map.serialize_entry("specifier", &self.specifier)?;
        } else {
            // RFC-style: name field carries the specifier
            map.serialize_entry("name", &self.specifier)?;
        }
        if let Some(pattern) = &self.pattern {
            if pattern.len() == 1 {
                map.serialize_entry("pattern", &pattern[0])?;
            } else {
                map.serialize_entry("pattern", pattern)?;
            }
        }
        if let Some(options_json) = &self.options_json {
            let value: Value =
                serde_json::from_str(options_json).map_err(serde::ser::Error::custom)?;
            map.serialize_entry("options", &value)?;
        }
        map.end()
    }
}

/// Custom JSON schema for `languagePlugins`.
pub fn language_plugins_schema(generator: &mut SchemaGenerator) -> Schema {
    let entry_schema = generator.subschema_for::<LanguagePluginEntry>();

    let array_schema = SchemaObject {
        instance_type: Some(InstanceType::Array.into()),
        array: Some(Box::new(ArrayValidation {
            items: Some(entry_schema.into()),
            unique_items: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    };

    SchemaObject {
        subschemas: Some(Box::new(SubschemaValidation {
            any_of: Some(vec![
                SchemaObject {
                    instance_type: Some(InstanceType::Null.into()),
                    ..Default::default()
                }
                .into(),
                array_schema.into(),
            ]),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

impl JsonSchema for LanguagePluginEntry {
    fn schema_name() -> String {
        "LanguagePluginEntry".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let string_schema = SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "Path or package name of the language plugin (e.g. `vue-language-plugin`)"
                        .to_string(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        };

        let mut object_properties = schemars::Map::new();
        object_properties.insert(
            "name".to_string(),
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "Plugin alias, or the plugin specifier when `specifier` is omitted.\n\n\
                         When both `name` and `specifier` are set, `name` is an alias \
                         (same as `jsPlugins`)."
                            .to_string(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );
        object_properties.insert(
            "specifier".to_string(),
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some("Path or package name of the language plugin".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );
        object_properties.insert(
            "pattern".to_string(),
            SchemaObject {
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "Optional project-owned globs selecting files for this language plugin.\n\n\
                         When omitted, Oxlint uses the plugin's `defaultFiles` \
                         (prefer extensions / filenames such as `.vue`)."
                            .to_string(),
                    ),
                    ..Default::default()
                })),
                subschemas: Some(Box::new(SubschemaValidation {
                    any_of: Some(vec![
                        SchemaObject {
                            instance_type: Some(InstanceType::String.into()),
                            ..Default::default()
                        }
                        .into(),
                        SchemaObject {
                            instance_type: Some(InstanceType::Array.into()),
                            array: Some(Box::new(ArrayValidation {
                                items: Some(
                                    Schema::Object(SchemaObject {
                                        instance_type: Some(InstanceType::String.into()),
                                        ..Default::default()
                                    })
                                    .into(),
                                ),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }
                        .into(),
                    ]),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );
        object_properties.insert(
            "options".to_string(),
            SchemaObject {
                instance_type: Some(InstanceType::Object.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some("Options forwarded to the language plugin".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );

        let object_with_name = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "Language plugin identified by `name` (RFC-style, or alias when `specifier` is also set)"
                        .to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: object_properties.clone(),
                required: ["name".to_string()].into_iter().collect(),
                additional_properties: Some(Box::new(Schema::Bool(false))),
                ..Default::default()
            })),
            ..Default::default()
        };

        let object_with_specifier = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "Language plugin identified by `specifier` (optional `name` alias, pattern, options)"
                        .to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: object_properties,
                required: ["specifier".to_string()].into_iter().collect(),
                additional_properties: Some(Box::new(Schema::Bool(false))),
                ..Default::default()
            })),
            ..Default::default()
        };

        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![
                    string_schema.into(),
                    object_with_name.into(),
                    object_with_specifier.into(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashSet;

    use super::*;

    #[test]
    fn test_deserialize_string() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "languagePlugins", default)]
            plugins: Option<FxHashSet<LanguagePluginEntry>>,
        }

        let json = serde_json::json!({ "languagePlugins": ["vue-language-plugin"] });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        let plugins = config.plugins.as_ref().unwrap();
        assert_eq!(plugins.len(), 1);
        let entry = plugins.iter().next().unwrap();
        assert_eq!(entry.specifier, "vue-language-plugin");
        assert!(entry.name.is_none());
        assert!(entry.pattern.is_none());
    }

    #[test]
    fn test_deserialize_rfc_object() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "languagePlugins", default)]
            plugins: Option<FxHashSet<LanguagePluginEntry>>,
        }

        let json = serde_json::json!({
            "languagePlugins": [{
                "name": "vue-language-plugin",
                "pattern": "packages/app2/*.vue",
                "options": { "script": { "jsx": true } }
            }]
        });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        let entry = config.plugins.as_ref().unwrap().iter().next().unwrap();
        assert_eq!(entry.specifier, "vue-language-plugin");
        assert!(entry.name.is_none());
        assert_eq!(entry.pattern.as_ref().unwrap(), &vec!["packages/app2/*.vue".to_string()]);
        assert!(entry.options().unwrap()["script"]["jsx"].as_bool().unwrap());
    }

    #[test]
    fn test_deserialize_alias_object() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "languagePlugins", default)]
            plugins: Option<FxHashSet<LanguagePluginEntry>>,
        }

        let json = serde_json::json!({
            "languagePlugins": [{
                "name": "vue",
                "specifier": "./vue-language-plugin.ts",
                "pattern": ["**/*.vue", "**/*.vuex"]
            }]
        });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        let entry = config.plugins.as_ref().unwrap().iter().next().unwrap();
        assert_eq!(entry.specifier, "./vue-language-plugin.ts");
        assert_eq!(entry.name.as_deref(), Some("vue"));
        assert_eq!(entry.pattern.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_deserialize_rejects_invalid() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[expect(dead_code)]
            #[serde(rename = "languagePlugins", default)]
            plugins: Option<FxHashSet<LanguagePluginEntry>>,
        }

        let json = serde_json::json!({
            "languagePlugins": [{ "name": "x", "specifier": "y", "extra": "z" }]
        });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());

        let json = serde_json::json!({ "languagePlugins": [{ "pattern": "*.vue" }] });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());
    }
}
