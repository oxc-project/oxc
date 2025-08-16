use std::{
    ffi::OsStr,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::{DiagnosticSender, DiagnosticService, OxcDiagnostic, Severity};
use oxc_linter::{
    AllowWarnDeny, ConfigStore, LintServiceOptions, ResolvedLinterState, read_to_string,
};
use oxc_span::{SourceType, Span};

/// State required to initialize the `tsgolint` linter.
#[derive(Debug, Clone)]
pub struct TsGoLintState<'a> {
    /// The path to the `tsgolint` executable (at least our our best guess at it).
    executable_path: PathBuf,
    /// Current working directory, used for rendering paths in diagnostics.
    cwd: PathBuf,
    /// The paths of files to lint
    paths: &'a Vec<Arc<OsStr>>,
    /// The configuration store for `tsgolint` (used to resolve configurations outside of `oxc_linter`)
    config_store: ConfigStore,
}

impl<'a> TsGoLintState<'a> {
    pub fn new(
        config_store: ConfigStore,
        paths: &'a Vec<Arc<OsStr>>,
        options: &LintServiceOptions,
    ) -> Self {
        TsGoLintState {
            config_store,
            executable_path: try_find_tsgolint_executable(options.cwd())
                .unwrap_or(PathBuf::from("tsgolint")),
            cwd: options.cwd().to_path_buf(),
            paths,
        }
    }

    pub fn lint(self, error_sender: DiagnosticSender) -> Result<(), String> {
        if self.paths.is_empty() {
            return Ok(());
        }

        let mut resolved_configs: FxHashMap<PathBuf, ResolvedLinterState> = FxHashMap::default();

        let json_input = self.json_input(&mut resolved_configs);

        let handler = std::thread::spawn(move || {
            let child = std::process::Command::new(&self.executable_path)
                .arg("headless")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();

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
                                    Ok(Some(tsgolint_diagnostic)) => {
                                        processed_up_to = cursor.position();

                                        // For now, ignore any `tsgolint` errors.
                                        if tsgolint_diagnostic.r#type == MessageType::Error {
                                            continue;
                                        }

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

                                        let oxc_diagnostic: OxcDiagnostic =
                                            tsgolint_diagnostic.into();
                                        let Some(severity) = severity else {
                                            // If the severity is not found, we should not report the diagnostic
                                            continue;
                                        };
                                        let oxc_diagnostic = oxc_diagnostic.with_severity(
                                            if severity == AllowWarnDeny::Deny {
                                                Severity::Error
                                            } else {
                                                Severity::Warning
                                            },
                                        );

                                        let diagnostics = DiagnosticService::wrap_diagnostics(
                                            cwd_clone.clone(),
                                            path.clone(),
                                            &read_to_string(&path)
                                                .unwrap_or_else(|_| String::new()),
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
                return Err(format!("tsgolint process exited with status: {exit_status}"));
            }

            match stdout_result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(err),
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
        resolved_configs: &mut FxHashMap<PathBuf, ResolvedLinterState>,
    ) -> TsGoLintInput {
        TsGoLintInput {
            files: self
                .paths
                .iter()
                .filter(|path| SourceType::from_path(Path::new(path)).is_ok())
                .map(|path| TsGoLintInputFile {
                    file_path: path.to_string_lossy().to_string(),
                    rules: {
                        let path_buf = PathBuf::from(path);
                        let resolved_config = resolved_configs
                            .entry(path_buf.clone())
                            .or_insert_with(|| self.config_store.resolve(&path_buf));

                        // Collect the rules that are enabled for this file
                        resolved_config
                            .rules
                            .iter()
                            .filter_map(|(rule, status)| {
                                if status.is_warn_deny() && rule.is_tsgolint_rule() {
                                    Some(rule.name().to_string())
                                } else {
                                    None
                                }
                            })
                            .collect()
                    },
                })
                .collect(),
        }
    }
}

/// Represents the input JSON to `tsgolint`, like:
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsGoLintInput {
    pub files: Vec<TsGoLintInputFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsGoLintInputFile {
    /// Absolute path to the file to lint
    pub file_path: String,
    /// List of rules to apply to this file
    /// Example: `["no-floating-promises"]`
    pub rules: Vec<String>,
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

/// Represents a message from `tsgolint`, ready to be converted into [`OxcDiagnostic`].
#[derive(Debug, Clone)]
pub struct TsGoLintDiagnostic {
    pub r#type: MessageType,
    pub range: Range,
    pub rule: String,
    pub message: RuleMessage,
    pub fixes: Vec<Fix>,
    pub suggestions: Vec<Suggestion>,
    pub file_path: PathBuf,
}

impl From<TsGoLintDiagnostic> for OxcDiagnostic {
    fn from(val: TsGoLintDiagnostic) -> Self {
        OxcDiagnostic::warn(val.message.description)
            .with_label(Span::new(val.range.pos, val.range.end))
            .with_error_code("typescript-eslint", val.rule)
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
) -> Result<Option<TsGoLintDiagnostic>, String> {
    let mut size_bytes = [0u8; 4];
    if cursor.read_exact(&mut size_bytes).is_err() {
        return Err("Failed to read size bytes".to_string());
    }
    let size = u32::from_le_bytes(size_bytes) as usize;

    // TODO: Use message type byte for diagnostic
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
    let payload = String::from_utf8_lossy(&payload_bytes);

    let payload = serde_json::from_str::<TsGoLintDiagnosticPayload>(&payload);

    match payload {
        Ok(diagnostic) => Ok(Some(TsGoLintDiagnostic {
            r#type: message_type,
            range: diagnostic.range,
            rule: diagnostic.rule,
            message: diagnostic.message,
            fixes: diagnostic.fixes,
            suggestions: diagnostic.suggestions,
            file_path: diagnostic.file_path,
        })),
        Err(e) => Err(format!("Failed to parse tsgolint payload: {e}")),
    }
}

/// Tries to find the `tsgolint` executable. In priority order, this will check:
/// 1. The `OXLINT_TSGOLINT_PATH` environment variable.
/// 2. The `tsgolint` binary in the current working directory's `node_modules/.bin` directory.
pub fn try_find_tsgolint_executable(cwd: &Path) -> Option<PathBuf> {
    // Check the environment variable first
    if let Ok(path) = std::env::var("OXLINT_TSGOLINT_PATH") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Some(path.join("tsgolint"));
        } else if path.is_file() {
            return Some(path);
        }
    }

    // executing a sub command in windows, needs a `cmd` or `ps1` extension.
    // `cmd` is the most compatible one with older systems
    let file = if cfg!(windows) { "tsgolint.CMD" } else { "tsgolint" };

    // Move upwards until we find a `package.json`, then look at `node_modules/.bin/tsgolint`
    let mut current_dir = cwd.to_path_buf();
    loop {
        let node_modules_bin = current_dir.join("node_modules").join(".bin").join(file);
        if node_modules_bin.exists() {
            return Some(node_modules_bin);
        }

        // If we reach the root directory, stop searching
        if !current_dir.pop() {
            break;
        }
    }

    None
}
