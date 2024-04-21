use std::{
    borrow::Cow,
    collections::HashMap,
    io::{BufWriter, Stdout, Write},
};

use crate::{
    miette::{Error, JSONReportHandler},
    GraphicalReportHandler, Severity,
};

/// stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
/// See `https://github.com/rust-lang/rust/issues/60673`.
fn writer() -> BufWriter<Stdout> {
    BufWriter::new(std::io::stdout())
}

#[allow(clippy::large_enum_variant)] // Lerge size is fine because this is a singleton
#[derive(Debug)]
#[non_exhaustive]
pub enum DiagnosticReporter {
    Graphical { handler: GraphicalReportHandler, writer: BufWriter<Stdout> },
    Json { diagnostics: Vec<Error> },
    Unix { total: usize, writer: BufWriter<Stdout> },
    Checkstyle { diagnostics: Vec<Error> },
}

impl DiagnosticReporter {
    pub fn new_graphical() -> Self {
        Self::Graphical { handler: GraphicalReportHandler::new(), writer: writer() }
    }

    pub fn new_json() -> Self {
        Self::Json { diagnostics: vec![] }
    }

    pub fn new_unix() -> Self {
        Self::Unix { total: 0, writer: writer() }
    }

    pub fn new_checkstyle() -> Self {
        Self::Checkstyle { diagnostics: vec![] }
    }

    pub fn finish(&mut self) {
        match self {
            Self::Graphical { writer, .. } => {
                writer.flush().unwrap();
            }
            // NOTE: this output does not conform to eslint json format yet
            // https://eslint.org/docs/latest/use/formatters/#json
            Self::Json { diagnostics } => {
                format_json(diagnostics);
            }
            Self::Unix { total, writer } => {
                if *total > 0 {
                    let line = format!("\n{total} problem{}\n", if *total > 1 { "s" } else { "" });
                    writer.write_all(line.as_bytes()).unwrap();
                }
                writer.flush().unwrap();
            }
            Self::Checkstyle { diagnostics } => {
                format_checkstyle(diagnostics);
            }
        }
    }

    pub fn render_diagnostics(&mut self, s: &[u8]) {
        match self {
            Self::Graphical { writer, .. } | Self::Unix { writer, .. } => {
                writer.write_all(s).unwrap();
            }
            Self::Json { .. } | Self::Checkstyle { .. } => {}
        }
    }

    pub fn render_error(&mut self, error: Error) -> Option<String> {
        match self {
            Self::Graphical { handler, .. } => {
                let mut output = String::new();
                handler.render_report(&mut output, error.as_ref()).unwrap();
                Some(output)
            }
            Self::Json { diagnostics } | Self::Checkstyle { diagnostics } => {
                diagnostics.push(error);
                None
            }
            Self::Unix { total: count, .. } => {
                *count += 1;
                Some(format_unix(&error))
            }
        }
    }
}

struct Info {
    line: usize,
    column: usize,
    filename: String,
    message: String,
    severity: Severity,
    rule_id: Option<String>,
}

