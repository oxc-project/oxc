use std::{ffi::OsStr, fs, path::Path, sync::Arc};

use indexmap::IndexSet;
use rayon::prelude::*;
use rustc_hash::FxBuildHasher;

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic};
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

pub struct FormatService {
    cwd: Box<Path>,
    // TODO: Just use `Vec`?
    paths: IndexSet<Arc<OsStr>, FxBuildHasher>,
}

impl FormatService {
    pub fn new<T>(cwd: T) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), paths: IndexSet::with_capacity_and_hasher(0, FxBuildHasher) }
    }

    pub fn with_paths(&mut self, paths: Vec<Arc<OsStr>>) -> &mut Self {
        self.paths = paths.into_iter().collect();
        self
    }

    // TODO: This should be check(), format(), write() ?
    pub fn run(&self, tx_error: &DiagnosticSender) {
        self.paths.iter().par_bridge().for_each(|path| {
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

            let is_changed = source_text != code;

            // If --write
            // TODO: Report
            // src/lib/highlight.ts 8ms
            // src/lib/index.ts 0ms (unchanged)
            fs::write(path, code)
                .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                .unwrap();

            // NOTE: `path` is needed as string since we do not pass `source_code` and `labels`
            let message = format!(
                "{} {}",
                path.to_string_lossy(),
                if is_changed { "" } else { "(unchanged)" }
            );
            let diagnostics = vec![OxcDiagnostic::warn(message).into()];
            tx_error.send((path.to_path_buf(), diagnostics)).unwrap();
        });
    }
}
