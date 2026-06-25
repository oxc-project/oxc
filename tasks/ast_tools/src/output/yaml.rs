use std::{fmt::Write, fs, path::Path};

use itertools::{Itertools, chain};

use crate::{AST_CHANGES_WATCH_LIST_PATH, RawOutput};

use super::{Output, add_header};

/// Add header to YAML.
pub fn print_yaml(code: &str, generator_path: &str) -> String {
    add_header(code, generator_path, "#")
}

impl Output {
    /// Generate CI watch list YAML file.
    ///
    /// This is used in `ast_changes` CI job to skip running `oxc_ast_tools`
    /// unless relevant files have changed.
    ///
    /// The watch list includes:
    /// * Glob patterns for watched crates (`{crate}/src/**/*.rs`)
    /// * Generated output file paths (excluding those already covered by crate globs)
    /// * `ast_tools` crate itself
    /// * CI workflow file
    /// * Config files (`Cargo.toml`, `Cargo.lock`, `package.json`, `oxfmtrc.jsonc`)
    ///
    /// `crate_paths` are crates containing AST types (discovered via `cargo metadata`).
    /// `oxc_*` dependency crates of `ast_tools` are also added, since changes to them
    /// may affect generated code.
    pub fn yaml_watch_list(
        outputs: &[RawOutput],
        mut crate_paths: Vec<String>,
        root_path: &Path,
    ) -> RawOutput {
        // Add trailing slashes to crate paths, so can use them for prefix matching below
        for path in &mut crate_paths {
            path.push('/');
        }

        // Add `oxc_*` dependency crates of `ast_tools` to the list.
        let cargo_toml = parse_toml("tasks/ast_tools/Cargo.toml", root_path);
        crate_paths.extend(
            cargo_toml
                .get("dependencies")
                .and_then(|v| v.as_table())
                .unwrap()
                .keys()
                .filter(|krate| {
                    // Exclude crates which are not in this monorepo e.g. `oxc_index`
                    krate.starts_with("oxc_")
                        && root_path.join("crates").join(krate.as_str()).is_dir()
                })
                .map(|krate| format!("crates/{krate}/")),
        );
        crate_paths.sort_unstable();
        crate_paths.dedup();

        // Convert crate paths to glob patterns
        let crate_globs = crate_paths.iter().map(|p| format!("{p}src/**/*.rs")).collect_vec();

        // Collect output paths, excluding `.rs` files already covered by crate globs
        #[expect(clippy::case_sensitive_file_extension_comparisons)]
        let output_paths = outputs.iter().map(|output| output.path.as_str()).filter(|path| {
            !path.ends_with(".rs")
                || !crate_paths
                    .iter()
                    .find_map(|crate_path| path.strip_prefix(crate_path.as_str()))
                    .is_some_and(|rest| rest.starts_with("src/"))
        });

        // Combine all paths
        let mut paths = chain!(
            [
                AST_CHANGES_WATCH_LIST_PATH,
                "tasks/ast_tools/src/**",
                ".github/workflows/ci.yml",
                "package.json",
                "oxfmtrc.jsonc",
                "Cargo.toml",
                "Cargo.lock",
            ],
            crate_globs.iter().map(String::as_str),
            output_paths,
        )
        .collect_vec();

        paths.sort_unstable();

        // Generate YAML
        let mut code = "src:\n".to_string();
        for path in paths {
            writeln!(code, "  - '{path}'").unwrap();
        }

        let output = Self::Yaml { path: AST_CHANGES_WATCH_LIST_PATH.to_string(), code };
        output.into_raw(file!())
    }
}

/// Parse TOML file.
fn parse_toml(path: &str, root_path: &Path) -> toml::Table {
    let toml_content = fs::read_to_string(root_path.join(path)).unwrap();
    toml::from_str(&toml_content).unwrap()
}
