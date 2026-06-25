use std::path::Path;

use oxc::transformer::BabelOptions;
use serde::Deserialize;

use crate::workspace_root;

/// output.json
#[derive(Debug, Default, Clone, Deserialize)]
pub struct BabelOutput {
    pub errors: Option<Vec<String>>,
}

fn read_output_json(path: &Path) -> Option<BabelOutput> {
    let dir = workspace_root().join(path);
    if let Some(json) = read_file::<BabelOutput>(&dir, "output.json") {
        return Some(json);
    }
    read_file::<BabelOutput>(&dir, "output.extended.json")
}

fn read_file<T: serde::de::DeserializeOwned>(path: &Path, file_name: &'static str) -> Option<T> {
    let file = path.with_file_name(file_name);
    if file.exists() {
        let file = std::fs::File::open(file).ok()?;
        let reader = std::io::BufReader::new(file);
        return serde_json::from_reader(reader).ok();
    }
    None
}

/// Determine if test should fail based on output.json and options.json
pub fn determine_should_fail(path: &Path, options: &BabelOptions) -> bool {
    let output_json = read_output_json(path);

    if let Some(output_json) = output_json {
        return output_json.errors.is_some_and(|errors| !errors.is_empty());
    }

    if options.throws.is_some() {
        return true;
    }

    // both files don't exist
    true
}
