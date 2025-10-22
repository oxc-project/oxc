use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::{gitignore::GitignoreBuilder, overrides::OverrideBuilder};

use oxc_formatter::get_supported_source_type;
use oxc_span::SourceType;

pub struct Walk {
    inner: ignore::WalkParallel,
}

impl Walk {
    pub fn build(
        cwd: &PathBuf,
        paths: &[PathBuf],
        ignore_paths: &[PathBuf],
        with_node_modules: bool,
        ignore_patterns: &[String],
    ) -> Result<Self, String> {
        let (target_paths, exclude_patterns) = normalize_paths(cwd, paths);

        // Add all non-`!` prefixed paths to the walker base
        let mut inner = ignore::WalkBuilder::new(
            target_paths
                .first()
                .expect("Expected paths parameter to Walk::build() to contain at least one path."),
        );
        if let Some(paths) = target_paths.get(1..) {
            for path in paths {
                inner.add(path);
            }
        }

        // NOTE: We are using `OverrideBuilder` only for exclusion.
        // This means there is no way to "re-include" a file once ignored.

        // Treat all `!` prefixed patterns as overrides to exclude
        if !exclude_patterns.is_empty() {
            let mut builder = OverrideBuilder::new(cwd);
            for pattern_str in exclude_patterns {
                builder
                    .add(&pattern_str)
                    .map_err(|_| format!("{pattern_str} is not a valid glob for override."))?;
            }
            let overrides = builder.build().map_err(|_| "Failed to build overrides".to_string())?;
            inner.overrides(overrides);
        }

        // Handle ignore files
        let mut builder = GitignoreBuilder::new(cwd);
        for ignore_path in &load_ignore_paths(cwd, ignore_paths) {
            if builder.add(ignore_path).is_some() {
                return Err(format!("Failed to add ignore file: {}", ignore_path.display()));
            }
        }
        // Handle `config.ignorePatterns`
        for pattern in ignore_patterns {
            if builder.add_line(None, pattern).is_err() {
                return Err(format!("Failed to add ignore pattern `{pattern}`"));
            }
        }
        let ignores = builder.build().map_err(|_| "Failed to build ignores".to_string())?;

        // NOTE: If return `false` here, it will not be `visit()`ed at all
        inner.filter_entry(move |entry| {
            // Skip stdin for now
            let Some(file_type) = entry.file_type() else {
                return false;
            };

            let is_dir = file_type.is_dir();

            if is_dir {
                // We are setting `.hidden(false)` on the `WalkBuilder` below,
                // it means we want to include hidden files and directories.
                // However, we (and also Prettier) still skip traversing certain directories.
                // https://prettier.io/docs/ignore#ignoring-files-prettierignore
                let is_skipped_dir = {
                    let dir_name = entry.file_name();
                    dir_name == ".git"
                        || dir_name == ".jj"
                        || dir_name == ".sl"
                        || dir_name == ".svn"
                        || dir_name == ".hg"
                        || (!with_node_modules && dir_name == "node_modules")
                };
                if is_skipped_dir {
                    return false;
                }
            }

            // Check ignore files, patterns
            let matched = ignores.matched(entry.path(), is_dir);
            if matched.is_ignore() && !matched.is_whitelist() {
                return false;
            }

            // NOTE: We can also check `get_supported_source_type()` here to skip.
            // But we want to pass parsed `SourceType` to `FormatService`,
            // so we do it later in the visitor instead.

            true
        });

        let inner = inner
            // Do not follow symlinks like Prettier does.
            // See https://github.com/prettier/prettier/pull/14627
            .follow_links(false)
            // Include hidden files and directories except those we explicitly skip above
            .hidden(false)
            // Do not respect `.gitignore` automatically, we handle it manually
            .ignore(false)
            .parents(false)
            .git_global(false)
            .git_ignore(false)
            .git_exclude(false)
            .build_parallel();
        Ok(Self { inner })
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

/// Normalize user input paths into `target_paths` and `exclude_patterns`.
/// - `target_paths`: Absolute paths to format
/// - `exclude_patterns`: Pattern strings to exclude (with `!` prefix)
fn normalize_paths(cwd: &Path, input_paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<String>) {
    let mut target_paths = vec![];
    let mut exclude_patterns = vec![];

    for path in input_paths {
        let path_str = path.to_string_lossy();

        // Instead of `oxlint`'s `--ignore-pattern=PAT`,
        // `oxfmt` supports `!` prefix in paths like Prettier.
        if path_str.starts_with('!') {
            exclude_patterns.push(path_str.to_string());
            continue;
        }

        // Otherwise, treat as target path

        if path.is_absolute() {
            target_paths.push(path.clone());
            continue;
        }

        // NOTE: `.` and cwd behaves differently, need to normalize
        let path = if path_str == "." {
            cwd.to_path_buf()
        } else if let Some(stripped) = path_str.strip_prefix("./") {
            cwd.join(stripped)
        } else {
            cwd.join(path)
        };
        target_paths.push(path);
    }

    // Default to cwd if no `target_paths` are provided
    if target_paths.is_empty() {
        target_paths.push(cwd.into());
    }

    (target_paths, exclude_patterns)
}

fn load_ignore_paths(cwd: &Path, ignore_paths: &[PathBuf]) -> Vec<PathBuf> {
    // If specified, just resolves absolute paths
    if !ignore_paths.is_empty() {
        return ignore_paths
            .iter()
            .map(|path| if path.is_absolute() { path.clone() } else { cwd.join(path) })
            .collect();
    }

    // Else, search for default ignore files in cwd
    [".gitignore", ".prettierignore"]
        .iter()
        .filter_map(|file_name| {
            let path = cwd.join(file_name);
            if path.exists() { Some(path) } else { None }
        })
        .collect::<Vec<_>>()
}

// ---

pub struct WalkEntry {
    pub path: PathBuf,
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

                if !file_type.is_dir()
                    && let Some(source_type) = get_supported_source_type(entry.path())
                {
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
