use std::{fs, path::Path, time::Instant};

use rayon::prelude::*;

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};

use crate::{command::OutputOptions, walk::WalkEntry};

pub struct FormatService {
    cwd: Box<Path>,
    output_options: OutputOptions,
    entries: Vec<WalkEntry>,
}

impl FormatService {
    pub fn new<T>(cwd: T, output_options: &OutputOptions) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), output_options: output_options.clone(), entries: Vec::new() }
    }

    pub fn with_entries(&mut self, entries: Vec<WalkEntry>) -> &mut Self {
        self.entries = entries;
        self
    }

    pub fn run(&self, tx_error: &DiagnosticSender) {
        self.entries.iter().par_bridge().for_each(|entry| {
            let start_time = Instant::now();

            let path = Path::new(&entry.path);
            let source_type = entry.source_type;

            // TODO: read_to_arena_str()?
            let source_text = fs::read_to_string(path).expect("Failed to read file");

            // TODO: AllocatorPool.get()?
            let allocator = Allocator::new();

            let ret = Parser::new(&allocator, &source_text, source_type)
                .with_options(ParseOptions {
                    // Enable all syntax features
                    allow_v8_intrinsics: true,
                    allow_return_outside_function: true,
                    ..ParseOptions::default()
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

            let options = FormatOptions {
                // semicolons: "always".parse().unwrap(),
                semicolons: "as-needed".parse().unwrap(),
                ..FormatOptions::default()
            };
            let code = Formatter::new(&allocator, options).build(&ret.program);

            let elapsed = start_time.elapsed();
            let is_changed = source_text != code;

            // Write back if needed
            if matches!(self.output_options, OutputOptions::Write) && is_changed {
                fs::write(path, code)
                    .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                    .unwrap();
            }

            // Notify if needed
            if let Some(diagnostic) = match (&self.output_options, is_changed) {
                (OutputOptions::Check | OutputOptions::Default, true) => Some(OxcDiagnostic::warn(
                    format!("{} ({}ms)", path.to_string_lossy(), elapsed.as_millis()),
                )),
                (OutputOptions::ListDifferent, true) => {
                    Some(OxcDiagnostic::warn(format!("{}", path.to_string_lossy())))
                }
                (OutputOptions::Write, _) => Some(OxcDiagnostic::warn(format!(
                    "{} {}ms{}",
                    path.to_string_lossy(),
                    elapsed.as_millis(),
                    if is_changed { "" } else { " (unchanged)" }
                ))),
                _ => None,
            } {
                tx_error.send((path.to_path_buf(), vec![diagnostic.into()])).unwrap();
            }
        });
    }
}
