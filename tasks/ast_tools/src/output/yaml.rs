use super::add_header;

/// Add header to YAML.
pub fn print_yaml(code: &str, generator_path: &str) -> String {
    add_header(code, generator_path, "#")
}
