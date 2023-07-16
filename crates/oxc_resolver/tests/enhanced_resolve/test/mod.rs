mod extension_alias;
mod extensions;
mod resolve;
mod simple;

use std::{env, path::PathBuf};

pub fn fixture() -> PathBuf {
    env::current_dir().unwrap().join("tests/enhanced_resolve/test/fixtures")
}
