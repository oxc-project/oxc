use schemars::{JsonSchema, schema_for};
use serde_json::Value;

/// Generates the JSON schema for a configuration type.
///
/// This function:
/// 1. Generates the schema using `schemars`
/// 2. Adds `allowComments` and `allowTrailingCommas` for vscode-json-languageservice
/// 3. Injects `markdownDescription` fields for better editor support
///
/// # Panics
/// Panics if the schema generation fails.
pub fn generate_schema_json<T: JsonSchema>() -> String {
    let mut schema = schema_for!(T);

    // Allow comments and trailing commas for vscode-json-languageservice
    // NOTE: This is NOT part of standard JSON Schema specification
    // https://github.com/microsoft/vscode-json-languageservice/blob/fb83547762901f32d8449d57e24666573016b10c/src/jsonLanguageTypes.ts#L151-L159
    schema.schema.extensions.insert("allowComments".to_string(), Value::Bool(true));
    schema.schema.extensions.insert("allowTrailingCommas".to_string(), Value::Bool(true));

    // Inject markdownDescription fields for better editor support (e.g., VS Code)
    let mut json = serde_json::to_value(&schema).unwrap();
    inject_markdown_descriptions(&mut json);

    serde_json::to_string_pretty(&json).unwrap()
}

/// Recursively inject `markdownDescription` fields into the JSON schema.
/// This is a non-standard field that some editors (like VS Code) use to render
/// markdown in hover tooltips.
fn inject_markdown_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // If this object has a `description` field, copy it to `markdownDescription`
            if let Some(Value::String(desc_str)) = map.get("description") {
                map.insert("markdownDescription".to_string(), Value::String(desc_str.clone()));
            }

            // Recursively process all values in the object
            for value in map.values_mut() {
                inject_markdown_descriptions(value);
            }
        }
        Value::Array(items) => {
            // Recursively process all items in the array
            for item in items {
                inject_markdown_descriptions(item);
            }
        }
        _ => {
            // Primitive values don't need processing
        }
    }
}
