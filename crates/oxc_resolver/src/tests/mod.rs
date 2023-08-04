mod alias;
mod browser_field;
mod exports_field;
mod extension_alias;
mod extensions;
mod fallback;
mod full_specified;
mod imports_field;
mod incorrect_description_file;
mod memory_fs;
mod resolve;
mod roots;
mod scoped_packages;
mod simple;
mod symlink;

use crate::Resolver;
use std::{env, path::PathBuf, sync::Arc, thread};

pub fn fixture() -> PathBuf {
    env::current_dir().unwrap().join("tests/enhanced_resolve/test/fixtures")
}

#[test]
fn threaded_environment() {
    let cwd = env::current_dir().unwrap();
    let resolver = Arc::new(Resolver::default());
    for _ in 0..2 {
        _ = thread::spawn({
            let cwd = cwd.clone();
            let resolver = Arc::clone(&resolver);
            move || {
                _ = resolver.resolve(cwd, ".");
            }
        })
        .join();
    }
}
