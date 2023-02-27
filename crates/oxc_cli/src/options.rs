use std::path::PathBuf;

use clap::ArgMatches;

pub struct CliOptions {
    pub quiet: bool,
    pub paths: Vec<PathBuf>,
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

        Ok(Self { quiet: matches.get_flag("quiet"), paths })
    }
}
