use std::path::Path;

use miette::{Diagnostic, NamedSource, SourceSpan};
use oxc_diagnostics::DiagnosticService;
use thiserror::Error as ThisError;
#[derive(Debug, Diagnostic, ThisError)]
#[error("something went wrong")]
#[diagnostic(help("try doing this instead"))]
struct Foo {
    // The Source that we're gonna be printing snippets out of.
    // This can be a String if you don't have or care about file names.
    #[source_code]
    src: NamedSource,
    // Snippets and highlights can be included in the diagnostic!
    #[label("This bit here")]
    bad_bit: SourceSpan,
}

#[test]
fn test() {
    use std::thread;

    let diagnostic_service =
        DiagnosticService::default().with_output_type(Some("json".to_string()));

    let sender = diagnostic_service.sender().clone();
    thread::spawn(move || {
        let src = "source\n  text\n    here".to_string();
        let diagnostics = DiagnosticService::wrap_diagnostics(
            Path::new("bad_file.rs"),
            &src,
            vec![
                Foo {
                    src: NamedSource::new("bad_file.rs", src.clone()),
                    bad_bit: (9, 4).into(),
                }
                .into(),
                Foo {
                    src: NamedSource::new("bad_file.rs", src.clone()),
                    bad_bit: (9, 4).into(),
                }
                .into(),
            ],
        );
        sender.send(Some(diagnostics)).unwrap();
        sender.send(None).unwrap();
    });

    diagnostic_service.run();
}
