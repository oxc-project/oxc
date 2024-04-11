use std::{cell::RefCell, mem, path::Path, rc::Rc};

use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;

pub type Ctx<'a> = Rc<TransformCtx<'a>>;

pub struct TransformCtx<'a> {
    pub ast: AstBuilder<'a>,

    pub semantic: Semantic<'a>,

    /// <https://babeljs.io/docs/options#filename>
    filename: String,

    errors: RefCell<Vec<Error>>,
}

impl<'a> TransformCtx<'a> {
    pub fn new(allocator: &'a Allocator, source_path: &Path, semantic: Semantic<'a>) -> Self {
        let ast = AstBuilder::new(allocator);
        let filename = source_path
            .file_stem() // omit file extension
            .map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string());
        let errors = RefCell::new(vec![]);
        Self { ast, semantic, filename, errors }
    }

    pub fn take_errors(&self) -> Vec<Error> {
        mem::take(&mut self.errors.borrow_mut())
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Add an Error
    #[allow(unused)]
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }
}
