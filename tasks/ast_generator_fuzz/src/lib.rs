//! Fuzz pipeline for generated ASTs.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use indicatif::{ProgressBar, ProgressStyle};
use rand::{SeedableRng, rngs::StdRng};

use oxc_allocator::Allocator;
use oxc_ast_generator::AstGenerator;
use oxc_codegen::Codegen;
use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource};
use oxc_parser::Parser;
use oxc_span::SourceType;

const SOURCE_TYPES: [SourceType; 5] = [
    SourceType::mjs(),
    SourceType::script(),
    SourceType::cjs(),
    SourceType::ts().with_module(true),
    SourceType::ts().with_script(true),
];

/// Generate an AST, print it, and reparse it.
///
/// Writes parser failures to a new `case-NNNN` directory and continues.
#[expect(clippy::print_stderr)]
pub fn run_once(seed: u64, source_type: SourceType) {
    if let Some(message) =
        run_case(Path::new("."), next_case_number(Path::new(".")), seed, source_type)
    {
        eprintln!("{message}");
    }
}

fn run_case(root: &Path, case_number: u64, seed: u64, source_type: SourceType) -> Option<String> {
    let ast_allocator = Allocator::default();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut generator = AstGenerator::new(&ast_allocator, &mut rng, source_type);
    let program = generator.generate();

    let source_text = Codegen::new().build(&program).code;
    check_parser_output(
        root,
        case_number,
        seed,
        source_type,
        &format!("{program:#?}"),
        &source_text,
    )
}

fn check_parser_output(
    root: &Path,
    case_number: u64,
    seed: u64,
    source_type: SourceType,
    ast: &str,
    source_text: &str,
) -> Option<String> {
    let parser_allocator = Allocator::default();
    let parsed = Parser::new(&parser_allocator, source_text, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        let raw_error =
            format!("panicked: {}\ndiagnostics: {:#?}", parsed.panicked, parsed.diagnostics);
        let graphical_error = render_graphical_diagnostics(&parsed.diagnostics, source_text);
        let case_path = write_failure_case(
            root,
            case_number,
            seed,
            source_type,
            ast,
            source_text,
            &raw_error,
            &graphical_error,
        );
        return Some(match case_path {
            Ok(case_path) => format!(
                "parser failed for {source_type:?} seed {seed}; artifacts written to {}",
                case_path.display()
            ),
            Err(error) => format!(
                "parser failed for {source_type:?} seed {seed}; failed to write artifacts: {error}"
            ),
        });
    }
    None
}

fn render_graphical_diagnostics(
    diagnostics: &oxc_diagnostics::Diagnostics,
    source_text: &str,
) -> String {
    let handler = GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
    let source = Arc::new(NamedSource::new("input.codegen", source_text.to_string()));
    let mut output = String::new();
    for diagnostic in diagnostics {
        let diagnostic = diagnostic.clone().with_source_code(Arc::clone(&source));
        handler.render_report(&mut output, diagnostic.as_ref()).unwrap();
    }
    output
}

/// Run a range of seeds across multiple worker threads for all supported source profiles.
///
/// # Panics
///
/// Panics when `threads` is zero.
pub fn run_range(seed: u64, iterations: u64, threads: usize) {
    run_range_impl(Path::new("."), seed, iterations, threads, None);
}

/// Run a range of seeds with a terminal progress bar.
///
/// # Panics
///
/// Panics when `threads` is zero.
pub fn run_range_with_progress(seed: u64, iterations: u64, threads: usize) {
    let profile_count = u64::try_from(SOURCE_TYPES.len()).unwrap();
    let progress = ProgressBar::new(iterations.saturating_mul(profile_count));
    progress.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ETA {eta_precise}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    run_range_impl(Path::new("."), seed, iterations, threads, Some(&progress));
    progress.finish();
}

fn run_range_impl(
    root: &Path,
    seed: u64,
    iterations: u64,
    threads: usize,
    progress: Option<&ProgressBar>,
) {
    assert!(threads > 0, "thread count must be greater than zero");
    let stride = u64::try_from(threads).unwrap();
    let profile_count = u64::try_from(SOURCE_TYPES.len()).unwrap();
    let first_case_number = next_case_number(root);

    thread::scope(|scope| {
        for worker in 0..threads {
            let progress = progress.cloned();
            scope.spawn(move || {
                let mut offset = u64::try_from(worker).unwrap();
                while offset < iterations {
                    let current_seed = seed.wrapping_add(offset);
                    let case_number =
                        first_case_number.saturating_add(offset.saturating_mul(profile_count));
                    for (profile_index, source_type) in SOURCE_TYPES.into_iter().enumerate() {
                        let failure = run_case(
                            root,
                            case_number.saturating_add(u64::try_from(profile_index).unwrap()),
                            current_seed,
                            source_type,
                        );
                        if let Some(message) = failure {
                            report_failure(&message, progress.as_ref());
                        }
                        if let Some(progress) = &progress {
                            progress.inc(1);
                        }
                    }
                    let Some(next) = offset.checked_add(stride) else { break };
                    offset = next;
                }
            });
        }
    });
}

