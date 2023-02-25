mod command;
mod options;
mod result;
mod walk;

use std::{fs, path::Path, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::{Error, Severity};
use oxc_linter::Linter;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walk::Walk;

pub use crate::{command::Command, options::CliOptions, result::CliRunResult};

pub struct Cli;

impl Cli {
    pub fn lint<P: AsRef<Path>>(path: P, cli_options: CliOptions) -> CliRunResult {
        let paths = Walk::new(path).iter().collect::<Vec<_>>();

        let number_of_diagnostics = paths
            .par_iter()
            .map(|path| {
                let diagnostics = Self::lint_path(path);

                diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.severity().unwrap() != Severity::Warning || !cli_options.quiet
                    })
                    .for_each(|diagnostic| {
                        println!("{diagnostic:?}");
                    });

                diagnostics.len()
            })
            .sum();

        CliRunResult::LintResult { number_of_files: paths.len(), number_of_diagnostics }
    }

    fn lint_path(path: &Path) -> Vec<Error> {
        let source_text = fs::read_to_string(path).expect("{name} not found");
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).expect("incorrect {path:?}");
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        let diagnostics = if ret.errors.is_empty() {
            let program = allocator.alloc(ret.program);
            let semantic = SemanticBuilder::new().build(program);
            Linter::new().run(&Rc::new(semantic))
        } else {
            ret.errors
        };

        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(source_text.clone()))
            .collect()
    }
}
