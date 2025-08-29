use std::{ffi::OsStr, fs, path::Path, sync::Arc, time::Instant};

use indexmap::IndexSet;
use rayon::prelude::*;
use rustc_hash::FxBuildHasher;

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic, Severity};
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::command::OutputOptions;

pub struct FormatService {
    cwd: Box<Path>,
    output_options: OutputOptions,
    // TODO: Just use `Vec`?
    paths: IndexSet<Arc<OsStr>, FxBuildHasher>,
}

impl FormatService {
    pub fn new<T>(cwd: T, output_options: &OutputOptions) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self {
            cwd: cwd.into(),
            output_options: output_options.clone(),
            paths: IndexSet::with_capacity_and_hasher(0, FxBuildHasher),
        }
    }

    pub fn with_paths(&mut self, paths: Vec<Arc<OsStr>>) -> &mut Self {
        self.paths = paths.into_iter().collect();
        self
    }

    pub fn run(&self, tx_error: &DiagnosticSender) {
        self.paths.iter().par_bridge().for_each(|path| {
            let start_time = Instant::now();

            let path = Path::new(path);
            let source_type =
                SourceType::from_path(path).expect("`path` should be valid SourceType");
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

            match self.output_options {
                OutputOptions::Write => {
                    if is_changed {
                        fs::write(path, code)
                            .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                            .unwrap();
                    }

                    let diagnostic = if is_changed {
                        OxcDiagnostic::warn(format!(
                            "{} {}ms",
                            path.to_string_lossy(),
                            elapsed.as_millis()
                        ))
                    } else {
                        OxcDiagnostic::warn(format!(
                            "{} {}ms (unchanged)",
                            path.to_string_lossy(),
                            elapsed.as_millis()
                        ))
                        .with_severity(Severity::Advice)
                    };
                    tx_error.send((path.to_path_buf(), vec![diagnostic.into()])).unwrap();
                }
                OutputOptions::Default => {
                    if is_changed {
                        let diagnostic = OxcDiagnostic::warn(format!("{}", path.to_string_lossy()));
                        tx_error.send((path.to_path_buf(), vec![diagnostic.into()])).unwrap();
                    }
                }
            }
        });
    }
}
