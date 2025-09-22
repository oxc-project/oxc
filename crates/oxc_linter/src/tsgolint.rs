use std::{
    collections::BTreeSet,
    ffi::OsStr,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic, Severity};
use oxc_span::{SourceType, Span};

use super::{AllowWarnDeny, ConfigStore, ResolvedLinterState, read_to_string};

#[cfg(feature = "language_server")]
use crate::{
    fixer::{CompositeFix, Message, PossibleFixes},
    lsp::{MessageWithPosition, message_to_message_with_position},
};

/// State required to initialize the `tsgolint` linter.
#[derive(Debug, Clone)]
pub struct TsGoLintState {
    /// The path to the `tsgolint` executable (at least our best guess at it).
    executable_path: PathBuf,
    /// Current working directory, used for rendering paths in diagnostics.
    cwd: PathBuf,
    /// The configuration store for `tsgolint` (used to resolve configurations outside of `oxc_linter`)
    config_store: ConfigStore,
    /// If `oxlint` will output the diagnostics or not.
    /// When `silent` is true, we do not need to access the file system for nice diagnostics messages.
    silent: bool,
}

impl TsGoLintState {
    pub fn new(cwd: &Path, config_store: ConfigStore) -> Self {
        let executable_path =
            try_find_tsgolint_executable(cwd).unwrap_or(PathBuf::from("tsgolint"));

        TsGoLintState { config_store, executable_path, cwd: cwd.to_path_buf(), silent: false }
    }

    /// Try to create a new TsGoLintState, returning an error if the executable cannot be found.
    ///
    /// # Errors
    /// Returns an error if the tsgolint executable cannot be found.
    pub fn try_new(cwd: &Path, config_store: ConfigStore) -> Result<Self, String> {
        let executable_path = try_find_tsgolint_executable(cwd)?;

        Ok(TsGoLintState { config_store, executable_path, cwd: cwd.to_path_buf(), silent: false })
    }

