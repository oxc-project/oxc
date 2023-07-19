mod alias;
mod browser_field;
mod extension_alias;
mod extensions;
mod fallback;
mod incorrect_description_file;
mod resolve;
mod scoped_packages;
mod simple;

use std::{env, path::PathBuf};

pub fn fixture() -> PathBuf {
    env::current_dir().unwrap().join("tests/enhanced_resolve/test/fixtures")
}
