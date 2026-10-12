use std::{path::PathBuf, str::FromStr};

use bpaf::Bpaf;

/// Caching
#[derive(Debug, Clone, Bpaf)]
pub struct CacheOptions {
    /// Only check changed files.
    ///
    /// Results are stored in `.oxlintcache` (see `--cache-location`) and reused while a file and
    /// the configuration are unchanged. Like ESLint, dependencies between files are not tracked,
    /// so results of rules that depend on other files (type-aware rules, `--type-check`, and
    /// cross-module rules of the `import` plugin) can be stale.
    /// Running without `--cache` deletes the cache file.
    #[bpaf(switch, hide_usage)]
    pub cache: bool,

    /// Path to the cache file or directory. Defaults to `.oxlintcache`.
    ///
    /// If a directory is given, the cache file is created inside it with a name derived from the
    /// current working directory.
    #[bpaf(argument("PATH"), optional, hide_usage)]
    pub cache_location: Option<PathBuf>,

    /// Strategy for detecting changed files. Possible values: `metadata`, `content`
    ///
    /// `metadata` compares file size and modification time. `content` compares a hash of the
    /// file contents, which survives tools that touch files, such as `git clone`.
    #[bpaf(argument("STRATEGY"), fallback(CacheStrategy::Metadata), hide_usage)]
    pub cache_strategy: CacheStrategy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CacheStrategy {
    #[default]
    Metadata,
    Content,
}

impl FromStr for CacheStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "metadata" => Ok(Self::Metadata),
            "content" => Ok(Self::Content),
            _ => Err(format!("'{s}' is not a known cache strategy")),
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::CacheStrategy;
    use crate::cli::{LintCommand, lint_command};

    fn get_lint_options(arg: &str) -> LintCommand {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice()).unwrap()
    }

    #[test]
    fn default() {
        let options = get_lint_options(".");
        assert!(!options.cache_options.cache);
        assert_eq!(options.cache_options.cache_location, None);
        assert_eq!(options.cache_options.cache_strategy, CacheStrategy::Metadata);
    }

    #[test]
    fn cache() {
        let options = get_lint_options("--cache .");
        assert!(options.cache_options.cache);
        assert_eq!(options.paths, vec![PathBuf::from(".")]);
    }

    #[test]
    fn cache_location() {
        let options = get_lint_options("--cache --cache-location foo/.cache .");
        assert_eq!(options.cache_options.cache_location, Some(PathBuf::from("foo/.cache")));
    }

    #[test]
    fn cache_strategy() {
        let options = get_lint_options("--cache --cache-strategy content .");
        assert_eq!(options.cache_options.cache_strategy, CacheStrategy::Content);

        let args = ["--cache-strategy", "mtime", "."].map(String::from);
        assert!(lint_command().run_inner(args.as_slice()).is_err());
    }
}