    /// Set to `true` to skip file system reads.
    /// When `silent` is true, we do not need to access the file system for nice diagnostics messages.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_silent(mut self, yes: bool) -> Self {
        self.silent = yes;
        self
    }

    /// # Panics
    /// - when `stdin` of subprocess cannot be opened
    /// - when `stdout` of subprocess cannot be opened
    /// - when `tsgolint` process cannot be awaited
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    pub fn lint(self, paths: &[Arc<OsStr>], error_sender: DiagnosticSender) -> Result<(), String> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();

        let json_input = self.json_input(paths, &mut resolved_configs);
        if json_input.configs.is_empty() {
            return Ok(());
        }

        let handler = std::thread::spawn(move || {
            let mut cmd = std::process::Command::new(&self.executable_path);
            cmd.arg("headless")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped());

            if let Ok(trace_file) = std::env::var("OXLINT_TSGOLINT_TRACE") {
                cmd.arg(format!("-trace={trace_file}"));
            }
            if let Ok(cpuprof_file) = std::env::var("OXLINT_TSGOLINT_CPU") {
                cmd.arg(format!("-cpuprof={cpuprof_file}"));
            }
            if let Ok(heap_file) = std::env::var("OXLINT_TSGOLINT_HEAP") {
                cmd.arg(format!("-heap={heap_file}"));
            }
            if let Ok(allocs_file) = std::env::var("OXLINT_TSGOLINT_ALLOCS") {
                cmd.arg(format!("-allocs={allocs_file}"));
            }

            let child = cmd.spawn();

            let mut child = match child {
                Ok(c) => c,
                Err(e) => {
                    return Err(format!(
                        "Failed to spawn tsgolint from path `{}`, with error: {e}",
                        self.executable_path.display()
                    ));
                }
            };

            let mut stdin = child.stdin.take().expect("Failed to open tsgolint stdin");

            // Write the input synchronously and handle BrokenPipe gracefully in case the child
            // exits early and closes its stdin.
            let json = serde_json::to_string(&json_input).expect("Failed to serialize JSON");
            if let Err(e) = stdin.write_all(json.as_bytes()) {
                // If the child closed stdin early, avoid crashing on SIGPIPE/BrokenPipe.
                if e.kind() != ErrorKind::BrokenPipe {
                    return Err(format!("Failed to write to tsgolint stdin: {e}"));
                }
            }
            // Explicitly drop stdin to send EOF to the child.
            drop(stdin);

            // Stream diagnostics as they are emitted, rather than waiting for all output
            let mut stdout = child.stdout.take().expect("Failed to open tsgolint stdout");

            // Process stdout stream in a separate thread to send diagnostics as they arrive
            let cwd_clone = self.cwd.clone();

            let stdout_handler = std::thread::spawn(move || -> Result<(), String> {
                let mut buffer = Vec::with_capacity(8192);
                let mut read_buf = [0u8; 8192];

                let mut source_text_map: FxHashMap<PathBuf, String> = FxHashMap::default();

                loop {
                    match stdout.read(&mut read_buf) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            buffer.extend_from_slice(&read_buf[..n]);

                            // Try to parse complete messages from buffer
                            let mut cursor = std::io::Cursor::new(buffer.as_slice());
                            let mut processed_up_to: u64 = 0;

                            while cursor.position() < buffer.len() as u64 {
                                let start_pos = cursor.position();
                                match parse_single_message(&mut cursor) {
                                    Ok(Some(TsGoLintMessage::Error(err))) => {
                                        return Err(err.error);
                                    }
                                    Ok(Some(TsGoLintMessage::Diagnostic(tsgolint_diagnostic))) => {
                                        processed_up_to = cursor.position();

                                        let path = tsgolint_diagnostic.file_path.clone();
                                        let Some(resolved_config) = resolved_configs.get(&path)
                                        else {
                                            // If we don't have a resolved config for this path, skip it. We should always
                                            // have a resolved config though, since we processed them already above.
                                            continue;
                                        };

                                        let severity = resolved_config.rules.iter().find_map(
                                            |(rule, status)| {
                                                if rule.name() == tsgolint_diagnostic.rule {
                                                    Some(*status)
                                                } else {
                                                    None
                                                }
                                            },
                                        );
                                        let Some(severity) = severity else {
                                            // If the severity is not found, we should not report the diagnostic
                                            continue;
                                        };

                                        let oxc_diagnostic: OxcDiagnostic =
                                            OxcDiagnostic::from(tsgolint_diagnostic);

                                        let oxc_diagnostic = oxc_diagnostic.with_severity(
                                            if severity == AllowWarnDeny::Deny {
                                                Severity::Error
                                            } else {
                                                Severity::Warning
                                            },
                                        );

                                        let source_text: &str = if self.silent {
                                            // The source text is not needed in silent mode.
                                            // The source text is only here to wrap the line before and after into a nice `oxc_diagnostic` Error
                                            ""
                                        } else if let Some(source_text) = source_text_map.get(&path)
                                        {
                                            source_text.as_str()
                                        } else {
                                            let source_text = read_to_string(&path)
                                                .unwrap_or_else(|_| String::new());
                                            // Insert and get a reference to the inserted string
                                            let entry = source_text_map
                                                .entry(path.clone())
                                                .or_insert(source_text);
                                            entry.as_str()
                                        };

                                        let diagnostics = DiagnosticService::wrap_diagnostics(
                                            cwd_clone.clone(),
                                            path.clone(),
                                            source_text,
                                            vec![oxc_diagnostic],
                                        );

                                        if error_sender.send((path, diagnostics)).is_err() {
                                            // Receiver has been dropped, stop processing
                                            return Ok(());
                                        }
                                    }
                                    Ok(None) => {
                                        // Successfully parsed but no diagnostic to add
                                        processed_up_to = cursor.position();
                                    }
                                    Err(_) => {
                                        // Could not parse a complete message, break and keep remaining data
                                        cursor.set_position(start_pos);
                                        break;
                                    }
                                }
                            }

                            // Keep unprocessed data for next iteration
                            if processed_up_to > 0 {
                                #[expect(clippy::cast_possible_truncation)]
                                buffer.drain(..processed_up_to as usize);
                            }
                        }
                        Err(e) => {
                            return Err(format!("Failed to read from tsgolint stdout: {e}"));
                        }
                    }
                }

                Ok(())
            });

            // Wait for process to complete and stdout processing to finish
            let exit_status = child.wait().expect("Failed to wait for tsgolint process");
            let stdout_result = stdout_handler.join();

            if !exit_status.success() {
                return Err(
                    if let Some(err) = &stdout_result.ok().and_then(std::result::Result::err) {
                        format!("exit status: {exit_status}, error: {err}")
                    } else {
                        format!("exit status: {exit_status}")
                    },
                );
            }

            match stdout_result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(format!("exit status: {exit_status}, error: {err}")),
                Err(_) => Err("Failed to join stdout processing thread".to_string()),
            }
        });

        match handler.join() {
            Ok(Ok(())) => {
                // Successfully ran tsgolint
                Ok(())
            }
            Ok(Err(err)) => Err(format!("Error running tsgolint: {err:?}")),
            Err(err) => Err(format!("Error running tsgolint: {err:?}")),
        }
    }

    /// # Panics
    /// - when `stdin` of subprocess cannot be opened
    /// - when `stdout` of subprocess cannot be opened
    /// - when `tsgolint` process cannot be awaited
    ///
    /// # Errors
    /// A human-readable error message indicating why the linting failed.
    #[cfg(feature = "language_server")]
    pub fn lint_source(
        &self,
        path: &Arc<OsStr>,
        source_text: String,
    ) -> Result<Vec<MessageWithPosition<'_>>, String> {
        use oxc_data_structures::rope::Rope;

        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();

        let json_input = self.json_input(std::slice::from_ref(path), &mut resolved_configs);
        let executable_path = self.executable_path.clone();

        let handler = std::thread::spawn(move || {
            let child = std::process::Command::new(&executable_path)
                .arg("headless")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();

            let mut child = match child {
                Ok(c) => c,
                Err(e) => {
                    return Err(format!(
                        "Failed to spawn tsgolint from path `{}`, with error: {e}",
                        executable_path.display()
                    ));
                }
            };

            let mut stdin = child.stdin.take().expect("Failed to open tsgolint stdin");

            // Write the input synchronously and handle BrokenPipe gracefully in case the child
            // exits early and closes its stdin.
            let json = serde_json::to_string(&json_input).expect("Failed to serialize JSON");
            if let Err(e) = stdin.write_all(json.as_bytes()) {
                // If the child closed stdin early, avoid crashing on SIGPIPE/BrokenPipe.
                if e.kind() != ErrorKind::BrokenPipe {
                    return Err(format!("Failed to write to tsgolint stdin: {e}"));
                }
            }
            // Explicitly drop stdin to send EOF to the child.
            drop(stdin);

            // Stream diagnostics as they are emitted, rather than waiting for all output
            let mut stdout = child.stdout.take().expect("Failed to open tsgolint stdout");

            let stdout_handler =
                std::thread::spawn(move || -> Result<Vec<MessageWithPosition<'_>>, String> {
                    let mut buffer = Vec::with_capacity(8192);
                    let mut read_buf = [0u8; 8192];

                    let mut result = vec![];

                    loop {
                        match stdout.read(&mut read_buf) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                buffer.extend_from_slice(&read_buf[..n]);

                                // Try to parse complete messages from buffer
                                let mut cursor = std::io::Cursor::new(buffer.as_slice());
                                let mut processed_up_to: u64 = 0;

                                while cursor.position() < buffer.len() as u64 {
                                    let start_pos = cursor.position();
                                    match parse_single_message(&mut cursor) {
                                        Ok(Some(TsGoLintMessage::Error(err))) => {
                                            return Err(err.error);
                                        }
                                        Ok(Some(TsGoLintMessage::Diagnostic(
                                            tsgolint_diagnostic,
                                        ))) => {
                                            processed_up_to = cursor.position();

                                            let path = tsgolint_diagnostic.file_path.clone();
                                            let Some(resolved_config) = resolved_configs.get(&path)
                                            else {
                                                // If we don't have a resolved config for this path, skip it. We should always
                                                // have a resolved config though, since we processed them already above.
                                                continue;
                                            };

                                            let severity = resolved_config.rules.iter().find_map(
                                                |(rule, status)| {
                                                    if rule.name() == tsgolint_diagnostic.rule {
                                                        Some(*status)
                                                    } else {
                                                        None
                                                    }
                                                },
                                            );
                                            let Some(severity) = severity else {
                                                // If the severity is not found, we should not report the diagnostic
                                                continue;
                                            };

                                            let mut message_with_position: MessageWithPosition<'_> =
                                                message_to_message_with_position(
                                                    &Message::from_tsgo_lint_diagnostic(
                                                        tsgolint_diagnostic,
                                                        &source_text,
                                                    ),
                                                    &source_text,
                                                    &Rope::from_str(&source_text),
                                                );

                                            message_with_position.severity =
                                                if severity == AllowWarnDeny::Deny {
                                                    Severity::Error
                                                } else {
                                                    Severity::Warning
                                                };

                                            result.push(message_with_position);
                                        }
                                        Ok(None) => {
                                            // Successfully parsed but no diagnostic to add
                                            processed_up_to = cursor.position();
                                        }
                                        Err(_) => {
                                            // Could not parse a complete message, break and keep remaining data
                                            cursor.set_position(start_pos);
                                            break;
                                        }
                                    }
                                }

                                // Keep unprocessed data for next iteration
                                if processed_up_to > 0 {
                                    #[expect(clippy::cast_possible_truncation)]
                                    buffer.drain(..processed_up_to as usize);
                                }
                            }
                            Err(e) => {
                                return Err(format!("Failed to read from tsgolint stdout: {e}"));
                            }
                        }
                    }

                    Ok(result)
                });

            // Wait for process to complete and stdout processing to finish
            let exit_status = child.wait().expect("Failed to wait for tsgolint process");
            let stdout_result = stdout_handler.join();

            if !exit_status.success() {
                return Err(format!("tsgolint process exited with status: {exit_status}"));
            }

            match stdout_result {
                Ok(Ok(diagnostics)) => Ok(diagnostics),
                Ok(Err(err)) => Err(err),
                Err(_) => Err("Failed to join stdout processing thread".to_string()),
            }
        });

        match handler.join() {
            Ok(Ok(diagnostics)) => {
                // Successfully ran tsgolint
                Ok(diagnostics)
            }
            Ok(Err(err)) => Err(format!("Error running tsgolint: {err:?}")),
            Err(err) => Err(format!("Error running tsgolint: {err:?}")),
        }
    }

    /// Create a JSON input for STDIN of tsgolint in this format:
    ///
    /// ```json
    /// {
    ///   "files": [
    ///     {
    ///       "file_path": "/absolute/path/to/file.ts",
    ///       "rules": ["rule-1", "another-rule"]
    ///     }
    ///   ]
    /// }
    /// ```
    #[inline]
    fn json_input(
        &self,
        paths: &[Arc<OsStr>],
        resolved_configs: &mut FxHashMap<PathBuf, ResolvedLinterState>,
    ) -> Payload {
        let mut config_groups: FxHashMap<BTreeSet<Rule>, Vec<String>> = FxHashMap::default();

        for path in paths {
            if SourceType::from_path(Path::new(path)).is_ok() {
                let path_buf = PathBuf::from(path);
                let file_path = path.to_string_lossy().to_string();

                let resolved_config = resolved_configs
                    .entry(path_buf.clone())
                    .or_insert_with(|| self.config_store.resolve(&path_buf));

                let rules: BTreeSet<Rule> = resolved_config
                    .rules
                    .iter()
                    .filter_map(|(rule, status)| {
                        if status.is_warn_deny() && rule.is_tsgolint_rule() {
                            Some(Rule { name: rule.name().to_string() })
                        } else {
                            None
                        }
                    })
                    .collect();

                config_groups.entry(rules).or_default().push(file_path);
            }
        }

        Payload {
            version: 2,
            configs: config_groups
                .into_iter()
                .map(|(rules, file_paths)| Config {
                    file_paths,
                    rules: rules.into_iter().collect(),
                })
                .collect(),
        }
    }
}

