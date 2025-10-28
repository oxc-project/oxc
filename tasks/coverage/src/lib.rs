#![expect(clippy::print_stdout, clippy::disallowed_methods)]

// Core
mod file_discovery;
mod runtime;
mod suite;
// Suites
mod babel;
mod misc;
mod node_compat_table;
mod test262;
mod typescript;

mod driver;
mod tools;

use std::{path::PathBuf, process::Command};

use oxc_tasks_common::project_root;
use runtime::Test262RuntimeCase;
use tools::estree::{AcornJsxSuite, EstreeJsxCase, EstreeTypescriptCase};

use crate::{
    babel::{BabelCase, BabelSuite},
    driver::Driver,
    file_discovery::{DiscoveredFiles, FileDiscoveryConfig},
    misc::{MiscCase, MiscSuite},
    node_compat_table::NodeCompatSuite,
    suite::Suite,
    test262::{Test262Case, Test262Suite},
    tools::{
        codegen::{CodegenBabelCase, CodegenMiscCase, CodegenTest262Case, CodegenTypeScriptCase},
        estree::EstreeTest262Case,
        formatter::{
            FormatterBabelCase, FormatterMiscCase, FormatterTest262Case, FormatterTypeScriptCase,
        },
        minifier::{MinifierBabelCase, MinifierNodeCompatCase, MinifierTest262Case},
        semantic::{
            SemanticBabelCase, SemanticMiscCase, SemanticTest262Case, SemanticTypeScriptCase,
        },
        transformer::{
            TransformerBabelCase, TransformerMiscCase, TransformerTest262Case,
            TransformerTypeScriptCase,
        },
    },
    typescript::{TranspileRunner, TypeScriptCase, TypeScriptSuite, TypeScriptTranspileCase},
};

pub fn workspace_root() -> PathBuf {
    project_root().join("tasks").join("coverage")
}

fn snap_root() -> PathBuf {
    workspace_root().join("snapshots")
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub debug: bool,
    pub filter: Option<String>,
    pub detail: bool,
    /// Print mismatch diff
    pub diff: bool,
}

