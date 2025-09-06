use std::{env, ffi::OsStr, io::Write, path::PathBuf, sync::Arc};

use ignore::overrides::OverrideBuilder;

use oxc_diagnostics::DiagnosticService;

use crate::{
    cli::{CliRunResult, FormatCommand},
    reporter::DefaultReporter,
    service::FormatService,
    walk::Walk,
};

#[derive(Debug)]
pub struct FormatRunner {
    options: FormatCommand,
    cwd: PathBuf,
}

impl FormatRunner {
    pub(crate) fn new(options: FormatCommand) -> Self {
        Self { options, cwd: env::current_dir().expect("Failed to get current working directory") }
    }

    pub(crate) fn run(self, stdout: &mut dyn Write) -> CliRunResult {
        let cwd = self.cwd;
        let FormatCommand { paths, output_options, .. } = self.options;

        let (exclude_patterns, regular_paths): (Vec<_>, Vec<_>) =
            paths.into_iter().partition(|p| p.to_string_lossy().starts_with('!'));

        // Need at least one regular path
        if regular_paths.is_empty() {
            print_and_flush_stdout(
                stdout,
                "Expected at least one target file/dir/glob(non-override pattern)\n",
            );
            return CliRunResult::FormatNoFilesFound;
        }

        // Build exclude patterns if any exist
        let override_builder = (!exclude_patterns.is_empty())
            .then(|| {
                let mut builder = OverrideBuilder::new(&cwd);
                for pattern in exclude_patterns {
                    builder.add(&pattern.to_string_lossy()).ok()?;
                }
                builder.build().ok()
            })
            .flatten();

        let walker = Walk::new(&regular_paths, override_builder);
        let paths = walker.paths();

        let files_to_format = paths
            .into_iter()
            // .filter(|path| !config_store.should_ignore(Path::new(path)))
            .collect::<Vec<Arc<OsStr>>>();

        if files_to_format.is_empty() {
            print_and_flush_stdout(stdout, "Expected at least one target file\n");
            return CliRunResult::FormatNoFilesFound;
        }

        let reporter = Box::new(DefaultReporter::default());
        let (mut diagnostic_service, tx_error) = DiagnosticService::new(reporter);

        rayon::spawn(move || {
            let mut format_service = FormatService::new(cwd, &output_options);
            format_service.with_paths(files_to_format);
            format_service.run(&tx_error);
        });
        // NOTE: This is a blocking
        let res = diagnostic_service.run(stdout);

        print_and_flush_stdout(stdout, &format!("{res:?}"));

        CliRunResult::FormatSucceeded
    }
}

pub fn print_and_flush_stdout(stdout: &mut dyn Write, message: &str) {
    use std::io::{Error, ErrorKind};
    fn check_for_writer_error(error: Error) -> Result<(), Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
            Ok(())
        } else {
            Err(error)
        }
    }

    stdout.write_all(message.as_bytes()).or_else(check_for_writer_error).unwrap();
    stdout.flush().unwrap();
}
