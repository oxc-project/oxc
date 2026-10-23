// Port of cmd/tsgolint/headless.go + internal/linter/linter.go.

use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rustc_hash::{FxHashMap, FxHashSet};
use tsrs_ast::{self as ast, Kind, Node, SourceFile, SourceFileParseOptions};
use tsrs_checker::Checker;
use tsrs_compiler::{CompilerHost, Program, ProgramOptions, new_compiler_host, new_program};
use tsrs_core::CompilerOptions;
use tsrs_core::tspath::{self, Path};
use tsrs_core::{P, Tristate};
use tsrs_vfs::{FS, bundled, cachedvfs, osvfs};

use crate::overlayfs::OverlayFS;
use crate::protocol::{self, MessageType, Output, Payload};
use crate::rule::{Ctx, KIND_SLOTS, Listener, ReportedDiagnostic, Rule, RuleVisitor};
use crate::rules;
use crate::tsconfig::{ExtendedConfigCache, ParseHost, TsConfigResolver};

#[derive(Default, Clone)]
pub struct Options {
    pub fix: bool,
    pub fix_suggestions: bool,
    pub debug_timings: bool,
}

fn parse_flags(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].trim_start_matches('-');
        let (name, value) = match a.split_once('=') {
            Some((n, v)) => (n, Some(v.to_string())),
            None => (a, None),
        };
        match name {
            "fix" => o.fix = value.is_none_or(|v| v == "true"),
            "fix-suggestions" => o.fix_suggestions = value.is_none_or(|v| v == "true"),
            "debug" => {
                let v = match value {
                    Some(v) => v,
                    None => {
                        i += 1;
                        args.get(i).cloned().unwrap_or_default()
                    }
                };
                for part in v.split(',') {
                    match part.trim() {
                        "timings" => o.debug_timings = true,
                        "" => {}
                        other => return Err(format!("unknown debug option: {other}")),
                    }
                }
            }
            // Go profiling flags (oxlint's OXLINT_TSGOLINT_TRACE/CPU/HEAP/ALLOCS). Accepted and ignored:
            // profile tsrslint with samply / Instruments instead.
            "trace" | "cpuprof" | "heap" | "allocs" => {
                if value.is_none() {
                    i += 1;
                }
                eprintln!("tsrslint: -{name} is a Go profiling flag and is ignored");
            }
            other => return Err(format!("flag provided but not defined: -{other}")),
        }
        i += 1;
    }
    Ok(o)
}

/// Rules configured for one payload config group.
struct RuleSet {
    rules: Vec<&'static dyn Rule>,
}

pub fn run_headless(args: &[String]) -> i32 {
    let debug = std::env::var("OXC_LOG").is_ok_and(|v| v == "debug");
    let t0 = Instant::now();
    let log = |msg: String| {
        if debug {
            eprintln!("[{:>9.3}s] {msg}", t0.elapsed().as_secs_f64());
        }
    };
    let opts = match parse_flags(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error parsing options: {e}");
            return 1;
        }
    };
    let cwd = match std::env::current_dir() {
        Ok(c) => tspath::normalize_path(&c.to_string_lossy()),
        Err(e) => {
            protocol::write_error_message(&format!("error getting current directory: {e}"));
            return 1;
        }
    };
    let mut data = Vec::new();
    if let Err(e) = std::io::stdin().read_to_end(&mut data) {
        protocol::write_error_message(&format!("error reading from stdin: {e}"));
        return 1;
    }
    let payload = match protocol::deserialize_payload(&data) {
        Ok(p) => p,
        Err(e) => {
            protocol::write_error_message(&format!("error parsing config: {e}"));
            return 1;
        }
    };
    match run(&payload, &cwd, &opts, &log) {
        Ok(()) => {
            crate::sched::report_totals(t0.elapsed());
            0
        }
        // The error message went to stdout through the sink.
        Err(_) => 1,
    }
}

/// Where a run's output goes (`protocol::Output`), called on the run's output thread.
pub type Sink = Box<dyn FnMut(Output) + Send>;

