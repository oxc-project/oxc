use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::{miette::NamedSource, GraphicalReportHandler, GraphicalTheme};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::{normalize_path, project_root};
use oxc_transform_conformance::TestRunnerOptions;
use oxc_transformer::{TransformOptions, Transformer};

fn root() -> PathBuf {
    project_root().join("tasks/coverage")
}

fn snap_root() -> PathBuf {
    project_root().join("tasks/transform_conformance")
}

const CASES: &[&str] = &[
    "typescript/tests/cases/conformance/enums",
    "babel/packages/babel-plugin-transform-typescript/test/fixtures/enum",
];

const CONFORMANCE_SNAPSHOT: &str = "typescript.snap.md";

fn filter_ext(p: &Path) -> bool {
    p.to_string_lossy().ends_with(".ts")
}

pub struct TypeScriptFixtures {
    options: TestRunnerOptions,
}

impl TypeScriptFixtures {
    pub fn new(options: TestRunnerOptions) -> Self {
        Self { options }
    }

    pub fn run(self) {
        let mut snapshot = String::new();

        for case in CASES {
            for path in Self::glob_files(&root().join(case), self.options.filter.as_ref()) {
                snapshot.push_str("# ");
                snapshot.push_str(&normalize_path(path.strip_prefix(&root()).unwrap()));
                snapshot.push('\n');
                snapshot.push_str("```");

                let (content, lang) = match Self::transform(&path) {
                    Ok(content) => (content, "typescript"),
                    Err(err) => (err, "error"),
                };
                snapshot.push_str(lang);
                snapshot.push('\n');
                snapshot.push_str(&content);
                snapshot.push_str("\n```\n\n");
            }
        }

        if self.options.filter.is_none() {
            fs::write(snap_root().join(CONFORMANCE_SNAPSHOT), snapshot).unwrap();
        }
    }
}

impl TypeScriptFixtures {
    fn transform_options() -> TransformOptions {
        // TODO: read options from slash directives
        TransformOptions::default()
    }

    fn glob_files(root: &Path, filter: Option<&String>) -> Vec<PathBuf> {
        let mut list: Vec<PathBuf> = WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .map(walkdir::DirEntry::into_path)
            .filter(|p| p.is_file())
            .filter(|p| filter_ext(p.as_path()))
            .filter(|p| filter.map_or(true, |f| p.to_string_lossy().contains(f)))
            .collect();

        list.sort_unstable();

        list
    }

    fn transform(path: &Path) -> Result<String, String> {
        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();
        let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();

        let semantic_ret = SemanticBuilder::new(&source_text, source_type)
            .with_trivias(parser_ret.trivias)
            .with_check_syntax_error(true)
            .build(&parser_ret.program);

        let errors = parser_ret.errors.into_iter().chain(semantic_ret.errors).collect::<Vec<_>>();

        if !errors.is_empty() {
            let handler =
                GraphicalReportHandler::new().with_theme(GraphicalTheme::unicode_nocolor());
            let mut output = String::new();
            for error in errors {
                let error = error.with_source_code(NamedSource::new(
                    &normalize_path(path.strip_prefix(&root()).unwrap()),
                    source_text.to_string(),
                ));
                handler.render_report(&mut output, error.as_ref()).unwrap();
                output.push('\n');
            }
            return Err(output);
        }

        let semantic = semantic_ret.semantic;
        let transformed_program = allocator.alloc(parser_ret.program);

        let result = Transformer::new(&allocator, source_type, semantic, Self::transform_options())
            .build(transformed_program);

        result
            .map(|()| {
                Codegen::<false>::new(source_text.len(), CodegenOptions).build(transformed_program)
            })
            .map_err(|e| e.iter().map(ToString::to_string).collect())
    }
}
