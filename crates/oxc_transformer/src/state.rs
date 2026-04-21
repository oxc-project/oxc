use std::{
    mem,
    path::{Path, PathBuf},
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SourceType;

use crate::{
    CompilerAssumptions, HelperLoaderOptions, Module, TransformOptions,
    common::{
        helper_loader::HelperLoaderStore, module_imports::ModuleImportsStore,
        statement_injector::StatementInjectorStore, top_level_statements::TopLevelStatementsStore,
        var_declarations::VarDeclarationsStore,
    },
};

pub struct TransformState<'a> {
    errors: Vec<OxcDiagnostic>,

    /// <https://babeljs.io/docs/options#filename>
    pub filename: String,

    /// Source path in the form of `<CWD>/path/to/file/input.js`
    pub source_path: PathBuf,

    pub source_type: SourceType,

    pub source_text: &'a str,

    pub module: Module,

    pub assumptions: CompilerAssumptions,

    // Helpers
    /// Manage helper loading
    pub helper_loader: HelperLoaderStore<'a>,
    /// Manage import statement globally
    pub module_imports: ModuleImportsStore<'a>,
    /// Manage inserting `var` statements globally
    pub var_declarations: VarDeclarationsStore<'a>,
    /// Manage inserting statements immediately before or after the target statement
    pub statement_injector: StatementInjectorStore<'a>,
    /// Manage inserting statements at top of program globally
    pub top_level_statements: TopLevelStatementsStore<'a>,

    // State for multiple plugins interacting
    /// `true` if class properties plugin is enabled
    pub is_class_properties_plugin_enabled: bool,
}

impl Default for TransformState<'_> {
    fn default() -> Self {
        Self {
            errors: vec![],
            filename: String::from("unknown"),
            source_path: PathBuf::new(),
            source_type: SourceType::default(),
            source_text: "",
            module: Module::default(),
            assumptions: CompilerAssumptions::default(),
            helper_loader: HelperLoaderStore::new(&HelperLoaderOptions::default()),
            module_imports: ModuleImportsStore::new(),
            var_declarations: VarDeclarationsStore::new(),
            statement_injector: StatementInjectorStore::new(),
            top_level_statements: TopLevelStatementsStore::new(),
            is_class_properties_plugin_enabled: false,
        }
    }
}

impl TransformState<'_> {
    pub fn new(source_path: &Path, options: &TransformOptions) -> Self {
        let filename = source_path
            .file_stem() // omit file extension
            .map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string());

        Self {
            errors: vec![],
            filename,
            source_path: source_path.to_path_buf(),
            source_type: SourceType::default(),
            source_text: "",
            module: options.env.module,
            assumptions: options.assumptions,
            helper_loader: HelperLoaderStore::new(&options.helper_loader),
            module_imports: ModuleImportsStore::new(),
            var_declarations: VarDeclarationsStore::new(),
            statement_injector: StatementInjectorStore::new(),
            top_level_statements: TopLevelStatementsStore::new(),
            is_class_properties_plugin_enabled: options.env.es2022.class_properties.is_some(),
        }
    }

    pub fn take_errors(&mut self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors)
    }

    /// Add an Error
    pub fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }
}
