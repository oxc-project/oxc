mod options;
mod server_formatter;
#[cfg(test)]
mod tester;

pub use server_formatter::{ServerFormatter, ServerFormatterBuilder};

const FORMAT_CONFIG_FILES: &[&str; 2] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];