impl AppArgs {
    fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }

    pub fn run_default(&self) {
        // Process each suite sequentially to minimize memory usage and I/O operations
        // Each suite is: 1) walked once, 2) files read once, 3) all tools run, 4) memory freed
        self.process_test262_suite();
        self.process_babel_suite();
        self.process_typescript_suite();
        self.process_misc_suite();
        self.run_transpiler();
    }

    fn process_test262_suite(&self) {
        // Discover files once (1 directory walk + 1 file read)
        let files = {
            let suite = Test262Suite::<Test262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "test262",
            })
        };

        // Run all tools sequentially with the same files
        Test262Suite::<Test262Case>::new().run_with_discovered_files(
            "parser_test262",
            self,
            &files,
        );
        Test262Suite::<SemanticTest262Case>::new().run_with_discovered_files(
            "semantic_test262",
            self,
            &files,
        );
        Test262Suite::<CodegenTest262Case>::new().run_with_discovered_files(
            "codegen_test262",
            self,
            &files,
        );
        Test262Suite::<FormatterTest262Case>::new().run_with_discovered_files(
            "formatter_test262",
            self,
            &files,
        );
        Test262Suite::<TransformerTest262Case>::new().run_with_discovered_files(
            "transformer_test262",
            self,
            &files,
        );
        Test262Suite::<MinifierTest262Case>::new().run_with_discovered_files(
            "minifier_test262",
            self,
            &files,
        );
        Test262Suite::<EstreeTest262Case>::new().run_with_discovered_files(
            "estree_test262",
            self,
            &files,
        );
        // Files dropped here, memory freed
    }

    fn process_babel_suite(&self) {
        let files = {
            let suite = BabelSuite::<BabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "babel",
            })
        };

        BabelSuite::<BabelCase>::new().run_with_discovered_files("parser_babel", self, &files);
        BabelSuite::<SemanticBabelCase>::new().run_with_discovered_files(
            "semantic_babel",
            self,
            &files,
        );
        BabelSuite::<CodegenBabelCase>::new().run_with_discovered_files(
            "codegen_babel",
            self,
            &files,
        );
        BabelSuite::<FormatterBabelCase>::new().run_with_discovered_files(
            "formatter_babel",
            self,
            &files,
        );
        BabelSuite::<TransformerBabelCase>::new().run_with_discovered_files(
            "transformer_babel",
            self,
            &files,
        );
        BabelSuite::<MinifierBabelCase>::new().run_with_discovered_files(
            "minifier_babel",
            self,
            &files,
        );
    }

    fn process_typescript_suite(&self) {
        let files = {
            let suite = TypeScriptSuite::<TypeScriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "typescript",
            })
        };

        TypeScriptSuite::<TypeScriptCase>::new().run_with_discovered_files(
            "parser_typescript",
            self,
            &files,
        );
        TypeScriptSuite::<SemanticTypeScriptCase>::new().run_with_discovered_files(
            "semantic_typescript",
            self,
            &files,
        );
        TypeScriptSuite::<CodegenTypeScriptCase>::new().run_with_discovered_files(
            "codegen_typescript",
            self,
            &files,
        );
        TypeScriptSuite::<FormatterTypeScriptCase>::new().run_with_discovered_files(
            "formatter_typescript",
            self,
            &files,
        );
        TypeScriptSuite::<TransformerTypeScriptCase>::new().run_with_discovered_files(
            "transformer_typescript",
            self,
            &files,
        );
        TypeScriptSuite::<EstreeTypescriptCase>::new().run_with_discovered_files(
            "estree_typescript",
            self,
            &files,
        );
    }

    fn process_misc_suite(&self) {
        let files = {
            let suite = MiscSuite::<MiscCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "misc",
            })
        };

        MiscSuite::<MiscCase>::new().run_with_discovered_files("parser_misc", self, &files);
        MiscSuite::<SemanticMiscCase>::new().run_with_discovered_files(
            "semantic_misc",
            self,
            &files,
        );
        MiscSuite::<CodegenMiscCase>::new().run_with_discovered_files("codegen_misc", self, &files);
        MiscSuite::<FormatterMiscCase>::new().run_with_discovered_files(
            "formatter_misc",
            self,
            &files,
        );
        MiscSuite::<TransformerMiscCase>::new().run_with_discovered_files(
            "transformer_misc",
            self,
            &files,
        );
    }

    pub fn run_parser(&self) {
        let test262_files = {
            let suite = Test262Suite::<Test262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "parser_test262",
            })
        };

        let babel_files = {
            let suite = BabelSuite::<BabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "parser_babel",
            })
        };

        let typescript_files = {
            let suite = TypeScriptSuite::<TypeScriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "parser_typescript",
            })
        };

        let misc_files = {
            let suite = MiscSuite::<MiscCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "parser_misc",
            })
        };

        Test262Suite::<Test262Case>::new().run_with_discovered_files(
            "parser_test262",
            self,
            &test262_files,
        );
        BabelSuite::<BabelCase>::new().run_with_discovered_files(
            "parser_babel",
            self,
            &babel_files,
        );
        TypeScriptSuite::<TypeScriptCase>::new().run_with_discovered_files(
            "parser_typescript",
            self,
            &typescript_files,
        );
        MiscSuite::<MiscCase>::new().run_with_discovered_files("parser_misc", self, &misc_files);
    }

    pub fn run_semantic(&self) {
        let test262_files = {
            let suite = Test262Suite::<SemanticTest262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "semantic_test262",
            })
        };

        let babel_files = {
            let suite = BabelSuite::<SemanticBabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "semantic_babel",
            })
        };

        let typescript_files = {
            let suite = TypeScriptSuite::<SemanticTypeScriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "semantic_typescript",
            })
        };

        let misc_files = {
            let suite = MiscSuite::<SemanticMiscCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "semantic_misc",
            })
        };

        Test262Suite::<SemanticTest262Case>::new().run_with_discovered_files(
            "semantic_test262",
            self,
            &test262_files,
        );
        BabelSuite::<SemanticBabelCase>::new().run_with_discovered_files(
            "semantic_babel",
            self,
            &babel_files,
        );
        TypeScriptSuite::<SemanticTypeScriptCase>::new().run_with_discovered_files(
            "semantic_typescript",
            self,
            &typescript_files,
        );
        MiscSuite::<SemanticMiscCase>::new().run_with_discovered_files(
            "semantic_misc",
            self,
            &misc_files,
        );
    }

    pub fn run_codegen(&self) {
        let test262_files = {
            let suite = Test262Suite::<CodegenTest262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "codegen_test262",
            })
        };

        let babel_files = {
            let suite = BabelSuite::<CodegenBabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "codegen_babel",
            })
        };

        let typescript_files = {
            let suite = TypeScriptSuite::<CodegenTypeScriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "codegen_typescript",
            })
        };

        let misc_files = {
            let suite = MiscSuite::<CodegenMiscCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "codegen_misc",
            })
        };

        Test262Suite::<CodegenTest262Case>::new().run_with_discovered_files(
            "codegen_test262",
            self,
            &test262_files,
        );
        BabelSuite::<CodegenBabelCase>::new().run_with_discovered_files(
            "codegen_babel",
            self,
            &babel_files,
        );
        TypeScriptSuite::<CodegenTypeScriptCase>::new().run_with_discovered_files(
            "codegen_typescript",
            self,
            &typescript_files,
        );
        MiscSuite::<CodegenMiscCase>::new().run_with_discovered_files(
            "codegen_misc",
            self,
            &misc_files,
        );
    }

    pub fn run_formatter(&self) {
        let test262_files = {
            let suite = Test262Suite::<FormatterTest262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "formatter_test262",
            })
        };

        let babel_files = {
            let suite = BabelSuite::<FormatterBabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "formatter_babel",
            })
        };

        let typescript_files = {
            let suite = TypeScriptSuite::<FormatterTypeScriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "formatter_typescript",
            })
        };

        let misc_files = {
            let suite = MiscSuite::<FormatterMiscCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "formatter_misc",
            })
        };

        Test262Suite::<FormatterTest262Case>::new().run_with_discovered_files(
            "formatter_test262",
            self,
            &test262_files,
        );
        BabelSuite::<FormatterBabelCase>::new().run_with_discovered_files(
            "formatter_babel",
            self,
            &babel_files,
        );
        TypeScriptSuite::<FormatterTypeScriptCase>::new().run_with_discovered_files(
            "formatter_typescript",
            self,
            &typescript_files,
        );
        MiscSuite::<FormatterMiscCase>::new().run_with_discovered_files(
            "formatter_misc",
            self,
            &misc_files,
        );
    }

    pub fn run_transformer(&self) {
        let test262_files = {
            let suite = Test262Suite::<TransformerTest262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "transformer_test262",
            })
        };

        let babel_files = {
            let suite = BabelSuite::<TransformerBabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "transformer_babel",
            })
        };

        let typescript_files = {
            let suite = TypeScriptSuite::<TransformerTypeScriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "transformer_typescript",
            })
        };

        let misc_files = {
            let suite = MiscSuite::<TransformerMiscCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "transformer_misc",
            })
        };

        Test262Suite::<TransformerTest262Case>::new().run_with_discovered_files(
            "transformer_test262",
            self,
            &test262_files,
        );
        BabelSuite::<TransformerBabelCase>::new().run_with_discovered_files(
            "transformer_babel",
            self,
            &babel_files,
        );
        TypeScriptSuite::<TransformerTypeScriptCase>::new().run_with_discovered_files(
            "transformer_typescript",
            self,
            &typescript_files,
        );
        MiscSuite::<TransformerMiscCase>::new().run_with_discovered_files(
            "transformer_misc",
            self,
            &misc_files,
        );
    }

    pub fn run_transpiler(&self) {
        TranspileRunner::<TypeScriptTranspileCase>::new().run("transpile", self);
    }

    pub fn run_estree(&self) {
        let test262_files = {
            let suite = Test262Suite::<EstreeTest262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "estree_test262",
            })
        };

        let acorn_jsx_files = {
            let suite = AcornJsxSuite::<EstreeJsxCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "estree_acorn_jsx",
            })
        };

        let typescript_files = {
            let suite = TypeScriptSuite::<EstreeTypescriptCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "estree_typescript",
            })
        };

        Test262Suite::<EstreeTest262Case>::new().run_with_discovered_files(
            "estree_test262",
            self,
            &test262_files,
        );
        AcornJsxSuite::<EstreeJsxCase>::new().run_with_discovered_files(
            "estree_acorn_jsx",
            self,
            &acorn_jsx_files,
        );
        TypeScriptSuite::<EstreeTypescriptCase>::new().run_with_discovered_files(
            "estree_typescript",
            self,
            &typescript_files,
        );
    }

    /// # Panics
    pub fn run_runtime(&self) {
        let path = workspace_root().join("src/runtime/runtime.js").to_string_lossy().to_string();
        let mut runtime_process = Command::new("node")
            .args(["--experimental-vm-modules", &path])
            .spawn()
            .expect("Run runtime.js failed");
        Test262Suite::<Test262RuntimeCase>::new().run_async(self);
        let _ = runtime_process.wait();
        let _ = runtime_process.kill();
    }

    pub fn run_minifier(&self) {
        let test262_files = {
            let suite = Test262Suite::<MinifierTest262Case>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "minifier_test262",
            })
        };

        let babel_files = {
            let suite = BabelSuite::<MinifierBabelCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "minifier_babel",
            })
        };

        let node_compat_files = {
            let suite = NodeCompatSuite::<MinifierNodeCompatCase>::new();
            DiscoveredFiles::discover(&FileDiscoveryConfig {
                test_root: suite.get_test_root(),
                filter: self.filter.as_deref(),
                skip_test_path: Box::new(|path| suite.skip_test_path(path)),
                skip_test_crawl: suite.skip_test_crawl(),
                suite_name: "minifier_node_compat",
            })
        };

        Test262Suite::<MinifierTest262Case>::new().run_with_discovered_files(
            "minifier_test262",
            self,
            &test262_files,
        );
        BabelSuite::<MinifierBabelCase>::new().run_with_discovered_files(
            "minifier_babel",
            self,
            &babel_files,
        );
        NodeCompatSuite::<MinifierNodeCompatCase>::new().run_with_discovered_files(
            "minifier_node_compat",
            self,
            &node_compat_files,
        );
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    AppArgs::default().run_default()
}
