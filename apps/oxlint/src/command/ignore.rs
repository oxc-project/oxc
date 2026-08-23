// `usage_rs::Args` generates public partial structs with underscore-prefixed fields.
#![allow(clippy::allow_attributes, clippy::pub_underscore_fields)]

use std::ffi::OsString;

use usage_rs::Args;

/// Ignore Files
#[derive(Debug, Clone, Args)]
pub struct IgnoreOptions {
    /// Specify the file to use as your `.eslintignore`
    #[usage(long, value_name = "PATH", default = ".eslintignore")]
    pub ignore_path: OsString,

    /// Specify patterns of files to ignore (in addition to those in `.eslintignore`)
    ///
    /// The supported syntax is the same as for `.eslintignore` and `.gitignore` files.
    /// You should quote your patterns in order to avoid shell interpretation of glob patterns.
    #[usage(long, value_name = "PAT")]
    pub ignore_pattern: Vec<String>,

    #[usage(
        long,
        help = "Disable excluding files from `.eslintignore` files, `--ignore-path` flags and `--ignore-pattern` flags"
    )]
    pub no_ignore: bool,
}

#[cfg(test)]
mod ignore_options {
    use std::{ffi::OsString, path::PathBuf};

    use super::{super::lint::LintCommand, IgnoreOptions};

    fn get_ignore_options(arg: &str) -> IgnoreOptions {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        LintCommand::parse_from(args.as_slice()).unwrap().ignore_options
    }

    #[test]
    fn default() {
        let options = get_ignore_options(".");
        assert_eq!(options.ignore_path, OsString::from(".eslintignore"));
        assert!(!options.no_ignore);
        assert!(options.ignore_pattern.is_empty());
    }

    #[test]
    fn ignore_path() {
        let options = get_ignore_options("--ignore-path .xxx foo.js");
        assert_eq!(options.ignore_path, PathBuf::from(".xxx"));
    }

    #[test]
    fn no_ignore() {
        let options = get_ignore_options("--no-ignore foo.js");
        assert!(options.no_ignore);
    }

    #[test]
    fn single_ignore_pattern() {
        let options = get_ignore_options("--ignore-pattern ./test foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test")]);
    }

    #[test]
    fn multiple_ignore_pattern() {
        let options = get_ignore_options("--ignore-pattern ./test --ignore-pattern bar.js foo.js");
        assert_eq!(options.ignore_pattern, vec![String::from("./test"), String::from("bar.js")]);
    }
}
