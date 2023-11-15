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
    project_root().join("tasks/coverage/typescript/tests/cases/conformance")
}

fn snap_root() -> PathBuf {
    project_root().join("tasks/transform_conformance")
}

fn fixture_root() -> PathBuf {
    snap_root().join("ts_fixutures")
}

const CASES: &[&str] = &["enums"];

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
        for case in CASES {
            for path in Self::glob_files(&root().join(case), self.options.filter.as_ref()) {
                let content = match Self::transform(&path) {
                    Ok(content) => content,
                    Err(err) => err.iter().map(ToString::to_string).collect(),
                };
                Self::write_result_file(&content, &path);
            }
        }
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

    fn write_result_file(content: &str, path: &Path) {
        let new_file_name = normalize_path(path.strip_prefix(&root()).unwrap())
            .split('/')
            .collect::<Vec<&str>>()
            .join("__");

        let target_path = fixture_root().join(new_file_name);
        fs::write(target_path, content).unwrap();
    }
}
