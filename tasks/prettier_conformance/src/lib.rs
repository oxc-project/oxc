use std::{
    fs,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry, WalkDir};

use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind, Program,
    },
    VisitMut,
};
use oxc_parser::Parser;
use oxc_prettier::{Prettier, PrettierOptions, TrailingComma};
use oxc_span::{Atom, SourceType};
use oxc_tasks_common::project_root;

// #[test]
// #[cfg(any(coverage, coverage_nightly))]
// fn test() {
// TestRunner::new(TestRunnerOptions::default()).run();
// }

#[derive(Default)]
pub struct TestRunnerOptions {
    pub filter: Option<String>,
}

#[derive(Default)]
pub struct SpecParser {
    source_text: String,
    calls: Vec<(PrettierOptions, Vec<(Atom, String)>)>,
}

impl SpecParser {
    pub fn parse(&mut self, spec: &Path) {
        let spec_content = fs::read_to_string(spec).unwrap_or_default();
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(spec).unwrap_or_default();
        let mut ret = Parser::new(&allocator, &spec_content, source_type).parse();
        self.source_text = spec_content.clone();
        self.visit_program(&mut ret.program);
    }
}

/// The test runner which walks the prettier repository and searches for formatting tests.
pub struct TestRunner {
    options: TestRunnerOptions,
    spec: SpecParser,
}

fn root() -> PathBuf {
    project_root().join("tasks/prettier_conformance")
}

fn fixtures_root() -> PathBuf {
    project_root().join(root()).join("prettier/tests/format/js")
}

