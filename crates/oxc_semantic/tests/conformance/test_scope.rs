use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Semantic;
use oxc_span::GetSpan;

use super::{ConformanceTest, TestResult};

/// Scope node ranges must be nested so position-based consumers cannot select an out-of-scope binding.
#[derive(Debug, Clone, Default)]
pub struct ScopeTest;

impl ConformanceTest for ScopeTest {
    fn name(&self) -> &'static str {
        "scope"
    }

    fn run_once(&self, semantic: &Semantic<'_>) -> TestResult {
        let scoping = semantic.scoping();
        let mut diagnostics = Vec::new();
        for scope_id in scoping.scope_descendants_from_root() {
            let Some(parent_id) = scoping.scope_parent_id(scope_id) else { continue };
            let span = semantic.nodes().kind(scoping.get_node_id(scope_id)).span();
            let parent_span = semantic.nodes().kind(scoping.get_node_id(parent_id)).span();
            if !parent_span.contains_inclusive(span) {
                diagnostics.push(
                    OxcDiagnostic::error(format!(
                        "Scope {scope_id:?} extends outside its parent {parent_id:?}"
                    ))
                    .with_label(span),
                );
            }
        }
        if diagnostics.is_empty() { TestResult::Pass } else { TestResult::Fail(diagnostics) }
    }
}
