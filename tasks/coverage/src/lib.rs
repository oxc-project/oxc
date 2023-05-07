#![feature(let_chains)]

mod babel;
mod formatter;
mod minifier;
mod suite;
mod test262;
mod typescript;

use std::path::PathBuf;

use crate::babel::{BabelCase, BabelSuite};
use crate::formatter::{FormatterBabelCase, FormatterTest262Case};
use crate::minifier::{MinifierBabelCase, MinifierTest262Case};
use crate::suite::Suite;
use crate::test262::{Test262Case, Test262Suite};
use crate::typescript::{TypeScriptCase, TypeScriptSuite};

/// # Panics
/// Invalid Project Root
#[must_use]
pub fn project_root() -> PathBuf {
    project_root::get_project_root().unwrap()
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub filter: Option<String>,
    pub detail: bool,
    /// Print mismatch diff
    pub diff: bool,
}

impl AppArgs {
    fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }

    pub fn run_all(&self) {
        self.run_test262();
        self.run_babel();
        self.run_typescript();
        self.run_formatter();
        self.run_minifier();
    }

    pub fn run_test262(&self) {
        Test262Suite::<Test262Case>::new().run("Test262", self);
    }

    pub fn run_babel(&self) {
        BabelSuite::<BabelCase>::new().run("Babel", self);
    }

    pub fn run_typescript(&self) {
        TypeScriptSuite::<TypeScriptCase>::new().run("TypeScript", self);
    }

    pub fn run_formatter(&self) {
        Test262Suite::<FormatterTest262Case>::new().run("Formatter_Test262", self);
        BabelSuite::<FormatterBabelCase>::new().run("Formatter_Babel", self);
    }

    pub fn run_minifier(&self) {
        Test262Suite::<MinifierTest262Case>::new().run("Minifier_Test262", self);
        BabelSuite::<MinifierBabelCase>::new().run("Minifier_Babel", self);
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    let args = AppArgs { filter: None, detail: false, diff: false };
    args.run_all()
}
