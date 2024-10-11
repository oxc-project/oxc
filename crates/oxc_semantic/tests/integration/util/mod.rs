mod class_tester;
mod expect;
mod symbol_tester;
use std::sync::Arc;

pub use class_tester::ClassTester;
pub use expect::Expect;
use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_cfg::DisplayDot;
use oxc_diagnostics::{Error, NamedSource, OxcDiagnostic};
use oxc_semantic::{dot::DebugDot, Semantic, SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
pub use symbol_tester::SymbolTester;

#[must_use]
pub struct SemanticTester<'a> {
    /// Memory arena that AST and [`Semantic`] will store data in.
    allocator: Allocator,
    /// Source type of the test case.
    source_type: SourceType,
    /// The source text of the test case.
    source_text: &'a str,
    /// Build a [`ControlFlowGraph`]?
    ///
    /// [`ControlFlowGraph`]: oxc_cfg::ControlFlowGraph
    cfg: bool,
    /// Build a child ids for scope tree?
    scope_tree_child_ids: bool,
    /// Expect semantic analysis to produce errors.
    ///
    /// Default is `false`.
    expect_errors: bool,
}

impl<'a> SemanticTester<'a> {
    /// Create a new tester for a TypeScript test case.
    ///
    /// Use [`SemanticTester::js`] for JavaScript test cases.
    pub fn ts(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::ts())
    }

    /// Create a new tester for a TypeScript test case with JSX.
    ///
    /// Use [`SemanticTester::ts`] for TypeScript test cases without JSX.
    pub fn tsx(source_text: &'static str) -> Self {
        Self::ts(source_text).with_jsx(true)
    }

    /// Create a new tester for a JavaScript test case.
    ///
    /// Use [`SemanticTester::ts`] for TypeScript test cases.
    pub fn js(source_text: &'static str) -> Self {
        Self::new(source_text, SourceType::mjs())
    }

    /// Create a new tester for some source text.
    ///
    /// You may find one of the other factory methods, like
    /// [`SemanticTester::ts`] or [`SemanticTester::js`], more convenient.
    pub fn new(source_text: &'a str, source_type: SourceType) -> Self {
        Self {
            allocator: Allocator::default(),
            source_type,
            source_text,
            cfg: false,
            scope_tree_child_ids: false,
            expect_errors: false,
        }
    }

    /// Set the [`SourceType`] to TypeScript (or JavaScript, using `false`)
    pub fn with_typescript(mut self, yes: bool) -> Self {
        self.source_type = SourceType::default().with_typescript(yes);
        self
    }

    /// Mark the [`SourceType`] as JSX
    ///
    /// If the source is currently TypeScript, it will become TSX.
    pub fn with_jsx(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_jsx(yes);
        self
    }

    /// Mark the [`SourceType`] as an ESM module.
    pub fn with_module(mut self, yes: bool) -> Self {
        self.source_type = self.source_type.with_module(yes);
        self
    }

    /// Enable or disable building a [`ControlFlowGraph`].
    ///
    /// [`ControlFlowGraph`]: oxc_cfg::ControlFlowGraph
    pub fn with_cfg(mut self, yes: bool) -> Self {
        self.cfg = yes;
        self
    }

    pub fn with_scope_tree_child_ids(mut self, yes: bool) -> Self {
        self.scope_tree_child_ids = yes;
        self
    }

    /// The program being tested is expected to produce errors during semantic analysis.
    ///
    /// By default, programs are expected to be error-free.
    ///
    /// # Example
    /// ```
    /// use crate::util::{SemanticTester as T};
    ///
    /// // Default behavior. You could omit `expect_errors(false)` here
    /// T::js("function foo(a, a) { }").expect_errors(false).has_root_symbol("foo").test()
    /// // Not allowed in TS
    /// T::ts("function foo(a, a) { }").expect_errors(true).has_root_symbol("foo").test()
    /// ```
    pub fn expect_errors(mut self, yes: bool) -> Self {
        self.expect_errors = yes;
        self
    }

    /// Parse the source text and produce a new [`Semantic`].
    ///
    /// Normally this will panic if semantic analysis produces any errors. Use
    /// # Panics
    /// - if parsing fails
    /// - if semantic analysis does/does not produce errors as expected
    #[allow(unstable_name_collisions)]
    pub fn build(&self) -> Semantic<'_> {
        let semantic_ret = self.build_with_errors();
        match (self.expect_errors, semantic_ret.errors.is_empty()) {
            (true, true) => panic!("Expected errors, but none were produced"),
            (false, false) => panic!(
                "Semantic analysis failed:\n\n{}",
                semantic_ret
                    .errors
                    .iter()
                    .map(ToString::to_string)
                    .intersperse("\n\n".to_owned())
                    .collect::<String>()
            ),
            _ => semantic_ret.semantic,
        }
    }

    /// Parse the source text into a new [`Semantic`], but preserves any errors that occur during
    /// semantic analysis
    /// # Panics
    ///
    #[allow(unstable_name_collisions)]
    pub fn build_with_errors(&self) -> SemanticBuilderReturn<'_> {
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
        SemanticBuilder::new()
            .with_check_syntax_error(true)
            .with_cfg(self.cfg)
            .with_scope_tree_child_ids(self.scope_tree_child_ids)
            .build(program)
    }

    pub fn basic_blocks_count(&self) -> usize {
        let built = self.build();
        built.cfg().map_or(0, |cfg| cfg.basic_blocks.len())
    }

    pub fn basic_blocks_printed(&self) -> String {
        let built = self.build();
        built.cfg().map_or_else(String::default, |cfg| {
            cfg.basic_blocks
                .iter()
                .map(DisplayDot::display_dot)
                .enumerate()
                .map(|(i, it)| {
                    format!(
                        "bb{i}: {{\n{}\n}}",
                        it.lines().map(|x| format!("\t{}", x.trim())).join("\n")
                    )
                })
                .join("\n\n")
        })
    }

    pub fn cfg_dot_diagram(&self) -> String {
        let semantic = self.build();
        semantic.cfg().map_or_else(String::default, |cfg| cfg.debug_dot(semantic.nodes().into()))
    }

    /// Tests that a symbol with the given name exists at the top-level scope and provides a
    /// wrapper for writing assertions about the found symbol.
    ///
    /// ## Fails
    /// If no symbol with the given name exists at the top-level scope.
    pub fn has_root_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_at_root(self, self.build(), name)
    }

    /// Find first symbol by name in the source code.
    ///
    /// ## Fails
    /// 1. No symbol with the given name exists,
    pub fn has_symbol(&self, name: &str) -> SymbolTester {
        SymbolTester::new_first_binding(self, self.build(), name)
    }

    /// Tests that a class with the given name exists
    ///
    /// ## Fails
    /// If no class with the given name exists.
    pub fn has_class(&self, name: &str) -> ClassTester {
        ClassTester::has_class(self.build(), name)
    }

    /// Check if semantic analysis produces an error matching `message`.
    ///
    /// Matching is done using `.contains()`.
    ///
    /// ## Fails
    /// - If [`parsing`] produces errors
    /// - If [`SemanticBuilder`] finishes without producing any errors.
    /// - If [`SemanticBuilder`] finishes with errors, but none of them match `message`
    ///
    /// [`parsing`]: oxc_parser::Parser::parse
    pub fn has_error(&self, message: &str) {
        let SemanticBuilderReturn { errors, .. } = self.build_with_errors();
        assert!(
            !errors.is_empty(),
            "Expected an error matching '{message}', but no errors were produced"
        );
        if errors.iter().any(|e| e.message.contains(message)) {
            return;
        }

        let num_errors = errors.len();
        let rendered_errors =
            self.wrap_diagnostics(errors).into_iter().map(|e| e.to_string()).join("\n\n");

        panic!(
            "Expected an error containing '{message}', but none of the {num_errors} matched:\n\n{rendered_errors}",
        )
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

    fn wrap_diagnostics(&self, diagnostics: Vec<OxcDiagnostic>) -> Vec<Error> {
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
