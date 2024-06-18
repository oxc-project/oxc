use schemars::JsonSchema;
use serde::Deserialize;

// <https://typescript-eslint.io/packages/parser/#configuration>
#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default)]
pub struct OxlintParseOptions {
    /// This option allow you to tell parser to act as if `emitDecoratorMetadata: true` is set in `tsconfig.json`.
    #[serde(rename = "emitDecoratorMetadata")]
    pub emit_decorator_metadata: bool,

    /// This option allow you to tell parser to act as if `experimentalDecorators: true` is set in `tsconfig.json`.
    #[serde(rename = "experimentalDecorators")]
    pub experimental_decorators: bool,
}

#[cfg(test)]
mod test {
    use super::OxlintParseOptions;
    use serde::Deserialize;

    #[test]
    fn test_parse_options() {
        let options = OxlintParseOptions::deserialize(&serde_json::json!({
            "emitDecoratorMetadata": true,
            "experimentalDecorators": true
        }))
        .unwrap();
        assert!(options.emit_decorator_metadata);
        assert!(options.experimental_decorators);
    }

    #[test]
    fn test_parse_options_default() {
        let options = OxlintParseOptions::default();
        assert!(!options.emit_decorator_metadata);
        assert!(!options.experimental_decorators);
    }

    #[test]
    fn test_one_lack_field() {
        let options = OxlintParseOptions::deserialize(&serde_json::json!({
            "emitDecoratorMetadata": true
        }))
        .unwrap();
        assert!(options.emit_decorator_metadata);
        assert!(!options.experimental_decorators);
    }
}
