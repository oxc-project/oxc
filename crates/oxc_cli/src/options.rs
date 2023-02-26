use std::path::PathBuf;

use clap::ArgMatches;

pub struct CliOptions {
    pub quiet: bool,
    pub path: PathBuf,
}

impl<'a> TryFrom<&'a ArgMatches> for CliOptions {
    type Error = &'a str;

    fn try_from(matches: &ArgMatches) -> Result<Self, Self::Error> {
        let path = matches.get_one::<PathBuf>("path").unwrap();
        if path.canonicalize().is_err() {
            return Err("err");
        }

        Ok(Self {
            quiet: matches.get_one::<String>("quiet").is_some(),
            path: matches.get_one::<PathBuf>("path").unwrap().into(),
        })
    }
}
