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
            let globbed = glob::glob(&path.to_string_lossy())
                .expect("Failed to read glob pattern")
                .map(|path| path.expect("Failed to read glob pattern"))
                .collect::<Vec<_>>();

            if globbed.is_empty() && path.canonicalize().is_err() {
                return Err("Unable to find globbed files");
            }
            paths.extend(globbed);
        }

        Ok(Self { quiet: matches.get_one::<bool>("quiet").is_some(), paths })
    }
}
