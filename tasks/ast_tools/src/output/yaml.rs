use std::fmt::Write;

use super::{Output, add_header};

/// Add header to YAML.
pub fn print_yaml(code: &str, generator_path: &str) -> String {
    add_header(code, generator_path, "#")
}

impl Output {
    /// Generate a watch list YAML file.
    ///
    /// The path of the watch list itself and `tasks/ast_tools/src/**` are added to the list.
    pub fn yaml_watch_list<'s>(
        watch_list_path: &'s str,
        paths: impl IntoIterator<Item = &'s str>,
    ) -> Self {
        let mut paths = paths
            .into_iter()
            .chain([watch_list_path, "tasks/ast_tools/src/**"])
            .collect::<Vec<_>>();
        paths.sort_unstable();

        let mut code = "src:\n".to_string();
        for path in paths {
            writeln!(code, "  - '{path}'").unwrap();
        }

        Self::Yaml { path: watch_list_path.to_string(), code }
    }
}
