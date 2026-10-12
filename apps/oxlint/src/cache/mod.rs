//! ESLint-style `--cache`: lint only files that changed since the last run, and replay the
//! stored diagnostics of the others.
//!
//! The whole feature lives in this module. `lint.rs` only calls [`LintCache::open`],
//! [`LintCache::begin`] and [`ActiveLintCache::finish`].
//!
//! A file's cached result is reused when the file is unchanged (size and modification time,
//! or a content hash with `--cache-strategy content`) and the settings fingerprint is
//! unchanged. Like ESLint, dependencies between files are not tracked.
//!
//! Results of linted files are captured by intercepting the diagnostic channel, so
//! diagnostics from every source (parser, rules, JS plugins, unused directives, tsgolint)
//! are cached.

mod diagnostic;
mod store;
#[cfg(test)]
mod test;

use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::{
    DiagnosticSender, DiagnosticService, Error, NamedSource, Severity, SourceCode,
};
use oxc_linter::OxlintSuppressionFileAction;

use crate::cli::{CacheOptions, CacheStrategy};

use diagnostic::CachedDiagnostic;
use store::{CacheEntry, CacheFile, FileStat, cache_path, hash_bytes, settings_hash};

/// Cache state between loading the cache file and partitioning the files to lint.
pub struct LintCache(Option<Session>);

struct Session {
    path: PathBuf,
    cwd: PathBuf,
    strategy: CacheStrategy,
    settings: String,
    entries: BTreeMap<String, CacheEntry>,
}

/// Cache state while linting. Call [`ActiveLintCache::finish`] once all diagnostics are sent.
pub struct ActiveLintCache(Option<Active>);

struct Active {
    session: Session,
    /// Keys of files whose cached results were replayed.
    hits: FxHashSet<String>,
    /// Files being linted, by key.
    misses: FxHashMap<String, Miss>,
    /// Receives diagnostics of the linted files.
    rx_error: mpsc::Receiver<Vec<Error>>,
    /// Forwards diagnostics to the diagnostic service.
    tx_error: DiagnosticSender,
}

struct Miss {
    path: Arc<OsStr>,
    /// The file's state before linting. `None` if the file cannot be cached.
    stat: Option<FileStat>,
    hash: Option<String>,
}

enum Lookup {
    Hit { key: String, diagnostics: Vec<Error> },
    Miss { key: String, miss: Miss },
}

impl LintCache {
    /// Load the cache file if `--cache` is enabled. Otherwise delete it, like ESLint.
    ///
    /// `settings` must cover everything besides the file itself that affects lint results.
    pub fn open(
        options: &CacheOptions,
        cwd: &Path,
        suppression_file_action: &OxlintSuppressionFileAction,
        tsconfig: Option<&Path>,
        settings: &dyn Debug,
    ) -> Self {
        let path = cache_path(options.cache_location.as_deref(), cwd);
        if !options.cache {
            if path.is_file() {
                let _ = fs::remove_file(&path);
            }
            return Self(None);
        }
        // Cached results are stored after suppressions are applied, and updating or checking
        // suppressions needs the results of every file. So bypass the cache when suppressions
        // are in use.
        if *suppression_file_action != OxlintSuppressionFileAction::None {
            return Self(None);
        }
        let settings = settings_hash(settings, tsconfig);
        let entries = CacheFile::load(&path, &settings);
        Self(Some(Session {
            path,
            cwd: cwd.to_path_buf(),
            strategy: options.cache_strategy,
            settings,
            entries,
        }))
    }

    /// Replay cached results to `tx_error`, and return the files that still need linting
    /// with the sender to report their diagnostics to.
    pub fn begin(
        self,
        files: Vec<Arc<OsStr>>,
        tx_error: DiagnosticSender,
        fix: bool,
        silent: bool,
    ) -> (Vec<Arc<OsStr>>, DiagnosticSender, ActiveLintCache) {
        let Some(session) = self.0 else {
            return (files, tx_error, ActiveLintCache(None));
        };

        let lookups =
            files.into_par_iter().map(|path| session.lookup(path, fix, silent)).collect::<Vec<_>>();

        let mut files_to_lint = vec![];
        let mut hits = FxHashSet::default();
        let mut misses = FxHashMap::default();
        for lookup in lookups {
            match lookup {
                Lookup::Hit { key, diagnostics } => {
                    if !diagnostics.is_empty() {
                        let _ = tx_error.send(diagnostics);
                    }
                    hits.insert(key);
                }
                Lookup::Miss { key, miss } => {
                    files_to_lint.push(Arc::clone(&miss.path));
                    misses.insert(key, miss);
                }
            }
        }

        let (tx_lint, rx_error) = mpsc::channel();
        let active = Active { session, hits, misses, rx_error, tx_error };
        (files_to_lint, tx_lint, ActiveLintCache(Some(active)))
    }
}

