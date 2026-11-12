use std::{
    mem,
    path::{Path, PathBuf},
};

use oxc_ast::ast::Class;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeId;

use crate::{
    CompilerAssumptions, HelperLoaderOptions, Module, TransformOptions,
    common::{
        class_lowering::{ClassLowering, ClassLoweringCache},
        class_transform::ClassTransform,
        helper_loader::HelperLoaderStore,
        module_imports::ModuleImportsStore,
        statement_injector::StatementInjectorStore,
        top_level_statements::TopLevelStatementsStore,
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
    /// Ownership of class field and static block lowering, shared by cooperating transforms.
    pub class_transform: ClassTransform,
    /// Recorded lowering decisions per class, shared by cooperating transforms.
    pub class_lowerings: ClassLoweringCache,
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
            class_transform: ClassTransform::default(),
            class_lowerings: ClassLoweringCache::default(),
        }
    }
}

impl<'a> TransformState<'a> {
    pub fn new(source_path: &Path, options: &TransformOptions) -> Self {
        let filename = source_path
            .file_stem() // omit file extension
            .map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string());

        let decorator_mode = options.decorator.mode(options.env.standard_decorators);
        let class_transform = ClassTransform::new(&options.env, decorator_mode);

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
            class_transform,
            class_lowerings: ClassLoweringCache::new(class_transform, decorator_mode),
        }
    }

    /// Record and return the lowering of a class which no transform has rewritten yet.
    pub fn class_lowering(&mut self, class: &Class<'a>) -> ClassLowering {
        self.class_lowerings.get_or_compute(class)
    }

    /// The lowering of a class which has already been entered.
    pub fn class_lowering_of(&self, scope_id: ScopeId) -> ClassLowering {
        self.class_lowerings.get(scope_id)
    }

    pub fn take_errors(&mut self) -> Vec<OxcDiagnostic> {
        mem::take(&mut self.errors)
    }

    /// Add an Error
    pub fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }
}
