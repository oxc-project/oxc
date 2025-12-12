use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::gitignore::GitignoreBuilder;

use crate::core::FormatFileStrategy;

pub struct Walk {
    inner: ignore::WalkParallel,
}

impl Walk {
    pub fn build(
        cwd: &Path,
        paths: &[PathBuf],
        ignore_paths: &[PathBuf],
        with_node_modules: bool,
        ignore_patterns: &[String],
    ) -> Result<Option<Self>, String> {
        //
        // Classify and normalize specified paths
        //
        let mut target_paths = vec![];
        let mut exclude_patterns = vec![];
        for path in paths {
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

            // NOTE: `.` and cwd behave differently, need to normalize
            let path = if path_str == "." {
                cwd.to_path_buf()
            } else if let Some(stripped) = path_str.strip_prefix("./") {
                cwd.join(stripped)
            } else {
                cwd.join(path)
            };
            target_paths.push(path);
        }
        // Default to cwd if no target paths are provided
        if target_paths.is_empty() {
            target_paths.push(cwd.to_path_buf());
        }

        //
        // Build ignores
        //
        let mut builder = GitignoreBuilder::new(cwd);
        // Handle ignore files
        for ignore_path in &load_ignore_paths(cwd, ignore_paths) {
            if builder.add(ignore_path).is_some() {
                return Err(format!("Failed to add ignore file: {}", ignore_path.display()));
            }
        }
        // Handle `config.ignorePatterns`
        for pattern in ignore_patterns {
            if builder.add_line(None, pattern).is_err() {
                return Err(format!(
                    "Failed to add ignore pattern `{pattern}` from `.ignorePatterns`"
                ));
            }
        }
        // Handle `!` prefixed paths as ignore patterns too
        for pattern in &exclude_patterns {
            // Remove the leading `!` because `GitignoreBuilder` uses `!` as negation
            let pattern =
                pattern.strip_prefix('!').expect("There should be a `!` prefix, already checked");
            if builder.add_line(None, pattern).is_err() {
                return Err(format!("Failed to add ignore pattern `{pattern}` from `!` prefix"));
            }
        }
        let ignores = builder.build().map_err(|_| "Failed to build ignores".to_string())?;

        //
        // Filter paths by ignores
        //
        // NOTE: Base paths passed to `WalkBuilder` are not filtered by `filter_entry()`,
        // so we need to filter them here before passing to the walker.
        let target_paths: Vec<_> = target_paths
            .into_iter()
            .filter(|path| {
                let matched = ignores.matched(path, path.is_dir());
                !matched.is_ignore() || matched.is_whitelist()
            })
            .collect();

        // If no target paths remain after filtering, return `None`.
        // Not an error, but nothing to format, leave it to the caller how to handle.
        let Some(first_path) = target_paths.first() else {
            return Ok(None);
        };

        // Add all non-`!` prefixed paths to the walker base
        let mut inner = ignore::WalkBuilder::new(first_path);
        for path in target_paths.iter().skip(1) {
            inner.add(path);
        }

        // NOTE: If return `false` here, it will not be `visit()`ed at all
        inner.filter_entry(move |entry| {
            let Some(file_type) = entry.file_type() else {
                return false;
            };
            let is_dir = file_type.is_dir();

            if is_dir {
                // We are setting `.hidden(false)` on the `WalkBuilder` below,
                // it means we want to include hidden files and directories.
                // However, we (and also Prettier) still skip traversing certain directories.
                // https://prettier.io/docs/ignore#ignoring-files-prettierignore
                let is_ignored_dir = {
                    let dir_name = entry.file_name();
                    dir_name == ".git"
                        || dir_name == ".jj"
                        || dir_name == ".sl"
                        || dir_name == ".svn"
                        || dir_name == ".hg"
                        || (!with_node_modules && dir_name == "node_modules")
                };
                if is_ignored_dir {
                    return false;
                }
            }

            // Check ignore files, patterns
            let matched = ignores.matched(entry.path(), is_dir);
            if matched.is_ignore() && !matched.is_whitelist() {
                return false;
            }

            // NOTE: In addition to ignoring based on ignore files and patterns here,
            // we also apply extra filtering in the visitor `visit()` below.
            // We need to return `bool` for `filter_entry()` here,
            // but we don't want to duplicate logic in the visitor again.
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
        Ok(Some(Self { inner }))
    }

    /// Stream entries through a channel as they are discovered
    pub fn stream_entries(self) -> mpsc::Receiver<FormatFileStrategy> {
        let (sender, receiver) = mpsc::channel::<FormatFileStrategy>();

        // Spawn the walk operation in a separate thread
        rayon::spawn(move || {
            let mut builder = WalkBuilder { sender };
            self.inner.visit(&mut builder);
            // Channel will be closed when builder is dropped
        });

        receiver
    }
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
        .into_iter()
        .filter_map(|file_name| {
            let path = cwd.join(file_name);
            path.exists().then_some(path)
        })
        .collect()
}

// ---

struct WalkBuilder {
    sender: mpsc::Sender<FormatFileStrategy>,
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for WalkBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(WalkVisitor { sender: self.sender.clone() })
    }
}

struct WalkVisitor {
    sender: mpsc::Sender<FormatFileStrategy>,
}

impl ignore::ParallelVisitor for WalkVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> ignore::WalkState {
        match entry {
            Ok(entry) => {
                let Some(file_type) = entry.file_type() else {
                    return ignore::WalkState::Continue;
                };

                // Use `is_file()` to detect symlinks to the directory named `.js`
                #[expect(clippy::filetype_is_file)]
                if file_type.is_file() {
                    // Determine this file should be handled or NOT
                    // Tier 1 = `.js`, `.tsx`, etc: JS/TS files supported by `oxc_formatter`
                    // Tier 2 = `.html`, `.json`, etc: Other files supported by Prettier
                    // (Tier 3 = `.astro`, `.svelte`, etc: Other files supported by Prettier plugins)
                    // Tier 4 = everything else: Not handled
                    let Ok(format_file_source) = FormatFileStrategy::try_from(entry.into_path())
                    else {
                        return ignore::WalkState::Continue;
                    };

                    #[cfg(not(feature = "napi"))]
                    if !matches!(format_file_source, FormatFileStrategy::OxcFormatter { .. }) {
                        return ignore::WalkState::Continue;
                    }

                    // Send each entry immediately through the channel
                    // If send fails, the receiver has been dropped, so stop walking
                    if self.sender.send(format_file_source).is_err() {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
