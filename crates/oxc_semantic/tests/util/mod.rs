mod expect;
mod symbol_tester;
use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_diagnostics::{miette::NamedSource, Error};
extern crate miette;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;

pub use expect::Expect;
pub use symbol_tester::SymbolTester;

pub struct SemanticTester {
    allocator: Allocator,
    source_type: SourceType,
    source_text: &'static str,
}

impl SemanticTester {
    /// Create a new tester for a TypeScript test case.
    ///
    /// Use [`SemanticTester::js`] for JavaScript test cases.
    #[allow(dead_code)]
    pub fn ts(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true).with_typescript(true))
    }

    /// Create a new tester for a JavaScript test case.
    ///
    /// Use [`SemanticTester::ts`] for TypeScript test cases.
    pub fn js(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true))
    }

    pub fn new(source_text: &'static str, source_type: SourceType) -> Self {
        Self { allocator: Allocator::default(), source_type, source_text }
    }

    /// Set the [`SourceType`] to TypeScript (or JavaScript, using `false`)
    #[allow(dead_code)]
    pub fn with_typescript(mut self, yes: bool) -> Self {
        self.source_type = SourceType::default().with_typescript(yes);
        self
    }

    /// Mark the [`SourceType`] as JSX
    #[allow(dead_code)]
    pub fn with_jsx(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_jsx(yes);
        self
    }

    #[allow(dead_code)]
    pub fn with_module(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_module(yes);
        self
    }

    /// Parse the source text and produce a new [`Semantic`]
    #[allow(unstable_name_collisions)]
    pub fn build(&self) -> Semantic<'_> {
        let parse =
            oxc_parser::Parser::new(&self.allocator, self.source_text, self.source_type).parse();

        assert!(
            parse.errors.is_empty(),
            "\n Failed to parse source:\n{}\n\n{}",
            self.source_text,
            parse
                .errors
                .iter()
                .map(|e| format!("{e}"))
                .intersperse("\n\n".to_owned())
                .collect::<String>()
        );

        let program = self.allocator.alloc(parse.program);
        let semantic_ret = SemanticBuilder::new(self.source_text, self.source_type)
            .with_check_syntax_error(true)
            .with_trivias(parse.trivias)
            .build_module_record(PathBuf::new(), program)
            .build(program);

        if !semantic_ret.errors.is_empty() {
            let report = self.wrap_diagnostics(semantic_ret.errors);
            panic!(
                "Semantic analysis failed:\n\n{}",
                report
                    .iter()
                    .map(ToString::to_string)
                    .intersperse("\n\n".to_owned())
                    .collect::<String>()
            );
        };

        semantic_ret.semantic
    }

    /// Tests that a symbol with the given name exists at the top-level scope and provides a
    /// wrapper for writing assertions about the found symbol.
    ///
    /// ## Fails
    /// If no symbol with the given name exists at the top-level scope.
    #[allow(dead_code)]
    pub fn has_root_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_at_root(self, self.build(), name)
    }

    /// Finds some symbol by name in the source code.
    ///
    /// ## Fails
    /// 1. No symbol with the given name exists,
    /// 2. More than one symbol with the given name exists, so a symbol cannot
    ///    be uniquely obtained.
    pub fn has_some_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_unique(self, self.build(), name)
    }

    fn wrap_diagnostics(&self, diagnostics: Vec<Error>) -> Vec<Error> {
        let name = "test".to_owned()
            + match (self.source_type.is_javascript(), self.source_type.is_jsx()) {
                (true, true) => ".jsx",
                (true, false) => ".js",
                (false, true) => ".tsx",
                (false, false) => ".ts",
            };

        let source = Arc::new(NamedSource::new(name, self.source_text.to_owned()));
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect()
    }
}
