use std::{
    collections::HashSet,
    fs,
    path::Path,
    rc::Rc,
    sync::{mpsc, Arc},
};

use oxc_allocator::Allocator;
use oxc_diagnostics::{DiagnosticSender, DiagnosticService, DiagnosticTuple};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{Fixer, LintContext, Linter, Message};

#[derive(Debug)]
pub enum PathWork {
    Begin(Box<Path>),
    Finish(DiagnosticTuple),
    Done,
}

pub struct LintService {
    linter: Arc<Linter>,
    pub tx_path: mpsc::Sender<PathWork>,
    pub rx_path: mpsc::Receiver<PathWork>,
}

impl LintService {
    pub fn new(linter: Arc<Linter>) -> Self {
        let (tx_path, rx_path) = mpsc::channel();
        Self { linter, tx_path, rx_path }
    }

    /// # Panics
    pub fn run(self, tx_error: &DiagnosticSender) {
        let tx_error = tx_error.clone();
        rayon::spawn(move || {
            let mut processing: HashSet<Box<Path>> = HashSet::new();
            let mut done = false;
            while let Ok(work) = self.rx_path.recv() {
                match work {
                    PathWork::Done => {
                        done = true;
                    }
                    PathWork::Begin(path) => {
                        processing.insert(path.clone());
                        self.run_path(path);
                    }
                    PathWork::Finish(diagnostics) => {
                        processing.remove(&diagnostics.0.clone().into_boxed_path());

                        if !diagnostics.1.is_empty() {
                            tx_error.send(Some(diagnostics)).unwrap();
                        }
                    }
                }

                if done && processing.is_empty() {
                    tx_error.send(None).unwrap();
                    break;
                }
            }
        });
    }

    /// # Panics
    pub fn run_path(&self, path: Box<Path>) {
        let linter = Arc::clone(&self.linter);
        let tx_path = self.tx_path.clone();
        rayon::spawn(move || {
            let allocator = Allocator::default();
            let source_text =
                fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {path:?}"));

            let mut messages = Self::run_source(&linter, &path, &allocator, &source_text, true);

            if linter.options().fix {
                let fix_result = Fixer::new(&source_text, messages).fix();
                fs::write(&path, fix_result.fixed_code.as_bytes()).unwrap();
                messages = fix_result.messages;
            }

            let errors = messages.into_iter().map(|m| m.error).collect();
            let diagnostics = DiagnosticService::wrap_diagnostics(&path, &source_text, errors);

            tx_path.send(PathWork::Finish(diagnostics)).unwrap();
        });
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
