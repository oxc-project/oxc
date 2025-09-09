use std::{ffi::OsStr, path::PathBuf, sync::Arc, sync::mpsc};

use ignore::overrides::Override;

use oxc_span::SourceType;

// Additional extensions from linguist-languages, which Prettier also supports
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/JavaScript.js
// No special extensions for TypeScript
// - https://github.com/ikatyang-collab/linguist-languages/blob/d1dc347c7ced0f5b42dd66c7d1c4274f64a3eb6b/data/TypeScript.js
const ADDITIONAL_JS_EXTENSIONS: &[&str] = &[
    "_js",
    "bones",
    "es",
    "es6",
    "frag",
    "gs",
    "jake",
    "javascript",
    "jsb",
    "jscad",
    "jsfl",
    "jslib",
    "jsm",
    "jspre",
    "jss",
    "njs",
    "pac",
    "sjs",
    "ssjs",
    "xsjs",
    "xsjslib",
];

fn is_supported_source_type(path: &std::path::Path) -> Option<SourceType> {
    let extension = path.extension()?.to_string_lossy();

    // Standard extensions, also supported by `oxc_span::VALID_EXTENSIONS`
    if let Ok(source_type) = SourceType::from_extension(&extension) {
        return Some(source_type);
    }
    // Additional extensions from linguist-languages, which Prettier also supports
    if ADDITIONAL_JS_EXTENSIONS.contains(&extension.as_ref()) {
        return Some(SourceType::default());
    }
    // `Jakefile` has no extension but is a valid JS file defined by linguist-languages
    if path.file_name() == Some(OsStr::new("Jakefile")) {
        return Some(SourceType::default());
    }

    None
}

// ---

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
                        if let Some(source_type) = is_supported_source_type(entry.path()) {
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

    pub fn entries(self) -> Vec<WalkEntry> {
        let (sender, receiver) = mpsc::channel::<Vec<WalkEntry>>();
        let mut builder = WalkBuilder { sender };
        self.inner.visit(&mut builder);
        drop(builder);
        receiver.into_iter().flatten().collect()
    }
}
