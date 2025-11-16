#![expect(clippy::print_stdout, clippy::disallowed_methods)]

// Core
pub mod discovery;
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

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use oxc_tasks_common::project_root;
use rayon::prelude::*;
use runtime::Test262RuntimeCase;
use tools::estree::{AcornJsxSuite, EstreeJsxCase, EstreeTypescriptCase};

use crate::{
    babel::{BabelCase, BabelSuite},
    discovery::DiscoveryCache,
    driver::Driver,
    misc::{MiscCase, MiscSuite},
    node_compat_table::NodeCompatSuite,
    suite::{Case, Suite},
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
    /// Cache for discovered test files
    pub discovery_cache: DiscoveryCache,
}

impl AppArgs {
    fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }

    /// Helper to get cached test262 file paths
    fn get_test262_paths(&self, skip_path: impl Fn(&Path) -> bool) -> &[PathBuf] {
        self.discovery_cache.get_test262_paths(self.filter.as_deref(), &skip_path)
    }

    /// Helper to get cached babel file paths
    fn get_babel_paths(&self, skip_path: impl Fn(&Path) -> bool) -> &[PathBuf] {
        self.discovery_cache.get_babel_paths(self.filter.as_deref(), &skip_path)
    }

    /// Helper to get cached typescript file paths
    fn get_typescript_paths(&self, skip_path: impl Fn(&Path) -> bool) -> &[PathBuf] {
        self.discovery_cache.get_typescript_paths(self.filter.as_deref(), &skip_path)
    }

    /// Helper to get cached misc file paths
    fn get_misc_paths(&self, skip_path: impl Fn(&Path) -> bool) -> &[PathBuf] {
        self.discovery_cache.get_misc_paths(self.filter.as_deref(), &skip_path)
    }

    /// Helper to run a suite with cached file paths
    ///
    /// Files are read on-demand when creating test cases, minimizing memory usage.
    fn run_suite_with_paths<T: Case, S: Suite<T>>(
        &self,
        mut suite: S,
        paths: &[PathBuf],
        name: &str,
    ) {
        suite.read_test_cases_from_paths(paths, self);
        if self.debug {
            suite.get_test_cases_mut().iter_mut().for_each(|case| {
                println!("{}", case.path().to_string_lossy());
                case.run();
            });
        } else {
            suite.get_test_cases_mut().par_iter_mut().for_each(Case::run);
        }
        suite.run_coverage(name, self);
    }

    /// Process all tools for test262 suite sequentially
    fn process_test262_suite(&self) {
        let paths =
            self.get_test262_paths(|path| Test262Suite::<Test262Case>::new().skip_test_path(path));

        // Parser
        self.run_suite_with_paths(Test262Suite::<Test262Case>::new(), paths, "parser_test262");

        // Semantic
        let paths = self.get_test262_paths(|path| {
            Test262Suite::<SemanticTest262Case>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            Test262Suite::<SemanticTest262Case>::new(),
            paths,
            "semantic_test262",
        );

        // Codegen
        let paths = self.get_test262_paths(|path| {
            Test262Suite::<CodegenTest262Case>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            Test262Suite::<CodegenTest262Case>::new(),
            paths,
            "codegen_test262",
        );

        // Formatter
        let paths = self.get_test262_paths(|path| {
            Test262Suite::<FormatterTest262Case>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            Test262Suite::<FormatterTest262Case>::new(),
            paths,
            "formatter_test262",
        );

        // Transformer
        let paths = self.get_test262_paths(|path| {
            Test262Suite::<TransformerTest262Case>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            Test262Suite::<TransformerTest262Case>::new(),
            paths,
            "transformer_test262",
        );

        // Minifier
        let paths = self.get_test262_paths(|path| {
            Test262Suite::<MinifierTest262Case>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            Test262Suite::<MinifierTest262Case>::new(),
            paths,
            "minifier_test262",
        );

        // Estree
        let paths = self.get_test262_paths(|path| {
            Test262Suite::<EstreeTest262Case>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            Test262Suite::<EstreeTest262Case>::new(),
            paths,
            "estree_test262",
        );
    }

    /// Process all tools for babel suite sequentially
    fn process_babel_suite(&self) {
        // Parser
        let paths =
            self.get_babel_paths(|path| BabelSuite::<BabelCase>::new().skip_test_path(path));
        self.run_suite_with_paths(BabelSuite::<BabelCase>::new(), paths, "parser_babel");

        // Semantic
        let paths = self
            .get_babel_paths(|path| BabelSuite::<SemanticBabelCase>::new().skip_test_path(path));
        self.run_suite_with_paths(BabelSuite::<SemanticBabelCase>::new(), paths, "semantic_babel");

        // Codegen
        let paths =
            self.get_babel_paths(|path| BabelSuite::<CodegenBabelCase>::new().skip_test_path(path));
        self.run_suite_with_paths(BabelSuite::<CodegenBabelCase>::new(), paths, "codegen_babel");

        // Formatter
        let paths = self
            .get_babel_paths(|path| BabelSuite::<FormatterBabelCase>::new().skip_test_path(path));
        self.run_suite_with_paths(
            BabelSuite::<FormatterBabelCase>::new(),
            paths,
            "formatter_babel",
        );

        // Transformer
        let paths = self
            .get_babel_paths(|path| BabelSuite::<TransformerBabelCase>::new().skip_test_path(path));
        self.run_suite_with_paths(
            BabelSuite::<TransformerBabelCase>::new(),
            paths,
            "transformer_babel",
        );

        // Minifier
        let paths = self
            .get_babel_paths(|path| BabelSuite::<MinifierBabelCase>::new().skip_test_path(path));
        self.run_suite_with_paths(BabelSuite::<MinifierBabelCase>::new(), paths, "minifier_babel");
    }

    /// Process all tools for typescript suite sequentially
    fn process_typescript_suite(&self) {
        // Parser
        let paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<TypeScriptCase>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            TypeScriptSuite::<TypeScriptCase>::new(),
            paths,
            "parser_typescript",
        );

        // Semantic
        let paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<SemanticTypeScriptCase>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            TypeScriptSuite::<SemanticTypeScriptCase>::new(),
            paths,
            "semantic_typescript",
        );

        // Codegen
        let paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<CodegenTypeScriptCase>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            TypeScriptSuite::<CodegenTypeScriptCase>::new(),
            paths,
            "codegen_typescript",
        );

        // Formatter
        let paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<FormatterTypeScriptCase>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            TypeScriptSuite::<FormatterTypeScriptCase>::new(),
            paths,
            "formatter_typescript",
        );

        // Transformer
        let paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<TransformerTypeScriptCase>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            TypeScriptSuite::<TransformerTypeScriptCase>::new(),
            paths,
            "transformer_typescript",
        );

        // Estree
        let paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<EstreeTypescriptCase>::new().skip_test_path(path)
        });
        self.run_suite_with_paths(
            TypeScriptSuite::<EstreeTypescriptCase>::new(),
            paths,
            "estree_typescript",
        );
    }

    /// Process all tools for misc suite sequentially
    fn process_misc_suite(&self) {
        // Parser
        let paths = self.get_misc_paths(|path| MiscSuite::<MiscCase>::new().skip_test_path(path));
        self.run_suite_with_paths(MiscSuite::<MiscCase>::new(), paths, "parser_misc");

        // Semantic
        let paths =
            self.get_misc_paths(|path| MiscSuite::<SemanticMiscCase>::new().skip_test_path(path));
        self.run_suite_with_paths(MiscSuite::<SemanticMiscCase>::new(), paths, "semantic_misc");

        // Codegen
        let paths =
            self.get_misc_paths(|path| MiscSuite::<CodegenMiscCase>::new().skip_test_path(path));
        self.run_suite_with_paths(MiscSuite::<CodegenMiscCase>::new(), paths, "codegen_misc");

        // Formatter
        let paths =
            self.get_misc_paths(|path| MiscSuite::<FormatterMiscCase>::new().skip_test_path(path));
        self.run_suite_with_paths(MiscSuite::<FormatterMiscCase>::new(), paths, "formatter_misc");

        // Transformer
        let paths = self
            .get_misc_paths(|path| MiscSuite::<TransformerMiscCase>::new().skip_test_path(path));
        self.run_suite_with_paths(
            MiscSuite::<TransformerMiscCase>::new(),
            paths,
            "transformer_misc",
        );
    }

    pub fn run_default(&self) {
        // Process each suite sequentially - only one suite in memory at a time
        self.process_test262_suite();
        self.process_babel_suite();
        self.process_typescript_suite();
        self.process_misc_suite();

        // Handle special cases that don't fit the standard suite pattern
        self.run_transpiler();
        AcornJsxSuite::<EstreeJsxCase>::new().run("estree_acorn_jsx", self);
        NodeCompatSuite::<MinifierNodeCompatCase>::new().run("minifier_node_compat", self);
    }

    pub fn run_parser(&self) {
        // Use cached file discovery for all suites - paths discovered once, reused across all tools
        let test262_paths =
            self.get_test262_paths(|path| Test262Suite::<Test262Case>::new().skip_test_path(path));
        let babel_paths =
            self.get_babel_paths(|path| BabelSuite::<BabelCase>::new().skip_test_path(path));
        let typescript_paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<TypeScriptCase>::new().skip_test_path(path)
        });
        let misc_paths =
            self.get_misc_paths(|path| MiscSuite::<MiscCase>::new().skip_test_path(path));

        // Run suites with cached paths
        self.run_suite_with_paths(
            Test262Suite::<Test262Case>::new(),
            test262_paths,
            "parser_test262",
        );
        self.run_suite_with_paths(BabelSuite::<BabelCase>::new(), babel_paths, "parser_babel");
        self.run_suite_with_paths(
            TypeScriptSuite::<TypeScriptCase>::new(),
            typescript_paths,
            "parser_typescript",
        );
        self.run_suite_with_paths(MiscSuite::<MiscCase>::new(), misc_paths, "parser_misc");
    }

    pub fn run_semantic(&self) {
        let test262_paths = self.get_test262_paths(|path| {
            Test262Suite::<SemanticTest262Case>::new().skip_test_path(path)
        });
        let babel_paths = self
            .get_babel_paths(|path| BabelSuite::<SemanticBabelCase>::new().skip_test_path(path));
        let typescript_paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<SemanticTypeScriptCase>::new().skip_test_path(path)
        });
        let misc_paths =
            self.get_misc_paths(|path| MiscSuite::<SemanticMiscCase>::new().skip_test_path(path));

        self.run_suite_with_paths(
            Test262Suite::<SemanticTest262Case>::new(),
            test262_paths,
            "semantic_test262",
        );
        self.run_suite_with_paths(
            BabelSuite::<SemanticBabelCase>::new(),
            babel_paths,
            "semantic_babel",
        );
        self.run_suite_with_paths(
            TypeScriptSuite::<SemanticTypeScriptCase>::new(),
            typescript_paths,
            "semantic_typescript",
        );
        self.run_suite_with_paths(
            MiscSuite::<SemanticMiscCase>::new(),
            misc_paths,
            "semantic_misc",
        );
    }

    pub fn run_codegen(&self) {
        let test262_paths = self.get_test262_paths(|path| {
            Test262Suite::<CodegenTest262Case>::new().skip_test_path(path)
        });
        let babel_paths =
            self.get_babel_paths(|path| BabelSuite::<CodegenBabelCase>::new().skip_test_path(path));
        let typescript_paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<CodegenTypeScriptCase>::new().skip_test_path(path)
        });
        let misc_paths =
            self.get_misc_paths(|path| MiscSuite::<CodegenMiscCase>::new().skip_test_path(path));

        self.run_suite_with_paths(
            Test262Suite::<CodegenTest262Case>::new(),
            test262_paths,
            "codegen_test262",
        );
        self.run_suite_with_paths(
            BabelSuite::<CodegenBabelCase>::new(),
            babel_paths,
            "codegen_babel",
        );
        self.run_suite_with_paths(
            TypeScriptSuite::<CodegenTypeScriptCase>::new(),
            typescript_paths,
            "codegen_typescript",
        );
        self.run_suite_with_paths(MiscSuite::<CodegenMiscCase>::new(), misc_paths, "codegen_misc");
    }

    pub fn run_formatter(&self) {
        let test262_paths = self.get_test262_paths(|path| {
            Test262Suite::<FormatterTest262Case>::new().skip_test_path(path)
        });
        let babel_paths = self
            .get_babel_paths(|path| BabelSuite::<FormatterBabelCase>::new().skip_test_path(path));
        let typescript_paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<FormatterTypeScriptCase>::new().skip_test_path(path)
        });
        let misc_paths =
            self.get_misc_paths(|path| MiscSuite::<FormatterMiscCase>::new().skip_test_path(path));

        self.run_suite_with_paths(
            Test262Suite::<FormatterTest262Case>::new(),
            test262_paths,
            "formatter_test262",
        );
        self.run_suite_with_paths(
            BabelSuite::<FormatterBabelCase>::new(),
            babel_paths,
            "formatter_babel",
        );
        self.run_suite_with_paths(
            TypeScriptSuite::<FormatterTypeScriptCase>::new(),
            typescript_paths,
            "formatter_typescript",
        );
        self.run_suite_with_paths(
            MiscSuite::<FormatterMiscCase>::new(),
            misc_paths,
            "formatter_misc",
        );
    }

    pub fn run_transformer(&self) {
        let test262_paths = self.get_test262_paths(|path| {
            Test262Suite::<TransformerTest262Case>::new().skip_test_path(path)
        });
        let babel_paths = self
            .get_babel_paths(|path| BabelSuite::<TransformerBabelCase>::new().skip_test_path(path));
        let typescript_paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<TransformerTypeScriptCase>::new().skip_test_path(path)
        });
        let misc_paths = self
            .get_misc_paths(|path| MiscSuite::<TransformerMiscCase>::new().skip_test_path(path));

        self.run_suite_with_paths(
            Test262Suite::<TransformerTest262Case>::new(),
            test262_paths,
            "transformer_test262",
        );
        self.run_suite_with_paths(
            BabelSuite::<TransformerBabelCase>::new(),
            babel_paths,
            "transformer_babel",
        );
        self.run_suite_with_paths(
            TypeScriptSuite::<TransformerTypeScriptCase>::new(),
            typescript_paths,
            "transformer_typescript",
        );
        self.run_suite_with_paths(
            MiscSuite::<TransformerMiscCase>::new(),
            misc_paths,
            "transformer_misc",
        );
    }

    pub fn run_transpiler(&self) {
        TranspileRunner::<TypeScriptTranspileCase>::new().run("transpile", self);
    }

    pub fn run_estree(&self) {
        let test262_paths = self.get_test262_paths(|path| {
            Test262Suite::<EstreeTest262Case>::new().skip_test_path(path)
        });
        let typescript_paths = self.get_typescript_paths(|path| {
            TypeScriptSuite::<EstreeTypescriptCase>::new().skip_test_path(path)
        });

        self.run_suite_with_paths(
            Test262Suite::<EstreeTest262Case>::new(),
            test262_paths,
            "estree_test262",
        );
        AcornJsxSuite::<EstreeJsxCase>::new().run("estree_acorn_jsx", self);
        self.run_suite_with_paths(
            TypeScriptSuite::<EstreeTypescriptCase>::new(),
            typescript_paths,
            "estree_typescript",
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
        let test262_paths = self.get_test262_paths(|path| {
            Test262Suite::<MinifierTest262Case>::new().skip_test_path(path)
        });
        let babel_paths = self
            .get_babel_paths(|path| BabelSuite::<MinifierBabelCase>::new().skip_test_path(path));

        self.run_suite_with_paths(
            Test262Suite::<MinifierTest262Case>::new(),
            test262_paths,
            "minifier_test262",
        );
        self.run_suite_with_paths(
            BabelSuite::<MinifierBabelCase>::new(),
            babel_paths,
            "minifier_babel",
        );
        NodeCompatSuite::<MinifierNodeCompatCase>::new().run("minifier_node_compat", self);
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    AppArgs::default().run_default()
}
