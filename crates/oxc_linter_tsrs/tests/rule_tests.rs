// Port of internal/rule_tester/rule_tester.go: runs tsgolint's own rule test cases (extracted to
// tests/cases/<rule>.json by tools/extract_rule_tests) against the Rust rules.
//
// Each case gets a fresh program over tests/fixtures (tsgolint's fixture directory, kept in this
// crate) with the case's code overlaid on file.ts / react.tsx. Parsed files are cached across programs
// (like tsrs's conformance runner and Go's harness) so the lib files are parsed once per thread.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::sync::Arc;

use oxc_linter_tsrs::linter::{Options, apply_rule_fixes, lint_files};
use oxc_linter_tsrs::overlayfs::OverlayFS;
use oxc_linter_tsrs::rule::{ReportedDiagnostic, Rule};
use oxc_linter_tsrs::tsconfig::ParseHost;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use tsrs_ast::{SourceFile, SourceFileParseOptions};
use tsrs_compiler::{CompilerHost, ProgramOptions, new_compiler_host, new_program};
use tsrs_core::tspath::{self, Path};
use tsrs_core::{CompilerOptions, P, ScriptKind, Tristate};
use tsrs_tsoptions::{self as tsoptions, ParsedCommandLine};
use tsrs_vfs::{FS, bundled, cachedvfs, osvfs};

