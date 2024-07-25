use oxc_semantic::{AstNode, Semantic, SymbolId};

use crate::suite::TestResult;

pub trait TestName {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

/// A test case that gets run on each [`AstNode`] within a [`Semantic`]'s AST.
pub trait NodeTestCase: TestName + Send + Sync + 'static {
    fn run_on_node<'a>(&self, node: &AstNode<'a>, semantic: &Semantic<'a>) -> TestResult;
}

/// A test case that gets run on each symbol found after a parse.
pub trait SymbolTestCase: TestName + Send + Sync + 'static {
    fn run_on_symbol(&self, symbol_id: SymbolId, semantic: &Semantic<'_>) -> TestResult;
}

/// A test case that gets run once on a [`Semantic`]
pub trait OnceTestCase: TestName + Send + Sync + 'static {
    fn run_once(&self, semantic: &Semantic<'_>) -> TestResult;
}

#[must_use]
#[non_exhaustive]
#[allow(clippy::struct_field_names)]
pub struct SemanticRunner {
    once_tests: Vec<Box<dyn OnceTestCase>>,
    node_tests: Vec<Box<dyn NodeTestCase>>,
    symbol_tests: Vec<Box<dyn SymbolTestCase>>,
}

impl SemanticRunner {
    pub fn with_capacity(once_tests: usize, node_tests: usize, symbol_tests: usize) -> Self {
        Self {
            once_tests: Vec::with_capacity(once_tests),
            node_tests: Vec::with_capacity(node_tests),
            symbol_tests: Vec::with_capacity(symbol_tests),
        }
    }

    #[allow(dead_code)]
    pub fn with_once_test<T: OnceTestCase>(mut self, test: T) -> Self {
        self.once_tests.push(Box::new(test));
        self
    }

    pub fn with_node_test<T: NodeTestCase>(mut self, test: T) -> Self {
        self.node_tests.push(Box::new(test));
        self
    }

    pub fn with_symbol_test<T: SymbolTestCase>(mut self, test: T) -> Self {
        self.symbol_tests.push(Box::new(test));
        self
    }

    pub fn test(&self, semantic: &Semantic<'_>) -> Vec<TestResult> {
        let mut results = Vec::with_capacity(
            // one result per once test
            self.once_tests.len() +
            // one result per node per test
            (self.node_tests.len() * semantic.nodes().len()) +
            // one result per symbol per test
            (self.symbol_tests.len() * semantic.symbols().len()),
        );

        // run all one-shot tests
        for test in &self.once_tests {
            results.push(test.run_once(semantic));
        }

        // run all node test cases on all nodes
        for test in &self.node_tests {
            for node in semantic.nodes().iter() {
                results.push(test.run_on_node(node, semantic));
            }
        }

        // run all symbol test cases on all symbols
        for test in &self.symbol_tests {
            for symbol_id in semantic.symbols().iter() {
                results.push(test.run_on_symbol(symbol_id, semantic));
            }
        }

        results
    }
}
