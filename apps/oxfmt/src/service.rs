use std::{fs, path::Path, sync::mpsc, time::Instant};

use cow_utils::CowUtils;
use rayon::prelude::*;

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};

use crate::{command::OutputOptions, walk::WalkEntry};

pub struct FormatService {
    cwd: Box<Path>,
    output_options: OutputOptions,
}

impl FormatService {
    pub fn new<T>(cwd: T, output_options: OutputOptions) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), output_options }
    }

    /// Process entries as they are received from the channel
    #[expect(clippy::needless_pass_by_value)]
    pub fn run_streaming(
        &self,
        rx_entry: mpsc::Receiver<WalkEntry>,
        tx_error: &DiagnosticSender,
        // Take ownership to close the channel when done
        tx_count: mpsc::Sender<()>,
    ) {
        rx_entry.into_iter().par_bridge().for_each(|entry| {
            self.process_entry(&entry, tx_error);
            // Signal that we processed one file (ignore send errors if receiver dropped)
            let _ = tx_count.send(());
        });
    }

    /// Process a single entry
    fn process_entry(&self, entry: &WalkEntry, tx_error: &DiagnosticSender) {
        let start_time = Instant::now();

        let path = Path::new(&entry.path);
        let source_type = entry.source_type;

        // TODO: Use `read_to_arena_str()` like `oxlint`?
        let source_text = fs::read_to_string(path).expect("Failed to read file");
        // TODO: Use `AllocatorPool.get()` like `oxlint`?
        let allocator = Allocator::new();

        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: false,
                // Enable all syntax features
                allow_v8_intrinsics: true,
                allow_return_outside_function: true,
                // `oxc_formatter` expects this to be false
                preserve_parens: false,
            })
            .parse();
        if !ret.errors.is_empty() {
            let diagnostics = DiagnosticService::wrap_diagnostics(
                self.cwd.clone(),
                path,
                &source_text,
                ret.errors,
            );
            tx_error.send((path.to_path_buf(), diagnostics)).unwrap();
            return;
        }

        // TODO: Read and apply config
        let options = FormatOptions::default();
        let code = Formatter::new(&allocator, options).build(&ret.program);

        let elapsed = start_time.elapsed();
        let is_changed = source_text != code;

        // Write back if needed
        if matches!(self.output_options, OutputOptions::DefaultWrite) && is_changed {
            fs::write(path, code)
                .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                .unwrap();
        }

        // Notify if needed
        let display_path = path
            // Show path relative to `cwd` for cleaner output
            .strip_prefix(&self.cwd)
            .unwrap_or(path)
            .to_string_lossy()
            // Normalize path separators for consistent output across platforms
            .cow_replace('\\', "/")
            .to_string();
        let elapsed = elapsed.as_millis();
        if let Some(diagnostic) = match (&self.output_options, is_changed) {
            (OutputOptions::Check, true) => {
                Some(OxcDiagnostic::warn(format!("{display_path} ({elapsed}ms)")))
            }
            (OutputOptions::ListDifferent, true) => Some(OxcDiagnostic::warn(display_path)),
            (OutputOptions::DefaultWrite, _) => Some(OxcDiagnostic::warn(format!(
                "{display_path} {elapsed}ms{}",
                if is_changed { "" } else { " (unchanged)" }
            ))),
            _ => None,
        } {
            tx_error.send((path.to_path_buf(), vec![diagnostic.into()])).unwrap();
        }
    }
}
