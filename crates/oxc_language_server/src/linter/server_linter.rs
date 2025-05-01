use std::sync::Arc;

use tower_lsp_server::lsp_types::Uri;

use oxc_linter::Linter;

use crate::linter::error_with_position::DiagnosticReport;
use crate::linter::isolated_lint_handler::IsolatedLintHandler;

use super::isolated_lint_handler::IsolatedLintHandlerOptions;

#[derive(Clone)]
pub struct ServerLinter {
    isolated_linter: Arc<IsolatedLintHandler>,
}

impl ServerLinter {
    pub fn new_with_linter(linter: Linter, options: IsolatedLintHandlerOptions) -> Self {
        let isolated_linter = Arc::new(IsolatedLintHandler::new(linter, options));

        Self { isolated_linter }
    }

    pub fn run_single(&self, uri: &Uri, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        self.isolated_linter.run_single(uri, content)
    }
}

#[cfg(test)]
mod test {
    use crate::tester::Tester;

    #[test]
    fn test_no_errors() {
        Tester::new("fixtures/linter/no_errors", None)
            .test_and_snapshot_single_file("hello_world.js");
    }

    #[test]
    fn test_no_console() {
        Tester::new("fixtures/linter/deny_no_console", None)
            .test_and_snapshot_single_file("hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        Tester::new("fixtures/linter/issue_9958", None).test_and_snapshot_single_file("issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        Tester::new("fixtures/linter/regexp_feature", None)
            .test_and_snapshot_single_file("index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new("fixtures/linter/astro", None).test_and_snapshot_single_file("debugger.astro");
        Tester::new("fixtures/linter/vue", None).test_and_snapshot_single_file("debugger.vue");
        Tester::new("fixtures/linter/svelte", None)
            .test_and_snapshot_single_file("debugger.svelte");
        // ToDo: fix Tester to work only with Uris and do not access the file system
        // Tester::new("fixtures/linter/nextjs").test_and_snapshot_single_file("%5B%5B..rest%5D%5D/debugger.ts");
    }

    #[test]
    fn test_invalid_syntax_file() {
        Tester::new("fixtures/linter/invalid_syntax", None)
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_debugger() {
        Tester::new("fixtures/linter/cross_module", None)
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        Tester::new("fixtures/linter/cross_module", None).test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        Tester::new("fixtures/linter/cross_module_nested_config", None)
            .test_and_snapshot_single_file("dep-a.ts");
        Tester::new("fixtures/linter/cross_module_nested_config", None)
            .test_and_snapshot_single_file("folder/folder-dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        Tester::new("fixtures/linter/cross_module_extended_config", None)
            .test_and_snapshot_single_file("dep-a.ts");
    }
}