/// Represents the input JSON to `tsgolint`, like:
///
/// ```json
/// {
///   "configs": [
///     {
///       "file_paths": ["/absolute/path/to/file.ts", "/another/file.ts"],
///       "rules": [
///         { "name": "rule-1" },
///         { "name": "another-rule" },
///       ]
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub version: i32,
    pub configs: Vec<Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Absolute path to the file to lint
    pub file_paths: Vec<String>,
    /// List of rules to apply to this file
    /// Example: `["no-floating-promises"]`
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Rule {
    pub name: String,
}

/// Represents the raw output binary data from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsGoLintDiagnosticPayload {
    pub range: Range,
    pub rule: String,
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
    pub suggestions: Vec<Suggestion>,
    pub file_path: PathBuf,
}

/// Represents the error payload from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TsGoLintErrorPayload {
    pub error: String,
}

#[derive(Debug, Clone)]
pub enum TsGoLintMessage {
    Diagnostic(TsGoLintDiagnostic),
    Error(TsGoLintError),
}

#[derive(Debug, Clone)]
pub struct TsGoLintDiagnostic {
    pub range: Range,
    pub rule: String,
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
    pub suggestions: Vec<Suggestion>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TsGoLintError {
    pub error: String,
}

impl From<TsGoLintDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintDiagnostic) -> Self {
        let mut d = OxcDiagnostic::warn(val.message.description)
            .with_label(Span::new(val.range.pos, val.range.end))
            .with_error_code("typescript-eslint", val.rule);
        if let Some(help) = val.message.help {
            d = d.with_help(help);
        }
        d
    }
}

