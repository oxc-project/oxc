use std::path::PathBuf;

use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{ArrayValidation, InstanceType, Metadata, ObjectValidation, Schema, SchemaObject},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// External parser entry containing the parser specifier, file patterns, and optional parser options
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalParserEntry {
    /// Directory containing the config file that specified this parser
    pub config_dir: PathBuf,
    /// Parser specifier (path, package name, or URL)
    pub specifier: String,
    /// File patterns to match (e.g., `["*.marko", "*.mdx"]`)
    pub patterns: Vec<String>,
    /// Optional parser options to pass to the parser
    pub parser_options: Option<serde_json::Value>,
}

/// Custom deserializer for `ExternalParserEntry`.
/// Supports:
/// * Object: `{ "parser": "@marko/compiler", "patterns": ["*.marko"] }`
/// * Object with options: `{ "parser": "@marko/compiler", "patterns": ["*.marko"], "parserOptions": { ... } }`
impl<'de> Deserialize<'de> for ExternalParserEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ParserObject {
            parser: String,
            patterns: Vec<String>,
            #[serde(rename = "parserOptions")]
            parser_options: Option<serde_json::Value>,
        }

        let obj = ParserObject::deserialize(deserializer)?;

        if obj.patterns.is_empty() {
            return Err(serde::de::Error::custom("patterns array must not be empty"));
        }

        Ok(ExternalParserEntry {
            config_dir: PathBuf::default(),
            specifier: obj.parser,
            patterns: obj.patterns,
            parser_options: obj.parser_options,
        })
    }
}

/// Custom serializer for `ExternalParserEntry`.
impl Serialize for ExternalParserEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct ParserObject<'e> {
            parser: &'e str,
            patterns: &'e [String],
            #[serde(rename = "parserOptions", skip_serializing_if = "Option::is_none")]
            parser_options: &'e Option<serde_json::Value>,
        }

        ParserObject {
            parser: self.specifier.as_str(),
            patterns: &self.patterns,
            parser_options: &self.parser_options,
        }
        .serialize(serializer)
    }
}

impl JsonSchema for ExternalParserEntry {
    fn schema_name() -> String {
        "ExternalParserEntry".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // Schema represents: { parser: string, patterns: string[], parserOptions?: object }
        let mut object_properties = schemars::Map::new();

        object_properties.insert(
            "parser".to_string(),
            SchemaObject {
                instance_type: Some(InstanceType::String.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "Path or package name of the parser.\n\n\
                         The parser must implement the standard ESLint parser interface:\n\
                         - `parse(code, options)` returning an ESTree-compatible AST, or\n\
                         - `parseForESLint(code, options)` returning `{ ast, scopeManager?, visitorKeys?, services? }`"
                            .to_string(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );

        object_properties.insert(
            "patterns".to_string(),
            SchemaObject {
                instance_type: Some(InstanceType::Array.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "Glob patterns for files that should use this parser (e.g., [\"*.marko\", \"*.mdx\"])"
                            .to_string(),
                    ),
                    ..Default::default()
                })),
                array: Some(Box::new(ArrayValidation {
                    items: Some(schemars::schema::SingleOrVec::Single(Box::new(
                        SchemaObject {
                            instance_type: Some(InstanceType::String.into()),
                            ..Default::default()
                        }
                        .into(),
                    ))),
                    min_items: Some(1),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );

        object_properties.insert(
            "parserOptions".to_string(),
            SchemaObject {
                instance_type: Some(InstanceType::Object.into()),
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "Options to pass to the parser's parse/parseForESLint function".to_string(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),
        );

        SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "External parser configuration for custom file types.\n\n\
                     Note: JS parsers are experimental and not subject to semver."
                        .to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: object_properties,
                required: vec!["parser".to_string(), "patterns".to_string()].into_iter().collect(),
                additional_properties: Some(Box::new(Schema::Bool(false))),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "jsParsers", default)]
            parsers: Option<Vec<ExternalParserEntry>>,
        }

        let json = serde_json::json!({
            "jsParsers": [
                { "parser": "@marko/compiler", "patterns": ["*.marko"] },
                { "parser": "./custom-parser.js", "patterns": ["*.custom", "*.ext"], "parserOptions": { "ecmaVersion": 2020 } }
            ]
        });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        let parsers = config.parsers.as_ref().unwrap();
        assert_eq!(parsers.len(), 2);
        assert_eq!(parsers[0].specifier, "@marko/compiler");
        assert_eq!(parsers[0].patterns, vec!["*.marko"]);
        assert!(parsers[0].parser_options.is_none());
        assert_eq!(parsers[1].specifier, "./custom-parser.js");
        assert_eq!(parsers[1].patterns, vec!["*.custom", "*.ext"]);
        assert!(parsers[1].parser_options.is_some());

        // Null
        let json = serde_json::json!({ "jsParsers": null });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        assert!(config.parsers.is_none());

        // Empty array
        let json = serde_json::json!({ "jsParsers": [] });
        let config: TestConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.parsers.unwrap().len(), 0);
    }

    #[test]
    fn test_deserialize_rejects_invalid() {
        #[derive(Deserialize)]
        struct TestConfig {
            #[expect(dead_code)]
            #[serde(rename = "jsParsers", default)]
            parsers: Option<Vec<ExternalParserEntry>>,
        }

        // Extra fields should be rejected
        let json = serde_json::json!({
            "jsParsers": [
                { "parser": "x", "patterns": ["*.x"], "extra": "z" }
            ]
        });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());

        // Missing required fields should be rejected
        let json = serde_json::json!({ "jsParsers": [{ "parser": "x" }] });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());

