use crate::{
    semantic::{NodeTestCase, TestName},
    suite::TestResult,
};
use oxc_ast::AstKind;

/// Tests that container nodes (i.e. nodes that create a new scope) always
/// contain a `scope_id` that is different from the scope they are declared in.
///
/// This tests, for example, that the `scope_id` for function bodies are always
/// different than the scope the function is defined in.
#[derive(Debug, Default, Clone)]
pub struct ContainerIds;

impl TestName for ContainerIds {
    fn name(&self) -> &'static str {
        "nodes/container_ids"
    }
}

impl NodeTestCase for ContainerIds {
    fn run_on_node<'a>(
        &self,
        node: &oxc_semantic::AstNode<'a>,
        semantic: &oxc_semantic::Semantic<'a>,
    ) -> TestResult {
        let Some(container_id) = node.kind().get_container_scope_id() else {
            return TestResult::Passed;
        };

        let node_scope_id = node.scope_id();

        if let AstKind::Program(_) = node.kind() {
            if node_scope_id == semantic.scopes().root_scope_id() {
                return TestResult::Passed;
            }
            return TestResult::Mismatch(
                format!("{node_scope_id:?}"),
                format!("{:?}", semantic.scopes().root_scope_id()),
            );
        }

        if container_id == node_scope_id {
            TestResult::UnexpectedMatch(
                format!("{container_id:?}"),
                Some(
                    "Containers should always create a new scope than the one they are declared in",
                ),
            )
        } else {
            TestResult::Passed
        }
    }
}
