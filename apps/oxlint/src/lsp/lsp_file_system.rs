use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_linter::RuntimeFileSystem;
use oxc_span::SourceCode;

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
    fn read_source_code<'a>(
        &self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<SourceCode<'a>, std::io::Error> {
        match self.map.get(path) {
            Some(s) => Ok(SourceCode::from_str_in(s, allocator)),
            None => SourceCode::read_in(path, allocator),
        }
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), std::io::Error> {
        panic!("writing file should not be allowed in Language Server");
    }
}
