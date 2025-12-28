use std::path::{Path, PathBuf};

use oxc_formatter::{FormatOptions, oxfmtrc::Oxfmtrc};
use serde_json::Value;

pub enum PrettierFileStrategy {
    External { parser_name: &'static str },
}

pub fn detect_prettier_file(path: &Path) -> Option<PrettierFileStrategy> {
    let extension = path.extension()?.to_str()?;

    let parser_name = match extension {
        "json" => "json",
        "jsonc" => "jsonc",
        "css" => "css",
        "md" => "markdown",
        _ => return None,
    };

    Some(PrettierFileStrategy::External { parser_name })
}

pub fn load_oxfmtrc(root: &Path) -> Result<(FormatOptions, Value), String> {
    let config_path = find_oxfmtrc(root);

    let json_string = match &config_path {
        Some(path) => {
            let mut json_string = std::fs::read_to_string(path)
                // Do not include OS error, it differs between platforms
                .map_err(|_| format!("Failed to read config {}: File not found", path.display()))?;
            json_strip_comments::strip(&mut json_string)
                .map_err(|err| format!("Failed to strip comments from {}: {err}", path.display()))?;
            json_string
        }
        None => "{}".to_string(),
    };

    let raw_config: Value = serde_json::from_str(&json_string)
        .map_err(|err| format!("Failed to parse config: {err}"))?;

    let oxfmtrc: Oxfmtrc = serde_json::from_value(raw_config.clone())
        .map_err(|err| format!("Failed to deserialize Oxfmtrc: {err}"))?;

    let (format_options, _) = oxfmtrc
        .into_options()
        .map_err(|err| format!("Failed to parse configuration.\n{err}"))?;

    let mut external_options = raw_config;
    Oxfmtrc::populate_prettier_config(&format_options, &mut external_options);

    Ok((format_options, external_options))
}

fn find_oxfmtrc(root: &Path) -> Option<PathBuf> {
    root.ancestors().find_map(|dir| {
        let json_path = dir.join(".oxfmtrc.json");
        if json_path.exists() {
            return Some(json_path);
        }
        let jsonc_path = dir.join(".oxfmtrc.jsonc");
        if jsonc_path.exists() {
            return Some(jsonc_path);
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use serde_json::Value;
    use tempfile::tempdir;

    use super::{detect_prettier_file, load_oxfmtrc};

    #[test]
    fn detect_prettier_file_extensions() {
        let cases = [
            ("file.json", "json"),
            ("file.jsonc", "jsonc"),
            ("file.css", "css"),
            ("file.md", "markdown"),
        ];

        for (path, parser_name) in cases {
            let strategy = detect_prettier_file(Path::new(path)).expect("expected strategy");
            match strategy {
                super::PrettierFileStrategy::External { parser_name: name } => {
                    assert_eq!(name, parser_name);
                }
            }
        }

        assert!(detect_prettier_file(Path::new("file.ts")).is_none());
    }

    #[test]
    fn load_oxfmtrc_defaults_when_missing() {
        let dir = tempdir().expect("tempdir");
        let result = load_oxfmtrc(dir.path());
        assert!(result.is_ok());
        let (_, external_options) = result.unwrap();
        assert!(external_options.is_object());
    }

    #[test]
    fn load_oxfmtrc_jsonc_with_comments() {
        let dir = tempdir().expect("tempdir");
        let config_path = dir.path().join(".oxfmtrc.jsonc");
        fs::write(
            &config_path,
            "{\n// comment\n\"printWidth\": 120\n}\n",
        )
        .expect("write config");

        let (_, external_options) = load_oxfmtrc(dir.path()).expect("load config");
        assert_eq!(
            external_options.get("printWidth").and_then(Value::as_u64),
            Some(120)
        );
    }
}
