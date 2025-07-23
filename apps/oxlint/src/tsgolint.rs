use std::{
    ffi::OsStr,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::rules::RuleEnum;
use oxc_span::Span;
use serde::{Deserialize, Serialize};

/// State required to initialize the `tsgolint` linter.
#[derive(Debug, Clone)]
pub struct TsGoLintState {
    /// Current working directory to run `tsgolint` in
    pub cwd: PathBuf,
    /// The paths of files to lint
    pub paths: Vec<Arc<OsStr>>,
    /// The rules to run when linting
    pub rules: Vec<RuleEnum>,
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

/// Represents the output binary data from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsGoLintDiagnostic {
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

/// Parses the binary output from `tsgolint` and returns the diagnostic data.
// Messages are encoded as follows:
// | Payload Size (uint32 LE) - 4 bytes | Message Type (uint8) - 1 byte | Payload |
pub fn parse_tsgolint_output(output: &[u8]) -> Result<Vec<TsGoLintDiagnostic>, String> {
    let mut diagnostics: Vec<TsGoLintDiagnostic> = Vec::new();

    // Parse the output binary data
    let mut cursor = std::io::Cursor::new(output);

    while cursor.position() < output.len() as u64 {
        match parse_single_message(&mut cursor) {
            Ok(diagnostic) => diagnostics.push(diagnostic),
            Err(e) => {
                // Ignore files not in tsconfig
                // https://github.com/oxc-project/tsgolint/issues/44
                if e == "file not matched by tsconfig" {
                    continue;
                }
                return Err(format!("Failed to parse tsgolint output: {e}"));
            }
        }
    }

    Ok(diagnostics)
}

fn parse_single_message(cursor: &mut std::io::Cursor<&[u8]>) -> Result<TsGoLintDiagnostic, String> {
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

    let mut payload_bytes = vec![0u8; size];
    if cursor.read_exact(&mut payload_bytes).is_err() {
        return Err("Failed to read payload bytes".to_string());
    }
    let payload = String::from_utf8_lossy(&payload_bytes);

    // Ignore files not in tsconfig
    // https://github.com/oxc-project/tsgolint/issues/44
    if payload.starts_with("{\"error") && payload.contains("not matched by tsconfig") {
        return Err("file not matched by tsconfig".to_string());
    }

    let payload = serde_json::from_str::<TsGoLintDiagnostic>(&payload);

    match payload {
        Ok(diagnostic) => Ok(diagnostic),
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

    // Check the local node_modules/.bin directory
    let local_bin_path = cwd.join("node_modules").join(".bin").join("tsgolint");
    if local_bin_path.is_file() {
        return Some(local_bin_path);
    }

    None
}