/// Starts a lint of `payload` on a thread with the stack type checking needs (what `main` does for the headless
/// binary) and returns its handle; the output goes to `sink`, ending with `Output::Error` when the run fails. `cwd`
/// is the host's working directory as `std::env::current_dir` reports it. For hosts that link tsrslint instead of
/// running the `tsgolint headless` binary.
pub fn spawn(
    payload: Payload,
    cwd: &str,
    opts: Options,
    sink: Sink,
) -> std::io::Result<std::thread::JoinHandle<Result<(), String>>> {
    let cwd = tspath::normalize_path(cwd);
    let debug = std::env::var("OXC_LOG").is_ok_and(|v| v == "debug");
    std::thread::Builder::new().name("tsrslint".to_string()).stack_size(512 << 20).spawn(
        move || {
            let t0 = Instant::now();
            let log = |msg: String| {
                if debug {
                    eprintln!("[{:>9.3}s] {msg}", t0.elapsed().as_secs_f64());
                }
            };
            let result = run_with_sink(&payload, &cwd, &opts, &log, sink);
            if result.is_ok() {
                crate::sched::report_totals(t0.elapsed());
            }
            result
        },
    )
}

/// The FS of the config parses and of every program. Its cache is the only one: program hosts use it as is
/// (`new_compiler_host`, not `new_cached_fs_compiler_host`), so lookups are cached once for the whole run.
fn base_fs(payload: &Payload) -> Arc<dyn FS> {
    let os: Arc<dyn FS> = Arc::new(osvfs::fs());
    #[cfg(target_os = "macos")]
    let os: Arc<dyn FS> = Arc::new(crate::boundedfs::BoundedReadsFS::new(os, 4));
    let base: Arc<dyn FS> = match &payload.source_overrides {
        Some(o) if !o.is_empty() => Arc::new(OverlayFS::new(os, o.clone())),
        _ => os,
    };
    Arc::new(bundled::wrap_fs(cachedvfs::from(base)))
}

/// Lints `payload` and writes the output to stdout in the headless protocol's framing (the `tsgolint headless`
/// binary).
pub fn run(
    payload: &Payload,
    cwd: &str,
    opts: &Options,
    log: &dyn Fn(String),
) -> Result<(), String> {
    let mut w = std::io::BufWriter::with_capacity(4096 * 100, std::io::stdout());
    let sink: Sink = Box::new(move |out| {
        let _ = match out {
            Output::Diagnostic(d) => protocol::write_message(&mut w, MessageType::Diagnostic, &d),
            Output::Timing(t) => {
                protocol::write_message(&mut w, MessageType::Timing, &t).and_then(|()| w.flush())
            }
            Output::Error(e) => {
                let _ = w.flush();
                protocol::write_error_message(&e);
                Ok(())
            }
        };
    });
    run_with_sink(payload, cwd, opts, log, sink)
}

/// Lints `payload` and hands every output message to `sink` (on the output thread); a failed run ends with
/// `Output::Error` carrying the error this returns. The sink is dropped when the run ends.
pub fn run_with_sink(
    payload: &Payload,
    cwd: &str,
    opts: &Options,
    log: &dyn Fn(String),
    sink: Sink,
) -> Result<(), String> {
    let (mut sink, result) = lint(payload, cwd, opts, log, sink);
    if let Err(e) = &result {
        sink(Output::Error(format!("error running linter: {e}")));
    }
    result
}

/// Rules of each payload config: one instance per distinct (name, options); unknown rules are a hard error (Go
/// panics).
fn build_rule_sets(payload: &Payload) -> Result<Vec<RuleSet>, String> {
    let mut rule_cache: FxHashMap<String, &'static dyn Rule> = FxHashMap::default();
    let mut rule_sets: Vec<RuleSet> = Vec::new();
    for config in &payload.configs {
        let mut set = RuleSet { rules: Vec::new() };
        for r in &config.rules {
            let key = format!(
                "{}\u{0}{}",
                r.name,
                r.options.as_ref().map(|o| o.to_string()).unwrap_or_default()
            );
            let rule = match rule_cache.get(&key) {
                Some(r) => *r,
                None => {
                    let created = rules::create_rule(&r.name, r.options.as_ref())?;
                    let leaked: &'static dyn Rule = Box::leak(created);
                    rule_cache.insert(key, leaked);
                    leaked
                }
            };
            set.rules.push(rule);
        }
        rule_sets.push(set);
    }
    Ok(rule_sets)
}