#[derive(Deserialize)]
struct Cases {
    groups: Vec<Group>,
    unconverted: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct Group {
    tsconfig: String,
    valid: Vec<Case>,
    invalid: Vec<Case>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Case {
    code: String,
    #[serde(default)]
    only: bool,
    #[serde(default)]
    skip: bool,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    options: Option<serde_json::Value>,
    #[serde(default)]
    tsconfig: String,
    #[serde(default)]
    tsx: bool,
    #[serde(default)]
    files: FxHashMap<String, String>,
    #[serde(default)]
    output: Vec<String>,
    #[serde(default)]
    errors: Vec<CaseError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaseError {
    message_id: String,
    #[serde(default)]
    line: i32,
    #[serde(default)]
    column: i32,
    #[serde(default)]
    end_line: i32,
    #[serde(default)]
    end_column: i32,
    #[serde(default)]
    suggestions: Vec<CaseSuggestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaseSuggestion {
    message_id: String,
    output: String,
}

/// Parsed files by (file name, external-module `jsx`, external-module `force`, text).
type SourceFileCache = FxHashMap<(String, bool, bool, String), P<SourceFile>>;

thread_local! {
    static SOURCE_FILE_CACHE: RefCell<SourceFileCache> = RefCell::new(FxHashMap::default());
}

struct CachedCompilerHost {
    inner: Arc<dyn CompilerHost>,
}

impl CompilerHost for CachedCompilerHost {
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
        let text = self.fs().read_file(&opts.file_name)?;
        let script_kind = tsrs_core::get_script_kind_from_file_name(&opts.file_name);
        if script_kind == ScriptKind::Unknown {
            return None;
        }
        let emi = opts.external_module_indicator_options;
        let key = (opts.file_name.clone(), emi.jsx, emi.force, text.clone());
        if let Some(cached) = SOURCE_FILE_CACHE.with(|c| c.borrow().get(&key).copied()) {
            return Some(cached);
        }
        let sf = tsrs_parser::parse_source_file(opts, &text, script_kind);
        SOURCE_FILE_CACHE.with(|c| c.borrow_mut().insert(key, sf));
        Some(sf)
    }
    fn get_resolved_project_reference(
        &self,
        file_name: &str,
        path: Path,
    ) -> Option<P<ParsedCommandLine>> {
        self.inner.get_resolved_project_reference(file_name, path)
    }
}

fn fixtures_root() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    tspath::normalize_path(&std::fs::canonicalize(p).unwrap().to_string_lossy())
}

fn base_fs() -> Arc<dyn FS> {
    thread_local! {
        static BASE: Arc<dyn FS> = Arc::new(cachedvfs::from(Arc::new(bundled::wrap_fs(osvfs::fs()))));
    }
    BASE.with(|b| b.clone())
}

fn run_linter(
    root: &str,
    tsconfig: &str,
    rule: &'static dyn Rule,
    code: &str,
    file_name: &str,
    tsx: bool,
    extra: &FxHashMap<String, String>,
) -> Result<(Vec<ReportedDiagnostic>, P<SourceFile>), String> {
    let file_name = if !file_name.is_empty() {
        file_name.to_string()
    } else if tsx {
        "react.tsx".to_string()
    } else {
        "file.ts".to_string()
    };
    let resolved = tspath::resolve_path(root, &[&file_name]);
    let mut virtual_files: FxHashMap<String, String> = FxHashMap::default();
    virtual_files.insert(resolved.clone(), code.to_string());
    for (k, v) in extra {
        virtual_files.insert(tspath::resolve_path(root, &[k]), v.clone());
    }
    let fs: Arc<dyn FS> = Arc::new(OverlayFS::new(base_fs(), virtual_files));
    let host: &'static ParseHost =
        Box::leak(Box::new(ParseHost { fs: fs.clone(), cwd: root.to_string() }));
    let (parsed, errors) = tsoptions::get_parsed_command_line_of_config_file(
        tsconfig,
        Some(&CompilerOptions::default()),
        None,
        host,
        None,
    );
    if !errors.is_empty() {
        return Err(format!("tsconfig errors: {}", errors.len()));
    }
    let config = P::new(parsed.ok_or("no config")?);
    let inner = new_compiler_host(root, fs, &bundled::lib_path(), None, None);
    let compiler_host: Arc<dyn CompilerHost> = Arc::new(CachedCompilerHost { inner });
    let mut popts = ProgramOptions::new(config, compiler_host);
    popts.use_source_of_project_reference = true;
    popts.single_threaded = Tristate::True;
    let program = new_program(popts);
    program.bind_source_files();
    let path = tspath::to_path(&resolved, root, host.fs.use_case_sensitive_file_names());
    let sf = program
        .source_files()
        .iter()
        .copied()
        .find(|sf| *sf.path() == path)
        .ok_or_else(|| format!("{resolved} not in program"))?;
    let opts = Options { fix: true, fix_suggestions: true, debug_timings: false };
    Ok((lint_files(program, &[sf], &[rule], &opts), sf))
}

fn line_col(file: P<SourceFile>, pos: i32) -> (i32, i32) {
    let (line, col) = tsrs_scanner::get_ecma_line_and_utf16_character_of_position(&*file, pos);
    (line + 1, col + 1)
}

/// Invalid cases (rule, group, index) that import Node builtins and need `@types/node` resolvable from
/// `tests/fixtures`, which this workspace does not install.
const NEEDS_TYPES_NODE: &[(&str, usize, usize)] =
    &[("no-deprecated", 0, 56), ("no-deprecated", 0, 125)];

/// Runs every converted case of `rule`; returns (passed, failed descriptions, skipped).
fn run_cases(rule_name: &str) -> (usize, Vec<String>, usize, usize) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/cases/{rule_name}.json"));
    let cases: Cases = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let root = fixtures_root();
    let mut passed = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();
    let only_mode = cases.groups.iter().any(|g| g.valid.iter().chain(&g.invalid).any(|c| c.only));
    let mut rule_cache: FxHashMap<String, &'static dyn Rule> = FxHashMap::default();
    let mut get_rule = |options: &Option<serde_json::Value>| -> Result<&'static dyn Rule, String> {
        let key = options.as_ref().map(|o| o.to_string()).unwrap_or_default();
        if let Some(r) = rule_cache.get(&key) {
            return Ok(*r);
        }
        let r: &'static dyn Rule =
            Box::leak(oxc_linter_tsrs::rules::create_rule(rule_name, options.as_ref())?);
        rule_cache.insert(key, r);
        Ok(r)
    };
    for (gi, g) in cases.groups.iter().enumerate() {
        for (i, c) in g.valid.iter().enumerate() {
            if c.skip || (only_mode && !c.only) {
                skipped += 1;
                continue;
            }
            let rule = match get_rule(&c.options) {
                Ok(r) => r,
                Err(e) => {
                    skipped += 1;
                    eprintln!("[{rule_name}] valid-{gi}-{i}: unsupported options: {e}");
                    continue;
                }
            };
            let tsconfig = if c.tsconfig.is_empty() { &g.tsconfig } else { &c.tsconfig };
            match run_linter(&root, tsconfig, rule, &c.code, &c.file_name, c.tsx, &c.files) {
                Ok((diags, _)) if diags.is_empty() => passed += 1,
                Ok((diags, _)) => failures.push(format!(
                    "valid-{gi}-{i}: expected no errors, got {:?}\n{}",
                    diags.iter().map(|d| d.message.id).collect::<Vec<_>>(),
                    c.code
                )),
                Err(e) => failures.push(format!("valid-{gi}-{i}: {e}")),
            }
        }
        for (i, c) in g.invalid.iter().enumerate() {
            if c.skip || NEEDS_TYPES_NODE.contains(&(rule_name, gi, i)) || (only_mode && !c.only) {
                skipped += 1;
                continue;
            }
            let rule = match get_rule(&c.options) {
                Ok(r) => r,
                Err(e) => {
                    skipped += 1;
                    eprintln!("[{rule_name}] invalid-{gi}-{i}: unsupported options: {e}");
                    continue;
                }
            };
            let tsconfig = if c.tsconfig.is_empty() { &g.tsconfig } else { &c.tsconfig };
            let mut problems = String::new();
            let first =
                match run_linter(&root, tsconfig, rule, &c.code, &c.file_name, c.tsx, &c.files) {
                    Ok(r) => r,
                    Err(e) => {
                        failures.push(format!("invalid-{gi}-{i}: {e}"));
                        continue;
                    }
                };
            // Fix loop (up to 10 passes).
            let mut outputs = Vec::new();
            let mut code = c.code.clone();
            let mut diags = first.0.clone();
            for pass in 0..10 {
                if pass > 0 {
                    diags = match run_linter(
                        &root,
                        tsconfig,
                        rule,
                        &code,
                        &c.file_name,
                        c.tsx,
                        &c.files,
                    ) {
                        Ok(r) => r.0,
                        Err(e) => {
                            let _ = write!(problems, "fix pass {pass}: {e}; ");
                            break;
                        }
                    };
                }
                let (fixed_code, fixed) = apply_rule_fixes(
                    &code,
                    diags.iter().map(|d| d.fixes.clone().unwrap_or_default()).collect(),
                );
                if !fixed {
                    break;
                }
                code = fixed_code;
                outputs.push(code.clone());
            }
            if outputs != c.output {
                let _ = write!(
                    problems,
                    "outputs differ: expected {:?}, got {:?}; ",
                    c.output, outputs
                );
            }
            let (initial, file) = first;
            if initial.len() != c.errors.len() {
                let _ = write!(
                    problems,
                    "expected {} errors, got {} {:?}; ",
                    c.errors.len(),
                    initial.len(),
                    initial.iter().map(|d| d.message.id).collect::<Vec<_>>()
                );
            } else {
                for (j, (e, d)) in c.errors.iter().zip(&initial).enumerate() {
                    if e.message_id != d.message.id {
                        let _ = write!(
                            problems,
                            "error {j}: message id {} != expected {}; ",
                            d.message.id, e.message_id
                        );
                    }
                    let (line, col) = line_col(file, d.pos);
                    let (end_line, end_col) = line_col(file, d.end);
                    for (name, want, got) in [
                        ("line", e.line, line),
                        ("column", e.column, col),
                        ("endLine", e.end_line, end_line),
                        ("endColumn", e.end_column, end_col),
                    ] {
                        if want != 0 && want != got {
                            let _ =
                                write!(problems, "error {j}: {name} {got} != expected {want}; ");
                        }
                    }
                    let sugg = d.suggestions.clone().unwrap_or_default();
                    if sugg.len() != e.suggestions.len() {
                        let _ = write!(
                            problems,
                            "error {j}: {} suggestions != expected {}; ",
                            sugg.len(),
                            e.suggestions.len()
                        );
                    } else {
                        for (k, (es, s)) in e.suggestions.iter().zip(&sugg).enumerate() {
                            if es.message_id != s.message.id {
                                let _ = write!(
                                    problems,
                                    "error {j} suggestion {k}: id {} != {}; ",
                                    s.message.id, es.message_id
                                );
                            } else {
                                let (out, _) = apply_rule_fixes(&c.code, vec![s.fixes.clone()]);
                                if out != es.output {
                                    let _ = write!(
                                        problems,
                                        "error {j} suggestion {k}: output {out:?} != expected {:?}; ",
                                        es.output
                                    );
                                }
                            }
                        }
                    }
                }
            }
            if problems.is_empty() {
                passed += 1;
            } else {
                failures.push(format!("invalid-{gi}-{i}: {problems}\n{}", c.code));
            }
        }
    }
    (passed, failures, skipped, cases.unconverted.len())
}

