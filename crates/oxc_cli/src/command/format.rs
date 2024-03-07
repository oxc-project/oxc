use std::path::PathBuf;

use bpaf::Bpaf;

use super::{
    ignore::{ignore_options, IgnoreOptions},
    misc_options, CliCommand, MiscOptions, VERSION,
};

/// Formatter for the JavaScript Oxidation Compiler
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub struct FormatCommand {
    #[bpaf(external(format_options))]
    pub format_options: FormatOptions,
}

impl FormatCommand {
    pub fn handle_threads(&self) {
        CliCommand::set_rayon_threads(self.format_options.misc_options.threads);
    }
}

#[derive(Debug, Clone, Bpaf)]
pub struct FormatOptions {
    #[bpaf(external)]
    pub misc_options: MiscOptions,

    #[bpaf(external)]
    pub ignore_options: IgnoreOptions,

    /// Single file, single path or list of paths
    #[bpaf(positional("PATH"), many)]
    pub paths: Vec<PathBuf>,
}
