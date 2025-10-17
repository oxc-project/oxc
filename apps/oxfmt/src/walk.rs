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
    sender: mpsc::Sender<WalkEntry>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor { sender: self.sender.clone() })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<WalkEntry>,
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                let Some(file_type) = entry.file_type() else {
                    return ignore::WalkState::Continue;
                };

                if file_type.is_dir() {
                    // We are setting `.hidden(false)` on the `WalkBuilder` below,
                    // it means we want to include hidden files and directories.
                    // However, we (and also Prettier) still skip traversing VCS directories.
                    // https://prettier.io/docs/ignore#ignoring-files-prettierignore
                    let dir_name = entry.file_name();
                    if matches!(dir_name.to_str(), Some(".git" | ".jj" | ".sl" | ".svn" | ".hg")) {
                        return ignore::WalkState::Skip;
                    }
                } else if let Some(source_type) = get_supported_source_type(entry.path()) {
                    let walk_entry =
                        WalkEntry { path: entry.path().as_os_str().into(), source_type };
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

impl Walk {
    /// Will not canonicalize paths.
    /// # Panics
    pub fn new(paths: &[PathBuf], override_builder: Option<Override>) -> Self {
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

    /// Stream entries through a channel as they are discovered
    pub fn stream_entries(self) -> mpsc::Receiver<WalkEntry> {
        let (sender, receiver) = mpsc::channel::<WalkEntry>();

        // Spawn the walk operation in a separate thread
        rayon::spawn(move || {
            let mut builder = WalkBuilder { sender };
            self.inner.visit(&mut builder);
            // Channel will be closed when builder is dropped
        });

        receiver
    }
}
