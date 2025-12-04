use std::path::PathBuf;

use bpaf::Bpaf;

const VERSION: &str = match option_env!("OXC_VERSION") {
    Some(v) => v,
    None => "dev",
};

#[expect(clippy::ptr_arg)]
fn validate_paths(paths: &Vec<PathBuf>) -> bool {
    if paths.is_empty() {
        true
    } else {
        paths.iter().all(|p| p.components().all(|c| c != std::path::Component::ParentDir))
    }
}

const PATHS_ERROR_MESSAGE: &str = "PATH must not contain \"..\"";

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version(VERSION))]
pub struct FormatCommand {
    #[bpaf(external, fallback(OutputOptions::Write), hide_usage)]
    pub output_options: OutputOptions,
    #[bpaf(external)]
    pub basic_options: BasicOptions,
    #[bpaf(external)]
    pub ignore_options: IgnoreOptions,
    #[bpaf(external)]
    pub misc_options: MiscOptions,
    /// Single file, single path or list of paths.
    /// If not provided, current working directory is used.
    /// Glob is supported only for exclude patterns like `'!**/fixtures/*.js'`.
    // `bpaf(fallback)` seems to have issues with `many` or `positional`,
    // so we implement the fallback behavior in code instead.
    #[bpaf(positional("PATH"), many, guard(validate_paths, PATHS_ERROR_MESSAGE))]
    pub paths: Vec<PathBuf>,
}

/// Output Options
#[derive(Debug, Clone, Bpaf)]
pub enum OutputOptions {
    /// Default - when no output option is specified, behaves like `--write` mode in Prettier
    #[bpaf(hide)]
    Write,
    /// Check mode - check if files are formatted, also show statistics
    #[bpaf(long)]
    Check,
    /// List mode - list files that would be changed
    #[bpaf(long)]
    ListDifferent,
}

/// Basic Options
#[derive(Debug, Clone, Bpaf)]
pub struct BasicOptions {
    /// Path to the configuration file
    #[bpaf(short, long, argument("PATH"))]
    pub config: Option<PathBuf>,
}

/// Ignore Options
#[derive(Debug, Clone, Bpaf)]
pub struct IgnoreOptions {
    /// Path to ignore file(s). Can be specified multiple times.
    /// If not specified, .gitignore and .prettierignore in the current directory are used.
    #[bpaf(argument("PATH"), many, hide_usage)]
    pub ignore_path: Vec<PathBuf>,
    /// Format code in node_modules directory (skipped by default)
    #[bpaf(switch, hide_usage)]
    pub with_node_modules: bool,
}

/// Misc Options
#[derive(Debug, Clone, Bpaf)]
pub struct MiscOptions {
    /// Start language server protocol (LSP) server
    #[bpaf(switch, hide_usage)]
    pub lsp: bool,
    /// Do not exit with error when pattern is unmatched
    #[bpaf(switch, hide_usage)]
    pub no_error_on_unmatched_pattern: bool,
    /// Number of threads to use. Set to 1 for using only 1 CPU core.
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<usize>,
}

impl FormatCommand {
    pub fn handle_threads(&self) {
        Self::init_rayon_thread_pool(self.misc_options.threads);
    }

    /// Initialize Rayon global thread pool with specified number of threads.
    ///
    /// If `--threads` option is not used, or `--threads 0` is given,
    /// default to the number of available CPU cores.
    #[expect(clippy::print_stderr)]
    fn init_rayon_thread_pool(threads: Option<usize>) {
        // Always initialize thread pool, even if using default thread count,
        // to ensure thread pool's thread count is locked after this point.
        // `rayon::current_num_threads()` will always return the same number after this point.
        //
        // If you don't initialize the global thread pool explicitly, or don't specify `num_threads`,
        // Rayon will initialize the thread pool when it's first used, with a thread count of
        // `std::thread::available_parallelism()`, and that thread count won't change thereafter.
        // So we don't *need* to initialize the thread pool here if we just want the default thread count.
        //
        // However, Rayon's docs state that:
        // > In the future, the default behavior may change to dynamically add or remove threads as needed.
        // https://docs.rs/rayon/1.11.0/rayon/struct.ThreadPoolBuilder.html#method.num_threads
        //
        // To ensure we continue to have a "locked" thread count, even after future Rayon upgrades,
        // we always initialize the thread pool and explicitly specify thread count here.

        let thread_count = if let Some(thread_count) = threads
            && thread_count > 0
        {
            thread_count
        } else if let Ok(thread_count) = std::thread::available_parallelism() {
            thread_count.get()
        } else {
            eprintln!(
                "Unable to determine available thread count. Defaulting to 1.\nConsider specifying the number of threads explicitly with `--threads` option."
            );
            1
        };

        rayon::ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();
    }
}
