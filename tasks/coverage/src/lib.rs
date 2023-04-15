#![feature(let_chains, is_some_and)]

mod babel;
mod printer;
mod suite;
mod test262;
mod typescript;

use std::path::PathBuf;

pub use crate::babel::{BabelCase, BabelSuite};
pub use crate::printer::{PrinterBabelCase, PrinterTest262Case};
pub use crate::suite::Suite;
pub use crate::test262::{Test262Case, Test262Suite};
pub use crate::typescript::{TypeScriptCase, TypeScriptSuite};

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
}
