mod enhanced_resolve;
mod memory_fs;

use std::{env, sync::Arc, thread};

pub(crate) use memory_fs::MemoryFS;
use oxc_resolver::Resolver;

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
