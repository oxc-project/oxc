use std::{
    ffi::OsStr,
    path::Path,
    sync::{Arc, mpsc},
};

use ignore::DirEntry;

use crate::linter::LINT_CONFIG_FILE;

pub struct ConfigWalker {
    inner: ignore::WalkParallel,
}

struct WalkBuilder {
    sender: mpsc::Sender<Vec<Arc<OsStr>>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkCollector { paths: vec![], sender: self.sender.clone() })
    }
}

struct WalkCollector {
    paths: Vec<Arc<OsStr>>,
    sender: mpsc::Sender<Vec<Arc<OsStr>>>,
}

impl Drop for WalkCollector {
    fn drop(&mut self) {
        let paths = std::mem::take(&mut self.paths);
        self.sender.send(paths).unwrap();
    }
}

impl ignore::ParallelVisitor for WalkCollector {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                if Self::is_wanted_entry(&entry) {
                    self.paths.push(entry.path().as_os_str().into());
                }
                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}

impl WalkCollector {
    fn is_wanted_entry(entry: &DirEntry) -> bool {
        let Some(file_type) = entry.file_type() else { return false };
        if file_type.is_dir() {
            return false;
        }
        let Some(file_name) = entry.path().file_name() else { return false };

        file_name == LINT_CONFIG_FILE
    }
}

impl ConfigWalker {
    /// Will not canonicalize paths.
    /// # Panics
    pub fn new(path: &Path) -> Self {
        let inner: ignore::WalkParallel = ignore::WalkBuilder::new(path)
            // disable skip hidden, which will not not search for files starting with a dot
            .hidden(false)
            // disable all gitignore features
            .parents(false)
            .ignore(false)
            .git_global(false)
            .follow_links(true)
            .build_parallel();

        Self { inner }
    }

    pub fn paths(self) -> Vec<Arc<OsStr>> {
        let (sender, receiver) = mpsc::channel::<Vec<Arc<OsStr>>>();
        let mut builder = WalkBuilder { sender };
        self.inner.visit(&mut builder);
        drop(builder);
        receiver.into_iter().flatten().collect()
    }
}
