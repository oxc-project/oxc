/// Add header to YAML.
pub fn print_yaml(code: &str, generator_path: &str) -> String {
    let header = generate_header(generator_path);
    format!("{header}{code}")
}

/// Creates a generated file warning + required information for a generated file.
fn generate_header(generator_path: &str) -> String {
    let generator_path = generator_path.replace('\\', "/");

    // TODO: Add generation date, AST source hash, etc here.
    format!(
        "# Auto-generated code, DO NOT EDIT DIRECTLY!\n\
        # To edit this generated file you have to edit `{generator_path}`\n\n"
    )
}
