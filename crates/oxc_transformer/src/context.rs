use std::{
    cell::RefCell,
    mem,
    path::{Path, PathBuf},
};

use oxc_ast::Trivias;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SourceType;

use crate::{
    common::{
        module_imports::ModuleImportsStore, top_level_statements::TopLevelStatementsStore,
        var_declarations::VarDeclarationsStore,
    },
    TransformOptions,
};

pub struct TransformCtx<'a> {
    errors: RefCell<Vec<OxcDiagnostic>>,

    pub trivias: Trivias,

    /// <https://babeljs.io/docs/options#filename>
    pub filename: String,

    /// Source path in the form of `<CWD>/path/to/file/input.js`
    pub source_path: PathBuf,

    pub source_type: SourceType,

    pub source_text: &'a str,

    // Helpers
    /// Manage import statement globally
    pub module_imports: ModuleImportsStore<'a>,
    /// Manage inserting `var` statements globally
    pub var_declarations: VarDeclarationsStore<'a>,
    /// Manage inserting statements at top of program globally
    pub top_level_statements: TopLevelStatementsStore<'a>,
}

impl<'a> TransformCtx<'a> {
    pub fn new(
        source_path: &Path,
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
            filename,
            source_path,
            source_type: SourceType::default(),
            source_text,
            trivias,
            module_imports: ModuleImportsStore::new(),
            var_declarations: VarDeclarationsStore::new(),
            top_level_statements: TopLevelStatementsStore::new(),
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
