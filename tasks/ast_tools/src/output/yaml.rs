use std::{fmt::Write, fs};

use rustc_hash::FxHashSet;

use crate::Codegen;

use super::{Output, add_header};

/// Add header to YAML.
pub fn print_yaml(code: &str, generator_path: &str) -> String {
    add_header(code, generator_path, "#")
}

impl Output {
    /// Generate a watch list YAML file.
    ///
    /// The following are added to the list:
    /// * The watch list file itself
    /// * `ast_tools` crate
    /// * `oxc_*` dependencies of `ast_tools`
    /// * CI workflow file
    pub fn yaml_watch_list<'s>(
        watch_list_path: &'s str,
        paths: impl IntoIterator<Item = &'s str>,
        codegen: &Codegen,
    ) -> Self {
        let mut paths = paths.into_iter().collect::<Vec<_>>();

        // Get `oxc_*` dependencies of `ast_tools`.
        // `ast_tools` uses these crates, so generated code may change if these crates are changed.
        let cargo_toml = parse_toml("tasks/ast_tools/Cargo.toml", codegen);
        let dependency_crates = cargo_toml
            .get("dependencies")
            .and_then(|v| v.as_table())
            .unwrap()
            .keys()
            .map(String::as_str)
            .filter(|krate| {
                // Exclude crates which are not in this monorepo e.g. `oxc_index`
                krate.starts_with("oxc_") && codegen.root_path().join("crates").join(krate).is_dir()
            })
            .collect::<FxHashSet<_>>();

        // Remove paths from `paths` which are in `src` dir of a dependency crate
        // (as `crate/{krate}/src/**` pattern will cover them anyway)
        paths.retain(|path| {
            if let Some(path) = path.strip_prefix("crates/")
                && let Some((krate, path)) = path.split_once('/')
                && path.starts_with("src/")
            {
                return !dependency_crates.contains(krate);
            }
            true
        });

        // Get paths for dependency crates
        let dependency_crate_paths = dependency_crates
            .into_iter()
            .map(|krate| format!("crates/{krate}/src/**"))
            .collect::<Vec<_>>();

        // Additional paths
        let additional_paths = [
            // This watch list file
            watch_list_path,
            // All code in `ast_tools`
            "tasks/ast_tools/src/**",
            // Workflow which runs `ast_tools`
            ".github/workflows/ci.yml",
            // Config files which affect `ast_tools`
            "package.json",
            "oxfmtrc.jsonc",
        ];

        // Add additional paths and dependency crate paths to `paths`, and sort
        paths.extend(
            additional_paths.into_iter().chain(dependency_crate_paths.iter().map(String::as_str)),
        );
        paths.sort_unstable();

        // Generate YAML
        let mut code = "src:\n".to_string();
        for path in paths {
            writeln!(code, "  - '{path}'").unwrap();
        }

        Self::Yaml { path: watch_list_path.to_string(), code }
    }
}

/// Parse TOML file.
fn parse_toml(path: &str, codegen: &Codegen) -> toml::Table {
    let toml_content = fs::read_to_string(codegen.root_path().join(path)).unwrap();
    toml::from_str(&toml_content).unwrap()
}
