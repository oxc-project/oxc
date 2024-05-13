use insta::{glob, Settings};
use oxc_diagnostics::{Error, OxcDiagnostic, Severity};
use std::{
    fmt::{self, Write},
    fs,
    path::Path,
};

use oxc_allocator::Allocator;
use oxc_parser::Parser;

use oxc_span::SourceType;

#[test]
fn test_does_parse() {
    fn do_test(path: &Path) {
        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(&path)
            .map_err(|e| {
                Error::msg(format!(
                    "Failed to parse source type from path: {:?} unknown extension {}",
                    path.display(),
                    e.0
                ))
            })
            .unwrap();
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        assert_eq!(
            ret.errors.len(),
            0,
            "{} failed to parse with {} errors: {:?}",
            path.display(),
            ret.errors.len(),
            ret.errors
        );

        let filename = path.file_name().and_then(|f| f.to_str()).unwrap();
        let mut settings = Settings::clone_current();
        settings.set_snapshot_suffix(filename);
        settings.bind(|| insta::assert_json_snapshot!(ret.program));
    }

    let mut settings = Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_allow_empty_glob(true);

    settings.set_info(&"Correctly parses without errors");
    settings.set_snapshot_path("snapshots/ts/true_pass");
    settings.bind(|| {
        glob!("fixtures/ts", "true_pass/*.ts", |path| do_test(path));
        glob!("fixtures/ts", "true_pass/*.tsx", |path| do_test(path));
    });

    settings.set_info(&"Incorrectly parses without errors when it shouldn't");
    settings.set_snapshot_path("snapshots/ts/false_pass");
    settings.bind(|| {
        glob!("fixtures/ts", "false_pass/*.ts", |path| do_test(path));
        glob!("fixtures/ts", "false_pass/*.tsx", |path| do_test(path));
    });
}

#[test]
fn test_does_not_parse() {
    fn do_test(path: &Path) {
        /// Just a ballpark for initializing the error message string with a
        /// reasonable capacity.
        const AVG_ERROR_MSG_LEN: usize = 512;

        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(&path)
            .map_err(|e| {
                Error::msg(format!(
                    "Failed to parse source type from path: {:?} unknown extension {}",
                    path.display(),
                    e.0
                ))
            })
            .unwrap();

        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        assert!(
            ret.errors.len() > 0,
            "File '{}' should not parse",
            path.file_name().and_then(|f| f.to_str()).unwrap()
        );
        let program_json = serde_json::to_string_pretty(&ret.program).unwrap();
        let mut error_str = String::with_capacity(ret.errors.len() * AVG_ERROR_MSG_LEN);
        format_errors(&mut error_str, &ret.errors).unwrap();
        let snapshot_str = format!(
            "============= ERRORS =============\n{}\n============= PROGRAM =============\n{}",
            error_str, program_json,
        );
        let mut settings = Settings::clone_current();
        settings.set_snapshot_suffix(path.file_name().and_then(|f| f.to_str()).unwrap());
        settings.bind(|| insta::assert_snapshot!(snapshot_str));
    }

    let mut settings = Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_allow_empty_glob(true);

    settings.set_info(&"Correctly fails to parse");
    settings.set_snapshot_path("snapshots/ts/true_fail");
    settings.bind(|| {
        glob!("fixtures/ts", "true_fail/*.ts", |path| do_test(path));
        glob!("fixtures/ts", "true_fail/*.tsx", |path| do_test(path));
    });

    settings.set_info(&"Should parse but doesn't");
    settings.set_snapshot_path("snapshots/ts/false_fail");
    settings.bind(|| {
        glob!("fixtures/ts", "false_fail/*.ts", |path| do_test(path));
        glob!("fixtures/ts", "false_fail/*.tsx", |path| do_test(path));
    })
}

fn format_errors(out: &mut String, err: &Vec<OxcDiagnostic>) -> fmt::Result {
    for error in err {
        out.push_str("- ");
        format_error(out, error)?;
        out.push('\n');
    }
    Ok(())
}

fn format_error<W: Write>(out: &mut W, err: &OxcDiagnostic) -> fmt::Result {
    let severity: &'static str = match err.severity {
        Severity::Advice => "advice",
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    write!(out, "{}: {}", severity, err.message)?;
    for label in err.labels.iter().map(|ls| ls.iter()).flatten() {
        write!(out, " (offset: {}, len: {})", label.offset(), label.len())?;
    }
    Ok(())
}
