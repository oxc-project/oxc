use std::{fs, path::Path, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{Fixer, LintContext, LintOptions, Linter, Message};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

#[derive(Clone)]
pub struct LintService {
    linter: Arc<Linter>,
}

impl LintService {
    pub fn new(options: LintOptions) -> Self {
        let linter = Arc::new(Linter::from_options(options));
        Self { linter }
    }

    pub fn linter(&self) -> &Linter {
        &self.linter
    }

    /// # Panics
    pub fn run(&self, paths: Vec<Box<Path>>, tx_error: &DiagnosticSender) {
        paths.into_par_iter().for_each_with(&self.linter, |linter, path| {
            Self::run_path(linter, &path, tx_error);
        });
        tx_error.send(None).unwrap();
    }

    /// # Panics
    fn run_path(linter: &Arc<Linter>, path: &Path, tx_error: &DiagnosticSender) {
        let tx_error = tx_error.clone();
        let allocator = Allocator::default();
        let source_text =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path:?}"));

        let mut messages = Self::run_source(linter, path, &allocator, &source_text, true);

        if linter.options().fix {
            let fix_result = Fixer::new(&source_text, messages).fix();
            fs::write(path, fix_result.fixed_code.as_bytes()).unwrap();
            messages = fix_result.messages;
        }

        let errors = messages.into_iter().map(|m| m.error).collect();
        let diagnostics = DiagnosticService::wrap_diagnostics(path, &source_text, errors);
        tx_error.send(Some(diagnostics)).unwrap();
    }

    pub(crate) fn run_source<'a>(
        linter: &Linter,
        path: &Path,
        allocator: &'a Allocator,
        source_text: &'a str,
        check_syntax_errors: bool,
    ) -> Vec<Message<'a>> {
        let source_type =
            SourceType::from_path(path).unwrap_or_else(|_| panic!("Incorrect {path:?}"));
        let ret = Parser::new(allocator, source_text, source_type)
            .allow_return_outside_function(true)
            .parse();

        if !ret.errors.is_empty() {
            return ret.errors.into_iter().map(|err| Message::new(err, None)).collect();
        };

        let program = allocator.alloc(ret.program);
        let semantic_ret = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .with_check_syntax_error(check_syntax_errors)
            .with_module_record_builder(true)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            return semantic_ret.errors.into_iter().map(|err| Message::new(err, None)).collect();
        };

        let lint_ctx = LintContext::new(&Rc::new(semantic_ret.semantic));
        linter.run(lint_ctx)
    }
}
