#![expect(clippy::print_stdout)]

use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct PackageJson {
    contributes: Contributes,
}

#[derive(Debug, Deserialize)]
struct Contributes {
    configuration: Configuration,
}

#[derive(Debug, Deserialize)]
struct Configuration {
    properties: serde_json::Map<String, Value>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: vscode_docs <check|update>");
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "check" => check_readme(),
        "update" => update_readme(),
        _ => {
            eprintln!("Unknown command: {}. Use 'check' or 'update'", command);
            std::process::exit(1);
        }
    }
}

fn check_readme() {
    let generated_content = generate_configuration_docs();
    let readme_path = get_readme_path();
    let readme_content = fs::read_to_string(&readme_path).expect("Failed to read README.md");

    let expected_content = replace_generated_section(&readme_content, &generated_content);

    if readme_content != expected_content {
        eprintln!("❌ README.md is out of date!");
        eprintln!("Run 'cargo run -p vscode_docs update' to update it.");
        std::process::exit(1);
    }

    println!("✅ README.md is up to date!");
}

fn update_readme() {
    let generated_content = generate_configuration_docs();
    let readme_path = get_readme_path();
    let readme_content = fs::read_to_string(&readme_path).expect("Failed to read README.md");

    let updated_content = replace_generated_section(&readme_content, &generated_content);

    fs::write(&readme_path, updated_content).expect("Failed to write README.md");

    println!("✅ README.md updated successfully!");
}

fn get_readme_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("../../editors/vscode/README.md")
}

fn get_package_json_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("../../editors/vscode/package.json")
}

fn replace_generated_section(readme: &str, generated: &str) -> String {
    const START_MARKER: &str = "<!-- START_GENERATED_CONFIGURATION -->";
    const END_MARKER: &str = "<!-- END_GENERATED_CONFIGURATION -->";

    if let Some(start_idx) = readme.find(START_MARKER) {
        if let Some(end_idx) = readme.find(END_MARKER) {
            let before = &readme[..start_idx + START_MARKER.len()];
            let after = &readme[end_idx..];
            return format!("{}\n{}\n{}", before, generated, after);
        }
    }

    // If markers don't exist, return original content
    readme.to_string()
}

fn generate_configuration_docs() -> String {
    let package_json_path = get_package_json_path();
    let package_json_content =
        fs::read_to_string(&package_json_path).expect("Failed to read package.json");

    let package_json: PackageJson =
        serde_json::from_str(&package_json_content).expect("Failed to parse package.json");

    let mut window_configs = Vec::new();
    let mut workspace_configs = Vec::new();

    // Sort properties by key for consistent output
    let mut properties: Vec<_> = package_json.contributes.configuration.properties.iter().collect();
    properties.sort_by_key(|(k, _)| *k);

    for (key, value) in properties {
        let scope = value.get("scope").and_then(|v| v.as_str()).unwrap_or("resource");

        // Skip deprecated fields (those with "deprecated": true or "markdownDeprecationMessage")
        let is_deprecated = value.get("deprecated").and_then(|v| v.as_bool()).unwrap_or(false)
            || value.get("markdownDeprecationMessage").is_some();

        if is_deprecated {
            continue;
        }

        let default_value = value
            .get("default")
            .map(|v| format_default_value(v))
            .unwrap_or_else(|| "-".to_string());

        let description = value
            .get("markdownDescription")
            .or_else(|| value.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let possible_values = get_possible_values(value);

        let row = ConfigRow {
            key: key.clone(),
            default_value,
            possible_values,
            description: description.to_string(),
        };

        if scope == "window" {
            window_configs.push(row);
        } else {
            workspace_configs.push(row);
        }
    }

    let mut output = String::new();

    output.push_str("\n### Window Configuration\n\n");
    output.push_str("Following configuration are supported via `settings.json` and effect the window editor:\n\n");
    output.push_str(&generate_table(&window_configs));

    output.push_str("\n### Workspace Configuration\n\n");
    output.push_str("Following configuration are supported via `settings.json` and can be changed for each workspace:\n\n");
    output.push_str(&generate_table(&workspace_configs));

    // Add FixKind section if there are any references to it
    let has_fixkind_ref = workspace_configs
        .iter()
        .any(|c| c.key == "oxc.fixKind" || c.description.to_lowercase().contains("fixkind"));

    if has_fixkind_ref {
        output.push_str("\n#### FixKind\n\n");
        output.push_str("- `\"safe_fix\"` (default)\n");
        output.push_str("- `\"safe_fix_or_suggestion\"`\n");
        output.push_str("- `\"dangerous_fix\"`\n");
        output.push_str("- `\"dangerous_fix_or_suggestion\"`\n");
        output.push_str("- `\"none\"`\n");
        output.push_str("- `\"all\"`\n");
    }

    output
}

struct ConfigRow {
    key: String,
    default_value: String,
    possible_values: String,
    description: String,
}

fn generate_table(configs: &[ConfigRow]) -> String {
    let mut output = String::new();

    output.push_str("| Key | Default Value | Possible Values | Description |\n");
    output.push_str("| --- | ------------- | --------------- | ----------- |\n");

    for config in configs {
        let key = format!("`{}`", config.key);
        let default_value = &config.default_value;
        let possible_values = &config.possible_values;
        let description = clean_markdown_for_table(&config.description);

        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            key, default_value, possible_values, description
        ));
    }

    output
}

fn format_default_value(value: &Value) -> String {
    match value {
        Value::Bool(b) => format!("`{}`", b),
        Value::String(s) => format!("`{}`", s),
        Value::Number(n) => format!("`{}`", n),
        Value::Null => "`null`".to_string(),
        Value::Object(_) => "`{}`".to_string(),
        Value::Array(_) => "`[]`".to_string(),
    }
}

fn get_possible_values(value: &Value) -> String {
    if let Some(enum_values) = value.get("enum") {
        if let Some(array) = enum_values.as_array() {
            let values: Vec<String> =
                array.iter().filter_map(|v| v.as_str()).map(|s| format!("`{}`", s)).collect();
            return values.join(" \\| ");
        }
    }

    if let Some(type_value) = value.get("type") {
        match type_value {
            Value::String(s) => {
                if s == "boolean" {
                    return "`true` \\| `false`".to_string();
                } else if s == "string" {
                    return "`<string>`".to_string();
                } else if s == "number" {
                    return "`<number>`".to_string();
                } else if s == "object" {
                    return "`Record<string, string>`".to_string();
                }
            }
            Value::Array(arr) => {
                let types: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| *s != "null")
                    .map(|s| {
                        if s == "string" {
                            "`<string>`".to_string()
                        } else if s == "number" {
                            "`<number>`".to_string()
                        } else {
                            format!("`<{}>`", s)
                        }
                    })
                    .collect();
                if !types.is_empty() {
                    return types.join(" \\| ");
                }
            }
            _ => {}
        }
    }

    "-".to_string()
}

fn clean_markdown_for_table(text: &str) -> String {
    // Remove newlines and extra spaces
    text.lines().map(|line| line.trim()).collect::<Vec<_>>().join(" ").trim().to_string()
}