/// `run_with_sink` without the error message: returns the sink with the result so the caller can send it.
fn lint(
    payload: &Payload,
    cwd: &str,
    opts: &Options,
    log: &dyn Fn(String),
    mut sink: Sink,
) -> (Sink, Result<(), String>) {
    crate::memstats::mark("run start");
    use_sparse_id_pages();
    let fs = base_fs(payload);
    let host: &'static ParseHost =
        Box::leak(Box::new(ParseHost { fs: Arc::clone(&fs), cwd: cwd.to_string() }));
    let ext_cache: &'static ExtendedConfigCache = Box::leak(Box::default());
    let declarations: &'static DeclarationCache = Box::leak(Box::default());

    let rule_sets = match build_rule_sets(payload) {
        Ok(sets) => sets,
        Err(e) => return (sink, Err(e)),
    };

    // File -> rule set.
    let use_case_sensitive = fs.use_case_sensitive_file_names();
    let mut files: Vec<String> = Vec::new();
    let mut file_rules: FxHashMap<Path, usize> = FxHashMap::default();
    for (idx, config) in payload.configs.iter().enumerate() {
        for f in &config.file_paths {
            let normalized = tspath::normalize_slashes(f);
            file_rules.insert(tspath::to_path(&normalized, cwd, use_case_sensitive), idx);
            files.push(normalized);
        }
    }
    log(format!("Starting to assign files to programs. Total files: {}", files.len()));
    let resolver = TsConfigResolver::new(host, ext_cache);
    let assignment = resolver.resolve_all(&files);
    crate::memstats::mark("files assigned to configs");
    let mut programs: Vec<(String, Vec<String>)> = Vec::new();
    let mut program_index: FxHashMap<String, usize> = FxHashMap::default();
    let mut unmatched: Vec<String> = Vec::new();
    for (file, config) in files.iter().zip(assignment) {
        if config.is_empty() {
            unmatched.push(file.clone());
        } else {
            let i = *program_index.entry(config.clone()).or_insert_with(|| {
                programs.push((config.clone(), Vec::new()));
                programs.len() - 1
            });
            programs[i].1.push(file.clone());
        }
    }
    log(format!(
        "Done assigning files to programs. Total programs: {}. Unmatched files: {}",
        programs.len(),
        unmatched.len()
    ));

    // Output thread (headless.go's diagnostics goroutine); it owns the sink until the channel closes.
    let (tx, rx) = mpsc::sync_channel::<protocol::Diagnostic>(4096);
    let writer = std::thread::spawn(move || {
        for d in rx {
            sink(Output::Diagnostic(d));
        }
        sink
    });

    let timings: Mutex<FxHashMap<&'static str, (Duration, u64)>> = Mutex::new(FxHashMap::default());
    let ctx = LintCtx {
        opts,
        rule_sets: &rule_sets,
        file_rules: &file_rules,
        tx: &tx,
        timings: &timings,
        use_case_sensitive,
        cwd,
    };

    let mut result = Ok(());
    for (config_file_name, config_files) in &programs {
        log(format!("Running linter on program: {config_file_name}"));
        let parsed = resolver.load(config_file_name);
        if !parsed.errors.is_empty() || parsed.config.is_none() {
            for d in &parsed.errors {
                send_internal_tsconfig_diag(&tx, *d, config_file_name);
            }
            continue;
        }
        let config = parsed.config.unwrap();
        if !config.errors.is_empty() && !suppress_program_diagnostics() {
            for d in &config.errors {
                send_internal_tsconfig_diag(&tx, *d, config_file_name);
            }
            continue;
        }
        let dir = tspath::get_directory_path(config_file_name);
        let compiler_host = shared_declarations_host(
            new_compiler_host(
                &dir,
                Arc::clone(&fs),
                &bundled::lib_path(),
                Some(Arc::new(ProgramExtCache(ext_cache))),
                None,
            ),
            declarations,
        );
        let mut popts = ProgramOptions::new(config, compiler_host);
        popts.use_source_of_project_reference = true;
        popts.single_threaded = Tristate::False;
        let program = new_program(popts);
        let program_diags = program.get_program_diagnostics();
        if !program_diags.is_empty() && !suppress_program_diagnostics() {
            for d in &program_diags {
                send_internal_tsconfig_diag(&tx, *d, config_file_name);
            }
            continue;
        }
        program.bind_source_files();
        log(format!("Program created with {} source files", program.source_files().len()));
        let wanted: FxHashMap<Path, &String> = config_files
            .iter()
            .map(|f| (tspath::to_path(f, &dir, use_case_sensitive), f))
            .collect();
        let source_files: Vec<P<SourceFile>> = program
            .source_files()
            .iter()
            .copied()
            .filter(|sf| wanted.contains_key(sf.path()))
            .collect();
        if source_files.len() != wanted.len() {
            result = Err(format!(
                "expected {} files in program {config_file_name}, found {}",
                wanted.len(),
                source_files.len()
            ));
            break;
        }
        crate::memstats::mark("program built and bound");
        lint_program(program, &source_files, payload, &ctx);
        crate::memstats::mark("program linted");
    }

    if result.is_ok() && !unmatched.is_empty() {
        log(format!("Running linter on inferred program with {} files", unmatched.len()));
        let compiler_host = shared_declarations_host(
            new_compiler_host(
                cwd,
                Arc::clone(&fs),
                &bundled::lib_path(),
                Some(Arc::new(ProgramExtCache(ext_cache))),
                None,
            ),
            declarations,
        );
        let config = P::new(tsrs_tsoptions::new_parsed_command_line(
            P::new(inferred_compiler_options()),
            unmatched.clone(),
            Vec::new(),
            tspath::ComparePathsOptions {
                use_case_sensitive_file_names: use_case_sensitive,
                current_directory: cwd.to_string(),
            },
        ));
        let mut popts = ProgramOptions::new(config, compiler_host);
        popts.single_threaded = Tristate::False;
        let program = new_program(popts);
        program.bind_source_files();
        let wanted: FxHashSet<Path> =
            unmatched.iter().map(|f| tspath::to_path(f, cwd, use_case_sensitive)).collect();
        let source_files: Vec<P<SourceFile>> = program
            .source_files()
            .iter()
            .copied()
            .filter(|sf| wanted.contains(sf.path()))
            .collect();
        crate::memstats::mark("inferred program built");
        lint_program(program, &source_files, payload, &ctx);
        crate::memstats::mark("inferred program linted");
    }

    drop(tx);
    let mut sink = match writer.join() {
        Ok(sink) => sink,
        // The sink went with the thread: nothing left to report through.
        Err(_) => return (Box::new(|_| {}), Err("output thread panicked".to_string())),
    };
    if let Err(e) = result {
        return (sink, Err(e));
    }

    if opts.debug_timings {
        let mut rules: Vec<protocol::RuleTiming> = timings
            .into_inner()
            .unwrap()
            .into_iter()
            .map(|(name, (d, calls))| protocol::RuleTiming {
                rule_name: name.to_string(),
                duration: d.as_nanos() as u64,
                calls,
            })
            .collect();
        // RuleTimingStore.Collect's order: slowest first, ties by name.
        rules.sort_by(|a, b| {
            b.duration.cmp(&a.duration).then_with(|| a.rule_name.cmp(&b.rule_name))
        });
        sink(Output::Timing(protocol::TimingPayload { rules }));
    }
    log("Linting Complete".to_string());
    (sink, Ok(()))
}

