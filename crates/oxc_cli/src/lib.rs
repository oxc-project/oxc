mod command;
mod walk;

use std::{fs, path::Path};

pub use command::Command;
use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walk::Walk;

pub struct Cli;

impl Cli {
    pub fn lint<P: AsRef<Path>>(path: P) {
        let paths = Walk::new(path).iter().collect::<Vec<_>>();
        paths.par_iter().for_each(|path| {
            Self::parse(path);
        });
    }

    fn parse(path: &Path) {
        let source_text = fs::read_to_string(path).expect("{name} not found");
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).expect("incorrect {path:?}");
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        if !ret.errors.is_empty() {
            for error in ret.errors {
                let error = error.with_source_code(source_text.clone());
                println!("{error:?}");
            }
        }
    }
}
