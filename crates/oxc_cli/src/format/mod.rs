use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::SourceType;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    command::FormatOptions,
    result::{CliRunResult, FormatResult},
    walk::Walk,
    Runner,
};

pub struct FormatRunner {
    options: FormatOptions,
}

impl Runner for FormatRunner {
    type Options = FormatOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    fn run(self) -> CliRunResult {
        let FormatOptions { paths, ignore_options, .. } = &self.options;

        if paths.is_empty() {
            return CliRunResult::InvalidOptions { message: "No paths are provided.".to_string() };
        }

        let now = std::time::Instant::now();

        let paths = Walk::new(paths, ignore_options).paths();

        paths.par_iter().for_each(|path| {
            Self::format(path);
        });

        CliRunResult::FormatResult(FormatResult {
            duration: now.elapsed(),
            number_of_files: paths.len(),
        })
    }
}

impl FormatRunner {
    fn format(path: &Path) {
        let source_text = std::fs::read_to_string(path).unwrap();
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        let _ = Prettier::new(&allocator, PrettierOptions::default()).build(&ret.program);
    }
}