/// Every program gets several checkers, and node and symbol ids come from process-wide counters, so each checker's
/// id-keyed link pages are mostly empty. Sparse pages hold the same links in less memory (the monolith, 6 checkers: -4%
/// peak footprint for about +1.5% lint-phase instructions; tsrs notes/mem-shared-base.md). Must run before
/// checkers exist.
fn use_sparse_id_pages() {
    tsrs_checker::links::set_sparse_id_pages(true);
}

fn suppress_program_diagnostics() -> bool {
    std::env::var("OXLINT_TSGOLINT_DANGEROUSLY_SUPPRESS_PROGRAM_DIAGNOSTICS")
        .is_ok_and(|v| v == "true")
}

struct ProgramExtCache(&'static ExtendedConfigCache);

/// Declaration and JSON files parsed for one program, by parse options.
type DeclarationCache = Mutex<FxHashMap<SourceFileParseOptions, P<SourceFile>>>;

/// Programs are built one after another and most of what they parse is shared: lib files and `node_modules`
/// declarations. Like `tsc --build`'s host (execute/build/host.go), every program gets declaration and JSON files
/// from one cache, so a later program (the inferred one, or another tsconfig) reuses the parsed and bound files of an
/// earlier one instead of parsing them again. A parse depends only on its `SourceFileParseOptions` and the file
/// text, binding only on the file, and checkers never write parser or binder data (several checkers of one program
/// already share it).
fn shared_declarations_host(
    inner: Arc<dyn CompilerHost>,
    cache: &'static DeclarationCache,
) -> Arc<dyn CompilerHost> {
    Arc::new(SharedDeclarationsHost { inner, cache })
}

struct SharedDeclarationsHost {
    inner: Arc<dyn CompilerHost>,
    cache: &'static DeclarationCache,
}

impl CompilerHost for SharedDeclarationsHost {
    fn fs(&self) -> &dyn FS {
        self.inner.fs()
    }
    fn default_library_path(&self) -> &str {
        self.inner.default_library_path()
    }
    fn get_current_directory(&self) -> &str {
        self.inner.get_current_directory()
    }
    fn trace(&self, msg: &'static tsrs_diagnostics::Message, args: &[&dyn std::fmt::Display]) {
        self.inner.trace(msg, args)
    }
    fn get_source_file(&self, opts: SourceFileParseOptions) -> Option<P<SourceFile>> {
        if !tspath::is_declaration_file_name(&opts.file_name)
            && !tspath::file_extension_is(&opts.file_name, tspath::EXTENSION_JSON)
        {
            return self.inner.get_source_file(opts);
        }
        if let Some(&file) = self.cache.lock().unwrap().get(&opts) {
            return Some(file);
        }
        // Parse outside the lock: the files of one program are parsed in parallel.
        let file = self.inner.get_source_file(opts.clone())?;
        Some(*self.cache.lock().unwrap().entry(opts).or_insert(file))
    }
    fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: Path,
    ) -> Option<P<tsrs_tsoptions::ParsedCommandLine>> {
        self.inner.get_resolved_project_reference(file_name, path)
    }
}

