pub(crate) mod command;
mod format;
mod init;
mod reporter;
mod result;
mod service;
mod walk;

pub(crate) const DEFAULT_CONFIG_NOTE_MESSAGE: &str =
    "Using Oxfmt's default configuration. Run `oxfmt --init` to set up Oxfmt in this project.\n";

pub use crate::core::utils::init_tracing;
#[cfg(feature = "napi")]
pub use command::MigrateSource;
pub use command::{FormatCommand, Mode, format_command};
pub use format::FormatRunner;
pub use init::{init_miette, init_rayon};
pub use result::CliRunResult;
