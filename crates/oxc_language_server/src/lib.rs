use rustc_hash::FxBuildHasher;

use crate::tester::Tester;

mod capabilities;
mod code_actions;
mod commands;
mod linter;
mod options;
mod tester;
mod worker;

type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;

const OXC_CONFIG_FILE: &str = ".oxlintrc.json";
#[tokio::main]
async fn main() {
    Tester::new("crates/oxc_language_server/fixtures/linter/oxc_resolver_memory_leak", None)
        .test_memory_leak("index.tsx", 100)
        .await;
}