impl tsrs_tsoptions::ExtendedConfigCache for ProgramExtCache {
    fn get_extended_config(
        &self,
        file_name: &str,
        path: &Path,
        resolution_stack: &[Path],
        host: &'static dyn tsrs_tsoptions::ParseConfigHost,
    ) -> P<tsrs_tsoptions::ExtendedConfigCacheEntry> {
        self.0.get_extended_config(file_name, path, resolution_stack, host)
    }
}

/// create_program.go CreateInferredProjectProgram's options.
fn inferred_compiler_options() -> CompilerOptions {
    use tsrs_core::{JsxEmit, ModuleKind, ModuleResolutionKind, ScriptTarget};
    CompilerOptions {
        allow_js: Tristate::True,
        module: ModuleKind::ESNext,
        module_resolution: ModuleResolutionKind::Bundler,
        target: ScriptTarget::ES2022,
        jsx: JsxEmit::ReactJSX,
        allow_importing_ts_extensions: Tristate::True,
        strict_null_checks: Tristate::True,
        strict_function_types: Tristate::True,
        source_map: Tristate::True,
        es_module_interop: Tristate::True,
        allow_non_ts_extensions: Tristate::True,
        resolve_json_module: Tristate::True,
        checkers: crate::sched::checkers(),
        ..Default::default()
    }
}

