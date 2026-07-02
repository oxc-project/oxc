//! Runtime tests - Test262 execution with Node.js
//!
//! Each eligible test262 case is run through three code-production pipelines
//! (codegen only; transform; minify) and each result is executed in a Node.js
//! server, verifying the test still passes. This catches semantic bugs in oxc's
//! codegen, transformer, and minifier.
//!
//! To keep all cores busy, one Node.js server is started per core; each rayon
//! worker sends its requests to a dedicated server.
//!
//! Requires `node` (>= 22) to be installed.

mod test262_status;

use std::{
    io::{self, BufRead, BufReader},
    process::{Child, Command, Stdio},
    thread,
};

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    minifier::{Minifier, MinifierOptions},
    parser::Parser,
    semantic::SemanticBuilder,
    span::SourceType,
    transformer::{HelperLoaderMode, TransformOptions, Transformer},
};
use oxc_tasks_common::local_agent;
use rayon::prelude::*;
use serde::Serialize;

use crate::{
    CoverageResult, Test262File, TestResult,
    test262::{Phase, TestFlag},
    workspace_root,
};
use test262_status::get_v8_test262_failure_paths;

const HOST: &str = "http://localhost";

static SKIP_FEATURES: &[&str] = &[
    // Node's version of V8 doesn't implement these
    "hashbang",
    "legacy-regexp",
    "regexp-duplicate-named-groups",
    "symbols-as-weakmap-keys",
    "tail-call-optimization",
    // We don't care about API-related things
    "ArrayBuffer",
    "change-array-by-copy",
    "DataView",
    "resizable-arraybuffer",
    "ShadowRealm",
    "cross-realm",
    "SharedArrayBuffer",
    "String.prototype.toWellFormed",
    "Symbol.match",
    "Symbol.replace",
    "Symbol.unscopables",
    "Temporal",
    "TypedArray",
    // Added in oxc
    "Array.fromAsync",
    "IsHTMLDDA",
    "iterator-helpers",
    "set-methods",
    "array-grouping",
    // stage 2
    "Intl.DurationFormat",
    // stage 3
    "decorators",
    "explicit-resource-management",
    "source-phase-imports",
    "import-defer",
];

static SKIP_INCLUDES: &[&str] = &[
    // We don't preserve "toString()" on functions
    "nativeFunctionMatcher.js",
];

static SKIP_TEST_CASES: &[&str] = &[
    // node.js runtime error
    "language/expressions/dynamic-import",
    "language/global-code/decl-func.js",
    "language/module-code",
    // formerly S11.13.2_A5.10_T5
    "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v",
    "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue",
    "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue",
    "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue",
    "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue",
];

static SKIP_ESID: &[&str] = &["sec-privatefieldget", "sec-privatefieldset"];

/// The JSON payload sent to the Node.js runtime server for each execution.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunRequest<'a> {
    code: String,
    includes: &'a [Box<str>],
    is_async: bool,
    is_module: bool,
    is_raw: bool,
    import_dir: &'a str,
}