impl Session {
    fn lookup(&self, path: Arc<OsStr>, fix: bool, silent: bool) -> Lookup {
        let key = file_key(&self.cwd, Path::new(&path));
        let stat = FileStat::read(Path::new(&path));
        let contents = match self.strategy {
            CacheStrategy::Metadata => None,
            CacheStrategy::Content => stat.and_then(|_| fs::read(Path::new(&path)).ok()),
        };
        let hash = contents.as_deref().map(hash_bytes);

        let entry = self.entries.get(&key).filter(|entry| {
            let unchanged = match self.strategy {
                CacheStrategy::Metadata => stat == Some(entry.stat()),
                CacheStrategy::Content => {
                    stat.is_some_and(|stat| stat.size == entry.size)
                        && hash.is_some()
                        && entry.hash == hash
                }
            };
            // Like ESLint, lint files with problems again when fixing, so they get fixed.
            unchanged && (!fix || entry.diagnostics.is_empty())
        });

        let Some(entry) = entry else {
            return Lookup::Miss { key, miss: Miss { path, stat, hash } };
        };
        if entry.diagnostics.is_empty() {
            return Lookup::Hit { key, diagnostics: vec![] };
        }

        // Renderers need the source text to show labels. It isn't printed with `--silent`.
        let source_text = if silent {
            Some(String::new())
        } else if let Some(contents) = contents {
            String::from_utf8(contents).ok()
        } else {
            fs::read_to_string(Path::new(&path)).ok()
        };
        let Some(source_text) = source_text else {
            return Lookup::Miss { key, miss: Miss { path, stat, hash } };
        };
        let source = Arc::new(NamedSource::new(&key, source_text));
        let diagnostics = entry
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.replay(Arc::clone(&source)))
            .collect();
        Lookup::Hit { key, diagnostics }
    }
}

impl ActiveLintCache {
    /// Record the results of linted files, forward them to the diagnostic service, and write
    /// the cache file.
    ///
    /// Must be called after all diagnostics are sent and the sender returned by
    /// [`LintCache::begin`] is dropped.
    #[expect(clippy::print_stderr)]
    pub fn finish(self) {
        let Some(Active { session, hits, misses, rx_error, tx_error }) = self.0 else {
            return;
        };

        let mut recorded = FxHashMap::<&str, Vec<CachedDiagnostic>>::default();
        // Set when a linted file may have failed without a diagnostic that names it,
        // in which case no new results are stored.
        let mut has_unattributed_error = false;
        for diagnostics in rx_error.try_iter() {
            for diagnostic in &diagnostics {
                let name = diagnostic.source_code().and_then(SourceCode::name);
                match name.and_then(|name| misses.get_key_value(name)) {
                    Some((key, _)) => recorded
                        .entry(key.as_str())
                        .or_default()
                        .push(CachedDiagnostic::from_diagnostic(diagnostic.as_ref())),
                    None if name.is_none_or(str::is_empty)
                        && !matches!(
                            diagnostic.severity(),
                            Some(Severity::Warning | Severity::Advice)
                        ) =>
                    {
                        has_unattributed_error = true;
                    }
                    None => {}
                }
            }
            let _ = tx_error.send(diagnostics);
        }
        // The diagnostic service runs until every sender is dropped.
        drop(tx_error);

        let Session { path, cwd, settings, mut entries, .. } = session;
        entries.retain(|key, _| {
            hits.contains(key) || (!misses.contains_key(key) && cwd.join(key).exists())
        });
        if !has_unattributed_error {
            let new_entries = misses
                .par_iter()
                .filter_map(|(key, miss)| {
                    let stat = miss.stat?;
                    // A file changed while linting (for example by `--fix`) is linted again
                    // next time, like ESLint does for fixed files.
                    if FileStat::read(Path::new(&miss.path)) != Some(stat) {
                        return None;
                    }
                    let entry = CacheEntry {
                        size: stat.size,
                        mtime: stat.mtime,
                        hash: miss.hash.clone(),
                        diagnostics: recorded.get(key.as_str()).cloned().unwrap_or_default(),
                    };
                    Some((key.clone(), entry))
                })
                .collect::<Vec<_>>();
            entries.extend(new_entries);
        }

        if let Err(error) = CacheFile::new(settings, entries).save(&path) {
            eprintln!("Failed to write cache file {}: {error}", path.display());
        }
    }
}

/// The name diagnostics of the file at `path` are reported under.
fn file_key(cwd: &Path, path: &Path) -> String {
    let source = DiagnosticService::named_source(cwd, path, "");
    source.name().unwrap_or_default().to_string()
}
