use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use log::debug;
use oxc_data_structures::rope::Rope;
use oxc_linter::{
    ConfigStore, LINTABLE_EXTENSIONS, TsGoLintState, loader::LINT_PARTIAL_LOADER_EXTENSIONS,
    read_to_string,
};
use rustc_hash::FxHashSet;
use std::time::Instant;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use crate::linter::error_with_position::{
    DiagnosticReport, generate_inverted_diagnostics, message_to_lsp_diagnostic,
};

pub struct TsgoLinter {
    state: TsGoLintState,
}

impl TsgoLinter {
    pub fn new(root_uri: &Path, config_store: ConfigStore) -> Self {
        let state = TsGoLintState::new(root_uri, config_store);
        Self { state }
    }

    pub fn lint_file(&self, uri: &Uri, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        let path = uri.to_file_path()?;

        if !Self::should_lint_path(&path) {
            return None;
        }

        let source_text = content.or_else(|| read_to_string(&path).ok())?;
        let rope = Rope::from_str(&source_text);

        // TODO: Avoid cloning the source text
        let t0 = Instant::now();
        let messages_group = self.state.lint_source(&[Arc::from(path.as_os_str())]).ok()?;
        let messages = messages_group.into_iter().next().map(|(_, v)| v)?;

        let mut diagnostics: Vec<DiagnosticReport> = messages
            .iter()
            .map(|e| message_to_lsp_diagnostic(e, uri, &source_text, &rope))
            .collect();

        let mut inverted_diagnostics = generate_inverted_diagnostics(&diagnostics, uri);
        diagnostics.append(&mut inverted_diagnostics);

        debug!(
            "[profile] tsgo single internal uri={:?} diagnostics={} ms={}",
            uri,
            diagnostics.len(),
            t0.elapsed().as_millis()
        );
        Some(diagnostics)
    }

    fn should_lint_path(path: &Path) -> bool {
        static WANTED_EXTENSIONS: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
        let wanted_exts = WANTED_EXTENSIONS.get_or_init(|| {
            LINTABLE_EXTENSIONS
                .iter()
                .filter(|ext| !LINT_PARTIAL_LOADER_EXTENSIONS.contains(ext))
                .copied()
                .collect()
        });

        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| wanted_exts.contains(ext))
    }

    /// Batch lint multiple URIs using a single tsgolint invocation.
    /// Returns vector of (Uri, DiagnosticReport) for each file with diagnostics.
    pub fn lint_batch(&self, uris: &[Uri]) -> Vec<(Uri, Vec<DiagnosticReport>)> {
        // Prepare eligible paths (filter out unsupported extensions early).
        let mut path_map: Vec<(Uri, PathBuf)> = Vec::with_capacity(uris.len());
        for uri in uris {
            if let Some(p) = uri.to_file_path() {
                let owned = p.into_owned();
                if Self::should_lint_path(&owned) {
                    path_map.push((uri.clone(), owned));
                }
            }
        }

        if path_map.is_empty() {
            return Vec::new();
        }

        let arcs: Vec<Arc<std::ffi::OsStr>> =
            path_map.iter().map(|(_, p)| Arc::from(p.as_os_str())).collect();
        // Collect simple metrics before invoking lint_source.
        let mut total_bytes: u64 = 0;
        let mut ext_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for (_, p) in &path_map {
            if let Ok(meta) = std::fs::metadata(p) {
                total_bytes += meta.len();
            }
            if let Some(ext) = p.extension().and_then(std::ffi::OsStr::to_str) {
                *ext_counts.entry(ext.to_string()).or_insert(0) += 1;
            }
        }
        let t0 = Instant::now();
        log::info!(
            "[tsgo] internal batch start eligible_paths={} total_bytes={} ext_stats={:?}",
            arcs.len(),
            total_bytes,
            ext_counts
        );
        let t_invoke = Instant::now();
        let batch_result = self.state.lint_source(&arcs);
        let invoke_ms = t_invoke.elapsed().as_millis();
        let mut out = Vec::new();

        match batch_result {
            Ok(grouped) => {
                for (path_buf, messages) in grouped.into_iter() {
                    // Map back to Uri by matching file path string
                    if let Some((uri, _)) =
                        path_map.iter().find(|(_, original)| original == &path_buf)
                    {
                        if let Ok(source_text) = read_to_string(&path_buf) {
                            let rope = Rope::from_str(&source_text);
                            let mut diagnostics: Vec<DiagnosticReport> = messages
                                .iter()
                                .map(|m| message_to_lsp_diagnostic(m, uri, &source_text, &rope))
                                .collect();
                            let mut inverted = generate_inverted_diagnostics(&diagnostics, uri);
                            diagnostics.append(&mut inverted);
                            out.push((uri.clone(), diagnostics));
                        }
                    }
                }
            }
            Err(_err) => {
                // On failure, degrade gracefully: no batch diagnostics (caller may fall back to sequential path).
            }
        }

        // Deterministic ordering
        out.sort_unstable_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        debug!(
            "[tsgo] internal batch done eligible_paths={} produced={} elapsed_ms_total={} invoke_ms={} total_bytes={}",
            path_map.len(),
            out.len(),
            t0.elapsed().as_millis(),
            invoke_ms,
            total_bytes
        );
        out
    }
}