/// Ensures every Node.js server subprocess is killed even if a test panics.
struct ServerGuard(Vec<Child>);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Run the runtime suite over the given test262 files.
///
/// A standard `run_tool` runner: it owns the Node.js server lifecycle (spawn,
/// readiness, graceful shutdown) and returns per-case results; printing and
/// snapshotting are handled by the caller (`AppArgs::run_tool`).
///
/// # Panics
/// Panics if a Node.js runtime server fails to start or never becomes ready.
pub fn run(files: &[Test262File]) -> Vec<CoverageResult> {
    // Warm the V8 status list once (it may fetch over the network) before entering
    // the parallel skip filter, so any failure surfaces deterministically.
    let _ = get_v8_test262_failure_paths();

    // One server per rayon worker keeps every core busy without spawning
    // servers nothing will talk to (the pool is 1 thread under --debug).
    let num_servers = rayon::current_num_threads();

    // A single agent, shared across the rayon threads. Servers respond even for
    // failing tests, so an `Err` here is transport-only (e.g. a request that
    // queued past the client timeout on a loaded machine) — retry once before
    // recording it as a failure; re-executing a test is harmless.
    let http = local_agent();
    let send = |port: u16, request: &RunRequest| -> Result<String, String> {
        let post = || {
            http.post(&format!("{HOST}:{port}/run"))
                .send_json(request)
                .map_err(|err| err.to_string())
                .and_then(|mut res| res.body_mut().read_to_string().map_err(|err| err.to_string()))
        };
        post().or_else(|_| post())
    };

    // Start the Node.js servers. `spawn_server` blocks until each has reported its
    // port, which doubles as the readiness signal, so no polling is needed.
    let mut children = Vec::with_capacity(num_servers);
    let mut ports = Vec::with_capacity(num_servers);
    for _ in 0..num_servers {
        let (child, port) = spawn_server();
        children.push(child);
        ports.push(port);
    }
    let server = ServerGuard(children);

    // Each rayon worker talks to its own server (round-robined if there are more
    // workers than servers); the three variants of a case share one server.
    let mut results: Vec<CoverageResult> = files
        .par_iter()
        .filter(|&file| !skip_case(file))
        .map(|file| {
            let port = ports[rayon::current_thread_index().unwrap_or(0) % ports.len()];
            run_case(file, port, &send)
        })
        .collect();

    // Ask each server to shut down gracefully, then ensure the processes are gone.
    for &port in &ports {
        let _ = http.delete(&format!("{HOST}:{port}")).call();
    }
    drop(server);

    // rayon collects out of order; sort so console output is deterministic.
    // (`snapshot_results` sorts internally, so the snapshot is stable regardless.)
    results.sort_by(|a, b| a.path.cmp(&b.path));

    results
}

/// Spawn a Node.js server and return it with the ephemeral port it bound to.
///
/// Blocks until the server prints its `PORT <n>` handshake line (emitted once it
/// is listening), which is the readiness signal. The rest of the server's stdout
/// is then drained to our stdout on a background thread — a chatty server (e.g.
/// `DEBUG=1`) would otherwise block once the pipe buffer fills.
fn spawn_server() -> (Child, u16) {
    let path = workspace_root().join("src/runtime/runtime.js");
    let mut child = Command::new("node")
        .args(["--experimental-vm-modules"])
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start runtime.js - ensure Node.js is installed");

    let stdout = child.stdout.take().expect("runtime.js stdout is piped");
    let mut reader = BufReader::new(stdout);

    let mut first_line = String::new();
    reader.read_line(&mut first_line).expect("failed to read the port line from runtime.js");
    let port = first_line
        .strip_prefix("PORT ")
        .and_then(|rest| rest.trim().parse::<u16>().ok())
        .unwrap_or_else(|| panic!("expected `PORT <n>` from runtime.js, got {first_line:?}"));

    thread::spawn(move || {
        let _ = io::copy(&mut reader, &mut io::stdout());
    });

    (child, port)
}

fn run_case(
    file: &Test262File,
    port: u16,
    send: &impl Fn(u16, &RunRequest) -> Result<String, String>,
) -> CoverageResult {
    // Compute the import dir once per file, rather than per variant (it calls
    // `workspace_root()`, which walks the filesystem to find the project root).
    let import_dir =
        workspace_root().join(file.path.parent().unwrap()).to_string_lossy().into_owned();
    let result = run_variants(file, &import_dir, port, send);
    CoverageResult { path: file.path.clone(), should_fail: false, result }
}

/// Run the three code-production variants in order, stopping at the first failure.
fn run_variants(
    file: &Test262File,
    import_dir: &str,
    port: u16,
    send: &impl Fn(u16, &RunRequest) -> Result<String, String>,
) -> TestResult {
    let code = get_code(file, false, false);
    let result = run_test_code(file, "codegen", code, import_dir, port, send);
    if result != TestResult::Passed {
        return result;
    }

    let code = get_code(file, true, false);
    let result = run_test_code(file, "transform", code, import_dir, port, send);
    if result != TestResult::Passed {
        return result;
    }

    let base_path = file.path.to_string_lossy();
    let test262_path = base_path.trim_start_matches("test262/test/");

    // Minifier does not conform to annexB.
    if test262_path.starts_with("annexB") {
        return TestResult::Passed;
    }

    // Unable to minify non-strict code, which may contain syntaxes the minifier does
    // not support (e.g. `with`).
    if file.meta.flags.contains(&TestFlag::NoStrict) {
        return TestResult::Passed;
    }

    // None of the minifier conform to "fn-name-cover.js"
    // `let xCover = (0, function() {});` xCover.name is ''
    // `let xCover = function() {};` xCover.name is 'xCover'
    // e.g. https://github.com/tc39/test262/blob/main/test/language/statements/let/fn-name-cover.js
    if test262_path.ends_with("fn-name-cover.js") {
        return TestResult::Passed;
    }

    let code = get_code(file, false, true);
    run_test_code(file, "minify", code, import_dir, port, send)
}