fn check(rule_name: &str) {
    if let Err(e) = oxc_linter_tsrs::rules::create_rule(rule_name, None) {
        if e.starts_with(oxc_linter_tsrs::rules::NOT_PORTED_PREFIX) {
            eprintln!("[{rule_name}] skipped: not ported yet");
            return;
        }
    }
    let (passed, failures, skipped, unconverted) = run_cases(rule_name);
    eprintln!(
        "[{rule_name}] passed {passed}, failed {}, skipped {skipped}, unconverted (not extracted) {unconverted}",
        failures.len()
    );
    for f in &failures {
        eprintln!("[{rule_name}] FAIL {f}\n");
    }
    assert!(failures.is_empty(), "{rule_name}: {} failing cases", failures.len());
}

macro_rules! rule_test {
    ($fn_name:ident, $rule:literal) => {
        #[test]
        fn $fn_name() {
            // Type checking recurses deeply.
            std::thread::Builder::new()
                .stack_size(256 << 20)
                .spawn(|| check($rule))
                .unwrap()
                .join()
                .unwrap();
        }
    };
}

rule_test!(await_thenable, "await-thenable");
rule_test!(consistent_return, "consistent-return");
rule_test!(consistent_type_exports, "consistent-type-exports");
rule_test!(dot_notation, "dot-notation");
rule_test!(no_array_delete, "no-array-delete");
rule_test!(no_base_to_string, "no-base-to-string");
rule_test!(no_confusing_void_expression, "no-confusing-void-expression");
rule_test!(no_deprecated, "no-deprecated");
rule_test!(no_duplicate_type_constituents, "no-duplicate-type-constituents");
rule_test!(no_floating_promises, "no-floating-promises");
rule_test!(no_generated_empty_object_type, "no-generated-empty-object-type");
rule_test!(no_for_in_array, "no-for-in-array");
rule_test!(no_implied_eval, "no-implied-eval");
rule_test!(no_meaningless_void_operator, "no-meaningless-void-operator");
rule_test!(no_misused_promises, "no-misused-promises");
rule_test!(no_misused_spread, "no-misused-spread");
rule_test!(no_mixed_enums, "no-mixed-enums");
rule_test!(no_redundant_type_constituents, "no-redundant-type-constituents");
rule_test!(no_unnecessary_boolean_literal_compare, "no-unnecessary-boolean-literal-compare");
rule_test!(no_unnecessary_condition, "no-unnecessary-condition");
rule_test!(no_unnecessary_qualifier, "no-unnecessary-qualifier");
rule_test!(no_unnecessary_template_expression, "no-unnecessary-template-expression");
rule_test!(no_unnecessary_type_arguments, "no-unnecessary-type-arguments");
rule_test!(no_unnecessary_type_assertion, "no-unnecessary-type-assertion");
rule_test!(no_unnecessary_type_conversion, "no-unnecessary-type-conversion");
rule_test!(no_unnecessary_type_parameters, "no-unnecessary-type-parameters");
rule_test!(no_unsafe_argument, "no-unsafe-argument");
rule_test!(no_unsafe_assignment, "no-unsafe-assignment");
rule_test!(no_unsafe_call, "no-unsafe-call");
rule_test!(no_unsafe_enum_comparison, "no-unsafe-enum-comparison");
rule_test!(no_unsafe_member_access, "no-unsafe-member-access");
rule_test!(no_unsafe_return, "no-unsafe-return");
rule_test!(no_unsafe_type_assertion, "no-unsafe-type-assertion");
rule_test!(no_unsafe_unary_minus, "no-unsafe-unary-minus");
rule_test!(no_useless_default_assignment, "no-useless-default-assignment");
rule_test!(non_nullable_type_assertion_style, "non-nullable-type-assertion-style");
rule_test!(only_throw_error, "only-throw-error");
rule_test!(prefer_find, "prefer-find");
rule_test!(prefer_includes, "prefer-includes");
rule_test!(prefer_nullish_coalescing, "prefer-nullish-coalescing");
rule_test!(prefer_optional_chain, "prefer-optional-chain");
rule_test!(prefer_promise_reject_errors, "prefer-promise-reject-errors");
rule_test!(prefer_readonly, "prefer-readonly");
rule_test!(prefer_readonly_parameter_types, "prefer-readonly-parameter-types");
rule_test!(prefer_reduce_type_parameter, "prefer-reduce-type-parameter");
rule_test!(prefer_regexp_exec, "prefer-regexp-exec");
rule_test!(prefer_return_this_type, "prefer-return-this-type");
rule_test!(prefer_string_starts_ends_with, "prefer-string-starts-ends-with");
rule_test!(promise_function_async, "promise-function-async");
rule_test!(related_getter_setter_pairs, "related-getter-setter-pairs");
rule_test!(require_array_sort_compare, "require-array-sort-compare");
rule_test!(require_await, "require-await");
rule_test!(restrict_plus_operands, "restrict-plus-operands");
rule_test!(restrict_template_expressions, "restrict-template-expressions");
rule_test!(return_await, "return-await");
rule_test!(strict_boolean_expressions, "strict-boolean-expressions");
rule_test!(strict_void_return, "strict-void-return");
rule_test!(switch_exhaustiveness_check, "switch-exhaustiveness-check");
rule_test!(unbound_method, "unbound-method");
rule_test!(use_unknown_in_catch_callback_variable, "use-unknown-in-catch-callback-variable");
