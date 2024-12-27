use std::path::{Path, PathBuf};
use std::process::Command;

use oxc::span::SourceType;

use crate::workspace_root;
use crate::{
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
    Driver,
};

pub struct EsbuildTest262Case {
    base: Test262Case,
    workspace_root: PathBuf,
}

impl Case for EsbuildTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code), workspace_root: workspace_root() }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.should_fail() || self.base.skip_test_case()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let source_type = SourceType::default().with_module(is_module);

        let mut driver =
            Driver { compress: true, codegen: true, remove_whitespace: true, ..Driver::default() };
        driver.run(source_text, source_type);
        let oxc = driver.printed;

        if oxc.is_empty() {
            self.base.set_result(TestResult::Passed);
            return;
        }

        let path = self.workspace_root.join(self.path());
        let esbuild = Command::new("esbuild")
            .arg("--minify-whitespace=true")
            .arg("--minify-syntax=true")
            .arg("--keep-names")
            .arg(&path)
            .output()
            .unwrap();

        if !esbuild.stderr.is_empty() {
            self.base.set_result(TestResult::Passed);
            return;
        }
        let esbuild = String::from_utf8(esbuild.stdout).unwrap();

        if esbuild.len() >= oxc.len() {
            self.base.set_result(TestResult::Passed);
            return;
        }

        let diff = oxc.len() - esbuild.len();

        if diff < 10 {
            println!("\n{}\n", self.base.path().to_string_lossy());
            println!("\n{oxc}\n");
            println!("\n{esbuild}\n");
        }

        self.base.set_result(TestResult::GenericError(">> ", format!("{diff}")));
    }
}
