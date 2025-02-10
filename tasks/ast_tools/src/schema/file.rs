use itertools::Itertools;

/// A Rust source file.
#[derive(Debug)]
pub struct File {
    /// Crate file is in e.g. `oxc_ast`
    pub krate: String,
    /// Import path excluding crate e.g. `::ast::js`
    pub import_path: String,
}

impl File {
    /// Create new [`File`] from a source path.
    pub fn new(file_path: &str) -> Self {
        // Convert file path to crate and import path.
        // `crates/oxc_ast/src/ast/js.rs` -> `oxc_ast`, `::ast::js`.
        // `crates/oxc_span/src/source_type/mod.rs` -> `oxc_span`, `::source_type`.
        // `crates/oxc_syntax/src/lib.rs` -> `oxc_syntax`, ``.
        let path = file_path.trim_end_matches(".rs").trim_end_matches("/mod");

        let mut parts = path.split('/');
        assert_eq!(parts.next(), Some("crates"));
        let krate = parts.next().unwrap().to_string();
        assert_eq!(parts.next(), Some("src"));

        let mut import_path = format!("::{}", parts.join("::"));
        if import_path == "::lib" {
            import_path = String::new();
        }

        Self { krate, import_path }
    }

    /// Get name of crate this [`File`] is in.
    pub fn krate(&self) -> &str {
        &self.krate
    }

    /// Get import path for this [`File`].
    pub fn import_path(&self) -> &str {
        &self.import_path
    }
}

#[cfg(test)]
mod test {
    use super::File;

    #[test]
    fn test_file_new() {
        let cases = [
            ("crates/oxc_ast/src/ast/js.rs", "oxc_ast", "::ast::js"),
            ("crates/oxc_span/src/source_type/mod.rs", "oxc_span", "::source_type"),
            ("crates/oxc_syntax/src/lib.rs", "oxc_syntax", ""),
        ];

        for (file_path, krate, import_path) in cases {
            let file = File::new(file_path);
            assert_eq!(file.krate(), krate);
            assert_eq!(file.import_path(), import_path);
        }
    }
}
