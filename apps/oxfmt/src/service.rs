#![expect(clippy::print_stdout)] // TODO
use std::{ffi::OsStr, fs, path::Path, sync::Arc};

use indexmap::IndexSet;
use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use rayon::prelude::*;
use rustc_hash::FxBuildHasher;

pub struct FormatService {
    paths: IndexSet<Arc<OsStr>, FxBuildHasher>,
}

impl FormatService {
    pub fn new() -> Self {
        Self { paths: IndexSet::with_capacity_and_hasher(0, FxBuildHasher) }
    }

    pub fn with_paths(&mut self, paths: Vec<Arc<OsStr>>) -> &mut Self {
        self.paths = paths.into_iter().collect();
        self
    }

    // TODO: This should be check(), format(), write() ?
    // TODO: tx_error
    pub fn run(&self) {
        self.paths.iter().par_bridge().for_each(|path| {
            let path = Path::new(path);
            let source_type =
                SourceType::from_path(path).expect("`path` should be valid SourceType");
            // TODO: AllocatorPool.get()?
            let allocator = Allocator::new();
            // TODO: read_to_arena_str()?
            let source_text = fs::read_to_string(path).unwrap(); // TODO: OxcDiagnostic

            let ret = Parser::new(&allocator, &source_text, source_type)
                .with_options(ParseOptions {
                    allow_v8_intrinsics: true,
                    allow_return_outside_function: true,
                    ..ParseOptions::default()
                })
                .parse();
            if !ret.errors.is_empty() {
                // TODO
            }

            let options = FormatOptions {
                // semicolons: "always".parse().unwrap(),
                semicolons: "as-needed".parse().unwrap(),
                ..FormatOptions::default()
            };
            let code = Formatter::new(&allocator, options).build(&ret.program);

            let is_changed = source_text != code;
            println!("{}{}", path.display(), if is_changed { "" } else { " (unchanged)" });

            // If --write
            fs::write(path, code)
                .map_err(|_| format!("Failed to write to '{}'", path.to_string_lossy()))
                .unwrap();
        });
    }
}
