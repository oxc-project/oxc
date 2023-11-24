use std::{
    fs::{self},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::Error;
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
                    Err(err) => (err.iter().map(ToString::to_string).collect(), "error"),
                };
                snapshot.push_str(lang);
                snapshot.push('\n');
                snapshot.push_str(&content);
                snapshot.push_str("\n```\n\n");
            }
        }

        fs::write(snap_root().join(CONFORMANCE_SNAPSHOT), snapshot).unwrap();
    }
}

impl TypeScriptFixtures {
    fn transform_options() -> TransformOptions {
        // TODO: read options from slash directives
        TransformOptions::default()
    }

    fn glob_files(root: &Path, filter: Option<&String>) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .map(walkdir::DirEntry::into_path)
            .filter(|p| p.is_file())
            .filter(|p| filter_ext(p.as_path()))
            .filter(|p| filter.map_or(true, |f| p.to_string_lossy().contains(f)))
            .collect()
    }

    fn transform(path: &Path) -> Result<String, Vec<Error>> {
        let allocator = Allocator::default();
        let source_text = fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();
        let ret = Parser::new(&allocator, &source_text, source_type).parse();

        if ret.program.is_empty() && !ret.errors.is_empty() {
            return Err(ret.errors);
        }

        let semantic = SemanticBuilder::new(&source_text, source_type)
            .with_trivias(ret.trivias)
            .build(&ret.program)
            .semantic;
        let transformed_program = allocator.alloc(ret.program);

        let result = Transformer::new(&allocator, source_type, semantic, Self::transform_options())
            .build(transformed_program);

        result.map(|()| {
            Codegen::<false>::new(source_text.len(), CodegenOptions).build(transformed_program)
        })
    }
}
