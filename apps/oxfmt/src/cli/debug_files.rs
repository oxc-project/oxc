use std::{io::Write, path::Path};

use cow_utils::CowUtils;

use super::result::CliRunResult;
use crate::core::utils::print_and_flush;

pub fn run_debug_files(
    files_to_format: impl IntoIterator<Item = impl AsRef<Path>>,
    cwd: &Path,
    stdout: &mut dyn Write,
) -> CliRunResult {
    let mut files = files_to_format.into_iter().collect::<Vec<_>>();
    files.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));

    let mut output = String::new();
    for file in files {
        push_cwd_relative_path(&mut output, file.as_ref(), cwd);
        output.push('\n');
    }

    if !output.is_empty() {
        print_and_flush(stdout, &output);
    }

    CliRunResult::FormatSucceeded
}

fn push_cwd_relative_path(output: &mut String, path: &Path, cwd: &Path) {
    let path = path.strip_prefix(cwd).unwrap_or(path);
    output.push_str(&path.to_string_lossy().cow_replace('\\', "/"));
}
