mod default;
mod json;

use std::io::Write;
use std::str::FromStr;

use crate::output_formatter::{default::DefaultOutputFormatter, json::JsonOutputFormatter};

pub struct OutputFormatter {
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputFormat {
    Default,
    /// GitHub Check Annotation
    /// <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-a-notice-message>
    Github,
    Json,
    Unix,
    Checkstyle,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "default" => Ok(Self::Default),
            "unix" => Ok(Self::Unix),
            "checkstyle" => Ok(Self::Checkstyle),
            "github" => Ok(Self::Github),
            _ => Err(format!("'{s}' is not a known format")),
        }
    }
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }
    // print all rules which are currently supported by oxlint
    pub fn all_rules<T: Write>(&self, writer: &mut T) {
        match self.format {
            OutputFormat::Json => JsonOutputFormatter::all_rules(writer),
            _ => DefaultOutputFormatter::all_rules(writer),
        }
    }
}