#[cfg(feature = "language_server")]
impl Message<'_> {
    /// Converts a `TsGoLintDiagnostic` into a `Message` with possible fixes.
    fn from_tsgo_lint_diagnostic(val: TsGoLintDiagnostic, source_text: &str) -> Self {
        let mut fixes =
            Vec::with_capacity(usize::from(!val.fixes.is_empty()) + val.suggestions.len());

        if !val.fixes.is_empty() {
            let fix_vec = val
                .fixes
                .iter()
                .map(|fix| crate::fixer::Fix {
                    content: fix.text.clone().into(),
                    span: Span::new(fix.range.pos, fix.range.end),
                    message: None,
                })
                .collect();

            fixes.push(CompositeFix::merge_fixes(fix_vec, source_text));
        }

        for suggestion in &val.suggestions {
            let fix_vec = suggestion
                .fixes
                .iter()
                .map(|fix| crate::fixer::Fix {
                    content: fix.text.clone().into(),
                    span: Span::new(fix.range.pos, fix.range.end),
                    message: Some(suggestion.message.description.clone().into()),
                })
                .collect();

            fixes.push(CompositeFix::merge_fixes(fix_vec, source_text));
        }

        let possible_fix = if fixes.is_empty() {
            PossibleFixes::None
        } else if fixes.len() == 1 {
            PossibleFixes::Single(fixes.into_iter().next().unwrap())
        } else {
            PossibleFixes::Multiple(fixes)
        };

        Self::new(val.into(), possible_fix)
    }
}

