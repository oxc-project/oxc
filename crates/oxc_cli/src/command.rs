use clap::{builder::ValueParser, Arg, ArgAction, Command};

#[must_use]
pub fn command() -> Command {
    Command::new("oxc")
        .bin_name("oxc")
        .version("alpha")
        .author("Boshen")
        .about("The JavaScript Oxidation Compiler")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(lint_subcommand())
}

fn lint_subcommand() -> Command {
    Command::new("lint")
            .alias("check")
            .about("Lint this repository.")
            .arg_required_else_help(true)
            .arg(
                Arg::new("fix")
                .long("fix")
                .required(false)
                .action(ArgAction::SetTrue)
                .help("This option allows you to enable oxc to fix as many issues as possible. If enabled, only unfixed issues are reported in the output")
            )
            .arg(
              Arg::new("quiet")
                .long("quiet")
                .required(false)
                .action(ArgAction::SetTrue)
                .help("This option allows you to disable reporting on warnings. If you enable this option, only errors are reported by oxc_lint.")
            )
            .arg(
                Arg::new("ignore-path")
                .long("ignore-path")
                .required(false)
                .help("This option allows you to specify the file to use as your .eslintignore.")
            )
            .arg(
                Arg::new("no-ignore")
                .long("no-ignore")
                .required(false)
                .action(ArgAction::SetTrue)
                .help("Disables excluding of files from .eslintignore files, --ignore-path flags, --ignore-pattern flags.")
            )
            .arg(
                Arg::new("ignore-pattern")
                .long("ignore-pattern")
                .required(false)
                .action(ArgAction::Append)
                .help("This option allows you to specify patterns of files to ignore (in addition to those in .eslintignore).")
            )
            .arg(
                Arg::new("max-warnings")
                  .long("max-warnings")
                  .value_parser(clap::value_parser!(usize))
                  .required(false)
                  .help("This option allows you to specify a warning threshold, which can be used to force oxc_lint to exit with an error status if there are too many warning-level rule violations in your project.")
              )
            .arg(
                Arg::new("path")
                    .value_name("PATH")
                    .num_args(1..)
                    .required(true)
                    .help("File or Directory paths to scan. Directories are scanned recursively.")
                    .value_parser(ValueParser::path_buf()),
            )
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use clap::ArgMatches;

    use super::command;

    #[test]
    fn verify_command() {
        command().debug_assert();
    }

    fn get_lint_matches(arg: &str) -> ArgMatches {
        let matches = command().try_get_matches_from(arg.split(' ')).unwrap();
        let matches = matches.subcommand_matches("lint");
        assert!(matches.is_some());
        matches.unwrap().clone()
    }

    #[test]
    fn test_lint_default() {
        let matches = get_lint_matches("oxc lint .");
        assert_eq!(matches.get_one::<PathBuf>("path"), Some(&PathBuf::from(".")));
        assert_eq!(matches.get_flag("fix"), false);
        assert_eq!(matches.get_flag("quiet"), false);
        assert_eq!(matches.get_one::<String>("ignore-path"), None);
        assert_eq!(matches.get_flag("no-ignore"), false);
        assert_eq!(matches.get_one::<String>("ignore-pattern"), None);
        assert_eq!(matches.get_one::<usize>("max-warnings"), None);
    }

    #[test]
    fn test_lint_path() {
        let matches = get_lint_matches("oxc lint .");
        assert_eq!(matches.get_one::<PathBuf>("path"), Some(&PathBuf::from(".")));
    }

    #[test]
    fn test_lint_multiple_paths() {
        let matches = get_lint_matches("oxc lint foo bar baz");
        assert_eq!(
            matches.get_many::<PathBuf>("path").unwrap().collect::<Vec<_>>(),
            [&PathBuf::from("foo"), &PathBuf::from("bar"), &PathBuf::from("baz")]
        );
    }

    #[test]
    fn test_lint_check_path() {
        let matches = get_lint_matches("oxc check /path/to/dir");
        assert_eq!(matches.get_one::<PathBuf>("path"), Some(&PathBuf::from("/path/to/dir")));
    }

    #[test]
    fn test_lint_quiet_true() {
        let matches = get_lint_matches("oxc lint foo.js --quiet");
        assert!(matches.get_flag("quiet"));
    }

    #[test]
    fn test_lint_fix_true() {
        let matches = get_lint_matches("oxc lint foo.js --fix");
        assert!(matches.get_flag("fix"));
    }

    #[test]
    fn test_lint_max_warnings() {
        let matches = get_lint_matches("oxc lint --max-warnings 10 foo.js");
        assert_eq!(matches.get_one::<usize>("max-warnings"), Some(&10));
    }

    #[test]
    fn test_lint_ignore_path() {
        let matches = get_lint_matches("oxc lint --ignore-path .gitignore foo.js");
        assert_eq!(matches.get_one::<String>("ignore-path"), Some(&".gitignore".to_string()));
    }

    #[test]
    fn test_lint_no_ignore() {
        let matches = get_lint_matches("oxc lint --no-ignore foo.js");
        assert!(matches.get_flag("no-ignore"));
    }

    #[test]
    fn test_lint_single_ignore_pattern() {
        let matches = get_lint_matches("oxc lint --ignore-pattern \"./test\" foo.js");
        assert_eq!(matches.get_one::<String>("ignore-pattern"), Some(&"\"./test\"".to_string()));
    }

    #[test]
    fn test_lint_multiple_ignore_pattern() {
        let matches = get_lint_matches(
            "oxc lint --ignore-pattern \"./test\" --ignore-pattern \"bar.js\" foo.js",
        );
        let ignore_pattern = matches.get_many::<String>("ignore-pattern").unwrap();
        let mut compare = vec![];
        for pattern in ignore_pattern {
            compare.push(pattern);
        }
        assert_eq!(compare, vec!["\"./test\"", "\"bar.js\""]);
    }
}
