use std::{fs, path::Path, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_diagnostics::Error;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{Fixer, LintContext, Linter};

pub struct LintService {
    linter: Arc<Linter>,
}

impl LintService {
    pub fn new(linter: Arc<Linter>) -> Self {
        Self { linter }
    }

    /// # Panics
    pub fn run(&self, path: &Path, source_text: &str) -> Option<Vec<Error>> {
        let allocator = Allocator::default();
        let source_type =
            SourceType::from_path(path).unwrap_or_else(|_| panic!("Incorrect {path:?}"));
        let ret = Parser::new(&allocator, source_text, source_type)
            .allow_return_outside_function(true)
            .parse();

        if !ret.errors.is_empty() {
            return Some(ret.errors);
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .with_check_syntax_error(true)
            .with_module_record_builder(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            return Some(semantic_ret.errors);
        };

        let lint_ctx = LintContext::new(&Rc::new(semantic_ret.semantic));
        let result = self.linter.run(lint_ctx);

        if result.is_empty() {
            return None;
        }

        if self.linter.options().fix {
            let fix_result = Fixer::new(source_text, result).fix();
            fs::write(path, fix_result.fixed_code.as_bytes()).unwrap();
            let errors = fix_result.messages.into_iter().map(|m| m.error).collect();
            return Some(errors);
        }

        Some(result.into_iter().map(|diagnostic| diagnostic.error).collect())
    }
}
