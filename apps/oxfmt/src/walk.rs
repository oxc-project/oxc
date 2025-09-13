use std::{ffi::OsStr, path::PathBuf, sync::Arc, sync::mpsc};

use ignore::overrides::Override;

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub struct Walk {
    inner: ignore::WalkParallel,
}

pub struct WalkEntry {
    pub path: Arc<OsStr>,
    pub source_type: SourceType,
}

struct WalkBuilder {
    sender: mpsc::Sender<Vec<WalkEntry>>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkCollector { entries: vec![], sender: self.sender.clone() })
    }
}

struct WalkCollector {
    entries: Vec<WalkEntry>,
    sender: mpsc::Sender<Vec<WalkEntry>>,
}

impl Drop for WalkCollector {
    fn drop(&mut self) {
        let entries = std::mem::take(&mut self.entries);
        self.sender.send(entries).unwrap();
    }
}

impl ignore::ParallelVisitor for WalkCollector {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                // Skip if we can't get file type or if it's a directory
                if let Some(file_type) = entry.file_type() {
                    if !file_type.is_dir() {
                        if let Some(source_type) = get_supported_source_type(entry.path()) {
                            self.entries.push(WalkEntry {
                                path: entry.path().as_os_str().into(),
                                source_type,
                            });
                        }
                    }
                }
                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
impl Walk {
    /// Will not canonicalize paths.
    /// # Panics
    pub fn new(paths: &[PathBuf], override_builder: Option<Override>) -> Self {
        assert!(!paths.is_empty(), "At least one path must be provided to Walk::new");

        let mut inner = ignore::WalkBuilder::new(
            paths
                .iter()
                .next()
                .expect("Expected paths parameter to Walk::new() to contain at least one path."),
        );

        if let Some(paths) = paths.get(1..) {
            for path in paths {
                inner.add(path);
            }
        }

        if let Some(override_builder) = override_builder {
            inner.overrides(override_builder);
        }

        // Do not follow symlinks like Prettier does.
        // See https://github.com/prettier/prettier/pull/14627
        let inner = inner.hidden(false).ignore(false).git_global(false).build_parallel();
        Self { inner }
    }

    pub fn collect_entries(self) -> Vec<WalkEntry> {
        let (sender, receiver) = mpsc::channel::<Vec<WalkEntry>>();
        let mut builder = WalkBuilder { sender };
        self.inner.visit(&mut builder);
        drop(builder);
        receiver.into_iter().flatten().collect()
    }
}
