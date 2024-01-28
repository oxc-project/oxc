mod class_tester;
mod expect;
mod symbol_tester;
use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_diagnostics::{miette::NamedSource, Error};
use oxc_semantic::{print_basic_block, Semantic, SemanticBuilder};
use oxc_span::SourceType;

pub use class_tester::ClassTester;
pub use expect::Expect;
use petgraph::dot::{Config, Dot};
pub use symbol_tester::SymbolTester;

pub struct SemanticTester<'a> {
    allocator: Allocator,
    source_type: SourceType,
    source_text: &'a str,
}

impl<'a> SemanticTester<'a> {
    /// Create a new tester for a TypeScript test case.
    ///
    /// Use [`SemanticTester::js`] for JavaScript test cases.
    pub fn ts(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true).with_typescript(true))
    }

    /// Create a new tester for a JavaScript test case.
    ///
    /// Use [`SemanticTester::ts`] for TypeScript test cases.
    pub fn js(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::default().with_module(true))
    }

    pub fn new(source_text: &'a str, source_type: SourceType) -> Self {
        Self { allocator: Allocator::default(), source_type, source_text }
    }

    /// Set the [`SourceType`] to TypeScript (or JavaScript, using `false`)
    #[must_use]
    pub fn with_typescript(mut self, yes: bool) -> Self {
        self.source_type = SourceType::default().with_typescript(yes);
        self
    }

    /// Mark the [`SourceType`] as JSX
    #[must_use]
    pub fn with_jsx(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_jsx(yes);
        self
    }

    #[must_use]
    pub fn with_module(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_module(yes);
        self
    }

    /// Parse the source text and produce a new [`Semantic`]
    /// # Panics
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

    pub fn basic_blocks_count(&self) -> usize {
        let built = self.build();
        built.cfg().basic_blocks.len()
    }

    pub fn basic_blocks_printed(&self) -> String {
        let built = self.build();
        built
            .cfg()
            .basic_blocks
            .iter()
            .map(print_basic_block)
            .enumerate()
            .map(|(i, it)| {
                format!(
                    "bb{i}: {{\n{}\n}}",
                    it.lines().map(|x| format!("\t{}", x.trim())).join("\n")
                )
            })
            .join("\n\n")
    }

    pub fn cfg_dot_diagram(&self) -> String {
        let built = self.build();
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &built.cfg().graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_graph, _edge| String::new(),
                // todo: We currently do not print edge types into cfg dot diagram
                // so they aren't snapshotted, but we could by uncommenting this.
                // &|_graph, edge| format!("label = {:?}", edge.weight()),
                &|_graph, node| format!(
                    "label = {:?}",
                    print_basic_block(&built.cfg().basic_blocks[*node.1],).trim()
                )
            )
        )
    }

    /// Tests that a symbol with the given name exists at the top-level scope and provides a
    /// wrapper for writing assertions about the found symbol.
    ///
    /// ## Fails
    /// If no symbol with the given name exists at the top-level scope.
    pub fn has_root_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_at_root(self, self.build(), name)
    }

    /// Tests that a class with the given name exists
    ///
    /// ## Fails
    /// If no class with the given name exists.
    pub fn has_class(&self, name: &str) -> ClassTester {
        ClassTester::has_class(self.build(), name)
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
