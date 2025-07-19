use std::{io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

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
    pub file_path: String,
    pub rules: Vec<String>,
}

/// Represents the output binary data from `tsgolint`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsGoLintDiagnostic {
    pub range: HeadlessRange,
    pub rule: String,
    pub message: HeadlessRuleMessage,
    pub fixes: Vec<HeadlessFix>,
    pub suggestions: Vec<HeadlessSuggestion>,
    pub file_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TsGoLintOutputMessageType {
    Error = 0,
    Diagnostic = 1,
}

impl TsGoLintOutputMessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Error),
            1 => Some(Self::Diagnostic),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeadlessRange {
    pub pos: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeadlessRuleMessage {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeadlessFix {
    pub text: String,
    pub range: HeadlessRange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeadlessSuggestion {
    pub message: HeadlessRuleMessage,
    pub fixes: Vec<HeadlessFix>,
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
            Err(e) => return Err(format!("Failed to parse tsgolint output: {}", e)),
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
    let mut message_type_byte = [0u8; 1];
    if cursor.read_exact(&mut message_type_byte).is_err() {
        return Err("Failed to read message type byte".to_string());
    }
    let _message_type = TsGoLintOutputMessageType::from_u8(message_type_byte[0])
        .ok_or("Invalid message type byte")?;

    let mut payload_bytes = vec![0u8; size]; // 4 bytes for size + 1 byte for type
    if cursor.read_exact(&mut payload_bytes).is_err() {
        return Err("Failed to read payload bytes".to_string());
    }

    let payload = String::from_utf8_lossy(&payload_bytes);
    let payload = serde_json::from_str::<TsGoLintDiagnostic>(&payload);

    match payload {
        Ok(diagnostic) => Ok(diagnostic),
        Err(e) => return Err(format!("Failed to parse payload: {}", e)),
    }
}
