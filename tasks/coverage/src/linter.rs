use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::Test262Case,
    typescript::TypeScriptCase,
};
use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{AllowWarnDeny, Fixer, LintContext, LintOptions, Linter, Message};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

/// # What this test does:
///
/// 1. First round of parsing + semantic analysis.
///     - Aborts if parser panics.
/// 2. Runs the linter with autofix on and all plugins and rules enabled.
/// 3. Applies fixes to the source code.
/// 4. Second round of parsing + semantic analysis, this time on the fixed code.
///     - Aborts if parser panics.
/// 5. Compares the number of semantic errors before and after fixing lint errors.
///     - If there are more errors after fixing lint errors, the test fails.
fn get_result(
    case_name: &str,
    file_name: &Path,
    source_text: &str,
    source_type: SourceType,
) -> TestResult {
    let allocator = Allocator::default();
    let pathbuf = file_name.to_path_buf();
    let boxed_path = pathbuf.clone().into_boxed_path();

    let do_write_failure = |parser_panicked_before: bool,
                            semantic_errors_before: Vec<OxcDiagnostic>,
                            fixes: Vec<Message>,
                            source_after_fixes: &str,
                            parser_panicked_after: bool,
                            semantic_errors_after: Vec<OxcDiagnostic>| {
        write_failure(
            case_name,
            file_name,
            source_text,
            source_type,
            parser_panicked_before,
            semantic_errors_before,
            fixes,
            source_after_fixes,
            parser_panicked_after,
            semantic_errors_after,
        )
        .expect("Failed to write linter conformance failure info");
    };

    // 1. First round of parsing + semantic analysis.

    let ParserReturn { program, trivias, panicked, errors } =
        Parser::new(&allocator, source_text, source_type).parse();

    if panicked {
        let error_string =
            errors.first().map_or_else(|| "(No error diagnostic)".to_string(), ToString::to_string);
        let msg = format!("Parser panicked on first pass: {error_string}");
        do_write_failure(true, vec![], vec![], "(Not fixed)", false, vec![]);
        return TestResult::ParseError(msg, panicked);
    }

    let program = allocator.alloc(program);

    let SemanticBuilderReturn { semantic, errors: semantic_errors } =
        SemanticBuilder::new(source_text, source_type)
            .with_trivias(trivias)
            .with_check_syntax_error(true)
            .build_module_record(pathbuf.clone(), program)
            .build(program);

    // 2. Run the linter with autofix on and all plugins and rules enabled.

    let options = LintOptions::default()
        .with_fix(true)
        .with_filter(vec![(AllowWarnDeny::Warn, String::from("all"))])
        .with_react_plugin(true)
        .with_unicorn_plugin(true)
        .with_typescript_plugin(true)
        .with_oxc_plugin(true)
        .with_import_plugin(true)
        .with_jsdoc_plugin(true)
        .with_jest_plugin(true)
        .with_jsx_a11y_plugin(true)
        .with_nextjs_plugin(true)
        .with_react_perf_plugin(true);
    let linter = Linter::from_options(options).unwrap();
    let semantic = Rc::new(semantic);
    let ctx = LintContext::new(boxed_path, semantic);
    let messages = linter.run(ctx.clone());

    // 3. Apply fixes to the source code.
    let fixes = Fixer::new(source_text, messages).fix();

    // 4. Second round of parsing + semantic analysis, this time on the fixed
    //    code.

    let ParserReturn { program: fixed_program, trivias, panicked, errors } =
        Parser::new(&allocator, &fixes.fixed_code, source_type).parse();

    if !errors.is_empty() {
        let error_string =
            errors.first().map_or_else(|| "(No error diagnostic)".to_string(), ToString::to_string);
        let msg = format!("Parser panicked on second pass: {error_string}");
        do_write_failure(
            false,
            semantic_errors,
            fixes.messages,
            &fixes.fixed_code,
            panicked,
            vec![],
        );
        return TestResult::ParseError(msg, panicked);
    }

    let program = allocator.alloc(fixed_program);
    let SemanticBuilderReturn { errors: fixed_semantic_errors, .. } =
        SemanticBuilder::new(&fixes.fixed_code, source_type)
            .with_trivias(trivias)
            .with_check_syntax_error(true)
            .build_module_record(pathbuf, program)
            .build(program);

    // 5. Compare the number of semantic errors before and after fixing lint

    let semantic_error_count = semantic_errors.len();
    let fixed_semantic_error_count = fixed_semantic_errors.len();
    #[allow(clippy::cast_possible_wrap)]
    let additional_errors = (fixed_semantic_error_count as isize) - (semantic_error_count as isize);

    if additional_errors > 0 {
        do_write_failure(
            false,
            semantic_errors,
            fixes.messages,
            &fixes.fixed_code,
            panicked,
            fixed_semantic_errors,
        );
        return TestResult::RuntimeError(format!(
            "{additional_errors} more semantic errors occurred after fixing lint errors than before ({fixed_semantic_error_count} vs {semantic_error_count})"
        ));
    }

    // If we reach this point, the test passed.
    TestResult::Passed
}

