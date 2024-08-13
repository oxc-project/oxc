use std::io::Write;
use std::io::{BufWriter, Stdout};

use crate::JSONReportHandler;

use super::DiagnosticReporter;
use crate::Error;

pub struct JsonReporter<W: ?Sized + Write = std::io::Stdout> {
    diagnostics: Vec<Error>,
    writer: BufWriter<W>,
}

impl Default for JsonReporter<Stdout> {
    fn default() -> Self {
        Self { diagnostics: Vec::new(), writer: BufWriter::new(std::io::stdout()) }
    }
}

impl<W: Write> DiagnosticReporter for JsonReporter<W> {
    // NOTE: this output does not conform to eslint json format yet
    // https://eslint.org/docs/latest/use/formatters/#json
    fn finish(&mut self) {
        self.format_json();
        self.writer.flush().unwrap();
    }

    fn render_diagnostics(&mut self, _s: &[u8]) {}

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

impl<W: Write> JsonReporter<W> {
    pub fn new(writer: W) -> Self {
        Self { diagnostics: Vec::new(), writer: BufWriter::new(writer) }
    }

    /// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-json>
    #[allow(clippy::print_stdout)]
    fn format_json(&mut self) {
        let handler = JSONReportHandler::new();
        let messages = self
            .diagnostics
            .drain(..)
            .map(|error| {
                let mut output = String::from("\t");
                handler.render_report(&mut output, error.as_ref()).unwrap();
                output
            })
            .collect::<Vec<_>>()
            .join(",\n");
        writeln!(self.writer, "[\n{messages}\n]").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::{reporter::DiagnosticReporter, OxcDiagnostic};
    use miette::{LabeledSpan, NamedSource};
    use serde_json::Value;

    use super::*;

    #[test]
    fn test_no_source() {
        let diagnostic = OxcDiagnostic::warn("Something happened!")
            .with_label(LabeledSpan::new(None, 0, 1))
            .with_help("Try something else.");
        let mut buf = Vec::new();

        {
            let mut reporter = JsonReporter::new(&mut buf);
            assert!(reporter.render_error(diagnostic.into()).is_none());
            reporter.finish();
        }

        let output = String::from_utf8(buf).unwrap();

        // Ensure the output is valid JSON, and is an array containing a single object
        let reparsed: Value =
            serde_json::from_str(&output).expect("JSONReporter did not render valid JSON.");
        let arr = reparsed.as_array().expect("Reporter should produce an array of objects.");
        assert_eq!(arr.len(), 1);
        let obj = arr[0].as_object().expect("Reporter should produce an array of objects.");
        // ensure expected keys are present
        {
            assert!(obj.contains_key("message"), "Missing message key");

            // {
            //   "labels": [
            //     { "span": { "length": 1, "offset": 0 } }
            //   ]
            // }
            //
            let labels = obj
                .get("labels")
                .expect("Missing labels key")
                .as_array()
                .expect("Labels should be an array");
            assert_eq!(labels.len(), 1);
            let span = labels[0]
                .as_object()
                .expect("Label should be an object")
                .get("span")
                .expect("Label should have a span");

            let len = span
                .get("length")
                .expect("label.span should have a length")
                .as_i64()
                .expect("label.span.length should be an integer");
            assert_eq!(len, 1);

            let offset = span
                .get("offset")
                .expect("label.span should have an offset")
                .as_i64()
                .expect("label.span.offset should be an integer");
            assert_eq!(offset, 0);
        }

        // snapshot testing using pretty-printed JSON to make diffs easier to read
        let pretty_output = serde_json::to_string_pretty(&reparsed).unwrap();
        insta::assert_snapshot!(pretty_output);
    }

    #[test]
    // #[allow(clippy::cast_possible_truncation)]
    // #[allow(clippy::cast_possible_wrap)]
    // #[allow(clippy::cast_sign_loss)]
    fn test_with_source() {
        const SOURCE: &str = "function main() {
    let x = 1;
}
";
        // cover "let x = 1;"
        const START: u32 = 22;
        const LEN: u32 = 10;
        /// 0-indexed line number
        const LINE: u32 = 1;
        /// 0-indexed column number
        const COL: u32 = 4;

        let diagnostic = OxcDiagnostic::warn("Something happened!")
            // label "let x = 1;"
            .with_label(LabeledSpan::new(None, START as usize, LEN as usize))
            .with_error_code("oxc", "1234")
            .with_help("Try something else.")
            .with_source_code(NamedSource::new("test.js", SOURCE));
        let mut buf = Vec::new();
        {
            let mut reporter = JsonReporter::new(&mut buf);
            assert!(reporter.render_error(diagnostic).is_none());
            reporter.finish();
        }

        let output = String::from_utf8(buf).unwrap();
        let reparsed: Value =
            serde_json::from_str(&output).expect("JSONReporter did not render valid JSON.");
        let diagnostic = reparsed
            .as_array()
            .expect("Reporter should produce an array of objects.")
            .first()
            .expect("Reporter should produce an array of objects.")
            .as_object()
            .expect("Reporter should produce an array of objects.");

        // verify span, it should have offset, size, line, and column
        let labels = diagnostic
            .get("labels")
            .expect("Missing labels key")
            .as_array()
            .expect("Labels should be an array");
        assert_eq!(labels.len(), 1);
        let span = labels[0]
            .as_object()
            .expect("Label should be an object")
            .get("span")
            .expect("Label should have a span");

        let offset = span.get("offset").expect("span has no offset").as_i64().unwrap();
        let len = span.get("length").expect("span has no length").as_i64().unwrap();

        assert_eq!(offset, i64::from(START));
        assert_eq!(len, i64::from(LEN));
        let start = usize::try_from(offset).unwrap();
        let end = usize::try_from(offset + len).unwrap();
        assert_eq!(&SOURCE[start..end], "let x = 1;");

        assert_eq!(span.get("line").expect("span has no line").as_i64().unwrap(), i64::from(LINE));
        assert_eq!(
            span.get("column").expect("span has no column").as_i64().unwrap(),
            i64::from(COL)
        );

        // snapshot testing using pretty-printed JSON to make diffs easier to
        // read
        let pretty_output = serde_json::to_string_pretty(&reparsed).unwrap();
        insta::assert_snapshot!(pretty_output);
    }
}