// TODO: Should this be removed and replaced with a `Span`?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Range {
    pub pos: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMessage {
    pub id: String,
    pub description: String,
    pub help: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fix {
    pub text: String,
    pub range: Range,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    Error = 0,
    Diagnostic = 1,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MessageType::Error),
            1 => Some(MessageType::Diagnostic),
            _ => None,
        }
    }
}

/// Parses a single message from the binary tsgolint output.
// Messages are encoded as follows:
// | Payload Size (uint32 LE) - 4 bytes | Message Type (uint8) - 1 byte | Payload |
fn parse_single_message(
    cursor: &mut std::io::Cursor<&[u8]>,
) -> Result<Option<TsGoLintMessage>, String> {
    let mut size_bytes = [0u8; 4];
    if cursor.read_exact(&mut size_bytes).is_err() {
        return Err("Failed to read size bytes".to_string());
    }
    let size = u32::from_le_bytes(size_bytes) as usize;

    let mut message_type_byte = [0u8; 1];
    if cursor.read_exact(&mut message_type_byte).is_err() {
        return Err("Failed to read message type byte".to_string());
    }
    let message_type = MessageType::from_u8(message_type_byte[0])
        .ok_or_else(|| "Invalid message type byte".to_string())?;

    let mut payload_bytes = vec![0u8; size];
    if cursor.read_exact(&mut payload_bytes).is_err() {
        return Err("Failed to read payload bytes".to_string());
    }
    let payload_str = String::from_utf8_lossy(&payload_bytes);

    match message_type {
        MessageType::Error => {
            let error_payload = serde_json::from_str::<TsGoLintErrorPayload>(&payload_str)
                .map_err(|e| format!("Failed to parse tsgolint error payload: {e}"))?;

            Ok(Some(TsGoLintMessage::Error(TsGoLintError { error: error_payload.error })))
        }
        MessageType::Diagnostic => {
            let diagnostic_payload =
                serde_json::from_str::<TsGoLintDiagnosticPayload>(&payload_str)
                    .map_err(|e| format!("Failed to parse tsgolint diagnostic payload: {e}"))?;

            Ok(Some(TsGoLintMessage::Diagnostic(TsGoLintDiagnostic {
                range: diagnostic_payload.range,
                rule: diagnostic_payload.rule,
                message: diagnostic_payload.message,
                fixes: diagnostic_payload.fixes,
                suggestions: diagnostic_payload.suggestions,
                file_path: diagnostic_payload.file_path,
            })))
        }
    }
}

