use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_linter::RuntimeFileSystem;
use oxc_parser::Source;
use oxc_span::SourceType;
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
    /// Read the file at `path` into `allocator`, as a [`Source`].
    ///
    /// Files added with [`LspFileSystem::add_file`] are copied from memory.
    /// Other files are read from disk.
    fn read_source<'a>(
        &self,
        path: &Path,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Source<'a>> {
        match self.map.get(path) {
            Some(s) => Ok(Source::from_str(s, source_type, allocator)),
            None => Source::read_from_file_with_source_type(path, source_type, allocator),
        }
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), std::io::Error> {
        panic!("writing file should not be allowed in Language Server");
    }
}
