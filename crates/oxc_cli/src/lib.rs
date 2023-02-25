mod command;
mod walk;

use std::{fs, path::Path, path::PathBuf, rc::Rc};

pub use command::Command;
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_diagnostics::miette::Report;
use oxc_linter::Linter;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walk::Walk;

pub struct Cli;

#[derive(Debug)]
pub struct LintResult {
    pub path: PathBuf,
    pub diagnostics: Vec<Report>,
}

impl Cli {
    pub fn lint<P: AsRef<Path>>(path: P) -> Option<Vec<LintResult>> {
        let paths = Walk::new(path).iter().collect::<Vec<_>>();

        if paths.is_empty() {
            return None;
        }

        let result: Vec<LintResult> = paths
            .par_iter()
            .map(|path| {
                let diagnostics = Self::lint_path(path);
                LintResult { path: path.to_path_buf(), diagnostics }
            })
            .collect();
        Some(result)
    }

    fn lint_path(path: &Path) -> Vec<Report> {
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
