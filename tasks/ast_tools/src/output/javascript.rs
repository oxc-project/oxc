use std::{
    io::Write,
    process::{Command, Stdio},
};

/// Format Javascript/Typescript code, and add header.
pub fn print_javascript(code: &str, generator_path: &str) -> String {
    let header = generate_header(generator_path);
    let code = format!("{header}{code}");
    format(&code)
}

/// Creates a generated file warning + required information for a generated file.
fn generate_header(generator_path: &str) -> String {
    let generator_path = generator_path.replace('\\', "/");

    // TODO: Add generation date, AST source hash, etc here.
    format!(
        "// Auto-generated code, DO NOT EDIT DIRECTLY!\n\
        // To edit this generated file you have to edit `{generator_path}`\n\n"
    )
}

/// Format JS/TS code with dprint.
fn format(source_text: &str) -> String {
    let mut dprint = Command::new("dprint")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["fmt", "--stdin", "dummy.ts"])
        .spawn()
        .expect("Failed to run dprint (is it installed?)");

    let stdin = dprint.stdin.as_mut().unwrap();
    stdin.write_all(source_text.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let output = dprint.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}