fn diagnostic_text(d: P<tsrs_ast::Diagnostic>) -> String {
    tsrs_compiler::diagnosticwriter::flatten_diagnostic_message(d, "\n")
}

fn send_internal_tsconfig_diag(
    tx: &mpsc::SyncSender<protocol::Diagnostic>,
    d: P<tsrs_ast::Diagnostic>,
    config_file_name: &str,
) {
    let file_path = d
        .file()
        .map(|f| f.file_name().to_string())
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| config_file_name.to_string());
    let loc = d.loc();
    let mut help = diagnostic_text(d);
    if help.contains("Please remove it from your configuration.") {
        help.push_str(
            "\nSee https://github.com/oxc-project/tsgolint/issues/351 for more information.",
        );
    }
    let _ = tx.send(protocol::Diagnostic {
        kind: 1,
        range: Some(protocol::Range { pos: loc.pos(), end: loc.end() }),
        message: protocol::Message {
            id: "tsconfig-error".to_string(),
            description: "Invalid tsconfig".to_string(),
            help: Some(help),
        },
        file_path: Some(file_path),
        labeled_ranges: Vec::new(),
        rule: None,
        fixes: Vec::new(),
        suggestions: Vec::new(),
    });
}

struct LintCtx<'a> {
    opts: &'a Options,
    rule_sets: &'a [RuleSet],
    file_rules: &'a FxHashMap<Path, usize>,
    tx: &'a mpsc::SyncSender<protocol::Diagnostic>,
    timings: &'a Mutex<FxHashMap<&'static str, (Duration, u64)>>,
    use_case_sensitive: bool,
    cwd: &'a str,
}

fn report_typescript_diagnostics(
    program: &'static Program,
    files: &[P<SourceFile>],
    payload: &Payload,
    tx: &mpsc::SyncSender<protocol::Diagnostic>,
) {
    if !payload.report_syntactic && !payload.report_semantic {
        return;
    }
    let ctx = tsrs_compiler::Context::default();
    for &file in files {
        let mut diags = Vec::new();
        if payload.report_syntactic {
            diags.extend(program.get_syntactic_diagnostics(&ctx, Some(file)));
        }
        if payload.report_semantic {
            diags.extend(program.get_semantic_diagnostics(&ctx, Some(file)));
        }
        for d in diags {
            if d.file().map(|f| f.file_name() == file.file_name()) != Some(true) {
                continue;
            }
            let loc = d.loc();
            let _ = tx.send(protocol::Diagnostic {
                kind: 1,
                range: Some(protocol::Range { pos: loc.pos(), end: loc.end() }),
                message: protocol::Message {
                    id: format!("TS{}", d.code()),
                    description: diagnostic_text(d),
                    help: None,
                },
                file_path: Some(file.file_name().to_string()),
                labeled_ranges: Vec::new(),
                rule: None,
                fixes: Vec::new(),
                suggestions: Vec::new(),
            });
        }
    }
}

fn lint_program(
    program: &'static Program,
    files: &[P<SourceFile>],
    payload: &Payload,
    lctx: &LintCtx,
) {
    report_typescript_diagnostics(program, files, payload, lctx.tx);
    // Each file goes to the checker tsrs assigned it (import-graph locality) until that checker runs out, then to
    // whichever checker is free (sched.rs): with a shared queue every checker resolves the types of nearly every
    // module.
    let todo: Vec<usize> =
        (0..files.len()).filter(|&i| lctx.file_rules.contains_key(files[i].path())).collect();
    crate::sched::for_each_file(
        program,
        files,
        &todo,
        || (Worker::new(lctx.opts.debug_timings), Vec::new()),
        |(worker, out): &mut (Worker, Vec<ReportedDiagnostic>), checker, i| {
            let file = files[i];
            let set = &lctx.rule_sets[lctx.file_rules[file.path()]];
            worker.lint_file(checker, program, file, &set.rules, lctx.opts, out);
            for d in out.drain(..) {
                let _ = lctx.tx.send(d.to_protocol());
            }
        },
        |(worker, _)| {
            if lctx.opts.debug_timings {
                let mut t = lctx.timings.lock().unwrap();
                #[expect(
                    clippy::iter_over_hash_type,
                    reason = "adding to per-rule sums gives the same totals in any order, and the timing output is sorted before it is written"
                )]
                for (name, (d, c)) in worker.timings {
                    let e = t.entry(name).or_default();
                    e.0 += d;
                    e.1 += c;
                }
            }
        },
    );
    let _ = (lctx.use_case_sensitive, lctx.cwd);
}

