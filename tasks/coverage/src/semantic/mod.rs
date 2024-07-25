mod tester;
#[allow(clippy::self_named_module_files)]
pub mod tests;

use std::sync::OnceLock;

use tester::{NodeTestCase, SemanticRunner, SymbolTestCase, TestName};

pub fn default_runner() -> &'static SemanticRunner {
    static DEFAULT_RUNNER: OnceLock<SemanticRunner> = OnceLock::new();

    DEFAULT_RUNNER.get_or_init(new_default_runner)
}

pub fn new_default_runner() -> SemanticRunner {
    SemanticRunner::with_capacity(0, 2, 1)
        .with_node_test(tests::nodes::container_ids::ContainerIds)
        .with_node_test(tests::nodes::identifiers_are_resolved::IdentifiersAreResolved)
        .with_symbol_test(tests::symbols::function_decls::FunctionDecls)
}
