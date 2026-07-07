use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Language options for files matched by an override.
///
/// This is a deliberately small subset of ESLint's `languageOptions`: only `parser` and
/// `parserOptions` are recognized. It exists to route files that oxlint's native parser
/// cannot parse (e.g. Ember's `.gjs`/`.gts` files) to an external (JS) parser.
///
/// Other ESLint `languageOptions` keys (`globals`, `env`, `ecmaVersion`, `sourceType`) are
/// rejected with an actionable config error rather than silently ignored; oxlint's top-level
/// `globals`/`env` settings are the equivalents.
///
/// Note: External parsers are only supported when running oxlint via the CLI (Node.js),
/// same as `jsPlugins`.
#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OxlintLanguageOptions {
    /// External parser for files matched by this override.
    ///
    /// A module specifier (path, package name, or URL), resolved the same way as `jsPlugins`
    /// entries. The module must export an ESLint-compatible parser
    /// (an object with a `parseForESLint` or `parse` method).
    ///
    /// The bare string form is intentional and forward-compatible: a future object form or a
    /// separate `languagePlugins` key (per RFC #21936) can be added alongside it without a
    /// breaking change, mirroring ESLint's own `languageOptions.parser` / `language` split.
    ///
    /// Files matched by an override with a `parser` are parsed by that parser instead of
    /// oxlint's native parser. Only JS plugin rules run on such files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser: Option<ExternalParserEntry>,

    /// Options passed verbatim to the external `parser`.
    ///
    /// Only takes effect when `parser` is also set. Unlike ESLint (which applies
    /// `parserOptions` to its built-in parser too), oxlint's native parser does not read them,
    /// so `parserOptions` without `parser` is rejected as a config error rather than silently
    /// having no effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parser_options: Option<serde_json::Value>,
}

impl<'de> Deserialize<'de> for OxlintLanguageOptions {
    /// Deserializes `languageOptions`, rejecting unsupported keys with actionable errors.
    ///
    /// # Errors
    /// Returns an error when an unsupported ESLint `languageOptions` key is present (pointing
    /// to oxlint's equivalent where one exists), or when `parserOptions` is set without a
    /// `parser` (which would silently have no effect, since oxlint's native parser ignores it).
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        // Deserialize permissively into a raw shape so unsupported keys can be reported with a
        // tailored message instead of serde's bare "unknown field" error. `deny_unknown_fields`
        // on the struct above is kept purely so the generated JSON schema stays strict.
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            #[serde(default)]
            parser: Option<ExternalParserEntry>,
            #[serde(default)]
            parser_options: Option<serde_json::Value>,
            #[serde(flatten)]
            rest: serde_json::Map<String, serde_json::Value>,
        }

        let raw = Raw::deserialize(deserializer)?;

        if let Some(unsupported) = raw.rest.keys().next() {
            let hint = match unsupported.as_str() {
                "globals" => {
                    ": use oxlint's top-level `globals` (or `env`) setting instead".to_string()
                }
                "env" => ": use oxlint's top-level `env` setting instead".to_string(),
                "ecmaVersion" => {
                    ": oxlint infers the ECMAScript version automatically".to_string()
                }
                "sourceType" => {
                    ": oxlint infers the module kind from the file extension and config".to_string()
                }
                _ => ", expected `parser` or `parserOptions`".to_string(),
            };
            return Err(D::Error::custom(format!(
                "`languageOptions.{unsupported}` is not supported by oxlint{hint}"
            )));
        }

        if raw.parser.is_none() && raw.parser_options.is_some() {
            return Err(D::Error::custom(
                "`languageOptions.parserOptions` has no effect without `languageOptions.parser`: \
                 oxlint applies `parserOptions` only to a custom `parser`, unlike ESLint, which \
                 also applies them to its built-in parser",
            ));
        }

        Ok(OxlintLanguageOptions { parser: raw.parser, parser_options: raw.parser_options })
    }
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
    fn test_deserialize_unsupported_keys() {
        // Unsupported ESLint keys are rejected with an actionable message pointing at the
        // oxlint equivalent, rather than a bare serde "unknown field" error.
        for (key, hint) in [
            ("globals", "top-level `globals`"),
            ("env", "top-level `env`"),
            ("ecmaVersion", "ECMAScript version"),
            ("sourceType", "module kind"),
        ] {
            let err = serde_json::from_value::<OxlintLanguageOptions>(serde_json::json!({
                key: serde_json::Value::Bool(true),
            }))
            .unwrap_err()
            .to_string();
            assert!(err.contains(&format!("`languageOptions.{key}`")), "{err}");
            assert!(err.contains(hint), "{err}");
        }

        // Truly unknown keys still error, listing the supported fields.
        let err = serde_json::from_value::<OxlintLanguageOptions>(serde_json::json!({
            "nonsense": true,
        }))
        .unwrap_err()
        .to_string();
        assert!(err.contains("expected `parser` or `parserOptions`"), "{err}");
    }

    #[test]
    fn test_deserialize_parser_options_without_parser() {
        // `parserOptions` alone would silently have no effect (oxlint's native parser ignores
        // it), so it is rejected rather than accepted as inert config.
        let err = serde_json::from_value::<OxlintLanguageOptions>(serde_json::json!({
            "parserOptions": { "project": true },
        }))
        .unwrap_err()
        .to_string();
        assert!(err.contains("has no effect without `languageOptions.parser`"), "{err}");

        // `parserOptions` with `parser` is accepted.
        let language_options: OxlintLanguageOptions = serde_json::from_value(serde_json::json!({
            "parser": "ember-eslint-parser",
            "parserOptions": { "project": true },
        }))
        .unwrap();
        assert_eq!(language_options.parser.unwrap().specifier, "ember-eslint-parser");
        assert_eq!(language_options.parser_options, Some(serde_json::json!({ "project": true })));
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