impl Info {
    fn new(diagnostic: &Error) -> Self {
        let mut line = 0;
        let mut column = 0;
        let mut filename = String::new();
        let mut message = String::new();
        let mut severity = Severity::Warning;
        let mut rule_id = None;
        if let Some(mut labels) = diagnostic.labels() {
            if let Some(source) = diagnostic.source_code() {
                if let Some(label) = labels.next() {
                    if let Ok(span_content) = source.read_span(label.inner(), 0, 0) {
                        line = span_content.line() + 1;
                        column = span_content.column() + 1;
                        if let Some(name) = span_content.name() {
                            filename = name.to_string();
                        };
                        if matches!(diagnostic.severity(), Some(Severity::Error)) {
                            severity = Severity::Error;
                        }
                        let msg = diagnostic.to_string();
                        // Our messages usually comes with `eslint(rule): message`
                        (rule_id, message) = msg.split_once(':').map_or_else(
                            || (None, msg.to_string()),
                            |(id, msg)| (Some(id.to_string()), msg.trim().to_string()),
                        );
                    }
                }
            }
        }
        Self { line, column, filename, message, severity, rule_id }
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-json>
fn format_json(diagnostics: &mut Vec<Error>) {
    let handler = JSONReportHandler::new();
    let messages = diagnostics
        .drain(..)
        .map(|error| {
            let mut output = String::from("\t");
            handler.render_report(&mut output, error.as_ref()).unwrap();
            output
        })
        .collect::<Vec<_>>()
        .join(",\n");
    println!("[\n{messages}\n]");
}

/// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-unix>
fn format_unix(diagnostic: &Error) -> String {
    let Info { line, column, filename, message, severity, rule_id } = Info::new(diagnostic);
    let severity = match severity {
        Severity::Error => "Error",
        _ => "Warning",
    };
    let rule_id =
        rule_id.map_or_else(|| Cow::Borrowed(""), |rule_id| Cow::Owned(format!("/{rule_id}")));
    format!("{filename}:{line}:{column}: {message} [{severity}{rule_id}]\n")
}

fn format_checkstyle(diagnostics: &[Error]) {
    let infos = diagnostics.iter().map(Info::new).collect::<Vec<_>>();
    let mut grouped: HashMap<String, Vec<Info>> = HashMap::new();
    for info in infos {
        grouped.entry(info.filename.clone()).or_default().push(info);
    }
    let messages = grouped.into_values().map(|infos| {
         let messages = infos
             .iter()
             .fold(String::new(), |mut acc, info| {
                 let Info { line, column, message, severity, rule_id, .. } = info;
                 let severity = match severity {
                     Severity::Error => "error",
                     _ => "warning",
                 };
                 let message = rule_id.as_ref().map_or_else(|| xml_escape(message), |rule_id| Cow::Owned(format!("{} ({rule_id})", xml_escape(message))));
                 let source = rule_id.as_ref().map_or_else(|| Cow::Borrowed(""), |rule_id| Cow::Owned(format!("eslint.rules.{rule_id}")));
                 let line = format!(r#"<error line="{line}" column="{column}" severity="{severity}" message="{message}" source="{source}" />"#);
                 acc.push_str(&line);
                 acc
             });
         let filename = &infos[0].filename;
         format!(r#"<file name="{filename}">{messages}</file>"#)
     }).collect::<Vec<_>>().join(" ");
    println!(
        r#"<?xml version="1.0" encoding="utf-8"?><checkstyle version="4.3">{messages}</checkstyle>"#
    );
}

/// <https://github.com/tafia/quick-xml/blob/6e34a730853fe295d68dc28460153f08a5a12955/src/escapei.rs#L84-L86>
fn xml_escape(raw: &str) -> Cow<str> {
    xml_escape_impl(raw, |ch| matches!(ch, b'<' | b'>' | b'&' | b'\'' | b'\"'))
}

fn xml_escape_impl<F: Fn(u8) -> bool>(raw: &str, escape_chars: F) -> Cow<str> {
    let bytes = raw.as_bytes();
    let mut escaped = None;
    let mut iter = bytes.iter();
    let mut pos = 0;
    while let Some(i) = iter.position(|&b| escape_chars(b)) {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let escaped = escaped.as_mut().expect("initialized");
        let new_pos = pos + i;
        escaped.extend_from_slice(&bytes[pos..new_pos]);
        match bytes[new_pos] {
            b'<' => escaped.extend_from_slice(b"&lt;"),
            b'>' => escaped.extend_from_slice(b"&gt;"),
            b'\'' => escaped.extend_from_slice(b"&apos;"),
            b'&' => escaped.extend_from_slice(b"&amp;"),
            b'"' => escaped.extend_from_slice(b"&quot;"),

            // This set of escapes handles characters that should be escaped
            // in elements of xs:lists, because those characters works as
            // delimiters of list elements
            b'\t' => escaped.extend_from_slice(b"&#9;"),
            b'\n' => escaped.extend_from_slice(b"&#10;"),
            b'\r' => escaped.extend_from_slice(b"&#13;"),
            b' ' => escaped.extend_from_slice(b"&#32;"),
            _ => unreachable!(
                "Only '<', '>','\', '&', '\"', '\\t', '\\r', '\\n', and ' ' are escaped"
            ),
        }
        pos = new_pos + 1;
    }

    if let Some(mut escaped) = escaped {
        if let Some(raw) = bytes.get(pos..) {
            escaped.extend_from_slice(raw);
        }
        #[allow(unsafe_code)]
        // SAFETY: we operate on UTF-8 input and search for an one byte chars only,
        // so all slices that was put to the `escaped` is a valid UTF-8 encoded strings
        Cow::Owned(unsafe { String::from_utf8_unchecked(escaped) })
    } else {
        Cow::Borrowed(raw)
    }
}
