use std::{
    cell::RefCell,
    mem,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, Trivias};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SourceType;

use crate::{
    common::VarDeclarationsStore, helpers::module_imports::ModuleImports, TransformOptions,
};

pub struct TransformCtx<'a> {
    errors: RefCell<Vec<OxcDiagnostic>>,

    pub trivias: Trivias,

    pub ast: AstBuilder<'a>,

    /// <https://babeljs.io/docs/options#filename>
    pub filename: String,

    /// Source path in the form of `<CWD>/path/to/file/input.js`
    pub source_path: PathBuf,

    pub source_type: SourceType,

    pub source_text: &'a str,

    // Helpers
    /// Manage import statement globally
    pub module_imports: ModuleImports<'a>,
    /// Manage inserting `var` statements globally
    pub var_declarations: VarDeclarationsStore<'a>,
}

impl<'a> TransformCtx<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_path: &Path,
        source_type: SourceType,
        source_text: &'a str,
        trivias: Trivias,
        options: &TransformOptions,
    ) -> Self {
        let filename = source_path
            .file_stem() // omit file extension
            .map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string());

        let source_path = source_path
            .strip_prefix(&options.cwd)
            .map_or_else(|_| source_path.to_path_buf(), |p| Path::new("<CWD>").join(p));

        Self {
            errors: RefCell::new(vec![]),
            ast: AstBuilder::new(allocator),
            filename,
            source_path,
            source_type,
            source_text,
            trivias,
            module_imports: ModuleImports::new(),
            var_declarations: VarDeclarationsStore::new(),
        }
    }

    pub fn take_errors(&self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors.borrow_mut())
    }

    /// Add an Error
    pub fn error(&self, error: OxcDiagnostic) {
        self.errors.borrow_mut().push(error);
    }
}
