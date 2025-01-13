use std::{borrow::Cow, io::Write};

use rustc_hash::FxHashMap;

use oxc_diagnostics::{
    reporter::{DiagnosticReporter, Info},
    Error, Severity,
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct CheckStyleOutputFormatter;

impl InternalFormatter for CheckStyleOutputFormatter {
    fn all_rules(&mut self, writer: &mut dyn Write) {
        writeln!(writer, "flag --rules with flag --format=checkstyle is not allowed").unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(CheckstyleReporter::default())
    }
}

#[derive(Default)]
struct CheckstyleReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for CheckstyleReporter {
    fn finish(&mut self) -> Option<String> {
        Some(format_checkstyle(&self.diagnostics))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

fn format_checkstyle(diagnostics: &[Error]) -> String {
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
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><checkstyle version="4.3">{messages}</checkstyle>"#
    )
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

#[cfg(test)]
mod test {
    use oxc_diagnostics::{reporter::DiagnosticReporter, NamedSource, OxcDiagnostic};
    use oxc_span::Span;

    use super::CheckstyleReporter;

    #[test]
    fn reporter() {
        let mut reporter = CheckstyleReporter::default();

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let first_result = reporter.render_error(error);

        // reporter keeps it in memory
        assert!(first_result.is_none());

        // report not gives us all diagnostics at ones
        let second_result = reporter.finish();

        assert!(second_result.is_some());
        assert_eq!(second_result.unwrap(), "<?xml version=\"1.0\" encoding=\"utf-8\"?><checkstyle version=\"4.3\"><file name=\"file://test.ts\"><error line=\"1\" column=\"1\" severity=\"warning\" message=\"error message\" source=\"\" /></file></checkstyle>");
    }
}
