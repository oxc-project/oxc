use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::add_header;

/// Format Javascript/Typescript code, and add header.
pub fn print_javascript(code: &str, generator_path: &str) -> String {
    let code = add_header(code, generator_path, "//");
    format(&code)
}

/// Format JS/TS code with `dprint`.
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