#[expect(clippy::print_stderr)]
fn report_failure(message: &str, progress: Option<&ProgressBar>) {
    if let Some(progress) = progress.filter(|progress| !progress.is_hidden()) {
        progress.println(message);
    } else {
        eprintln!("{message}");
    }
}

fn next_case_number(root: &Path) -> u64 {
    let mut next = 0;
    let Ok(entries) = fs::read_dir(root) else { return next };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else { continue };
        let Some(number) = file_name.strip_prefix("case-") else { continue };
        let Ok(number) = number.parse::<u64>() else { continue };
        next = next.max(number.saturating_add(1));
    }
    next
}

fn write_failure_case(
    root: &Path,
    case_number: u64,
    seed: u64,
    source_type: SourceType,
    ast: &str,
    codegen: &str,
    raw_error: &str,
    graphical_error: &str,
) -> io::Result<PathBuf> {
    let case_path = root.join(format!("case-{case_number:04}"));
    fs::create_dir(&case_path)?;
    let report = format!(
        "seed: {seed}\nsource type: {source_type:?}\n\n\
         AST Input:\n```text\n{ast}\n```\n\n\
         code-gened input:\n```text\n{codegen}\n```\n\n\
         parser error (raw):\n```text\n{raw_error}\n```\n\n\
         parser error (graphical diagnostic reporter):\n```text\n{graphical_error}\n```\n"
    );
    fs::write(case_path.join("case.md"), report)?;
    Ok(case_path)
}

