use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

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
        oxfmtrc_path: Option<&Path>,
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
        // Use multiple matchers, each with correct root for pattern resolution:
        // - Ignore files: root = parent directory of the ignore file
        // - `.ignorePatterns`: root = parent directory of `.oxfmtrc.json`
        // - Exclude paths (`!` prefix): root = cwd
        //
        // NOTE: Git ignore files are handled by `WalkBuilder` itself
        let mut matchers: Vec<Gitignore> = vec![];

        // 1. Handle formatter ignore files (`.prettierignore`, or `--ignore-path`)
        // Patterns are relative to the ignore file location
        for ignore_path in &load_ignore_paths(cwd, ignore_paths)? {
            let (gitignore, err) = Gitignore::new(ignore_path);
            if let Some(err) = err {
                return Err(format!(
                    "Failed to parse ignore file {}: {err}",
                    ignore_path.display()
                ));
            }
            matchers.push(gitignore);
        }

        // 2. Handle `oxfmtrc.ignorePatterns`
        // Patterns are relative to the config file location
        if !ignore_patterns.is_empty()
            && let Some(oxfmtrc_path) = oxfmtrc_path
        {
            let mut builder = GitignoreBuilder::new(
                oxfmtrc_path.parent().expect("`oxfmtrc_path` should have a parent directory"),
            );
            for pattern in ignore_patterns {
                if builder.add_line(None, pattern).is_err() {
                    return Err(format!(
                        "Failed to add ignore pattern `{pattern}` from `.ignorePatterns`"
                    ));
                }
            }
            let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
            matchers.push(gitignore);
        }

        // 3. Handle `!` prefixed paths
        // These are relative to cwd
        if !exclude_patterns.is_empty() {
            let mut builder = GitignoreBuilder::new(cwd);
            for pattern in &exclude_patterns {
                // Remove the leading `!` because `GitignoreBuilder` uses `!` as negation
                let pattern = pattern
                    .strip_prefix('!')
                    .expect("There should be a `!` prefix, already checked");
                if builder.add_line(None, pattern).is_err() {
                    return Err(format!(
                        "Failed to add ignore pattern `{pattern}` from `!` prefix"
                    ));
                }
            }
            let gitignore = builder.build().map_err(|_| "Failed to build ignores".to_string())?;
            matchers.push(gitignore);
        }

        //
        // Filter paths by formatter ignores
        //
        // NOTE: Base paths passed to `WalkBuilder` are not filtered by `filter_entry()`,
        // so we need to filter them here before passing to the walker.
        // This is needed for cases like `husky`, may specify ignored paths as staged files.
        // NOTE: Git ignored paths are not filtered here.
        let target_paths: Vec<_> = target_paths
            .into_iter()
            .filter(|path| !is_ignored(&matchers, path, path.is_dir()))
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
            if is_ignored(&matchers, entry.path(), is_dir) {
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
            // Do not respect `.ignore` file
            .ignore(false)
            // Do not search upward
            // NOTE: Prettier only searches current working directory
            .parents(false)
            // Also do not respect globals
            .git_global(false)
            // But respect downward nested `.gitignore` files
            // NOTE: Prettier does not: https://github.com/prettier/prettier/issues/4081
            .git_ignore(true)
            // Also do not respect `.git/info/exclude`
            .git_exclude(false)
            // Git is not required
            .require_git(false)
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

// ---

/// Check if a path should be ignored by any of the matchers.
/// A path is ignored if any matcher says it's ignored (and not whitelisted in that same matcher).
fn is_ignored(matchers: &[Gitignore], path: &Path, is_dir: bool) -> bool {
    for matcher in matchers {
        let matched = matcher.matched(path, is_dir);
        if matched.is_ignore() && !matched.is_whitelist() {
            return true;
        }
    }
    false
}

fn load_ignore_paths(cwd: &Path, ignore_paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    // If specified, resolve absolute paths and check existence
    if !ignore_paths.is_empty() {
        let mut result = Vec::with_capacity(ignore_paths.len());
        for path in ignore_paths {
            let path = if path.is_absolute() { path.clone() } else { cwd.join(path) };
            if !path.exists() {
                return Err(format!("{}: File not found", path.display()));
            }
            result.push(path);
        }
        return Ok(result);
    }

    // Else, search for default ignore files in cwd
    // These are optional, do not error if not found
    Ok(std::iter::once(".prettierignore")
        .filter_map(|file_name| {
            let path = cwd.join(file_name);
            path.exists().then_some(path)
        })
        .collect())
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
                    // Tier 2 = `.toml`, etc: Some files supported by `oxfmt` directly
                    // Tier 3 = `.html`, `.json`, etc: Other files supported by Prettier
                    // (Tier 4 = `.astro`, `.svelte`, etc: Other files supported by Prettier plugins)
                    // Everything else: Ignored
                    let Ok(strategy) = FormatFileStrategy::try_from(entry.into_path()) else {
                        return ignore::WalkState::Continue;
                    };

                    #[cfg(not(feature = "napi"))]
                    if !strategy.can_format_without_external() {
                        return ignore::WalkState::Continue;
                    }

                    // Send each entry immediately through the channel
                    // If send fails, the receiver has been dropped, so stop walking
                    if self.sender.send(strategy).is_err() {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            }
            Err(_err) => ignore::WalkState::Skip,
        }
    }
}
