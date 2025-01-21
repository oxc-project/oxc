mod checkstyle;
mod default;
mod github;
mod json;
mod stylish;
mod unix;

use std::io::{BufWriter, Stdout, Write};
use std::str::FromStr;

use checkstyle::CheckStyleOutputFormatter;
use github::GithubOutputFormatter;
use stylish::StylishOutputFormatter;
use unix::UnixOutputFormatter;

use oxc_diagnostics::reporter::DiagnosticReporter;

use crate::output_formatter::{default::DefaultOutputFormatter, json::JsonOutputFormatter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OutputFormat {
    Default,
    /// GitHub Check Annotation
    /// <https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-a-notice-message>
    Github,
    Json,
    Unix,
    Checkstyle,
    Stylish,
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
            "stylish" => Ok(Self::Stylish),
            _ => Err(format!("'{s}' is not a known format")),
        }
    }
}

trait InternalFormatter {
    fn all_rules(&mut self, writer: &mut dyn Write);

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter>;
}

pub struct OutputFormatter {
    internal_formatter: Box<dyn InternalFormatter>,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { internal_formatter: Self::get_internal_formatter(format) }
    }

    fn get_internal_formatter(format: OutputFormat) -> Box<dyn InternalFormatter> {
        match format {
            OutputFormat::Json => Box::<JsonOutputFormatter>::default(),
            OutputFormat::Checkstyle => Box::<CheckStyleOutputFormatter>::default(),
            OutputFormat::Github => Box::new(GithubOutputFormatter),
            OutputFormat::Unix => Box::<UnixOutputFormatter>::default(),
            OutputFormat::Default => Box::new(DefaultOutputFormatter),
            OutputFormat::Stylish => Box::<StylishOutputFormatter>::default(),
        }
    }

    pub fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        self.internal_formatter.all_rules(writer);
    }

    pub fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        self.internal_formatter.get_diagnostic_reporter()
    }
}
