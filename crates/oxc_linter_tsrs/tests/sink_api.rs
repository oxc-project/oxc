//! The in-process entry point (`linter::spawn` with a `Sink`): a host that links tsrslint gets the diagnostics the
//! headless binary would write, as values, and the run's error as the last message.

use std::sync::mpsc;

use oxc_linter_tsrs::linter::{Options, Sink, spawn};
use oxc_linter_tsrs::protocol::{HeadlessConfig, HeadlessRule, Output, Payload};

fn project(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tsrslint-sink-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("tsconfig.json"),
        r#"{ "compilerOptions": { "target": "es2020", "strict": true } }"#,
    )
    .unwrap();
    dir.canonicalize().unwrap()
}

fn run(dir: &std::path::Path, file: &std::path::Path, rule: &str) -> Vec<Output> {
    let payload = Payload {
        version: 2,
        configs: vec![HeadlessConfig {
            file_paths: vec![file.to_string_lossy().into_owned()],
            rules: vec![HeadlessRule { name: rule.to_string(), options: None }],
        }],
        source_overrides: None,
        report_syntactic: false,
        report_semantic: false,
    };
    let (tx, rx) = mpsc::channel();
    let sink: Sink = Box::new(move |out| tx.send(out).unwrap());
    let result =
        spawn(payload, &dir.to_string_lossy(), Options::default(), sink).unwrap().join().unwrap();
    let outputs: Vec<Output> = rx.into_iter().collect();
    match (&result, outputs.last()) {
        (Ok(()), Some(Output::Error(e))) => panic!("ok run ended with an error message: {e}"),
        (Err(e), last) => assert!(
            matches!(last, Some(Output::Error(m)) if m.contains(e)),
            "failed run must end with its error message: {e}"
        ),
        _ => {}
    }
    outputs
}

#[test]
fn host_receives_rule_diagnostics_through_the_sink() {
    let dir = project("ok");
    let file = dir.join("a.ts");
    std::fs::write(&file, "async function f() {}\nf();\n").unwrap();
    let outputs = run(&dir, &file, "no-floating-promises");
    let diags: Vec<_> = outputs
        .iter()
        .filter_map(|o| match o {
            Output::Diagnostic(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(diags.len(), 1, "{outputs:?}");
    assert_eq!(diags[0].rule.as_deref(), Some("no-floating-promises"));
    assert_eq!(diags[0].file_path.as_deref(), Some(&*file.to_string_lossy()));
    assert!(diags[0].range.is_some());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn failed_run_ends_with_an_error_message() {
    let dir = project("err");
    let file = dir.join("a.ts");
    std::fs::write(&file, "export {};\n").unwrap();
    let outputs = run(&dir, &file, "no-such-rule");
    assert!(matches!(outputs.last(), Some(Output::Error(_))), "{outputs:?}");
    std::fs::remove_dir_all(&dir).ok();
}
