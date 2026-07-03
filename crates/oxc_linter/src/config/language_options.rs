use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Language options for files matched by an override, mirroring ESLint's `languageOptions`.
///
/// Currently supports routing files to an external (JS) parser, for file types which
/// oxlint's native parser cannot parse (e.g. Ember's `.gjs`/`.gts` files).
///
/// Note: External parsers are only supported when running oxlint via the CLI (Node.js),
/// same as `jsPlugins`.
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OxlintLanguageOptions {
    /// External parser for files matched by this override.
    ///
    /// A module specifier (path, package name, or URL), resolved the same way as `jsPlugins`
    /// entries. The module must export an ESLint-compatible parser
    /// (an object with a `parseForESLint` or `parse` method).
    ///
    /// Files matched by an override with a `parser` are parsed by that parser instead of
    /// oxlint's native parser. Only JS plugin rules run on such files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser: Option<ExternalParserEntry>,

    /// Options passed verbatim to the external parser.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser_options: Option<serde_json::Value>,
}

/// External parser entry containing the parser module specifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalParserEntry {
    /// Directory containing the config file that specified this parser
    pub config_dir: PathBuf,
    /// Parser module specifier (path, package name, or URL)
    pub specifier: String,
}

/// Custom deserializer for `ExternalParserEntry`, deserializes from a specifier string.
impl<'de> Deserialize<'de> for ExternalParserEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let specifier = String::deserialize(deserializer)?;
        Ok(ExternalParserEntry { config_dir: PathBuf::default(), specifier })
    }
}

/// Custom serializer for `ExternalParserEntry`, serializes as just the specifier string.
impl Serialize for ExternalParserEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.specifier.serialize(serializer)
    }
}

impl JsonSchema for ExternalParserEntry {
    fn schema_name() -> String {
        "ExternalParserEntry".to_string()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema: schemars::schema::SchemaObject = generator.subschema_for::<String>().into();
        schema.metadata().description =
            Some("Path or package name of an ESLint-compatible parser module".to_string());
        schema.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let language_options: OxlintLanguageOptions = serde_json::from_value(serde_json::json!({
            "parser": "ember-eslint-parser",
        }))
        .unwrap();
        let parser = language_options.parser.unwrap();
        assert_eq!(parser.specifier, "ember-eslint-parser");
        assert_eq!(parser.config_dir, PathBuf::default());

        // Parser must be a string, not an object
        assert!(
            serde_json::from_value::<OxlintLanguageOptions>(serde_json::json!({
                "parser": { "specifier": "ember-eslint-parser" },
            }))
            .is_err()
        );
    }

    #[test]
    fn test_serialize() {
        let language_options = OxlintLanguageOptions {
            parser: Some(ExternalParserEntry {
                config_dir: PathBuf::default(),
                specifier: "ember-eslint-parser".to_string(),
            }),
            parser_options: None,
        };
        let json = serde_json::to_value(&language_options).unwrap();
        assert_eq!(json, serde_json::json!({ "parser": "ember-eslint-parser" }));
    }
}
