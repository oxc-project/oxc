use std::{ffi::OsStr, path::PathBuf, sync::mpsc};

use ignore::overrides::Override;

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub struct Walk {
    inner: ignore::WalkParallel,
    with_node_modules: bool,
}

impl Walk {
    /// Will not canonicalize paths.
    /// # Panics
    pub fn new(
        paths: &[PathBuf],
        override_builder: Option<Override>,
        with_node_modules: bool,
    ) -> Self {
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
        Self { inner, with_node_modules }
    }

    /// Stream entries through a channel as they are discovered
    pub fn stream_entries(self) -> mpsc::Receiver<WalkEntry> {
        let (sender, receiver) = mpsc::channel::<WalkEntry>();
        let with_node_modules = self.with_node_modules;

        // Spawn the walk operation in a separate thread
        rayon::spawn(move || {
            let mut builder = WalkBuilder { sender, with_node_modules };
            self.inner.visit(&mut builder);
            // Channel will be closed when builder is dropped
        });

        receiver
    }
}

pub struct WalkEntry {
    pub path: PathBuf,
    pub source_type: SourceType,
}

struct WalkBuilder {
    sender: mpsc::Sender<WalkEntry>,
    with_node_modules: bool,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor {
            sender: self.sender.clone(),
            with_node_modules: self.with_node_modules,
        })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<WalkEntry>,
    with_node_modules: bool,
}

impl WalkVisitor {
    // We are setting `.hidden(false)` on the `WalkBuilder`,
    // it means we want to include hidden files and directories.
    // However, we (and also Prettier) still skip traversing certain directories.
    // https://prettier.io/docs/ignore#ignoring-files-prettierignore
    fn is_skipped_dir(&self, dir_name: &OsStr) -> bool {
        dir_name == ".git"
            || dir_name == ".jj"
            || dir_name == ".sl"
            || dir_name == ".svn"
            || dir_name == ".hg"
            || (!self.with_node_modules && dir_name == "node_modules")
    }
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                let Some(file_type) = entry.file_type() else {
                    return ignore::WalkState::Continue;
                };

                if file_type.is_dir() {
                    if self.is_skipped_dir(entry.file_name()) {
                        return ignore::WalkState::Skip;
                    }
                    return ignore::WalkState::Continue;
                }

                if let Some(source_type) = get_supported_source_type(entry.path()) {
                    let walk_entry = WalkEntry { path: entry.path().to_path_buf(), source_type };
                    // Send each entry immediately through the channel
                    // If send fails, the receiver has been dropped, so stop walking
                    if self.sender.send(walk_entry).is_err() {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
