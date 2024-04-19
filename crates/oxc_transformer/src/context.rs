use std::{
    cell::RefCell,
    mem,
    path::{Path, PathBuf},
    rc::Rc,
};

use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

use crate::{helpers::module_imports::ModuleImports, TransformOptions};

pub type Ctx<'a> = Rc<TransformCtx<'a>>;

pub struct TransformCtx<'a> {
    pub ast: AstBuilder<'a>,

    pub semantic: Semantic<'a>,

    /// <https://babeljs.io/docs/options#filename>
    filename: String,

    /// Source path in the form of `<CWD>/path/to/file/input.js`
    source_path: PathBuf,

    errors: RefCell<Vec<Error>>,

    // Helpers
    /// Manage import statement globally
    pub module_imports: ModuleImports<'a>,
}

impl<'a> TransformCtx<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_path: &Path,
        semantic: Semantic<'a>,
        options: &TransformOptions,
    ) -> Self {
        let filename = source_path
            .file_stem() // omit file extension
            .map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string());

        let source_path = source_path
            .strip_prefix(&options.cwd)
            .map_or_else(|_| source_path.to_path_buf(), |p| Path::new("<CWD>").join(p));

        Self {
            ast: AstBuilder::new(allocator),
            semantic,
            filename,
            source_path,
            errors: RefCell::new(vec![]),
            module_imports: ModuleImports::new(allocator),
        }
    }

    pub fn take_errors(&self) -> Vec<Error> {
        mem::take(&mut self.errors.borrow_mut())
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Add an Error
    #[allow(unused)]
    pub fn error<T: Into<Error>>(&self, error: T) {
        self.errors.borrow_mut().push(error.into());
    }

    pub fn source_type(&self) -> &SourceType {
        self.semantic.source_type()
    }
}
