//! Conformance tests.
//!
//! Since these cases are a contract-as-code, they _must be well documented_. When adding a new
//! test, please describe what behavior it guarantees in as plain language as possible.

use std::{borrow::Cow, sync::Arc};

use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource, OxcDiagnostic};
use oxc_semantic::{AstNode, Semantic, SymbolId};

use crate::TestContext;

mod test_identifier_reference;
mod test_symbol_declaration;

pub fn conformance_suite() -> SemanticConformance {
    SemanticConformance::default()
        .with_test(test_symbol_declaration::SymbolDeclarationTest)
        .with_test(test_identifier_reference::IdentifierReferenceTest)
}

pub trait ConformanceTest {
    fn name(&self) -> &'static str;

    #[must_use]
    #[allow(dead_code, unused_variables)]
    fn run_once(&self, semantic: &Semantic<'_>) -> TestResult {
        TestResult::Pass
    }

    #[must_use]
    #[allow(unused_variables)]
    fn run_on_node<'a>(&self, node: &AstNode<'a>, semantic: &Semantic<'a>) -> TestResult {
        TestResult::Pass
    }

    #[must_use]
    #[allow(unused_variables)]
    fn run_on_symbol(&self, symbol_id: SymbolId, semantic: &Semantic<'_>) -> TestResult {
        TestResult::Pass
    }
}

pub struct SemanticConformance {
    tests: Vec<Box<dyn ConformanceTest>>,
    reporter: GraphicalReportHandler,
}

impl Default for SemanticConformance {
    fn default() -> Self {
        Self {
            tests: Vec::new(),
            reporter: GraphicalReportHandler::default()
                .with_theme(GraphicalTheme::unicode_nocolor()),
        }
    }
}

impl SemanticConformance {
    /// Add a test case to the conformance suite.
    pub fn with_test<Test: ConformanceTest + 'static>(mut self, test: Test) -> Self {
        self.tests.push(Box::new(test));
        self
    }

    pub fn run_on_source(&self, ctx: &TestContext<'_>) -> String {
        let named_source = Arc::new(NamedSource::new(
            ctx.path.to_string_lossy(),
            ctx.semantic.source_text().to_string(),
        ));

        let results = self
            .run(&ctx.semantic)
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&named_source)))
            .collect::<Vec<_>>();

        if results.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        for result in results {
            self.reporter.render_report(&mut output, result.as_ref()).unwrap();
        }

        output
    }

    fn run(&self, semantic: &Semantic) -> Vec<OxcDiagnostic> {
        let mut diagnostics = Vec::new();
        for test in &self.tests {
            // Run file-level tests
            self.record_results(&mut diagnostics, test.as_ref(), test.run_once(semantic));

            // Run AST node tests
            for node in semantic.nodes() {
                self.record_results(
                    &mut diagnostics,
                    test.as_ref(),
                    test.run_on_node(node, semantic),
                );
            }

            // Run symbol tests
            for symbol_id in semantic.symbols().symbol_ids() {
                self.record_results(
                    &mut diagnostics,
                    test.as_ref(),
                    test.run_on_symbol(symbol_id, semantic),
                );
            }
        }

        diagnostics
    }

    #[allow(clippy::unused_self)]
    fn record_results(
        &self,
        diagnostics: &mut Vec<OxcDiagnostic>,
        test: &dyn ConformanceTest,
        result: TestResult,
    ) {
        if let TestResult::Fail(reasons) = result {
            diagnostics.extend(
                reasons.into_iter().map(|reason| reason.with_error_code_scope(test.name())),
            );
        }
    }
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Pass,
    Fail(/* reasons */ Vec<OxcDiagnostic>),
}
impl From<String> for TestResult {
    fn from(reason: String) -> Self {
        TestResult::Fail(vec![OxcDiagnostic::error(Cow::Owned(reason))])
    }
}
impl From<Option<String>> for TestResult {
    fn from(result: Option<String>) -> Self {
        match result {
            Some(reason) => TestResult::Fail(vec![OxcDiagnostic::error(Cow::Owned(reason))]),
            None => TestResult::Pass,
        }
    }
}

impl From<OxcDiagnostic> for TestResult {
    fn from(diagnostic: OxcDiagnostic) -> Self {
        TestResult::Fail(vec![diagnostic])
    }
}
impl From<Vec<OxcDiagnostic>> for TestResult {
    fn from(diagnostics: Vec<OxcDiagnostic>) -> Self {
        TestResult::Fail(diagnostics)
    }
}
impl FromIterator<OxcDiagnostic> for TestResult {
    fn from_iter<I: IntoIterator<Item = OxcDiagnostic>>(iter: I) -> Self {
        TestResult::Fail(iter.into_iter().collect())
    }
}