/// Lints `files` of `program` with `rules` and returns the diagnostics (used by the rule tests; the
/// headless path streams instead).
pub fn lint_files(
    program: &'static Program,
    files: &[P<SourceFile>],
    rules: &[&'static dyn Rule],
    opts: &Options,
) -> Vec<ReportedDiagnostic> {
    use_sparse_id_pages();
    let all: Mutex<Vec<ReportedDiagnostic>> = Mutex::new(Vec::new());
    let todo: Vec<usize> = (0..files.len()).collect();
    crate::sched::for_each_file(
        program,
        files,
        &todo,
        || (Worker::new(false), Vec::new()),
        |(worker, out): &mut (Worker, Vec<ReportedDiagnostic>), checker, i| {
            worker.lint_file(checker, program, files[i], rules, opts, out);
        },
        |(_, out)| all.lock().unwrap().extend(out),
    );
    all.into_inner().unwrap()
}

/// Go linter.ApplyRuleFixes: applies non-overlapping fixes in position order.
pub fn apply_rule_fixes(
    code: &str,
    mut fix_sets: Vec<Vec<crate::rule::RuleFix>>,
) -> (String, bool) {
    fix_sets.retain(|f| !f.is_empty());
    for f in &mut fix_sets {
        f.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.end.cmp(&b.end)));
    }
    fix_sets
        .sort_by(|a, b| a[0].pos.cmp(&b[0].pos).then(a[a.len() - 1].end.cmp(&b[b.len() - 1].end)));
    let mut out = String::with_capacity(code.len());
    let mut last = 0usize;
    let mut fixed = false;
    for fixes in &fix_sets {
        if last > fixes[0].pos as usize {
            continue;
        }
        for f in fixes {
            fixed = true;
            out.push_str(&code[last..f.pos as usize]);
            out.push_str(&f.text);
            last = f.end as usize;
        }
    }
    out.push_str(&code[last..]);
    (out, fixed)
}

/// Per-worker scratch state: listener dispatch table reused across files.
struct Worker {
    dispatch: Vec<Vec<u16>>,
    touched: Vec<usize>,
    timed: bool,
    timings: FxHashMap<&'static str, (Duration, u64)>,
}

impl Worker {
    fn new(timed: bool) -> Worker {
        Worker {
            dispatch: vec![Vec::new(); 6 * KIND_SLOTS],
            touched: Vec::new(),
            timed,
            timings: FxHashMap::default(),
        }
    }

