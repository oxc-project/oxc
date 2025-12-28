mod options;
mod external_formatter_bridge;
mod server_formatter;
#[cfg(test)]
mod tester;

pub use external_formatter_bridge::{ExternalFormatterBridge, NoopBridge};
pub use server_formatter::ServerFormatterBuilder;

const FORMAT_CONFIG_FILES: &[&str; 2] = &[".oxfmtrc.json", ".oxfmtrc.jsonc"];