/// Default fuzz worker count: one third of available CPU cores, with a minimum of one.
pub fn default_thread_count() -> usize {
    thread::available_parallelism().map_or(1, |cores| (cores.get() / 3).max(1))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oxc_ast::ast_kind::AstKind;
    use oxc_ast_visit::Visit;

    use super::*;

    #[derive(Default)]
    struct KindCollector(BTreeSet<&'static str>);

    impl<'a> Visit<'a> for KindCollector {
        fn enter_node(&mut self, kind: AstKind<'a>) {
            let name = match kind {
                AstKind::AccessorProperty(_) => "AccessorProperty",
                AstKind::ArrayExpression(_) => "ArrayExpression",
                AstKind::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
                AstKind::AssignmentExpression(_) => "AssignmentExpression",
                AstKind::AwaitExpression(_) => "AwaitExpression",
                AstKind::BigIntLiteral(_) => "BigIntLiteral",
                AstKind::BinaryExpression(_) => "BinaryExpression",
                AstKind::BlockStatement(_) => "BlockStatement",
                AstKind::BreakStatement(_) => "BreakStatement",
                AstKind::CallExpression(_) => "CallExpression",
                AstKind::CatchClause(_) => "CatchClause",
                AstKind::ChainExpression(_) => "ChainExpression",
                AstKind::Class(_) => "Class",
                AstKind::ComputedMemberExpression(_) => "ComputedMemberExpression",
                AstKind::ConditionalExpression(_) => "ConditionalExpression",
                AstKind::ContinueStatement(_) => "ContinueStatement",
                AstKind::DebuggerStatement(_) => "DebuggerStatement",
                AstKind::Decorator(_) => "Decorator",
                AstKind::DoWhileStatement(_) => "DoWhileStatement",
                AstKind::EmptyStatement(_) => "EmptyStatement",
                AstKind::ExportAllDeclaration(_) => "ExportAllDeclaration",
                AstKind::ExportDeclaration(_) => "ExportDeclaration",
                AstKind::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
                AstKind::ExportFromDeclaration(_) => "ExportFromDeclaration",
                AstKind::ExportNamedDeclaration(_) => "ExportNamedDeclaration",
                AstKind::ExpressionStatement(_) => "ExpressionStatement",
                AstKind::ForInStatement(_) => "ForInStatement",
                AstKind::ForOfStatement(_) => "ForOfStatement",
                AstKind::ForStatement(_) => "ForStatement",
                AstKind::FormalParameter(_) => "FormalParameter",
                AstKind::FormalParameters(_) => "FormalParameters",
                AstKind::Function(_) => "Function",
                AstKind::IfStatement(_) => "IfStatement",
                AstKind::ImportDeclaration(_) => "ImportDeclaration",
                AstKind::ImportExpression(_) => "ImportExpression",
                AstKind::ImportMeta(_) => "ImportMeta",
                AstKind::LabeledStatement(_) => "LabeledStatement",
                AstKind::LogicalExpression(_) => "LogicalExpression",
                AstKind::MethodDefinition(_) => "MethodDefinition",
                AstKind::NewExpression(_) => "NewExpression",
                AstKind::NewTarget(_) => "NewTarget",
                AstKind::ObjectExpression(_) => "ObjectExpression",
                AstKind::ParenthesizedExpression(_) => "ParenthesizedExpression",
                AstKind::PropertyDefinition(_) => "PropertyDefinition",
                AstKind::PrivateFieldExpression(_) => "PrivateFieldExpression",
                AstKind::PrivateIdentifier(_) => "PrivateIdentifier",
                AstKind::PrivateInExpression(_) => "PrivateInExpression",
                AstKind::RegExpLiteral(_) => "RegExpLiteral",
                AstKind::ReturnStatement(_) => "ReturnStatement",
                AstKind::SequenceExpression(_) => "SequenceExpression",
                AstKind::StaticBlock(_) => "StaticBlock",
                AstKind::StaticMemberExpression(_) => "StaticMemberExpression",
                AstKind::StringLiteral(_) => "StringLiteral",
                AstKind::Super(_) => "Super",
                AstKind::SwitchStatement(_) => "SwitchStatement",
                AstKind::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
                AstKind::TemplateLiteral(_) => "TemplateLiteral",
                AstKind::ThisExpression(_) => "ThisExpression",
                AstKind::ThrowStatement(_) => "ThrowStatement",
                AstKind::TryStatement(_) => "TryStatement",
                AstKind::TSAsExpression(_) => "TSAsExpression",
                AstKind::TSConditionalType(_) => "TSConditionalType",
                AstKind::TSEnumDeclaration(_) => "TSEnumDeclaration",
                AstKind::TSGlobalDeclaration(_) => "TSGlobalDeclaration",
                AstKind::TSExportAssignment(_) => "TSExportAssignment",
                AstKind::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
                AstKind::TSImportType(_) => "TSImportType",
                AstKind::TSIndexedAccessType(_) => "TSIndexedAccessType",
                AstKind::TSInstantiationExpression(_) => "TSInstantiationExpression",
                AstKind::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
                AstKind::TSIntersectionType(_) => "TSIntersectionType",
                AstKind::TSLiteralType(_) => "TSLiteralType",
                AstKind::TSMappedType(_) => "TSMappedType",
                AstKind::TSExternalModuleDeclaration(_) => "TSExternalModuleDeclaration",
                AstKind::TSNamespaceDeclaration(_) => "TSNamespaceDeclaration",
                AstKind::TSNonNullExpression(_) => "TSNonNullExpression",
                AstKind::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration",
                AstKind::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
                AstKind::TSTupleType(_) => "TSTupleType",
                AstKind::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
                AstKind::TSTypeAssertion(_) => "TSTypeAssertion",
                AstKind::TSTypeLiteral(_) => "TSTypeLiteral",
                AstKind::TSTypeOperator(_) => "TSTypeOperator",
                AstKind::TSTypeQuery(_) => "TSTypeQuery",
                AstKind::TSTypeReference(_) => "TSTypeReference",
                AstKind::TSUnionType(_) => "TSUnionType",
                AstKind::UnaryExpression(_) => "UnaryExpression",
                AstKind::UpdateExpression(_) => "UpdateExpression",
                AstKind::VariableDeclaration(_) => "VariableDeclaration",
                AstKind::WhileStatement(_) => "WhileStatement",
                AstKind::WithStatement(_) => "WithStatement",
                AstKind::YieldExpression(_) => "YieldExpression",
                _ => return,
            };
            self.0.insert(name);
        }
    }

    const EXPECTED_REACHABLE_KINDS: &[&str] = &[
        "AccessorProperty",
        "ArrayExpression",
        "ArrowFunctionExpression",
        "AssignmentExpression",
        "AwaitExpression",
        "BigIntLiteral",
        "BinaryExpression",
        "BlockStatement",
        "BreakStatement",
        "CallExpression",
        "CatchClause",
        "ChainExpression",
        "Class",
        "ComputedMemberExpression",
        "ConditionalExpression",
        "ContinueStatement",
        "DebuggerStatement",
        "Decorator",
        "DoWhileStatement",
        "EmptyStatement",
        "ExportAllDeclaration",
        "ExportDeclaration",
        "ExportDefaultDeclaration",
        "ExportFromDeclaration",
        "ExportNamedDeclaration",
        "ExpressionStatement",
        "ForInStatement",
        "ForOfStatement",
        "ForStatement",
        "FormalParameter",
        "FormalParameters",
        "Function",
        "IfStatement",
        "ImportDeclaration",
        "ImportExpression",
        "ImportMeta",
        "LabeledStatement",
        "LogicalExpression",
        "MethodDefinition",
        "NewExpression",
        "NewTarget",
        "ObjectExpression",
        "ParenthesizedExpression",
        "PropertyDefinition",
        "PrivateFieldExpression",
        "PrivateIdentifier",
        "PrivateInExpression",
        "RegExpLiteral",
        "ReturnStatement",
        "SequenceExpression",
        "StaticBlock",
        "StaticMemberExpression",
        "StringLiteral",
        "Super",
        "SwitchStatement",
        "TaggedTemplateExpression",
        "TemplateLiteral",
        "ThisExpression",
        "ThrowStatement",
        "TryStatement",
        "TSAsExpression",
        "TSConditionalType",
        "TSEnumDeclaration",
        "TSExportAssignment",
        "TSGlobalDeclaration",
        "TSImportEqualsDeclaration",
        "TSImportType",
        "TSIndexedAccessType",
        "TSInstantiationExpression",
        "TSInterfaceDeclaration",
        "TSIntersectionType",
        "TSLiteralType",
        "TSMappedType",
        "TSExternalModuleDeclaration",
        "TSNamespaceDeclaration",
        "TSNonNullExpression",
        "TSNamespaceExportDeclaration",
        "TSSatisfiesExpression",
        "TSTupleType",
        "TSTypeAliasDeclaration",
        "TSTypeAssertion",
        "TSTypeLiteral",
        "TSTypeOperator",
        "TSTypeQuery",
        "TSTypeReference",
        "TSUnionType",
        "UnaryExpression",
        "UpdateExpression",
        "VariableDeclaration",
        "WhileStatement",
        "WithStatement",
        "YieldExpression",
    ];

    #[test]
    fn supported_grammar_is_reachable() {
        let mut collector = KindCollector::default();

        for seed in 0..20_000 {
            for source_type in SOURCE_TYPES {
                let ast_allocator = Allocator::default();
                let mut rng = StdRng::seed_from_u64(seed);
                let mut generator = AstGenerator::new(&ast_allocator, &mut rng, source_type);
                let program = generator.generate::<oxc_ast::ast::Program<'_>>();

                let default_exports = program
                    .body
                    .iter()
                    .filter(|statement| {
                        matches!(statement, oxc_ast::ast::Statement::ExportDefaultDeclaration(_))
                    })
                    .count();
                assert!(default_exports <= 1, "seed {seed} generated multiple default exports");

                collector.visit_program(&program);
            }
        }

        let missing = EXPECTED_REACHABLE_KINDS
            .iter()
            .copied()
            .filter(|kind| !collector.0.contains(kind))
            .collect::<Vec<_>>();
        assert!(missing.is_empty(), "unreachable AST kinds: {missing:?}");
    }

    #[test]
    fn generated_range_completes() {
        let root = std::env::temp_dir()
            .join(format!("oxc-ast-generator-fuzz-range-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();

        run_range_impl(&root, 0, 1_000, 2, None);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parser_failure_writes_artifacts_without_panicking() {
        let root = std::env::temp_dir()
            .join(format!("oxc-ast-generator-fuzz-parser-failure-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();

        let failure =
            check_parser_output(&root, 0, 42, SourceType::mjs(), "synthetic AST", "const = ;");

        assert!(failure.is_some());
        assert!(root.join("case-0000/case.md").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn default_threads_is_never_zero() {
        assert!(default_thread_count() > 0);
    }

    #[test]
    fn writes_failure_case_files() {
        let root =
            std::env::temp_dir().join(format!("oxc-ast-generator-fuzz-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();

        let case_path = write_failure_case(
            &root,
            7,
            42,
            SourceType::ts().with_module(true),
            "ast",
            "code",
            "raw error",
            "graphical error",
        )
        .unwrap();

        assert_eq!(case_path.file_name().unwrap(), "case-0007");
        let report = fs::read_to_string(case_path.join("case.md")).unwrap();
        assert!(report.contains("seed: 42"));
        assert!(report.contains("source type:"));
        assert!(report.contains("AST Input:\n```text\nast\n```"));
        assert!(report.contains("code-gened input:\n```text\ncode\n```"));
        assert!(report.contains("parser error (raw):\n```text\nraw error\n```"));
        assert!(report.contains(
            "parser error (graphical diagnostic reporter):\n```text\ngraphical error\n```"
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
