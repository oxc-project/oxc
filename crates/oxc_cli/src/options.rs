use std::path::PathBuf;

use clap::ArgMatches;
use glob::Pattern;

pub struct CliOptions {
    pub quiet: bool,
    pub paths: Vec<PathBuf>,
    pub ignore_path: String,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<Pattern>,
}

impl<'a> TryFrom<&'a ArgMatches> for CliOptions {
    type Error = &'a str;

    fn try_from(matches: &ArgMatches) -> Result<Self, Self::Error> {
        let mut paths = vec![];

        for path in matches.get_many::<PathBuf>("path").unwrap() {
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

        let ignore_path = get_ignore_path(matches);
        let no_ignore = matches.get_flag("no-ignore");
        let ignore_pattern = get_ignore_pattern(matches);

        Ok(Self { quiet: matches.get_flag("quiet"), paths, ignore_path, no_ignore, ignore_pattern })
    }
}

fn get_ignore_path(matches: &ArgMatches) -> String {
    matches.get_one::<String>("ignore-path").map_or(".eslintignore".to_string(), ToOwned::to_owned)
}

fn get_ignore_pattern(matches: &ArgMatches) -> Vec<Pattern> {
    let mut result = vec![];
    let Some(ignore_pattern) = matches.get_many::<String>("ignore-pattern") else {return result};
    for pattern in ignore_pattern {
        if let Ok(pattern) = Pattern::new(pattern) {
            result.push(pattern);
        }
    }

    result
}
