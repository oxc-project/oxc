use clap::{builder::ValueParser, Arg, ArgAction, Command};

pub fn lint_command() -> Command {
    Command::new("lint")
            .alias("check")
            .about("Lint this repository.")
            .arg_required_else_help(true)
            .arg(
                Arg::new("path")
                    .value_name("PATH")
                    .num_args(1..)
                    .required(true)
                    .value_parser(ValueParser::path_buf())
                    .help("File or Directory paths to scan. Directories are scanned recursively.")
            )
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
                .value_parser(ValueParser::path_buf())
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
                  .default_value(None)
                  .required(false)
                  .help("This option allows you to specify a warning threshold, which can be used to force oxc_lint to exit with an error status if there are too many warning-level rule violations in your project.")
              )
}
