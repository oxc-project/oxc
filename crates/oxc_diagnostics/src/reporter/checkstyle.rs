use std::borrow::Cow;

use super::{DiagnosticReporter, Info};
use crate::{Error, Severity};
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct CheckstyleReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for CheckstyleReporter {
    fn finish(&mut self) {
        format_checkstyle(&self.diagnostics);
    }

    fn render_diagnostics(&mut self, _s: &[u8]) {}

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

#[allow(clippy::print_stdout)]
fn format_checkstyle(diagnostics: &[Error]) {
    let infos = diagnostics.iter().map(Info::new).collect::<Vec<_>>();
    let mut grouped: FxHashMap<String, Vec<Info>> = FxHashMap::default();
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

        // SAFETY: we operate on UTF-8 input and search for an one byte chars only,
        // so all slices that was put to the `escaped` is a valid UTF-8 encoded strings
        Cow::Owned(unsafe { String::from_utf8_unchecked(escaped) })
    } else {
        Cow::Borrowed(raw)
    }
}