#[allow(clippy::too_many_arguments)]
fn write_failure(
    case_name: &str,
    file_name: &Path,
    source_text: &str,
    source_type: SourceType,
    parser_panicked_before: bool,
    semantic_errors_before: Vec<OxcDiagnostic>,
    fixes: Vec<Message>,
    source_after_fixes: &str,
    parser_panicked_after: bool,
    semantic_errors_after: Vec<OxcDiagnostic>,
) -> io::Result<()> {
    let dir = Path::new("./tasks/coverage/failures/linter").join(case_name).join(file_name);
    fs::create_dir_all(&dir).expect("Error creating failure directory for linter coverage");

    let ext = match (source_type.is_jsx(), source_type.is_javascript()) {
        (true, true) => ".jsx",
        (false, true) => ".js",
        (true, false) => ".tsx",
        (false, false) => ".ts",
    };

    fs::write(dir.join(format!("original{ext}")), source_text)
        .expect("Error writing original source file");
    fs::write(dir.join(format!("fixed_source{ext}")), source_after_fixes)
        .expect("Error writing fixed source file");

    let file =
        fs::File::create(dir.join("failure_info.txt")).expect("Error creating failure info file");
    let mut w = io::BufWriter::new(file);

    macro_rules! print_header_sep {
        () => {
            writeln!(w, "{}", "=".repeat(80))
        };
    }
    macro_rules! print_section_sep {
        () => {
            writeln!(w, "\n{}", "-".repeat(80))
        };
    }
    macro_rules! print_list {
        ($list:expr) => {
            if $list.is_empty() {
                writeln!(w, "None")?;
            } else {
                for el in $list {
                    writeln!(w, "{:#?}", el)?;
                }
            }
        };
    }

    // print_header_sep(&mut w)?;
    print_header_sep!()?;
    writeln!(w, "file name: {}", file_name.display())?;
    writeln!(w, "source type:\n{source_type:#?}\n")?;
    writeln!(w, "fixes applied: {}", fixes.len())?;
    writeln!(w, "parser panicked before: {parser_panicked_before}")?;
    writeln!(w, "semantic errors before: {}", semantic_errors_before.len())?;
    writeln!(w, "parser panicked after: {parser_panicked_after}")?;
    writeln!(w, "semantic errors after: {}", semantic_errors_after.len())?;
    print_header_sep!()?;
    writeln!(w)?;

    writeln!(w, "fixes applied:")?;
    print_list!(fixes);

    print_section_sep!()?;
    writeln!(w, "semantic errors before:")?;
    print_list!(semantic_errors_before);

    print_section_sep!()?;
    writeln!(w, "semantic errors after:")?;
    print_list!(semantic_errors_after);

    w.flush()
}

// TODO: use concat_idents! macro when it's stable
macro_rules! impl_case {
    ($Ty:ident, $Inner:ident, $name:tt) => {
        impl Case for $Ty {
            fn new(path: PathBuf, code: String) -> Self {
                Self { base: $Inner::new(path, code) }
            }

            fn code(&self) -> &str {
                self.base.code()
            }

            fn path(&self) -> &Path {
                self.base.path()
            }

            fn test_result(&self) -> &TestResult {
                self.base.test_result()
            }

            fn skip_test_case(&self) -> bool {
                self.base.skip_test_case() || self.base.should_fail()
            }

            fn run(&mut self) {
                let source_text = self.base.code();
                let source_type = self.base.source_type();
                let result = get_result($name, self.base.path(), source_text, source_type);
                self.base.set_result(result);
            }
        }
    };
}

pub struct LinterTest262Case {
    base: Test262Case,
}

impl_case!(LinterTest262Case, Test262Case, "test262");

pub struct LinterBabelCase {
    base: BabelCase,
}
impl_case!(LinterBabelCase, BabelCase, "babel");

pub struct LinterTypeScriptCase {
    base: TypeScriptCase,
}
impl_case!(LinterTypeScriptCase, TypeScriptCase, "typescript");

pub struct LinterMiscCase {
    base: MiscCase,
}
impl_case!(LinterMiscCase, MiscCase, "misc");
