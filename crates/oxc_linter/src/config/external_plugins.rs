use std::path::PathBuf;

use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{
        ArrayValidation, InstanceType, Metadata, ObjectValidation, Schema, SchemaObject,
        SubschemaValidation,
    },
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// External plugin entry containing the plugin specifier and optional custom name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalPluginEntry {
    /// Directory containing the config file that specified this plugin
    pub config_dir: PathBuf,
    /// Plugin specifier (path, package name, or URL)
    pub specifier: String,
    /// Optional custom name/alias for the plugin
    pub name: Option<String>,
}

/// Custom deserializer for `ExternalPluginEntry`.
/// Supports:
/// * String: `"my-plugin"`
/// * Object: `{ "name": "my-alias", "specifier": "my-plugin" }`
impl<'de> Deserialize<'de> for ExternalPluginEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PluginObject {
            name: String,
            specifier: String,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PluginEntry {
            String(String),
            Object(PluginObject),
        }

        let entry = PluginEntry::deserialize(deserializer)?;

        let (specifier, name) = match entry {
            PluginEntry::String(specifier) => (specifier, None),
            PluginEntry::Object(obj) => (obj.specifier, Some(obj.name)),
        };
        Ok(ExternalPluginEntry { config_dir: PathBuf::default(), specifier, name })
    }
}

/// Custom serializer for `ExternalPluginEntry`.
/// * Serializes entry without alias as just the specifier e.g. `"my-plugin"`.
/// * Serializes entry with alias as an object e.g. `{ "name": "my-alias", "specifier": "my-plugin" }`.
impl Serialize for ExternalPluginEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct PluginObject<'e> {
            name: &'e str,
            specifier: &'e str,
        }

        if let Some(name) = &self.name {
            PluginObject { name: name.as_str(), specifier: self.specifier.as_str() }
                .serialize(serializer)
        } else {
            self.specifier.serialize(serializer)
        }
    }
}

/// Custom JSON schema generator for external plugins that includes uniqueItems constraint
pub fn external_plugins_schema(generator: &mut SchemaGenerator) -> Schema {
    let entry_schema = generator.subschema_for::<ExternalPluginEntry>();

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

impl JsonSchema for ExternalPluginEntry {
    fn schema_name() -> String {
        "ExternalPluginEntry".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // Schema represents: string | { name: string, specifier: string }
        let string_schema = SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            metadata: Some(Box::new(Metadata {
                description: Some("Path or package name of the plugin".to_string()),
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
                        "Custom name/alias for the plugin.\n\n\
                         Note: The following plugin names are reserved because they are implemented \
                         natively in Rust within oxlint and cannot be used for JS plugins:\n\
                         - react (includes react-hooks)\n\
                         - unicorn\n\
                         - typescript (includes @typescript-eslint)\n\
                         - oxc\n\
                         - import (includes import-x)\n\
                         - jsdoc\n\
                         - jest\n\
                         - vitest\n\
                         - jsx-a11y\n\
                         - nextjs\n\
                         - react-perf\n\
                         - promise\n\
                         - node\n\
                         - vue\n\
                         - eslint\n\n\
                         If you need to use the JavaScript version of any of these plugins, \
                         provide a custom alias to avoid conflicts."
                            .to_string()
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
                    description: Some("Path or package name of the plugin".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );

        let object_schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                description: Some("Plugin with custom name/alias".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: object_properties,
                required: vec!["name".to_string(), "specifier".to_string()].into_iter().collect(),
                additional_properties: Some(Box::new(Schema::Bool(false))),
                ..Default::default()
            })),
            ..Default::default()
        };

        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![string_schema.into(), object_schema.into()]),
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
    fn test_deserialize() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "jsPlugins", default)]
            plugins: Option<FxHashSet<ExternalPluginEntry>>,
        }

        let json = serde_json::json!({
            "jsPlugins": [
                "./plugin.ts",
                { "name": "custom", "specifier": "./plugin2.ts" }
            ]
        });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        let plugins = config.plugins.as_ref().unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins.iter().filter(|e| e.name.is_some()).count(), 1);

        // Null
        let json = serde_json::json!({ "jsPlugins": null });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        assert!(config.plugins.is_none());

        // Empty array
        let json = serde_json::json!({ "jsPlugins": [] });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.plugins.unwrap().len(), 0);
    }

    #[test]
    fn test_deserialize_mixed_formats() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "jsPlugins", default)]
            plugins: Option<FxHashSet<ExternalPluginEntry>>,
        }

        // Mix string and object formats
        let json = serde_json::json!({
            "jsPlugins": [
                "eslint-plugin-import",
                { "name": "custom", "specifier": "./plugin.ts" }
            ]
        });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        let plugins = config.plugins.as_ref().unwrap();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins.iter().filter(|e| e.name.is_some()).count(), 1);
    }

    #[test]
    fn test_deserialize_rejects_invalid() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[expect(dead_code)]
            #[serde(rename = "jsPlugins", default)]
            plugins: Option<FxHashSet<ExternalPluginEntry>>,
        }

        // Extra fields should be rejected
        let json = serde_json::json!({
            "jsPlugins": [
                { "name": "x", "specifier": "y", "extra": "z" }
            ]
        });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());

        // Missing required fields should be rejected
        let json = serde_json::json!({ "jsPlugins": [{ "name": "x" }] });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());

        // Object with single arbitrary field should be rejected
        let json = serde_json::json!({ "jsPlugins": [{ "alias": "plugin" }] });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());
    }

    #[test]
    fn test_serialize() {
        let mut plugins = FxHashSet::default();
        plugins.insert(ExternalPluginEntry {
            config_dir: PathBuf::default(),
            specifier: "./plugin.ts".to_string(),
            name: None,
        });
        plugins.insert(ExternalPluginEntry {
            config_dir: PathBuf::default(),
            specifier: "./plugin2.ts".to_string(),
            name: Some("custom".to_string()),
        });

        let json = serde_json::to_value(Some(plugins)).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        // Null
        let json = serde_json::to_value(&None::<FxHashSet<ExternalPluginEntry>>).unwrap();
        assert!(json.is_null());
    }
}
