use itertools::Itertools;

/// A Rust source file.
#[derive(Debug)]
pub struct File {
    /// Crate file is in e.g. `oxc_ast`
    pub krate: String,
    /// `true` if file is in a NAPI package, rather than a crate
    #[allow(dead_code, clippy::allow_attributes)]
    pub is_napi: bool,
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
        let (krate, is_napi) = match parts.next().unwrap() {
            "crates" => (parts.next().unwrap().to_string(), false),
            "napi" => (format!("napi/{}", parts.next().unwrap()), true),
            _ => panic!("Expected path beginning with `crates/` or `napi/`: `{path}`"),
        };
        assert_eq!(parts.next(), Some("src"));

        let mut import_path = format!("::{}", parts.join("::"));
        if import_path == "::lib" {
            import_path.clear();
        }

        Self { krate, is_napi, import_path }
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
            ("crates/oxc_ast/src/ast/js.rs", "oxc_ast", "::ast::js", false),
            ("crates/oxc_span/src/source_type/mod.rs", "oxc_span", "::source_type", false),
            ("crates/oxc_syntax/src/lib.rs", "oxc_syntax", "", false),
            ("napi/parser/src/blah.rs", "napi/parser", "::blah", true),
            ("napi/parser/src/lib.rs", "napi/parser", "", true),
        ];

        for (file_path, krate, import_path, is_napi) in cases {
            let file = File::new(file_path);
            assert_eq!(file.krate(), krate);
            assert_eq!(file.import_path(), import_path);
            assert_eq!(file.is_napi, is_napi);
        }
    }
}