fn run_test_code(
    file: &Test262File,
    case: &'static str,
    code: String,
    import_dir: &str,
    port: u16,
    send: &impl Fn(u16, &RunRequest) -> Result<String, String>,
) -> TestResult {
    let flags = &file.meta.flags;

    let request = RunRequest {
        code,
        includes: file.meta.includes.as_ref(),
        is_async: flags.contains(&TestFlag::Async),
        is_module: flags.contains(&TestFlag::Module),
        is_raw: flags.contains(&TestFlag::Raw),
        import_dir,
    };

    match send(port, &request) {
        Ok(output) => {
            // A passing test produces no output. A runtime-phase negative test passes
            // when it throws the expected error type.
            let passed = output.is_empty()
                || file.meta.negative.as_ref().is_some_and(|negative| {
                    negative.phase.is_runtime() && output.starts_with(&*negative.error_type)
                });
            if passed { TestResult::Passed } else { TestResult::GenericError(case, output) }
        }
        Err(error) => TestResult::GenericError(case, error),
    }
}

/// Produce the executable code for a single variant.
///
/// * `transform` runs the full transformer pipeline (down-levelling syntax).
/// * `minify` runs the compressor (without mangling) and prints minified output.
fn get_code(file: &Test262File, transform: bool, minify: bool) -> String {
    let source_text = file.code.as_str();
    let flags = &file.meta.flags;
    let is_module = flags.contains(&TestFlag::Module);
    let is_only_strict = flags.contains(&TestFlag::OnlyStrict);
    let source_type = SourceType::cjs().with_script(!is_module).with_module(is_module);
    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, source_text, source_type).parse().program;

    if transform {
        let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
        let mut options = TransformOptions::enable_all();
        options.jsx.refresh = None;
        options.helper_loader.mode = HelperLoaderMode::External;
        options.typescript.only_remove_type_imports = true;
        Transformer::new(&allocator, &file.path, &options)
            .build_with_scoping(scoping, &mut program);
    }

    let scoping = if minify {
        Minifier::new(MinifierOptions { mangle: None, ..MinifierOptions::default() })
            .minify(&allocator, &mut program)
            .scoping
    } else {
        None
    };

    let mut text = Codegen::new()
        .with_options(if minify { CodegenOptions::minify() } else { CodegenOptions::default() })
        .with_scoping(scoping)
        .build(&program)
        .code;
    if is_only_strict {
        text = format!("\"use strict\";\n{text}");
    }
    if is_module {
        text = format!("{text}\n export {{}}");
    }
    text
}

/// Decide whether a case is ineligible for runtime execution.
///
/// These lists encode years of test262 triage and are ported verbatim from the
/// original runner.
fn skip_case(file: &Test262File) -> bool {
    let base_path = file.path.to_string_lossy();
    let test262_path = base_path.trim_start_matches("test262/test/");
    let meta = &file.meta;
    let includes = &meta.includes;
    let features = &meta.features;

    // Negative tests that fail at parse time can't produce runnable code.
    meta.negative.as_ref().is_some_and(|negative| negative.phase == Phase::Parse)
        || meta.esid.as_ref().is_some_and(|esid| SKIP_ESID.contains(&esid.as_ref()))
        || base_path.contains("built-ins")
        || base_path.contains("staging")
        || base_path.contains("intl402")
        || includes.iter().any(|include| SKIP_INCLUDES.contains(&include.as_ref()))
        || features.iter().any(|feature| SKIP_FEATURES.contains(&feature.as_ref()))
        || SKIP_TEST_CASES.iter().any(|path| test262_path.starts_with(path))
        || get_v8_test262_failure_paths().iter().any(|path| {
            if let Some(path) = path.strip_suffix('*') {
                test262_path.starts_with(path)
            } else {
                test262_path.trim_end_matches(".js") == path
            }
        })
        || file.code.contains("$262")
        || file.code.contains("$DONOTEVALUATE()")
}
