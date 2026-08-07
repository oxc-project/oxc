use std::ffi::OsString;

use bpaf::{Bpaf, Parser};

pub use oxc_config::NoIgnoreKinds;

/// What the `cli` kind covers in Oxlint;
/// the rest of the `--no-ignore` help text is assembled by [`oxc_config::no_ignore_kinds`].
const CLI_KIND_HELP: &str = "`.eslintignore` files, `--ignore-path` and `--ignore-pattern` flags (those flags are ignored even when passed)";

/// Tell `oxc_config` about Oxlint specific `cli` kind, and also default kind for bare `--no-ignore`.
/// Bare `--no-ignore` disables every ignore source, as the flag name promises;
/// `--no-ignore=cli` is the escape hatch for the historical bare `--no-ignore` behavior.
fn no_ignore_kinds() -> impl Parser<NoIgnoreKinds> {
    oxc_config::no_ignore_kinds(NoIgnoreKinds::ALL_NAME, CLI_KIND_HELP)
}

/// Ignore Files
#[derive(Debug, Clone, Bpaf)]
pub struct IgnoreOptions {
    /// Specify the file to use as your `.eslintignore`
    #[bpaf(argument("PATH"), fallback(".eslintignore".into()), hide_usage)]
    pub ignore_path: OsString,

    /// Specify patterns of files to ignore (in addition to those in `.eslintignore`)
    ///
    /// The supported syntax is the same as for `.eslintignore` and `.gitignore` files.
    /// You should quote your patterns in order to avoid shell interpretation of glob patterns.
    #[bpaf(argument("PAT"), many, hide_usage)]
    pub ignore_pattern: Vec<String>,

    #[bpaf(external(no_ignore_kinds), hide_usage)]
    pub no_ignore: NoIgnoreKinds,
}

#[cfg(test)]
mod ignore_options {
    use std::{ffi::OsString, path::PathBuf};

    use super::{
        super::lint::{LintCommand, lint_command},
        IgnoreOptions, NoIgnoreKinds,
    };

    fn run(arg: &str) -> Result<LintCommand, bpaf::ParseFailure> {
        let args = arg.split(' ').map(std::string::ToString::to_string).collect::<Vec<_>>();
        lint_command().run_inner(args.as_slice())
    }

    fn get_ignore_options(arg: &str) -> IgnoreOptions {
        run(arg).unwrap().ignore_options
    }

    #[test]
    fn default() {
        let options = get_ignore_options(".");
        assert_eq!(options.ignore_path, OsString::from(".eslintignore"));
        assert_eq!(options.no_ignore, NoIgnoreKinds::NONE);
        assert!(options.ignore_pattern.is_empty());
    }

    #[test]
    fn ignore_path() {
        let options = get_ignore_options("--ignore-path .xxx foo.js");
        assert_eq!(options.ignore_path, PathBuf::from(".xxx"));
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

    #[test]
    fn no_ignore() {
        let options = get_ignore_options("--no-ignore foo.js");
        assert_eq!(options.no_ignore, NoIgnoreKinds::ALL);
    }

    /// The escape hatch for the historical bare `--no-ignore` behavior.
    #[test]
    fn no_ignore_cli() {
        let options = get_ignore_options("--no-ignore=cli foo.js");
        assert_eq!(options.no_ignore, NoIgnoreKinds::CLI);
    }

    /// An attached `=KINDS` value reaches `NoIgnoreKinds::from_str`;
    /// the value combinations themselves are unit-tested in `oxc_config`.
    #[test]
    fn no_ignore_vcs() {
        let options = get_ignore_options("--no-ignore=vcs foo.js");
        assert_eq!(options.no_ignore, NoIgnoreKinds { vcs: true, cli: false, config: false });
    }

    /// The kind-validation error must escape bpaf instead of being swallowed;
    /// see the parser-composition rationale on `oxc_config::no_ignore_kinds`.
    #[test]
    fn no_ignore_unknown_kind() {
        let err = run("--no-ignore=bogus foo.js").unwrap_err().unwrap_stderr();
        assert!(err.contains("'bogus' is not a known ignore kind"), "{err}");
    }

    /// The kind list must be attached with `=`; a space-separated word after
    /// bare `--no-ignore` is a positional path, never a kind list.
    #[test]
    fn no_ignore_space_separated_value_is_a_path() {
        let options = get_ignore_options("--no-ignore vcs");
        assert_eq!(options.no_ignore, NoIgnoreKinds::ALL);
    }

    /// Named option parsing must respect the end-of-options separator.
    #[test]
    fn no_ignore_after_double_dash_is_a_path() {
        let command = run("-- --no-ignore").unwrap();
        assert_eq!(command.ignore_options.no_ignore, NoIgnoreKinds::NONE);
        assert_eq!(command.paths, vec![PathBuf::from("--no-ignore")]);
    }

    /// Keep `--no-ignore` in bpaf's option metadata for typo suggestions and shell completion.
    #[test]
    fn no_ignore_typo_suggestion() {
        let err = run("--no-ignores foo.js").unwrap_err().unwrap_stderr();
        assert!(err.contains("did you mean `--no-ignore`"), "{err}");
    }
}