    fn lint_file(
        &mut self,
        checker: &mut Checker,
        program: &'static Program,
        file: P<SourceFile>,
        rules: &[&'static dyn Rule],
        opts: &Options,
        out: &mut Vec<ReportedDiagnostic>,
    ) {
        crate::utils::clear_type_strings();
        let mut visitors: Vec<Box<dyn RuleVisitor>> = Vec::with_capacity(rules.len());
        for (idx, rule) in rules.iter().enumerate() {
            let mut ctx = Ctx {
                checker: &mut *checker,
                program,
                file,
                fix: opts.fix,
                fix_suggestions: opts.fix_suggestions,
                rule_name: rule.name(),
                out: &mut *out,
            };
            let start = self.timed.then(Instant::now);
            let v = rule.create_visitor(&mut ctx);
            if let Some(s) = start {
                let e = self.timings.entry(rule.name()).or_default();
                e.0 += s.elapsed();
                e.1 += 1;
            }
            for l in v.listeners() {
                let slot = l.slot();
                if self.dispatch[slot].is_empty() {
                    self.touched.push(slot);
                }
                self.dispatch[slot].push(idx as u16);
            }
            visitors.push(v);
        }
        let mut walker = Walker {
            worker: self,
            checker,
            program,
            file,
            rules,
            visitors: &mut visitors,
            opts,
            out,
        };
        walker.walk_file();
        for slot in self.touched.drain(..) {
            self.dispatch[slot].clear();
        }
    }
}

struct Walker<'a> {
    worker: &'a mut Worker,
    checker: &'a mut Checker,
    program: &'static Program,
    file: P<SourceFile>,
    rules: &'a [&'static dyn Rule],
    visitors: &'a mut Vec<Box<dyn RuleVisitor>>,
    opts: &'a Options,
    out: &'a mut Vec<ReportedDiagnostic>,
}

impl Walker<'_> {
    #[inline]
    fn run(&mut self, listener: Listener, node: P<Node>) {
        let slot = listener.slot();
        let n = self.worker.dispatch[slot].len();
        for j in 0..n {
            let idx = self.worker.dispatch[slot][j] as usize;
            let rule = self.rules[idx];
            let mut ctx = Ctx {
                checker: &mut *self.checker,
                program: self.program,
                file: self.file,
                fix: self.opts.fix,
                fix_suggestions: self.opts.fix_suggestions,
                rule_name: rule.name(),
                out: &mut *self.out,
            };
            if self.worker.timed {
                let s = Instant::now();
                self.visitors[idx].visit(&mut ctx, listener, node);
                let e = self.worker.timings.entry(rule.name()).or_default();
                e.0 += s.elapsed();
                e.1 += 1;
            } else {
                self.visitors[idx].visit(&mut ctx, listener, node);
            }
        }
    }

    /// linter.go visitLintNodes.
    fn walk_file(&mut self) {
        let root = self.file.as_node();
        root.for_each_child(&mut |c| self.child_visitor(c));
    }

    fn pattern_visitor(&mut self, node: P<Node>) {
        let kind = node.kind();
        self.run(Listener::Enter(kind), node);
        self.run(Listener::AllowPattern(kind), node);
        match kind {
            Kind::ArrayLiteralExpression => {
                for &e in node.elements() {
                    self.pattern_visitor(e);
                }
            }
            Kind::ObjectLiteralExpression => {
                for &p in node.properties() {
                    self.pattern_visitor(p);
                }
            }
            Kind::SpreadElement | Kind::SpreadAssignment => {
                if let Some(e) = node.expression() {
                    self.pattern_visitor(e);
                }
            }
            Kind::PropertyAssignment => {
                if let Some(i) = node.initializer() {
                    self.pattern_visitor(i);
                }
            }
            _ => {
                node.for_each_child(&mut |c| self.child_visitor(c));
            }
        }
        self.run(Listener::AllowPatternExit(kind), node);
        self.run(Listener::Exit(kind), node);
    }

    fn child_visitor(&mut self, node: P<Node>) -> bool {
        let kind = node.kind();
        self.run(Listener::Enter(kind), node);
        match kind {
            Kind::ArrayLiteralExpression | Kind::ObjectLiteralExpression => {
                self.run(Listener::NotAllowPattern(kind), node);
                node.for_each_child(&mut |c| self.child_visitor(c));
                self.run(Listener::NotAllowPatternExit(kind), node);
            }
            _ => {
                if ast::is_assignment_expression(node, true) {
                    let e = node.as_binary_expression();
                    self.pattern_visitor(e.left);
                    self.child_visitor(e.operator_token);
                    self.child_visitor(e.right.get());
                } else {
                    node.for_each_child(&mut |c| self.child_visitor(c));
                }
            }
        }
        self.run(Listener::Exit(kind), node);
        false
    }
}
