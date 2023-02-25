mod command;
mod walk;

use std::{fs, path::Path, rc::Rc};

pub use command::Command;
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_linter::Linter;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walk::Walk;

pub struct Cli;

impl Cli {
    pub fn lint<P: AsRef<Path>>(path: P) {
        let paths = Walk::new(path).iter().collect::<Vec<_>>();
        paths.par_iter().for_each(|path| {
            Self::lint_path(path);
        });
    }

    fn lint_path(path: &Path) {
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

        for diagnostic in diagnostics {
            let diagnostic = diagnostic.with_source_code(source_text.clone());
            println!("{diagnostic:?}");
        }
    }
}
