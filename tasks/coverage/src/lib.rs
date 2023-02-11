mod suite;
mod test262;

use std::{
    env,
    path::{Path, PathBuf},
};

pub use crate::suite::Suite;
pub use crate::test262::{Test262Case, Test262Suite};

/// # Panics
/// Invalid Project Root
#[must_use]
pub fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(2)
    .unwrap()
    .to_path_buf()
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub filter: Option<String>,
    pub detail: bool,
    /// Print mismatch diff
    pub diff: bool,
}

impl AppArgs {
    const fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }
}
