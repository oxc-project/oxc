//! Tests for the styled-components plugin.
//!
//! Regression tests for <https://github.com/oxc-project/oxc/issues/30335>:
//! the generated `componentId` must not depend on the absolute path the project
//! is built from, nor on whether the file path is passed as an absolute or
//! relative path.

use std::{
    fs,
    path::{Path, PathBuf},
};

use oxc_span::SourceType;
use oxc_transformer::{PluginsOptions, StyledComponentsOptions, TransformOptions};

use crate::test_with_source_type_and_path;

const SOURCE: &str =
    "import styled from 'styled-components';\nconst Bar = styled.header`color: red;`;\n";

fn transform(source_path: &Path) -> String {
    let options = TransformOptions {
        plugins: PluginsOptions {
            styled_components: Some(StyledComponentsOptions::default()),
            ..PluginsOptions::default()
        },
        ..TransformOptions::default()
    };
    test_with_source_type_and_path(SOURCE, SourceType::default(), source_path, &options)
        .expect("transform should succeed")
}

/// Creates a fake project `<root>/package.json` with `src/components/Header.jsx` layout,
/// and returns the path of the styled-components file.
///
/// Only `package.json` needs to exist on disk for the tests; the source file path is
/// only used for hashing.
fn create_project(root: &Path) -> PathBuf {
    fs::create_dir_all(root.join("src/components")).unwrap();
    fs::write(root.join("package.json"), r#"{ "name": "ssg-sample" }"#).unwrap();
    root.join("src/components/Header.jsx")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("oxc-styled-components-{}-{}", std::process::id(), name));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The same project checked out at different absolute paths must produce the
/// same `componentId`s.
#[test]
fn component_id_is_independent_of_absolute_path() {
    let temp = unique_temp_dir("absolute");
    let path_a = create_project(&temp.join("proj-a"));
    let path_b = create_project(&temp.join("proj-b"));
    let path_c = create_project(&temp.join("deeper/x/y/proj-a"));

    let output_a = transform(&path_a);
    let output_b = transform(&path_b);
    let output_c = transform(&path_c);

    assert_eq!(output_a, output_b, "different parent directories must not change the output");
    assert_eq!(output_a, output_c, "a deeper checkout path must not change the output");
    assert!(
        output_a.contains("componentId: 'sc-"),
        "componentId should be generated, got:\n{output_a}"
    );

    fs::remove_dir_all(temp).unwrap();
}

/// Different files within the same project must still get different `componentId`s,
/// even if their source code is identical.
#[test]
fn component_id_is_unique_per_file() {
    let temp = unique_temp_dir("unique");
    let header_path = create_project(&temp.join("proj-a"));
    let footer_path = temp.join("proj-a/src/components/Footer.jsx");

    let header_output = transform(&header_path);
    let footer_output = transform(&footer_path);

    assert_ne!(
        header_output, footer_output,
        "different files in the same project must get different componentIds"
    );

    fs::remove_dir_all(temp).unwrap();
}

/// The `componentId` must not depend on whether the file path is passed as an
/// absolute or a relative path.
#[test]
fn component_id_is_independent_of_relative_and_absolute_path() {
    let relative = Path::new("src/components/Header.jsx");
    let absolute = std::env::current_dir().unwrap().join(relative);

    let relative_output = transform(relative);
    let absolute_output = transform(&absolute);

    assert_eq!(
        relative_output, absolute_output,
        "relative and absolute forms of the same path must produce the same output"
    );
}
