use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc},
    time::Instant,
};

use cow_utils::CowUtils;
use rayon::prelude::*;

use oxc_diagnostics::{DiagnosticSender, DiagnosticService};

use super::command::OutputMode;
use crate::core::{FormatResult, FormatStrategy, SourceFormatter, utils};

pub enum SuccessResult {
    Changed(String),
    Unchanged,
}

pub struct FormatService {
    cwd: Box<Path>,
    format_mode: OutputMode,
    formatter: SourceFormatter,
}

impl FormatService {
    pub fn new<T>(cwd: T, format_mode: OutputMode, formatter: SourceFormatter) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), format_mode, formatter }
    }

    /// Process entries as they are received from the channel
    pub fn run_streaming(
        &self,
        rx_entry: mpsc::Receiver<FormatStrategy>,
        tx_error: &DiagnosticSender,
        tx_success: &mpsc::Sender<SuccessResult>,
    ) {
        rx_entry.into_iter().par_bridge().for_each(|strategy| {
            let start_time = matches!(self.format_mode, OutputMode::Check).then(Instant::now);

            let path: Arc<Path> = Arc::clone(strategy.path());
            let Ok(source_text) = utils::read_to_string(&path) else {
                // This happens if binary file is attempted to be formatted
                // e.g. `.ts` for MPEG-TS video file
                let diagnostics = DiagnosticService::wrap_diagnostics(
                    self.cwd.clone(),
                    &path,
                    "",
                    vec![
                        oxc_diagnostics::OxcDiagnostic::error(format!(
                            "Failed to read file: {}",
                            path.display()
                        ))
                        .with_help("This may be due to the file being a binary or inaccessible."),
                    ],
                );
                let _ = tx_error.send(diagnostics);
                return;
            };

            let (code, is_changed) = match self.formatter.format(&source_text, strategy) {
                FormatResult::Success { code, is_changed } => (code, is_changed),
                FormatResult::Error(diagnostics) => {
                    let errors = DiagnosticService::wrap_diagnostics(
                        self.cwd.clone(),
                        &path,
                        &source_text,
                        diagnostics,
                    );
                    let _ = tx_error.send(errors);
                    return;
                }
            };

            // Write back if needed
            if matches!(self.format_mode, OutputMode::Write) && is_changed {
                match fs::write(&path, &code) {
                    Ok(()) => (),
                    Err(err) => {
                        let diagnostics = DiagnosticService::wrap_diagnostics(
                            self.cwd.clone(),
                            &path,
                            "",
                            vec![oxc_diagnostics::OxcDiagnostic::error(format!(
                                "Failed to save '{}': {err}",
                                path.display()
                            ))],
                        );
                        let _ = tx_error.send(diagnostics);
                        return;
                    }
                }
            }

            // Report result
            let result = match (&self.format_mode, is_changed) {
                (OutputMode::Check | OutputMode::ListDifferent, true) => {
                    let display_path = path
                        // Show path relative to `cwd` for cleaner output
                        .strip_prefix(&self.cwd)
                        .unwrap_or(path.as_ref())
                        .to_string_lossy()
                        // Normalize path separators for consistent output across platforms
                        .cow_replace('\\', "/")
                        .to_string();

                    if matches!(self.format_mode, OutputMode::Check) {
                        let elapsed = start_time.unwrap().elapsed().as_millis();
                        SuccessResult::Changed(format!("{display_path} ({elapsed}ms)"))
                    } else {
                        SuccessResult::Changed(display_path)
                    }
                }
                _ => SuccessResult::Unchanged,
            };
            let _ = tx_success.send(result);
        });
    }
}