/// Tries to find the `tsgolint` executable. In priority order, this will check:
/// 1. The `OXLINT_TSGOLINT_PATH` environment variable.
/// 2. The `tsgolint` binary in the current working directory's `node_modules/.bin` directory.
///
/// # Errors
/// Returns an error if `OXLINT_TSGOLINT_PATH` is set but does not exist or is not a file.
/// Returns an error if the tsgolint executable could not be resolve inside `node_modules/.bin`.
pub fn try_find_tsgolint_executable(cwd: &Path) -> Result<PathBuf, String> {
    // Check the environment variable first
    if let Ok(path_str) = std::env::var("OXLINT_TSGOLINT_PATH") {
        let path = PathBuf::from(&path_str);
        if path.is_dir() {
            let tsgolint_path = path.join("tsgolint");
            if tsgolint_path.exists() {
                return Ok(tsgolint_path);
            }
            return Err(format!(
                "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to directory '{path_str}' but 'tsgolint' binary not found inside"
            ));
        }
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "Failed to find tsgolint executable: OXLINT_TSGOLINT_PATH points to '{path_str}' which does not exist"
        ));
    }

    // executing a sub command in windows, needs a `cmd` or `ps1` extension.
    // `cmd` is the most compatible one with older systems
    let file = if cfg!(windows) { "tsgolint.CMD" } else { "tsgolint" };

    // Move upwards until we find a `package.json`, then look at `node_modules/.bin/tsgolint`
    let mut current_dir = cwd.to_path_buf();
    loop {
        let node_modules_bin = current_dir.join("node_modules").join(".bin").join(file);
        if node_modules_bin.exists() {
            return Ok(node_modules_bin);
        }

        // If we reach the root directory, stop searching
        if !current_dir.pop() {
            break;
        }
    }

    Err("Failed to find tsgolint executable".to_string())
}

#[cfg(test)]
#[cfg(feature = "language_server")]
mod test {
    use oxc_diagnostics::{LabeledSpan, OxcCode, Severity};
    use oxc_span::{GetSpan, Span};

