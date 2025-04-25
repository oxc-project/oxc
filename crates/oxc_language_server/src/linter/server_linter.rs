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
    #[cfg(test)]
    pub fn new(options: IsolatedLintHandlerOptions) -> Self {
        use oxc_linter::{ConfigStoreBuilder, FixKind, LintOptions};

        let config_store =
            ConfigStoreBuilder::default().build().expect("Failed to build config store");
        let linter = Linter::new(LintOptions::default(), config_store).with_fix(FixKind::SafeFix);

        let isolated_linter = Arc::new(IsolatedLintHandler::new(linter, options));

        Self { isolated_linter }
    }

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
    use std::path::PathBuf;

    use super::*;
    use crate::linter::tester::Tester;
    use oxc_linter::{ConfigStoreBuilder, LintFilter, LintFilterKind, LintOptions, Oxlintrc};
    use rustc_hash::FxHashMap;

    #[test]
    fn test_no_errors() {
        Tester::new()
            .with_snapshot_suffix("no_errors")
            .test_and_snapshot_single_file("fixtures/linter/hello_world.js");
    }

    #[test]
    fn test_no_console() {
        let config_store = ConfigStoreBuilder::default()
            .with_filter(&LintFilter::deny(LintFilterKind::parse("no-console".into()).unwrap()))
            .build()
            .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store);

        Tester::new_with_linter(linter)
            .with_snapshot_suffix("deny_no_console")
            .test_and_snapshot_single_file("fixtures/linter/hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        let config_store = ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from("fixtures/linter/issue_9958/.oxlintrc.json"))
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store);

        Tester::new_with_linter(linter)
            .test_and_snapshot_single_file("fixtures/linter/issue_9958/issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        let config_store = ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from("fixtures/linter/regexp_feature/.oxlintrc.json"))
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store);

        Tester::new_with_linter(linter)
            .test_and_snapshot_single_file("fixtures/linter/regexp_feature/index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new().test_and_snapshot_single_file("fixtures/linter/astro/debugger.astro");
        Tester::new().test_and_snapshot_single_file("fixtures/linter/vue/debugger.vue");
        Tester::new().test_and_snapshot_single_file("fixtures/linter/svelte/debugger.svelte");
        // ToDo: fix Tester to work only with Uris and do not access the file system
        // Tester::new().test_and_snapshot_single_file("fixtures/linter/nextjs/%5B%5B..rest%5D%5D/debugger.ts");
    }

    #[test]
    fn test_cross_module_debugger() {
        let config_store: oxc_linter::ConfigStore = ConfigStoreBuilder::from_oxlintrc(
            false,
            Oxlintrc::from_file(&PathBuf::from("fixtures/linter/cross_module/.oxlintrc.json"))
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store);
        let server_linter = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions {
                use_cross_module: true,
                root_path: std::env::current_dir().expect("could not get current dir"),
            },
        );
        Tester::new_with_server_linter(server_linter)
            .test_and_snapshot_single_file("fixtures/linter/cross_module/debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        let config_store = ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from("fixtures/linter/cross_module/.oxlintrc.json"))
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store);
        let server_linter = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions {
                use_cross_module: true,
                root_path: std::env::current_dir().expect("could not get current dir"),
            },
        );

        Tester::new_with_server_linter(server_linter)
            .test_and_snapshot_single_file("fixtures/linter/cross_module/dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        let folder_config_path =
            &PathBuf::from("fixtures/linter/cross_module_nested_config/folder/.oxlintrc.json");
        let default_store =
            ConfigStoreBuilder::from_oxlintrc(false, Oxlintrc::default()).unwrap().build().unwrap();
        let folder_store = ConfigStoreBuilder::from_oxlintrc(
            false,
            Oxlintrc::from_file(folder_config_path).unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();

        let folder_folder_absolute_path =
            std::env::current_dir().unwrap().join(folder_config_path.parent().unwrap());

        // do not insert the default store
        let mut nested_configs = FxHashMap::default();
        nested_configs.insert(folder_folder_absolute_path, folder_store);

        let linter =
            Linter::new_with_nested_configs(LintOptions::default(), default_store, nested_configs);
        let server_linter = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions {
                use_cross_module: true,
                root_path: std::env::current_dir().expect("could not get current dir"),
            },
        );

        Tester::new_with_server_linter(server_linter.clone())
            .test_and_snapshot_single_file("fixtures/linter/cross_module_nested_config/dep-a.ts");

        Tester::new_with_server_linter(server_linter).test_and_snapshot_single_file(
            "fixtures/linter/cross_module_nested_config/folder/folder-dep-a.ts",
        );
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        // ConfigStore searches for the extended config by itself
        // but the LSP still finds the second config with the file walker
        // to simulate the behavior, we build it like the server
        let extended_config_path =
            &PathBuf::from("fixtures/linter/cross_module_extended_config/.oxlintrc.json");
        let folder_config_path =
            &PathBuf::from("fixtures/linter/cross_module_extended_config/config/.oxlintrc.json");

        let default_store =
            ConfigStoreBuilder::from_oxlintrc(false, Oxlintrc::default()).unwrap().build().unwrap();
        let extended_store = ConfigStoreBuilder::from_oxlintrc(
            false,
            Oxlintrc::from_file(extended_config_path).unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let folder_store = ConfigStoreBuilder::from_oxlintrc(
            false,
            Oxlintrc::from_file(folder_config_path).unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();

        let folder_folder_absolute_path =
            std::env::current_dir().unwrap().join(folder_config_path.parent().unwrap());
        let extended_folder_absolute_path =
            std::env::current_dir().unwrap().join(extended_config_path.parent().unwrap());

        // do not insert the default store
        let mut nested_configs = FxHashMap::default();
        nested_configs.insert(folder_folder_absolute_path, folder_store);
        nested_configs.insert(extended_folder_absolute_path, extended_store);

        let linter =
            Linter::new_with_nested_configs(LintOptions::default(), default_store, nested_configs);

        let server_linter = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions {
                use_cross_module: true,
                root_path: std::env::current_dir().expect("could not get current dir"),
            },
        );

        Tester::new_with_server_linter(server_linter)
            .test_and_snapshot_single_file("fixtures/linter/cross_module_extended_config/dep-a.ts");
    }
}
