use std::path::PathBuf;

use glob::Pattern;

use crate::command::LintCommand;

#[derive(Debug)]
pub struct CliOptions {
    pub quiet: bool,
    pub fix: bool,
    pub max_warnings: Option<usize>,
    pub paths: Vec<PathBuf>,
    pub ignore_path: String,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<Pattern>,
}

impl<'a> TryFrom<&'a LintCommand> for CliOptions {
    type Error = &'a str;

    fn try_from(matches: &LintCommand) -> Result<Self, Self::Error> {
        let mut paths = vec![];

        for path in &matches.paths {
            let glob_result =
                glob::glob(&path.to_string_lossy()).map_err(|_| "Failed to read glob pattern")?;
            let globbed = glob_result
                .map(|path_result| path_result.map_err(|_| "Failed to read path"))
                .collect::<Result<Vec<PathBuf>, &str>>()?;

            if globbed.is_empty() && path.canonicalize().is_err() {
                return Err("Unable to find globbed files");
            }

            paths.extend(globbed);
        }

        let result = Self {
            quiet: matches.quiet,
            fix: matches.fix,
            max_warnings: matches.max_warnings,
            paths,
            ignore_path: matches.ignore_path.clone(),
            no_ignore: matches.no_ignore,
            ignore_pattern: matches.ignore_pattern.clone(),
        };

        Ok(result)
    }
}