    use crate::{
        fixer::{Message, PossibleFixes},
        tsgolint::{Fix, Range, RuleMessage, Suggestion, TsGoLintDiagnostic},
    };

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_basic() {
        let diagnostic = TsGoLintDiagnostic {
            range: Range { pos: 0, end: 10 },
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: Some("Some help".into()),
            },
            fixes: vec![],
            suggestions: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.error.message, "Some description");
        assert_eq!(message.error.severity, Severity::Warning);
        assert_eq!(message.span(), Span::new(0, 10));
        assert_eq!(
            message.error.code,
            OxcCode { scope: Some("typescript-eslint".into()), number: Some("some_rule".into()) }
        );
        assert!(message.error.labels.as_ref().is_some());
        assert_eq!(message.error.labels.as_ref().unwrap().len(), 1);
        assert_eq!(message.error.labels.as_ref().unwrap()[0], LabeledSpan::new(None, 0, 10));
        assert_eq!(message.error.help, Some("Some help".into()));
        assert!(message.fixes.is_empty());
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_fixes() {
        let diagnostic = TsGoLintDiagnostic {
            range: Range { pos: 0, end: 10 },
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![
                Fix { text: "fixed".into(), range: Range { pos: 0, end: 5 } },
                Fix { text: "hello".into(), range: Range { pos: 5, end: 10 } },
            ],
            suggestions: vec![],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.fixes.len(), 1);
        assert_eq!(
            message.fixes,
            PossibleFixes::Single(crate::fixer::Fix {
                content: "fixedhello".into(),
                span: Span::new(0, 10),
                message: None,
            })
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_multiple_suggestions() {
        let diagnostic = TsGoLintDiagnostic {
            range: Range { pos: 0, end: 10 },
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![],
            suggestions: vec![
                Suggestion {
                    message: RuleMessage {
                        id: "some_id".into(),
                        description: "Suggestion 1".into(),
                        help: None,
                    },
                    fixes: vec![Fix { text: "hello".into(), range: Range { pos: 0, end: 5 } }],
                },
                Suggestion {
                    message: RuleMessage {
                        id: "some_id".into(),
                        description: "Suggestion 2".into(),
                        help: None,
                    },
                    fixes: vec![
                        Fix { text: "hello".into(), range: Range { pos: 0, end: 5 } },
                        Fix { text: "world".into(), range: Range { pos: 5, end: 10 } },
                    ],
                },
            ],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(
            message.fixes,
            PossibleFixes::Multiple(vec![
                crate::fixer::Fix {
                    content: "hello".into(),
                    span: Span::new(0, 5),
                    message: Some("Suggestion 1".into()),
                },
                crate::fixer::Fix {
                    content: "helloworld".into(),
                    span: Span::new(0, 10),
                    message: Some("Suggestion 2".into()),
                },
            ])
        );
    }

    #[test]
    fn test_message_from_tsgo_lint_diagnostic_with_fix_and_suggestions() {
        let diagnostic = TsGoLintDiagnostic {
            range: Range { pos: 0, end: 10 },
            rule: "some_rule".into(),
            message: RuleMessage {
                id: "some_id".into(),
                description: "Some description".into(),
                help: None,
            },
            fixes: vec![Fix { text: "fixed".into(), range: Range { pos: 0, end: 5 } }],
            suggestions: vec![Suggestion {
                message: RuleMessage {
                    id: "some_id".into(),
                    description: "Suggestion 1".into(),
                    help: None,
                },
                fixes: vec![Fix { text: "Suggestion 1".into(), range: Range { pos: 0, end: 5 } }],
            }],
            file_path: "some/file/path".into(),
        };

        let message = Message::from_tsgo_lint_diagnostic(diagnostic, "Some text over 10 bytes.");

        assert_eq!(message.fixes.len(), 2);
        assert_eq!(
            message.fixes,
            PossibleFixes::Multiple(vec![
                crate::fixer::Fix { content: "fixed".into(), span: Span::new(0, 5), message: None },
                crate::fixer::Fix {
                    content: "Suggestion 1".into(),
                    span: Span::new(0, 5),
                    message: Some("Suggestion 1".into()),
                },
            ])
        );
    }
}