impl VisitMut<'_> for SpecParser {
    fn visit_program(&mut self, program: &mut Program<'_>) {
        self.visit_statements(&mut program.body);
    }

    fn visit_call_expression(&mut self, expr: &mut CallExpression<'_>) {
        let Some(ident) = expr.callee.get_identifier_reference() else { return };
        if ident.name != "run_spec" {
            return;
        }
        let mut parsers = vec![];
        let mut snapshot_options = vec![];
        let mut options = PrettierOptions::default();

        if let Some(argument) = expr.arguments.get(1) {
            if let Argument::Expression(Expression::ArrayExpression(arr_expr)) = argument {
                parsers = arr_expr
                    .elements
                    .iter()
                    .filter_map(|el| {
                        if let ArrayExpressionElement::Expression(Expression::StringLiteral(
                            literal,
                        )) = el
                        {
                            return Some(literal.value.to_string());
                        }
                        None
                    })
                    .collect::<Vec<String>>();
            }
        } else {
            return;
        }

        if let Some(Argument::Expression(Expression::ObjectExpression(obj_expr))) =
            expr.arguments.get(2)
        {
            obj_expr.properties.iter().for_each(|item| {
                if let ObjectPropertyKind::ObjectProperty(obj_prop) = item {
                    if let Some(name) = obj_prop.key.static_name() {
                        match &obj_prop.value {
                            Expression::BooleanLiteral(literal) => {
                                if name == "semi" {
                                    options.semi = literal.value;
                                }
                                snapshot_options.push((name, literal.value.to_string()));
                            }
                            Expression::NumberLiteral(literal) => {
                                if name == "printWidth" {
                                    options.print_width =
                                        str::parse(&literal.value.to_string()).unwrap_or(80);
                                }
                                snapshot_options.push((name, literal.value.to_string()));
                            }
                            Expression::StringLiteral(literal) => {
                                if name == "trailingComma" {
                                    options.trailing_comma = match literal.value.as_str() {
                                        "none" => TrailingComma::None,
                                        "es5" => TrailingComma::ES5,
                                        _ => TrailingComma::All,
                                    }
                                }
                                snapshot_options.push((name, format!("\"{}\"", literal.value)));
                            }
                            _ => {}
                        };
                    };
                }
            });
        }

        snapshot_options.push((
            "parsers".into(),
            format!(
                "[{}]",
                parsers.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ")
            ),
        ));
        if !snapshot_options.iter().any(|item| item.0 == "printWidth") {
            snapshot_options.push(("printWidth".into(), "80".into()));
        }

        snapshot_options.sort_by(|a, b| a.0.cmp(&b.0));

        self.calls.push((options, snapshot_options));
    }
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        Self { options, spec: SpecParser::default() }
    }

    /// # Panics
    #[allow(clippy::cast_precision_loss)]
    pub fn run(mut self) {
        let fixture_root = fixtures_root();
        // Read the first level of directories that contain `__snapshots__`
        let mut dirs = WalkDir::new(&fixture_root)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                self.options
                    .filter
                    .as_ref()
                    .map_or(true, |name| e.path().to_string_lossy().contains(name))
            })
            .filter(|e| e.file_type().is_dir() && e.path().join("__snapshots__").exists())
            .map(DirEntry::into_path)
            .collect::<Vec<_>>();

        dirs.sort_unstable();

        let mut failed = vec![];

        for dir in &dirs {
            // Get jsfmt.spec.js and all the other input files
            let (specs, mut inputs): (Vec<PathBuf>, Vec<PathBuf>) = WalkDir::new(dir)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
                .map(|e| e.path().to_path_buf())
                .partition(|path| path.file_name().is_some_and(|name| name == "jsfmt.spec.js"));

            self.spec.parse(&specs[0]);

            inputs.sort_unstable();
            if !self.test_snapshot(&specs[0], &inputs) {
                failed.push(format!(
                    "* {}",
                    dir.strip_prefix(&fixture_root).unwrap().to_string_lossy()
                ));
            }
        }

        let total = dirs.len();
        let passed = total - failed.len();
        let percentage = (passed as f64 / total as f64) * 100.0;
        let heading = format!("Compatibility: {passed}/{total} ({percentage:.2}%)");
        println!("{heading}");

        if self.options.filter.is_none() {
            let failed = failed.join("\n");
            let snapshot = format!("{heading}\n\n# Failed\n\n{failed}");
            fs::write(root().join("prettier.snap.md"), snapshot).unwrap();
        }
    }

    fn test_snapshot(&self, spec_path: &Path, inputs: &[PathBuf]) -> bool {
        return self.spec.calls.iter().any(|spec| {
            let inputs = inputs
                .iter()
                .map(|path| {
                    let input = fs::read_to_string(path).unwrap();
                    self.get_single_snapshot(path, &input, spec.0, spec.1.clone())
                })
                .collect::<Vec<_>>()
                .join("\n");

            let snapshot = format!("// Jest Snapshot v1, https://goo.gl/fbAQLP\n{inputs}\n");

            let expected_file =
                spec_path.parent().unwrap().join("__snapshots__/jsfmt.spec.js.snap");
            let expected = fs::read_to_string(expected_file).unwrap();

            snapshot.contains(&expected)
        });
    }

    fn get_single_snapshot(
        &self,
        path: &Path,
        input: &str,
        prettier_options: PrettierOptions,
        snapshot_options: Vec<(Atom, String)>,
    ) -> String {
        let filename = path.file_name().unwrap().to_string_lossy();
        let output = Self::prettier(path, input);
        let snapshot_options = snapshot_options
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        if self.options.filter.is_some() {
            println!("Input:");
            println!("{input}");
            println!("Output:");
            println!("{output}");
        }

        let space_line = " ".repeat(prettier_options.print_width);
        let full_equal_sign = "=".repeat(prettier_options.print_width);

        let get_text_line = |text: &'static str| {
            let equal_half_length = (prettier_options.print_width - text.len()) / 2;
            let sign = "=".repeat(equal_half_length);
            let string = format!("{sign}{text}{sign}");

            if (prettier_options.print_width - string.len()) == 1 {
                format!("{string}=")
            } else {
                string
            }
        };

        let options_line = get_text_line("options");
        let input_line = get_text_line("input");
        let output_line = get_text_line("output");

        format!(
            r#"
exports[`{filename} format 1`] = `
{options_line}
{snapshot_options}
{space_line}| printWidth
{input_line}
{input}
{output_line}
{output}
{full_equal_sign}
`;"#
        )
    }

    fn prettier(path: &Path, source_text: &str) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        Prettier::new(&allocator, source_text, ret.trivias, PrettierOptions::default())
            .build(&ret.program)
    }
}