        let json = serde_json::json!({ "jsParsers": [{ "patterns": ["*.x"] }] });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());

        // Empty patterns array should be rejected
        let json = serde_json::json!({ "jsParsers": [{ "parser": "x", "patterns": [] }] });
        assert!(serde_json::from_value::<TestConfig>(json).is_err());
    }

    #[test]
    fn test_serialize() {
        let parsers = vec![
            ExternalParserEntry {
                config_dir: PathBuf::default(),
                specifier: "@marko/compiler".to_string(),
                patterns: vec!["*.marko".to_string()],
                parser_options: None,
            },
            ExternalParserEntry {
                config_dir: PathBuf::default(),
                specifier: "./custom-parser.js".to_string(),
                patterns: vec!["*.custom".to_string()],
                parser_options: Some(serde_json::json!({ "ecmaVersion": 2020 })),
            },
        ];

        let json = serde_json::to_value(Some(parsers)).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["parser"], "@marko/compiler");
        assert_eq!(arr[0]["patterns"], serde_json::json!(["*.marko"]));
        assert!(arr[0].get("parserOptions").is_none());
        assert_eq!(arr[1]["parser"], "./custom-parser.js");
        assert_eq!(arr[1]["parserOptions"]["ecmaVersion"], 2020);

        // Null
        let json = serde_json::to_value(&None::<Vec<ExternalParserEntry>>).unwrap();
        assert!(json.is_null());
    }

    #[test]
    fn test_roundtrip() {
        let parsers = vec![ExternalParserEntry {
            config_dir: PathBuf::default(),
            specifier: "@marko/compiler".to_string(),
            patterns: vec!["*.marko".to_string(), "*.marko.js".to_string()],
            parser_options: Some(serde_json::json!({ "sourceType": "module" })),
        }];

        let serialized = serde_json::to_string(&parsers).unwrap();
        let deserialized: Vec<ExternalParserEntry> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsers.len(), deserialized.len());
        assert_eq!(parsers[0].specifier, deserialized[0].specifier);
        assert_eq!(parsers[0].patterns, deserialized[0].patterns);
        assert_eq!(parsers[0].parser_options, deserialized[0].parser_options);
    }
}
