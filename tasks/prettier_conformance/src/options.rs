use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct TestRunnerOptions {
    pub language: TestLanguage,
    pub debug: bool,
    pub filter: Option<String>,
}

#[derive(Default, Clone, Copy, Eq, PartialEq)]
pub enum TestLanguage {
    #[default]
    Js,
    Ts,
    Json,
    Jsonc,
}

impl TestLanguage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Js => "js",
            Self::Ts => "ts",
            Self::Json => "json",
            Self::Jsonc => "jsonc",
        }
    }

    /// Prettier's test fixtures roots for different languages.
    pub fn fixtures_roots(self, base: &Path) -> Vec<PathBuf> {
        match self {
            Self::Js => ["js", "jsx"].iter().map(|dir| base.join(dir)).collect::<Vec<_>>(),
            // There is no `tsx` directory, just check it works with TS
            // `SourceType`.`variant` is handled by spec file extension
            Self::Ts => ["typescript", "jsx"].iter().map(|dir| base.join(dir)).collect::<Vec<_>>(),
            // The JSON formatter targets the `json` parser.
            // `with-comment/` is shared with `Jsonc`: each call lists its own parser,
            // so `spec.rs` keeps only the `json` ones here.
            // Out-of-scope (TODO) siblings:
            // - `json5-as-json-with-trailing-commas/`: `json5` parser
            // - `json-superset/`: inline `snippets` shape, not parseable by `spec.rs`
            // - `range/`: range-formatting tests, not a whole-file format
            Self::Json => {
                vec![base.join("json").join("json"), base.join("json").join("with-comment")]
            }
            // The `jsonc` parser. `with-comment/` is shared with `Json` (see above);
            // `spec.rs` keeps only the `jsonc` calls here.
            Self::Jsonc => {
                vec![base.join("json").join("jsonc"), base.join("json").join("with-comment")]
            }
        }
    }
}
