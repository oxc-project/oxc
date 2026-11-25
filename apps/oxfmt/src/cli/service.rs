use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

use cow_utils::CowUtils;
use rayon::prelude::*;
use tracing::debug_span;

use oxc_diagnostics::{DiagnosticSender, DiagnosticService};

use super::{cache::FormatCache, command::OutputMode};
use crate::core::{ExternalCalls, FormatPlan, FormatResult, SourceFormatter, utils};

pub enum SuccessResult {
    /// Path with elapsed time, only measured in check mode
    Changed(String, Option<Duration>),
    Unchanged,
}

pub struct FormatService {
    cwd: Box<Path>,
    format_mode: OutputMode,
    formatter: SourceFormatter,
    cache: Option<FormatCache>,
}

impl FormatService {
    pub fn new<T>(
        cwd: T,
        format_mode: OutputMode,
        formatter: SourceFormatter,
        cache: Option<FormatCache>,
    ) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), format_mode, formatter, cache }
    }

    /// Process entries as they are received from the channel
    pub fn run_streaming(
        self,
        rx_entry: mpsc::Receiver<FormatPlan>,
        tx_error: &DiagnosticSender,
        tx_success: &mpsc::Sender<SuccessResult>,
    ) {
        rx_entry.into_iter().par_bridge().for_each(|plan| {
            let start_time = matches!(self.format_mode, OutputMode::Check).then(Instant::now);

            let path = Arc::clone(&plan.path);
            let Ok(source_text) = debug_span!("oxfmt::read", path = %path.display())
                .in_scope(|| utils::read_to_string(&path))
            else {
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

            let key =
                self.cache.as_ref().map(|cache| (cache, FormatCache::key(&plan, &source_text)));
            if key.is_some_and(|(cache, key)| cache.is_formatted(key)) {
                let _ = tx_success.send(SuccessResult::Unchanged);
                return;
            }
            let (result, with_node) =
                ExternalCalls::track(|| self.formatter.format(&source_text, plan));
            if let Some((cache, key)) = key
                && !with_node
                && matches!(result, FormatResult::Success { is_changed: false, .. })
            {
                cache.set_formatted(key);
            }
            let (code, is_changed) = match result {
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

                    let elapsed = start_time.map(|start| start.elapsed());
                    SuccessResult::Changed(display_path, elapsed)
                }
                _ => SuccessResult::Unchanged,
            };
            let _ = tx_success.send(result);
        });

        if let Some(cache) = self.cache {
            cache.save();
        }
    }
}
