use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_linter::{RuntimeFileSystem, read_to_arena_str};
#[derive(Default)]
pub struct LspFileSystem {
    map: FxHashMap<PathBuf, Arc<str>>,
}

impl LspFileSystem {
    pub fn add_file(&mut self, path: PathBuf, content: Arc<str>) {
        self.map.insert(path, content);
    }
}

impl RuntimeFileSystem for LspFileSystem {
    fn read_to_arena_str<'a>(
        &'a self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error> {
        match self.map.get(path) {
            Some(s) => Ok(&**s),
            None => read_to_arena_str(path, allocator),
        }
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), std::io::Error> {
        panic!("writing file should not be allowed in Language Server");
    }
}
