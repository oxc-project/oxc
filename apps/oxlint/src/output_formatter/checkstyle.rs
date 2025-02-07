use std::borrow::Cow;

use rustc_hash::FxHashMap;

use oxc_diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
    Error, Severity,
};

use crate::output_formatter::{xml_utils::xml_escape, InternalFormatter};

#[derive(Debug, Default)]
pub struct CheckStyleOutputFormatter;

impl InternalFormatter for CheckStyleOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(CheckstyleReporter::default())
    }
}

/// Reporter to output diagnostics in checkstyle format
///
/// Checkstyle Format Documentation: <https://checkstyle.sourceforge.io/>
#[derive(Default)]
struct CheckstyleReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for CheckstyleReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
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
                 let Info { start, message, severity, rule_id, .. } = info;
                 let severity = match severity {
                     Severity::Error => "error",
                     _ => "warning",
                 };
                 let message = rule_id.as_ref().map_or_else(|| xml_escape(message), |rule_id| Cow::Owned(format!("{} ({rule_id})", xml_escape(message))));
                 let source = rule_id.as_ref().map_or_else(|| Cow::Borrowed(""), |rule_id| Cow::Owned(format!("eslint.rules.{rule_id}")));
                 let line = format!(r#"<error line="{}" column="{}" severity="{severity}" message="{message}" source="{source}" />"#, start.line, start.column);
                 acc.push_str(&line);
                 acc
             });
         let filename = &infos[0].filename;
         format!(r#"<file name="{filename}">{messages}</file>"#)
     }).collect::<Vec<_>>().join(" ");
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><checkstyle version=\"4.3\">{messages}</checkstyle>\n"
    )
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{
        reporter::{DiagnosticReporter, DiagnosticResult},
        NamedSource, OxcDiagnostic,
    };
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
        let second_result = reporter.finish(&DiagnosticResult::default());

        assert!(second_result.is_some());
        assert_eq!(second_result.unwrap(), "<?xml version=\"1.0\" encoding=\"utf-8\"?><checkstyle version=\"4.3\"><file name=\"file://test.ts\"><error line=\"1\" column=\"1\" severity=\"warning\" message=\"error message\" source=\"\" /></file></checkstyle>\n");
    }
}
