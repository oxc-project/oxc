// The oxlint <-> tsgolint headless protocol (cmd/tsgolint/payload.go + headless.go).
// docs/headless-protocol.md describes it in full.

use std::io::Write;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

// ---- stdin payload ----------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct HeadlessRule {
    pub name: String,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeadlessConfig {
    pub file_paths: Vec<String>,
    pub rules: Vec<HeadlessRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Payload {
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub configs: Vec<HeadlessConfig>,
    #[serde(default)]
    pub source_overrides: Option<FxHashMap<String, String>>,
    #[serde(default)]
    pub report_syntactic: bool,
    #[serde(default)]
    pub report_semantic: bool,
}

#[derive(Deserialize)]
struct PayloadV1File {
    file_path: String,
    rules: Vec<String>,
}

#[derive(Deserialize)]
struct PayloadV1 {
    files: Vec<PayloadV1File>,
}

pub fn deserialize_payload(data: &[u8]) -> Result<Payload, String> {
    #[derive(Deserialize)]
    struct VersionCheck {
        #[serde(default)]
        version: i32,
    }
    let version = serde_json::from_slice::<VersionCheck>(data).map_err(|e| e.to_string())?.version;
    if version == 2 {
        return serde_json::from_slice::<Payload>(data)
            .map_err(|e| format!("failed to deserialize V2 payload: {e}"));
    }
    if version != 0 {
        return Err(format!("unsupported version `{version}`: expected `unset` or `2`"));
    }
    let v1: PayloadV1 = serde_json::from_slice(data)
        .map_err(|e| format!("failed to deserialize V1 payload: {e}"))?;
    if v1.files.is_empty() {
        return Err("V1 payload has no files".to_string());
    }
    Ok(Payload {
        version: 2,
        configs: v1
            .files
            .into_iter()
            .map(|f| HeadlessConfig {
                file_paths: vec![f.file_path],
                rules: f
                    .rules
                    .into_iter()
                    .map(|name| HeadlessRule { name, options: None })
                    .collect(),
            })
            .collect(),
        source_overrides: None,
        report_syntactic: false,
        report_semantic: false,
    })
}

// ---- stdout messages --------------------------------------------------------------------------

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum MessageType {
    Error = 0,
    Diagnostic = 1,
    Timing = 2,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub pos: i32,
    pub end: i32,
}

#[derive(Serialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Fix {
    pub text: String,
    pub range: Range,
}

#[derive(Serialize, Clone, Debug)]
pub struct Suggestion {
    pub message: Message,
    pub fixes: Vec<Fix>,
}

#[derive(Serialize, Clone, Debug)]
pub struct LabeledRange {
    pub label: String,
    pub range: Range,
}

/// One diagnostic message. `kind` 0 = rule diagnostic, 1 = internal (tsconfig / TypeScript) diagnostic.
#[derive(Serialize, Clone, Debug)]
pub struct Diagnostic {
    pub kind: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    pub message: Message,
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labeled_ranges: Vec<LabeledRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<Fix>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<Suggestion>,
}

#[derive(Serialize, Debug)]
pub struct RuleTiming {
    pub rule_name: String,
    pub duration: u64,
    pub calls: u64,
}

#[derive(Serialize, Debug)]
pub struct TimingPayload {
    pub rules: Vec<RuleTiming>,
}

/// One output message of a run, as `linter::run_with_sink` delivers it: diagnostics as they are found, then the
/// timing message (`-debug timings`), or the error that ended the run. The headless binary writes each one with
/// `write_message`; a host that links tsrslint gets the values.
#[derive(Debug)]
pub enum Output {
    Diagnostic(Diagnostic),
    Timing(TimingPayload),
    /// The run failed; the text is the headless protocol's error message. Always the last message.
    Error(String),
}

/// `| payload length: u32 LE | message type: u8 | JSON payload |`
pub fn write_message(
    w: &mut impl Write,
    ty: MessageType,
    payload: &impl Serialize,
) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(payload).expect("serializable payload");
    let mut header = [0u8; 5];
    header[..4].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
    header[4] = ty as u8;
    w.write_all(&header)?;
    w.write_all(&bytes)
}

pub fn write_error_message(text: &str) {
    #[derive(Serialize)]
    struct E<'a> {
        error: &'a str,
    }
    let mut out = std::io::stdout().lock();
    let _ = write_message(&mut out, MessageType::Error, &E { error: text });
    let _ = out.flush();
}
