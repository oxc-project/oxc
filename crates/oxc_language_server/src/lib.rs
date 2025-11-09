use rustc_hash::FxBuildHasher;

mod backend;
mod capabilities;
mod file_system;
mod formatter;
mod linter;
mod options;
mod tool;
mod utils;
mod worker;

pub use crate::backend::Backend;
pub use crate::linter::ServerLinter;
pub use crate::worker::WorkspaceWorker;

pub type ConcurrentHashMap<K, V> = papaya::HashMap<K, V, FxBuildHasher>;
